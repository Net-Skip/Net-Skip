// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! The address bar's text: deciding whether typed input is an address or a search, upgrading
//! plain HTTP, and the short form the bar shows while it is not being edited.

use url::{Host, Url};

/// What submitting the address bar does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    /// Load this URL.
    Navigate(String),
    /// Search for these terms.
    Search(String),
}

/// Schemes the bar loads in the page. Others (`mailto:`, `tel:`) go to the system; anything
/// else is searched for, so `javascript:` typed or pasted never runs.
const PAGE_SCHEMES: &[&str] = &["http", "https", "file", "about", "data", "view-source"];
const SYSTEM_SCHEMES: &[&str] = &["mailto", "tel", "sms", "facetime", "maps", "itms-apps"];

/// Generic top-level domains common enough that a bare `name.tld` is meant as an address.
/// Every country-code domain is accepted as well, and any internationalized one.
/// A word ending in anything else (`node.js`, `notes.txt`) is searched for, as Chrome does.
const GENERIC_TLDS: &[&str] = &[
    "aero",
    "agency",
    "ai",
    "app",
    "art",
    "arpa",
    "asia",
    "bank",
    "bar",
    "beer",
    "berlin",
    "bike",
    "biz",
    "blog",
    "book",
    "build",
    "business",
    "cafe",
    "camp",
    "capital",
    "care",
    "cat",
    "center",
    "chat",
    "church",
    "city",
    "click",
    "cloud",
    "club",
    "codes",
    "coffee",
    "college",
    "com",
    "community",
    "company",
    "computer",
    "consulting",
    "coop",
    "design",
    "dev",
    "digital",
    "directory",
    "earth",
    "eco",
    "edu",
    "education",
    "email",
    "energy",
    "engineering",
    "eus",
    "events",
    "exchange",
    "expert",
    "farm",
    "film",
    "finance",
    "fish",
    "fit",
    "foo",
    "food",
    "foundation",
    "free",
    "fun",
    "fund",
    "game",
    "games",
    "garden",
    "gay",
    "gle",
    "global",
    "gmbh",
    "goog",
    "google",
    "gov",
    "green",
    "group",
    "guide",
    "guru",
    "health",
    "help",
    "host",
    "house",
    "info",
    "ink",
    "institute",
    "int",
    "jobs",
    "kitchen",
    "land",
    "lat",
    "law",
    "life",
    "link",
    "live",
    "llc",
    "lol",
    "london",
    "love",
    "ltd",
    "media",
    "menu",
    "mil",
    "mobi",
    "moe",
    "money",
    "museum",
    "music",
    "name",
    "net",
    "network",
    "new",
    "news",
    "nexus",
    "ngo",
    "ninja",
    "nyc",
    "one",
    "onion",
    "online",
    "ooo",
    "org",
    "page",
    "paris",
    "partners",
    "photo",
    "photography",
    "photos",
    "pics",
    "pink",
    "place",
    "plus",
    "post",
    "press",
    "pro",
    "productions",
    "quebec",
    "radio",
    "recipes",
    "red",
    "rest",
    "rocks",
    "run",
    "sale",
    "school",
    "science",
    "scot",
    "services",
    "sh",
    "shop",
    "show",
    "site",
    "social",
    "software",
    "solutions",
    "space",
    "store",
    "stream",
    "studio",
    "style",
    "support",
    "systems",
    "team",
    "tech",
    "technology",
    "tel",
    "test",
    "tips",
    "today",
    "tokyo",
    "tools",
    "top",
    "town",
    "toys",
    "trade",
    "travel",
    "tube",
    "tv",
    "uno",
    "vip",
    "vision",
    "wales",
    "wang",
    "watch",
    "web",
    "website",
    "wiki",
    "win",
    "wine",
    "work",
    "works",
    "world",
    "wtf",
    "xxx",
    "xyz",
    "yoga",
    "zone",
];

