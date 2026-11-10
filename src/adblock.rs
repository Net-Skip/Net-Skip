// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Ad and tracker blocking, with Vivaldi's levels: Off, Light (the default), Strong and
//! Aggressive, chosen in Settings and per site from the page menu. The engine and the filter
//! lists are netskip-adblock's: the lists download from their publishers and stay current, and
//! nothing is compiled into the app.
//!
//! Settings' Filter Lists page shows each list a level uses and lets the user switch lists on
//! and off, add lists by address, and choose the two policies a list cannot express (blocking
//! whole pages, and requests to the page's own site). Any such change makes the level Custom,
//! so a preset always means its own lists. Every change, a preset's or a site's included, is
//! applied the same way: the new lists download and compile while the old ones keep blocking,
//! and the switch happens only once everything is ready.
//!
//! Each web view is connected the way its engine allows: Android, WebView2, Qt and ArkWeb ask
//! [`Filter`] about every request; WebKit (iOS, macOS, WebKitGTK) runs compiled content rule
//! lists, chosen per tab by the level for the site it shows. Element hiding is the crate's
//! document-start script everywhere, answered through a script message.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use day::prelude::*;
use day_piece_settings::prelude::Pref;
use day_piece_webview::{
    BlockedLoad, ContentRuleList, RequestDecision, RequestFilter, RequestInfo, ResourceKind,
    ScriptMessage, UserScript,
};
use netskip_adblock::{AdBlock, Level, Purpose, Settings, SiteLevels, Source, SourceId};
use netskip_model::address;
use serde::{Deserialize, Serialize};

use crate::{fixtures, res};

/// The level every site gets unless it has its own: `off`, `light`, `strong`, `aggressive`, or
/// `custom` for the selection in [`CUSTOM`].
pub static LEVEL: Pref<String> = Pref::new("netskip.blocking.level", || "light".to_string());
/// The sites with a level of their own, as netskip-adblock's `SiteLevels` JSON.
pub static SITE_LEVELS: Pref<String> = Pref::new("netskip.blocking.sites", String::new);
/// The Custom level's lists and policies, and the lists added by address, as [`CustomLists`]
/// JSON.
pub static CUSTOM: Pref<String> = Pref::new("netskip.blocking.custom", String::new);

/// What the Custom level uses, and the lists the user added by address (kept while a preset is
/// in use, so switching back to Custom finds them).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomLists {
    /// The lists switched on, built-in or added.
    pub sources: Vec<SourceId>,
    /// Block navigations to blocked pages.
    pub document_blocking: bool,
    /// Block requests to the page's own site.
    pub same_site_blocking: bool,
    /// The lists added by address.
    pub added: Vec<Source>,
}

impl CustomLists {
    /// The stored selection. A tracked read.
    pub fn get() -> CustomLists {
        serde_json::from_str(&CUSTOM.get()).unwrap_or_default()
    }

    fn get_untracked() -> CustomLists {
        serde_json::from_str(&CUSTOM.get_untracked()).unwrap_or_default()
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string(self) {
            CUSTOM.set(json);
        }
    }

    fn level(&self) -> Level {
        Level::Custom {
            sources: self.sources.clone(),
            document_blocking: self.document_blocking,
            same_site_blocking: self.same_site_blocking,
        }
    }
}

/// The global level the stored settings ask for. A tracked read.
pub fn global_level() -> Level {
    match LEVEL.get().as_str() {
        "custom" => CustomLists::get().level(),
        id => level_from_id(id),
    }
}

/// The settings the stored preferences ask for: what the engine runs once a change is applied.
fn desired_settings() -> Settings {
    untrack(|| Settings {
        level: global_level(),
        extra_sources: Vec::new(),
        custom_sources: CustomLists::get().added,
        sites: serde_json::from_str(&SITE_LEVELS.get()).unwrap_or_default(),
    })
}

static ENGINE: OnceLock<Option<AdBlock>> = OnceLock::new();

