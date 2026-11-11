// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! The browser's chrome around the pages.
//!
//! On a computer or a tablet: a tab strip, then a toolbar with Back, Forward, the address field
//! and the page's actions, as Chrome, Brave and Vivaldi lay it out. On a phone, Safari's shape:
//! the page fills the screen, and the address field and the five buttons sit at the bottom
//! within reach of a thumb, with the tabs in a grid behind the tabs button. The pages are
//! one `document_tabs` host whatever the width, so a phone rotating or an iPad window resizing
//! changes the chrome and never reloads a page.

use std::rc::Rc;

use day::prelude::*;
use day_piece_webview::{WebProfile, web_view};
use netskip_model::address;
use netskip_model::complete::{Row, RowKind};

use crate::browser::{Browser, Omnibox, Tab, is_blank, row_text};
use crate::library::{self, Pane};
use crate::native::AddressFieldLook;
use crate::{favicons, fixtures, res, scripts, settings};

/// Whether the window is phone-narrow: Safari's bottom bar instead of the desktop toolbar.
/// A tracked read.
pub fn compact() -> bool {
    day::size_class().is_some_and(|c| {
        c.width == WidthClass::Compact || (c.height == HeightClass::Compact && TOUCH)
    })
}

/// A neutral shade over the window background: darker in light mode, lighter in dark mode, so
/// the address bar, the tab strip and the suggestion panel read as layers on every platform's
/// own background color. Tracked: it follows the appearance.
pub fn tone(light: f64, dark: f64) -> Color {
    if day::dark_mode() {
        Color::rgba(1.0, 1.0, 1.0, dark)
    } else {
        Color::rgba(0.0, 0.0, 0.0, light)
    }
}

/// An opaque elevated surface: the window background with a slight lift.
fn elevated() -> Color {
    let bg = day::system_color(SystemColor::Background);
    let lift = if day::dark_mode() { 0.06 } else { 0.0 };
    Color::rgba(bg.r + lift, bg.g + lift, bg.b + lift, 1.0)
}

/// Phones and tablets, where a short landscape window still means a phone held sideways.
const TOUCH: bool = cfg!(any(
    target_os = "ios",
    target_os = "android",
    target_env = "ohos"
));

day::routes! {
    /// The full-screen tab grid.
    pub enum Overview {
        Tabs => "tabs",
    }
}

/// The window's content.
pub fn window(browser: Browser) -> impl Piece {
    let overview = Signal::new(None::<Overview>);
    watch(
        move || browser.overview.get(),
        move |open, _| overview.set(open.then_some(Overview::Tabs)),
    );
    watch(
        move || overview.get(),
        move |open, _| {
            if open.is_none() && browser.overview.get_untracked() {
                browser.overview.set(false);
            }
        },
    );
    zstack((
        document_tabs(
            browser.tabs,
            res::str::new_tab(),
            res::str::close_tab(),
            move |key: &String| browser.tab(key).map(|t| t.label()).unwrap_or_default(),
            move |key: &String| tab_content(browser, key.clone()),
        )
        .native(|| false)
        .on_new(move || browser.new_tab(false))
        .on_close(move |key: String| browser.close(&key))
        .chrome(move |_actions| {
            // As in Safari, the tab bar appears once a second tab opens.
            when(
                move || !compact() && browser.tabs.keys().len() > 1,
                move || tab_strip(browser),
            )
        })
        .layout(move |strip, content| {
            column((
                // Safari's order: the toolbar, then the tab bar beneath it. On a phone, the bar
                // above the page when it has one (Chrome's, or Safari's moved to the top).
                when(move || !compact(), move || toolbar(browser)),
                when(
                    move || compact() && address_on_top(),
                    move || phone_top(browser),
                ),
                strip,
                zstack((
                    content,
                    when(
                        move || browser.omnibox.showing(),
                        move || suggestions(browser),
                    ),
                ))
                .grow(Dimension::Both),
                when(
                    move || browser.current().is_some_and(|t| t.find.open.get()),
                    move || find_bar(browser),
                ),
                // Find in page takes the bottom of a phone's window to itself, as in Safari.
                when(
                    move || {
                        compact()
                            && !(CHROME_LAYOUT && address_on_top())
                            && !browser.current().is_some_and(|t| t.find.open.get())
                    },
                    move || phone_bottom(browser),
                ),
            ))
        })
        .strip_id(day::id!(tabs))
        .grow(Dimension::Both),
        cover(overview, move |_| overview_page(browser))
            .background(|_| day::system_color(SystemColor::Background)),
        cover(browser.library, move |pane: &Pane| {
            library::library_page(browser, *pane)
        })
        .sheet(),
    ))
    .grow(Dimension::Both)
    // Chrome's Back on a phone: the system back goes back in the page's history while the tab
    // can, and leaves the app once it cannot. Covers (the tab grid, the library) close first.
    .on_system_back(
        move || browser.current().is_some_and(|t| t.can_go_back()),
        move || {
            if let Some(t) = browser.current_untracked() {
                t.go_back();
            }
        },
    )
}

/// One tab's document: its web view, with the start page over it when the tab has none.
fn tab_content(browser: Browser, key: String) -> impl Piece {
    let tab = browser.make_tab(&key);
    zstack((
        when(move || tab.has_view.get(), move || page_view(browser, tab)),
        when(
            move || tab.start.get(),
            move || library::start_page(browser, tab),
        ),
    ))
    .grow(Dimension::Both)
}

/// The tab's web view, in the tab's own profile, wired to its commands.
fn page_view(browser: Browser, tab: Tab) -> impl Piece {
    let view = match fixtures::provider() {
        Some(provider) => web_view(tab.location).resources(provider),
        None => web_view(tab.location),
    };
    let profile =
        if tab.private && day_piece_webview::private_browsing_support() != Support::Unsupported {
            WebProfile::private(format!("netskip-private-{}", tab.id))
        } else {
            WebProfile::default()
        };
    // Content blocking: a request filter where the engine asks about each request, the rule
    // lists for the site's level on WebKit, and element hiding everywhere.
    let view = match crate::adblock::filter() {
        Some(filter) => view.request_filter(filter),
        None => view,
    };
    view.profile(profile)
        .content_rules(move || crate::adblock::content_rules(&tab.url()))
        .on_content_rules_error(|e| warn!("content rules rejected: {e:?}"))
        .user_script(crate::adblock::cosmetic_script())
        .on_script_message(
            crate::adblock::SCRIPT_CHANNEL,
            crate::adblock::script_message,
        )
        .on_blocked(move |loads| crate::adblock::note_blocked(tab.blocked, loads))
        .go(tab.go)
        .back(tab.back)
        .forward(tab.forward)
        .reload(tab.reload)
        .stop(tab.stop)
        .js(tab.js)
        .page_state(tab.page)
        .scroll_state(tab.scroll)
        .zoom(move || tab.zoom.get() as f64 / 100.0)
        .desktop_mode(tab.desktop)
        .javascript(move || settings::JAVASCRIPT.get())
        .on_load(move || browser.loaded(tab))
        .on_new_tab(
            res::str::open_in_new_tab(),
            res::str::open_in_background_tab(),
            move |req| browser.open_from_page(tab, &req.url, req.background),
        )
        .id(day::id!(page, tab.id))
        .grow(Dimension::Both)
}

// ---------------------------------------------------------------------------------------------
// Shared pieces
// ---------------------------------------------------------------------------------------------