/// The country-code top-level domains in the root zone.
const COUNTRY_TLDS: &[&str] = &[
    "ac", "ad", "ae", "af", "ag", "ai", "al", "am", "ao", "aq", "ar", "as", "at", "au", "aw", "ax",
    "az", "ba", "bb", "bd", "be", "bf", "bg", "bh", "bi", "bj", "bm", "bn", "bo", "br", "bs", "bt",
    "bw", "by", "bz", "ca", "cc", "cd", "cf", "cg", "ch", "ci", "ck", "cl", "cm", "cn", "co", "cr",
    "cu", "cv", "cw", "cx", "cy", "cz", "de", "dj", "dk", "dm", "do", "dz", "ec", "ee", "eg", "er",
    "es", "et", "eu", "fi", "fj", "fk", "fm", "fo", "fr", "ga", "gb", "gd", "ge", "gf", "gg", "gh",
    "gi", "gl", "gm", "gn", "gp", "gq", "gr", "gs", "gt", "gu", "gw", "gy", "hk", "hm", "hn", "hr",
    "ht", "hu", "id", "ie", "il", "im", "in", "io", "iq", "ir", "is", "it", "je", "jm", "jo", "jp",
    "ke", "kg", "kh", "ki", "km", "kn", "kp", "kr", "kw", "ky", "kz", "la", "lb", "lc", "li", "lk",
    "lr", "ls", "lt", "lu", "lv", "ly", "ma", "mc", "md", "me", "mg", "mh", "mk", "ml", "mm", "mn",
    "mo", "mp", "mq", "mr", "ms", "mt", "mu", "mv", "mw", "mx", "my", "mz", "na", "nc", "ne", "nf",
    "ng", "ni", "nl", "no", "np", "nr", "nu", "nz", "om", "pa", "pe", "pf", "pg", "ph", "pk", "pl",
    "pm", "pn", "pr", "ps", "pt", "pw", "py", "qa", "re", "ro", "rs", "ru", "rw", "sa", "sb", "sc",
    "sd", "se", "sg", "sh", "si", "sk", "sl", "sm", "sn", "so", "sr", "ss", "st", "su", "sv", "sx",
    "sy", "sz", "tc", "td", "tf", "tg", "th", "tj", "tk", "tl", "tm", "tn", "to", "tr", "tt", "tv",
    "tw", "tz", "ua", "ug", "uk", "us", "uy", "uz", "va", "vc", "ve", "vg", "vi", "vn", "vu", "wf",
    "ws", "ye", "yt", "za", "zm", "zw",
];

/// Classify what the user submitted. Returns `None` for empty input.
///
/// The rules, in order:
/// - a leading `?` forces a search for the rest;
/// - an explicit page scheme (`https://…`, `about:blank`, `file:///…`) is an address;
/// - `localhost`, an IP literal, or `host:port` is an address on `http`;
/// - text with a space is a search;
/// - `name.tld` with a known top-level domain, optionally with a port, path,
///   query or fragment, is an address on `https`;
/// - a single word followed by `/` (`intranet/`) is an address on `http`;
/// - anything else is a search.
pub fn classify(text: &str) -> Option<Input> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Some(rest) = text.strip_prefix('?') {
        let rest = rest.trim();
        return (!rest.is_empty()).then(|| Input::Search(rest.to_string()));
    }
    if let Some(url) = explicit_url(text) {
        return Some(Input::Navigate(url));
    }
    if text.chars().any(char::is_whitespace) {
        return Some(Input::Search(text.to_string()));
    }
    if let Some(url) = bare_address(text) {
        return Some(Input::Navigate(url));
    }
    Some(Input::Search(text.to_string()))
}

/// Schemes a page may send to another app without asking the user first: the system's own
/// mail, phone, message, map and store apps. Any other app's scheme is asked about, so a page
/// cannot start an app the user did not expect.
const APP_SCHEMES: &[&str] = &[
    "mailto",
    "tel",
    "sms",
    "facetime",
    "facetime-audio",
    "maps",
    "geo",
    "itms-apps",
    "itms-appss",
    "market",
];

/// Whether a link to `url` leaves for another app without a question ([`APP_SCHEMES`]).
pub fn opens_without_asking(url: &str) -> bool {
    url.split_once(':')
        .is_some_and(|(scheme, _)| APP_SCHEMES.contains(&scheme.to_ascii_lowercase().as_str()))
}