thread_local! {
    /// Bumped when the engine's lists or status change, so the rule lists and the settings'
    /// status line read it again.
    static VERSION: Signal<u64> = Signal::global(0);
    /// WebKit's rule lists per level id, as last exported and read from disk: the export is many
    /// megabytes, so it is built once per change, in the background.
    static RULES: RefCell<HashMap<Level, Vec<ContentRuleList>>> = RefCell::new(HashMap::new());
    /// The levels whose export is being built right now.
    static BUILDING: RefCell<HashSet<Level>> = RefCell::new(HashSet::new());
}

/// The level stored under `id`, `Light` for an unknown one.
pub fn level_from_id(id: &str) -> Level {
    match id {
        "off" => Level::Off,
        "strong" => Level::Strong,
        "aggressive" => Level::Aggressive,
        _ => Level::Light,
    }
}

/// The presets Settings and the page menu offer, in order.
pub const LEVELS: [&str; 4] = ["off", "light", "strong", "aggressive"];

/// A level's name, as Settings shows it.
pub fn level_label(id: &str) -> String {
    match id {
        "off" => res::str::blocking_off().format(),
        "strong" => res::str::blocking_strong().format(),
        "aggressive" => res::str::blocking_aggressive().format(),
        "custom" => res::str::blocking_custom().format(),
        _ => res::str::blocking_light().format(),
    }
}

/// Start the blocker: load the cached engine, then download whatever lists are due, in the
/// background. Call once at launch.
pub fn install() {
    // Fixture runs and debug builds name their level at launch, so a walkthrough or a test
    // against live sites needs no trip through Settings.
    if (fixtures::active() || cfg!(debug_assertions))
        && let Some(id) = day::env("NETSKIP_BLOCKING").filter(|id| LEVELS.contains(&id.as_str()))
    {
        LEVEL.set(id);
    }
    ENGINE.get_or_init(|| {
        let mut builder = AdBlock::builder();
        // Fixture runs keep their own lists: the real ones a normal launch downloaded must not
        // stand in for the fixture tree's, nor the other way round.
        if fixtures::active()
            && let Some(cache) = day::storage::cache_dir()
        {
            builder = builder.cache_dir(cache.join("adblock-fixtures"));
        }
        let adblock = match builder.build() {
            Ok(a) => a,
            Err(e) => {
                warn!("content blocking unavailable: {e}");
                return None;
            }
        };
        // The stored settings apply at once: their engines load from the cache directory.
        adblock.set_settings(desired_settings());
        // The engine reports from its own threads; the UI hears about it on the main one.
        adblock.on_change(|| {
            day::reactive::on_main(|| {
                VERSION.with(|v| v.update(|n| *n += 1));
                info!("content blocking: {:?}", engine().map(AdBlock::status));
            });
        });
        adblock.start();
        Some(adblock)
    });
    // Any change to the stored settings (a preset, a list, an option, a site's level) applies
    // once its lists are ready.
    watch(
        || (LEVEL.get(), CUSTOM.get(), SITE_LEVELS.get()),
        |_, _| apply(),
    );
}

thread_local! {
    /// The newest change being applied: an older one that finishes later is dropped.
    static APPLYING: Signal<u64> = Signal::global(0);
    /// The newest change that finished applying (equal to [`APPLYING`] when none is under way).
    static APPLIED: Signal<u64> = Signal::global(0);
    /// Why the last change could not be applied, for the status line.
    static APPLY_ERROR: Signal<Option<String>> = Signal::global(None);
    /// A change was applied after the current page loaded: offer to reload it.
    static RELOAD_OFFER: Signal<bool> = Signal::global(false);
}

/// Whether a change is being prepared. A tracked read.
pub fn applying() -> bool {
    APPLYING.with(|a| a.get()) != APPLIED.with(|a| a.get())
}

