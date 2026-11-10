// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Address-bar completion: the inline completion typed text grows into, the suggestion list
//! under the bar, and the frecency score both rank by.

use crate::address::{self, Input};

/// A page the user has been to or saved, as completion sees it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Candidate {
    pub url: String,
    pub title: String,
    /// How many times the page was visited.
    pub visits: u32,
    /// How many of those visits started from the address bar (typed or chosen there).
    pub typed: u32,
    /// The last visit, in milliseconds since the Unix epoch. Zero for a bookmark never visited.
    pub last_visit: i64,
    pub bookmarked: bool,
}

const DAY_MS: i64 = 86_400_000;

/// How strongly a page is "yours": visits weighted by how recent the last one was, typed
/// visits counting three times, bookmarks a little extra. The recency buckets are Firefox's
/// frecency buckets (4, 14, 31 and 90 days).
pub fn frecency(c: &Candidate, now: i64) -> f64 {
    let age_days = (now - c.last_visit).max(0) / DAY_MS;
    let recency = match age_days {
        _ if c.last_visit == 0 => 20.0,
        0..=3 => 100.0,
        4..=13 => 70.0,
        14..=30 => 50.0,
        31..=89 => 30.0,
        _ => 10.0,
    };
    let weight = f64::from(c.visits.max(1)) + 2.0 * f64::from(c.typed);
    let bookmark = if c.bookmarked { 1.4 } else { 1.0 };
    weight * recency * bookmark
}

/// What the address bar grows typed text into: the full field text, how many characters of it
/// the user typed (the rest is shown selected), and the page Return loads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inline {
    pub text: String,
    pub typed_chars: usize,
    pub url: String,
}

impl Inline {
    /// The completed part, which the field shows selected after the caret.
    pub fn suffix(&self) -> String {
        self.text.chars().skip(self.typed_chars).collect()
    }
}

/// Whether an edit should be completed inline. Deleting (the new text is a strict prefix of
/// the old) never completes, so Backspace removes a completion rather than having it return;
/// repeating the same text does not either.
pub fn should_complete(previous: &str, typed: &str) -> bool {
    !(typed == previous || (previous.starts_with(typed) && typed.len() < previous.len()))
}

/// Compare `typed` against the start of `form` without regard to case, char by char, and
/// return the remainder of `form` when it extends `typed`.
fn extends<'a>(typed: &str, form: &'a str) -> Option<&'a str> {
    let mut rest = form;
    for t in typed.chars() {
        let mut fc = rest.chars();
        let f = fc.next()?;
        if !t.to_lowercase().eq(f.to_lowercase()) {
            return None;
        }
        rest = fc.as_str();
    }
    Some(rest)
}

struct Site {
    /// The host as visited, `www.` included when the site uses it.
    host: String,
    scheme: String,
    port: Option<u16>,
    score: f64,
}

/// The inline completion for `typed`, or `None`.
///
/// Typed text inside a host completes to that host, the way Safari's address bar does: `git`
/// becomes `github.com`, loading the site's front page, with the host's score the sum of every
/// visited page on it. A leading `www.` may be typed or left out. Once the text holds a `/`, it
/// completes to the best whole address that extends it. Typed case is kept; the completed rest
/// is the page's own spelling. Text with spaces, a leading `?`, or nothing to add is not
/// completed.
pub fn inline(typed: &str, candidates: &[Candidate], now: i64) -> Option<Inline> {
    if typed.is_empty() || typed.chars().any(char::is_whitespace) || typed.starts_with('?') {
        return None;
    }
    let lower = typed.to_lowercase();
    let scheme_prefix = ["https://", "http://"]
        .into_iter()
        .find(|p| lower.starts_with(p) || p.starts_with(&lower));
    // `ht` could be the start of `https://` or of a host such as `hts.example`: hosts first.
    let body: &str = match scheme_prefix {
        Some(p) if lower.starts_with(p) => &typed[p.len()..],
        _ => typed,
    };
    let typed_scheme = scheme_prefix.filter(|p| lower.starts_with(p));
    if typed_scheme.is_some() && body.is_empty() {
        return None;
    }

    if body.contains('/') {
        return complete_path(typed, body, typed_scheme, candidates, now);
    }

    let mut sites: Vec<Site> = Vec::new();
    for c in candidates {
        let Ok(url) = url::Url::parse(&c.url) else {
            continue;
        };
        if !matches!(url.scheme(), "http" | "https") {
            continue;
        }
        let Some(host) = url.host_str() else { continue };
        let score = frecency(c, now);
        match sites
            .iter_mut()
            .find(|s| s.host.eq_ignore_ascii_case(host) && s.port == url.port())
        {
            Some(site) => {
                site.score += score;
                if url.scheme() == "https" {
                    site.scheme = "https".into();
                }
            }
            None => sites.push(Site {
                host: host.to_ascii_lowercase(),
                scheme: url.scheme().into(),
                port: url.port(),
                score,
            }),
        }
    }
    sites.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.host.cmp(&b.host)));
    for site in &sites {
        if let Some(want) = typed_scheme
            && want.trim_end_matches("://") != site.scheme
        {
            continue;
        }
        let bare = site.host.strip_prefix("www.").unwrap_or(&site.host);
        let mut forms = vec![bare.to_string()];
        if bare != site.host {
            forms.push(site.host.clone());
        }
        if let Some(port) = site.port {
            forms = forms.into_iter().map(|f| format!("{f}:{port}")).collect();
        }
        for form in forms {
            if let Some(rest) = extends(body, &form) {
                if rest.is_empty() {
                    return None;
                }
                let port = site.port.map(|p| format!(":{p}")).unwrap_or_default();
                return Some(Inline {
                    text: format!("{typed}{rest}"),
                    typed_chars: typed.chars().count(),
                    url: format!("{}://{}{port}/", site.scheme, site.host),
                });
            }
        }
    }
    None
}