/// A site's icon: its favicon, or a monogram on the site's own color.
pub fn site_icon(url: impl Fn() -> String + Copy + 'static, size: f64) -> impl Piece {
    switch(
        move || favicons::for_url(&url()),
        move |icon| match icon {
            Some(icon) => image(icon.0.to_vec())
                .frame(size * 0.7, size * 0.7)
                .aligned(Alignment::Center)
                .frame(size, size)
                .background(move || day::system_color(SystemColor::Surface))
                .corner_radius(size * 0.22)
                .any(),
            None => label(move || address::monogram(&url()))
                .font(Font::System(size * 0.45))
                .bold()
                .color(Color::WHITE)
                .aligned(Alignment::Center)
                .frame(size, size)
                .background(move || {
                    Color::hsl(f64::from(address::monogram_hue(&url())), 0.55, 0.48)
                })
                .corner_radius(size * 0.22)
                .any(),
        },
    )
}

/// The small icon in a tab's strip entry: its favicon, the private glyph, or a monogram.
fn tab_icon(tab: Tab, size: f64) -> impl Piece {
    switch(
        move || (tab.icon.get(), tab.has_page(), tab.private),
        move |(icon, page, private)| match (icon, page, private) {
            (Some(icon), true, _) => image(icon.0.to_vec()).frame(size, size).any(),
            (_, false, true) => vector(res::vectors::incognito)
                .tint(Color::hex(0x8e8e93))
                .frame(size, size)
                .any(),
            (_, false, false) => vector(res::vectors::globe)
                .tint(Color::hex(0x8e8e93))
                .frame(size, size)
                .any(),
            (None, true, _) => label(move || address::monogram(&tab.url()))
                .font(Font::System(size * 0.6))
                .bold()
                .color(Color::WHITE)
                .aligned(Alignment::Center)
                .frame(size, size)
                .background(move || {
                    Color::hsl(f64::from(address::monogram_hue(&tab.url())), 0.55, 0.48)
                })
                .corner_radius(size * 0.25)
                .any(),
        },
    )
}

/// A borderless toolbar button with one of the app's own glyphs.
fn tool_glyph(text: LocalizedText, glyph: VectorName, action: impl Fn() + 'static) -> impl Piece {
    button(text).image(glyph).icon_only().plain().action(action)
}

/// The thin progress line: the accent color across the share of the load done.
fn progress_line(browser: Browser, height: f64) -> impl Piece {
    canvas(move |d, size| {
        let Some(tab) = browser.current() else {
            return;
        };
        let p = tab.progress.get();
        if !p.visible {
            return;
        }
        let w = size.width * p.value.clamp(0.0, 1.0);
        d.fill(
            Shape::Rect(Rect::new(0.0, 0.0, w, size.height)),
            day::system_color(SystemColor::Accent),
        );
    })
    .height(height)
    .grow(Dimension::Width)
    .a11y(|a| a.hidden())
    .id(day::id!(progress))
}

// ---------------------------------------------------------------------------------------------
// The address field
// ---------------------------------------------------------------------------------------------

/// The field's text binding: typing goes through the omnibox, which completes and suggests.
#[derive(Clone)]
struct AddressText(Browser);

impl Binding<String> for AddressText {
    fn read(&self) -> String {
        self.0.omnibox.text.get()
    }
    fn peek(&self) -> String {
        self.0.omnibox.text.get_untracked()
    }
    fn write(&self, v: String) {
        let browser = self.0;
        browser.omnibox.edited(browser, v);
    }
}

/// The address field, with the lock (or magnifier) before it and Reload or Clear after it.
/// The address text field itself, shared by the desktop toolbar and the phones' bars: the
/// omnibox's text, completion selection, arrow keys and focus.
fn address_input(browser: Browser) -> impl Piece {
    let omnibox = browser.omnibox;
    // The field shows the page's site at rest and the whole address while editing.
    watch(
        move || (Omnibox::resting_text(browser), omnibox.focused.get()),
        move |(rest, focused), _| {
            if !*focused {
                omnibox.text.set(rest.clone());
            }
        },
    );
    watch(
        move || omnibox.focused.get(),
        move |focused, previous| {
            if *focused && previous == Some(&false) {
                omnibox.begin_editing(browser);
            } else if !*focused && previous == Some(&true) {
                omnibox.end_editing(browser);
            }
        },
    );
    text_field(AddressText(browser))
        .placeholder(move || {
            let engine = settings::engine();
            res::str::address_placeholder(settings::engine_name(&engine)).format()
        })
        .input_purpose(InputPurpose::Url)
        .submit_label(SubmitLabel::Go)
        .on_submit(move || omnibox.submit(browser))
        .select(omnibox.select_seq, move || {
            let (start, end) = omnibox.selection.get_untracked();
            start..end
        })
        .on_key(move |key| match key.key.as_str() {
            "ArrowUp" => {
                omnibox.arrow(false);
            }
            "ArrowDown" => {
                omnibox.arrow(true);
            }
            "Escape" => omnibox.escape(browser),
            _ => {}
        })
        .focused(omnibox.focused)
        .frameless()
        .id(day::id!(address))
        .grow(Dimension::Width)
}

fn address_field(browser: Browser) -> impl Piece {
    let omnibox = browser.omnibox;
    let secure = move || {
        browser
            .current()
            .is_some_and(|t| t.has_page() && address::is_secure(&t.url()))
    };
    let leading = switch(
        move || (omnibox.focused.get(), secure()),
        move |(editing, secure)| match (editing, secure) {
            (false, true) => vector(res::vectors::lock)
                .tint(Color::hex(0x8e8e93))
                .frame(14.0, 14.0)
                .a11y(|a| a.label(res::str::secure_connection()))
                .id(day::id!(address_lock))
                .any(),
            _ => vector(res::vectors::search)
                .tint(Color::hex(0x8e8e93))
                .frame(16.0, 16.0)
                .a11y(|a| a.hidden())
                .any(),
        },
    );
    let field = address_input(browser);
    let trailing = switch(
        move || {
            let editing = omnibox.focused.get();
            let has_text = !omnibox.text.with(String::is_empty);
            let page = browser.current().is_some_and(|t| t.has_page());
            let loading = browser.current().is_some_and(|t| t.loading());
            (editing && has_text, !editing && page, loading)
        },
        move |(clear, reload, loading)| {
            if *clear {
                button(res::str::clear_text())
                    .icon(Symbol::Close)
                    .icon_only()
                    .plain()
                    .action(move || omnibox.edited(browser, String::new()))
                    .id(day::id!(address_clear))
                    .any()
            } else if *reload {
                button(if *loading {
                    res::str::stop_loading()
                } else {
                    res::str::reload()
                })
                .icon(if *loading {
                    Symbol::Close
                } else {
                    Symbol::Refresh
                })
                .icon_only()
                .plain()
                .action(move || {
                    if let Some(t) = browser.current_untracked() {
                        t.reload_or_stop();
                    }
                })
                .id(day::id!(reload))
                .any()
            } else {
                spacer().width(0.0).any()
            }
        },
    );
    row((leading, field, trailing))
        .spacing(6.0)
        .align(VAlign::Center)
        .padding(Insets::new(5.0, 10.0, 5.0, 8.0))
        .background(move || tone(0.06, 0.09))
        .corner_radius(10.0)
        .overlay_aligned(
            Alignment::Bottom,
            progress_line(browser, 2.5).padding(Insets::new(0.0, 8.0, 0.0, 8.0)),
        )
        .id(day::id!(address_bar))
}

// ---------------------------------------------------------------------------------------------
// Desktop and tablet
// ---------------------------------------------------------------------------------------------

