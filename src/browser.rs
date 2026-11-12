// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! One window's browser: its tabs, what each tab shows, and the address bar's editing state.
//!
//! The tabs are a Day [`TabSet`] of string keys; `document_tabs` keeps every tab's web view
//! alive while it is open, so switching tabs never reloads a page. Each tab's state ([`Tab`])
//! is created inside its own document scope and registered here while it lives.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use day::prelude::*;
use day_piece_webview::{JsHandle, LoadError, LoadErrorKind, PageState, ScrollState};
use netskip_model::address::{self, Input};
use netskip_model::complete::{self, Candidate, Inline, Limits, Row, RowKind};
use netskip_model::history;
use netskip_model::search::Engine;
use netskip_model::tabs::{ClosedTab, ClosedTabs, LoadProgress, OpenerRun, Session};

use crate::{fixtures, library, scripts, settings, store};

/// The preference key the open tabs are saved under between launches.
const SESSION_KEY: &str = "netskip.session";

/// A page's favicon, PNG bytes. Compared by identity so a signal holding one is cheap.
#[derive(Clone)]
pub struct Icon(pub Rc<[u8]>);
impl PartialEq for Icon {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// Find in page: the bar's query and the match it shows.
#[derive(Clone, Copy)]
pub struct Find {
    pub open: Signal<bool>,
    pub query: Signal<String>,
    /// (current match, 1-based; total matches).
    pub matches: Signal<(u32, u32)>,
}

/// Why a tab shows an error page instead of its web page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageErrorKind {
    /// The page did not load.
    Load(LoadErrorKind),
    /// The page's process ended twice in a row, so it was not reloaded again.
    Crashed,
}

/// The error page a tab shows, and the address it is about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageError {
    pub kind: PageErrorKind,
    /// The real address that failed.
    pub url: String,
}

/// One tab. `Copy`: it holds handles, not values.
#[derive(Clone, Copy)]
pub struct Tab {
    pub id: u64,
    pub private: bool,
    /// The tab the page was opened from, for placing and closing.
    pub opener: Option<u64>,
    /// The address the web view loads and reports: the physical one in fixture mode.
    pub location: Signal<String>,
    pub title: Signal<String>,
    pub icon: Signal<Option<Icon>>,
    pub can_back: Signal<bool>,
    pub can_forward: Signal<bool>,
    pub progress: Signal<LoadProgress>,
    /// The start page shows over the web view.
    pub start: Signal<bool>,
    /// The tab has a web view. A tab opened on the start page gets one with its first page,
    /// so a stack of new tabs costs no web engine instances.
    pub has_view: Signal<bool>,
    /// The tab's history began at the start page, so Back from its first page returns there.
    pub from_start: Signal<bool>,
    /// The next committed load came from the address bar (it counts as typed in history).
    pub typed: Signal<bool>,
    /// The page zoom, in percent.
    pub zoom: Signal<i64>,
    /// The tab asks sites for their desktop pages (Request Desktop Website).
    pub desktop: Signal<bool>,
    pub find: Find,
    pub go: Trigger,
    pub back: Trigger,
    pub forward: Trigger,
    pub reload: Trigger,
    pub stop: Trigger,
    pub js: JsHandle,
    /// Bumped whenever a load starts or ends, so the page-state poll restarts.
    pub loads: Signal<u64>,
    /// The page snapshot the tab overview shows, PNG bytes.
    pub snapshot: Signal<Option<Icon>>,
    /// Where the page is scrolled, on the platforms that report it (iOS, Android).
    pub scroll: Signal<ScrollState>,
    /// Requests content blocking stopped on this page, where the engine reports them.
    pub blocked: Signal<usize>,
    /// What the engine reports about the page: title, progress, back/forward availability.
    pub page: Signal<PageState>,
    /// The error page showing over the web view, until the next load.
    pub error: Signal<Option<PageError>>,
    /// Where to scroll the page once it has loaded again, after its web view was let go.
    pub restore_scroll: Signal<f64>,
    /// When the tab was last shown or given a web view (ms), which decides whose web view is
    /// let go first.
    pub used: Signal<i64>,
    /// When the page's process last ended (ms), to tell one crash from a page that keeps crashing.
    pub crashed_at: Signal<i64>,
    /// The `http://` address the user typed, while its upgrade to HTTPS loads: where to go
    /// when the site turns out to have no HTTPS.
    pub http_fallback: Signal<Option<String>>,
}

impl Tab {
    /// The page's real address, as the user and history see it.
    pub fn url(self) -> String {
        fixtures::to_logical(&self.location.get())
    }

    pub fn url_untracked(self) -> String {
        fixtures::to_logical(&self.location.get_untracked())
    }

    /// Whether a page is showing rather than the start page or a blank view.
    pub fn has_page(self) -> bool {
        !self.start.get() && !is_blank(&self.location.get())
    }

    /// The tab's title for the strip and the overview: the page title, its site, or New Tab.
    pub fn label(self) -> String {
        if self.start.get() || is_blank(&self.location.get()) {
            return if self.private {
                res::str::private_tab().format()
            } else {
                res::str::new_tab().format()
            };
        }
        let title = self.title.get();
        if title.trim().is_empty() {
            address::display(&self.url(), false)
        } else {
            title
        }
    }

    /// Load a page's real address in this tab.
    pub fn load(self, url: &str) {
        let physical = fixtures::to_physical(url);
        batch(|| {
            if self.start.get_untracked() && is_blank(&self.location.get_untracked()) {
                self.from_start.set(true);
            }
            self.start.set(false);
            self.find.open.set(false);
            self.error.set(None);
            self.restore_scroll.set(0.0);
            self.http_fallback.set(None);
            self.location.set(physical);
            self.progress
                .set(self.progress.get_untracked().report(true, 0.0));
            self.loads.update(|n| *n += 1);
        });
        if self.has_view.get_untracked() {
            self.go.notify();
        } else {
            // The web view is built now and loads the address it is created with.
            self.has_view.set(true);
        }
    }

    pub fn go_back(self) {
        if self.start.get_untracked() {
            return;
        }
        self.error.set(None);
        if self.can_back.get_untracked() || !self.from_start.get_untracked() {
            self.back.notify();
            self.loads.update(|n| *n += 1);
            return;
        }
        // The cached answer may predate the last navigation: ask the engine before deciding
        // between the page's own history and the start page this tab began on.
        day::task(async move {
            let back = match self.js.navigation_state().await {
                Ok(state) => state.can_go_back,
                Err(_) => false,
            };
            if back {
                self.can_back.set(true);
                self.back.notify();
                self.loads.update(|n| *n += 1);
            } else {
                self.start.set(true);
            }
        });
    }

