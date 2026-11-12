// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Tab rules: where a new tab goes, the recently closed list, and the saved session that
//! reopens the tabs at the next launch.

use serde::{Deserialize, Serialize};

/// Where a new tab goes in `order`.
///
/// A tab opened from a page (a link opened in a new tab) goes right after the tab it was opened
/// from, after any tabs already opened from it, so a run of links opened from one page reads
/// left to right in the order they were opened, as in Chrome and Safari. `last_child` is the
/// most recent tab opened from `opener`, while that run is unbroken. A tab opened from the new
/// tab button or the menu (`opener` = `None`) goes at the end.
pub fn insertion_index<K: PartialEq>(
    order: &[K],
    opener: Option<&K>,
    last_child: Option<&K>,
) -> usize {
    let Some(opener) = opener else {
        return order.len();
    };
    let Some(at) = order.iter().position(|k| k == opener) else {
        return order.len();
    };
    if let Some(child) = last_child
        && let Some(c) = order.iter().position(|k| k == child)
        && c > at
    {
        return c + 1;
    }
    at + 1
}

/// Tracks the run of tabs opened from one page, for [`insertion_index`]. The run ends when the
/// user selects a different tab or opens a tab without an opener.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenerRun<K> {
    opener: Option<K>,
    last_child: Option<K>,
}

impl<K: Clone + PartialEq> OpenerRun<K> {
    pub fn new() -> Self {
        OpenerRun {
            opener: None,
            last_child: None,
        }
    }

    /// The last child opened from `opener` in the current run.
    pub fn last_child(&self, opener: &K) -> Option<&K> {
        (self.opener.as_ref() == Some(opener))
            .then_some(self.last_child.as_ref())
            .flatten()
    }

    /// Record that `child` was opened from `opener`. A foreground open selects the child, which
    /// ends the run (the selection moved away from the opener).
    pub fn opened(&mut self, opener: Option<&K>, child: &K, foreground: bool) {
        match opener {
            Some(o) if !foreground => {
                self.opener = Some(o.clone());
                self.last_child = Some(child.clone());
            }
            _ => self.reset(),
        }
    }

    /// Selecting `key` keeps the run only while the opener stays selected.
    pub fn selected(&mut self, key: &K) {
        if self.opener.as_ref() != Some(key) {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        self.opener = None;
        self.last_child = None;
    }
}

/// A closed tab, kept so Reopen Closed Tab can bring it back where it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosedTab {
    pub url: String,
    pub title: String,
    /// Its position in the tab order when it closed.
    pub index: usize,
}

/// The recently closed tabs, newest last, bounded. Private tabs and blank tabs are never kept.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClosedTabs {
    entries: Vec<ClosedTab>,
}

impl ClosedTabs {
    /// How many closed tabs are remembered.
    pub const CAPACITY: usize = 25;

    pub fn new() -> Self {
        Self::default()
    }

    /// Remember a closed tab. Returns whether it was kept.
    pub fn push(&mut self, tab: ClosedTab, private: bool) -> bool {
        if private || tab.url.is_empty() || tab.url == "about:blank" {
            return false;
        }
        self.entries.push(tab);
        if self.entries.len() > Self::CAPACITY {
            self.entries.remove(0);
        }
        true
    }

    /// The most recently closed tab, removed from the list.
    pub fn pop(&mut self) -> Option<ClosedTab> {
        self.entries.pop()
    }

    /// Newest first, for a Recently Closed menu.
    pub fn recent(&self) -> impl Iterator<Item = &ClosedTab> {
        self.entries.iter().rev()
    }

    /// Remove and return the entry `newest_first_index` places from the newest.
    pub fn take(&mut self, newest_first_index: usize) -> Option<ClosedTab> {
        let len = self.entries.len();
        (newest_first_index < len).then(|| self.entries.remove(len - 1 - newest_first_index))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// One tab in a saved session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedTab {
    pub url: String,
    #[serde(default)]
    pub title: String,
    /// When the tab was last looked at, in milliseconds since the Unix epoch; `0` when unknown
    /// (a session saved before this was recorded), which never expires.
    #[serde(default)]
    pub viewed: i64,
}

/// The tabs to reopen at the next launch: their pages in order, and which was selected.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub tabs: Vec<SavedTab>,
    #[serde(default)]
    pub selected: usize,
}

