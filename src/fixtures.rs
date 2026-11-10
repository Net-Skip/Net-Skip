// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Fixture mode: browsing without a network, for dayscripts and demos.
//!
//! `NETSKIP_FIXTURES=1` at launch serves every page from the files under
//! `tests/fixtures/web/<host>/…` (compiled in by build.rs) and answers the app's own requests
//! (search suggestions, favicons) from the same files through day-part-http's simulation.
//! Pages load through a day-piece-webview resource namespace, so the address the web view
//! loads differs from the one the user sees; [`to_physical`] and [`to_logical`] map between
//! them ([`netskip_model::fixtures`] has the rules and their tests). Any address without a
//! fixture file gets a generated page titled with its host, and an engine's results page gets
//! a generated list of results, so every typed address and search loads.

use std::sync::{Arc, OnceLock};

use day_part_http::simulation::{Reply, Simulation};
use day_piece_webview::{ResourceProvider, ResourceRequest, ResourceResponse};
use netskip_model::fixtures::{
    FixtureRequest, FixtureUrls, candidate_files, escape_html, mime_for, parse_request,
};
use netskip_model::search::{ENGINES, Engine};

mod table {
    include!(concat!(env!("OUT_DIR"), "/web_fixtures.rs"));
}

struct Active {
    urls: FixtureUrls,
    provider: ResourceProvider,
}

static ACTIVE: OnceLock<Option<Active>> = OnceLock::new();

/// Turn fixture mode on when the launch asks for it. Call once, before the first window.
pub fn install() {
    ACTIVE.get_or_init(|| {
        if day::env("NETSKIP_FIXTURES").as_deref() != Some("1") {
            return None;
        }
        let urls_cell: Arc<OnceLock<FixtureUrls>> = Arc::new(OnceLock::new());
        let handler_urls = urls_cell.clone();
        let provider = ResourceProvider::new(move |req| match handler_urls.get() {
            Some(urls) => serve(urls, &req),
            None => ResourceResponse::error(503),
        });
        let urls = FixtureUrls::new(provider.base_url());
        let _ = urls_cell.set(urls.clone());
        install_http();
        day::info!("fixture mode: pages come from tests/fixtures/web");
        Some(Active { urls, provider })
    });
}

/// Whether fixture mode is on.
pub fn active() -> bool {
    matches!(ACTIVE.get(), Some(Some(_)))
}

/// The resource namespace every web view needs in fixture mode.
pub fn provider() -> Option<ResourceProvider> {
    ACTIVE.get()?.as_ref().map(|a| a.provider.clone())
}

/// The address the web view loads for a page's real address.
pub fn to_physical(url: &str) -> String {
    match ACTIVE.get() {
        Some(Some(a)) => a.urls.to_physical(url),
        _ => url.to_string(),
    }
}

/// A page's real address, from what the web view reports.
pub fn to_logical(url: &str) -> String {
    match ACTIVE.get() {
        Some(Some(a)) => a.urls.to_logical(url),
        _ => url.to_string(),
    }
}

fn file(path: &str) -> Option<&'static [u8]> {
    table::FILES
        .iter()
        .find(|(name, _)| *name == path)
        .map(|(_, bytes)| *bytes)
}

fn html(body: String) -> ResourceResponse {
    ResourceResponse::new("text/html; charset=utf-8", body.into_bytes())
}

fn serve(urls: &FixtureUrls, req: &ResourceRequest) -> ResourceResponse {
    let Some(fr) = parse_request(&req.path) else {
        return ResourceResponse::not_found();
    };
    for name in candidate_files(&fr) {
        if let Some(bytes) = file(&name) {
            let mime = mime_for(&name);
            if mime.starts_with("text/html") {
                let text = String::from_utf8_lossy(bytes);
                return html(urls.rewrite_html(&text));
            }
            return ResourceResponse::new(mime, bytes.to_vec());
        }
    }
    let logical = format!(
        "{}://{}{}{}",
        fr.scheme,
        fr.host,
        fr.path,
        req.query
            .as_deref()
            .filter(|q| !q.is_empty())
            .map(|q| format!("?{q}"))
            .unwrap_or_default()
    );
    if let Some(page) = results_page(&logical) {
        return html(urls.rewrite_html(&page));
    }
    if fr
        .path
        .rsplit('/')
        .next()
        .is_some_and(|last| last.contains('.'))
        && !fr.path.ends_with(".html")
    {
        // A stylesheet, script or image with no fixture: missing, as on a real server.
        return ResourceResponse::not_found();
    }
    html(urls.rewrite_html(&generic_page(&fr)))
}