    pub fn go_forward(self) {
        if self.start.get_untracked() && !is_blank(&self.location.get_untracked()) {
            self.start.set(false);
            self.wake();
            return;
        }
        self.error.set(None);
        self.forward.notify();
        self.loads.update(|n| *n += 1);
    }

    pub fn can_go_back(self) -> bool {
        !self.start.get() && (self.can_back.get() || self.from_start.get())
    }

    pub fn can_go_forward(self) -> bool {
        self.can_forward.get() || (self.start.get() && !is_blank(&self.location.get()))
    }

    pub fn loading(self) -> bool {
        self.progress.get().visible
    }

    pub fn reload_or_stop(self) {
        if self.progress.get_untracked().visible {
            self.stop.notify();
            self.progress
                .set(self.progress.get_untracked().report(false, 1.0));
        } else if !self.start.get_untracked() {
            self.error.set(None);
            self.reload.notify();
            self.progress
                .set(self.progress.get_untracked().report(true, 0.0));
            self.loads.update(|n| *n += 1);
        }
    }

    /// Whether the tab's page is not in memory: it has an address and no web view.
    pub fn asleep(self) -> bool {
        !self.has_view.get_untracked() && !is_blank(&self.location.get_untracked())
    }

    /// Let the tab's web view go, keeping its address, title, picture and scroll position.
    /// The page loads again when the tab is next shown ([`Tab::wake`]). Its back and forward
    /// history goes with the web view.
    pub fn unload(self) {
        if !self.has_view.get_untracked() {
            return;
        }
        batch(|| {
            self.restore_scroll.set(self.scroll.get_untracked().offset);
            self.has_view.set(false);
            self.can_back.set(false);
            self.can_forward.set(false);
            self.find.open.set(false);
            self.progress.set(LoadProgress::default());
        });
    }

    /// Give the tab its web view back and load its page again.
    pub fn wake(self) {
        if !self.asleep() || self.start.get_untracked() {
            return;
        }
        batch(|| {
            self.error.set(None);
            self.used.set(crate::now_ms());
            self.progress
                .set(self.progress.get_untracked().report(true, 0.0));
            self.loads.update(|n| *n += 1);
            self.has_view.set(true);
        });
    }

    /// The error page's Try Again: load the address that failed again. (A reload would not do:
    /// after a load fails, WebKit's current page is still the one before it.) A tab whose web
    /// view is gone gets it back, which loads the page.
    pub fn retry(self) {
        let Some(error) = self.error.get_untracked() else {
            return;
        };
        if self.has_view.get_untracked() && !error.url.is_empty() {
            self.load(&error.url);
        } else {
            self.error.set(None);
            self.wake();
        }
    }

    /// Set the page zoom (the web view applies it natively) and remember it for the page's
    /// site, as Safari does: the site opens at that zoom from then on.
    pub fn set_zoom(self, percent: i64) {
        let percent = percent.clamp(50, 300);
        self.zoom.set(percent);
        if let Some(site) = address::site(&self.url_untracked()) {
            settings::remember_site_zoom(&site, percent);
        }
    }
}

/// Whether the web view shows nothing yet.
pub fn is_blank(url: &str) -> bool {
    url.is_empty() || url == "about:blank"
}

/// What a tab starts with, kept until its document scope builds the [`Tab`].
#[derive(Clone, Debug)]
struct Seed {
    url: String,
    title: String,
    private: bool,
    opener: Option<u64>,
    /// A tab restored from the last session: its page loads when the tab is first shown.
    lazy: bool,
}

/// How many web views a window keeps in memory. A phone or tablet reloads the pages of the
/// tabs used longest ago when the user returns to them; a computer has room for more.
const LIVE_VIEWS: usize = if cfg!(any(
    target_os = "ios",
    target_os = "android",
    target_env = "ohos"
)) {
    8
} else {
    32
};

/// Two ends of a page's process closer together than this (ms) are a page that keeps
/// crashing, and it is not reloaded a second time.
const CRASH_LOOP_MS: i64 = 15_000;

/// One window's browser. `Copy`: handles only.
#[derive(Clone, Copy)]
pub struct Browser {
    pub tabs: TabSet<String>,
    serial: Signal<u64>,
    seeds: Signal<HashMap<u64, Seed>>,
    live: Signal<HashMap<u64, Tab>>,
    closed: Signal<Rc<RefCell<ClosedTabs>>>,
    closed_count: Signal<usize>,
    /// The phone's bars are tucked away while the user reads down a page, as Safari and
    /// Chrome do; scrolling up, reaching either end, or tapping the bar brings them back.
    pub chrome_hidden: Signal<bool>,
    /// When each tab was last looked at (ms since the epoch), for Settings' Close Tabs.
    viewed: Signal<HashMap<String, i64>>,
    run: Signal<Rc<RefCell<OpenerRun<u64>>>>,
    /// The tab overview (the grid of tabs) is showing.
    pub overview: Signal<bool>,
    /// The library (bookmarks and history) is showing, on this tab of it.
    pub library: Signal<Option<library::Pane>>,
    /// The sign-in sheet is showing, for the server in `sign_in_request`.
    pub sign_in: Signal<Option<crate::chrome::SignIn>>,
    pub sign_in_request: Signal<Option<day_piece_webview::AuthRequest>>,
    pub omnibox: Omnibox,
    /// This is the app's first window, which restores and saves the session.
    primary: bool,
}

