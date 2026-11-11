// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Search engines: the catalog, query and suggestion URLs, the suggestion formats, and reading
//! a search back out of a results page's URL.

use url::Url;

/// The placeholder a query template carries where the encoded search terms go. `%s` is accepted
/// in a custom template too, the spelling Chrome and Firefox use for keyword searches.
pub const TERMS: &str = "{searchTerms}";

/// How an engine answers its suggestion endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuggestionFormat {
    /// OpenSearch suggestions: `["query", ["one", "two", …], …]`.
    OpenSearch,
    /// A bare JSON array of strings: `["one", "two"]`.
    Strings,
    /// Qwant's envelope: `{"data": {"items": [{"value": "one"}, …]}}`.
    Qwant,
}

/// One search engine the browser can send queries to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchEngine {
    /// The stable identifier stored in preferences. Matches the ids the Skip-based Net Skip
    /// stored, so a migrated preference keeps its choice.
    pub id: &'static str,
    /// The engine's own name. A proper noun, shown untranslated.
    pub name: &'static str,
    /// The page the home button opens when no home page is set.
    pub home: &'static str,
    /// The results page, with [`TERMS`] where the query goes.
    pub query: &'static str,
    /// The suggestion endpoint and its format, where the engine offers one without a key.
    pub suggest: Option<(&'static str, SuggestionFormat)>,
}

/// Every built-in engine, in the order the settings list shows them. DuckDuckGo leads as the default.
pub const ENGINES: &[SearchEngine] = &[
    SearchEngine {
        id: "duckduckgo",
        name: "DuckDuckGo",
        home: "https://duckduckgo.com/",
        query: "https://duckduckgo.com/?q={searchTerms}",
        suggest: Some((
            "https://duckduckgo.com/ac/?q={searchTerms}&type=list",
            SuggestionFormat::OpenSearch,
        )),
    },
    SearchEngine {
        id: "kagi",
        name: "Kagi",
        home: "https://kagi.com/",
        query: "https://kagi.com/search?q={searchTerms}",
        suggest: Some((
            "https://kagi.com/api/autosuggest?q={searchTerms}",
            SuggestionFormat::OpenSearch,
        )),
    },
    SearchEngine {
        id: "google",
        name: "Google",
        home: "https://www.google.com/",
        query: "https://www.google.com/search?q={searchTerms}",
        suggest: Some((
            "https://suggestqueries.google.com/complete/search?client=firefox&q={searchTerms}",
            SuggestionFormat::OpenSearch,
        )),
    },
    SearchEngine {
        id: "bing",
        name: "Bing",
        home: "https://www.bing.com/",
        query: "https://www.bing.com/search?q={searchTerms}",
        suggest: Some((
            "https://api.bing.com/osjson.aspx?query={searchTerms}",
            SuggestionFormat::OpenSearch,
        )),
    },
    SearchEngine {
        id: "ecosia",
        name: "Ecosia",
        home: "https://www.ecosia.org/",
        query: "https://www.ecosia.org/search?q={searchTerms}",
        suggest: Some((
            "https://ac.ecosia.org/autocomplete?q={searchTerms}&type=list",
            SuggestionFormat::OpenSearch,
        )),
    },
    SearchEngine {
        id: "qwant",
        name: "Qwant",
        home: "https://www.qwant.com/",
        query: "https://www.qwant.com/?q={searchTerms}",
        suggest: Some((
            "https://api.qwant.com/v3/suggest?q={searchTerms}",
            SuggestionFormat::Qwant,
        )),
    },
    SearchEngine {
        id: "startpage",
        name: "Startpage",
        home: "https://www.startpage.com/",
        query: "https://www.startpage.com/do/dsearch?query={searchTerms}",
        suggest: Some((
            "https://www.startpage.com/osuggestions?q={searchTerms}",
            SuggestionFormat::OpenSearch,
        )),
    },
    SearchEngine {
        id: "swisscows",
        name: "Swisscows",
        home: "https://swisscows.com/",
        query: "https://swisscows.com/web?query={searchTerms}",
        suggest: Some((
            "https://api.swisscows.com/suggest?query={searchTerms}",
            SuggestionFormat::Strings,
        )),
    },
    SearchEngine {
        id: "yahoo",
        name: "Yahoo!",
        home: "https://search.yahoo.com/",
        query: "https://search.yahoo.com/search?p={searchTerms}",
        suggest: Some((
            "https://search.yahoo.com/sugg/os?command={searchTerms}&output=fxjson",
            SuggestionFormat::OpenSearch,
        )),
    },
    SearchEngine {
        id: "searx",
        name: "SearXNG",
        home: "https://searx.be/",
        query: "https://searx.be/search?q={searchTerms}",
        suggest: None,
    },
    SearchEngine {
        id: "yandex",
        name: "Yandex",
        home: "https://yandex.com/",
        query: "https://yandex.com/search/?text={searchTerms}",
        suggest: Some((
            "https://suggest.yandex.com/suggest-ff.cgi?part={searchTerms}",
            SuggestionFormat::OpenSearch,
        )),
    },
    SearchEngine {
        id: "baidu",
        name: "Baidu",
        home: "https://www.baidu.com/",
        query: "https://www.baidu.com/s?wd={searchTerms}",
        suggest: None,
    },
    SearchEngine {
        id: "dogpile",
        name: "Dogpile",
        home: "https://www.dogpile.com/",
        query: "https://www.dogpile.com/serp?q={searchTerms}",
        suggest: None,
    },
    SearchEngine {
        id: "gigablast",
        name: "Gigablast",
        home: "https://www.gigablast.com/",
        query: "https://www.gigablast.com/search?q={searchTerms}",
        suggest: None,
    },
];

