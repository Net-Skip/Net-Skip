// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! What the library shows: the start page's Favorites and Frequently Visited, and the
//! Bookmarks and History lists. Reads go to the library worker ([`crate::store`]) and land in
//! signals that keep their last value while the next read runs.

use std::convert::Infallible;
use std::future::Future;

use day::prelude::*;
use day::reactive::Resource;
use day_persistence::DbError;
use netskip_model::address;
use netskip_model::complete::{self, Candidate};
use netskip_model::history::{self, ClearRange, DayGroup};

use crate::browser::{Browser, Tab};
use crate::chrome::site_icon;
use crate::store::{self, Bookmark, HistoryPage};
use crate::{res, settings};

day::routes! {
    /// The library's two lists.
    pub enum Pane {
        Bookmarks => "bookmarks",
        History => "history",
    }
}

thread_local! {
    /// Bumped after every committed library write; every library snapshot refetches on it.
    // `global`: created on first use, wherever that is, and never disposed with that scope.
    static VERSION: Signal<u64> = Signal::global(0);
}

/// The library changed: refresh everything showing it.
pub fn changed() {
    VERSION.with(|v| v.update(|n| *n += 1));
}

fn version() -> u64 {
    VERSION.with(|v| v.get())
}

/// A signal holding what `fetch` last produced for `source`'s current value. The previous value
/// stays while the next loads, and a superseded fetch never lands.
fn snapshot<S, T, F>(
    initial: T,
    source: impl Fn() -> S + 'static,
    fetch: impl Fn(S) -> F + 'static,
) -> Signal<T>
where
    S: Clone + PartialEq + 'static,
    T: 'static,
    F: Future<Output = Result<T, DbError>> + 'static,
{
    let value = Signal::new(initial);
    Resource::new(source, move |s| {
        let fetched = fetch(s);
        async move {
            match fetched.await {
                Ok(v) => value.set(v),
                Err(e) => warn!("library read failed: {e}"),
            }
            Ok::<(), Infallible>(())
        }
    });
    value
}

/// Every bookmark, in order. Tracked.
pub fn bookmarks() -> Signal<Vec<Bookmark>> {
    snapshot(Vec::new(), version, |_| store::read(store::bookmarks))
}

/// History, newest first, filtered by `query`.
pub fn history_pages(query: Signal<String>) -> Signal<Vec<HistoryPage>> {
    snapshot(
        Vec::new(),
        move || (version(), query.get()),
        |(_, q)| store::read(move |db| store::history(db, &q, 1000)),
    )
}

/// The start page's sites, from recent history and bookmarks.
fn recent() -> Signal<Vec<Candidate>> {
    snapshot(Vec::new(), version, |_| {
        store::read(|db| store::recent_candidates(db, 400))
    })
}

/// Bookmark a page, or remove its bookmark when it has one.
pub fn toggle_bookmark(url: String, title: String) {
    let done = store::write(move |db| match store::bookmark_for(db, &url)? {
        Some(b) => store::remove_bookmark(db, b.id),
        None => store::add_bookmark(db, &url, &title, crate::now_ms()).map(|_| ()),
    });
    day::task(async move {
        if let Err(e) = done.await {
            error!("changing a bookmark failed: {e}");
        }
        changed();
    });
}

/// Whether the page has a bookmark, against the bookmarks snapshot `marks`. Tracked.
pub fn is_bookmarked(marks: Signal<Vec<Bookmark>>, url: &str) -> bool {
    let key = address::history_key(url);
    key.is_some() && marks.with(|m| m.iter().any(|b| address::history_key(&b.url) == key))
}

fn remove_bookmark(id: u64) {
    let done = store::write(move |db| store::remove_bookmark(db, id));
    day::task(async move {
        let _ = done.await;
        changed();
    });
}

fn move_bookmark(id: u64, index: usize) {
    let done = store::write(move |db| store::move_bookmark(db, id, index));
    day::task(async move {
        let _ = done.await;
        changed();
    });
}

