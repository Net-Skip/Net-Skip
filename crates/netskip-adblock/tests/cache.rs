// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! The engine cache file (serialize, reload, same decisions; a damaged file is rebuilt) and
//! the WebKit content rule export (valid JSON, rule shapes, chunking, versioning).

mod common;

use common::*;
use netskip_adblock::{AdBlock, ContentRules, Level, RequestInfo, ResourceKind, Status};
use serde_json::Value;

fn decisions(adblock: &AdBlock) -> Vec<(bool, Option<String>)> {
    let page = "https://news.example/";
    [
        ("https://ads.example/a.png", ResourceKind::Image),
        ("https://ads.example/allowed.js", ResourceKind::Script),
        ("https://tracker.example/t.js", ResourceKind::Script),
        ("https://widgets.example/w.js", ResourceKind::Script),
        ("https://hosts-blocked.example/", ResourceKind::Xhr),
        ("https://strict.example/promo/ok.js", ResourceKind::Script),
        ("https://example.com/", ResourceKind::Image),
    ]
    .iter()
    .map(|(url, kind)| {
        let d = adblock.decide(&RequestInfo::new(url, page, *kind));
        (d.block, d.matched_list.map(|id| id.to_string()))
    })
    .collect()
}

fn engine_file(dir: &std::path::Path) -> std::path::PathBuf {
    std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            name.starts_with("engine-custom-") && name.ends_with(".bin")
        })
        .expect("an engine file")
}

#[test]
fn engine_cache_round_trip() {
    let level = fixture_level(false, false);
    let (adblock, _server, clock, dir) = ready_blocker(level.clone());
    let before = decisions(&adblock);
    let cosmetic_before = adblock.cosmetic("https://news.example/");
    assert_eq!(before[0], (true, Some(ads_source().id.to_string())));
    drop(adblock);

    // Rewrite the stored lists behind the cache's back. A reloaded engine must come from the
    // cache file, so it still answers as before.
    for source in [ads_source(), privacy_source(), hosts_source()] {
        std::fs::write(
            dir.join(format!("{}.txt", source.id)),
            "||unrelated.example^\n",
        )
        .unwrap();
    }
    let offline = ListServer::new();
    let again = blocker(&dir, &offline, &clock, level.clone());
    block_on(again.prepare()).unwrap();
    assert_eq!(decisions(&again), before);
    assert_eq!(again.cosmetic("https://news.example/"), cosmetic_before);
    drop(again);

    // A damaged cache file is rebuilt from the stored lists, which now block nothing of it.
    let file = engine_file(&dir);
    let mut bytes = std::fs::read(&file).unwrap();
    let mid = bytes.len() / 2;
    bytes.truncate(mid);
    std::fs::write(&file, bytes).unwrap();
    let rebuilt = blocker(&dir, &offline, &clock, level);
    block_on(rebuilt.prepare()).unwrap();
    assert!(matches!(rebuilt.status(), Status::Ready { lists: 3, .. }));
    assert!(decisions(&rebuilt).iter().all(|(block, _)| !block));
    assert!(
        rebuilt
            .decide(&RequestInfo::new(
                "https://unrelated.example/x.js",
                "https://news.example/",
                ResourceKind::Script
            ))
            .block
    );
}

fn rules_of(rules: &ContentRules) -> Vec<Vec<Value>> {
    rules
        .lists
        .iter()
        .map(|l| {
            let json = l.read_json().unwrap();
            let parsed: Value = serde_json::from_str(&json).expect("valid JSON");
            let array = parsed.as_array().expect("a JSON array").clone();
            assert_eq!(array.len(), l.rules);
            array
        })
        .collect()
}

fn action(rule: &Value) -> &str {
    rule["action"]["type"].as_str().unwrap()
}