/// The tab strip: one entry per tab with its icon, title and close button, then New Tab.
fn tab_strip(browser: Browser) -> impl Piece {
    row((row((each(
        items(move || browser.tabs.keys(), |k: &String| k.clone()),
        move |slot: ItemSlot<String, String>| {
            // The strip builds before a new tab's document registers its state.
            let key = slot.key();
            keyed(
                move || browser.tab(&key).map(|t| t.id),
                move |id: &u64| tab_chip(browser, id.to_string()),
            )
        },
    ),))
    .spacing(4.0)
    .fit(RowFit::Scroll)
    .height(36.0)
    .grow(Dimension::Width),))
    .spacing(4.0)
    .align(VAlign::Center)
    .padding(Insets::new(6.0, 8.0, 4.0, 8.0))
    .background(move || tone(0.07, 0.0))
    .background(move || {
        if day::dark_mode() {
            Color::rgba(0.0, 0.0, 0.0, 0.35)
        } else {
            Color::CLEAR
        }
    })
    .id(day::id!(tab_strip))
}

fn tab_chip(browser: Browser, key: String) -> impl Piece {
    let id: u64 = key.parse().unwrap_or_default();
    let selected = move || browser.tabs.selected().and_then(|k| k.parse().ok()) == Some(id);
    let hovered = Signal::new(false);
    let Some(t) = untrack(|| browser.tab(&key)) else {
        return spacer().width(0.0).any();
    };
    // Safari's tab: the close button at the leading edge while the pointer is over the tab,
    // the icon and title centered in what remains.
    let close = switch(
        move || hovered.get(),
        move |shown| {
            if *shown {
                vector(res::vectors::close)
                    .tint(Color::hex(0x8e8e93))
                    .frame(14.0, 14.0)
                    .padding(3.0)
                    .on_tap(move || browser.close(&id.to_string()))
                    .a11y(|a| a.role(Role::Button).label(res::str::close_tab()))
                    .id(day::id!(tab_close, id))
                    .any()
            } else {
                spacer().frame(20.0, 20.0).any()
            }
        },
    );
    row((
        close,
        row((
            tab_icon(t, 16.0),
            label(move || t.label())
                .font(Font::Subheadline)
                .single_line(),
        ))
        .spacing(6.0)
        .align(VAlign::Center)
        .aligned(Alignment::Center)
        .grow(Dimension::Width),
        spacer().frame(20.0, 20.0),
    ))
    .spacing(4.0)
    .align(VAlign::Center)
    .padding(Insets::new(4.0, 6.0, 4.0, 6.0))
    .min_width(140.0)
    .height(30.0)
    .background(move || {
        if selected() {
            tone(0.0, 0.10)
        } else if hovered.get() {
            tone(0.05, 0.05)
        } else {
            Color::CLEAR
        }
    })
    .background(move || {
        if selected() && !day::dark_mode() {
            Color::WHITE
        } else {
            Color::CLEAR
        }
    })
    .corner_radius(7.0)
    .on_hover(move |at| hovered.set(at.is_some()))
    .on_tap(move || {
        browser.select(&id.to_string());
    })
    .context_menu(move || tab_menu(browser, id.to_string()))
    .a11y(move |a| a.role(Role::Tab).label(move || t.label()))
    .id(day::id!(tab, id))
    .grow(Dimension::Width)
    .any()
}

fn tab_menu(browser: Browser, key: String) -> Vec<MenuEntry> {
    let (k1, k2, k3) = (key.clone(), key.clone(), key.clone());
    vec![
        menu_item(res::str::duplicate_tab()).action(move || browser.duplicate(&k1)),
        menu_separator(),
        menu_item(res::str::close_tab()).action(move || browser.close(&k2)),
        menu_item(res::str::close_other_tabs()).action(move || browser.close_others(&k3)),
        menu_item(res::str::reopen_closed_tab())
            .enabled(move || browser.has_closed())
            .action(move || browser.reopen_closed()),
    ]
}

/// The toolbar above the pages on a computer or a tablet.
fn toolbar(browser: Browser) -> impl Piece {
    let marks = library::bookmarks();
    let current = move || browser.current();
    row((
        button(res::str::back())
            .icon(Symbol::Back)
            .icon_only()
            .plain()
            .enabled(move || current().is_some_and(|t| t.can_go_back()))
            .action(move || {
                if let Some(t) = browser.current_untracked() {
                    t.go_back();
                }
            })
            .id(day::id!(back)),
        button(res::str::forward())
            .icon(Symbol::Forward)
            .icon_only()
            .plain()
            .enabled(move || current().is_some_and(|t| t.can_go_forward()))
            .action(move || {
                if let Some(t) = browser.current_untracked() {
                    t.go_forward();
                }
            })
            .id(day::id!(forward)),
        button(res::str::home())
            .icon(Symbol::Home)
            .icon_only()
            .plain()
            .action(move || browser.go_home())
            .id(day::id!(home)),
        address_field(browser)
            .max_width(820.0)
            .aligned(Alignment::Center)
            .grow(Dimension::Width),
        button(move || {
            let on = current().is_some_and(|t| library::is_bookmarked(marks, &t.url()));
            if on {
                res::str::remove_bookmark().format()
            } else {
                res::str::add_bookmark().format()
            }
        })
        .icon(move || {
            if current().is_some_and(|t| library::is_bookmarked(marks, &t.url())) {
                Symbol::Star
            } else {
                Symbol::Bookmark
            }
        })
        .icon_only()
        .plain()
        .enabled(move || current().is_some_and(|t| t.has_page()))
        .action(move || bookmark_current(browser))
        .id(day::id!(bookmark)),
        button(res::str::share())
            .icon(Symbol::Share)
            .icon_only()
            .plain()
            .enabled(move || {
                current().is_some_and(|t| t.has_page())
                    && day::share_support() != Support::Unsupported
            })
            .action(move || share_current(browser))
            .id(day::id!(share)),
        tool_glyph(
            res::str::show_library(),
            res::vectors::bookmarks,
            move || browser.library.set(Some(Pane::Bookmarks)),
        )
        .id(day::id!(library)),
        button(res::str::new_tab())
            .icon(Symbol::Add)
            .icon_only()
            .plain()
            .action(move || browser.new_tab(false))
            .id(day::id!(tab_new)),
        tool_glyph(res::str::show_all_tabs(), res::vectors::tabs, move || {
            browser.show_overview()
        })
        .id(day::id!(tabs_button)),
        menu_button(res::str::more(), move || page_menu(browser))
            .icon(Symbol::More)
            .icon_only()
            .plain()
            .id(day::id!(more)),
    ))
    .spacing(8.0)
    .align(VAlign::Center)
    .padding(Insets::new(6.0, 10.0, 6.0, 10.0))
    .background(move || day::system_color(SystemColor::Background))
    .id(day::id!(toolbar))
}

/// Bookmark the selected tab's page, or remove its bookmark.
pub fn bookmark_current(browser: Browser) {
    if let Some(t) = browser.current_untracked()
        && t.has_page()
    {
        let url = t.url_untracked();
        let reported = t.title.get_untracked();
        let js = t.js;
        // The title comes from the page itself: the engine's report of a new page's title can
        // trail its address, and a page bookmarked just as it loaded took the title of the page
        // before it. The tab's title stands in where the page cannot be asked.
        day::task(async move {
            let title = js
                .eval("document.title")
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<String>(&json).ok())
                .filter(|title| !title.trim().is_empty())
                .unwrap_or(reported);
            library::toggle_bookmark(url, title);
        });
    }
}

/// Share the selected tab's page through the system share sheet, which also carries Safari's
/// page actions where the platform's sheet can (Add Bookmark, Find on Page).
pub fn share_current(browser: Browser) {
    if let Some(t) = browser.current_untracked()
        && t.has_page()
    {
        let url = t.url_untracked();
        let marked = library::is_bookmarked(library::bookmarks(), &url);
        let bookmark = if marked {
            share_action(res::str::remove_bookmark())
        } else {
            share_action(res::str::add_bookmark())
        };
        let _ = day::Share::url(url)
            .title(t.title.get_untracked())
            .action(
                bookmark
                    .symbol(Symbol::Bookmark)
                    .action(move || bookmark_current(browser)),
            )
            .action(
                share_action(res::str::find_on_page())
                    .symbol(Symbol::Search)
                    .action(move || open_find(browser)),
            )
            .anchor(day::id!(share))
            .present();
    }
}