fn delete_history(ids: Vec<u64>) {
    let done = store::write(move |db| store::delete_pages(db, &ids));
    day::task(async move {
        let _ = done.await;
        changed();
    });
}

// ---------------------------------------------------------------------------------------------
// The start page
// ---------------------------------------------------------------------------------------------

/// The page a new tab shows: Favorites (the bookmarks) and Frequently Visited sites, as icon
/// tiles, like Safari's start page.
pub fn start_page(browser: Browser, tab: Tab) -> impl Piece {
    let marks = bookmarks();
    let recent = recent();
    let frequent = move || {
        if !settings::SHOW_FREQUENT.get() || tab.private {
            return Vec::new();
        }
        complete::top_sites(&recent.get(), crate::now_ms(), 8)
    };
    let open = move |url: String| {
        tab.typed.set(true);
        tab.load(&url);
    };
    let section_title = |text: LocalizedText| {
        label(text)
            .font(Font::Title3)
            .bold()
            .padding(Insets::new(0.0, 4.0, 8.0, 4.0))
    };
    scroll(
        column((
            when(
                move || tab.private,
                move || {
                    row((
                        vector(res::vectors::incognito)
                            .tint(Color::hex(0x8e8e93))
                            .frame(28.0, 28.0),
                        label(res::str::private_explainer()).secondary(),
                    ))
                    .spacing(Space::S)
                    .padding(Space::M)
                    .card()
                    .id(day::id!(private_explainer))
                },
            ),
            section_title(res::str::favorites()),
            when(
                move || marks.with(Vec::is_empty),
                || {
                    label(res::str::favorites_empty())
                        .secondary()
                        .padding(Insets::new(0.0, 4.0, 8.0, 4.0))
                        .id(day::id!(favorites_empty))
                },
            ),
            row((each(
                items(move || marks.get(), |b: &Bookmark| b.id),
                move |slot: ItemSlot<Bookmark, u64>| {
                    let url = move || slot.field(|b| b.url.clone());
                    let title = move || {
                        slot.field(|b| {
                            if b.title.trim().is_empty() {
                                address::display(&b.url, false)
                            } else {
                                b.title.clone()
                            }
                        })
                    };
                    start_tile(url, title, move || open(url()))
                        .context_menu(move || {
                            let id = slot.key();
                            let u = url();
                            vec![
                                menu_item(res::str::open_in_new_tab()).action(move || {
                                    browser.open(&u, true, tab.private, None);
                                }),
                                menu_separator(),
                                menu_item(res::str::remove_bookmark())
                                    .action(move || remove_bookmark(id)),
                            ]
                        })
                        .id_of(move || day::id!(favorite, slot.key()))
                },
            ),))
            .fit(RowFit::Wrap { run_spacing: 16.0 })
            .spacing(16.0),
            when(
                move || !frequent().is_empty(),
                move || {
                    column((
                        section_title(res::str::frequently_visited()),
                        row((each(
                            items(frequent, |c: &Candidate| c.url.clone()),
                            move |slot: ItemSlot<Candidate, String>| {
                                let url = move || slot.field(|c| c.url.clone());
                                let title = move || {
                                    slot.field(|c| {
                                        if c.title.trim().is_empty() {
                                            address::display(&c.url, false)
                                        } else {
                                            c.title.clone()
                                        }
                                    })
                                };
                                start_tile(url, title, move || open(url()))
                                    .id_of(move || day::id!(frequent, slot.key()))
                            },
                        ),))
                        .fit(RowFit::Wrap { run_spacing: 16.0 })
                        .spacing(16.0),
                    ))
                    .align(HAlign::Leading)
                    .spacing(Space::S)
                },
            ),
        ))
        .align(HAlign::Leading)
        .spacing(Space::M)
        .padding(Space::L)
        .grow(Dimension::Width)
        .max_width(860.0)
        .aligned(Alignment::Top)
        .grow(Dimension::Width),
    )
    .background(move || day::system_color(SystemColor::Background))
    .id(day::id!(start_page, tab.id))
}