/// Apply the stored settings: download what they lack and compile their engines while the
/// current ones keep blocking, export WebKit's rule lists for them, then switch. When a list
/// cannot be downloaded the current settings stay, and the preferences go back to them.
fn apply() {
    let Some(a) = engine() else {
        return;
    };
    let desired = desired_settings();
    if desired == a.settings() {
        return;
    }
    let seq = APPLYING.with(|s| {
        s.update(|n| *n += 1);
        s.get_untracked()
    });
    APPLY_ERROR.with(|e| e.set(None));
    let a = a.clone();
    day::task(async move {
        let current = move || APPLYING.with(|s| s.get_untracked()) == seq;
        let report = a.stage(desired.clone()).await;
        if !current() {
            return;
        }
        // Refused only for a list the change switches on that has no copy: a list the current
        // settings already lack (an empty or unreachable list) changes nothing by staying out.
        let applied = a.settings();
        let in_use: Vec<SourceId> = applied
            .active_levels()
            .iter()
            .flat_map(|level| applied.sources_for(level))
            .collect();
        let newly_missing: Vec<&SourceId> = report
            .missing
            .iter()
            .filter(|id| !in_use.contains(id))
            .collect();
        if !newly_missing.is_empty() || report.error.is_some() {
            warn!("filter lists not applied: {report:?}");
            let names: Vec<String> = newly_missing
                .into_iter()
                .filter_map(|id| desired.source(id).map(|s| s.title.into_owned()))
                .collect();
            APPLY_ERROR.with(|e| e.set(Some(names.join(", "))));
            restore_prefs(&a.settings());
            APPLIED.with(|s| s.set(seq));
            return;
        }
        // WebKit's views take their rules from these exports: have the new ones ready before
        // the switch, so a view goes from the old rules straight to the new.
        if day_piece_webview::content_rules_support() != Support::Unsupported {
            for level in desired.active_levels() {
                if let Some(lists) = export(&a, &desired, &level).await {
                    RULES.with(|r| r.borrow_mut().insert(level, lists));
                }
                if !current() {
                    return;
                }
            }
        }
        a.set_settings(desired);
        RELOAD_OFFER.with(|o| o.set(true));
        APPLIED.with(|s| s.set(seq));
        VERSION.with(|v| v.update(|n| *n += 1));
    });
}

/// Put the preferences back to `applied`, after a change that could not be applied. Lists added
/// by address stay: they were checked and stored when they were added.
fn restore_prefs(applied: &Settings) {
    let mut custom = CustomLists::get_untracked();
    match &applied.level {
        Level::Custom {
            sources,
            document_blocking,
            same_site_blocking,
        } => {
            custom.sources = sources.clone();
            custom.document_blocking = *document_blocking;
            custom.same_site_blocking = *same_site_blocking;
            custom.save();
        }
        level => LEVEL.set(level.id().to_string()),
    }
    if applied.level.id() == "custom" {
        LEVEL.set("custom".to_string());
    }
    if let Ok(json) = serde_json::to_string(&applied.sites) {
        SITE_LEVELS.set(json);
    }
}

/// Offer to reload the page after a change; the offer goes once taken or dismissed.
pub fn reload_offered() -> bool {
    RELOAD_OFFER.with(|o| o.get())
}

/// Reload the current page with the new lists.
pub fn reload_current_page() {
    RELOAD_OFFER.with(|o| o.set(false));
    if let Some(browser) =
        crate::browser::Browser::try_ambient().or_else(crate::browser::Browser::focused)
        && let Some(tab) = browser.current_untracked()
    {
        tab.reload_or_stop();
    }
}

/// Whether `id` is one of the lists the global level uses. A tracked read.
pub fn list_enabled(id: &SourceId) -> bool {
    global_level().sources().contains(id)
}

/// Switch one list on or off: the level becomes Custom, with the current level's other lists
/// and policies.
pub fn set_list_enabled(id: &SourceId, on: bool) {
    let level = untrack(global_level);
    let mut custom = CustomLists::get_untracked();
    let mut sources = level.sources();
    sources.retain(|s| s != id);
    if on {
        sources.push(id.clone());
    }
    custom.sources = sources;
    custom.document_blocking = level.document_blocking();
    custom.same_site_blocking = level.same_site_blocking();
    custom.save();
    LEVEL.set("custom".to_string());
}