/// A URL the system opens rather than the page (`mailto:`, `tel:`), when `url` is one.
pub fn is_system_url(url: &str) -> bool {
    Url::parse(url).is_ok_and(|u| SYSTEM_SCHEMES.contains(&u.scheme()))
}

fn explicit_url(text: &str) -> Option<String> {
    let (scheme, rest) = text.split_once(':')?;
    let scheme_lower = scheme.to_ascii_lowercase();
    if !scheme
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        || scheme.is_empty()
    {
        return None;
    }
    // `localhost:3000` and `example.com:8080/x` look like a scheme followed by a path.
    if rest.chars().next().is_some_and(|c| c.is_ascii_digit()) && !text.contains("://") {
        return None;
    }
    if !PAGE_SCHEMES.contains(&scheme_lower.as_str())
        && !SYSTEM_SCHEMES.contains(&scheme_lower.as_str())
    {
        return None;
    }
    if text.chars().any(char::is_whitespace) {
        // Spaces in a web address's path or query are encoded (a pasted link often has them),
        // but a space before the path ends begins a sentence: `https://example.com and more`,
        // `note: buy milk`.
        let web = matches!(scheme_lower.as_str(), "http" | "https");
        let authority = rest.trim_start_matches('/');
        let authority = &authority[..authority.find(['/', '?', '#']).unwrap_or(authority.len())];
        if !(web && rest.starts_with('/') && !authority.chars().any(char::is_whitespace))
            && scheme_lower != "data"
        {
            return None;
        }
    }
    let url = Url::parse(text).ok()?;
    if matches!(url.scheme(), "http" | "https") && url.host_str().is_none_or(str::is_empty) {
        return None;
    }
    Some(url.to_string())
}

fn bare_address(text: &str) -> Option<String> {
    // Split the authority (host and port, maybe credentials) from the path, query and fragment.
    let end = text.find(['/', '?', '#']).unwrap_or(text.len());
    let (authority, tail) = text.split_at(end);
    // Credentials need a password to read as an address: `user@example.com` is an email
    // address someone is looking up, `user:pass@example.com` a login.
    let host_port = match authority.rsplit_once('@') {
        Some((user, host)) if user.contains(':') && !user.starts_with(':') => host,
        Some(_) => return None,
        None => authority,
    };
    let (host, port) = split_port(host_port)?;
    if host.is_empty() {
        return None;
    }
    let lower = host.to_ascii_lowercase();
    let local = lower == "localhost" || lower.ends_with(".localhost") || is_ip_literal(&lower);
    let scheme = if local {
        "http"
    } else if port.is_some() && !lower.contains('.') {
        // `devbox:8080`: a machine name with a port, a developer's own server.
        "http"
    } else if lower.contains('.') {
        if !valid_hostname(&lower) || !known_tld(&lower) {
            return None;
        }
        "https"
    } else if tail.starts_with('/') && valid_label(&lower) {
        // `intranet/`: a single-word host, made explicit by the slash.
        "http"
    } else {
        return None;
    };
    let url = Url::parse(&format!("{scheme}://{text}")).ok()?;
    url.host_str()?;
    Some(url.to_string())
}

fn split_port(host_port: &str) -> Option<(&str, Option<u16>)> {
    if let Some(inner) = host_port.strip_prefix('[') {
        // An IPv6 literal: `[::1]` or `[::1]:8080`.
        let (addr, rest) = inner.split_once(']')?;
        let port = match rest.strip_prefix(':') {
            Some(p) => Some(p.parse().ok()?),
            None if rest.is_empty() => None,
            None => return None,
        };
        return Some((&host_port[..addr.len() + 2], port));
    }
    match host_port.rsplit_once(':') {
        Some((host, port)) => Some((host, Some(port.parse().ok()?))),
        None => Some((host_port, None)),
    }
}