/// One site on the start page: its icon over its name.
fn start_tile(
    url: impl Fn() -> String + Copy + 'static,
    title: impl Fn() -> String + 'static,
    action: impl Fn() + 'static,
) -> impl Piece {
    column((
        site_icon(url, 60.0),
        label(title)
            .font(Font::Caption)
            .max_lines(2)
            .align(TextAlign::Center)
            .width(76.0),
    ))
    .spacing(6.0)
    .width(80.0)
    .on_tap(action)
}

// ---------------------------------------------------------------------------------------------
// Bookmarks and History
// ---------------------------------------------------------------------------------------------

/// Phones and tablets: the library's search is the platform's page search (a
/// `UISearchController` under the sheet's navigation bar, the app bar's search action on
/// Android). The desktops' sheets have no toolbar of their own, so they show a field.
const TOUCH: bool = cfg!(any(
    target_os = "ios",
    target_os = "android",
    target_env = "ohos"
));

/// The library sheet, laid out as Safari's: the pane's name as the title with Done beside it,
/// the Bookmarks / History switch across the top of the content, the search under it, the
/// lists inset grouped, and Edit or Clear in the bottom bar.
pub fn library_page(browser: Browser, pane: Pane) -> impl Piece {
    let selected = Signal::new(match pane {
        Pane::Bookmarks => 0usize,
        Pane::History => 1,
    });
    let query = Signal::new(String::new());
    let editing = Signal::new(false);
    let path = Signal::new(Vec::<String>::new());
    let close = move || browser.library.set(None);
    let lists = switch(
        move || selected.get(),
        move |pane| match pane {
            0 => bookmarks_list(browser, query, editing).any(),
            _ => history_list(browser, query).any(),
        },
    )
    .grow(Dimension::Both);
    let prompt = move || match selected.get() {
        0 => res::str::search_bookmarks().format(),
        _ => res::str::search_history().format(),
    };
    let switcher = picker(
        selected,
        [(0usize, res::str::bookmarks()), (1, res::str::history())],
    )
    .segmented()
    .grow(Dimension::Width)
    .id(day::id!(library_pane))
    .padding(Insets::new(
        Space::S.points(),
        Space::M.points(),
        0.0,
        Space::M.points(),
    ));
    let root = if TOUCH {
        column((
            switcher,
            lists.searchable(Search::new(query).prompt(prompt)),
        ))
        .spacing(Space::S)
        .grow(Dimension::Both)
        .any()
    } else {
        // A desktop sheet has no bar of its own: Done and Clear sit beside the switch.
        column((
            row((
                switcher,
                when(
                    move || selected.get() == 1,
                    || {
                        button(res::str::clear_history_button())
                            .action(choose_clear_range)
                            .id(day::id!(clear_history))
                    },
                ),
                button(res::str::done())
                    .prominent()
                    .action(close)
                    .id(day::id!(library_done)),
            ))
            .spacing(Space::S)
            .align(VAlign::Center),
            text_field(query)
                .placeholder(prompt)
                .id(day::id!(library_search))
                .padding(Insets::new(0.0, Space::M.points(), 0.0, Space::M.points())),
            lists,
        ))
        .spacing(Space::S)
        .grow(Dimension::Both)
        .any()
    };
    // Each pane keeps its own search and leaves editing.
    watch(
        move || selected.get(),
        move |_, _| {
            query.set(String::new());
            editing.set(false);
        },
    );
    nav_stack(path, root.id(day::id!(library_page)))
        .title(move || match selected.get() {
            0 => res::str::bookmarks().format(),
            _ => res::str::history().format(),
        })
        .toolbar(move || {
            if !TOUCH {
                return Vec::new();
            }
            let mut items = vec![
                toolbar_button(day::id!(library_done), res::str::done())
                    .placement(ToolbarPlacement::Primary)
                    .prominent()
                    .action(close),
            ];
            if selected.get() == 0 {
                let label = if editing.get() {
                    res::str::done()
                } else {
                    res::str::edit()
                };
                items.push(
                    toolbar_button(day::id!(library_edit), label)
                        .placement(ToolbarPlacement::Bottom)
                        .action(move || editing.update(|e| *e = !*e)),
                );
            } else {
                items.push(
                    toolbar_button(day::id!(clear_history), res::str::clear_history_button())
                        .placement(ToolbarPlacement::Bottom)
                        .action(choose_clear_range),
                );
            }
            items
        })
        .grow(Dimension::Both)
}