/// Change one of the policies a list does not set: the level becomes Custom, with the current
/// level's lists.
pub fn set_policy(document_blocking: Option<bool>, same_site_blocking: Option<bool>) {
    let level = untrack(global_level);
    let mut custom = CustomLists::get_untracked();
    custom.sources = level.sources();
    custom.document_blocking = document_blocking.unwrap_or(level.document_blocking());
    custom.same_site_blocking = same_site_blocking.unwrap_or(level.same_site_blocking());
    custom.save();
    LEVEL.set("custom".to_string());
}

thread_local! {
    /// Adding a list by address: where the check stands.
    static ADDING: Signal<AddState> = Signal::global(AddState::Idle);
}

/// Where adding a list by address stands.
#[derive(Clone, Debug, PartialEq)]
pub enum AddState {
    Idle,
    Checking,
    Failed(String),
    Added {
        title: String,
        rules: u32,
        unsupported: u32,
    },
}

/// Where adding a list stands. A tracked read.
pub fn add_state() -> AddState {
    ADDING.with(|a| a.get())
}

/// Download and check the list at `url`, then add it and switch it on.
pub fn add_list(url: String) {
    let Some(a) = engine() else {
        return;
    };
    // Fixture mode answers downloads at their real addresses (day-part-http's simulation).
    let url = url.trim().to_string();
    ADDING.with(|s| s.set(AddState::Checking));
    let a = a.clone();
    day::task(async move {
        match a.check_list(&url).await {
            Ok(check) => {
                let mut custom = CustomLists::get_untracked();
                if !custom.added.iter().any(|s| s.id == check.source.id) {
                    custom.added.push(check.source.clone());
                }
                custom.save();
                set_list_enabled(&check.source.id, true);
                ADDING.with(|s| {
                    s.set(AddState::Added {
                        title: check.source.title.to_string(),
                        rules: check.rules.network + check.rules.cosmetic,
                        unsupported: check.rules.unsupported,
                    })
                });
            }
            Err(error) => ADDING.with(|s| s.set(AddState::Failed(error))),
        }
    });
}

/// Remove a list added by address, and switch it off.
pub fn remove_list(id: &SourceId) {
    if untrack(|| list_enabled(id)) {
        set_list_enabled(id, false);
    }
    let mut custom = CustomLists::get_untracked();
    custom.added.retain(|s| &s.id != id);
    custom.sources.retain(|s| s != id);
    custom.save();
}

/// The lists added by address. A tracked read.
pub fn added_lists() -> Vec<Source> {
    CustomLists::get().added
}

/// One list's state for its row: purpose, source, and how its last download went. A tracked
/// read.
pub fn list_detail(source: &Source) -> String {
    VERSION.with(|v| v.get());
    let purpose = match source.purpose {
        Purpose::Privacy => res::str::blocking_purpose_privacy(),
        Purpose::Ads => res::str::blocking_purpose_ads(),
        Purpose::Cookies => res::str::blocking_purpose_cookies(),
        Purpose::Annoyances => res::str::blocking_purpose_annoyances(),
        Purpose::Social => res::str::blocking_purpose_social(),
        Purpose::Regional => res::str::blocking_purpose_regional(),
        Purpose::Security => res::str::blocking_purpose_security(),
    }
    .format();
    let host = source
        .urls
        .first()
        .map(|u| netskip_adblock::url_host(u).to_string())
        .unwrap_or_default();
    let info = engine().and_then(|a| a.lists().into_iter().find(|l| l.source.id == source.id));
    let failed = info.as_ref().is_some_and(|l| {
        matches!(
            l.state.last_result,
            Some(netskip_adblock::UpdateOutcome::Failed { .. })
        )
    });
    let status = match info {
        Some(l) if l.downloaded && failed => res::str::blocking_list_stale().format(),
        Some(l) if l.downloaded => match l.state.fetched_at {
            Some(at) => updated_ago(at),
            None => res::str::blocking_list_not_downloaded().format(),
        },
        Some(_) if failed => res::str::blocking_list_failed().format(),
        _ => res::str::blocking_list_not_downloaded().format(),
    };
    [purpose, host, status]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

fn updated_ago(at: u64) -> String {
    let now = (crate::now_ms() / 1000).max(0) as u64;
    let secs = now.saturating_sub(at);
    match secs {
        0..60 => res::str::blocking_list_updated_now().format(),
        60..3_600 => res::str::blocking_list_updated_minutes((secs / 60) as i64).format(),
        3_600..86_400 => res::str::blocking_list_updated_hours((secs / 3_600) as i64).format(),
        _ => res::str::blocking_list_updated_days((secs / 86_400) as i64).format(),
    }
}

/// The built-in lists, general ones first, then regional ones.
pub fn builtin_lists(regional: bool) -> Vec<Source> {
    netskip_adblock::catalog()
        .iter()
        .filter(|s| (s.purpose == Purpose::Regional) == regional)
        .cloned()
        .collect()
}

/// The running blocker, when content blocking could start.
pub fn engine() -> Option<&'static AdBlock> {
    ENGINE.get()?.as_ref()
}

