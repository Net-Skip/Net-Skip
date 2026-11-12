// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Settings: the search engine, how the address bar and new tabs behave, and clearing what the
//! browser remembers (https://daybrite.dev/docs/reference/settings).

use crate::{browser::Browser, res, store};
use day::prelude::*;
use day_piece_settings::prelude::*;
use netskip_model::address;
use netskip_model::history::ClearRange;
use netskip_model::search::{self, CUSTOM_ENGINE, ENGINES, Engine};

/// The search engine's id ([`netskip_model::search::ENGINES`], or `custom`).
pub static SEARCH_ENGINE: Pref<String> = Pref::new("netskip.search.engine", || {
    search::DEFAULT_ENGINE.to_string()
});
/// A custom engine's results-page template, with `%s` where the search terms go.
pub static CUSTOM_SEARCH: Pref<String> = Pref::new("netskip.search.custom", String::new);
/// Ask the search engine for suggestions while typing in the address bar.
pub static SUGGESTIONS: Pref<bool> = Pref::new("netskip.search.suggestions", || true);
/// Offer pages from history and bookmarks in the address bar (Safari Suggestions).
pub static LIBRARY_SUGGESTIONS: Pref<bool> = Pref::new("netskip.search.library", || true);
/// Links a page opens in a new tab open behind the current one.
pub static OPEN_IN_BACKGROUND: Pref<bool> = Pref::new("netskip.links.background", || false);
/// The phone's address bar sits at the top of the window: Chrome's place, and the default on
/// Android; Safari's bottom bar is the default elsewhere.
pub static ADDRESS_AT_TOP: Pref<bool> =
    Pref::new("netskip.address.top", || crate::chrome::CHROME_LAYOUT);
/// Close tabs not looked at for this many days at launch; `0` closes them only by hand.
pub static CLOSE_TABS_DAYS: Pref<i64> = Pref::new("netskip.tabs.close_after", || 0);
/// The page the home button and new windows open; empty means the start page.
pub static HOME_PAGE: Pref<String> = Pref::new("netskip.home", String::new);
/// Show the page's whole address in the address bar instead of only its site.
pub static FULL_ADDRESS: Pref<bool> = Pref::new("netskip.address.full", || false);
/// Load typed `http://` addresses over HTTPS.
pub static UPGRADE_HTTPS: Pref<bool> = Pref::new("netskip.https.upgrade", || true);
/// Reopen the last session's tabs at launch.
pub static RESTORE_TABS: Pref<bool> = Pref::new("netskip.tabs.restore", || true);
/// Show the start page's Frequently Visited sites.
pub static SHOW_FREQUENT: Pref<bool> = Pref::new("netskip.start.frequent", || true);
/// Ask websites for their desktop pages on phones and tablets (Request Desktop Website).
pub static DESKTOP_SITES: Pref<bool> = Pref::new("netskip.sites.desktop", || false);
/// Let pages run JavaScript.
pub static JAVASCRIPT: Pref<bool> = Pref::new("netskip.sites.javascript", || true);
/// What Net Skip tells sites it is: the platform browser's string, the engine's own, a preset
/// (`netskip_model::address::USER_AGENTS`), or the string in [`USER_AGENT_CUSTOM`].
pub static USER_AGENT: Pref<String> = Pref::new("netskip.sites.user_agent", || {
    address::USER_AGENT_DEFAULT.to_string()
});
pub static USER_AGENT_CUSTOM: Pref<String> =
    Pref::new("netskip.sites.user_agent.custom", String::new);
/// The zoom chosen for each site, `site=percent` lines; a site not listed opens at [`PAGE_ZOOM`].
pub static SITE_ZOOM: Pref<String> = Pref::new("netskip.sites.zoom", String::new);

/// The zoom `site` opens at.
pub fn site_zoom(site: &str) -> i64 {
    SITE_ZOOM
        .get_untracked()
        .lines()
        .find_map(|l| l.strip_prefix(site)?.strip_prefix('=')?.parse().ok())
        .unwrap_or_else(|| PAGE_ZOOM.get_untracked())
}

/// Remember `percent` as `site`'s zoom; the default zoom forgets it.
pub fn remember_site_zoom(site: &str, percent: i64) {
    let mut lines: Vec<String> = SITE_ZOOM
        .get_untracked()
        .lines()
        .filter(|l| l.split_once('=').is_none_or(|(s, _)| s != site))
        .map(str::to_string)
        .collect();
    if percent != PAGE_ZOOM.get_untracked() {
        lines.push(format!("{site}={percent}"));
    }
    SITE_ZOOM.set(lines.join("\n"));
}

/// The zoom new pages open at, as a percentage.
pub static PAGE_ZOOM: Pref<i64> = Pref::new("netskip.zoom", || 100);

