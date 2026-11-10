// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! netskip-adblock: ad and tracker blocking for Day apps.
//!
//! The crate keeps a catalog of filter lists ([`catalog`]), turns a blocking [`Level`] into a
//! set of lists, downloads them into the app's cache directory and keeps them current, compiles
//! them off the UI thread, and answers two questions from any thread: should this request be
//! blocked ([`AdBlock::decide`]), and which elements should this page hide
//! ([`AdBlock::cosmetic`]). It has no UI and no web view code. A web view host connects it to
//! its engine through a request callback, the WebKit content rule export
//! ([`AdBlock::content_rules`]), and the document-start script ([`cosmetic_script`]).
//!
//! ```no_run
//! use netskip_adblock::{AdBlock, Level, RequestInfo, ResourceKind};
//!
//! let adblock = AdBlock::builder().build()?;
//! adblock.set_level(Level::Strong);
//! adblock.start(); // load the cached engine, then download what is due, in the background
//!
//! // Later, from a web view's request callback on any thread:
//! let decision = adblock.decide(&RequestInfo::new(
//!     "https://ads.example/banner.js",
//!     "https://news.example/",
//!     ResourceKind::Script,
//! ));
//! if decision.block {
//!     // answer the request with an empty 403
//! }
//! # Ok::<(), netskip_adblock::Error>(())
//! ```
//!
//! No list is compiled into the binary. Until the first download finishes, [`AdBlock::status`]
//! is [`Status::WaitingForLists`] and nothing is blocked.

use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, Weak};
use std::task::{Context, Wake, Waker};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use day_part_http::Client;
use serde::Deserialize;

mod catalog;
mod compile;
mod update;

// The README's and the guide's examples compile as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
#[cfg(doctest)]
#[doc = include_str!("../docs/adblock.md")]
struct GuideDoctests;

pub use catalog::{Level, Settings};
pub use catalog::{
    ListFormat, Purpose, SiteLevels, Source, SourceId, builtin_source, catalog, regional_sources,
};
pub use compile::{ContentRuleList, ContentRules, WEBKIT_RULE_LIMIT};
pub use update::{
    Clock, DEFAULT_REFRESH, FIRST_RETRY, ListState, MAX_REFRESH, MAX_RETRY, MIN_REFRESH,
    RuleCounts, SystemClock, UpdateOutcome, parse_expires, refresh_interval, retry_delay,
};

use compile::{Compiled, Input};
use update::{Fetched, list_path, unix};

/// The largest list download accepted. EasyList is about 2 MB.
pub const DEFAULT_MAX_LIST_BYTES: usize = 16 * 1024 * 1024;

// --- Errors ---

/// Why an operation failed.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The platform reports no cache directory and none was configured.
    NoCacheDirectory,
    /// A file in the cache directory could not be read or written.
    Io(String),
    /// The compiler or the WebKit export failed.
    Engine(String),
    /// The background job was cancelled or could not start.
    Cancelled,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoCacheDirectory => f.write_str("no cache directory is available"),
            Error::Io(m) => write!(f, "cache file error: {m}"),
            Error::Engine(m) => write!(f, "filter compiler error: {m}"),
            Error::Cancelled => f.write_str("the background job did not run"),
        }
    }
}

impl std::error::Error for Error {}

// --- Requests ---

/// What a request loads, as the web view reports it. Engines that do not report a type
/// (Android's `shouldInterceptRequest`) pass [`ResourceKind::Unknown`], and the kind is
/// inferred from the URL's extension.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    /// A page loaded into a frame; with `main_frame` set, a navigation of the whole tab.
    Document,
    /// A page loaded into an iframe.
    Subdocument,
    /// JavaScript.
    Script,
    /// CSS.
    Stylesheet,
    /// An image.
    Image,
    /// A web font.
    Font,
    /// Audio or video.
    Media,
    /// `XMLHttpRequest`.
    Xhr,
    /// `fetch()`.
    Fetch,
    /// A WebSocket connection.
    WebSocket,
    /// A beacon or `<a ping>`.
    Ping,
    /// A plug-in object.
    Object,
    /// Anything else the engine names.
    Other,
    /// The engine does not say.
    #[default]
    Unknown,
}

impl ResourceKind {
    /// The kind a URL's extension suggests, for engines that report none. A URL without a
    /// known extension is [`ResourceKind::Other`].
    pub fn from_url(url: &str) -> ResourceKind {
        let path = url.split(['?', '#']).next().unwrap_or_default();
        let path = path
            .split_once("://")
            .map(|(_, rest)| rest.split_once('/').map(|(_, p)| p).unwrap_or(""))
            .unwrap_or(path);
        let last = path.rsplit('/').next().unwrap_or_default();
        let Some((_, ext)) = last.rsplit_once('.') else {
            return ResourceKind::Other;
        };
        match ext.to_ascii_lowercase().as_str() {
            "js" | "mjs" | "cjs" => ResourceKind::Script,
            "css" => ResourceKind::Stylesheet,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "svg" | "ico" | "bmp" => {
                ResourceKind::Image
            }
            "woff" | "woff2" | "ttf" | "otf" | "eot" => ResourceKind::Font,
            "mp4" | "webm" | "mp3" | "m4a" | "m4v" | "ogg" | "oga" | "wav" | "m3u8" | "mpd" => {
                ResourceKind::Media
            }
            "html" | "htm" => ResourceKind::Subdocument,
            "json" | "xml" => ResourceKind::Xhr,
            _ => ResourceKind::Other,
        }
    }
}