/// The level the site of `url` gets. A tracked read.
pub fn level_for(url: &str) -> Level {
    VERSION.with(|v| v.get());
    LEVEL.get();
    match (engine(), address::site(url)) {
        (Some(a), Some(site)) => a.level_for(&site),
        _ => untrack(global_level),
    }
}

/// Give the site of `url` its own level, or `None` to follow the default again.
pub fn set_site_level(url: &str, level: Option<Level>) {
    let Some(site) = address::site(url) else {
        return;
    };
    let mut sites: SiteLevels =
        serde_json::from_str(&SITE_LEVELS.get_untracked()).unwrap_or_default();
    sites.set(&site, level);
    // Applied once the site's lists are ready (`apply`).
    if let Ok(json) = serde_json::to_string(&sites) {
        SITE_LEVELS.set(json);
    }
}

/// One line on where the lists stand, for Settings. A tracked read.
pub fn status_text() -> String {
    VERSION.with(|v| v.get());
    if applying() {
        return res::str::blocking_status_preparing().format();
    }
    if let Some(lists) = APPLY_ERROR.with(|e| e.get()) {
        return res::str::blocking_status_kept(lists).format();
    }
    match engine().map(AdBlock::status) {
        None => crate::res::str::blocking_unavailable().format(),
        Some(netskip_adblock::Status::Off) => crate::res::str::blocking_status_off().format(),
        Some(netskip_adblock::Status::WaitingForLists | netskip_adblock::Status::Loading) => {
            crate::res::str::blocking_status_waiting().format()
        }
        Some(netskip_adblock::Status::Ready { lists, .. }) => {
            crate::res::str::blocking_status_ready(lists as i64).format()
        }
        Some(netskip_adblock::Status::Failed { .. }) => {
            crate::res::str::blocking_status_failed().format()
        }
    }
}

/// Download every list now (Settings' Update Lists Now).
pub fn update_now() {
    if let Some(a) = engine() {
        let a = a.clone();
        day::task(async move {
            let report = a.update_now().await;
            info!("filter lists updated: {report:?}");
            VERSION.with(|v| v.update(|n| *n += 1));
        });
    }
}

/// The request filter the Chromium engines, Qt and ArkWeb consult. Fixture mode's pages load
/// from one physical address; the filter judges the real addresses they stand for.
pub struct Filter(AdBlock);

/// The filter for a web view, when content blocking runs.
pub fn filter() -> Option<Arc<dyn RequestFilter>> {
    engine().map(|a| Arc::new(Filter(a.clone())) as Arc<dyn RequestFilter>)
}

impl RequestFilter for Filter {
    fn decide(&self, request: &RequestInfo<'_>) -> RequestDecision {
        let url = fixtures::to_logical(request.url);
        let page = request
            .top_url
            .map(fixtures::to_logical)
            .unwrap_or_default();
        let kind = if request.main_frame {
            netskip_adblock::ResourceKind::Document
        } else {
            map_kind(request.kind)
        };
        let mut info = netskip_adblock::RequestInfo::new(&url, &page, kind);
        info.main_frame = request.main_frame;
        let block = self.0.decide(&info).block;
        if block {
            RequestDecision::Block
        } else {
            RequestDecision::Allow
        }
    }
}