/// The zoom steps the View menu and the page menu move through, Safari's.
pub const ZOOM_STEPS: [i64; 11] = [50, 75, 85, 100, 115, 125, 150, 175, 200, 250, 300];

/// The engine searches go to now. A tracked read.
pub fn engine() -> Engine {
    Engine::resolve(&SEARCH_ENGINE.get(), &CUSTOM_SEARCH.get())
}

/// What the web views tell sites they are, from the setting. A tracked read.
pub fn user_agent() -> day_piece_webview::UserAgent {
    use address::UserAgentChoice;
    match address::user_agent_choice(&USER_AGENT.get(), &USER_AGENT_CUSTOM.get()) {
        UserAgentChoice::Browser => day_piece_webview::UserAgent::Browser,
        UserAgentChoice::Engine => day_piece_webview::UserAgent::Engine,
        UserAgentChoice::Text(text) => day_piece_webview::UserAgent::Custom(text),
    }
}

/// The engine's name for "Search DuckDuckGo", or the custom template's host.
pub fn engine_name(engine: &Engine) -> String {
    match engine.name() {
        Some(name) => name.to_string(),
        None => engine.host().unwrap_or_default(),
    }
}

/// The address the home button opens, when one is set.
pub fn home_url() -> Option<String> {
    netskip_model::address::home_page(&HOME_PAGE.get_untracked())
}