impl Ambient for Browser {
    fn create() -> Self {
        let browser = Browser {
            tabs: TabSet::new(),
            serial: Signal::new(0),
            seeds: Signal::new(HashMap::new()),
            live: Signal::new(HashMap::new()),
            closed: Signal::new(Rc::new(RefCell::new(ClosedTabs::new()))),
            closed_count: Signal::new(0),
            chrome_hidden: Signal::new(false),
            viewed: Signal::new(HashMap::new()),
            run: Signal::new(Rc::new(RefCell::new(OpenerRun::new()))),
            overview: Signal::new(false),
            library: Signal::new(None),
            sign_in: Signal::new(None),
            sign_in_request: Signal::new(None),
            omnibox: Omnibox::new(),
            // Fixture runs neither restore nor save a session: each script starts from one tab.
            primary: PRIMARY.with(|p| !p.replace(true)) && !fixtures::active(),
        };
        browser.restore();
        // Slide the phone's bars away while the user scrolls down a page with a finger, and
        // back when they scroll up, reach either end, or switch tabs. Once the bars move, the
        // same drag cannot move them back for a moment: the page shifts under the finger as
        // they go, and some engines scroll to keep up with it.
        let moved_at = std::cell::Cell::new(i64::MIN);
        watch(
            move || browser.current().map(|t| (t.id, t.scroll.get())),
            move |now, before| {
                let hidden = browser.chrome_hidden.get_untracked();
                let hide = match (now.as_ref(), before.and_then(|b| b.as_ref())) {
                    (Some((id, s)), Some((was, b))) if id == was => {
                        let settling = s.dragging
                            && crate::now_ms().saturating_sub(moved_at.get()) < BARS_SETTLE_MS;
                        if settling {
                            hidden
                        } else {
                            bars_hidden(s, b, hidden)
                        }
                    }
                    _ => false,
                };
                if hide != hidden {
                    moved_at.set(crate::now_ms());
                    with_animation(AnimSpec::ease_out(220), || browser.chrome_hidden.set(hide));
                }
            },
        );
        // The opener run ends when the user picks another tab. Both the tab left and the tab
        // shown count as looked at now (Close Tabs ages tabs from the last time they showed).
        watch(
            move || browser.tabs.selected(),
            move |selected, before| {
                if let Some(key) = selected.as_ref().and_then(|k| k.parse::<u64>().ok()) {
                    browser.run.get_untracked().borrow_mut().selected(&key);
                }

                let now = crate::now_ms();
                // Read before `update`: a signal read inside another's update closure re-enters
                // the reactive runtime.
                let open = untrack(|| browser.tabs.keys());
                browser.viewed.update(|v| {
                    for key in [selected.clone(), before.cloned().flatten()]
                        .into_iter()
                        .flatten()
                    {
                        v.insert(key, now);
                    }
                    v.retain(|k, _| open.contains(k));
                });
                browser.omnibox.end_editing(browser);
                // The tab now showing gets its page back if it was let go, and the views
                // used longest ago make room for it.
                if let Some(tab) = browser.current_untracked() {
                    tab.used.set(now);
                    tab.wake();
                }
                browser.trim_views(LIVE_VIEWS);
            },
        );
        // A tab that loads its first page takes a web view too.
        watch(
            move || {
                browser
                    .live
                    .with(|m| m.values().filter(|t| t.has_view.get()).count())
            },
            move |views, _| {
                if *views > LIVE_VIEWS {
                    browser.trim_views(LIVE_VIEWS);
                }
            },
        );
        // A web view is told what to call itself when it is built. A new user agent lets every
        // tab's view go, and the one showing is built again, which loads its page with it.
        watch(settings::user_agent, move |_, before| {
            if before.is_none() {
                return;
            }
            browser.trim_views(0);
            if let Some(tab) = browser.current_untracked() {
                tab.unload();
                day::task(async move {
                    day::sleep(0).await;
                    tab.wake();
                });
            }
        });
        // Under memory pressure only the page on screen stays in memory.
        day::on_lifecycle(Lifecycle::DidReceiveMemoryWarning, move || {
            browser.trim_views(1);
        });
        if browser.primary {
            // Save the session whenever a tab opens, closes, moves or loads another page.
            watch(
                move || browser.session(),
                move |session, _| {
                    day::prefs::set(SESSION_KEY, &session.encode());
                },
            );
        }
        browser
    }
}