/// One request to judge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestInfo<'a> {
    /// The URL being requested.
    pub url: &'a str,
    /// The page that makes the request: the top-level page, or the frame's address where the
    /// engine reports one. Per-site levels and the statistics use its host.
    pub source_url: &'a str,
    /// What is being loaded.
    pub kind: ResourceKind,
    /// Whether this is the top-level frame's own document (a navigation).
    pub main_frame: bool,
}

impl<'a> RequestInfo<'a> {
    /// A subresource request from the page at `source_url`.
    pub fn new(url: &'a str, source_url: &'a str, kind: ResourceKind) -> RequestInfo<'a> {
        RequestInfo {
            url,
            source_url,
            kind,
            main_frame: false,
        }
    }

    /// A top-level navigation to `url`.
    pub fn navigation(url: &'a str) -> RequestInfo<'a> {
        RequestInfo {
            url,
            source_url: url,
            kind: ResourceKind::Document,
            main_frame: true,
        }
    }

    /// The kind, with [`ResourceKind::Unknown`] resolved from the URL.
    pub fn resolved_kind(&self) -> ResourceKind {
        match self.kind {
            ResourceKind::Unknown if self.main_frame => ResourceKind::Document,
            ResourceKind::Unknown => match ResourceKind::from_url(self.url) {
                ResourceKind::Subdocument if self.main_frame => ResourceKind::Document,
                kind => kind,
            },
            kind => kind,
        }
    }

    /// The host whose level applies: the source's, or the URL's for a navigation.
    pub fn page_host(&self) -> &'a str {
        if self.main_frame || self.source_url.is_empty() {
            url_host(self.url)
        } else {
            url_host(self.source_url)
        }
    }
}

/// The host of a URL: between `://` and the path, without user info or port.
pub fn url_host(url: &str) -> &str {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit('@').next().unwrap_or_default();
    if let Some(v6) = host.strip_prefix('[') {
        return v6.split(']').next().unwrap_or_default();
    }
    host.split(':').next().unwrap_or_default()
}

/// The answer for one request.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Decision {
    /// Block the request. A host answers with an empty response (Android: 403 with no body) or
    /// cancels it; for a navigation it shows an interstitial page.
    pub block: bool,
    /// The list whose rule blocked the request.
    pub matched_list: Option<SourceId>,
    /// A replacement body for `$redirect` rules. The crate loads no redirect resources, so this
    /// is always `None` for now; hosts that support it should treat `Some` as the response.
    pub redirect: Option<String>,
}

/// The element hiding rules for one page.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CosmeticResources {
    /// CSS selectors to hide with `display: none !important`: the page's site-specific rules
    /// and the generic rules that are not plain class or id selectors.
    pub hide_selectors: Vec<String>,
    /// Selectors the page excepts. Pass them to [`AdBlock::generic_selectors`].
    pub exceptions: Vec<String>,
    /// The page turns generic hiding off (`$generichide`): do not ask for generic selectors.
    pub generichide: bool,
}

// --- Status and reports ---

/// What the blocker can do for the global level right now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// The global level is [`Level::Off`].
    Off,
    /// None of the level's lists has been downloaded yet; nothing is blocked.
    WaitingForLists,
    /// The lists are stored and the engine is being loaded or compiled.
    Loading,
    /// Blocking with `lists` lists. `missing` names lists of the level that are not
    /// downloaded yet.
    Ready {
        /// How many lists the engine holds.
        lists: usize,
        /// Lists of the level without a stored copy.
        missing: Vec<SourceId>,
    },
    /// The last compile failed and no engine is loaded.
    Failed {
        /// What went wrong. Not localized.
        error: String,
    },
}

/// What one update run did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpdateReport {
    /// Each active list and how its download ended.
    pub results: Vec<(SourceId, UpdateOutcome)>,
    /// Whether an engine was rebuilt or loaded afterwards.
    pub recompiled: bool,
    /// Another update was running, so this one did nothing.
    pub already_running: bool,
}

/// What [`AdBlock::stage`] did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StageReport {
    /// Each list that had no stored copy, and how its download ended.
    pub results: Vec<(SourceId, UpdateOutcome)>,
    /// Lists the staged settings use that still have no stored copy: their levels block
    /// without them.
    pub missing: Vec<SourceId>,
    /// A compile error, when an engine could not be built.
    pub error: Option<String>,
}

impl StageReport {
    /// Whether every list downloaded and every engine compiled.
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.error.is_none()
    }
}

/// A list checked by [`AdBlock::check_list`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListCheck {
    /// The list's definition, ready for [`Settings::custom_sources`].
    pub source: Source,
    /// Its rules, including the ones this engine cannot apply.
    pub rules: RuleCounts,
    /// Its size in bytes.
    pub bytes: u64,
}

