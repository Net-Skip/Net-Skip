// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
use netskip_model::search::*;

#[test]
fn kagi_is_the_default() {
    assert_eq!(default_engine().id, DEFAULT_ENGINE);
    assert_eq!(default_engine().name, "Kagi");
}

#[test]
fn every_previous_net_skip_engine_is_available() {
    for id in [
        "duckduckgo",
        "google",
        "swisscows",
        "bing",
        "yahoo",
        "ecosia",
        "qwant",
        "startpage",
        "searx",
        "yandex",
        "baidu",
        "gigablast",
        "dogpile",
        "kagi",
    ] {
        assert!(engine(id).is_some(), "{id} missing");
    }
}

#[test]
fn ids_are_unique_and_templates_are_well_formed() {
    for (i, e) in ENGINES.iter().enumerate() {
        assert!(
            ENGINES[i + 1..].iter().all(|o| o.id != e.id),
            "{} repeated",
            e.id
        );
        assert!(e.query.starts_with("https://"), "{}", e.id);
        assert!(e.query.contains(TERMS), "{}", e.id);
        assert!(
            e.home.starts_with("https://") && e.home.ends_with('/'),
            "{}",
            e.id
        );
        if let Some((s, _)) = e.suggest {
            assert!(s.starts_with("https://") && s.contains(TERMS), "{}", e.id);
        }
        // Every results page reads its own terms back.
        let engine = Engine::BuiltIn(e);
        let url = engine.query_url("rust & café");
        assert_eq!(
            engine.terms_in(&url).as_deref(),
            Some("rust & café"),
            "{}",
            e.id
        );
    }
}

#[test]
fn query_urls_encode_terms() {
    let kagi = Engine::resolve("kagi", "");
    assert_eq!(
        kagi.query_url("rust lang"),
        "https://kagi.com/search?q=rust+lang"
    );
    assert_eq!(
        kagi.query_url("a&b=c?"),
        "https://kagi.com/search?q=a%26b%3Dc%3F"
    );
    assert_eq!(
        kagi.query_url("  padded  "),
        "https://kagi.com/search?q=padded"
    );
    assert_eq!(
        kagi.query_url("日本"),
        "https://kagi.com/search?q=%E6%97%A5%E6%9C%AC"
    );
    let ddg = Engine::resolve("duckduckgo", "");
    assert_eq!(ddg.query_url("x"), "https://duckduckgo.com/?q=x");
    assert_eq!(
        ddg.suggestion_url("x y").unwrap().0,
        "https://duckduckgo.com/ac/?q=x+y&type=list"
    );
}

#[test]
fn unknown_ids_fall_back_to_the_default() {
    assert_eq!(
        Engine::resolve("altavista", ""),
        Engine::BuiltIn(default_engine())
    );
    assert_eq!(Engine::resolve("", ""), Engine::BuiltIn(default_engine()));
    // A custom choice with an unusable template searches with the default too.
    assert_eq!(
        Engine::resolve(CUSTOM_ENGINE, "no placeholder"),
        Engine::BuiltIn(default_engine())
    );
}

#[test]
fn custom_templates() {
    assert_eq!(
        normalize_template("search.example.org/find?q=%s").as_deref(),
        Some("https://search.example.org/find?q={searchTerms}")
    );
    assert_eq!(
        normalize_template(" https://x.org/?s={searchTerms} ").as_deref(),
        Some("https://x.org/?s={searchTerms}")
    );
    assert_eq!(normalize_template("https://x.org/"), None);
    assert_eq!(normalize_template(""), None);
    assert_eq!(normalize_template("ftp://x.org/?q=%s"), None);
    assert_eq!(normalize_template("javascript:%s"), None);

    let custom = Engine::resolve(CUSTOM_ENGINE, "https://www.example.org/find?q=%s&lang=en");
    assert_eq!(custom.name(), None);
    assert_eq!(custom.host().as_deref(), Some("example.org"));
    assert_eq!(
        custom.query_url("a b"),
        "https://www.example.org/find?q=a+b&lang=en"
    );
    assert_eq!(custom.home_url(), "https://www.example.org/");
    assert_eq!(custom.suggestion_url("a"), None);
    assert_eq!(
        custom
            .terms_in("https://example.org/find?lang=en&q=hello+world")
            .as_deref(),
        Some("hello world")
    );
}

#[test]
fn terms_are_only_read_from_the_engines_own_results_page() {
    let google = Engine::resolve("google", "");
    assert_eq!(
        google
            .terms_in("https://www.google.com/search?q=cats")
            .as_deref(),
        Some("cats")
    );
    assert_eq!(
        google
            .terms_in("https://google.com/search?q=cats&hl=fr")
            .as_deref(),
        Some("cats")
    );
    assert_eq!(google.terms_in("https://www.google.com/maps?q=cats"), None);
    assert_eq!(google.terms_in("https://evil.example/search?q=cats"), None);
    assert_eq!(google.terms_in("https://www.google.com/search?q="), None);
    assert_eq!(google.terms_in("garbage"), None);
}

#[test]
fn opensearch_suggestions() {
    let body =
        br#"["sail",["sailor moon","sail loft","Sailor Moon","  ","sailfish","sailboat"],[],[]]"#;
    assert_eq!(
        parse_suggestions(SuggestionFormat::OpenSearch, body, 3),
        vec!["sailor moon", "sail loft", "sailfish"]
    );
}

#[test]
fn string_and_qwant_suggestions() {
    assert_eq!(
        parse_suggestions(SuggestionFormat::Strings, br#"["a","b"]"#, 10),
        vec!["a", "b"]
    );
    let qwant = br#"{"status":"success","data":{"items":[{"value":"sailor","suggestType":0},{"value":"sailboat"}]}}"#;
    assert_eq!(
        parse_suggestions(SuggestionFormat::Qwant, qwant, 10),
        vec!["sailor", "sailboat"]
    );
}

#[test]
fn malformed_suggestions_are_empty() {
    for format in [
        SuggestionFormat::OpenSearch,
        SuggestionFormat::Strings,
        SuggestionFormat::Qwant,
    ] {
        assert!(parse_suggestions(format, b"<html>", 5).is_empty());
        assert!(parse_suggestions(format, b"", 5).is_empty());
        assert!(parse_suggestions(format, b"{}", 5).is_empty());
        assert!(parse_suggestions(format, b"[1,[2,3]]", 5).is_empty());
    }
}