/// The engine a fresh install searches with.
pub const DEFAULT_ENGINE: &str = "duckduckgo";

/// The id the preferences store for a user-supplied template.
pub const CUSTOM_ENGINE: &str = "custom";

/// The engine to search with: a built-in one, or a template the user typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Engine {
    BuiltIn(&'static SearchEngine),
    /// A results-page template holding `%s` or [`TERMS`].
    Custom(String),
}

impl Engine {
    /// The engine for a stored choice. An unknown id, or a custom choice whose template is
    /// unusable, falls back to the default engine rather than leaving search broken.
    pub fn resolve(id: &str, custom_template: &str) -> Engine {
        if id == CUSTOM_ENGINE
            && let Some(template) = normalize_template(custom_template)
        {
            return Engine::Custom(template);
        }
        Engine::BuiltIn(engine(id).unwrap_or_else(default_engine))
    }

    /// The engine's display name; `None` for a custom template, which the UI names itself.
    pub fn name(&self) -> Option<&'static str> {
        match self {
            Engine::BuiltIn(e) => Some(e.name),
            Engine::Custom(_) => None,
        }
    }

    /// The host a custom template points at, for naming it ("Search example.org").
    pub fn host(&self) -> Option<String> {
        let template = match self {
            Engine::BuiltIn(e) => e.query,
            Engine::Custom(t) => t,
        };
        Url::parse(&template.replace(TERMS, "x"))
            .ok()
            .and_then(|u| {
                u.host_str()
                    .map(|h| h.trim_start_matches("www.").to_string())
            })
    }

    /// The results page for `terms`.
    pub fn query_url(&self, terms: &str) -> String {
        match self {
            Engine::BuiltIn(e) => fill(e.query, terms),
            Engine::Custom(t) => fill(t, terms),
        }
    }

    /// The suggestion request for `terms`, when the engine has an endpoint.
    pub fn suggestion_url(&self, terms: &str) -> Option<(String, SuggestionFormat)> {
        match self {
            Engine::BuiltIn(e) => e.suggest.map(|(t, f)| (fill(t, terms), f)),
            Engine::Custom(_) => None,
        }
    }

    /// The home page: the engine's front page, or a custom template's origin.
    pub fn home_url(&self) -> String {
        match self {
            Engine::BuiltIn(e) => e.home.to_string(),
            Engine::Custom(t) => Url::parse(&t.replace(TERMS, "x"))
                .map(|u| format!("{}/", u.origin().ascii_serialization()))
                .unwrap_or_else(|_| default_engine().home.to_string()),
        }
    }

    /// The search terms when `url` is this engine's results page, so the address bar can show
    /// what was searched for rather than the results page's address.
    pub fn terms_in(&self, url: &str) -> Option<String> {
        let template = match self {
            Engine::BuiltIn(e) => e.query,
            Engine::Custom(t) => t.as_str(),
        };
        terms_for_template(template, url)
    }
}

