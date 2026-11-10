// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! Request decisions, element hiding, the cosmetic script's messages, per-site levels and the
//! statistics, over the hand-written lists in tests/data.

mod common;

use common::*;
use netskip_adblock::{
    AdBlock, Level, Purpose, RequestInfo, ResourceKind, SourceId, Status, cosmetic_script,
};

fn blocks(adblock: &AdBlock, url: &str, page: &str, kind: ResourceKind) -> bool {
    adblock.decide(&RequestInfo::new(url, page, kind)).block
}

#[test]
fn network_rules() {
    let (adblock, ..) = ready_blocker(fixture_level(false, false));
    let page = "https://news.example/today";

    // ||ads.example^, attributed to the list that holds it.
    let d = adblock.decide(&RequestInfo::new(
        "https://ads.example/banner.png",
        page,
        ResourceKind::Image,
    ));
    assert!(d.block);
    assert_eq!(d.matched_list, Some(ads_source().id));
    assert_eq!(d.redirect, None);
    assert!(blocks(
        &adblock,
        "https://cdn.ads.example/x.js",
        page,
        ResourceKind::Script
    ));

    // @@||ads.example/allowed.js
    assert!(!blocks(
        &adblock,
        "https://ads.example/allowed.js",
        page,
        ResourceKind::Script
    ));
    // An exception in one list cancels a block in another.
    assert!(!blocks(
        &adblock,
        "https://ads.example/cross-allowed.png",
        page,
        ResourceKind::Image
    ));
    // ...unless the block is $important.
    let d = adblock.decide(&RequestInfo::new(
        "https://strict.example/promo/ok.js",
        page,
        ResourceKind::Script,
    ));
    assert!(d.block);
    assert_eq!(d.matched_list, Some(ads_source().id));

    // ||tracker.example^$third-party: blocked from another site, allowed on its own.
    let d = adblock.decide(&RequestInfo::new(
        "https://tracker.example/t.js",
        page,
        ResourceKind::Script,
    ));
    assert!(d.block);
    assert_eq!(d.matched_list, Some(privacy_source().id));
    assert!(!blocks(
        &adblock,
        "https://tracker.example/t.js",
        "https://tracker.example/",
        ResourceKind::Script
    ));

    // $domain=news.example|~safe.example
    assert!(blocks(
        &adblock,
        "https://widgets.example/w.js",
        page,
        ResourceKind::Script
    ));
    assert!(blocks(
        &adblock,
        "https://widgets.example/w.js",
        "https://sub.news.example/",
        ResourceKind::Script
    ));
    assert!(!blocks(
        &adblock,
        "https://widgets.example/w.js",
        "https://safe.example/",
        ResourceKind::Script
    ));
    assert!(!blocks(
        &adblock,
        "https://widgets.example/w.js",
        "https://other.example/",
        ResourceKind::Script
    ));

    // Hosts lines block the whole host; `localhost` is ignored.
    let d = adblock.decide(&RequestInfo::new(
        "https://hosts-blocked.example/a.gif",
        page,
        ResourceKind::Image,
    ));
    assert!(d.block);
    assert_eq!(d.matched_list, Some(hosts_source().id));
    assert!(blocks(
        &adblock,
        "https://another-blocked.example/",
        page,
        ResourceKind::Xhr
    ));
    assert!(!blocks(
        &adblock,
        "http://localhost/x.js",
        page,
        ResourceKind::Script
    ));

    // Unrelated requests pass.
    assert!(!blocks(
        &adblock,
        "https://news.example/app.js",
        page,
        ResourceKind::Script
    ));
    assert!(!blocks(
        &adblock,
        "https://example.com/",
        page,
        ResourceKind::Image
    ));
}

#[test]
fn script_type_rules_and_kind_inference() {
    // /banner/ad-*.js$script, on a third-party host.
    let (adblock, ..) = ready_blocker(fixture_level(false, false));
    let page = "https://news.example/";
    let url = "https://cdn.example/banner/ad-1.js";
    assert!(blocks(&adblock, url, page, ResourceKind::Script));
    assert!(!blocks(&adblock, url, page, ResourceKind::Image));
    // Android reports no type: the extension says script.
    assert!(blocks(&adblock, url, page, ResourceKind::Unknown));
    assert_eq!(ResourceKind::from_url(url), ResourceKind::Script);
    assert_eq!(
        ResourceKind::from_url("https://a.example/s.CSS?v=1#x"),
        ResourceKind::Stylesheet
    );
    assert_eq!(
        ResourceKind::from_url("https://a.example/img/logo.webp"),
        ResourceKind::Image
    );
    assert_eq!(
        ResourceKind::from_url("https://a.example/api"),
        ResourceKind::Other
    );
    assert_eq!(
        ResourceKind::from_url("https://a.example"),
        ResourceKind::Other
    );
}