/// Open find in page on the selected tab.
pub fn open_find(browser: Browser) {
    if let Some(t) = browser.current_untracked()
        && t.has_page()
    {
        t.find.open.set(true);
    }
}

/// The page menu (⋯): find, zoom, copy the link, settings.
fn page_menu(browser: Browser) -> Vec<MenuEntry> {
    let has_page = move || browser.current().is_some_and(|t| t.has_page());
    vec![
        menu_item(res::str::new_tab())
            .id(day::id!(more_new_tab))
            .action(move || browser.new_tab(false)),
        menu_item(res::str::new_private_tab())
            .id(day::id!(more_private_tab))
            .action(move || browser.new_tab(true)),
        menu_separator(),
        menu_item(res::str::find_on_page())
            .id(day::id!(more_find))
            .enabled(has_page)
            .action(move || open_find(browser)),
        menu_item(res::str::zoom_in())
            .id(day::id!(more_zoom_in))
            .enabled(has_page)
            .action(move || zoom(browser, 1)),
        menu_item(res::str::zoom_out())
            .id(day::id!(more_zoom_out))
            .enabled(has_page)
            .action(move || zoom(browser, -1)),
        menu_item(move || {
            let z = browser.current().map(|t| t.zoom.get()).unwrap_or(100);
            res::str::actual_size_percent(z).format()
        })
        .id(day::id!(more_zoom_reset))
        .enabled(has_page)
        .action(move || zoom(browser, 0)),
        menu_separator(),
        // Phones and tablets ask sites for their desktop pages; a desktop already gets them.
        menu_item(move || {
            if browser.current().is_some_and(|t| t.desktop.get()) {
                res::str::request_mobile_site().format()
            } else {
                res::str::request_desktop_site().format()
            }
        })
        .id(day::id!(more_desktop_site))
        .enabled(move || has_page() && TOUCH)
        .action(move || toggle_desktop(browser)),
        sub_menu(res::str::blocking_this_site(), blocking_menu(browser)),
        menu_item(res::str::hide_toolbar())
            .id(day::id!(more_hide_toolbar))
            .enabled(move || has_page() && compact())
            .action(move || {
                with_animation(AnimSpec::ease_out(220), || browser.chrome_hidden.set(true))
            }),
        menu_separator(),
        menu_item(res::str::copy_link())
            .id(day::id!(more_copy_link))
            .enabled(has_page)
            .action(move || {
                if let Some(t) = browser.current_untracked() {
                    day_part_clipboard::set_text(&t.url_untracked());
                }
            }),
        menu_item(res::str::history())
            .id(day::id!(more_history))
            .action(move || browser.library.set(Some(Pane::History))),
        menu_separator(),
        menu_item(res::str::settings())
            .id(day::id!(more_settings))
            .action(|| {
                day_piece_settings::open();
            }),
    ]
}

/// Step the page zoom: `1` in, `-1` out, `0` back to the default.
pub fn zoom(browser: Browser, step: i32) {
    let Some(t) = browser.current_untracked() else {
        return;
    };
    let now = t.zoom.get_untracked();
    let steps = settings::ZOOM_STEPS;
    let next = match step {
        0 => settings::PAGE_ZOOM.get_untracked(),
        s if s > 0 => steps.iter().copied().find(|z| *z > now).unwrap_or(now),
        _ => steps
            .iter()
            .rev()
            .copied()
            .find(|z| *z < now)
            .unwrap_or(now),
    };
    t.set_zoom(next);
}

// ---------------------------------------------------------------------------------------------
// Suggestions
// ---------------------------------------------------------------------------------------------

/// The suggestion list under the address field: the whole page on a phone, a dropdown under
/// the field on a wider window.
fn suggestions(browser: Browser) -> impl Piece {
    switch(compact, move |phone| {
        if *phone {
            scroll(suggestion_rows(browser))
                .background(move || day::system_color(SystemColor::Background))
                .grow(Dimension::Both)
                .id(day::id!(suggestions))
                .any()
        } else {
            // At most a dozen rows: the panel sizes to them, as a desktop browser's does.
            suggestion_rows(browser)
                .background(elevated)
                .corner_radius(12.0)
                .overlay(
                    rounded_rectangle(12.0)
                        .stroke(tone(0.12, 0.14), 1.0)
                        .grow(Dimension::Both),
                )
                .grow(Dimension::Width)
                .max_width(820.0)
                .padding(Insets::new(4.0, 12.0, 0.0, 12.0))
                .aligned(Alignment::Top)
                .grow(Dimension::Both)
                .id(day::id!(suggestions))
                .any()
        }
    })
}

fn suggestion_rows(browser: Browser) -> impl Piece {
    let omnibox = browser.omnibox;
    let rows = move || {
        omnibox
            .rows
            .get()
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>()
    };
    column((each(
        items(rows, |(i, r): &(usize, Row)| (*i, row_text(r))),
        move |slot: ItemSlot<(usize, Row), (usize, String)>| {
            let index = slot.key().0;
            suggestion_row(browser, index, move || slot.field(|(_, r)| r.clone()))
        },
    ),))
    .spacing(0.0)
    .padding(Insets::new(4.0, 0.0, 4.0, 0.0))
}

fn suggestion_row(
    browser: Browser,
    index: usize,
    item: impl Fn() -> Row + Copy + 'static,
) -> impl Piece {
    let omnibox = browser.omnibox;
    let highlighted = move || omnibox.highlight.get() == Some(index);
    let engine_name = move || settings::engine_name(&settings::engine());
    let (title, subtitle) = (
        move || match item() {
            Row::Page { title, url, .. } => {
                if title.trim().is_empty() {
                    address::display(&url, true)
                } else {
                    title
                }
            }
            Row::Search { terms, .. } => terms,
            Row::Visit { url } => address::display(&url, true),
        },
        move || match item() {
            Row::Page {
                url,
                kind: RowKind::OpenTab,
                ..
            } => res::str::switch_to_tab(address::display(&url, false)).format(),
            Row::Page { url, .. } => address::display(&url, true),
            Row::Search {
                suggested: false, ..
            } => res::str::search_with(engine_name()).format(),
            Row::Search {
                suggested: true, ..
            } => String::new(),
            Row::Visit { .. } => res::str::go_to_site().format(),
        },
    );
    let glyph = move || match item() {
        Row::Page {
            kind: RowKind::TopHit,
            ..
        } => res::vectors::star_fill,
        Row::Page {
            kind: RowKind::Bookmark,
            ..
        } => res::vectors::star,
        Row::Page {
            kind: RowKind::OpenTab,
            ..
        } => res::vectors::tab_switch,
        Row::Page {
            kind: RowKind::History,
            ..
        } => res::vectors::history,
        Row::Search { .. } => res::vectors::search,
        Row::Visit { .. } => res::vectors::globe,
    };
    row((
        switch(glyph, |g| {
            vector(g.clone())
                .tint(Color::hex(0x8e8e93))
                .frame(18.0, 18.0)
                .any()
        }),
        column((
            label(title).single_line(),
            when(
                move || !subtitle().is_empty(),
                move || {
                    label(subtitle)
                        .font(Font::Caption)
                        .secondary()
                        .single_line()
                },
            ),
        ))
        .align(HAlign::Leading)
        .spacing(1.0)
        .grow(Dimension::Width),
        when(
            move || {
                matches!(
                    item(),
                    Row::Search {
                        suggested: true,
                        ..
                    }
                )
            },
            move || {
                button(res::str::fill_in_search())
                    .image(res::vectors::insert_text)
                    .icon_only()
                    .plain()
                    .action(move || {
                        let text = row_text(&item());
                        omnibox.edited(browser, format!("{text} "));
                    })
            },
        ),
    ))
    .spacing(10.0)
    .align(VAlign::Center)
    .padding(Insets::new(8.0, 14.0, 8.0, 14.0))
    .background(move || {
        if highlighted() {
            day::system_color(SystemColor::Accent).with_alpha(0.22)
        } else {
            Color::CLEAR
        }
    })
    .on_tap(move || omnibox.choose(browser, item()))
    .id(day::id!(suggestion, index))
}