/// One list as the app's settings screen shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListInfo {
    /// The list's definition.
    pub source: Source,
    /// What the updater knows about it.
    pub state: ListState,
    /// Whether a stored copy exists.
    pub downloaded: bool,
}

/// Blocked requests, in total and per purpose.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockedCounts {
    /// Every blocked request.
    pub total: u64,
    by_purpose: [u64; 7],
}

impl BlockedCounts {
    /// Blocked requests whose list has this purpose.
    pub fn get(&self, purpose: Purpose) -> u64 {
        self.by_purpose[purpose.index()]
    }

    fn add(&mut self, purpose: Purpose) {
        self.total += 1;
        self.by_purpose[purpose.index()] += 1;
    }
}

// --- The cosmetic script ---

/// The message channel name the cosmetic script posts to. A host registers it as a WebKit
/// script message handler, an Android web message listener's object name, or reads it from
/// WebView2's `WebMessageReceived`.
pub const SCRIPT_CHANNEL: &str = "dayAdblock";

/// The element hiding script: inject it at document start into every frame. It posts JSON
/// strings on [`SCRIPT_CHANNEL`]; pass each one to [`AdBlock::handle_script_message`] and
/// evaluate the JavaScript it returns in the frame that sent it.
pub fn cosmetic_script() -> &'static str {
    include_str!("cosmetic.js")
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum ScriptMessage {
    Page {
        url: String,
    },
    Ids {
        url: String,
        #[serde(default)]
        classes: Vec<String>,
        #[serde(default)]
        ids: Vec<String>,
    },
}

/// The most classes or ids one script message may carry; the script sends at most 500.
const MAX_SCRIPT_NAMES: usize = 2_000;

// --- The handle ---

/// Configures an [`AdBlock`].
#[must_use]
pub struct Builder {
    cache_dir: Option<PathBuf>,
    client: Option<Client>,
    clock: Arc<dyn Clock>,
    settings: Settings,
    max_list_bytes: usize,
    webkit_rule_limit: usize,
}

impl Builder {
    /// The directory for lists, `state.json`, compiled engines and WebKit rules. Default:
    /// `adblock/` in the app's cache directory (`day_spec::storage::cache_dir()`).
    pub fn cache_dir(mut self, dir: impl Into<PathBuf>) -> Builder {
        self.cache_dir = Some(dir.into());
        self
    }

    /// The HTTP client for list downloads. Default: `Client::new()`, the platform's stack.
    /// Tests pass a client whose session carries a `day_part_http::simulation::Simulation`.
    pub fn client(mut self, client: Client) -> Builder {
        self.client = Some(client);
        self
    }

    /// The clock for the update schedule. Default: [`SystemClock`].
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Builder {
        self.clock = clock;
        self
    }

    /// The initial settings, usually the ones the app stored last time.
    pub fn settings(mut self, settings: Settings) -> Builder {
        self.settings = settings;
        self
    }

    /// The largest list download accepted. Default: [`DEFAULT_MAX_LIST_BYTES`].
    pub fn max_list_bytes(mut self, bytes: usize) -> Builder {
        self.max_list_bytes = bytes.max(1);
        self
    }

    /// The most rules in one exported WebKit list. Default: [`WEBKIT_RULE_LIMIT`].
    pub fn webkit_rule_limit(mut self, rules: usize) -> Builder {
        self.webkit_rule_limit = rules.max(1);
        self
    }

    /// The blocker. Reads `state.json` from the cache directory; compiles nothing yet.
    pub fn build(self) -> Result<AdBlock, Error> {
        let dir = match self.cache_dir {
            Some(dir) => dir,
            None => day_spec::storage::cache_dir()
                .ok_or(Error::NoCacheDirectory)?
                .join("adblock"),
        };
        let lists = update::read_state(&dir);
        Ok(AdBlock(Arc::new(Inner {
            client: self.client.unwrap_or_default(),
            clock: self.clock,
            max_list_bytes: self.max_list_bytes,
            webkit_rule_limit: self.webkit_rule_limit,
            settings: RwLock::new(self.settings),
            lists: Mutex::new(lists),
            compiled: RwLock::new(HashMap::new()),
            staged: Mutex::new(Vec::new()),
            failure: Mutex::new(None),
            building: AtomicUsize::new(0),
            updating: AtomicBool::new(false),
            started: AtomicBool::new(false),
            timer: Mutex::new(None),
            listeners: Mutex::new(Vec::new()),
            stats: Mutex::new(HashMap::new()),
            dir,
        })))
    }
}

type Listener = Arc<dyn Fn() + Send + Sync>;

