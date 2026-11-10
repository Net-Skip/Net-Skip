// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! List downloads: conditional requests, the `! Expires:` schedule, backoff after failures, and
//! the cache directory's `state.json`.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use day_part_http::{CacheValidators, Client, HttpError, Request};
use serde::{Deserialize, Serialize};

use crate::catalog::{Fnv, ListFormat, Source, SourceId};

/// The shortest refresh interval a list can ask for.
pub const MIN_REFRESH: Duration = Duration::from_secs(60 * 60);
/// The longest refresh interval a list can ask for.
pub const MAX_REFRESH: Duration = Duration::from_secs(14 * 24 * 60 * 60);
/// The refresh interval of a list without an `! Expires:` line.
pub const DEFAULT_REFRESH: Duration = Duration::from_secs(24 * 60 * 60);
/// The first retry delay after a failed download; each further failure doubles it.
pub const FIRST_RETRY: Duration = Duration::from_secs(15 * 60);
/// The longest retry delay.
pub const MAX_RETRY: Duration = Duration::from_secs(12 * 60 * 60);

/// Wall-clock time for the update schedule. Tests install a clock they can move.
pub trait Clock: Send + Sync + 'static {
    /// The current time.
    fn now(&self) -> SystemTime;
}

/// The system clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

/// Seconds since the Unix epoch; times before it count as zero.
pub(crate) fn unix(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// How the last download attempt of a list ended.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum UpdateOutcome {
    /// A new copy was downloaded and stored.
    Updated,
    /// The server answered `304 Not Modified`; the stored copy is current.
    NotModified,
    /// The list was not due and was not requested.
    NotDue,
    /// Every URL failed. The last good copy, if any, stays in use.
    Failed {
        /// What went wrong, for logs and diagnostics. Not localized.
        error: String,
    },
}

/// Approximate rule counts of a stored list, from a line scan at download time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuleCounts {
    /// Network rules (including hosts lines and exceptions).
    pub network: u32,
    /// Element hiding rules and their exceptions.
    pub cosmetic: u32,
    /// Rules counted above that this blocker cannot apply: scriptlets and other script
    /// injection, HTML filtering, CSS injection, resource redirects and response rewriting.
    /// Approximate, from the rule's syntax.
    pub unsupported: u32,
}

/// What the updater knows about one list. Stored in `state.json`; times are Unix seconds.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ListState {
    /// The URL the stored copy came from.
    pub url: Option<String>,
    /// The stored copy's `ETag`.
    pub etag: Option<String>,
    /// The stored copy's `Last-Modified`.
    pub last_modified: Option<String>,
    /// When the server last confirmed the copy (a `200` or a `304`).
    pub fetched_at: Option<u64>,
    /// When the stored copy last changed (a `200`).
    pub changed_at: Option<u64>,
    /// When the list is due again.
    pub next_fetch: Option<u64>,
    /// The refresh interval in seconds the list asks for, after clamping.
    pub refresh_secs: Option<u64>,
    /// Failures since the last success.
    pub failures: u32,
    /// The stored copy's size in bytes.
    pub bytes: u64,
    /// A hash of the stored copy, part of every compiled engine's cache key.
    pub content_hash: Option<String>,
    /// Approximate rule counts of the stored copy.
    pub rules: RuleCounts,
    /// How the last attempt ended.
    pub last_result: Option<UpdateOutcome>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct StateFile {
    pub(crate) version: u32,
    pub(crate) lists: BTreeMap<SourceId, ListState>,
}

pub(crate) const STATE_VERSION: u32 = 1;