fn is_ip_literal(host: &str) -> bool {
    if host.starts_with('[') {
        return Url::parse(&format!("http://{host}/")).is_ok();
    }
    let parts: Vec<&str> = host.split('.').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.parse::<u8>().is_ok())
}

fn valid_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

fn valid_hostname(host: &str) -> bool {
    let host = host.strip_suffix('.').unwrap_or(host);
    host.split('.').all(valid_label)
}

fn known_tld(host: &str) -> bool {
    let host = host.strip_suffix('.').unwrap_or(host);
    let Some(tld) = host.rsplit('.').next() else {
        return false;
    };
    if tld.starts_with("xn--") || !tld.is_ascii() {
        return true;
    }
    if tld.chars().any(|c| !c.is_ascii_alphabetic()) {
        return false;
    }
    // RFC 2606's reserved names are addresses too: documentation, test and fixture sites use
    // them (`news.example`), and nothing searchable lives there.
    COUNTRY_TLDS.contains(&tld)
        || GENERIC_TLDS.contains(&tld)
        || matches!(tld, "example" | "test" | "invalid")
}

/// Whether a host is a developer's local machine or an address literal, which an HTTPS upgrade
/// leaves alone: `localhost`, a single-label name, an IPv4 or IPv6 literal.
pub fn is_local_host(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || !host.contains('.')
        || is_ip_literal(&host)
        || host.starts_with('[')
}

/// Rewrite a typed `http://` address to `https://` (the "Upgrade to HTTPS" setting), except for
/// local hosts, where a developer meant plain HTTP.
pub fn upgrade_to_https(url: &str) -> String {
    let Ok(mut parsed) = Url::parse(url) else {
        return url.to_string();
    };
    if parsed.scheme() != "http" {
        return url.to_string();
    }
    let local = match parsed.host() {
        Some(Host::Domain(d)) => is_local_host(d),
        Some(_) => true,
        None => true,
    };
    if local || parsed.port().is_some() {
        return url.to_string();
    }
    if parsed.set_scheme("https").is_err() {
        return url.to_string();
    }
    parsed.to_string()
}

/// The short form the address bar shows while it is not being edited: the site's host without
/// `www.` for web pages (Safari's default), the whole address otherwise. With `full` the whole
/// address shows, minus a trailing slash on a bare host.
pub fn display(url: &str, full: bool) -> String {
    let Ok(parsed) = Url::parse(url) else {
        return url.to_string();
    };
    if !matches!(parsed.scheme(), "http" | "https") {
        return url.to_string();
    }
    if full {
        let s = parsed.to_string();
        return if parsed.path() == "/" && parsed.query().is_none() && parsed.fragment().is_none() {
            s.trim_end_matches('/').to_string()
        } else {
            s
        };
    }
    let host = idna::domain_to_unicode(parsed.host_str().unwrap_or_default()).0;
    let host = host.strip_prefix("www.").unwrap_or(&host);
    match parsed.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_string(),
    }
}

/// The text to edit when the bar gains focus: the whole address, decoded for reading
/// (`%20` stays, but punycode hosts and percent-encoded letters read as typed).
pub fn editable(url: &str) -> String {
    if url == "about:blank" {
        return String::new();
    }
    match Url::parse(url) {
        Ok(parsed) if matches!(parsed.scheme(), "http" | "https") => {
            let s = parsed.to_string();
            percent_encoding::percent_decode_str(&s)
                .decode_utf8()
                .map(|d| {
                    if d.chars().any(|c| c.is_whitespace() || c.is_control()) {
                        s.clone()
                    } else {
                        d.into_owned()
                    }
                })
                .unwrap_or(s)
        }
        _ => url.to_string(),
    }
}

/// Whether the page is served securely, for the lock in the address bar.
pub fn is_secure(url: &str) -> bool {
    Url::parse(url).is_ok_and(|u| matches!(u.scheme(), "https" | "about" | "data"))
}

/// The registrable-looking host of a page for display and grouping: lowercased, without
/// `www.`; `None` for pages that have no host.
pub fn site(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    Some(host.strip_prefix("www.").unwrap_or(&host).to_string())
}