// ---------------------------------------------------------------------------------------------
// Find in page
// ---------------------------------------------------------------------------------------------

fn find_bar(browser: Browser) -> impl Piece {
    let Some(tab) = untrack(|| browser.current()) else {
        return spacer().height(0.0).any();
    };
    let find = tab.find;
    let run = move |script: String| {
        let js = tab.js;
        day::task(async move {
            if let Ok(reply) = js.eval(script).await {
                find.matches.set(scripts::parse_matches(&reply));
            }
        });
    };
    watch(move || find.query.get(), move |q, _| run(scripts::find(q)));
    let close = move || {
        find.open.set(false);
        find.query.set(String::new());
        let js = tab.js;
        day::task(async move {
            let _ = js.eval(scripts::find_clear()).await;
        });
    };
    row((
        text_field(find.query)
            .placeholder(res::str::find_placeholder())
            .submit_label(SubmitLabel::Search)
            .on_submit(move || run(scripts::find_next()))
            .on_key(move |key| match key.key.as_str() {
                "ArrowDown" => run(scripts::find_next()),
                "ArrowUp" => run(scripts::find_previous()),
                "Escape" => close(),
                _ => {}
            })
            .focused(Signal::new(true))
            .id(day::id!(find_field))
            .grow(Dimension::Width),
        label(move || {
            let (current, total) = find.matches.get();
            if find.query.with(|q| q.is_empty()) {
                String::new()
            } else if total == 0 {
                res::str::find_none().format()
            } else {
                res::str::find_count(res::str::FindCount {
                    current: i64::from(current),
                    total: i64::from(total),
                })
                .format()
            }
        })
        .font(Font::Caption)
        .secondary()
        .id(day::id!(find_count)),
        button(res::str::find_previous())
            .image(res::vectors::arrow_up)
            .icon_only()
            .plain()
            .enabled(move || find.matches.get().1 > 0)
            .action(move || run(scripts::find_previous()))
            .id(day::id!(find_previous)),
        button(res::str::find_next())
            .image(res::vectors::arrow_down)
            .icon_only()
            .plain()
            .enabled(move || find.matches.get().1 > 0)
            .action(move || run(scripts::find_next()))
            .id(day::id!(find_next)),
        button(res::str::done())
            .action(close)
            .id(day::id!(find_done)),
    ))
    .spacing(6.0)
    .align(VAlign::Center)
    .padding(Insets::new(6.0, 10.0, 6.0, 10.0))
    .background(move || day::system_color(SystemColor::Surface))
    .id(day::id!(find_bar))
    .any()
}

// ---------------------------------------------------------------------------------------------
// Phone
// ---------------------------------------------------------------------------------------------

// ---------------------------------------------------------------------------------------------
// Phones: Safari's bars on iOS, Chrome's on Android
// ---------------------------------------------------------------------------------------------

/// Chrome's layout on Android and HarmonyOS: the omnibox across the top with Home, the tab count
/// and the ⋮ menu, and no bottom toolbar (the system's Back takes the page back). Safari's
/// elsewhere: the address bar over five buttons at the bottom.
pub const CHROME_LAYOUT: bool = cfg!(any(target_os = "android", target_env = "ohos"));

/// The phone's address bar sits at the top of the window (Settings > Tabs). A tracked read.
pub fn address_on_top() -> bool {
    settings::ADDRESS_AT_TOP.get()
}

/// The bars above the page on a phone: Chrome's toolbar, or Safari's address bar when the user
/// moved it to the top. Gone while the bars are hidden.
fn phone_top(browser: Browser) -> impl Piece {
    let hidden = browser.chrome_hidden;
    switch(
        move || {
            (
                hidden.get() && !browser.omnibox.focused.get(),
                CHROME_LAYOUT,
            )
        },
        move |(hidden, chrome)| match (hidden, chrome) {
            (false, true) => chrome_bar(browser).any(),
            (true, true) => spacer().height(0.0).any(),
            (false, false) => safari_pill_bar(browser).any(),
            (true, false) => mini_bar(browser).any(),
        },
    )
    .id(day::id!(top_bar))
}

/// The bars under the page on a phone: Safari's address bar and toolbar (or only the toolbar
/// when the address bar is at the top), Chrome's toolbar when the user moved it down.
fn phone_bottom(browser: Browser) -> impl Piece {
    let hidden = browser.chrome_hidden;
    let omnibox = browser.omnibox;
    // Editing never rebuilds the bar: the field would lose its focus with it. The toolbar under
    // Safari's address bar follows editing on its own.
    switch(
        move || {
            (
                hidden.get() && !omnibox.focused.get(),
                CHROME_LAYOUT,
                address_on_top(),
            )
        },
        move |(hidden, chrome, top)| match (hidden, chrome, top) {
            (true, true, _) => spacer().height(0.0).any(),
            (false, true, _) => chrome_bar(browser).any(),
            // Safari tucks its bars into the page's site name at the bottom edge.
            (true, false, false) => mini_bar(browser).any(),
            (true, false, true) => spacer().height(0.0).any(),
            (false, false, true) => safari_toolbar(browser).any(),
            (false, false, false) => column((
                safari_pill_bar(browser),
                when(
                    move || !omnibox.focused.get(),
                    move || safari_toolbar(browser),
                ),
            ))
            .any(),
        },
    )
    .background(bar_color)
    .id(day::id!(bottom_bar))
}

/// The bars' backdrop: the grouped background Safari's bars sit on.
fn bar_color() -> Color {
    if day::dark_mode() {
        Color::hex(0x1c1c1e)
    } else {
        Color::hex(0xf2f2f7)
    }
}

/// The address pill's own color: white over the light bar, a lifted gray over the dark one.
fn pill_color() -> Color {
    if day::dark_mode() {
        Color::hex(0x3a3a3c)
    } else {
        Color::hex(0xffffff)
    }
}

/// The site the page is on, as the bars show it at rest: `en.wikipedia.org`.
fn resting_site(browser: Browser) -> String {
    match browser.current() {
        Some(t) if t.has_page() => address::display(&t.url(), false),
        _ => String::new(),
    }
}