fn complete_path(
    typed: &str,
    body: &str,
    typed_scheme: Option<&str>,
    candidates: &[Candidate],
    now: i64,
) -> Option<Inline> {
    let mut ranked: Vec<(&Candidate, f64)> =
        candidates.iter().map(|c| (c, frecency(c, now))).collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.url.cmp(&b.0.url)));
    for (c, _) in ranked {
        if let Some(want) = typed_scheme
            && !c.url.starts_with(want)
        {
            continue;
        }
        let after_scheme = c.url.split_once("://").map_or(c.url.as_str(), |(_, r)| r);
        let bare = after_scheme.strip_prefix("www.").unwrap_or(after_scheme);
        for form in [bare, after_scheme] {
            if let Some(rest) = extends(body, form) {
                if rest.is_empty() {
                    return None;
                }
                return Some(Inline {
                    text: format!("{typed}{rest}"),
                    typed_chars: typed.chars().count(),
                    url: c.url.clone(),
                });
            }
        }
    }
    None
}

/// Where Up or Down moves the highlight in a list of `len` rows: from nothing, Down takes the
/// first row and Up the last; past either end the highlight goes (`None`), and the address bar
/// shows what was typed again, as DuckDuckGo's does.
pub fn step_highlight(len: usize, current: Option<usize>, down: bool) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match (current, down) {
        (None, true) => Some(0),
        (None, false) => Some(len - 1),
        (Some(i), true) if i + 1 < len => Some(i + 1),
        (Some(i), false) if i > 0 && i <= len => Some(i - 1),
        _ => None,
    }
}

/// What a suggestion row is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowKind {
    /// The best match, which the inline completion also points at.
    TopHit,
    History,
    Bookmark,
    /// The page is open in another tab; choosing it switches to that tab.
    OpenTab,
}

/// One row of the suggestion list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Row {
    /// A page: from history, bookmarks or an open tab.
    Page {
        url: String,
        title: String,
        kind: RowKind,
    },
    /// Search for these terms. `suggested` marks the engine's suggestions, as opposed to the
    /// typed text itself.
    Search { terms: String, suggested: bool },
    /// Go to the address typed.
    Visit { url: String },
}

/// How many of each kind of row the list holds.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub remote: usize,
    pub pages: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            remote: 4,
            pages: 5,
        }
    }
}

/// Whether every word of `query` occurs in the page's title or address.
fn matches_words(words: &[String], c: &Candidate) -> bool {
    let title = c.title.to_lowercase();
    let url = address::strip_scheme(&c.url).to_lowercase();
    words.iter().all(|w| title.contains(w) || url.contains(w))
}

