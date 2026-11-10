// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Net Skip: a web browser for phones, tablets and computers, built on
//! [Day](https://daybrite.dev) with the platform's own web engine in every tab.
//!
//! - [`browser`]: a window's tabs and the address bar's editing state.
//! - [`chrome`]: the toolbar, tab strip, bottom bar, suggestions and tab grid.
//! - [`library`]: the start page, bookmarks and history.
//! - [`store`]: history and bookmarks on disk.
//! - [`settings`]: the search engine and the other preferences.
//! - [`fixtures`]: browsing from bundled pages, for tests and demos.

use day::prelude::*;

pub mod adblock;
pub mod browser;
pub mod chrome;
mod clock;
pub mod favicons;
pub mod fixtures;
pub mod library;
mod native;
mod scripts;
pub mod settings;
pub mod store;

use browser::Browser;
pub use clock::{now_ms, utc_offset_secs};
use library::Pane;
use netskip_model::history::DayGroup;

// Typed constants for everything under resource/: https://daybrite.dev/docs/how-to/resources
day::resources!();
// The mobile entry point; desktop enters through src/main.rs.
day::day_start!(options: res::window_options(), root);

/// The window's content: the browser, its menus, and Settings.
pub fn root() -> impl Piece {
    fixtures::install();
    adblock::install();
    let _prefs = settings::install();
    // Open the library off the UI thread so the first keystroke finds it ready.
    std::thread::spawn(|| {
        store::worker();
    });
    Browser::scoped(move |browser| {
        app_menu(move || menus(browser));
        // A web link the system hands over (Net Skip as the default browser, a shared link, a
        // link opened from another app) opens in a new tab.
        day::on_open_url(move |url| {
            let web = url::Url::parse(url).is_ok_and(|u| matches!(u.scheme(), "http" | "https"));
            if web {
                browser.open(url, true, false, None);
            }
            web
        });
        chrome::window(browser)
    })
}