pub(crate) fn read_state(dir: &Path) -> BTreeMap<SourceId, ListState> {
    std::fs::read(dir.join("state.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<StateFile>(&bytes).ok())
        .filter(|s| s.version == STATE_VERSION)
        .map(|s| s.lists)
        .unwrap_or_default()
}

pub(crate) fn write_state(
    dir: &Path,
    lists: &BTreeMap<SourceId, ListState>,
) -> std::io::Result<()> {
    let file = StateFile {
        version: STATE_VERSION,
        lists: lists.clone(),
    };
    let json = serde_json::to_vec_pretty(&file).map_err(std::io::Error::other)?;
    write_atomic(&dir.join("state.json"), &json)
}

/// Write through a temporary file and a rename, so a reader never sees half a file.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

/// The path of a list's stored copy.
pub(crate) fn list_path(dir: &Path, id: &SourceId) -> std::path::PathBuf {
    dir.join(format!("{}.txt", id.file_stem()))
}

/// The interval in a list's `! Expires:` header line, as written (`4 days`, `12 hours`,
/// `1 days (update frequency)`). Only the comment lines before the first rule are read.
pub fn parse_expires(text: &str) -> Option<Duration> {
    for line in text.lines().take(64) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('[') {
            continue;
        }
        let Some(comment) = line.strip_prefix('!') else {
            // Hosts files use `#` comments; the first rule ends the header either way.
            if line.starts_with('#') {
                continue;
            }
            break;
        };
        let comment = comment.trim();
        let Some((key, value)) = comment.split_once(':') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case("expires") {
            continue;
        }
        let mut words = value.split_whitespace();
        let amount: u64 = words.next()?.parse().ok()?;
        let unit = words.next().unwrap_or("days").to_ascii_lowercase();
        let secs = if unit.starts_with('h') {
            amount.checked_mul(60 * 60)?
        } else if unit.starts_with('d') {
            amount.checked_mul(24 * 60 * 60)?
        } else {
            return None;
        };
        return Some(Duration::from_secs(secs));
    }
    None
}

/// The refresh interval for a list that asks for `expires`: clamped to [`MIN_REFRESH`] ..
/// [`MAX_REFRESH`], and [`DEFAULT_REFRESH`] when the list asks for nothing.
pub fn refresh_interval(expires: Option<Duration>) -> Duration {
    expires
        .unwrap_or(DEFAULT_REFRESH)
        .clamp(MIN_REFRESH, MAX_REFRESH)
}

/// The wait after `failures` consecutive failed downloads: [`FIRST_RETRY`], doubled per
/// further failure, at most [`MAX_RETRY`].
pub fn retry_delay(failures: u32) -> Duration {
    let doublings = failures.saturating_sub(1).min(16);
    FIRST_RETRY.saturating_mul(1 << doublings).min(MAX_RETRY)
}

/// Up to a tenth of `interval`, fixed per list and attempt, so many installs do not ask a list
/// server at the same second. Derived from a hash rather than a random source, which keeps the
/// schedule reproducible in tests.
pub(crate) fn jitter(id: &SourceId, at: u64, interval: Duration) -> u64 {
    let mut h = Fnv::new();
    h.write(id.as_str().as_bytes());
    h.write(&at.to_le_bytes());
    h.finish() % (interval.as_secs() / 10 + 1)
}

/// The format of `text`: a list declared as adblock syntax whose first rules all look like
/// hosts lines (`0.0.0.0 ads.example`) is read as a hosts file. Lists added by URL default to
/// adblock syntax, and a hosts file read that way would block nothing.
pub(crate) fn detect_format(text: &str, declared: ListFormat) -> ListFormat {
    if declared == ListFormat::Hosts {
        return declared;
    }
    let mut rules = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('!'))
        .take(20)
        .peekable();
    if rules.peek().is_none() {
        return declared;
    }
    let hosts = rules.all(|line| {
        let mut fields = line
            .split('#')
            .next()
            .unwrap_or_default()
            .split_whitespace();
        matches!(
            (fields.next(), fields.next(), fields.next()),
            (Some("0.0.0.0" | "127.0.0.1" | "::" | "::1"), Some(_), None)
        )
    });
    if hosts { ListFormat::Hosts } else { declared }
}

/// The list's own name, from a `! Title:` line among its first header lines.
pub(crate) fn list_title(text: &str) -> Option<String> {
    text.lines().take(40).map(str::trim).find_map(|line| {
        let rest = line.strip_prefix('!')?.trim_start();
        let title = rest.strip_prefix("Title:")?.trim();
        (!title.is_empty()).then(|| title.chars().take(80).collect())
    })
}