/// The suggestion list for `typed`.
///
/// Order: the top hit (the inline completion's page, or the typed address itself when it is
/// one), a search for the typed text, the engine's suggestions, then matching pages from
/// history and bookmarks, best first, where each typed word must appear in the page's title or
/// address. A page open in another tab is offered as a switch to that tab. Each page appears
/// once.
pub fn rows(
    typed: &str,
    inline: Option<&Inline>,
    candidates: &[Candidate],
    open_tabs: &[String],
    remote: &[String],
    now: i64,
    limits: Limits,
) -> Vec<Row> {
    let typed = typed.trim();
    if typed.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<Row> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let key = |url: &str| address::history_key(url).unwrap_or_else(|| url.to_string());
    let open = |url: &str| open_tabs.iter().any(|t| key(t) == key(url));

    let classified = address::classify(typed);
    if let Some(hit) = inline {
        let title = candidates
            .iter()
            .filter(|c| key(&c.url) == key(&hit.url))
            .map(|c| c.title.clone())
            .next()
            .unwrap_or_default();
        out.push(Row::Page {
            url: hit.url.clone(),
            title,
            kind: RowKind::TopHit,
        });
        seen.push(key(&hit.url));
    } else if let Some(Input::Navigate(url)) = &classified {
        out.push(Row::Visit { url: url.clone() });
        seen.push(key(url));
    }
    let terms = match &classified {
        Some(Input::Search(t)) => t.clone(),
        _ => typed.to_string(),
    };
    out.push(Row::Search {
        terms: terms.clone(),
        suggested: false,
    });
    for s in remote
        .iter()
        .filter(|s| !s.eq_ignore_ascii_case(&terms))
        .take(limits.remote)
    {
        out.push(Row::Search {
            terms: s.clone(),
            suggested: true,
        });
    }

    let words: Vec<String> = typed
        .trim_start_matches('?')
        .to_lowercase()
        .split_whitespace()
        .map(|w| {
            w.trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_start_matches("www.")
                .to_string()
        })
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        return out;
    }
    let lower = words.join(" ");
    let mut pages: Vec<(&Candidate, f64)> = candidates
        .iter()
        .filter(|c| matches_words(&words, c))
        .map(|c| {
            let mut score = frecency(c, now);
            if address::strip_scheme(&c.url)
                .to_lowercase()
                .starts_with(&lower)
            {
                score *= 1.5;
            }
            (c, score)
        })
        .collect();
    pages.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.url.cmp(&b.0.url)));
    let mut added = 0;
    for (c, _) in pages {
        if added >= limits.pages {
            break;
        }
        let k = key(&c.url);
        if seen.contains(&k) {
            continue;
        }
        seen.push(k);
        let kind = if open(&c.url) {
            RowKind::OpenTab
        } else if c.bookmarked {
            RowKind::Bookmark
        } else {
            RowKind::History
        };
        out.push(Row::Page {
            url: c.url.clone(),
            title: c.title.clone(),
            kind,
        });
        added += 1;
    }
    out
}

/// The sites the start page's Frequently Visited grid shows: one entry per site (its most
/// frecent page's title, the site's front page if it was visited, else that page), sites
/// ranked by their summed frecency, bookmarked sites left out since Favorites already shows
/// them.
pub fn top_sites(candidates: &[Candidate], now: i64, count: usize) -> Vec<Candidate> {
    struct Entry {
        site: String,
        score: f64,
        best: Candidate,
        best_score: f64,
        front: Option<Candidate>,
        bookmarked: bool,
    }
    let mut entries: Vec<Entry> = Vec::new();
    for c in candidates {
        if c.visits == 0 {
            continue;
        }
        let Some(site) = address::site(&c.url) else {
            continue;
        };
        let score = frecency(c, now);
        let is_front =
            url::Url::parse(&c.url).is_ok_and(|u| u.path() == "/" && u.query().is_none());
        match entries.iter_mut().find(|e| e.site == site) {
            Some(e) => {
                e.score += score;
                e.bookmarked |= c.bookmarked;
                if score > e.best_score {
                    e.best = c.clone();
                    e.best_score = score;
                }
                if is_front {
                    e.front = Some(c.clone());
                }
            }
            None => entries.push(Entry {
                site,
                score,
                best: c.clone(),
                best_score: score,
                front: is_front.then(|| c.clone()),
                bookmarked: c.bookmarked,
            }),
        }
    }
    entries.retain(|e| !e.bookmarked);
    entries.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.site.cmp(&b.site)));
    entries
        .into_iter()
        .take(count)
        .map(|e| e.front.unwrap_or(e.best))
        .collect()
}