/// A results page for a search on any built-in engine.
fn results_page(logical: &str) -> Option<String> {
    let (engine, terms) = ENGINES.iter().find_map(|e| {
        let engine = Engine::BuiltIn(e);
        engine.terms_in(logical).map(|t| (e.name, t))
    })?;
    let terms_html = escape_html(&terms);
    let lower = terms.to_lowercase();
    let mut results = String::new();
    for (url, title, blurb) in SITES {
        let hit = lower
            .split_whitespace()
            .any(|w| title.to_lowercase().contains(w) || blurb.to_lowercase().contains(w));
        if hit || results.is_empty() {
            results.push_str(&format!(
                "<li><a href=\"{url}\">{title}</a><p>{blurb}</p></li>"
            ));
        }
    }
    Some(format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>{terms_html} – {engine}</title><style>body{{font:16px -apple-system,system-ui,sans-serif;margin:24px;max-width:720px}}\
         li{{margin:14px 0}}p{{margin:4px 0;color:#555}}</style></head><body>\
         <h1 id=\"results-title\">Results for “{terms_html}”</h1><ol id=\"results\">{results}</ol></body></html>"
    ))
}

/// The fixture sites a results page can list.
const SITES: &[(&str, &str, &str)] = &[
    (
        "https://example.com/",
        "Example Domain",
        "A page for use in illustrative examples.",
    ),
    (
        "https://example.com/news/",
        "Example News",
        "Harbor news, weather and the week's events.",
    ),
    (
        "https://example.org/",
        "Example Organization",
        "An organization page with a member directory.",
    ),
    (
        "https://example.net/",
        "Example Network",
        "Notes about the network and its services.",
    ),
];

/// The page for an address that has no fixture: its host as the title, and a way back.
fn generic_page(fr: &FixtureRequest) -> String {
    let host = escape_html(&fr.host);
    let path = escape_html(&fr.path);
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>{host}</title><style>body{{font:16px -apple-system,system-ui,sans-serif;margin:24px}}</style></head>\
         <body><h1 id=\"generic-title\">{host}</h1><p>A generated fixture page for <code>{path}</code>.</p>\
         <p><a id=\"generic-home\" href=\"https://example.com/\">Example Domain</a></p></body></html>"
    )
}

/// Answer the app's own HTTP requests from the fixture tree: favicons, and search
/// suggestions made from the typed text. Nothing reaches the network.
fn install_http() {
    let simulation = Simulation::new();
    simulation.set_strict(true);
    simulation.handle(
        |_, _| true,
        |request| {
            let Ok(url) = url::Url::parse(&request.url) else {
                return Ok(Reply::new(400, Vec::new()));
            };
            let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
            let fr = FixtureRequest {
                scheme: url.scheme().to_string(),
                host: host.clone(),
                path: url.path().to_string(),
            };
            for name in candidate_files(&fr) {
                if let Some(bytes) = file(&name) {
                    return Ok(
                        Reply::new(200, bytes.to_vec()).header("Content-Type", mime_for(&name))
                    );
                }
            }
            if let Some(q) = suggestion_terms(&url) {
                let list: Vec<String> = ["news", "weather", "history", "recipes"]
                    .iter()
                    .map(|w| format!("{q} {w}"))
                    .collect();
                let body = serde_json::json!([q, list]).to_string();
                return Ok(Reply::new(200, body.into_bytes())
                    .header("Content-Type", "application/x-suggestions+json"));
            }
            Ok(Reply::new(404, Vec::new()))
        },
    );
    day_part_http::Session::global().set_simulation(Some(simulation));
}

/// The typed text in a suggestion request to any built-in engine.
fn suggestion_terms(url: &url::Url) -> Option<String> {
    let host = url.host_str()?.trim_start_matches("www.");
    let is_suggest = ENGINES.iter().any(|e| {
        e.suggest.is_some_and(|(t, _)| {
            url::Url::parse(&t.replace("{searchTerms}", "x")).is_ok_and(|s| {
                s.host_str().map(|h| h.trim_start_matches("www.")) == Some(host)
                    && s.path() == url.path()
            })
        })
    });
    if !is_suggest {
        return None;
    }
    url.query_pairs()
        .find(|(k, _)| matches!(k.as_ref(), "q" | "query" | "command" | "part"))
        .map(|(_, v)| v.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_holds_the_fixture_sites() {
        assert!(file("example.com/index.html").is_some());
        assert!(file("example.com/news/index.html").is_some());
        assert!(file("example.org/index.html").is_some());
        assert!(file("example.com/favicon.png").is_some());
    }

    #[test]
    fn results_pages_name_the_search() {
        let page = results_page("https://kagi.com/search?q=harbor+news").unwrap();
        assert!(page.contains("Results for “harbor news”"));
        assert!(page.contains("https://example.com/news/"));
        assert!(results_page("https://example.com/search?q=x").is_none());
    }

    #[test]
    fn suggestion_requests_are_recognized() {
        let u = url::Url::parse("https://kagi.com/api/autosuggest?q=sail").unwrap();
        assert_eq!(suggestion_terms(&u).as_deref(), Some("sail"));
        let u = url::Url::parse("https://duckduckgo.com/ac/?q=sail&type=list").unwrap();
        assert_eq!(suggestion_terms(&u).as_deref(), Some("sail"));
        let u = url::Url::parse("https://kagi.com/search?q=sail").unwrap();
        assert_eq!(suggestion_terms(&u), None);
    }

    #[test]
    fn generated_pages_escape_the_address() {
        let fr = parse_request("https/x.example/<script>").unwrap();
        assert!(!generic_page(&fr).contains("<script>"));
    }
}