/// The menu bar, with the shortcuts every browser shares.
fn menus(browser: Browser) -> Vec<MenuEntry> {
    let has_page = move || browser.current().is_some_and(|t| t.has_page());
    let with_tab = move |f: fn(browser::Tab)| {
        move || {
            if let Some(t) = browser.current_untracked() {
                f(t);
            }
        }
    };
    let mut tabs_menu = vec![
        menu_item(res::str::show_all_tabs())
            .shortcut(Shortcut::primary("\\").shift())
            .action(move || browser.show_overview())
            .id(day::id!(menu_show_tabs)),
        menu_separator(),
        menu_item(res::str::next_tab())
            .shortcut(Shortcut::primary("]").shift())
            .action(move || browser.select_next(false)),
        menu_item(res::str::previous_tab())
            .shortcut(Shortcut::primary("[").shift())
            .action(move || browser.select_next(true)),
        menu_separator(),
    ];
    for i in 0..9 {
        let title = if i == 8 {
            res::str::last_tab()
        } else {
            res::str::tab_number(i as i64 + 1)
        };
        tabs_menu.push(
            menu_item(title)
                .shortcut(Shortcut::primary((i + 1).to_string()))
                .action(move || browser.select_index(i)),
        );
    }
    let mut closed_menu: Vec<MenuEntry> = browser
        .recently_closed()
        .into_iter()
        .take(12)
        .enumerate()
        .map(|(i, (title, url))| {
            let text = if title.trim().is_empty() {
                netskip_model::address::display(&url, true)
            } else {
                title
            };
            menu_item(text).action(move || browser.reopen_closed_at(i))
        })
        .collect();
    if closed_menu.is_empty() {
        closed_menu.push(menu_item(res::str::no_closed_tabs()).enabled(false));
    }
    vec![
        sub_menu(
            res::str::menu_file(),
            vec![
                menu_item(res::str::new_tab())
                    .shortcut(Shortcut::primary("t"))
                    .action(move || browser.new_tab(false))
                    .id(day::id!(menu_new_tab)),
                menu_item(res::str::new_private_tab())
                    .shortcut(Shortcut::primary("n").shift())
                    .action(move || browser.new_tab(true))
                    .id(day::id!(menu_new_private_tab)),
                menu_item(res::str::open_location())
                    .shortcut(Shortcut::primary("l"))
                    .action(move || browser.omnibox.focused.set(true))
                    .id(day::id!(menu_open_location)),
                menu_separator(),
                menu_item(res::str::close_tab())
                    .shortcut(Shortcut::primary("w"))
                    .action(move || browser.close_current())
                    .id(day::id!(menu_close_tab)),
                menu_item(res::str::reopen_closed_tab())
                    .shortcut(Shortcut::primary("t").shift())
                    .enabled(move || browser.has_closed())
                    .action(move || browser.reopen_closed())
                    .id(day::id!(menu_reopen_tab)),
                menu_role(MenuRole::CloseWindow),
                menu_separator(),
                menu_item(res::str::share())
                    .enabled(has_page)
                    .action(move || chrome::share_current(browser)),
            ],
        ),
        sub_menu(
            res::str::menu_edit(),
            vec![
                menu_role(MenuRole::Undo),
                menu_role(MenuRole::Redo),
                menu_separator(),
                menu_role(MenuRole::Cut),
                menu_role(MenuRole::Copy),
                menu_role(MenuRole::Paste),
                menu_role(MenuRole::SelectAll),
                menu_separator(),
                menu_item(res::str::find_on_page())
                    .shortcut(Shortcut::primary("f"))
                    .enabled(has_page)
                    .action(move || chrome::open_find(browser))
                    .id(day::id!(menu_find)),
            ],
        ),
        sub_menu(
            res::str::menu_view(),
            vec![
                menu_item(res::str::reload())
                    .shortcut(Shortcut::primary("r"))
                    .enabled(has_page)
                    .action(with_tab(|t| {
                        if !t.loading() {
                            t.reload_or_stop();
                        }
                    }))
                    .id(day::id!(menu_reload)),
                menu_item(res::str::stop_loading())
                    .shortcut(Shortcut::primary("."))
                    .enabled(move || browser.current().is_some_and(|t| t.loading()))
                    .action(with_tab(|t| {
                        if t.loading() {
                            t.reload_or_stop();
                        }
                    })),
                menu_separator(),
                menu_item(res::str::zoom_in())
                    .shortcut(Shortcut::primary("+"))
                    .enabled(has_page)
                    .action(move || chrome::zoom(browser, 1))
                    .id(day::id!(menu_zoom_in)),
                menu_item(res::str::zoom_out())
                    .shortcut(Shortcut::primary("-"))
                    .enabled(has_page)
                    .action(move || chrome::zoom(browser, -1))
                    .id(day::id!(menu_zoom_out)),
                menu_item(res::str::actual_size())
                    .shortcut(Shortcut::primary("0"))
                    .enabled(has_page)
                    .action(move || chrome::zoom(browser, 0))
                    .id(day::id!(menu_zoom_reset)),
            ],
        ),
        sub_menu(
            res::str::menu_history(),
            vec![
                menu_item(res::str::back())
                    .shortcut(Shortcut::primary("["))
                    .enabled(move || browser.current().is_some_and(|t| t.can_go_back()))
                    .action(with_tab(|t| t.go_back()))
                    .id(day::id!(menu_back)),
                menu_item(res::str::forward())
                    .shortcut(Shortcut::primary("]"))
                    .enabled(move || browser.current().is_some_and(|t| t.can_go_forward()))
                    .action(with_tab(|t| t.go_forward()))
                    .id(day::id!(menu_forward)),
                menu_item(res::str::home())
                    .shortcut(Shortcut::primary("h").shift())
                    .action(move || browser.go_home())
                    .id(day::id!(menu_home)),
                menu_separator(),
                sub_menu(res::str::recently_closed(), closed_menu),
                menu_separator(),
                menu_item(res::str::show_all_history())
                    .shortcut(Shortcut::primary("y"))
                    .action(move || browser.library.set(Some(Pane::History)))
                    .id(day::id!(menu_history)),
            ],
        ),
        sub_menu(
            res::str::menu_bookmarks(),
            vec![
                menu_item(res::str::add_bookmark())
                    .shortcut(Shortcut::primary("d"))
                    .enabled(has_page)
                    .action(move || chrome::bookmark_current(browser))
                    .id(day::id!(menu_add_bookmark)),
                menu_item(res::str::show_bookmarks())
                    .shortcut(Shortcut::primary("b").alt())
                    .action(move || browser.library.set(Some(Pane::Bookmarks)))
                    .id(day::id!(menu_bookmarks)),
            ],
        ),
        sub_menu(res::str::menu_tabs(), tabs_menu),
    ]
}

/// A visit time on the history list: the clock time for today and yesterday, the date before.
pub fn format_time(ms: i64) -> String {
    use day_localization::{DateFields, format_date};
    let offset = utc_offset_secs();
    let local = (ms / 1000 + offset) as f64;
    let days = netskip_model::history::local_day(now_ms(), offset)
        - netskip_model::history::local_day(ms, offset);
    if days <= 6 {
        format_date(local, DateFields::HourMinute)
    } else {
        format_date(local, DateFields::MonthDay)
    }
}

/// A history day heading.
pub fn format_day_group(group: DayGroup) -> String {
    use day_localization::{DateFields, format_date};
    match group {
        DayGroup::Today => res::str::today().format(),
        DayGroup::Yesterday => res::str::yesterday().format(),
        DayGroup::Weekday(w) => match w {
            0 => res::str::monday(),
            1 => res::str::tuesday(),
            2 => res::str::wednesday(),
            3 => res::str::thursday(),
            4 => res::str::friday(),
            5 => res::str::saturday(),
            _ => res::str::sunday(),
        }
        .format(),
        DayGroup::Date(day) => format_date(day as f64 * 86_400.0, DateFields::YearMonthDay),
    }
}