struct Inner {
    dir: PathBuf,
    client: Client,
    clock: Arc<dyn Clock>,
    max_list_bytes: usize,
    webkit_rule_limit: usize,
    settings: RwLock<Settings>,
    lists: Mutex<BTreeMap<SourceId, ListState>>,
    compiled: RwLock<HashMap<Level, Arc<Compiled>>>,
    /// Levels compiled by [`AdBlock::stage`] for settings not applied yet: kept through the
    /// background cycle's pruning until [`AdBlock::set_settings`] applies them.
    staged: Mutex<Vec<Level>>,
    /// The global level's last compile error, cleared by the next success.
    failure: Mutex<Option<String>>,
    building: AtomicUsize,
    updating: AtomicBool,
    started: AtomicBool,
    timer: Mutex<Option<day_async::TimerId>>,
    listeners: Mutex<Vec<Listener>>,
    stats: Mutex<HashMap<String, BlockedCounts>>,
}

/// Locks here guard plain data; a panic while one is held leaves nothing half-written that a
/// later reader could misread, so a poisoned lock is still used.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

fn read<T>(l: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    l.read().unwrap_or_else(|p| p.into_inner())
}

fn write<T>(l: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    l.write().unwrap_or_else(|p| p.into_inner())
}

/// The blocker: a cheap, cloneable, `Send + Sync` handle. Every clone shares one state, so a
/// web view's background request callback can hold one while the UI holds another.
#[derive(Clone)]
pub struct AdBlock(Arc<Inner>);

const _: () = {
    const fn shared<T: Send + Sync>() {}
    shared::<AdBlock>();
};

impl AdBlock {
    /// Configure a blocker.
    pub fn builder() -> Builder {
        Builder {
            cache_dir: None,
            client: None,
            clock: Arc::new(SystemClock),
            settings: Settings::default(),
            max_list_bytes: DEFAULT_MAX_LIST_BYTES,
            webkit_rule_limit: WEBKIT_RULE_LIMIT,
        }
    }

    /// The cache directory.
    pub fn cache_dir(&self) -> &Path {
        &self.0.dir
    }

    // --- Settings ---

    /// The current settings, for the app to persist.
    pub fn settings(&self) -> Settings {
        read(&self.0.settings).clone()
    }

    /// Replace the settings. A started blocker loads or downloads what the new settings need
    /// in the background.
    pub fn set_settings(&self, settings: Settings) {
        *write(&self.0.settings) = settings;
        lock(&self.0.staged).clear();
        self.settings_changed();
    }

    /// Get `settings` ready without applying them: download every list they use that has no
    /// stored copy, and compile the engine of each level they use, while the current settings
    /// keep blocking. Then [`AdBlock::set_settings`] applies them with nothing left to wait
    /// for, and pages stay protected throughout. Lists that already have a copy are not
    /// downloaded again (the regular schedule keeps them current). Resolves with what each
    /// download did and which lists are still missing.
    pub async fn stage(&self, settings: Settings) -> StageReport {
        self.0.clone().stage(settings).await
    }

    /// Download the list at `url` and check it before it is used: an `https` address that
    /// answers with a filter list, not a web page or an empty file. The copy is stored, so
    /// adding the list afterwards (in [`Settings::custom_sources`]) needs no second download.
    /// [`RuleCounts::unsupported`] counts the rules this engine cannot apply.
    pub async fn check_list(&self, url: &str) -> Result<ListCheck, String> {
        self.0.clone().check_list(url).await
    }

    /// The global level.
    pub fn level(&self) -> Level {
        read(&self.0.settings).level.clone()
    }

    /// Change the global level.
    pub fn set_level(&self, level: Level) {
        write(&self.0.settings).level = level;
        self.settings_changed();
    }

    /// Set a site's own level, or remove it with `None`.
    pub fn set_site_level(&self, host: &str, level: Option<Level>) {
        write(&self.0.settings).sites.set(host, level);
        self.settings_changed();
    }

    /// The level that applies to pages on `host`.
    pub fn level_for(&self, host: &str) -> Level {
        read(&self.0.settings).level_for(host).clone()
    }

    fn settings_changed(&self) {
        if self.0.started.load(Ordering::Acquire) {
            cycle(self.0.clone());
        }
        self.0.notify();
    }

    // --- Lifecycle ---

    /// Load or compile the engines in the background, download the lists that are due, and
    /// keep doing so on the lists' schedules until [`AdBlock::stop`]. Call it once at launch.
    pub fn start(&self) {
        if !self.0.started.swap(true, Ordering::AcqRel) {
            cycle(self.0.clone());
        }
    }

    /// Stop the update schedule. Work already running finishes.
    pub fn stop(&self) {
        self.0.started.store(false, Ordering::Release);
        if let Some(id) = lock(&self.0.timer).take() {
            day_async::unschedule(id);
        }
    }

    /// Call `listener` after any change to the status, the lists, or the settings. It runs on
    /// whichever thread made the change; capture a reactive setter to bring it to the UI.
    pub fn on_change(&self, listener: impl Fn() + Send + Sync + 'static) {
        lock(&self.0.listeners).push(Arc::new(listener));
    }

    /// The status for the global level.
    pub fn status(&self) -> Status {
        let level = self.level();
        if level.is_off() {
            return Status::Off;
        }
        if let Some(c) = read(&self.0.compiled).get(&level) {
            return Status::Ready {
                lists: c.list_count(),
                missing: c.missing.clone(),
            };
        }
        if self.0.building.load(Ordering::Acquire) > 0 {
            return Status::Loading;
        }
        if let Some(error) = lock(&self.0.failure).clone() {
            return Status::Failed { error };
        }
        if self.0.inputs(&level).0.is_empty() {
            Status::WaitingForLists
        } else {
            Status::Loading
        }
    }