/// Safari's Clear History choices, as the platform's action sheet: the last hour, today,
/// today and yesterday, all history.
fn choose_clear_range() {
    day::task(async move {
        let picked = Alert::new(res::str::clear_history_prompt())
            .message(res::str::clear_history_message())
            .button(res::str::clear_last_hour(), ClearRange::LastHour)
            .button(res::str::clear_today(), ClearRange::Today)
            .button(res::str::clear_two_days(), ClearRange::TodayAndYesterday)
            .destructive(res::str::clear_all_history(), ClearRange::All)
            .sheet()
            .cancel(res::str::cancel())
            .present()
            .await;
        if let Some(range) = picked {
            settings::clear_history(range);
        }
    });
}

fn bookmarks_list(browser: Browser, query: Signal<String>, editing: Signal<bool>) -> impl Piece {
    let marks = bookmarks();
    let shown = move || {
        let q = query.get().to_lowercase();
        marks
            .get()
            .into_iter()
            .filter(|b| {
                q.is_empty()
                    || b.title.to_lowercase().contains(&q)
                    || b.url.to_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    };
    let filtering = move || !query.get().trim().is_empty();
    zstack((
        list(
            items(shown, |b: &Bookmark| b.id),
            move |slot: ItemSlot<Bookmark, u64>| {
                page_row(
                    move || slot.field(|b| b.title.clone()),
                    move || slot.field(|b| b.url.clone()),
                    None::<fn() -> String>,
                )
                .context_menu(move || bookmark_menu(browser, slot.get()))
                .id_of(move || day::id!(bookmark_row, slot.key()))
            },
        )
        .on_activate(move |id: u64| {
            if let Some(b) = marks.get_untracked().into_iter().find(|b| b.id == id) {
                browser.load_in_current(&b.url, true);
            }
        })
        .deletable(Delete::new(move |index| {
            if let Some(b) = untrack(shown).get(index) {
                remove_bookmark(b.id);
            }
        }))
        .reorderable(
            Reorder::new(move |from, to| {
                if let Some(b) = untrack(shown).get(from) {
                    move_bookmark(b.id, to);
                }
            })
            .enabled(move || !filtering()),
        )
        .editing(editing)
        .inset_grouped()
        .id(day::id!(bookmarks_list)),
        when(
            move || shown().is_empty(),
            move || {
                label(move || {
                    if filtering() {
                        res::str::no_matches().format()
                    } else {
                        res::str::bookmarks_empty().format()
                    }
                })
                .secondary()
                .aligned(Alignment::Center)
                .id(day::id!(bookmarks_empty))
            },
        ),
    ))
}

fn bookmark_menu(browser: Browser, b: Bookmark) -> Vec<MenuEntry> {
    let (url, id) = (b.url.clone(), b.id);
    let copy = b.url.clone();
    vec![
        menu_item(res::str::open_in_new_tab()).action(move || {
            browser.open(&url, true, false, None);
            browser.library.set(None);
        }),
        menu_item(res::str::copy_link()).action(move || {
            day_part_clipboard::set_text(&copy);
        }),
        menu_separator(),
        menu_item(res::str::remove_bookmark()).action(move || remove_bookmark(id)),
    ]
}

/// One history entry, flattened for the list with its day heading.
#[derive(Clone, PartialEq)]
struct Entry {
    page: HistoryPage,
    group: DayGroup,
}

fn history_list(browser: Browser, query: Signal<String>) -> impl Piece {
    let pages = history_pages(query);
    let entries = move || {
        let now = crate::now_ms();
        let offset = crate::utc_offset_secs();
        pages
            .get()
            .into_iter()
            .map(|page| Entry {
                group: history::day_group(page.last_visit, now, offset),
                page,
            })
            .collect::<Vec<_>>()
    };
    zstack((
        list(
            items(entries, |e: &Entry| e.page.id),
            move |slot: ItemSlot<Entry, u64>| {
                row((
                    page_row(
                        move || slot.field(|e| e.page.title.clone()),
                        move || slot.field(|e| e.page.url.clone()),
                        None::<fn() -> String>,
                    )
                    .grow(Dimension::Width),
                    // Without the platform's swipe actions, each row carries its own remove
                    // button, as Chrome's history does.
                    when(
                        move || capability(Cap::ListSwipeActions) == Support::Unsupported,
                        move || {
                            button(res::str::delete_history_item())
                                .icon(Symbol::Close)
                                .icon_only()
                                .plain()
                                .action(move || delete_history(vec![slot.key()]))
                                .id(day::id!(history_remove, slot.key()))
                        },
                    ),
                ))
                .align(VAlign::Center)
                .context_menu(move || {
                    let e = slot.get();
                    let (url, id) = (e.page.url.clone(), e.page.id);
                    let copy = url.clone();
                    vec![
                        menu_item(res::str::open_in_new_tab()).action(move || {
                            browser.open(&url, true, false, None);
                            browser.library.set(None);
                        }),
                        menu_item(res::str::copy_link()).action(move || {
                            day_part_clipboard::set_text(&copy);
                        }),
                        menu_separator(),
                        menu_item(res::str::delete_history_item())
                            .action(move || delete_history(vec![id])),
                    ]
                })
                .id_of(move || day::id!(history_row, slot.key()))
            },
        )
        .sections(
            move |i| {
                untrack(entries)
                    .get(i)
                    .map(|e| e.group)
                    .unwrap_or(DayGroup::Today)
            },
            |slot: ItemSlot<Entry, u64>| {
                label(move || crate::format_day_group(slot.field(|e| e.group)))
                    .font(Font::Footnote)
                    .bold()
            },
        )
        .on_activate(move |id: u64| {
            if let Some(p) = pages.get_untracked().into_iter().find(|p| p.id == id) {
                browser.load_in_current(&p.url, true);
            }
        })
        // Safari's swipe to delete, as a destructive trailing action: a full swipe removes
        // the page, and a day's heading goes with its last page.
        .swipe_trailing(move |i| {
            let id = untrack(entries).get(i).map(|e| e.page.id);
            vec![
                swipe_action(res::str::delete())
                    .destructive()
                    .symbol(Symbol::Delete)
                    .action(move || {
                        if let Some(id) = id {
                            delete_history(vec![id]);
                        }
                    }),
            ]
        })
        .inset_grouped()
        .id(day::id!(history_list)),
        when(
            move || pages.with(Vec::is_empty),
            move || {
                label(move || {
                    if query.get().trim().is_empty() {
                        res::str::history_empty().format()
                    } else {
                        res::str::no_matches().format()
                    }
                })
                .secondary()
                .aligned(Alignment::Center)
                .id(day::id!(history_empty))
            },
        ),
    ))
    .grow(Dimension::Both)
}

/// A page in a list: its icon, title (or address) and address, and an optional trailing detail.
fn page_row<D: Fn() -> String + 'static>(
    title: impl Fn() -> String + 'static,
    url: impl Fn() -> String + Copy + 'static,
    detail: Option<D>,
) -> impl Piece {
    let shown_title = move || {
        let t = title();
        if t.trim().is_empty() {
            address::display(&url(), false)
        } else {
            t
        }
    };
    row((
        site_icon(url, 28.0),
        column((
            label(shown_title).single_line(),
            label(move || address::display(&url(), true))
                .font(Font::Caption)
                .secondary()
                .single_line(),
        ))
        .align(HAlign::Leading)
        .spacing(2.0)
        .grow(Dimension::Width),
        detail.map(|d| label(d).font(Font::Caption).secondary()),
    ))
    .spacing(Space::S)
    .padding(Insets::new(6.0, 4.0, 6.0, 4.0))
}