/// Safari's address bar: a floating pill holding the page menu, the site (or the field while
/// editing) and Reload, with Cancel beside it while editing.
fn safari_pill_bar(browser: Browser) -> impl Piece {
    let omnibox = browser.omnibox;
    let editing = move || omnibox.focused.get();
    let has_page = move || browser.current().is_some_and(|t| t.has_page());
    let secure = move || {
        browser
            .current()
            .is_some_and(|t| t.has_page() && address::is_secure(&t.url()))
    };
    // At rest the site name sits centered over the field, as Safari shows it; a tap starts
    // editing in the field underneath.
    let resting = move || {
        row((
            spacer(),
            when(secure, || {
                vector(res::vectors::lock)
                    .tint(Color::hex(0x8e8e93))
                    .frame(12.0, 12.0)
                    .a11y(|a| a.label(res::str::secure_connection()))
            }),
            label(move || {
                let site = resting_site(browser);
                if site.is_empty() {
                    let engine = settings::engine();
                    res::str::address_placeholder(settings::engine_name(&engine)).format()
                } else {
                    site
                }
            })
            .single_line()
            .color(move || {
                if has_page() {
                    day::system_color(SystemColor::Text)
                } else {
                    day::system_color(SystemColor::SecondaryText)
                }
            }),
            spacer(),
        ))
        .spacing(4.0)
        .align(VAlign::Center)
        .background(pill_color)
        .on_tap(move || omnibox.focused.set(true))
        .id(day::id!(address_resting))
    };
    let pill = row((
        when(
            move || !editing(),
            move || {
                menu_button(res::str::page_menu(), move || page_menu(browser))
                    .icon(Symbol::More)
                    .icon_only()
                    .plain()
                    .enabled(has_page)
                    .id(day::id!(page_menu))
            },
        ),
        zstack((address_input(browser), when(move || !editing(), resting))).grow(Dimension::Width),
        switch(
            move || {
                let text = !omnibox.text.with(String::is_empty);
                let loading = browser.current().is_some_and(|t| t.loading());
                (editing(), text, has_page(), loading)
            },
            move |(editing, text, page, loading)| {
                if *editing && *text {
                    button(res::str::clear_text())
                        .icon(Symbol::Close)
                        .icon_only()
                        .plain()
                        .action(move || omnibox.edited(browser, String::new()))
                        .id(day::id!(address_clear))
                        .any()
                } else if !*editing && *page {
                    button(if *loading {
                        res::str::stop_loading()
                    } else {
                        res::str::reload()
                    })
                    .icon(if *loading {
                        Symbol::Close
                    } else {
                        Symbol::Refresh
                    })
                    .icon_only()
                    .plain()
                    .action(move || {
                        if let Some(t) = browser.current_untracked() {
                            t.reload_or_stop();
                        }
                    })
                    .id(day::id!(reload))
                    .any()
                } else {
                    spacer().width(0.0).any()
                }
            },
        ),
    ))
    .spacing(4.0)
    .align(VAlign::Center)
    .height(44.0)
    .padding(Insets::new(0.0, 6.0, 0.0, 10.0))
    .background(pill_color)
    .corner_radius(12.0)
    .overlay(
        rounded_rectangle(12.0)
            .stroke(tone(0.08, 0.1), 0.5)
            .grow(Dimension::Both),
    )
    .overlay_aligned(
        Alignment::Bottom,
        progress_line(browser, 2.5).padding(Insets::new(0.0, 14.0, 0.0, 14.0)),
    )
    .grow(Dimension::Width)
    .id(day::id!(address_bar));
    row((
        pill,
        when(editing, move || {
            button(res::str::cancel())
                .plain()
                .action(move || omnibox.end_editing(browser))
                .id(day::id!(address_cancel))
        }),
    ))
    .spacing(10.0)
    .align(VAlign::Center)
    .padding(Insets::new(8.0, 20.0, 6.0, 20.0))
    .background(bar_color)
}

/// One of the toolbar's five buttons: the same glyph size and the same share of the width each,
/// so they sit evenly as Safari's do.
fn bar_button(
    id: ElementId,
    text: LocalizedText,
    symbol: Symbol,
    enabled: impl Fn() -> bool + 'static,
    action: impl Fn() + 'static,
    menu: Option<Rc<dyn Fn() -> Vec<MenuEntry>>>,
) -> impl Piece {
    let b = button(text)
        .icon(symbol)
        .icon_only()
        .plain()
        .enabled(enabled)
        .action(action);
    // The id names the button itself, so it goes on before any modifier that wraps it.
    match menu {
        Some(menu) => b
            .context_menu(move || menu())
            .id(id)
            .grow(Dimension::Width)
            .any(),
        None => b.id(id).grow(Dimension::Width).any(),
    }
}

/// Safari's toolbar: Back, Forward, Share, Bookmarks and Tabs, spread evenly across the bar.
fn safari_toolbar(browser: Browser) -> impl Piece {
    let current = move || browser.current();
    row((
        bar_button(
            day::id!(back),
            res::str::back(),
            Symbol::Back,
            move || current().is_some_and(|t| t.can_go_back()),
            move || {
                if let Some(t) = browser.current_untracked() {
                    t.go_back();
                }
            },
            None,
        ),
        bar_button(
            day::id!(forward),
            res::str::forward(),
            Symbol::Forward,
            move || current().is_some_and(|t| t.can_go_forward()),
            move || {
                if let Some(t) = browser.current_untracked() {
                    t.go_forward();
                }
            },
            None,
        ),
        bar_button(
            day::id!(share),
            res::str::share(),
            Symbol::Share,
            move || current().is_some_and(|t| t.has_page()),
            move || share_current(browser),
            Some(Rc::new(move || page_menu(browser))),
        ),
        bar_button(
            day::id!(library),
            res::str::show_library(),
            Symbol::Book,
            || true,
            move || browser.library.set(Some(Pane::Bookmarks)),
            Some(Rc::new(move || {
                vec![
                    menu_item(res::str::add_bookmark()).action(move || bookmark_current(browser)),
                    menu_item(res::str::history())
                        .action(move || browser.library.set(Some(Pane::History))),
                    menu_item(res::str::find_on_page()).action(move || open_find(browser)),
                    menu_item(res::str::settings()).action(|| {
                        day_piece_settings::open();
                    }),
                ]
            })),
        ),
        bar_button(
            day::id!(tabs_button),
            res::str::show_all_tabs(),
            Symbol::Tabs,
            || true,
            move || browser.show_overview(),
            Some(Rc::new(move || tabs_menu(browser))),
        ),
    ))
    .align(VAlign::Center)
    .height(44.0)
    .padding(Insets::new(0.0, 8.0, 2.0, 8.0))
    .background(bar_color)
}

/// The tabs button's long-press menu: new tabs, and closing this one.
fn tabs_menu(browser: Browser) -> Vec<MenuEntry> {
    vec![
        menu_item(res::str::new_tab()).action(move || browser.new_tab(false)),
        menu_item(res::str::new_private_tab()).action(move || browser.new_tab(true)),
        menu_separator(),
        menu_item(res::str::close_tab()).action(move || browser.close_current()),
    ]
}

/// The bars tucked away while reading: only the site's name, small, at the window's edge. A tap
/// brings the bars back.
fn mini_bar(browser: Browser) -> impl Piece {
    label(move || resting_site(browser))
        .font(Font::Caption)
        .single_line()
        .align(TextAlign::Center)
        .grow(Dimension::Width)
        .padding(Insets::new(6.0, 16.0, 4.0, 16.0))
        .background(bar_color)
        .on_tap(move || {
            with_animation(AnimSpec::ease_out(220), || browser.chrome_hidden.set(false))
        })
        .a11y(|a| a.role(Role::Button).label(res::str::show_toolbar()))
        .id(day::id!(mini_bar))
}