/// The key history and bookmarks deduplicate by: the URL without its fragment, as the `url`
/// crate canonicalizes it (lowercase scheme and host, default port dropped, `/` path on a bare
/// host). `None` for pages history never records.
pub fn history_key(url: &str) -> Option<String> {
    let mut parsed = Url::parse(url).ok()?;
    if !matches!(parsed.scheme(), "http" | "https" | "file") {
        return None;
    }
    parsed.set_fragment(None);
    Some(parsed.to_string())
}

/// The page's address without its scheme and `www.`, which is what a user types and what
/// completion matches against: `https://www.example.com/a?b` → `example.com/a?b`.
pub fn strip_scheme(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    rest.strip_prefix("www.").unwrap_or(rest).to_string()
}

/// A home-page setting made into an address: blank means "none", a bare host gets `https://`.
pub fn home_page(setting: &str) -> Option<String> {
    match classify(setting)? {
        Input::Navigate(url) => Some(url),
        Input::Search(_) => None,
    }
}

/// The monogram a site without a favicon shows: the first letter of its host, uppercased.
pub fn monogram(url: &str) -> String {
    site(url)
        .map(|s| idna::domain_to_unicode(&s).0)
        .and_then(|s| s.chars().find(|c| c.is_alphanumeric()))
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_else(|| "•".into())
}

/// A stable hue (0–359) for a site's monogram tile, so each site keeps its color.
pub fn monogram_hue(url: &str) -> u16 {
    let key = site(url).unwrap_or_default();
    let mut h: u32 = 2166136261;
    for b in key.bytes() {
        h = (h ^ u32::from(b)).wrapping_mul(16777619);
    }
    (h % 360) as u16
}

/// The user agents Settings offers beside the default, as `(id, name, string)`. The names are
/// products and stay as they are in every language. The strings are what each browser sends
/// today with its version reduced the way the browser itself reduces it; a site reads them to
/// choose a layout, so the phone ones get phone pages and the computer ones desktop pages.
pub const USER_AGENTS: &[(&str, &str, &str)] = &[
    (
        "safari-iphone",
        "Safari (iPhone)",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.6 Mobile/15E148 Safari/604.1",
    ),
    (
        "chrome-android",
        "Chrome (Android)",
        "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Mobile Safari/537.36",
    ),
    (
        "firefox-android",
        "Firefox (Android)",
        "Mozilla/5.0 (Android 15; Mobile; rv:144.0) Gecko/144.0 Firefox/144.0",
    ),
    (
        "safari-mac",
        "Safari (Mac)",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.6 Safari/605.1.15",
    ),
    (
        "chrome-windows",
        "Chrome (Windows)",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36",
    ),
    (
        "firefox-windows",
        "Firefox (Windows)",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:144.0) Gecko/20100101 Firefox/144.0",
    ),
];

/// The setting's value for the platform browser's own string, which is the default.
pub const USER_AGENT_DEFAULT: &str = "default";
/// The setting's value for the engine's embedded-view string.
pub const USER_AGENT_ENGINE: &str = "engine";
/// The setting's value for a string the user typed.
pub const USER_AGENT_CUSTOM: &str = "custom";

/// What a user agent setting asks the web view for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UserAgentChoice {
    /// The platform browser's string.
    Browser,
    /// The engine's own.
    Engine,
    /// This string.
    Text(String),
}

/// Resolve the setting `id` (with the typed `custom` string). An unknown id, and a custom
/// choice with nothing typed, are the default.
pub fn user_agent_choice(id: &str, custom: &str) -> UserAgentChoice {
    if id == USER_AGENT_ENGINE {
        return UserAgentChoice::Engine;
    }
    if id == USER_AGENT_CUSTOM {
        let custom = custom.trim();
        return if custom.is_empty() {
            UserAgentChoice::Browser
        } else {
            UserAgentChoice::Text(custom.to_string())
        };
    }
    USER_AGENTS
        .iter()
        .find(|(preset, _, _)| *preset == id)
        .map(|(_, _, text)| UserAgentChoice::Text((*text).to_string()))
        .unwrap_or(UserAgentChoice::Browser)
}