fn map_kind(kind: ResourceKind) -> netskip_adblock::ResourceKind {
    use netskip_adblock::ResourceKind as K;
    match kind {
        ResourceKind::Document => K::Document,
        ResourceKind::Subdocument => K::Subdocument,
        ResourceKind::Script => K::Script,
        ResourceKind::Stylesheet => K::Stylesheet,
        ResourceKind::Image => K::Image,
        ResourceKind::Font => K::Font,
        ResourceKind::Media => K::Media,
        ResourceKind::Xhr => K::Xhr,
        ResourceKind::Fetch => K::Fetch,
        ResourceKind::WebSocket => K::WebSocket,
        ResourceKind::Ping => K::Ping,
        ResourceKind::Object => K::Object,
        ResourceKind::Other => K::Other,
        _ => K::Unknown,
    }
}

/// WebKit's rule lists for a page on `url`'s site: the lists of the level that site gets. A
/// tracked read. The first read of a level starts its export in the background and answers
/// with no rules; the lists arrive when it finishes, and again whenever the lists change.
pub fn content_rules(url: &str) -> Vec<ContentRuleList> {
    // Only WebKit runs rule lists; elsewhere the request filter does the blocking.
    if day_piece_webview::content_rules_support() == Support::Unsupported {
        return Vec::new();
    }
    let level = level_for(url);
    let Some(a) = engine() else {
        return Vec::new();
    };
    if level.is_off() {
        return Vec::new();
    }
    let cached = RULES.with(|r| r.borrow().get(&level).cloned());
    // Export again after each change the engine reports (`level_for` tracks it); an unchanged
    // export publishes nothing.
    let due = BUILDING.with(|b| b.borrow_mut().insert(level.clone()));
    if due {
        let a = a.clone();
        day::task(async move {
            let settings = a.settings();
            if let Some(lists) = export(&a, &settings, &level).await {
                let changed = RULES.with(|r| {
                    let mut r = r.borrow_mut();
                    let same = r.get(&level).is_some_and(|old| {
                        old.first().map(|l| &l.version) == lists.first().map(|l| &l.version)
                    });
                    r.insert(level.clone(), lists);
                    !same
                });
                if changed {
                    VERSION.with(|v| v.update(|n| *n += 1));
                }
            }
            BUILDING.with(|b| b.borrow_mut().remove(&level));
        });
    }
    cached.unwrap_or_default()
}

/// WebKit's rule lists for `level` under `settings`, read from the export netskip-adblock writes
/// (and caches) in the background.
async fn export(a: &AdBlock, settings: &Settings, level: &Level) -> Option<Vec<ContentRuleList>> {
    match a.content_rules_for(settings, level).await {
        Ok(rules) => {
            let lists: Vec<ContentRuleList> = rules
                .lists
                .iter()
                .filter_map(|l| {
                    let json = std::fs::read_to_string(&l.path).ok()?;
                    Some(ContentRuleList::new(
                        l.identifier.clone(),
                        rules.version.clone(),
                        json,
                    ))
                })
                .collect();
            info!(
                "content rules for {}: version {}, {} lists",
                level.id(),
                rules.version,
                rules.lists.len()
            );
            Some(lists)
        }
        Err(e) => {
            warn!("content rules for {} unavailable: {e}", level.id());
            None
        }
    }
}

/// The element-hiding script every page runs at document start.
pub fn cosmetic_script() -> UserScript {
    UserScript::new(netskip_adblock::cosmetic_script())
}

/// The channel the element-hiding script reports on.
pub const SCRIPT_CHANNEL: &str = netskip_adblock::SCRIPT_CHANNEL;

/// Answer the element-hiding script: the selectors to hide on its page.
pub fn script_message(message: ScriptMessage) -> Option<String> {
    let a = engine()?;
    let body = fixtures_logical_message(&message.payload);
    a.handle_script_message(&body)
}

/// Fixture mode reports physical page addresses; the engine knows the sites by their real ones.
fn fixtures_logical_message(body: &str) -> String {
    if !fixtures::active() {
        return body.to_string();
    }
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(mut v) => {
            if let Some(url) = v.get("url").and_then(|u| u.as_str()) {
                let logical = fixtures::to_logical(url);
                v["url"] = serde_json::Value::String(logical);
            }
            v.to_string()
        }
        Err(_) => body.to_string(),
    }
}