thread_local! {
    static PRIMARY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// How long, in milliseconds, a drag cannot undo the bars it just moved.
const BARS_SETTLE_MS: i64 = 350;

/// Whether the phone's bars should be hidden after the page scrolled from `before` to `now`:
/// hidden by a drag down more than a few points once the page is past the top, shown by a move
/// up, at the top, at the bottom once the finger lifts, or on a short page; otherwise as they
/// were.
///
/// A report whose viewport changed height is the bars' own doing and changes nothing. Hiding a
/// bar above the page moves the page's top edge under the finger, and Android's WebView scrolls
/// the page back by the bar's height to keep it under the finger; read as a drag up, that
/// showed the bars again, which moved the page back down, many times a second.
pub fn bars_hidden(now: &ScrollState, before: &ScrollState, hidden: bool) -> bool {
    const SLOP: f64 = 6.0;
    // Past any bar's height, so the scroll back as the bar goes never reaches the top.
    const HIDE_AFTER: f64 = 80.0;
    let delta = now.offset - before.offset;
    if (now.viewport_height - before.viewport_height).abs() > 0.5 {
        hidden
    } else if now.at_top() || now.content_height <= now.viewport_height + 1.0 {
        false
    } else if now.at_bottom() {
        // Not while dragging: the bars coming back shorten the page view, which ends the bottom.
        hidden && now.dragging
    } else if delta > SLOP && now.dragging && now.offset > HIDE_AFTER {
        true
    } else if delta < -SLOP {
        false
    } else {
        hidden
    }
}

impl Browser {
    /// The tab with this key while it is open. A tracked read.
    pub fn tab(self, key: &str) -> Option<Tab> {
        let id = key.parse::<u64>().ok()?;
        self.live.with(|m| m.get(&id).copied())
    }

    /// The selected tab. A tracked read.
    pub fn current(self) -> Option<Tab> {
        self.tabs.selected().and_then(|k| self.tab(&k))
    }

    pub fn current_untracked(self) -> Option<Tab> {
        untrack(|| self.current())
    }

    /// Every open tab, in order. A tracked read.
    pub fn all(self) -> Vec<Tab> {
        self.tabs
            .keys()
            .iter()
            .filter_map(|k| self.tab(k))
            .collect()
    }

    /// The pages open in tabs, for the suggestion list's Switch to Tab rows.
    fn open_urls(self) -> Vec<String> {
        untrack(|| self.all().iter().map(|t| t.url()).collect())
    }

    /// Open a tab on `url` (the start page for an empty one). A tab opened from another
    /// (`opener`) goes after it; any other goes last.
    pub fn open(self, url: &str, foreground: bool, private: bool, opener: Option<u64>) -> String {
        self.open_seed(
            Seed {
                url: url.to_string(),
                title: String::new(),
                private,
                opener,
                lazy: false,
            },
            foreground,
            None,
        )
    }

    /// A tab with a known title, for the ones that come back: reopened, or restored from the
    /// last session (`lazy`, which loads its page when the tab is first shown).
    fn open_titled(
        self,
        url: &str,
        title: &str,
        foreground: bool,
        at: Option<usize>,
        lazy: bool,
    ) -> String {
        self.open_seed(
            Seed {
                url: url.to_string(),
                title: title.to_string(),
                private: false,
                opener: None,
                lazy,
            },
            foreground,
            at,
        )
    }

    fn open_seed(self, seed: Seed, foreground: bool, at: Option<usize>) -> String {
        let opener = seed.opener;
        self.serial.update(|n| *n += 1);
        let id = self.serial.get_untracked();
        let key = id.to_string();
        let order: Vec<u64> = self
            .tabs
            .keys()
            .iter()
            .filter_map(|k| k.parse().ok())
            .collect();
        let index = at.unwrap_or_else(|| {
            let run = self.run.get_untracked();
            let run = run.borrow();
            netskip_model::tabs::insertion_index(
                &order,
                opener.as_ref(),
                opener.as_ref().and_then(|o| run.last_child(o)),
            )
        });
        batch(|| {
            self.seeds.update(|s| {
                s.insert(id, seed);
            });
            // The page being left can be pictured only while it shows: an engine that stops
            // drawing a background view (WebView2) cannot picture it afterwards.
            if foreground {
                self.snapshot_current();
            }
            self.tabs.open(key.clone(), foreground);
            self.tabs.move_to(&key, index);
        });
        self.run
            .get_untracked()
            .borrow_mut()
            .opened(opener.as_ref(), &id, foreground);
        if foreground {
            self.overview.set(false);
        }
        key
    }

    /// A new tab: the start page, or the home page when one is set. The address bar takes
    /// focus so the user can type straight away.
    pub fn new_tab(self, private: bool) {
        let private =
            private || untrack(|| self.current().is_some_and(|t| t.private) && self.overview.get());
        self.open("", true, private, None);
        self.omnibox.focused.set(true);
    }

    /// Close a tab, remembering it for Reopen Closed Tab, and select where its opener or its
    /// siblings are, as DuckDuckGo does. Closing the last tab leaves a new one.
    pub fn close(self, key: &str) {
        let order = self.tabs.keys();
        let selected = self.tabs.selected();
        if let Some(tab) = self.tab(key) {
            let index = order.iter().position(|k| k == key).unwrap_or(0);
            if !tab.start.get_untracked() {
                let kept = self.closed.get_untracked().borrow_mut().push(
                    ClosedTab {
                        url: tab.url_untracked(),
                        title: tab.title.get_untracked(),
                        index,
                    },
                    tab.private,
                );
                if kept {
                    self.closed_count.update(|n| *n += 1);
                }
            }
        }
        let next = (selected.as_deref() == Some(key))
            .then(|| {
                netskip_model::tabs::select_after_close(&order, &key.to_string(), |k| {
                    self.tab(k).and_then(|t| t.opener).map(|o| o.to_string())
                })
            })
            .flatten();
        batch(|| {
            self.tabs.close(&key.to_string());
            if let Some(next) = next {
                self.tabs.select(&next);
            }
        });
        if self.tabs.keys().is_empty() {
            // A window always has a tab: closing the last one leaves a fresh start page. Opening
            // a tab leaves the tab grid, but closing the last card in the grid must not: the grid
            // stays, showing the new empty tab, as Safari's does.
            let in_overview = self.overview.get_untracked();
            self.open("", true, false, None);
            if in_overview {
                self.overview.set(true);
            }
        }
    }

    pub fn close_current(self) {
        if let Some(key) = self.tabs.selected() {
            self.close(&key);
        }
    }

    /// Close every tab but `key`.
    pub fn close_others(self, key: &str) {
        for k in self.tabs.keys() {
            if k != key {
                self.close(&k);
            }
        }
    }

    /// Reopen the most recently closed tab where it was.
    pub fn reopen_closed(self) {
        let entry = self.closed.get_untracked().borrow_mut().pop();
        if let Some(tab) = entry {
            self.closed_count.update(|n| *n = n.saturating_sub(1));
            let at = tab.index.min(self.tabs.keys().len());
            self.open_titled(&tab.url, &tab.title, true, Some(at), false);
        }
    }

    /// Whether a closed tab can be reopened. A tracked read.
    pub fn has_closed(self) -> bool {
        self.closed_count.get() > 0
    }

    /// The recently closed tabs, newest first, as (title, url).
    pub fn recently_closed(self) -> Vec<(String, String)> {
        self.closed_count.get();
        self.closed
            .get_untracked()
            .borrow()
            .recent()
            .map(|t| (t.title.clone(), t.url.clone()))
            .collect()
    }

    /// Reopen the closed tab `index` places from the newest.
    pub fn reopen_closed_at(self, index: usize) {
        let entry = self.closed.get_untracked().borrow_mut().take(index);
        if let Some(tab) = entry {
            self.closed_count.update(|n| *n = n.saturating_sub(1));
            let at = tab.index.min(self.tabs.keys().len());
            self.open_titled(&tab.url, &tab.title, true, Some(at), false);
        }
    }

    pub fn forget_closed_tabs(self) {
        self.closed.get_untracked().borrow_mut().clear();
        self.closed_count.set(0);
    }

    /// Duplicate the selected tab.
    pub fn duplicate(self, key: &str) {
        if let Some(tab) = self.tab(key) {
            let url = tab.url_untracked();
            let id = tab.id;
            self.open(&url, true, tab.private, Some(id));
        }
    }

    /// Select the tab at `index` (⌘1…⌘8), or the last one (⌘9), as every browser does.
    pub fn select_index(self, index: usize) {
        let keys = self.tabs.keys();
        let key = if index >= 8 {
            keys.last()
        } else {
            keys.get(index)
        };
        if let Some(key) = key.cloned() {
            self.select(&key);
        }
    }

    /// Select the tab `key`, picturing the page being left for the tab grid first.
    pub fn select(self, key: &str) {
        if untrack(|| self.tabs.selected()).as_deref() != Some(key) {
            self.snapshot_current();
        }
        self.tabs.select(&key.to_string());
    }

    /// Select the next tab, or the previous one, picturing the page being left first.
    pub fn select_next(self, backwards: bool) {
        if self.tabs.keys().len() > 1 {
            self.snapshot_current();
        }
        self.tabs.select_next(backwards);
    }

    /// Load `url` in the selected tab, or a new one when there is none.
    pub fn load_in_current(self, url: &str, typed: bool) {
        if address::is_system_url(url) {
            open_url(url);
            return;
        }
        let tab = match self.current_untracked() {
            Some(tab) => tab,
            None => {
                let key = self.open("", true, false, None);
                match untrack(|| self.tab(&key)) {
                    Some(tab) => tab,
                    None => {
                        // The tab's document builds in this same turn; load once it exists.
                        let url = url.to_string();
                        day::task(async move {
                            day::sleep(0).await;
                            if let Some(tab) = self.current_untracked() {
                                tab.typed.set(typed);
                                tab.load(&url);
                            }
                        });
                        return;
                    }
                }
            }
        };
        tab.typed.set(typed);
        tab.load(url);
        self.overview.set(false);
        self.library.set(None);
    }

    /// Do what submitting the address bar asks: search or navigate.
    pub fn go(self, input: Input, typed: bool) {
        match input {
            Input::Navigate(url) => {
                let upgraded = if settings::UPGRADE_HTTPS.get_untracked() {
                    address::upgrade_to_https(&url)
                } else {
                    url.clone()
                };
                self.load_in_current(&upgraded, typed);
                // A site with no HTTPS still opens: the address as typed loads if the
                // upgraded one fails (`load_failed`).
                if upgraded != url
                    && let Some(tab) = self.current_untracked()
                {
                    tab.http_fallback.set(Some(url));
                }
            }
            Input::Search(terms) => {
                let url = untrack(settings::engine).query_url(&terms);
                self.load_in_current(&url, typed);
            }
        }
    }

    /// The home page, or the start page when none is set.
    pub fn go_home(self) {
        match settings::home_url() {
            Some(url) => self.load_in_current(&url, false),
            None => {
                if let Some(tab) = self.current_untracked() {
                    if !tab.start.get_untracked() {
                        tab.from_start.set(false);
                    }
                    tab.start.set(true);
                }
            }
        }
    }

    /// Let go of the web views used longest ago until at most `keep` remain. The selected tab
    /// always keeps its own.
    pub fn trim_views(self, keep: usize) {
        // The window may be gone by the time the system reports memory pressure.
        let Some(live) = self.live.try_get() else {
            return;
        };
        let views: Vec<(u64, i64)> = live
            .values()
            .filter(|t| t.has_view.get_untracked())
            .map(|t| (t.id, t.used.get_untracked()))
            .collect();
        let selected = self.current_untracked().map(|t| t.id);
        for id in netskip_model::tabs::views_to_unload(&views, selected.as_ref(), keep) {
            if let Some(tab) = live.get(&id) {
                tab.unload();
            }
        }
    }

    /// The process running `tab`'s page ended. Its web view is rebuilt, which loads the page
    /// again, unless that just happened: a page that keeps crashing shows an error page.
    pub fn renderer_gone(self, tab: Tab) {
        let now = crate::now_ms();
        let again = now.saturating_sub(tab.crashed_at.get_untracked()) < CRASH_LOOP_MS;
        tab.crashed_at.set(now);
        tab.unload();
        if again {
            tab.error.set(Some(PageError {
                kind: PageErrorKind::Crashed,
                url: tab.url_untracked(),
            }));
        } else if self.current_untracked().is_some_and(|t| t.id == tab.id) {
            // A turn later, so the dead view is released before its replacement is built.
            day::task(async move {
                day::sleep(0).await;
                tab.wake();
            });
        }
    }

    /// `tab`'s page did not load: show the error page until the next load.
    pub fn load_failed(self, tab: Tab, error: &LoadError) {
        let url = if error.url.is_empty() {
            tab.url_untracked()
        } else {
            fixtures::to_logical(&error.url)
        };
        warn!(
            "load failed ({:?}, {}): {}",
            error.kind, error.code, error.description
        );
        // The address was typed as http:// and upgraded: the site has no working HTTPS, so
        // load what was typed.
        let no_https = matches!(
            error.kind,
            LoadErrorKind::Connection
                | LoadErrorKind::Secure
                | LoadErrorKind::Certificate
                | LoadErrorKind::Timeout
        );
        if no_https && let Some(plain) = tab.http_fallback.get_untracked() {
            tab.load(&plain);
            return;
        }
        batch(|| {
            tab.error.set(Some(PageError {
                kind: PageErrorKind::Load(error.kind),
                url,
            }));
            tab.progress
                .set(tab.progress.get_untracked().report(false, 1.0));
        });
        let _ = self;
    }

    /// The tab a web page asked for (a link opened in a new tab, `target=_blank`).
    pub fn open_from_page(self, opener: Tab, url: &str, background: bool) {
        let url = fixtures::to_logical(url);
        let background = background || settings::OPEN_IN_BACKGROUND.get_untracked();
        self.open(&url, !background, opener.private, Some(opener.id));
    }

    /// Clear cookies, caches and storage in the browser's profiles.
    pub fn clear_website_data(self) {
        crate::favicons::clear();
        if let Some(tab) = self.current_untracked() {
            let js = tab.js;
            day::task(async move {
                if let Err(e) = js.clear_data().await {
                    warn!("clearing website data failed: {e}");
                }
            });
        }
    }

    /// The session to save: the open tabs' pages in order and the selected one.
    fn session(self) -> Session {
        let keys = self.tabs.keys();
        let selected = self.tabs.selected();
        let viewed = self.viewed.get();
        let tabs: Vec<(String, String, bool, i64)> = keys
            .iter()
            .filter_map(|k| self.tab(k).map(|t| (k, t)))
            .map(|(k, t)| {
                let url = if t.start.get() {
                    String::new()
                } else {
                    t.url()
                };
                let seen = viewed.get(k).copied().unwrap_or(0);
                (url, t.title.get(), t.private, seen)
            })
            .collect();
        let selected_index = selected.and_then(|s| keys.iter().position(|k| *k == s));
        Session::capture(
            tabs.iter()
                .map(|(u, t, p, v)| (u.as_str(), t.as_str(), *p, *v)),
            selected_index,
        )
    }

    /// Open the saved session's tabs, or one tab on the home page or start page.
    fn restore(self) {
        let mut session = if self.primary && settings::RESTORE_TABS.get_untracked() {
            day::prefs::get(SESSION_KEY)
                .map(|s| Session::decode(&s))
                .unwrap_or_default()
        } else {
            Session::default()
        };
        let days = settings::CLOSE_TABS_DAYS.get_untracked();
        if days > 0 {
            session.expire(crate::now_ms(), days * 86_400_000);
        }
        // A dayscript or a link the OS opened the app with names its own first page.
        let launch = day::env("NETSKIP_START_URL");
        if session.tabs.is_empty() || launch.is_some() {
            let url = launch.or_else(settings::home_url).unwrap_or_default();
            self.open(&url, true, false, None);
            return;
        }
        let mut keys = Vec::new();
        for tab in &session.tabs {
            let key = self.open_titled(&tab.url, &tab.title, false, None, true);
            if tab.viewed > 0 {
                self.viewed.update(|v| {
                    v.insert(key.clone(), tab.viewed);
                });
            }
            keys.push(key);
        }
        if let Some(key) = keys.get(session.selected) {
            self.tabs.select(key);
        }
    }

    /// Build a tab's state inside its document scope. Called by the document's builder.
    pub fn make_tab(self, key: &str) -> Tab {
        let id = key.parse::<u64>().unwrap_or_default();
        let seed = self
            .seeds
            .with_untracked(|s| s.get(&id).cloned())
            .unwrap_or(Seed {
                url: String::new(),
                title: String::new(),
                private: false,
                opener: None,
                lazy: false,
            });
        let blank = seed.url.is_empty();
        // A restored tab loads its page when it is first shown; the one the session had
        // selected is showing now.
        let selected = untrack(|| self.tabs.selected()).as_deref() == Some(key);
        let asleep = seed.lazy && !selected;
        let tab = Tab {
            id,
            private: seed.private,
            opener: seed.opener,
            location: Signal::new(if blank {
                "about:blank".to_string()
            } else {
                fixtures::to_physical(&seed.url)
            }),
            title: Signal::new(seed.title.clone()),
            icon: Signal::new(None),
            can_back: Signal::new(false),
            can_forward: Signal::new(false),
            progress: Signal::new(if blank || asleep {
                LoadProgress::default()
            } else {
                LoadProgress::default().report(true, 0.0)
            }),
            start: Signal::new(blank),
            has_view: Signal::new(!blank && !asleep),
            from_start: Signal::new(false),
            typed: Signal::new(false),
            zoom: Signal::new(settings::PAGE_ZOOM.get_untracked()),
            desktop: Signal::new(settings::DESKTOP_SITES.get_untracked()),
            find: Find {
                open: Signal::new(false),
                query: Signal::new(String::new()),
                matches: Signal::new((0, 0)),
            },
            go: Trigger::new(),
            back: Trigger::new(),
            forward: Trigger::new(),
            reload: Trigger::new(),
            stop: Trigger::new(),
            js: JsHandle::new(),
            loads: Signal::new(0),
            snapshot: Signal::new(None),
            scroll: Signal::new(ScrollState::default()),
            blocked: Signal::new(0),
            page: Signal::new(PageState::default()),
            error: Signal::new(None),
            restore_scroll: Signal::new(0.0),
            used: Signal::new(crate::now_ms()),
            crashed_at: Signal::new(i64::MIN / 2),
            http_fallback: Signal::new(None),
        };
        // A new page starts its own count of blocked requests.
        watch(
            move || tab.page.get().loading,
            move |loading, was| {
                if *loading && was == Some(&false) {
                    tab.blocked.set(0);
                }
            },
        );
        self.live.update(|m| {
            m.insert(id, tab);
        });
        self.seeds.update(|s| {
            s.remove(&id);
        });
        let live = self.live;
        day::reactive::Scope::current().on_cleanup(move || {
            if live.try_get().is_some() {
                live.update(|m| {
                    m.remove(&id);
                });
            }
        });
        watch(
            move || tab.page.get(),
            move |state, _| apply_state(tab, state),
        );
        tab
    }

    /// Show the tab grid, with a fresh picture of the page being left.
    ///
    /// The grid waits for the picture, briefly: taken after it opens, the old picture would
    /// show first and the new one pop in, and an engine that stops drawing a covered page
    /// could not take it at all.
    pub fn show_overview(self) {
        let Some(tab) = self.current_untracked().filter(|t| can_snapshot(*t)) else {
            self.overview.set(true);
            return;
        };
        let shown = Rc::new(std::cell::Cell::new(false));
        let show = move |shown: &std::cell::Cell<bool>| {
            if !shown.replace(true) {
                self.overview.set(true);
            }
        };
        let waited = shown.clone();
        day::task(async move {
            day::sleep(OVERVIEW_SNAPSHOT_WAIT_MS).await;
            show(&waited);
        });
        day::task(async move {
            take_snapshot(tab).await;
            show(&shown);
        });
    }

    /// Take a picture of the selected tab's page for the tab grid.
    pub fn snapshot_current(self) {
        if let Some(tab) = self.current_untracked() {
            snapshot(tab);
        }
    }

    /// A load finished in `tab`: read the page's state, record the visit, fetch the icon.
    pub fn loaded(self, tab: Tab) {
        let typed = tab.typed.get_untracked();
        tab.typed.set(false);
        tab.loads.update(|n| *n += 1);
        let js = tab.js;
        // What finished loading under an error page is the engine's own error page (Android):
        // it is not a visit, and has no icon or picture worth keeping.
        if tab.error.get_untracked().is_some() {
            return;
        }
        // A page loaded again after its web view was let go returns to where it was scrolled.
        let scroll = tab.restore_scroll.get_untracked();
        if scroll > 1.0 {
            tab.restore_scroll.set(0.0);
            day::task(async move {
                let _ = js.eval(format!("window.scrollTo(0, {scroll:.0})")).await;
            });
        }
        day::task(async move {
            let url = tab.url_untracked();
            let title = tab.title.get_untracked();
            if let Some(visit) = history::visit(&url, &title, typed, tab.private) {
                let done = store::write(move |db| {
                    store::record_visit(
                        db,
                        &visit.key,
                        visit.title.as_deref(),
                        visit.typed,
                        crate::now_ms(),
                    )
                });
                if let Err(e) = done.await {
                    warn!("recording a visit failed: {e}");
                }
                library::changed();
            }
            // Each site opens at the zoom last chosen for it, else the default.
            if let Some(site) = address::site(&url) {
                tab.zoom.set(settings::site_zoom(&site));
            }
            if let Ok(info) = js.eval(scripts::PAGE_INFO).await {
                crate::favicons::load(tab, &url, &info);
            }
            if tab.find.open.get_untracked() {
                tab.find.open.set(false);
            }
            // Let the page paint before the tab grid's picture of it is taken.
            day::sleep(700).await;
            snapshot(tab);
        });
        let _ = self;
    }
}

/// Fold the engine's report into the tab: back and forward, the title (which history keeps
/// once it arrives), and the progress line.
fn apply_state(tab: Tab, state: &PageState) {
    batch(|| {
        tab.can_back.set(state.can_go_back);
        tab.can_forward.set(state.can_go_forward);
        let title = state.title.trim().to_string();
        // Under an error page the engine may show a page of its own, whose title is not the
        // site's.
        if !title.is_empty()
            && tab.title.get_untracked() != title
            && tab.error.get_untracked().is_none()
        {
            tab.title.set(title.clone());
            // The title belongs to the address reported with it. The tab's own address can
            // trail it: going back on Android reports the earlier page's address and title
            // together, while the tab still holds the page being left.
            let url = if state.url.is_empty() {
                tab.url_untracked()
            } else {
                fixtures::to_logical(&state.url)
            };
            if !tab.private && history::records(&url, false) {
                let key = address::history_key(&url).unwrap_or(url);
                let done = store::write(move |db| store::set_title(db, &key, &title));
                day::task(async move {
                    if done.await.is_ok() {
                        library::changed();
                    }
                });
            }
        }
        let progress = tab.progress.get_untracked();
        tab.progress
            .set(progress.report(state.loading, state.progress));
    });
}

/// How long, in milliseconds, the tab grid waits for the picture of the page being left.
const OVERVIEW_SNAPSHOT_WAIT_MS: u32 = 400;

/// Whether the tab has a page to picture.
fn can_snapshot(tab: Tab) -> bool {
    tab.has_view.get_untracked() && !tab.start.get_untracked()
}

/// Picture the tab's page for the tab grid, in the background.
fn snapshot(tab: Tab) {
    if can_snapshot(tab) {
        day::task(take_snapshot(tab));
    }
}

/// Picture the tab's page for the tab grid, scaled to a card's width on a 3× screen. A page
/// the engine cannot picture now (one off screen) keeps its last picture.
async fn take_snapshot(tab: Tab) {
    if let Ok(png) = tab.js.snapshot(480).await
        && !png.is_empty()
    {
        tab.snapshot.set(Some(Icon(Rc::from(png))));
    }
}

// ---------------------------------------------------------------------------------------------
// The address bar
// ---------------------------------------------------------------------------------------------

/// The address bar's editing state, one per window: the text, what the user typed of it, the
/// inline completion and the suggestion list under the bar.
#[derive(Clone, Copy)]
pub struct Omnibox {
    /// The field's text, inline completion included.
    pub text: Signal<String>,
    pub focused: Signal<bool>,
    /// What the user typed; the text is this plus the completion's selected rest.
    typed: Signal<String>,
    pub inline: Signal<Option<Inline>>,
    pub rows: Signal<Vec<Row>>,
    /// The row the arrow keys moved to.
    pub highlight: Signal<Option<usize>>,
    candidates: Signal<Rc<Vec<Candidate>>>,
    remote: Signal<Vec<String>>,
    /// Counts edits, so a late answer for older text is dropped.
    generation: Signal<u64>,
    /// The selection the field should show, as (start, end) in characters, re-applied on
    /// every change of `select_seq`.
    pub selection: Signal<(usize, usize)>,
    pub select_seq: Trigger,
}

impl Omnibox {
    fn new() -> Self {
        Omnibox {
            text: Signal::new(String::new()),
            focused: Signal::new(false),
            typed: Signal::new(String::new()),
            inline: Signal::new(None),
            rows: Signal::new(Vec::new()),
            highlight: Signal::new(None),
            candidates: Signal::new(Rc::new(Vec::new())),
            remote: Signal::new(Vec::new()),
            generation: Signal::new(0),
            selection: Signal::new((0, 0)),
            select_seq: Trigger::new(),
        }
    }

    /// The text the bar shows while not editing: the search for an engine's results page,
    /// else the site (or whole address, by setting), empty on the start page. Tracked.
    pub fn resting_text(browser: Browser) -> String {
        let Some(tab) = browser.current() else {
            return String::new();
        };
        if tab.start.get() || is_blank(&tab.location.get()) {
            return String::new();
        }
        let url = tab.url();
        if let Some(terms) = settings::engine().terms_in(&url) {
            return terms;
        }
        address::display(&url, settings::FULL_ADDRESS.get())
    }

    /// The text to edit when the bar takes focus: the whole address, or the search terms.
    fn editing_text(browser: Browser) -> String {
        untrack(|| {
            let Some(tab) = browser.current() else {
                return String::new();
            };
            if tab.start.get() || is_blank(&tab.location.get()) {
                return String::new();
            }
            let url = tab.url();
            settings::engine()
                .terms_in(&url)
                .unwrap_or_else(|| address::editable(&url))
        })
    }

    /// Select characters `start..end` of the field.
    pub fn select(self, start: usize, end: usize) {
        self.selection.set((start, end));
        self.select_seq.notify();
    }

    /// The field took focus: show the whole address, selected, ready to type over.
    pub fn begin_editing(self, browser: Browser) {
        let text = Self::editing_text(browser);
        batch(|| {
            self.generation.update(|n| *n += 1);
            self.text.set(text.clone());
            self.typed.set(text.clone());
            self.inline.set(None);
            self.rows.set(Vec::new());
            self.remote.set(Vec::new());
            self.highlight.set(None);
        });
        self.select(0, text.chars().count());
    }

    /// Editing ended (focus left, a submit, Escape): back to the resting text.
    pub fn end_editing(self, browser: Browser) {
        batch(|| {
            self.generation.update(|n| *n += 1);
            self.inline.set(None);
            self.rows.set(Vec::new());
            self.remote.set(Vec::new());
            self.highlight.set(None);
            if self.focused.get_untracked() {
                self.focused.set(false);
            }
            let rest = untrack(|| Self::resting_text(browser));
            self.text.set(rest.clone());
            self.typed.set(rest);
        });
    }

    /// Whether the suggestion list is showing. Tracked.
    pub fn showing(self) -> bool {
        self.focused.get() && !self.rows.with(Vec::is_empty)
    }

    /// The user changed the field's text.
    pub fn edited(self, browser: Browser, text: String) {
        let previous = self.typed.get_untracked();
        if text == self.text.get_untracked() && text == previous {
            return;
        }
        let complete = complete::should_complete(&previous, &text);
        batch(|| {
            self.generation.update(|n| *n += 1);
            self.text.set(text.clone());
            self.typed.set(text.clone());
            self.inline.set(None);
            self.highlight.set(None);
            if text.trim().is_empty() {
                self.rows.set(Vec::new());
                self.remote.set(Vec::new());
            }
        });
        if text.trim().is_empty() {
            return;
        }
        self.query(browser, text, complete);
    }

    /// Ask the library for pages matching `typed`, then the engine for suggestions.
    fn query(self, browser: Browser, typed: String, complete: bool) {
        let generation = self.generation.get_untracked();
        let q = typed.clone();
        let terms = typed.trim().to_string();
        // Net Skip Suggestions off: the bar offers no pages from history or bookmarks.
        let local = settings::LIBRARY_SUGGESTIONS.get_untracked();
        let read = store::read(move |db| {
            if local {
                store::candidates(db, &q, store::CANDIDATE_LIMIT)
            } else {
                Ok(Vec::new())
            }
        });
        day::task(async move {
            let candidates = read.await.unwrap_or_else(|e| {
                warn!("completion read failed: {e}");
                Vec::new()
            });
            if self.generation.get_untracked() != generation {
                return;
            }
            let now = crate::now_ms();
            let inline = if complete {
                complete::inline(&typed, &candidates, now)
            } else {
                None
            };
            self.candidates.set(Rc::new(candidates));
            batch(|| {
                self.inline.set(inline.clone());
                if let Some(inline) = &inline {
                    self.text.set(inline.text.clone());
                }
                self.recompute(browser);
            });
            if let Some(inline) = inline {
                self.select(inline.typed_chars, inline.text.chars().count());
            }
        });
        if !settings::SUGGESTIONS.get_untracked()
            || untrack(|| browser.current().is_some_and(|t| t.private))
        {
            return;
        }
        let Some((url, format)) = untrack(settings::engine).suggestion_url(&terms) else {
            return;
        };
        day::task(async move {
            // Wait for a pause in typing before asking the engine.
            day::sleep(160).await;
            if self.generation.get_untracked() != generation {
                return;
            }
            let request =
                day_part_http::Request::get(&url).timeout(std::time::Duration::from_secs(4));
            let Ok(response) = day_part_http::fetch_future(request).await else {
                return;
            };
            if self.generation.get_untracked() != generation || response.status != 200 {
                return;
            }
            let found = netskip_model::search::parse_suggestions(format, &response.body, 6);
            self.remote.set(found);
            self.recompute(browser);
        });
    }

    fn recompute(self, browser: Browser) {
        let typed = self.typed.get_untracked();
        let inline = self.inline.get_untracked();
        let candidates = self.candidates.get_untracked();
        let remote = self.remote.get_untracked();
        let rows = complete::rows(
            &typed,
            inline.as_ref(),
            &candidates,
            &browser.open_urls(),
            &remote,
            crate::now_ms(),
            Limits::default(),
        );
        self.rows.set(rows);
    }

    /// Up or Down in the field: move through the rows, showing each in the field. Past either
    /// end the highlight goes and the typed text comes back, as DuckDuckGo's bar does.
    pub fn arrow(self, down: bool) -> bool {
        let rows = self.rows.get_untracked();
        if rows.is_empty() {
            return false;
        }
        let next = complete::step_highlight(rows.len(), self.highlight.get_untracked(), down);
        batch(|| {
            self.highlight.set(next);
            let text = match next.and_then(|i| rows.get(i)) {
                Some(row) => row_text(row),
                None => self
                    .inline
                    .get_untracked()
                    .map(|i| i.text)
                    .unwrap_or_else(|| self.typed.get_untracked()),
            };
            self.text.set(text);
        });
        let len = self.text.get_untracked().chars().count();
        self.select(len, len);
        true
    }

    /// Escape: the first one closes the list (keeping what was typed), the second gives up
    /// editing and puts the address back.
    pub fn escape(self, browser: Browser) {
        if !self.rows.with_untracked(Vec::is_empty) {
            batch(|| {
                self.generation.update(|n| *n += 1);
                self.rows.set(Vec::new());
                self.inline.set(None);
                self.highlight.set(None);
                self.text.set(self.typed.get_untracked());
            });
        } else {
            self.end_editing(browser);
        }
    }

    /// Return in the field: the highlighted row, else the inline completion, else the text.
    pub fn submit(self, browser: Browser) {
        let rows = self.rows.get_untracked();
        if let Some(row) = self.highlight.get_untracked().and_then(|i| rows.get(i)) {
            self.choose(browser, row.clone());
            return;
        }
        let text = self.text.get_untracked();
        if let Some(inline) = self.inline.get_untracked()
            && inline.text == text
        {
            self.end_editing(browser);
            browser.load_in_current(&inline.url, true);
            return;
        }
        let Some(input) = address::classify(&text) else {
            return;
        };
        self.end_editing(browser);
        browser.go(input, true);
    }

    /// A row was chosen, by tap or Return.
    pub fn choose(self, browser: Browser, row: Row) {
        self.end_editing(browser);
        match row {
            Row::Page {
                url,
                kind: RowKind::OpenTab,
                ..
            } => {
                let key = untrack(|| {
                    let want = address::history_key(&url);
                    browser
                        .all()
                        .into_iter()
                        .find(|t| address::history_key(&t.url()) == want)
                        .map(|t| t.id.to_string())
                });
                match key {
                    Some(key) => {
                        browser.select(&key);
                    }
                    None => browser.load_in_current(&url, true),
                }
            }
            Row::Page { url, .. } | Row::Visit { url } => browser.load_in_current(&url, true),
            Row::Search { terms, .. } => browser.go(Input::Search(terms), true),
        }
    }
}

/// The text a row puts in the field when the arrow keys reach it.
pub fn row_text(row: &Row) -> String {
    match row {
        Row::Page { url, .. } | Row::Visit { url } => address::editable(url),
        Row::Search { terms, .. } => terms.clone(),
    }
}

/// The engine searches go to, for row labels. Tracked.
pub fn engine() -> Engine {
    settings::engine()
}

use crate::res;

#[cfg(test)]
mod bar_tests {
    use super::*;

    fn at(offset: f64, dragging: bool) -> ScrollState {
        ScrollState {
            offset,
            content_height: 5000.0,
            viewport_height: 700.0,
            dragging,
        }
    }

    #[test]
    fn a_drag_down_hides_and_a_move_up_shows() {
        assert!(bars_hidden(&at(400.0, true), &at(380.0, true), false));
        assert!(!bars_hidden(&at(360.0, false), &at(380.0, false), true));
        // A fling down without a finger keeps the bars as they were.
        assert!(!bars_hidden(&at(900.0, false), &at(800.0, false), false));
        assert!(bars_hidden(&at(900.0, false), &at(800.0, false), true));
        // Small jitter changes nothing.
        assert!(bars_hidden(&at(402.0, true), &at(400.0, true), true));
    }

    #[test]
    fn the_ends_and_short_pages_show_the_bars() {
        assert!(!bars_hidden(&at(0.0, true), &at(-20.0, true), true));
        // The bottom shows them once the finger lifts, and keeps them as they are until then.
        assert!(!bars_hidden(&at(4300.0, false), &at(4300.0, true), true));
        assert!(bars_hidden(&at(4300.0, true), &at(4200.0, true), true));
        assert!(!bars_hidden(&at(4300.0, true), &at(4200.0, true), false));
        let short = ScrollState {
            offset: 30.0,
            content_height: 720.0,
            viewport_height: 700.0,
            dragging: true,
        };
        assert!(!bars_hidden(&short, &at(10.0, true), false));
    }

    /// The Android loop: the bar hides, the view grows, and the page scrolls back under the finger.
    #[test]
    fn the_bars_own_resize_changes_nothing() {
        let hiding = at(156.4, true);
        let grown = ScrollState {
            offset: 104.6,
            viewport_height: 756.0,
            ..hiding
        };
        assert!(bars_hidden(&grown, &hiding, true));
        // And the same for the bars coming back.
        let shrunk = ScrollState {
            offset: 160.0,
            viewport_height: 700.0,
            ..hiding
        };
        assert!(!bars_hidden(&shrunk, &grown, false));
    }

    #[test]
    fn a_drag_near_the_top_keeps_the_bars() {
        assert!(!bars_hidden(&at(40.0, true), &at(20.0, true), false));
        assert!(bars_hidden(&at(100.0, true), &at(80.0, true), false));
    }
}