#[test]
fn same_site_and_document_blocking_follow_the_level() {
    let first_party = "https://news.example/banner/ad-1.js";
    let page = "https://news.example/";

    let (light, ..) = ready_blocker(fixture_level(false, false));
    // Same-site requests pass when same-site blocking is off.
    assert!(!blocks(&light, first_party, page, ResourceKind::Script));
    // A navigation to a blocked host passes when document blocking is off.
    assert!(
        !light
            .decide(&RequestInfo::navigation("https://ads.example/"))
            .block
    );
    // An iframe from a blocked host is still blocked.
    assert!(blocks(
        &light,
        "https://ads.example/frame.html",
        page,
        ResourceKind::Subdocument
    ));

    let (strict, ..) = ready_blocker(fixture_level(true, true));
    assert!(blocks(&strict, first_party, page, ResourceKind::Script));
    let d = strict.decide(&RequestInfo::navigation("https://ads.example/"));
    assert!(d.block);
    assert_eq!(d.matched_list, Some(ads_source().id));
    assert!(
        !strict
            .decide(&RequestInfo::navigation("https://news.example/"))
            .block
    );
}

#[test]
fn element_hiding() {
    let (adblock, ..) = ready_blocker(fixture_level(false, false));

    // ##.ad-banner is generic: it comes back through generic_selectors for a seen class.
    let other = adblock.cosmetic("https://other.example/");
    assert!(!other.generichide);
    assert!(!other.hide_selectors.contains(&".sponsored".to_string()));
    // Generic rules that are not plain class or id selectors come with the page.
    assert!(
        other
            .hide_selectors
            .contains(&"div[data-ad-slot]".to_string())
    );
    let found = adblock.generic_selectors(
        "https://other.example/",
        &["ad-banner".into(), "story".into()],
        &[],
        &other.exceptions,
    );
    assert_eq!(found, vec![".ad-banner".to_string()]);

    // news.example##.sponsored and news.example#@#.ad-banner
    let news = adblock.cosmetic("https://www.news.example/story");
    assert!(news.hide_selectors.contains(&".sponsored".to_string()));
    assert!(news.exceptions.contains(&".ad-banner".to_string()));
    let found = adblock.generic_selectors(
        "https://www.news.example/story",
        &["ad-banner".into()],
        &[],
        &news.exceptions,
    );
    assert!(found.is_empty(), "{found:?}");

    // @@||quiet.example^$generichide
    let quiet = adblock.cosmetic("https://quiet.example/");
    assert!(quiet.generichide);
    assert!(
        !quiet
            .hide_selectors
            .contains(&"div[data-ad-slot]".to_string())
    );
}

#[test]
fn scriptlets_are_dropped() {
    // The fixture has a `##+js(...)` and a `#$#` rule for news.example; neither may surface as
    // a selector.
    let (adblock, ..) = ready_blocker(fixture_level(false, false));
    let news = adblock.cosmetic("https://news.example/");
    assert!(
        news.hide_selectors
            .iter()
            .all(|s| !s.contains("js(") && !s.contains("abort-on")),
        "{:?}",
        news.hide_selectors
    );
}