/// Count rules in a downloaded list, or say why it is not one.
pub(crate) fn scan_list(text: &str, format: ListFormat) -> Result<RuleCounts, String> {
    let format = detect_format(text, format);
    let head = text.trim_start_matches('\u{feff}').trim_start();
    if head.starts_with('<') {
        return Err("the response is an HTML page, not a filter list".into());
    }
    let mut counts = RuleCounts::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
            continue;
        }
        match format {
            ListFormat::Hosts => {
                if line.starts_with('#') {
                    continue;
                }
                let fields = line.split('#').next().unwrap_or_default();
                if (1..=2).contains(&fields.split_whitespace().count()) {
                    counts.network += 1;
                }
            }
            ListFormat::Adblock => {
                if line.starts_with("# ") || line == "#" {
                    continue;
                }
                if is_cosmetic(line) {
                    counts.cosmetic += 1;
                } else {
                    counts.network += 1;
                }
                if is_unsupported(line) {
                    counts.unsupported += 1;
                }
            }
        }
    }
    if counts.network == 0 && counts.cosmetic == 0 {
        return Err("the list has no rules".into());
    }
    Ok(counts)
}

/// A rule this blocker cannot apply: script injection (`##+js(…)`, `#%#`), CSS injection
/// (`#$#`), HTML filtering (`$$`, `##^`), and network options that need resources or rewrite
/// responses.
fn is_unsupported(line: &str) -> bool {
    const MARKERS: [&str; 6] = ["##+js(", "#@#+js(", "#%#", "#$#", "$$", "##^"];
    const OPTIONS: [&str; 6] = [
        "redirect=",
        "redirect-rule=",
        "replace=",
        "removeparam",
        "urltransform",
        "hls=",
    ];
    if MARKERS.iter().any(|m| line.contains(m)) {
        return true;
    }
    match line.rfind('$') {
        Some(at) if !is_cosmetic(line) => {
            let options = &line[at + 1..];
            options.split(',').any(|o| {
                OPTIONS
                    .iter()
                    .any(|known| o.trim_start_matches('~').starts_with(known))
            })
        }
        _ => false,
    }
}

fn is_cosmetic(line: &str) -> bool {
    ["##", "#@#", "#?#", "#@?#", "#$#", "#@$#", "#%#"]
        .iter()
        .any(|m| line.contains(m))
}

/// The result of one download attempt for one list.
pub(crate) enum Fetched {
    /// A new copy, already validated, with the state to store beside it.
    Updated { text: String, state: ListState },
    /// The stored copy is current.
    NotModified { state: ListState },
    /// Every URL failed.
    Failed { state: ListState },
}

/// Download `source`, revalidating the stored copy when there is one.
pub(crate) async fn fetch(
    client: &Client,
    source: &Source,
    previous: &ListState,
    have_copy: bool,
    now: u64,
    max_bytes: usize,
) -> Fetched {
    let mut last_error = String::from("the list has no download URL");
    for url in &source.urls {
        let url: &str = url;
        let mut request = Request::get(url)
            .header("Accept", "text/plain, */*;q=0.5")
            .timeout(Duration::from_secs(60));
        let same_url = previous.url.as_deref() == Some(url);
        if have_copy && same_url {
            request = request.conditional(&CacheValidators {
                etag: previous.etag.clone(),
                last_modified: previous.last_modified.clone(),
            });
        }
        let response = match client.fetch_limited_future(request, max_bytes).await {
            Ok(response) => response,
            Err(HttpError::BodyTooLarge { limit }) => {
                last_error = format!("{url}: the list is larger than {limit} bytes");
                continue;
            }
            Err(e) => {
                last_error = format!("{url}: {e}");
                continue;
            }
        };
        match response.status {
            304 if have_copy && same_url => {
                let mut state = previous.clone();
                let validators = CacheValidators {
                    etag: state.etag.clone(),
                    last_modified: state.last_modified.clone(),
                }
                .updated(&response);
                state.etag = validators.etag;
                state.last_modified = validators.last_modified;
                let interval = Duration::from_secs(
                    state
                        .refresh_secs
                        .unwrap_or(refresh_interval(None).as_secs()),
                );
                succeed(
                    &mut state,
                    &source.id,
                    now,
                    interval,
                    UpdateOutcome::NotModified,
                );
                return Fetched::NotModified { state };
            }
            200..=299 => {
                let text = String::from_utf8_lossy(&response.body).into_owned();
                let rules = match scan_list(&text, source.format) {
                    Ok(rules) => rules,
                    Err(e) => {
                        last_error = format!("{url}: {e}");
                        continue;
                    }
                };
                let validators = CacheValidators::default().updated(&response);
                let interval = refresh_interval(parse_expires(&text));
                let mut state = ListState {
                    url: Some(url.to_string()),
                    etag: validators.etag,
                    last_modified: validators.last_modified,
                    changed_at: Some(now),
                    refresh_secs: Some(interval.as_secs()),
                    bytes: text.len() as u64,
                    content_hash: Some(format!("{:016x}", crate::catalog::fnv1a(text.as_bytes()))),
                    rules,
                    ..ListState::default()
                };
                succeed(
                    &mut state,
                    &source.id,
                    now,
                    interval,
                    UpdateOutcome::Updated,
                );
                return Fetched::Updated { text, state };
            }
            status => {
                last_error = format!("{url}: HTTP {status}");
            }
        }
    }
    let mut state = previous.clone();
    state.failures = state.failures.saturating_add(1);
    let delay = retry_delay(state.failures);
    state.next_fetch = Some(now + delay.as_secs() + jitter(&source.id, now, delay));
    state.last_result = Some(UpdateOutcome::Failed { error: last_error });
    Fetched::Failed { state }
}

