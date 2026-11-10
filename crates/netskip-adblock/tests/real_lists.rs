// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! Build time, cache size, load time and a few known decisions with real filter lists. The
//! lists are not part of the repository (EasyList is share-alike); point `DAY_ADBLOCK_LISTS` at
//! a directory holding `easylist.txt`, `easyprivacy.txt` and optionally
//! `abp-filters-anti-cv.txt` and `fanboy-cookiemonster.txt`:
//!
//! ```sh
//! DAY_ADBLOCK_LISTS=/path/to/lists cargo test --release --test real_lists -- --ignored --nocapture
//! ```

mod common;

use std::time::{Duration, Instant};

use common::*;
use netskip_adblock::{AdBlock, Level, Purpose, RequestInfo, ResourceKind, Settings, Source};

#[test]
#[ignore = "needs real lists: DAY_ADBLOCK_LISTS=<dir>"]
fn real_lists() {
    let Ok(dir) = std::env::var("DAY_ADBLOCK_LISTS") else {
        println!("DAY_ADBLOCK_LISTS is not set; nothing to measure");
        return;
    };
    let dir = std::path::Path::new(&dir);
    let server = ListServer::new();
    let mut sources = Vec::new();
    for (file, purpose) in [
        ("easylist.txt", Purpose::Ads),
        ("easyprivacy.txt", Purpose::Privacy),
        ("abp-filters-anti-cv.txt", Purpose::Ads),
        ("fanboy-cookiemonster.txt", Purpose::Cookies),
    ] {
        let Ok(text) = std::fs::read_to_string(dir.join(file)) else {
            continue;
        };
        println!("{file}: {} bytes", text.len());
        let url = format!("https://lists.test/{file}");
        server.serve(&url, Serve::Body(text));
        sources.push(Source::custom(&url).with_title(file).with_purpose(purpose));
    }
    assert!(!sources.is_empty(), "no lists in {}", dir.display());
    let level = Level::Custom {
        sources: sources.iter().map(|s| s.id.clone()).collect(),
        document_blocking: false,
        same_site_blocking: false,
    };
    let settings = Settings {
        level: level.clone(),
        custom_sources: sources,
        ..Settings::default()
    };
    let cache = scratch_dir("real");
    let clock = FakeClock::new();
    let build = |server: &ListServer| {
        AdBlock::builder()
            .cache_dir(&cache)
            .client(server.client())
            .clock(std::sync::Arc::new(clock.clone()))
            .settings(settings.clone())
            .build()
            .expect("blocker")
    };

    let adblock = build(&server);
    let started = Instant::now();
    let report = block_on(adblock.update_now());
    println!("download (simulated) and compile: {:?}", started.elapsed());
    assert!(report.recompiled, "{report:?}");

    let engine = std::fs::read_dir(&cache)
        .unwrap()
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with("engine-"))
        .expect("engine file");
    println!(
        "engine cache file: {:.1} MB",
        engine.metadata().unwrap().len() as f64 / 1e6
    );

    let page = "https://news.example/";
    let checks = [
        (
            "https://securepubads.g.doubleclick.net/tag/js/gpt.js",
            ResourceKind::Script,
            true,
        ),
        (
            "https://www.google-analytics.com/analytics.js",
            ResourceKind::Script,
            true,
        ),
        (
            "https://www.googletagmanager.com/gtag/js?id=G-1",
            ResourceKind::Script,
            true,
        ),
        ("https://example.com/", ResourceKind::Image, false),
        (
            "https://news.example/static/app.js",
            ResourceKind::Script,
            false,
        ),
    ];
    for (url, kind, expected) in checks {
        let d = adblock.decide(&RequestInfo::new(url, page, kind));
        println!("{url}: block={} list={:?}", d.block, d.matched_list);
        assert_eq!(d.block, expected, "{url}");
    }

    let n = 20_000;
    let started = Instant::now();
    for i in 0..n {
        let (url, kind, _) = checks[i % checks.len()];
        std::hint::black_box(adblock.decide(&RequestInfo::new(url, page, kind)));
    }
    let per = started.elapsed() / n as u32;
    println!("decide: {per:?} per request");
    assert!(per < Duration::from_millis(1));

    let started = Instant::now();
    let cosmetic = adblock.cosmetic("https://www.cnn.com/");
    println!(
        "cosmetic(cnn.com): {} selectors, generichide={}, {:?}",
        cosmetic.hide_selectors.len(),
        cosmetic.generichide,
        started.elapsed()
    );
    drop(adblock);

    // A second launch loads the cache file instead of compiling.
    let offline = ListServer::new();
    let again = build(&offline);
    let started = Instant::now();
    block_on(again.prepare()).unwrap();
    println!(
        "second launch, engine loaded from cache: {:?}",
        started.elapsed()
    );
    let d = again.decide(&RequestInfo::new(checks[0].0, page, checks[0].1));
    assert!(d.block);
}