/// Chrome's toolbar: Home, the omnibox (the site, or the field while editing), the tab count
/// and the ⋮ menu. While editing, the omnibox takes the whole bar.
fn chrome_bar(browser: Browser) -> impl Piece {
    let omnibox = browser.omnibox;
    let editing = move || omnibox.focused.get();
    let count = move || browser.tabs.keys().len();
    let secure = move || {
        browser
            .current()
            .is_some_and(|t| t.has_page() && address::is_secure(&t.url()))
    };
    let omnibox_pill = row((
        switch(
            move || (editing(), secure()),
            move |(editing, secure)| match (editing, secure) {
                (false, true) => vector(res::vectors::lock)
                    .tint(day::system_color(SystemColor::SecondaryText))
                    .frame(16.0, 16.0)
                    .a11y(|a| a.label(res::str::secure_connection()))
                    .id(day::id!(address_lock))
                    .any(),
                _ => vector(res::vectors::search)
                    .tint(day::system_color(SystemColor::SecondaryText))
                    .frame(18.0, 18.0)
                    .a11y(|a| a.hidden())
                    .any(),
            },
        ),
        address_input(browser),
        when(
            move || editing() && !omnibox.text.with(String::is_empty),
            move || {
                button(res::str::clear_text())
                    .icon(Symbol::Close)
                    .icon_only()
                    .plain()
                    .action(move || omnibox.edited(browser, String::new()))
                    .id(day::id!(address_clear))
            },
        ),
    ))
    .spacing(10.0)
    .align(VAlign::Center)
    .height(40.0)
    .padding(Insets::new(0.0, 14.0, 0.0, 10.0))
    // Chrome's omnibox: the theme's container tone on the bar's surface.
    .background(move || day::system_color(SystemColor::Surface))
    .corner_radius(20.0)
    .overlay_aligned(
        Alignment::Bottom,
        progress_line(browser, 2.5).padding(Insets::new(0.0, 18.0, 0.0, 18.0)),
    )
    .grow(Dimension::Width)
    .id(day::id!(address_bar));
    row((
        when(
            move || !editing(),
            move || {
                button(res::str::home())
                    .icon(Symbol::Home)
                    .icon_only()
                    .plain()
                    .action(move || browser.go_home())
                    .id(day::id!(home))
            },
        ),
        omnibox_pill,
        when(
            move || !editing(),
            move || {
                row((
                    button(res::str::new_tab())
                        .icon(Symbol::Add)
                        .icon_only()
                        .plain()
                        .action(move || browser.new_tab(false))
                        .id(day::id!(new_tab_button))
                        .width(44.0),
                    // Chrome's tab switcher: the count in a rounded square.
                    label(move || {
                        let n = count();
                        if n > 99 {
                            ":D".to_string()
                        } else {
                            n.to_string()
                        }
                    })
                    .font(Font::Footnote)
                    .bold()
                    .color(move || day::system_color(SystemColor::SecondaryText))
                    .align(TextAlign::Center)
                    .frame(22.0, 22.0)
                    // In the bar's icon color, beside the home, new-tab and menu glyphs.
                    .overlay(
                        rounded_rectangle(5.0)
                            .stroke(day::system_color(SystemColor::SecondaryText), 2.0)
                            .grow(Dimension::Both),
                    )
                    .padding(Insets::new(11.0, 13.0, 11.0, 13.0))
                    .on_tap(move || browser.show_overview())
                    .context_menu(move || tabs_menu(browser))
                    .a11y(move |a| {
                        a.role(Role::Button)
                            .label(res::str::tab_count(count() as i64))
                    })
                    .id(day::id!(tabs_button)),
                    menu_button(res::str::more(), move || chrome_menu(browser))
                        .icon(Symbol::More)
                        .icon_only()
                        .plain()
                        .id(day::id!(more))
                        .width(44.0),
                ))
                .align(VAlign::Center)
                // The omnibox gives way to these, never the other way round.
                .min_width(136.0)
            },
        ),
        when(editing, move || {
            button(res::str::cancel())
                .plain()
                .action(move || omnibox.end_editing(browser))
                .id(day::id!(address_cancel))
        }),
    ))
    .spacing(4.0)
    .align(VAlign::Center)
    .height(56.0)
    .padding(Insets::new(0.0, 4.0, 0.0, 4.0))
    // The window's own surface, which the status bar above shows too, so the two read as one
    // bar, as in Chrome.
    .background(move || day::system_color(SystemColor::Background))
}

/// Chrome's ⋮ menu: the page's commands first, then the browser's.
fn chrome_menu(browser: Browser) -> Vec<MenuEntry> {
    let has_page = move || browser.current().is_some_and(|t| t.has_page());
    let marked = move || {
        browser
            .current()
            .is_some_and(|t| library::is_bookmarked(library::bookmarks(), &t.url()))
    };
    vec![
        menu_item(res::str::forward())
            .icon(Symbol::Forward)
            .enabled(move || browser.current().is_some_and(|t| t.can_go_forward()))
            .id(day::id!(more_forward))
            .action(move || {
                if let Some(t) = browser.current_untracked() {
                    t.go_forward();
                }
            }),
        menu_item(move || {
            if marked() {
                res::str::remove_bookmark().format()
            } else {
                res::str::add_bookmark().format()
            }
        })
        .icon(Symbol::Star)
        .enabled(has_page)
        .action(move || bookmark_current(browser)),
        menu_item(res::str::reload())
            .icon(Symbol::Refresh)
            .enabled(has_page)
            .action(move || {
                if let Some(t) = browser.current_untracked() {
                    t.reload_or_stop();
                }
            }),
        menu_separator(),
        menu_item(res::str::new_tab())
            .icon(Symbol::Add)
            .action(move || browser.new_tab(false)),
        menu_item(res::str::new_private_tab()).action(move || browser.new_tab(true)),
        menu_separator(),
        menu_item(res::str::history())
            .icon(Symbol::History)
            .id(day::id!(more_history))
            .action(move || browser.library.set(Some(Pane::History))),
        menu_item(res::str::bookmarks())
            .icon(Symbol::Book)
            .action(move || browser.library.set(Some(Pane::Bookmarks))),
        menu_separator(),
        menu_item(res::str::share())
            .icon(Symbol::Share)
            .enabled(has_page)
            .action(move || share_current(browser)),
        menu_item(res::str::find_on_page())
            .icon(Symbol::Search)
            .enabled(has_page)
            .action(move || open_find(browser)),
        sub_menu(res::str::text_size(), text_size_menu(browser)),
        sub_menu(res::str::blocking_this_site(), blocking_menu(browser)),
        menu_item(res::str::desktop_site())
            .checked(move || browser.current().is_some_and(|t| t.desktop.get()))
            .enabled(has_page)
            .action(move || toggle_desktop(browser)),
        menu_separator(),
        menu_item(res::str::settings())
            .icon(Symbol::Settings)
            .id(day::id!(more_settings))
            .action(|| {
                day_piece_settings::open();
            }),
    ]
}

/// Content blocking for the page's site, as Vivaldi's shield offers it: the default level or one
/// of its own, with what the page had blocked where the engine reports it.
fn blocking_menu(browser: Browser) -> Vec<MenuEntry> {
    let url = move || browser.current().map(|t| t.url()).unwrap_or_default();
    let label = crate::adblock::level_label;
    let own = move || {
        let site = address::site(&url())?;
        let sites: netskip_adblock::SiteLevels =
            serde_json::from_str(&crate::adblock::SITE_LEVELS.get()).ok()?;
        sites.get(&site).map(|l| l.id())
    };
    let mut entries = vec![
        menu_item(move || {
            let n = browser.current().map(|t| t.blocked.get()).unwrap_or(0);
            res::str::blocked_count(n as i64).format()
        })
        .enabled(|| false),
        menu_separator(),
        menu_item(move || {
            res::str::blocking_site_default(label(&crate::adblock::LEVEL.get())).format()
        })
        .checked(move || own().is_none())
        .action(move || crate::adblock::set_site_level(&url(), None)),
    ];
    for id in crate::adblock::LEVELS {
        entries.push(
            menu_item(label(id))
                .checked(move || own() == Some(id))
                .action(move || {
                    crate::adblock::set_site_level(&url(), Some(crate::adblock::level_from_id(id)))
                }),
        );
    }
    entries
}

/// Ask the current tab's site for its desktop page, or its mobile page again.
pub fn toggle_desktop(browser: Browser) {
    if let Some(t) = browser.current_untracked() {
        t.desktop.update(|d| *d = !*d);
    }
}