/// Count what a page had blocked, for the page menu's shield line.
pub fn note_blocked(count: Signal<usize>, loads: &[BlockedLoad]) {
    count.update(|n| *n += loads.len());
}

/// Settings' Filter Lists page: the level in use and where its lists stand, every list with
/// a switch, lists added by address, the two policies, and Update Now and Restore Preset.
pub fn filter_lists_page() -> impl Piece {
    scroll(
        form((
            section((
                label(move || level_label(&LEVEL.get())).font(Font::Headline),
                label(status_text)
                    .font(Font::Footnote)
                    .color(move || day::system_color(SystemColor::SecondaryText))
                    .id(day::id!(filter_lists_status)),
                when(reload_offered, || {
                    row((
                        label(res::str::blocking_reload_offer())
                            .font(Font::Footnote)
                            .grow(Dimension::Width),
                        button(res::str::blocking_reload_page())
                            .action(reload_current_page)
                            .id(day::id!(filter_lists_reload)),
                    ))
                    .spacing(8.0)
                    .align(VAlign::Center)
                }),
                row((
                    button(res::str::blocking_update())
                        .action(update_now)
                        .id(day::id!(filter_lists_update)),
                    spacer(),
                    menu_button(res::str::blocking_restore_preset(), || {
                        LEVELS
                            .iter()
                            .map(|id| {
                                menu_item(level_label(id))
                                    .checked(move || LEVEL.get() == *id)
                                    .action(move || LEVEL.set(id.to_string()))
                                    .id(day::id!(restore_preset, *id))
                            })
                            .collect::<Vec<_>>()
                    })
                    .id(day::id!(filter_lists_restore)),
                ))
                .align(VAlign::Center),
                label(res::str::blocking_lists_caption())
                    .font(Font::Footnote)
                    .color(move || day::system_color(SystemColor::SecondaryText)),
            )),
            section((column(
                builtin_lists(false)
                    .into_iter()
                    .map(|source| list_row(source, false))
                    .collect::<Vec<_>>(),
            )
            .spacing(12.0),))
            .title(res::str::blocking_lists_general()),
            section((column(
                builtin_lists(true)
                    .into_iter()
                    .map(|source| list_row(source, false))
                    .collect::<Vec<_>>(),
            )
            .spacing(12.0),))
            .title(res::str::blocking_lists_regional()),
            section((
                switch(
                    || {
                        added_lists()
                            .iter()
                            .map(|s| s.id.clone())
                            .collect::<Vec<_>>()
                    },
                    |ids| {
                        let added = untrack(added_lists);
                        column(
                            ids.iter()
                                .filter_map(|id| added.iter().find(|s| &s.id == id).cloned())
                                .map(|source| list_row(source, true))
                                .collect::<Vec<_>>(),
                        )
                        .spacing(12.0)
                    },
                ),
                add_list_form(),
            ))
            .title(res::str::blocking_lists_added()),
            section((
                policy_row(
                    res::str::blocking_document_blocking(),
                    res::str::blocking_document_blocking_caption(),
                    || global_level().document_blocking(),
                    |on| set_policy(Some(on), None),
                    day::id!(document_blocking),
                ),
                policy_row(
                    res::str::blocking_same_site(),
                    res::str::blocking_same_site_caption(),
                    || global_level().same_site_blocking(),
                    |on| set_policy(None, Some(on)),
                    day::id!(same_site_blocking),
                ),
            ))
            .title(res::str::blocking_options()),
        ))
        .padding(16.0),
    )
}

/// A switch that follows `state` and calls `set` when the user flips it.
fn synced_toggle(
    state: impl Fn() -> bool + Clone + 'static,
    set: impl Fn(bool) + 'static,
    id: day::ElementId,
) -> impl Piece {
    let on = Signal::new(untrack(state.clone()));
    let read = state.clone();
    watch(read, move |now, _| on.set(*now));
    watch(
        move || on.get(),
        move |value, before| {
            if before.is_some() && *value != untrack(state.clone()) {
                set(*value);
            }
        },
    );
    toggle(on).id(id)
}

