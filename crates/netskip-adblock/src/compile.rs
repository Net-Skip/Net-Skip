// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! The compiled matcher: one adblock-rust engine per list, the engine cache file, the request
//! and cosmetic queries, and the WebKit content rule export.
//!
//! Each list gets its own engine so a blocked request can name the list that blocked it, which
//! the per-purpose statistics need. A combined engine forgets that once its optimizer fuses
//! rules from different lists. Exceptions still work across lists: a request is blocked when
//! some list blocks it and no list excepts it, or when some list marks the block `$important`.
//! `$generichide` exceptions are copied into every engine for the same reason.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use adblock::lists::{FilterFormat, ParseOptions};
use adblock::{Engine, FilterSet};
use serde::{Deserialize, Serialize};

use crate::catalog::{Fnv, Level, ListFormat, Purpose, SourceId};
use crate::update::write_atomic;
use crate::{CosmeticResources, Decision, Error, RequestInfo, ResourceKind};

/// Bumped when preprocessing or the cache file layout changes.
const FORMAT_VERSION: u32 = 1;
/// Bumped when the WebKit export's shape changes, so exports written by an older build are
/// rebuilt (2: generic element hiding and procedural selectors left to the cosmetic script).
const WEBKIT_EXPORT_VERSION: u32 = 2;
const MAGIC: &[u8; 8] = b"DAYADBK\x01";
/// The adblock-rust release line; its serialized format is only promised within one release.
const ENGINE_VERSION: &str = "adblock-0.13";

/// One stored list, ready to compile.
#[derive(Clone, Debug)]
pub(crate) struct Input {
    pub(crate) id: SourceId,
    pub(crate) purpose: Purpose,
    pub(crate) format: ListFormat,
    pub(crate) path: PathBuf,
    pub(crate) content_hash: String,
}

/// The cache key of a level compiled from `inputs`.
pub(crate) fn inputs_hash(level: &Level, inputs: &[Input]) -> u64 {
    let mut h = Fnv::new();
    h.write(env!("CARGO_PKG_VERSION").as_bytes());
    h.write(ENGINE_VERSION.as_bytes());
    h.write(&FORMAT_VERSION.to_le_bytes());
    h.write(level.cache_key().as_bytes());
    for input in inputs {
        h.write(input.id.as_str().as_bytes());
        h.write(&[0, input.purpose as u8, input.format as u8]);
        h.write(input.content_hash.as_bytes());
        h.write(&[0]);
    }
    h.finish()
}

struct SourceEngine {
    id: SourceId,
    purpose: Purpose,
    engine: Engine,
}

/// The matcher for one level.
pub(crate) struct Compiled {
    pub(crate) hash: u64,
    engines: Vec<SourceEngine>,
    document_blocking: bool,
    same_site_blocking: bool,
    /// The level's lists that had no stored copy at compile time.
    pub(crate) missing: Vec<SourceId>,
}

// The engine is shared by request callbacks on background threads. adblock-rust is `Send +
// Sync` with its `single-thread` feature off, which this checks at compile time.
const _: () = {
    const fn shared<T: Send + Sync>() {}
    shared::<Engine>();
    shared::<Compiled>();
};

/// Builds are serialized: two at once would only compete for memory, and a second build of the
/// same level finds the first one's cache file.
static BUILD: Mutex<()> = Mutex::new(());

/// Drop the rules this crate never runs: scriptlets and snippets (`#$#`, `##+js(...)`,
/// `#%#`), whose code would need the scriptlet library adblock-rust leaves to its host.
fn preprocess(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let t = line.trim();
        if t.contains("#$#")
            || t.contains("#@$#")
            || t.contains("#%#")
            || t.contains("#@%#")
            || t.contains("##+js(")
            || t.contains("#@#+js(")
        {
            continue;
        }
        out.push_str(t);
        out.push('\n');
    }
    out
}

fn is_generichide_exception(line: &str) -> bool {
    if !line.starts_with("@@") {
        return false;
    }
    let Some((_, options)) = line.rsplit_once('$') else {
        return false;
    };
    options
        .split(',')
        .any(|o| matches!(o.trim(), "generichide" | "ghide" | "elemhide" | "ehide"))
}

fn parse_options(format: ListFormat) -> ParseOptions {
    ParseOptions {
        format: match format {
            ListFormat::Adblock => FilterFormat::Standard,
            ListFormat::Hosts => FilterFormat::Hosts,
        },
        ..ParseOptions::default()
    }
}