#[test]
fn cosmetic_script_messages() {
    let (adblock, ..) = ready_blocker(fixture_level(false, false));
    let script = cosmetic_script();
    assert!(script.contains("\"dayAdblock\""));
    assert!(script.contains("__dayAdblock"));

    let page = adblock
        .handle_script_message(r#"{"v":1,"type":"page","url":"https://news.example/"}"#)
        .expect("page answer");
    let parsed: serde_json::Value = serde_json::from_str(&page).expect("the answer is JSON data");
    assert!(parsed["hide"].is_array());
    assert!(page.contains(".sponsored"));

    let ids = adblock
        .handle_script_message(
            r#"{"v":1,"type":"ids","url":"https://other.example/","classes":["ad-banner","x"],"ids":[]}"#,
        )
        .expect("ids answer");
    assert!(ids.contains(r#"{"hide":[".ad-banner"]}"#), "{ids}");
    // Nothing to hide: no answer.
    assert_eq!(
        adblock.handle_script_message(
            r#"{"type":"ids","url":"https://other.example/","classes":["story"]}"#
        ),
        None
    );
    // generichide pages get no generic answers.
    assert_eq!(
        adblock.handle_script_message(
            r#"{"type":"ids","url":"https://quiet.example/","classes":["ad-banner"]}"#
        ),
        None
    );
    assert_eq!(adblock.handle_script_message("not json"), None);
    assert_eq!(adblock.handle_script_message(r#"{"type":"other"}"#), None);
}

#[test]
fn per_site_levels() {
    let (adblock, ..) = ready_blocker(fixture_level(false, false));
    let url = "https://ads.example/a.png";
    assert!(blocks(
        &adblock,
        url,
        "https://news.example/",
        ResourceKind::Image
    ));

    adblock.set_site_level("www.news.example", Some(Level::Off));
    assert_eq!(adblock.level_for("news.example"), Level::Off);
    assert_eq!(adblock.level_for("m.news.example"), Level::Off);
    assert!(!blocks(
        &adblock,
        url,
        "https://m.news.example/",
        ResourceKind::Image
    ));
    assert!(
        adblock
            .cosmetic("https://news.example/")
            .hide_selectors
            .is_empty()
    );
    assert!(blocks(
        &adblock,
        url,
        "https://other.example/",
        ResourceKind::Image
    ));

    // The settings round-trip through serde for the app to store.
    let json = serde_json::to_string(&adblock.settings()).unwrap();
    let back: netskip_adblock::Settings = serde_json::from_str(&json).unwrap();
    assert_eq!(back, adblock.settings());
    assert!(json.contains(r#""news.example":{"level":"off"}"#), "{json}");

    adblock.set_site_level("news.example", None);
    assert!(blocks(
        &adblock,
        url,
        "https://news.example/",
        ResourceKind::Image
    ));

    // The global level off blocks nothing and says so.
    adblock.set_level(Level::Off);
    assert_eq!(adblock.status(), Status::Off);
    assert!(!blocks(
        &adblock,
        url,
        "https://news.example/",
        ResourceKind::Image
    ));
}

#[test]
fn statistics_count_per_host_and_purpose() {
    let (adblock, ..) = ready_blocker(fixture_level(false, false));
    let page = "https://www.news.example/";
    for _ in 0..3 {
        blocks(
            &adblock,
            "https://ads.example/a.png",
            page,
            ResourceKind::Image,
        );
    }
    blocks(
        &adblock,
        "https://tracker.example/t.js",
        page,
        ResourceKind::Script,
    );
    blocks(
        &adblock,
        "https://news.example/app.js",
        page,
        ResourceKind::Script,
    );
    blocks(
        &adblock,
        "https://ads.example/a.png",
        "https://other.example/",
        ResourceKind::Image,
    );

    let news = adblock.blocked("news.example");
    assert_eq!(news.total, 4);
    assert_eq!(news.get(Purpose::Ads), 3);
    assert_eq!(news.get(Purpose::Privacy), 1);
    assert_eq!(adblock.blocked_total().total, 5);
    assert_eq!(adblock.blocked_by_host().len(), 2);

    // Counting from many threads at once loses nothing.
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let a = adblock.clone();
            std::thread::spawn(move || {
                for _ in 0..100 {
                    a.decide(&RequestInfo::new(
                        "https://ads.example/a.png",
                        "https://busy.example/",
                        ResourceKind::Image,
                    ));
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    assert_eq!(adblock.blocked("busy.example").get(Purpose::Ads), 800);

    adblock.reset_stats(Some("news.example"));
    assert_eq!(adblock.blocked("news.example").total, 0);
    adblock.reset_stats(None);
    assert_eq!(adblock.blocked_total().total, 0);
}

#[test]
fn levels_select_their_lists() {
    use netskip_adblock::{Settings, catalog};
    assert_eq!(
        ids(&Level::Light.sources()),
        ["easyprivacy", "fanboy-social"]
    );
    assert_eq!(
        ids(&Level::Strong.sources()),
        [
            "easylist",
            "abp-anti-cv",
            "adblock-warning-removal",
            "easyprivacy",
            "fanboy-social"
        ]
    );
    let aggressive = Level::Aggressive.sources();
    for id in Level::Strong.sources() {
        assert!(aggressive.contains(&id));
    }
    assert!(aggressive.contains(&SourceId::PETER_LOWE));
    assert!(aggressive.contains(&SourceId::EASYLIST_COOKIE));
    assert!(Level::Off.sources().is_empty());
    assert!(Level::Aggressive.document_blocking() && Level::Aggressive.same_site_blocking());
    assert!(!Level::Strong.document_blocking() && !Level::Light.same_site_blocking());

    // Extra sources join every level but Off.
    let settings = Settings {
        level: Level::Light,
        extra_sources: vec![SourceId::EASYLIST_GERMANY, SourceId::EASYPRIVACY],
        ..Settings::default()
    };
    assert_eq!(
        ids(&settings.sources_for(&Level::Light)),
        ["easyprivacy", "fanboy-social", "easylist-germany"]
    );
    assert!(settings.sources_for(&Level::Off).is_empty());

    // Every built-in list resolves, every id is unique, and every URL is upstream.
    let mut seen = std::collections::HashSet::new();
    for source in catalog() {
        assert!(seen.insert(source.id.clone()), "duplicate {}", source.id);
        assert!(!source.urls.is_empty());
        for url in &source.urls {
            assert!(url.starts_with("https://"), "{url}");
            assert!(!url.contains("vivaldi"), "{url}");
        }
        assert!(source.license.is_some());
    }
    for level in [Level::Light, Level::Strong, Level::Aggressive] {
        for id in level.sources() {
            assert!(netskip_adblock::builtin_source(&id).is_some(), "{id}");
        }
    }
    assert_eq!(
        netskip_adblock::regional_sources("de-AT")[0].id,
        SourceId::EASYLIST_GERMANY
    );
    // Level ids and serde names are stable.
    assert_eq!(
        serde_json::to_string(&Level::Strong).unwrap(),
        r#"{"level":"strong"}"#
    );
    assert_eq!(
        Level::Custom {
            sources: vec![],
            document_blocking: false,
            same_site_blocking: false
        }
        .id(),
        "custom"
    );
}