impl Session {
    /// A session from the open tabs as `(url, title, private, viewed)`, leaving private tabs out
    /// (they end with the app) and moving the selection to stay on the same tab where it can.
    pub fn capture<'a>(
        tabs: impl IntoIterator<Item = (&'a str, &'a str, bool, i64)>,
        selected: Option<usize>,
    ) -> Session {
        let mut out = Vec::new();
        // The selected tab's position among the kept ones, or the nearest kept tab before it
        // when the selected tab was private.
        let mut pick = 0;
        for (i, (url, title, private, viewed)) in tabs.into_iter().enumerate() {
            if private {
                continue;
            }
            if selected.is_some_and(|s| i <= s) {
                pick = out.len();
            }
            out.push(SavedTab {
                url: url.to_string(),
                title: title.to_string(),
                viewed,
            });
        }
        Session {
            selected: pick.min(out.len().saturating_sub(1)),
            tabs: out,
        }
    }

    /// Safari's Close Tabs setting: drop the tabs not looked at within `max_age_ms` of `now`.
    /// The selected tab always stays, and so does a tab whose viewing time is unknown; the
    /// selection follows its tab to its new position.
    pub fn expire(&mut self, now: i64, max_age_ms: i64) {
        let selected = self.selected;
        let mut kept = Vec::new();
        let mut pick = 0;
        for (i, tab) in std::mem::take(&mut self.tabs).into_iter().enumerate() {
            let stale = tab.viewed > 0 && now.saturating_sub(tab.viewed) > max_age_ms;
            if i == selected {
                pick = kept.len();
            } else if stale {
                continue;
            }
            kept.push(tab);
        }
        self.selected = pick.min(kept.len().saturating_sub(1));
        self.tabs = kept;
    }

    pub fn encode(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// A saved session, or an empty one when the text is missing or unreadable.
    pub fn decode(text: &str) -> Session {
        let mut s: Session = serde_json::from_str(text).unwrap_or_default();
        if s.selected >= s.tabs.len() {
            s.selected = s.tabs.len().saturating_sub(1);
        }
        s
    }
}

/// The tab to select when the selected tab `closing` closes, following DuckDuckGo's macOS
/// order so a page opened from another hands the selection back where it came from: the next
/// tab opened from the same page, the previous one, the page it was opened from, the next tab
/// opened from the closing one, the previous such tab, then the right neighbor, then the left.
/// `None` when no other tab remains. `opener_of` answers for any key in `order`.
pub fn select_after_close<K: Clone + PartialEq>(
    order: &[K],
    closing: &K,
    opener_of: impl Fn(&K) -> Option<K>,
) -> Option<K> {
    let at = order.iter().position(|k| k == closing)?;
    let parent = opener_of(closing).filter(|p| order.contains(p) && p != closing);
    let right = || order[at + 1..].iter();
    let left = || order[..at].iter().rev();
    if let Some(parent) = &parent {
        let sibling = |k: &&K| opener_of(k).as_ref() == Some(parent);
        if let Some(k) = right().find(sibling).or_else(|| left().find(sibling)) {
            return Some(k.clone());
        }
        return Some(parent.clone());
    }
    let child = |k: &&K| opener_of(k).as_ref() == Some(closing);
    right()
        .find(child)
        .or_else(|| left().find(child))
        .or_else(|| right().next())
        .or_else(|| left().next())
        .cloned()
}

/// The progress line under the address bar: it appears at 10% the moment a load starts (the
/// engine reports nothing useful until the first response), only ever moves forward through a
/// load's redirects, and reports complete before it hides.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LoadProgress {
    /// 0.0–1.0.
    pub value: f64,
    pub visible: bool,
}

impl LoadProgress {
    /// The value a starting load shows.
    pub const START: f64 = 0.1;

    /// Fold the engine's report into the line: `loading` and its estimate, 0.0–1.0.
    pub fn report(self, loading: bool, estimate: f64) -> LoadProgress {
        let estimate = if estimate.is_finite() {
            estimate.clamp(0.0, 1.0)
        } else {
            0.0
        };
        match (loading, self.visible) {
            (true, false) => LoadProgress {
                value: estimate.max(Self::START),
                visible: true,
            },
            (true, true) => LoadProgress {
                value: self.value.max(estimate),
                visible: true,
            },
            (false, true) => LoadProgress {
                value: 1.0,
                visible: false,
            },
            (false, false) => self,
        }
    }
}

/// The tabs whose web views to let go so that at most `keep` stay live: the ones used longest
/// ago, never the selected tab. `views` is every tab that holds a web view, with when it was
/// last used. A phone keeps a handful of pages in memory and reloads the rest when the user
/// returns to them, as Safari and Chrome do.
pub fn views_to_unload<K: Clone + PartialEq>(
    views: &[(K, i64)],
    selected: Option<&K>,
    keep: usize,
) -> Vec<K> {
    let excess = views.len().saturating_sub(keep.max(1));
    let mut others: Vec<&(K, i64)> = views
        .iter()
        .filter(|(key, _)| Some(key) != selected)
        .collect();
    others.sort_by_key(|(_, used)| *used);
    others
        .into_iter()
        .take(excess)
        .map(|(key, _)| key.clone())
        .collect()
}

/// A byte count as people read one: `512 bytes`, `1.5 KB`, `20 MB`, `3.2 GB`, in powers of
/// 1000 as the stores and file managers show sizes. One decimal below ten of a unit.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if bytes < 1000 {
        return format!("{bytes} bytes");
    }
    let mut value = bytes as f64 / 1000.0;
    let mut unit = 0;
    while value >= 999.5 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    if value < 9.95 {
        format!("{value:.1} {}", UNITS[unit])
    } else {
        format!("{value:.0} {}", UNITS[unit])
    }
}