/// The built-in engine with this id.
pub fn engine(id: &str) -> Option<&'static SearchEngine> {
    ENGINES.iter().find(|e| e.id == id)
}

/// The default engine, DuckDuckGo.
pub fn default_engine() -> &'static SearchEngine {
    // The catalog's first entry is the default; the test `duckduckgo_is_the_default` holds it there.
    &ENGINES[0]
}

/// A custom template made usable: `%s` becomes [`TERMS`], a missing scheme becomes `https://`,
/// and a template with no placeholder or no host is refused.
pub fn normalize_template(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut template = trimmed.replace("%s", TERMS);
    if !template.contains(TERMS) {
        return None;
    }
    if !template.contains("://") {
        template = format!("https://{template}");
    }
    let probe = Url::parse(&template.replace(TERMS, "x")).ok()?;
    if !matches!(probe.scheme(), "http" | "https") || probe.host_str().is_none() {
        return None;
    }
    Some(template)
}

/// Encode the terms for a query string: spaces as `+`, everything else reserved escaped.
pub fn encode_terms(terms: &str) -> String {
    url::form_urlencoded::byte_serialize(terms.trim().as_bytes()).collect()
}

fn fill(template: &str, terms: &str) -> String {
    template.replace(TERMS, &encode_terms(terms))
}

/// Read the terms back out of a results URL made from `template`: same host (ignoring a
/// leading `www.`), same path, and the query parameter the template puts the terms in.
fn terms_for_template(template: &str, url: &str) -> Option<String> {
    let marker = "netskipterms";
    let template = Url::parse(&template.replace(TERMS, marker)).ok()?;
    let param = template
        .query_pairs()
        .find(|(_, v)| v == marker)
        .map(|(k, _)| k.into_owned())?;
    let url = Url::parse(url).ok()?;
    let host = |u: &Url| {
        u.host_str()
            .map(|h| h.trim_start_matches("www.").to_ascii_lowercase())
    };
    if host(&template) != host(&url) || template.path() != url.path() {
        return None;
    }
    url.query_pairs()
        .find(|(k, _)| *k == param)
        .map(|(_, v)| v.into_owned())
        .filter(|v| !v.trim().is_empty())
}

/// Parse a suggestion response. Malformed input yields no suggestions rather than an error: a
/// suggestion list is a convenience, and an engine changing its format must not break typing.
/// Blank entries and repeats are dropped, and the list is capped at `limit`.
pub fn parse_suggestions(format: SuggestionFormat, body: &[u8], limit: usize) -> Vec<String> {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return Vec::new();
    };
    let items: Vec<String> = match format {
        SuggestionFormat::OpenSearch => value
            .get(1)
            .and_then(|v| v.as_array())
            .map(|a| strings(a))
            .unwrap_or_default(),
        SuggestionFormat::Strings => value.as_array().map(|a| strings(a)).unwrap_or_default(),
        SuggestionFormat::Qwant => value
            .pointer("/data/items")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|i| i.get("value").and_then(|v| v.as_str()))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    };
    let mut out: Vec<String> = Vec::new();
    for item in items {
        let item = item.trim().to_string();
        if !item.is_empty() && !out.iter().any(|o| o.eq_ignore_ascii_case(&item)) {
            out.push(item);
        }
        if out.len() >= limit {
            break;
        }
    }
    out
}

fn strings(a: &[serde_json::Value]) -> Vec<String> {
    a.iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect()
}