/// One list: its name, what it blocks, where it comes from and how its last download went, a
/// switch, and for a list added by address a Remove button.
fn list_row(source: Source, removable: bool) -> impl Piece {
    let id = source.id.clone();
    let (on_id, set_id, remove_id) = (id.clone(), id.clone(), id.clone());
    let detail_source = source.clone();
    row((
        column((
            label(source.title.to_string()),
            label(move || list_detail(&detail_source))
                .font(Font::Footnote)
                .align(TextAlign::Leading)
                .color(move || day::system_color(SystemColor::SecondaryText)),
        ))
        .spacing(2.0)
        .align(HAlign::Leading)
        .grow(Dimension::Width),
        when(
            move || removable,
            move || {
                let id = remove_id.clone();
                let element = day::id!(remove_list, id.as_str());
                button(res::str::blocking_remove_list())
                    .plain()
                    .action(move || remove_list(&id))
                    .id(element)
            },
        ),
        synced_toggle(
            move || list_enabled(&on_id),
            move |on| set_list_enabled(&set_id, on),
            day::id!(filter_list, id.as_str()),
        ),
    ))
    .spacing(8.0)
    .align(VAlign::Center)
}

fn policy_row<M1, M2>(
    title: impl IntoText<M1>,
    caption: impl IntoText<M2>,
    state: impl Fn() -> bool + Clone + 'static,
    set: impl Fn(bool) + 'static,
    id: day::ElementId,
) -> impl Piece {
    row((
        column((
            label(title),
            label(caption)
                .font(Font::Footnote)
                .align(TextAlign::Leading)
                .color(move || day::system_color(SystemColor::SecondaryText)),
        ))
        .spacing(2.0)
        .align(HAlign::Leading)
        .grow(Dimension::Width),
        synced_toggle(state, set, id),
    ))
    .spacing(8.0)
    .align(VAlign::Center)
}

/// The address field and Add button, with where the check stands under them.
fn add_list_form() -> impl Piece {
    let url = Signal::new(String::new());
    // Ready for the next address once a list is added.
    watch(add_state, move |state, _| {
        if matches!(state, AddState::Added { .. }) {
            url.set(String::new());
        }
    });
    let add = move || {
        let text = url.get_untracked();
        if !text.trim().is_empty() && add_state() != AddState::Checking {
            add_list(text);
        }
    };
    column((
        row((
            text_field(url)
                .placeholder(res::str::blocking_add_list_placeholder())
                .input_purpose(InputPurpose::Url)
                .submit_label(SubmitLabel::Done)
                .on_submit(add)
                .id(day::id!(add_list_url))
                .grow(Dimension::Width),
            button(res::str::blocking_add_list())
                .action(add)
                .enabled(move || add_state() != AddState::Checking && !url.get().trim().is_empty())
                .id(day::id!(add_list)),
        ))
        .spacing(8.0)
        .align(VAlign::Center),
        label(move || match add_state() {
            AddState::Idle => res::str::blocking_add_list_caption().format(),
            AddState::Checking => res::str::blocking_add_checking().format(),
            AddState::Failed(error) => res::str::blocking_add_failed(error).format(),
            AddState::Added {
                title,
                rules,
                unsupported,
            } => {
                let added = res::str::blocking_add_done(res::str::BlockingAddDone {
                    rules: rules as i64,
                    title,
                })
                .format();
                if unsupported > 0 {
                    format!(
                        "{added} {}",
                        res::str::blocking_add_unsupported(unsupported as i64).format()
                    )
                } else {
                    added
                }
            }
        })
        .font(Font::Footnote)
        .color(move || day::system_color(SystemColor::SecondaryText))
        .align(TextAlign::Leading)
        .id(day::id!(add_list_status)),
    ))
    .spacing(6.0)
    .align(HAlign::Leading)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_read_back_from_their_ids() {
        for id in LEVELS {
            assert_eq!(level_from_id(id).id(), id);
        }
        assert_eq!(level_from_id("bogus").id(), "light");
    }
}