fn succeed(
    state: &mut ListState,
    id: &SourceId,
    now: u64,
    interval: Duration,
    outcome: UpdateOutcome,
) {
    state.fetched_at = Some(now);
    state.failures = 0;
    state.next_fetch = Some(now + interval.as_secs() + jitter(id, now, interval));
    state.last_result = Some(outcome);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expires_lines_parse_in_hours_and_days() {
        let list = "[Adblock Plus 2.0]\n! Version: 1\n! Expires: 4 days (update frequency)\n||a^";
        assert_eq!(parse_expires(list), Some(Duration::from_secs(4 * 86_400)));
        assert_eq!(
            parse_expires("! Expires: 1 hours\n||a^"),
            Some(Duration::from_secs(3_600))
        );
        assert_eq!(
            parse_expires("! Expires: 18 hours\n"),
            Some(Duration::from_secs(18 * 3_600))
        );
        // A rule ends the header.
        assert_eq!(parse_expires("||a^\n! Expires: 2 days\n"), None);
        assert_eq!(parse_expires("! Expires: soon\n"), None);
    }

    #[test]
    fn intervals_clamp_and_retries_back_off() {
        assert_eq!(refresh_interval(None), DEFAULT_REFRESH);
        assert_eq!(refresh_interval(Some(Duration::from_secs(60))), MIN_REFRESH);
        assert_eq!(
            refresh_interval(Some(Duration::from_secs(90 * 86_400))),
            MAX_REFRESH
        );
        assert_eq!(retry_delay(1), FIRST_RETRY);
        assert_eq!(retry_delay(2), FIRST_RETRY * 2);
        assert_eq!(retry_delay(3), FIRST_RETRY * 4);
        assert_eq!(retry_delay(40), MAX_RETRY);
    }

    #[test]
    fn jitter_stays_under_a_tenth() {
        let id = SourceId::EASYLIST;
        for at in 0..200 {
            assert!(jitter(&id, at, DEFAULT_REFRESH) <= DEFAULT_REFRESH.as_secs() / 10);
        }
    }

    #[test]
    fn html_and_empty_bodies_are_not_lists() {
        assert!(scan_list("<!doctype html><p>hi", ListFormat::Adblock).is_err());
        assert!(scan_list("! only comments\n", ListFormat::Adblock).is_err());
        let counts = scan_list("||a^\n##.b\n@@||c^\n", ListFormat::Adblock).unwrap();
        assert_eq!((counts.network, counts.cosmetic), (2, 1));
        let hosts = scan_list(
            "# c\n0.0.0.0 a.example\n127.0.0.1 b.example # x\n",
            ListFormat::Hosts,
        )
        .unwrap();
        assert_eq!(hosts.network, 2);
    }

    #[test]
    fn hosts_files_are_recognized_in_custom_lists() {
        let hosts = "# a hosts file\n127.0.0.1 localhost\n0.0.0.0 ads.example # x\n";
        assert_eq!(detect_format(hosts, ListFormat::Adblock), ListFormat::Hosts);
        assert_eq!(
            detect_format("||ads.example^\n0.0.0.0 b.example\n", ListFormat::Adblock),
            ListFormat::Adblock
        );
        assert_eq!(
            detect_format("! empty\n", ListFormat::Adblock),
            ListFormat::Adblock
        );
        let counts = scan_list(hosts, ListFormat::Adblock).unwrap();
        assert_eq!((counts.network, counts.cosmetic), (2, 0));
    }
}