/// Settings and About, installed once for the app.
pub fn install() -> Prefs {
    let mut engines: Vec<(String, String)> = ENGINES
        .iter()
        .map(|e| (e.id.to_string(), e.name.to_string()))
        .collect();
    engines.push((
        CUSTOM_ENGINE.to_string(),
        res::str::search_custom().format(),
    ));
    let zooms: Vec<(i64, String)> = ZOOM_STEPS.iter().map(|z| (*z, format!("{z}%"))).collect();
    let mut agents = vec![(
        address::USER_AGENT_DEFAULT.to_string(),
        res::str::user_agent_default().format(),
    )];
    agents.extend(
        address::USER_AGENTS
            .iter()
            .map(|(id, name, _)| (id.to_string(), name.to_string())),
    );
    agents.push((
        address::USER_AGENT_ENGINE.to_string(),
        res::str::user_agent_engine().format(),
    ));
    agents.push((
        address::USER_AGENT_CUSTOM.to_string(),
        res::str::search_custom().format(),
    ));
    Preferences::new()
        .about(
            About::from_manifest()
                .tagline(res::str::about_tagline())
                .copyright("© The App Fair Project")
                .website("https://appfair.org/")
                .source("https://github.com/Net-Skip/Net-Skip")
                .privacy("https://appfair.org/privacy/"),
        )
        .group(general_group(res::locales::ALL))
        .group(
            Group::new("search", res::str::settings_search())
                .icon(res::vectors::search)
                .section(
                    Section::new()
                        .row(rows::choice(
                            &SEARCH_ENGINE,
                            res::str::search_engine(),
                            engines,
                        ))
                        .row(
                            rows::text(&CUSTOM_SEARCH, res::str::search_custom_template())
                                .placeholder(res::str::search_custom_placeholder())
                                .caption(res::str::search_custom_caption())
                                .available(|| SEARCH_ENGINE.get() == CUSTOM_ENGINE),
                        )
                        .row(
                            rows::toggle(&SUGGESTIONS, res::str::search_suggestions())
                                .caption(res::str::search_suggestions_caption()),
                        )
                        .row(
                            rows::toggle(&LIBRARY_SUGGESTIONS, res::str::library_suggestions())
                                .caption(res::str::library_suggestions_caption()),
                        ),
                ),
        )
        .group(
            Group::new("browsing", res::str::settings_browsing())
                .icon(res::vectors::tab_group)
                .section(
                    Section::new()
                        .row(
                            rows::text(&HOME_PAGE, res::str::home_page())
                                .placeholder(res::str::home_page_placeholder()),
                        )
                        .row(rows::toggle(&SHOW_FREQUENT, res::str::show_frequent())),
                )
                .section(
                    Section::new()
                        .title(res::str::settings_tabs())
                        .row(rows::toggle(&RESTORE_TABS, res::str::restore_tabs()))
                        .row(
                            rows::toggle(&ADDRESS_AT_TOP, res::str::address_at_top())
                                .caption(res::str::address_at_top_caption()),
                        )
                        .row(rows::toggle(
                            &OPEN_IN_BACKGROUND,
                            res::str::open_links_background(),
                        ))
                        .row(rows::choice(
                            &CLOSE_TABS_DAYS,
                            res::str::close_tabs(),
                            vec![
                                (0, res::str::close_tabs_manually().format()),
                                (1, res::str::close_tabs_day().format()),
                                (7, res::str::close_tabs_week().format()),
                                (30, res::str::close_tabs_month().format()),
                            ],
                        )),
                )
                .section(
                    Section::new()
                        .title(res::str::settings_address_bar())
                        .row(rows::toggle(&FULL_ADDRESS, res::str::full_address()))
                        .row(
                            rows::toggle(&UPGRADE_HTTPS, res::str::upgrade_https())
                                .caption(res::str::upgrade_https_caption()),
                        )
                        .row(rows::choice(&PAGE_ZOOM, res::str::page_zoom(), zooms)),
                )
                .section(
                    Section::new()
                        .title(res::str::settings_websites())
                        .row(
                            rows::choice(&USER_AGENT, res::str::user_agent(), agents)
                                .caption(res::str::user_agent_caption()),
                        )
                        .row(
                            rows::text(&USER_AGENT_CUSTOM, res::str::user_agent_custom())
                                .placeholder("Mozilla/5.0 …")
                                .available(|| USER_AGENT.get() == address::USER_AGENT_CUSTOM),
                        ),
                ),
        )
        .group(
            Group::new("privacy", res::str::settings_privacy())
                .icon(res::vectors::lock)
                .section(
                    Section::new()
                        .title(res::str::blocking())
                        .row(
                            rows::choice(
                                &crate::adblock::LEVEL,
                                res::str::blocking_level(),
                                vec![
                                    ("off".to_string(), res::str::blocking_off().format()),
                                    ("light".to_string(), res::str::blocking_light().format()),
                                    ("strong".to_string(), res::str::blocking_strong().format()),
                                    (
                                        "aggressive".to_string(),
                                        res::str::blocking_aggressive().format(),
                                    ),
                                    ("custom".to_string(), res::str::blocking_custom().format()),
                                ],
                            )
                            .caption(res::str::blocking_caption()),
                        )
                        .row(
                            rows::navigate(
                                res::str::blocking_filter_lists(),
                                crate::adblock::filter_lists_page,
                            )
                            .id(day::id!(filter_lists)),
                        )
                        .row(rows::note(crate::adblock::status_text).id(day::id!(blocking_status)))
                        .row(
                            rows::action(
                                res::str::blocking_reload_page(),
                                crate::adblock::reload_current_page,
                            )
                            .caption(res::str::blocking_reload_offer())
                            .available(crate::adblock::reload_offered)
                            .id(day::id!(blocking_reload)),
                        )
                        .row(
                            rows::action(res::str::blocking_update(), crate::adblock::update_now)
                                .id(day::id!(blocking_update)),
                        ),
                )
                .section(
                    Section::new()
                        .row(
                            rows::action(res::str::clear_history(), || {
                                clear_history(ClearRange::All)
                            })
                            .destructive()
                            .confirm(res::str::clear_history_confirm())
                            .confirm_label(res::str::clear_history())
                            .id(day::id!(clear_history_all)),
                        )
                        .row(
                            rows::action(res::str::clear_website_data(), clear_website_data)
                                .destructive()
                                .caption(res::str::clear_website_data_caption())
                                .confirm(res::str::clear_website_data_confirm())
                                .confirm_label(res::str::clear_website_data())
                                .id(day::id!(clear_website_data)),
                        )
                        .row(
                            rows::action(res::str::clear_bookmarks(), clear_bookmarks)
                                .destructive()
                                .confirm(res::str::clear_bookmarks_confirm())
                                .confirm_label(res::str::clear_bookmarks())
                                .id(day::id!(clear_bookmarks)),
                        ),
                ),
        )
        .install()
}

/// Remove history in `range`, and the recently closed tabs with it when all of it goes.
pub fn clear_history(range: ClearRange) {
    let since = range.since(crate::now_ms(), crate::utc_offset_secs());
    let done = store::write(move |db| store::clear_history(db, since));
    day::task(async move {
        match done.await {
            Ok(n) => info!("cleared {n} history pages"),
            Err(e) => error!("clearing history failed: {e}"),
        }
        crate::library::changed();
    });
    if range == ClearRange::All
        && let Some(browser) = Browser::try_ambient().or_else(Browser::focused)
    {
        browser.forget_closed_tabs();
    }
}

/// Remove every bookmark.
fn clear_bookmarks() {
    let done = store::write(|db| {
        for b in store::bookmarks(db)? {
            store::remove_bookmark(db, b.id)?;
        }
        Ok(())
    });
    day::task(async move {
        if let Err(e) = done.await {
            error!("clearing bookmarks failed: {e}");
        }
        crate::library::changed();
    });
}

/// Remove cookies, caches and site storage from the browser's profile.
fn clear_website_data() {
    if let Some(browser) = Browser::try_ambient().or_else(Browser::focused) {
        browser.clear_website_data();
    }
}