#[test]
fn webkit_export_shapes() {
    let level = fixture_level(false, false);
    let (adblock, ..) = ready_blocker(level.clone());
    let rules = block_on(adblock.content_rules(&level)).unwrap();
    assert_eq!(rules.lists.len(), 1);
    assert!(rules.lists[0].identifier.starts_with("day-adblock-custom-"));
    let all = rules_of(&rules).remove(0);
    for rule in &all {
        assert!(rule["trigger"]["url-filter"].is_string(), "{rule}");
    }

    let blocks_ads = all.iter().any(|r| {
        action(r) == "block"
            && r["trigger"]["url-filter"]
                .as_str()
                .unwrap()
                .contains("ads\\.example")
    });
    assert!(blocks_ads, "||ads.example^ becomes a block rule");
    let tracker = all
        .iter()
        .find(|r| {
            action(r) == "block"
                && r["trigger"]["url-filter"]
                    .as_str()
                    .unwrap()
                    .contains("tracker\\.example")
        })
        .expect("tracker rule");
    assert_eq!(
        tracker["trigger"]["load-type"],
        serde_json::json!(["third-party"])
    );
    let hide = |selector: &str| {
        all.iter()
            .find(|r| action(r) == "css-display-none" && r["action"]["selector"] == selector)
            .cloned()
    };
    // Generic hiding is the cosmetic script's: as WebKit rules it would stall heavy pages.
    assert!(hide(".ad-banner").is_none());
    let sponsored = hide(".sponsored").expect("site rule");
    assert_eq!(
        sponsored["trigger"]["if-domain"],
        serde_json::json!(["*news.example"])
    );
    let allowed = all.iter().any(|r| {
        action(r) == "ignore-previous-rules"
            && r["trigger"]["url-filter"]
                .as_str()
                .unwrap()
                .contains("allowed\\.js")
    });
    assert!(
        allowed,
        "@@||ads.example/allowed.js becomes ignore-previous-rules"
    );
    // Same-site blocking off: the last rules except first-party loads.
    let first_party = all.iter().any(|r| {
        action(r) == "ignore-previous-rules"
            && r["trigger"]["load-type"] == serde_json::json!(["first-party"])
            && r["trigger"]["url-filter"] == ".*"
            && !r["trigger"]["resource-type"]
                .as_array()
                .unwrap()
                .contains(&Value::from("document"))
    });
    assert!(first_party);
    // No scriptlet reaches WebKit.
    let text = rules.lists[0].read_json().unwrap();
    assert!(!text.contains("js(") && !text.contains("abort-on-property-read"));
    // Exceptions come after every block and hide rule.
    let first_exception = all
        .iter()
        .position(|r| action(r) == "ignore-previous-rules")
        .unwrap();
    assert!(
        all[first_exception..]
            .iter()
            .all(|r| action(r) == "ignore-previous-rules")
    );

    // Unchanged lists keep the version and reuse the files.
    let again = block_on(adblock.content_rules(&level)).unwrap();
    assert_eq!(again, rules);

    // Document and same-site blocking change the exceptions and the version.
    let strict = fixture_level(true, true);
    adblock.set_level(strict.clone());
    let strict_rules = block_on(adblock.content_rules(&strict)).unwrap();
    assert_ne!(strict_rules.version, rules.version);
    let strict_all = rules_of(&strict_rules).remove(0);
    assert!(!strict_all.iter().any(|r| {
        action(r) == "ignore-previous-rules"
            && r["trigger"]["load-type"] == serde_json::json!(["first-party"])
    }));

    // Off exports nothing.
    let off = block_on(adblock.content_rules(&Level::Off)).unwrap();
    assert!(off.lists.is_empty());
}

#[test]
fn webkit_export_chunks_under_the_limit() {
    let level = fixture_level(false, false);
    let dir = scratch_dir("chunks");
    let server = ListServer::with_fixtures();
    let clock = FakeClock::new();
    let whole = {
        let adblock = blocker(&dir, &server, &clock, level.clone());
        block_on(adblock.update_now());
        rules_of(&block_on(adblock.content_rules(&level)).unwrap()).remove(0)
    };
    let exceptions: Vec<&Value> = whole
        .iter()
        .filter(|r| action(r) == "ignore-previous-rules")
        .collect();
    let others: Vec<&Value> = whole
        .iter()
        .filter(|r| action(r) != "ignore-previous-rules")
        .collect();

    let limit = exceptions.len() + 3;
    let small = netskip_adblock::AdBlock::builder()
        .cache_dir(&dir)
        .client(server.client())
        .clock(std::sync::Arc::new(clock.clone()))
        .settings(fixture_settings(level.clone()))
        .webkit_rule_limit(limit)
        .build()
        .unwrap();
    let chunked = block_on(small.content_rules(&level)).unwrap();
    let lists = rules_of(&chunked);
    assert_eq!(lists.len(), others.len().div_ceil(3));
    let mut seen = Vec::new();
    for (n, list) in lists.iter().enumerate() {
        assert!(list.len() <= limit);
        assert_eq!(
            chunked.lists[n].identifier.rsplit('-').next(),
            Some(n.to_string().as_str())
        );
        // Every chunk ends with all the exceptions, so each one stands alone in WebKit.
        let tail: Vec<&Value> = list[list.len() - exceptions.len()..].iter().collect();
        assert_eq!(tail, exceptions);
        seen.extend(list[..list.len() - exceptions.len()].iter().cloned());
    }
    assert_eq!(seen.iter().collect::<Vec<_>>(), others);

    // A limit that leaves no room for blocks is an error, not an empty export.
    let tiny = netskip_adblock::AdBlock::builder()
        .cache_dir(scratch_dir("tiny"))
        .client(server.client())
        .clock(std::sync::Arc::new(clock.clone()))
        .settings(fixture_settings(level.clone()))
        .webkit_rule_limit(1)
        .build()
        .unwrap();
    block_on(tiny.update_now());
    assert!(block_on(tiny.content_rules(&level)).is_err());
}