/// Read and preprocess each input. A list that cannot be read is reported as missing.
fn read_inputs(inputs: &[Input]) -> (Vec<(Input, String)>, Vec<SourceId>) {
    let mut texts = Vec::new();
    let mut unreadable = Vec::new();
    for input in inputs {
        match std::fs::read(&input.path) {
            Ok(bytes) => texts.push((input.clone(), preprocess(&String::from_utf8_lossy(&bytes)))),
            Err(_) => unreadable.push(input.id.clone()),
        }
    }
    (texts, unreadable)
}

fn engine_file(dir: &Path, level: &Level, hash: u64) -> PathBuf {
    dir.join(format!("engine-{}-{hash:016x}.bin", level.cache_key()))
}

/// Remove files named `<prefix><anything>` other than `keep`.
fn remove_stale(dir: &Path, prefix: &str, keep: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if path != keep && name.to_string_lossy().starts_with(prefix) {
            let _ = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };
        }
    }
}

/// Load the level's engines from the cache file, or build them from the stored lists and
/// write the cache file. Blocking: run it on a pool thread.
pub(crate) fn load_or_build(
    dir: &Path,
    level: &Level,
    inputs: &[Input],
    mut missing: Vec<SourceId>,
) -> Result<Compiled, Error> {
    let _guard = BUILD.lock().unwrap_or_else(|p| p.into_inner());
    let hash = inputs_hash(level, inputs);
    let path = engine_file(dir, level, hash);
    if let Ok(bytes) = std::fs::read(&path)
        && let Some(engines) = decode(&bytes, inputs)
    {
        return Ok(Compiled {
            hash,
            engines,
            document_blocking: level.document_blocking(),
            same_site_blocking: level.same_site_blocking(),
            missing,
        });
    }
    let (texts, unreadable) = read_inputs(inputs);
    if !unreadable.is_empty() {
        // A list vanished between the state check and now (the system cleared the cache).
        // Build nothing; the caller downloads it again.
        return Err(Error::Io(format!(
            "stored lists are missing: {}",
            unreadable
                .iter()
                .map(SourceId::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    let shared: String = texts
        .iter()
        .flat_map(|(_, text)| text.lines().filter(|l| is_generichide_exception(l)))
        .fold(String::new(), |mut acc, line| {
            acc.push_str(line);
            acc.push('\n');
            acc
        });
    let mut engines = Vec::with_capacity(texts.len());
    for (input, text) in texts {
        let mut set = FilterSet::new(false);
        let format = crate::update::detect_format(&text, input.format);
        set.add_filter_list(text, parse_options(format));
        if !shared.is_empty() {
            set.add_filter_list(shared.clone(), ParseOptions::default());
        }
        engines.push(SourceEngine {
            id: input.id,
            purpose: input.purpose,
            engine: Engine::new_with_filter_set(set),
        });
    }
    let bytes = encode(&engines);
    // A failed cache write costs the next launch a rebuild, nothing more.
    if write_atomic(&path, &bytes).is_ok() {
        remove_stale(dir, &format!("engine-{}-", level.cache_key()), &path);
    }
    missing.sort();
    Ok(Compiled {
        hash,
        engines,
        document_blocking: level.document_blocking(),
        same_site_blocking: level.same_site_blocking(),
        missing,
    })
}

/// `MAGIC`, a count, then per list: id length (u16), id, purpose index (u8), payload length
/// (u32), the engine's own serialization. All integers little-endian.
fn encode(engines: &[SourceEngine]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(engines.len() as u32).to_le_bytes());
    for e in engines {
        let id = e.id.as_str().as_bytes();
        let payload = e.engine.serialize();
        out.extend_from_slice(&(id.len() as u16).to_le_bytes());
        out.extend_from_slice(id);
        out.push(e.purpose as u8);
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
    }
    out
}

fn decode(bytes: &[u8], inputs: &[Input]) -> Option<Vec<SourceEngine>> {
    fn take<'a>(bytes: &mut &'a [u8], n: usize) -> Option<&'a [u8]> {
        if bytes.len() < n {
            return None;
        }
        let (head, rest) = bytes.split_at(n);
        *bytes = rest;
        Some(head)
    }
    let mut rest = bytes;
    if take(&mut rest, MAGIC.len())? != MAGIC {
        return None;
    }
    let count = u32::from_le_bytes(take(&mut rest, 4)?.try_into().ok()?) as usize;
    if count != inputs.len() {
        return None;
    }
    let mut engines = Vec::with_capacity(count);
    for input in inputs {
        let id_len = u16::from_le_bytes(take(&mut rest, 2)?.try_into().ok()?) as usize;
        let id = std::str::from_utf8(take(&mut rest, id_len)?).ok()?;
        let purpose = *Purpose::ALL.get(usize::from(*take(&mut rest, 1)?.first()?))?;
        let len = u32::from_le_bytes(take(&mut rest, 4)?.try_into().ok()?) as usize;
        let payload = take(&mut rest, len)?;
        if id != input.id.as_str() || purpose != input.purpose {
            return None;
        }
        let mut engine = Engine::default();
        engine.deserialize(payload).ok()?;
        engines.push(SourceEngine {
            id: input.id.clone(),
            purpose,
            engine,
        });
    }
    rest.is_empty().then_some(engines)
}

// --- Queries ---

impl ResourceKind {
    fn adblock_type(self) -> &'static str {
        match self {
            ResourceKind::Document => "document",
            ResourceKind::Subdocument => "subdocument",
            ResourceKind::Script => "script",
            ResourceKind::Stylesheet => "stylesheet",
            ResourceKind::Image => "image",
            ResourceKind::Font => "font",
            ResourceKind::Media => "media",
            // Filter lists have one type for both: `$xmlhttprequest` covers `fetch`.
            ResourceKind::Xhr | ResourceKind::Fetch => "xmlhttprequest",
            ResourceKind::WebSocket => "websocket",
            ResourceKind::Ping => "ping",
            ResourceKind::Object => "object",
            ResourceKind::Other | ResourceKind::Unknown => "other",
        }
    }
}

impl Compiled {
    /// The number of lists in this matcher.
    pub(crate) fn list_count(&self) -> usize {
        self.engines.len()
    }

    /// Whether to block a request, and which list blocked it.
    pub(crate) fn decide(&self, info: &RequestInfo<'_>) -> (Decision, Option<Purpose>) {
        let allow = (Decision::default(), None);
        let kind = info.resolved_kind();
        let top_document = kind == ResourceKind::Document && info.main_frame;
        if top_document && !self.document_blocking {
            return allow;
        }
        // A navigation is judged against its own address: `$domain=` and `$third-party` on a
        // document rule describe the page being opened.
        let source = if top_document || info.source_url.is_empty() {
            info.url
        } else {
            info.source_url
        };
        let Ok(request) =
            adblock::request::Request::new(info.url, source, kind.adblock_type(), "get")
        else {
            return allow;
        };
        if !top_document && !request.is_third_party && !self.same_site_blocking {
            return allow;
        }
        let results: Vec<_> = self
            .engines
            .iter()
            .map(|e| e.engine.check_network_request(&request))
            .collect();
        if let Some(i) = results.iter().position(|r| r.important) {
            return self.blocked_by(i);
        }
        let Some(blocker) = results
            .iter()
            .position(|r| r.filter.is_some() && r.exception.is_none())
        else {
            return allow;
        };
        if results.iter().any(|r| r.exception.is_some()) {
            return allow;
        }
        // Lists that matched no block rule have not looked for exceptions yet.
        for (i, e) in self.engines.iter().enumerate() {
            if results[i].filter.is_some() {
                continue;
            }
            let r = e.engine.check_network_request_subset(&request, true, false);
            if r.important {
                return self.blocked_by(i);
            }
            if r.exception.is_some() {
                return allow;
            }
        }
        self.blocked_by(blocker)
    }

    fn blocked_by(&self, i: usize) -> (Decision, Option<Purpose>) {
        let e = &self.engines[i];
        (
            Decision {
                block: true,
                matched_list: Some(e.id.clone()),
                redirect: None,
            },
            Some(e.purpose),
        )
    }

    /// The element hiding rules for a page.
    pub(crate) fn cosmetic(&self, url: &str) -> CosmeticResources {
        let mut hide = HashSet::new();
        let mut exceptions = HashSet::new();
        let mut generichide = false;
        for e in &self.engines {
            let r = e.engine.url_cosmetic_resources(url);
            generichide |= r.generichide;
            hide.extend(r.hide_selectors);
            exceptions.extend(r.exceptions);
        }
        let mut hide_selectors: Vec<String> = hide
            .into_iter()
            .filter(|s| !exceptions.contains(s))
            .collect();
        hide_selectors.sort();
        let mut exceptions: Vec<String> = exceptions.into_iter().collect();
        exceptions.sort();
        CosmeticResources {
            hide_selectors,
            exceptions,
            generichide,
        }
    }

    /// Generic hiding selectors for classes and ids seen on a page.
    pub(crate) fn generic_selectors(
        &self,
        classes: &[String],
        ids: &[String],
        exceptions: &[String],
    ) -> Vec<String> {
        let exceptions: HashSet<String> = exceptions.iter().cloned().collect();
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for e in &self.engines {
            for s in e
                .engine
                .hidden_class_id_selectors(classes, ids, &exceptions)
            {
                if seen.insert(s.clone()) {
                    out.push(s);
                }
            }
        }
        out
    }
}

// --- WebKit content rules ---

/// WebKit's per-list rule limit (`maxRuleCount` in WebKit's `ContentExtensionParser.cpp`).
pub const WEBKIT_RULE_LIMIT: usize = 150_000;

/// A level exported as WebKit content rule lists, stored in the cache directory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentRules {
    /// Changes whenever the rules change: compare it with the version a host compiled last and
    /// recompile only when it differs.
    pub version: String,
    /// The lists, each within WebKit's per-list rule limit.
    pub lists: Vec<ContentRuleList>,
}