/// Text size: larger, smaller, and back to the default, with the current size as the title.
fn text_size_menu(browser: Browser) -> Vec<MenuEntry> {
    vec![
        menu_item(res::str::zoom_in())
            .icon(Symbol::ZoomIn)
            .action(move || zoom(browser, 1)),
        menu_item(res::str::zoom_out())
            .icon(Symbol::ZoomOut)
            .action(move || zoom(browser, -1)),
        menu_item(move || {
            let z = browser.current().map(|t| t.zoom.get()).unwrap_or(100);
            res::str::actual_size_percent(z).format()
        })
        .icon(Symbol::ZoomReset)
        .action(move || zoom(browser, 0)),
    ]
}

// ---------------------------------------------------------------------------------------------
// The tab grid
// ---------------------------------------------------------------------------------------------

/// Every tab as a card, two across on a phone and more on a wider screen, with New Tab and Done
/// along the bottom, like Safari's tab overview.
fn overview_page(browser: Browser) -> impl Piece {
    let count = move || browser.tabs.keys().len();
    column((
        scroll(
            // Two cards across on a phone, more on a wider screen, as Safari lays them out.
            switch(
                move || {
                    // A cover may not carry the window's size class: a touch screen without one
                    // is taken for a phone.
                    let phone = compact() || (TOUCH && day::size_class().is_none());
                    let across = if phone { 2 } else { 4 };
                    (browser.tabs.keys(), across)
                },
                move |(keys, across)| {
                    let rows: Vec<Vec<String>> = keys.chunks(*across).map(|c| c.to_vec()).collect();
                    let across = *across;
                    column((each(
                        items(move || rows.clone(), |r: &Vec<String>| r.join(",")),
                        move |slot: ItemSlot<Vec<String>, String>| {
                            let keys = slot.get();
                            let mut cards: Vec<AnyPiece> = keys
                                .into_iter()
                                .map(|key| {
                                    keyed(
                                        move || browser.tab(&key).map(|t| t.id),
                                        move |id: &u64| tab_card(browser, id.to_string()),
                                    )
                                    .grow(Dimension::Width)
                                    .any()
                                })
                                .collect();
                            while cards.len() < across {
                                cards.push(spacer().grow(Dimension::Width).any());
                            }
                            row(cards).spacing(16.0).align(VAlign::Top)
                        },
                    ),))
                    .spacing(16.0)
                    .padding(Space::L)
                    .any()
                },
            ),
        )
        .grow(Dimension::Both)
        .id(day::id!(tab_grid)),
        row((
            button(res::str::new_tab())
                .icon(Symbol::Add)
                .icon_only()
                .plain()
                .action(move || browser.new_tab(false))
                .context_menu(move || {
                    vec![
                        menu_item(res::str::new_private_tab())
                            .action(move || browser.new_tab(true)),
                        menu_item(res::str::reopen_closed_tab())
                            .enabled(move || browser.has_closed())
                            .action(move || browser.reopen_closed()),
                    ]
                })
                .id(day::id!(overview_new)),
            label(move || res::str::tab_count(count() as i64).format())
                .font(Font::Headline)
                .align(TextAlign::Center)
                .grow(Dimension::Width)
                .id(day::id!(tab_count)),
            button(res::str::done())
                .plain()
                .action(move || browser.overview.set(false))
                .id(day::id!(overview_done)),
        ))
        .align(VAlign::Center)
        .padding(Insets::new(8.0, 16.0, 8.0, 16.0))
        .background(move || day::system_color(SystemColor::Surface)),
    ))
    .grow(Dimension::Both)
    .id(day::id!(overview))
}

fn tab_card(browser: Browser, key: String) -> impl Piece {
    let Some(tab) = untrack(|| browser.tab(&key)) else {
        return spacer().width(0.0).any();
    };
    // By the tab's id, so the predicate is `Copy` and every part of the card can read it.
    let id = tab.id;
    let selected = move || browser.tabs.selected().and_then(|k| k.parse::<u64>().ok()) == Some(id);
    let (k1, k2) = (key.clone(), key.clone());
    let close = button(res::str::close_tab())
        .icon(Symbol::Close)
        .icon_only()
        .plain()
        .compact()
        .id(day::id!(card_close, k1.clone()))
        .action(move || browser.close(&k1));
    let preview = switch(
        move || (tab.snapshot.get(), tab.has_page()),
        move |(snapshot, page)| match (snapshot, page) {
            (Some(shot), true) => image(shot.0.to_vec()).grow(Dimension::Both).any(),
            (_, true) => site_icon(move || tab.url(), 56.0)
                .aligned(Alignment::Center)
                .grow(Dimension::Both)
                .any(),
            (_, false) => vector(if tab.private {
                res::vectors::incognito
            } else {
                res::vectors::globe
            })
            .tint(Color::hex(0x8e8e93))
            .frame(40.0, 40.0)
            .aligned(Alignment::Center)
            .grow(Dimension::Both)
            .any(),
        },
    )
    .background(move || day::system_color(SystemColor::Background))
    .aspect_ratio(0.78)
    .grow(Dimension::Width);
    let title = move || {
        label(move || tab.label())
            .font(Font::Footnote)
            .single_line()
    };
    let card = if CHROME_LAYOUT {
        // Chrome's card: a header with the site's icon, title and close, over the page; the
        // selected tab's header takes a tint of the accent and its frame the accent itself. The
        // tint stays light so the title and the close glyph read on it in their usual colors.
        let accent = move || day::system_color(SystemColor::Accent);
        column((
            row((
                tab_icon(tab, 16.0),
                title()
                    .color(move || day::system_color(SystemColor::Text))
                    .grow(Dimension::Width),
                close,
            ))
            .spacing(6.0)
            .align(VAlign::Center)
            .padding(Insets::new(6.0, 10.0, 6.0, 10.0))
            .background(move || {
                if selected() {
                    let a = accent();
                    Color::rgba(a.r, a.g, a.b, 0.22)
                } else {
                    day::system_color(SystemColor::Surface)
                }
            }),
            preview,
        ))
        .grow(Dimension::Width)
        .corner_radius(16.0)
        .overlay(when(selected, move || {
            rounded_rectangle(16.0)
                .stroke(accent(), 3.0)
                .grow(Dimension::Both)
        }))
        .any()
    } else {
        // Safari's card: the page itself, rounded, with a close button in its corner, and the
        // site's icon and title under it; the selected tab is ringed in the accent color.
        column((
            preview
                .corner_radius(12.0)
                .overlay_aligned(
                    Alignment::TopTrailing,
                    close
                        .frame(26.0, 26.0)
                        .background(move || Color::rgba(0.0, 0.0, 0.0, 0.45))
                        .corner_radius(13.0)
                        .padding(6.0),
                )
                .overlay(when(selected, || {
                    rounded_rectangle(12.0)
                        .stroke(day::system_color(SystemColor::Accent), 3.0)
                        .grow(Dimension::Both)
                }))
                .overlay(
                    rounded_rectangle(12.0)
                        .stroke(tone(0.1, 0.15), 0.5)
                        .grow(Dimension::Both),
                )
                .grow(Dimension::Width),
            row((spacer(), tab_icon(tab, 14.0), title(), spacer()))
                .spacing(5.0)
                .align(VAlign::Center),
        ))
        .spacing(6.0)
        .grow(Dimension::Width)
        .any()
    };
    card.on_tap(move || {
        browser.tabs.select(&k2);
        browser.overview.set(false);
    })
    .context_menu({
        let key = key.clone();
        move || tab_menu(browser, key.clone())
    })
    .id(day::id!(tab_card, key))
    .any()
}

/// Whether a tab shows its start page; used by the grid's empty cards.
pub fn is_start(tab: Tab) -> bool {
    tab.start.get() || is_blank(&tab.location.get())
}