    /// Load or compile the engine of every level in use (the global one and each site's).
    /// An engine whose lists have not changed is loaded from its cache file. Resolves when
    /// every engine is ready; the work runs on a pool thread.
    pub async fn prepare(&self) -> Result<(), Error> {
        self.0.clone().prepare().await
    }

    /// Download every list in use that is due, then recompile what changed.
    pub async fn update_due(&self) -> UpdateReport {
        self.0.clone().update(false).await
    }

    /// Download every list in use now, whatever its schedule (still conditional, so an
    /// unchanged list costs a `304`), then recompile what changed.
    pub async fn update_now(&self) -> UpdateReport {
        self.0.clone().update(true).await
    }

    /// When the next list is due, or `None` when no list is in use.
    pub fn next_update(&self) -> Option<SystemTime> {
        self.0
            .next_update()
            .map(|secs| UNIX_EPOCH + Duration::from_secs(secs))
    }

    /// Every list in use and every list with a stored copy, with the updater's state.
    pub fn lists(&self) -> Vec<ListInfo> {
        let settings = self.settings();
        let lists = lock(&self.0.lists).clone();
        let mut ids: Vec<SourceId> = Vec::new();
        for level in settings.active_levels() {
            for id in settings.sources_for(&level) {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        for id in lists.keys() {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
        ids.into_iter()
            .filter_map(|id| {
                let source = settings.source(&id)?;
                Some(ListInfo {
                    downloaded: list_path(&self.0.dir, &id).is_file(),
                    state: lists.get(&id).cloned().unwrap_or_default(),
                    source,
                })
            })
            .collect()
    }

    // --- Queries ---

    fn compiled_for(&self, host: &str) -> Option<Arc<Compiled>> {
        let settings = read(&self.0.settings);
        let level = settings.level_for(host);
        if level.is_off() {
            return None;
        }
        read(&self.0.compiled).get(level).cloned()
    }

    /// Whether to block a request. Safe on any thread and fast (microseconds). A blocked
    /// request counts toward the page host's [`AdBlock::blocked`] statistics.
    pub fn decide(&self, request: &RequestInfo<'_>) -> Decision {
        let host = request.page_host();
        let Some(compiled) = self.compiled_for(host) else {
            return Decision::default();
        };
        let (decision, purpose) = compiled.decide(request);
        if let Some(purpose) = purpose {
            lock(&self.0.stats)
                .entry(catalog::normalize_host(host))
                .or_default()
                .add(purpose);
        }
        decision
    }

    /// The element hiding rules for the page at `url`.
    pub fn cosmetic(&self, url: &str) -> CosmeticResources {
        match self.compiled_for(url_host(url)) {
            Some(c) => c.cosmetic(url),
            None => CosmeticResources::default(),
        }
    }

    /// Generic hiding selectors that match the classes and ids seen on the page at
    /// `page_url`, less the page's `exceptions` (from [`AdBlock::cosmetic`]).
    pub fn generic_selectors(
        &self,
        page_url: &str,
        classes: &[String],
        ids: &[String],
        exceptions: &[String],
    ) -> Vec<String> {
        match self.compiled_for(url_host(page_url)) {
            Some(c) => c.generic_selectors(classes, ids, exceptions),
            None => Vec::new(),
        }
    }

    /// Answer a message the [`cosmetic_script`] posted. Returns JavaScript to evaluate in the
    /// frame that sent the message, or `None` when there is nothing to apply or the message is
    /// not one of the script's.
    pub fn handle_script_message(&self, message: &str) -> Option<String> {
        let message: ScriptMessage = serde_json::from_str(message).ok()?;
        let answer = match message {
            ScriptMessage::Page { url } => {
                let r = self.cosmetic(&url);
                if r.hide_selectors.is_empty() && !r.generichide {
                    return None;
                }
                serde_json::json!({ "hide": r.hide_selectors, "generichide": r.generichide })
            }
            ScriptMessage::Ids {
                url,
                mut classes,
                mut ids,
            } => {
                classes.truncate(MAX_SCRIPT_NAMES);
                ids.truncate(MAX_SCRIPT_NAMES);
                let r = self.cosmetic(&url);
                if r.generichide {
                    return None;
                }
                let hide = self.generic_selectors(&url, &classes, &ids, &r.exceptions);
                if hide.is_empty() {
                    return None;
                }
                serde_json::json!({ "hide": hide })
            }
        };
        // Data, never code: the host hands it back to the script as the reply to its message,
        // and the script applies it (`window.__dayAdblock.apply`).
        Some(answer.to_string())
    }

    /// The level's lists as WebKit content rule lists (`WKContentRuleListStore`,
    /// `WebKitUserContentFilterStore`), exported once per change and cached in the cache
    /// directory. Compare [`ContentRules::version`] with the version compiled last and
    /// recompile only when it differs. The work runs on a pool thread.
    pub async fn content_rules(&self, level: &Level) -> Result<ContentRules, Error> {
        let settings = self.settings();
        self.content_rules_for(&settings, level).await
    }

    /// [`AdBlock::content_rules`] for `level` under `settings`, which need not be applied yet:
    /// export a staged level's rules before applying it, so WebKit views switch from the old
    /// rules straight to the new ones.
    pub async fn content_rules_for(
        &self,
        settings: &Settings,
        level: &Level,
    ) -> Result<ContentRules, Error> {
        let (inputs, _) = self.0.inputs_in(settings, level);
        let dir = self.0.dir.clone();
        let level = level.clone();
        let limit = self.0.webkit_rule_limit;
        if level.is_off() || inputs.is_empty() {
            return Ok(ContentRules {
                version: String::from("empty"),
                lists: Vec::new(),
            });
        }
        day_async::spawn_blocking(move || compile::content_rules(&dir, &level, &inputs, limit))
            .await
            .map_err(|_| Error::Cancelled)?
    }

    // --- Statistics ---

    /// Requests blocked on pages of `host` since the last reset.
    pub fn blocked(&self, host: &str) -> BlockedCounts {
        lock(&self.0.stats)
            .get(&catalog::normalize_host(host))
            .copied()
            .unwrap_or_default()
    }

    /// Blocked requests per page host, sorted by host.
    pub fn blocked_by_host(&self) -> Vec<(String, BlockedCounts)> {
        let mut out: Vec<_> = lock(&self.0.stats)
            .iter()
            .map(|(h, c)| (h.clone(), *c))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Requests blocked on every page since the last reset.
    pub fn blocked_total(&self) -> BlockedCounts {
        let mut total = BlockedCounts::default();
        for c in lock(&self.0.stats).values() {
            total.total += c.total;
            for (t, n) in total.by_purpose.iter_mut().zip(c.by_purpose) {
                *t += n;
            }
        }
        total
    }

    /// Forget the counts for `host`, or for every host with `None`.
    pub fn reset_stats(&self, host: Option<&str>) {
        let mut stats = lock(&self.0.stats);
        match host {
            Some(host) => {
                stats.remove(&catalog::normalize_host(host));
            }
            None => stats.clear(),
        }
    }
}

impl Inner {
    fn now(&self) -> u64 {
        unix(self.clock.now())
    }

    fn notify(&self) {
        let listeners = lock(&self.listeners).clone();
        for listener in listeners {
            listener();
        }
    }

    /// The stored lists `level` uses, and the ones it lacks.
    fn inputs(&self, level: &Level) -> (Vec<Input>, Vec<SourceId>) {
        let settings = read(&self.settings).clone();
        self.inputs_in(&settings, level)
    }

    /// [`Inner::inputs`] under `settings`, applied or not.
    fn inputs_in(&self, settings: &Settings, level: &Level) -> (Vec<Input>, Vec<SourceId>) {
        let lists = lock(&self.lists);
        let mut inputs = Vec::new();
        let mut missing = Vec::new();
        for id in settings.sources_for(level) {
            let Some(source) = settings.source(&id) else {
                missing.push(id);
                continue;
            };
            let path = list_path(&self.dir, &id);
            match lists.get(&id).and_then(|s| s.content_hash.clone()) {
                Some(content_hash) if path.is_file() => inputs.push(Input {
                    id,
                    purpose: source.purpose,
                    format: source.format,
                    path,
                    content_hash,
                }),
                _ => missing.push(id),
            }
        }
        (inputs, missing)
    }

    fn active_sources(&self) -> Vec<Source> {
        let settings = read(&self.settings).clone();
        sources_of(&settings)
    }

    async fn prepare(self: Arc<Self>) -> Result<(), Error> {
        let settings = read(&self.settings).clone();
        let levels = settings.active_levels();
        {
            let staged = lock(&self.staged).clone();
            write(&self.compiled)
                .retain(|level, _| levels.contains(level) || staged.contains(level));
        }
        let mut result = Ok(());
        for level in levels {
            if let Err(e) = self.clone().compile_level(&settings, &level).await {
                result = Err(e);
            }
        }
        result
    }

    /// Compile `level`'s engine under `settings` unless the compiled one is current. A level
    /// with no stored list has no engine.
    async fn compile_level(
        self: Arc<Self>,
        settings: &Settings,
        level: &Level,
    ) -> Result<(), Error> {
        let global = read(&self.settings).level.clone();
        let (inputs, missing) = self.inputs_in(settings, level);
        if inputs.is_empty() {
            write(&self.compiled).remove(level);
            return Ok(());
        }
        let hash = compile::inputs_hash(level, &inputs);
        let current = read(&self.compiled).get(level).map(|c| c.hash);
        if current == Some(hash) {
            return Ok(());
        }
        self.building.fetch_add(1, Ordering::AcqRel);
        self.notify();
        let dir = self.dir.clone();
        let job_level = level.clone();
        let built = day_async::spawn_blocking(move || {
            compile::load_or_build(&dir, &job_level, &inputs, missing)
        })
        .await
        .map_err(|_| Error::Cancelled)
        .and_then(|r| r);
        self.building.fetch_sub(1, Ordering::AcqRel);
        let result = match built {
            Ok(compiled) => {
                write(&self.compiled).insert(level.clone(), Arc::new(compiled));
                if *level == global {
                    *lock(&self.failure) = None;
                }
                Ok(())
            }
            Err(e) => {
                if *level == global {
                    *lock(&self.failure) = Some(e.to_string());
                }
                Err(e)
            }
        };
        self.notify();
        result
    }

    async fn stage(self: Arc<Self>, settings: Settings) -> StageReport {
        let levels = settings.active_levels();
        lock(&self.staged).extend(levels.iter().cloned());
        let needed: Vec<Source> = sources_of(&settings)
            .into_iter()
            .filter(|s| !list_path(&self.dir, &s.id).is_file())
            .collect();
        let (results, _) = self.clone().fetch_sources(needed, true).await;
        let mut report = StageReport {
            results,
            ..StageReport::default()
        };
        for level in &levels {
            if let Err(e) = self.clone().compile_level(&settings, level).await {
                report.error = Some(e.to_string());
            }
            let (_, missing) = self.inputs_in(&settings, level);
            for id in missing {
                if !report.missing.contains(&id) {
                    report.missing.push(id);
                }
            }
        }
        report
    }

    async fn check_list(self: Arc<Self>, url: &str) -> Result<ListCheck, String> {
        let url = url.trim();
        let host = url
            .strip_prefix("https://")
            .map(|rest| rest.split(['/', '?', '#']).next().unwrap_or_default())
            .unwrap_or_default();
        if host.is_empty() || url.chars().any(char::is_whitespace) {
            return Err("a filter list address must start with https://".into());
        }
        let source = Source::custom(url);
        let (results, _) = self.clone().fetch_sources(vec![source.clone()], true).await;
        match results.into_iter().next().map(|(_, outcome)| outcome) {
            Some(UpdateOutcome::Updated | UpdateOutcome::NotModified) => {}
            Some(UpdateOutcome::Failed { error }) => return Err(error),
            _ => return Err("the list was not downloaded".into()),
        }
        let state = lock(&self.lists)
            .get(&source.id)
            .cloned()
            .unwrap_or_default();
        // The list's own name, from its `! Title:` header, where it has one.
        let path = list_path(&self.dir, &source.id);
        let title = day_async::spawn_blocking(move || {
            let text = std::fs::read_to_string(path).ok()?;
            update::list_title(&text)
        })
        .await
        .ok()
        .flatten();
        let source = match title {
            Some(title) => source.with_title(title),
            None => source,
        };
        Ok(ListCheck {
            source,
            rules: state.rules,
            bytes: state.bytes,
        })
    }

    /// Download `sources`, storing each new copy and the updater's state. `force` downloads
    /// even a list that is not due. Returns each list's outcome and whether any copy changed.
    async fn fetch_sources(
        self: Arc<Self>,
        sources: Vec<Source>,
        force: bool,
    ) -> (Vec<(SourceId, UpdateOutcome)>, bool) {
        let mut results = Vec::new();
        let mut changed = false;
        for source in sources {
            let now = self.now();
            let path = list_path(&self.dir, &source.id);
            let have_copy = path.is_file();
            let previous = lock(&self.lists)
                .get(&source.id)
                .cloned()
                .unwrap_or_default();
            if !force && have_copy && previous.next_fetch.is_some_and(|at| at > now) {
                results.push((source.id.clone(), UpdateOutcome::NotDue));
                continue;
            }
            let fetched = update::fetch(
                &self.client,
                &source,
                &previous,
                have_copy,
                now,
                self.max_list_bytes,
            )
            .await;
            let (state, text) = match fetched {
                Fetched::Updated { text, state } => (state, Some(text)),
                Fetched::NotModified { state } | Fetched::Failed { state } => (state, None),
            };
            let mut outcome = state.last_result.clone().unwrap_or(UpdateOutcome::NotDue);
            let mut state = state;
            if let Some(text) = text {
                let path = path.clone();
                let written =
                    day_async::spawn_blocking(move || update::write_atomic(&path, text.as_bytes()))
                        .await;
                match written {
                    Ok(Ok(())) => changed = true,
                    Ok(Err(e)) => {
                        state = previous.clone();
                        state.last_result = Some(UpdateOutcome::Failed {
                            error: format!("could not store the list: {e}"),
                        });
                        outcome = state.last_result.clone().unwrap_or(UpdateOutcome::NotDue);
                    }
                    Err(_) => {
                        state = previous.clone();
                        outcome = UpdateOutcome::Failed {
                            error: Error::Cancelled.to_string(),
                        };
                    }
                }
            }
            let snapshot = {
                let mut lists = lock(&self.lists);
                lists.insert(source.id.clone(), state);
                lists.clone()
            };
            let dir = self.dir.clone();
            // A lost state file costs one unconditional download per list, nothing more.
            let _ = day_async::spawn_blocking(move || update::write_state(&dir, &snapshot)).await;
            results.push((source.id.clone(), outcome));
            self.notify();
        }
        (results, changed)
    }
}

/// Every list `settings` use, built-in or custom, without duplicates.
fn sources_of(settings: &Settings) -> Vec<Source> {
    {
        let mut out: Vec<Source> = Vec::new();
        for level in settings.active_levels() {
            for id in settings.sources_for(&level) {
                if out.iter().any(|s| s.id == id) {
                    continue;
                }
                if let Some(source) = settings.source(&id) {
                    out.push(source);
                }
            }
        }
        out
    }
}

impl Inner {
    fn next_update(&self) -> Option<u64> {
        let sources = self.active_sources();
        let lists = lock(&self.lists);
        let now = self.now();
        sources
            .iter()
            .map(|s| {
                let have = list_path(&self.dir, &s.id).is_file();
                match lists.get(&s.id).and_then(|st| st.next_fetch) {
                    Some(at) if have => at,
                    _ => now,
                }
            })
            .min()
    }

    async fn update(self: Arc<Self>, force: bool) -> UpdateReport {
        if self.updating.swap(true, Ordering::AcqRel) {
            return UpdateReport {
                already_running: true,
                ..UpdateReport::default()
            };
        }
        struct Release<'a>(&'a AtomicBool);
        impl Drop for Release<'_> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Release);
            }
        }
        let _release = Release(&self.updating);

        let sources = self.active_sources();
        let (results, changed) = self.clone().fetch_sources(sources, force).await;
        let mut report = UpdateReport {
            results,
            ..UpdateReport::default()
        };
        drop(_release);

        let needs_engine = {
            let settings = read(&self.settings);
            let compiled = read(&self.compiled);
            settings
                .active_levels()
                .iter()
                .any(|l| !compiled.contains_key(l))
        };
        if changed || needs_engine {
            let before: Vec<u64> = read(&self.compiled).values().map(|c| c.hash).collect();
            let _ = self.clone().prepare().await;
            let after: Vec<u64> = read(&self.compiled).values().map(|c| c.hash).collect();
            report.recompiled = before != after;
        }
        report
    }
}

// --- Background work ---

/// One background cycle: engines, then due downloads, then the timer for the next one.
fn cycle(inner: Arc<Inner>) {
    spawn_detached(async move {
        let _ = inner.clone().prepare().await;
        let _ = inner.clone().update(false).await;
        if !inner.started.load(Ordering::Acquire) {
            return;
        }
        let now = inner.now();
        let delay = inner
            .next_update()
            .map(|at| at.saturating_sub(now))
            .unwrap_or(24 * 60 * 60)
            .clamp(60, 24 * 60 * 60);
        let weak: Weak<Inner> = Arc::downgrade(&inner);
        let id = day_async::schedule(Duration::from_secs(delay), move || {
            if let Some(inner) = weak.upgrade()
                && inner.started.load(Ordering::Acquire)
            {
                cycle(inner);
            }
        });
        if let Some(old) = lock(&inner.timer).replace(id) {
            day_async::unschedule(old);
        }
    });
}

/// Drive `future` to completion with no executor: each wake polls it on the waking thread
/// (the HTTP transport's, the blocking pool's, or the timer's). The futures here do short
/// work between awaits and hand file I/O and compiling to the blocking pool.
fn spawn_detached(future: impl Future<Output = ()> + Send + 'static) {
    const IDLE: u8 = 0;
    const POLLING: u8 = 1;
    const NOTIFIED: u8 = 2;
    const DONE: u8 = 3;

    type Boxed = Pin<Box<dyn Future<Output = ()> + Send>>;
    struct Task {
        state: AtomicU8,
        future: Mutex<Option<Boxed>>,
    }

    impl Wake for Task {
        fn wake(self: Arc<Self>) {
            self.wake_by_ref();
        }

        fn wake_by_ref(self: &Arc<Self>) {
            // Whoever moves the task from IDLE to POLLING polls it; a wake during a poll asks
            // that poller to go round once more.
            loop {
                match self.state.compare_exchange(
                    IDLE,
                    POLLING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => break,
                    Err(POLLING) => {
                        if self
                            .state
                            .compare_exchange(
                                POLLING,
                                NOTIFIED,
                                Ordering::AcqRel,
                                Ordering::Acquire,
                            )
                            .is_ok()
                        {
                            return;
                        }
                    }
                    Err(_) => return,
                }
            }
            let waker = Waker::from(self.clone());
            let mut cx = Context::from_waker(&waker);
            loop {
                let ready = {
                    let mut slot = lock(&self.future);
                    match slot.as_mut() {
                        Some(f) => f.as_mut().poll(&mut cx).is_ready(),
                        None => true,
                    }
                };
                if ready {
                    *lock(&self.future) = None;
                    self.state.store(DONE, Ordering::Release);
                    return;
                }
                if self
                    .state
                    .compare_exchange(POLLING, IDLE, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    return;
                }
                // Woken while polling: poll again.
                self.state.store(POLLING, Ordering::Release);
            }
        }
    }

    let task = Arc::new(Task {
        state: AtomicU8::new(IDLE),
        future: Mutex::new(Some(Box::pin(future))),
    });
    task.wake_by_ref();
}