/// One WebKit content rule list: a JSON array of `{trigger, action}` objects.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentRuleList {
    /// A stable identifier for `WKContentRuleListStore`: `day-adblock-<level>-<n>`.
    pub identifier: String,
    /// The JSON file.
    pub path: PathBuf,
    /// The number of rules in the file.
    pub rules: usize,
}

impl ContentRuleList {
    /// The JSON text, for `compileContentRuleList(forIdentifier:encodedContentRuleList:)`.
    pub fn read_json(&self) -> std::io::Result<String> {
        std::fs::read_to_string(&self.path)
    }
}

/// Requests to the page's own site are never blocked when same-site blocking is off. Every
/// type but documents, so the rule does not also cancel the page's element hiding rules.
fn first_party_exception() -> serde_json::Value {
    serde_json::json!({
        "trigger": {
            "url-filter": ".*",
            "load-type": ["first-party"],
            "resource-type": ["image", "style-sheet", "script", "font", "raw", "svg-document", "media"]
        },
        "action": { "type": "ignore-previous-rules" }
    })
}

/// Export a level's lists as WebKit content rules, cached under
/// `webkit-<level>-<hash>/`. Blocking: run it on a pool thread.
pub(crate) fn content_rules(
    dir: &Path,
    level: &Level,
    inputs: &[Input],
    limit: usize,
) -> Result<ContentRules, Error> {
    let mut h = Fnv::new();
    h.write(&inputs_hash(level, inputs).to_le_bytes());
    h.write(b"webkit");
    h.write(&WEBKIT_EXPORT_VERSION.to_le_bytes());
    h.write(&(limit as u64).to_le_bytes());
    let version = format!("{:016x}", h.finish());
    let key = level.cache_key();
    let folder = dir.join(format!("webkit-{key}-{version}"));
    let manifest = folder.join("manifest.json");
    if let Ok(bytes) = std::fs::read(&manifest)
        && let Ok(rules) = serde_json::from_slice::<ContentRules>(&bytes)
        && rules.version == version
        && rules.lists.iter().all(|l| l.path.is_file())
    {
        return Ok(rules);
    }

    let (texts, unreadable) = read_inputs(inputs);
    if let Some(id) = unreadable.first() {
        return Err(Error::Io(format!("stored list {id} is missing")));
    }
    let mut set = FilterSet::new(true);
    for (input, text) in texts {
        let format = crate::update::detect_format(&text, input.format);
        set.add_filter_list(text, parse_options(format));
    }
    let (rules, _) = set
        .into_content_blocking()
        .map_err(|()| Error::Engine("content blocking export failed".into()))?;
    let fp_documents = adblock::content_blocking::ignore_previous_fp_documents();
    let mut blocks = Vec::new();
    let mut exceptions = Vec::new();
    for mut rule in rules {
        if rule == fp_documents && level.document_blocking() {
            continue;
        }
        // Generic element hiding (a selector for every site) stays out of WebKit's rules: tens of
        // thousands of them become one style sheet applied to every page, which stalls heavy
        // pages. The cosmetic script hides them instead, for the classes and ids a page uses.
        if matches!(
            rule.action.typ,
            adblock::content_blocking::CbType::CssDisplayNone
        ) && rule.trigger.if_domain.is_none()
        {
            continue;
        }
        // Procedural cosmetic filters (`:has-text()`, `:-abp-contains()`, `:xpath()`, …) are not
        // CSS: WebKit would reject the selector, so they are left out rather than shipped.
        if rule
            .action
            .selector
            .as_deref()
            .is_some_and(is_procedural_selector)
        {
            continue;
        }
        // adblock-rust writes a cosmetic rule's domains bare, which WebKit matches as that
        // host only; `news.example##.x` also covers `www.news.example`, which needs the `*`.
        for domains in [&mut rule.trigger.if_domain, &mut rule.trigger.unless_domain]
            .into_iter()
            .flatten()
        {
            for d in domains.iter_mut() {
                if !d.starts_with('*') {
                    d.insert(0, '*');
                }
            }
        }
        let ignore = matches!(
            rule.action.typ,
            adblock::content_blocking::CbType::IgnorePreviousRules
        );
        let value = serde_json::to_value(&rule).map_err(|e| Error::Engine(e.to_string()))?;
        if ignore {
            exceptions.push(value);
        } else {
            blocks.push(value);
        }
    }
    if !level.same_site_blocking() {
        exceptions.push(first_party_exception());
    }
    // `ignore-previous-rules` only reaches rules earlier in the same list, so every chunk ends
    // with all the exceptions.
    let room = limit.saturating_sub(exceptions.len());
    if room == 0 && !blocks.is_empty() {
        return Err(Error::Engine(format!(
            "{} exception rules leave no room under the limit of {limit}",
            exceptions.len()
        )));
    }

    let tmp = dir.join(format!("webkit-{key}-{version}.tmp"));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|e| Error::Io(e.to_string()))?;
    let mut lists = Vec::new();
    for (n, chunk) in blocks.chunks(room.max(1)).enumerate() {
        let mut list: Vec<&serde_json::Value> = chunk.iter().collect();
        list.extend(exceptions.iter());
        let json = serde_json::to_vec(&list).map_err(|e| Error::Engine(e.to_string()))?;
        let name = format!("rules-{n}.json");
        std::fs::write(tmp.join(&name), json).map_err(|e| Error::Io(e.to_string()))?;
        lists.push(ContentRuleList {
            identifier: format!("day-adblock-{key}-{n}"),
            path: folder.join(&name),
            rules: list.len(),
        });
    }
    let rules = ContentRules { version, lists };
    let json = serde_json::to_vec_pretty(&rules).map_err(|e| Error::Engine(e.to_string()))?;
    std::fs::write(tmp.join("manifest.json"), json).map_err(|e| Error::Io(e.to_string()))?;
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::rename(&tmp, &folder).map_err(|e| Error::Io(e.to_string()))?;
    remove_stale(dir, &format!("webkit-{key}-"), &folder);
    Ok(rules)
}

/// Whether `selector` uses a uBlock Origin or Adblock Plus procedural pseudo-class, which only a
/// scriptlet can apply. `:has()` is plain CSS in WebKit and Chromium, so it stays.
fn is_procedural_selector(selector: &str) -> bool {
    const PROCEDURAL: [&str; 12] = [
        ":has-text(",
        ":-abp-",
        ":xpath(",
        ":upward(",
        ":matches-css",
        ":matches-attr(",
        ":matches-path(",
        ":min-text-length(",
        ":watch-attr(",
        ":remove(",
        ":style(",
        ":others(",
    ];
    PROCEDURAL.iter().any(|p| selector.contains(p))
}

#[cfg(test)]
mod procedural_tests {
    use super::is_procedural_selector;

    #[test]
    fn procedural_selectors_are_recognised() {
        assert!(is_procedural_selector(
            ".bg-gray-100:has-text(Advertisement)"
        ));
        assert!(is_procedural_selector("div:-abp-contains(Sponsored)"));
        assert!(!is_procedural_selector("article:has(> .sponsored)"));
        assert!(!is_procedural_selector(".ad-banner"));
    }
}
