// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Fixture browsing: the address mapping that lets tests load web pages from files bundled with
//! the app instead of the network.
//!
//! A web view cannot intercept `https://` itself on every engine (WebKit refuses a custom
//! handler for the web's own schemes), so fixture mode loads every page through an app-owned
//! resource namespace instead. The page the user sees, `https://example.com/news/`, is the
//! *logical* address; the web view loads the *physical* one, `<base>https/example.com/news/`,
//! where `<base>` is the namespace's root. History, the address bar and bookmarks only ever see
//! logical addresses. Absolute links inside a fixture page are rewritten to physical ones as
//! the page is served, so following a link stays inside the fixtures.

/// Maps between logical and physical addresses for one resource namespace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureUrls {
    base: String,
}

impl FixtureUrls {
    /// `base` is the namespace's root, with a trailing slash.
    pub fn new(base: impl Into<String>) -> Self {
        let mut base = base.into();
        if !base.ends_with('/') {
            base.push('/');
        }
        FixtureUrls { base }
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    /// The physical address for a logical `http(s)` one; other addresses pass through.
    pub fn to_physical(&self, logical: &str) -> String {
        for scheme in ["https", "http"] {
            if let Some(rest) = logical.strip_prefix(&format!("{scheme}://")) {
                // The device's own loopback is never a fixture: a dayscript reaches a port
                // nothing listens on to see the error page, and a developer a local server.
                let host = rest.split(['/', ':', '?', '#']).next().unwrap_or_default();
                if matches!(host, "127.0.0.1" | "localhost") {
                    return logical.to_string();
                }
                let rest = if rest.contains('/') {
                    rest.to_string()
                } else {
                    // A bare host: give it the path a browser would.
                    match rest.find(['?', '#']) {
                        Some(i) => format!("{}/{}", &rest[..i], &rest[i..]),
                        None => format!("{rest}/"),
                    }
                };
                return format!("{}{scheme}/{rest}", self.base);
            }
        }
        logical.to_string()
    }

    /// The logical address for a physical one; anything outside the namespace passes through.
    pub fn to_logical(&self, physical: &str) -> String {
        let Some(rest) = physical.strip_prefix(&self.base) else {
            return physical.to_string();
        };
        match rest.split_once('/') {
            Some((scheme @ ("https" | "http"), rest)) => format!("{scheme}://{rest}"),
            _ => physical.to_string(),
        }
    }

    /// Rewrite the absolute `http(s)` addresses in a fixture page's attributes and CSS to
    /// physical ones, so links, images and form actions stay inside the fixtures.
    pub fn rewrite_html(&self, html: &str) -> String {
        // One pass over the text: a second pass would rewrite the base itself where the base is an
        // `http://` address (WebView2's virtual host).
        let mut out = String::with_capacity(html.len());
        let mut rest = html;
        while let Some(at) = rest.find(['"', '\'', '(']) {
            let (before, from) = rest.split_at(at);
            out.push_str(before);
            let quote = &from[..1];
            let tail = &from[1..];
            let scheme = ["https", "http"]
                .into_iter()
                .find(|s| tail.strip_prefix(s).is_some_and(|t| t.starts_with("://")));
            match scheme {
                Some(scheme) => {
                    out.push_str(quote);
                    out.push_str(&self.base);
                    out.push_str(scheme);
                    out.push('/');
                    rest = &tail[scheme.len() + 3..];
                }
                None => {
                    out.push_str(quote);
                    rest = tail;
                }
            }
        }
        out.push_str(rest);
        out
    }
}

/// A fixture request, from the path the resource namespace received: the scheme, the host and
/// the path inside the site (always starting with `/`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureRequest {
    pub scheme: String,
    pub host: String,
    pub path: String,
}

/// Split a namespace path (`https/example.com/news/`) into its parts.
pub fn parse_request(path: &str) -> Option<FixtureRequest> {
    let path = path.trim_start_matches('/');
    let (scheme, rest) = path.split_once('/')?;
    if !matches!(scheme, "https" | "http") {
        return None;
    }
    let (host, path) = match rest.split_once('/') {
        Some((h, p)) => (h, format!("/{p}")),
        None => (rest, "/".to_string()),
    };
    if host.is_empty() {
        return None;
    }
    Some(FixtureRequest {
        scheme: scheme.to_string(),
        host: host.to_ascii_lowercase(),
        path,
    })
}

/// The file a request reads inside the fixture tree: `<host>/<path>`, with `index.html` for a
/// directory and `.html` tried for an extensionless page (the caller tries them in order).
pub fn candidate_files(req: &FixtureRequest) -> Vec<String> {
    let host = req.host.strip_prefix("www.").unwrap_or(&req.host);
    let path = req.path.trim_start_matches('/');
    if path.is_empty() || path.ends_with('/') {
        return vec![format!("{host}/{path}index.html")];
    }
    let mut out = vec![format!("{host}/{path}")];
    if !path.rsplit('/').next().unwrap_or("").contains('.') {
        out.push(format!("{host}/{path}.html"));
        out.push(format!("{host}/{path}/index.html"));
    }
    out
}

/// The MIME type a fixture file is served with, from its extension.
pub fn mime_for(path: &str) -> &'static str {
    match path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Escape text for an HTML page the fixture server generates.
pub fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
