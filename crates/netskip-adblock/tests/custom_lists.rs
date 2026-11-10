// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! Changing the lists in use without a gap in protection: staging downloads and compiles new
//! settings while the old ones keep blocking, applying them is immediate, a failed download is
//! reported and leaves the old protection in place, and a list added by address is checked
//! before it is used.

mod common;

use common::*;
use netskip_adblock::{Level, RequestInfo, ResourceKind, Source, SourceId, Status, UpdateOutcome};

const EXTRA_URL: &str = "https://lists.test/extra.txt";
const EXTRA: &str = "! Title: Extra\n||extra.example^\n";

fn blocks(adblock: &netskip_adblock::AdBlock, url: &str) -> bool {
    adblock
        .decide(&RequestInfo::new(
            url,
            "https://news.example/",
            ResourceKind::Image,
        ))
        .block
}

fn privacy_only() -> Level {
    Level::Custom {
        sources: vec![privacy_source().id],
        document_blocking: false,
        same_site_blocking: false,
    }
}

/// The fixture settings with the ad list and a list added by address switched on.
fn wider(adblock: &netskip_adblock::AdBlock) -> netskip_adblock::Settings {
    let extra = Source::custom(EXTRA_URL);
    let mut settings = adblock.settings();
    settings.level = Level::Custom {
        sources: vec![ads_source().id, privacy_source().id, extra.id.clone()],
        document_blocking: false,
        same_site_blocking: false,
    };
    settings.custom_sources.push(extra);
    settings
}

#[test]
fn staged_settings_apply_without_a_gap() {
    let (adblock, server, _clock, _dir) = ready_blocker(privacy_only());
    server.serve(EXTRA_URL, Serve::Body(EXTRA.into()));
    assert!(blocks(&adblock, "https://tracker.example/t.gif"));
    assert!(!blocks(&adblock, "https://ads.example/a.png"));

    let next = wider(&adblock);
    let report = block_on(adblock.stage(next.clone()));
    assert!(report.is_complete(), "{report:?}");
    // Only the lists with no stored copy were downloaded: the ad list and the new one.
    let mut fetched: Vec<_> = report
        .results
        .iter()
        .map(|(id, o)| (id.clone(), o.clone()))
        .collect();
    fetched.sort_by(|a, b| a.0.cmp(&b.0));
    let mut expected = vec![
        (ads_source().id, UpdateOutcome::Updated),
        (Source::custom(EXTRA_URL).id, UpdateOutcome::Updated),
    ];
    expected.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(fetched, expected);
    // Staging changed nothing yet: the old level still decides.
    assert_eq!(adblock.level(), privacy_only());
    assert!(blocks(&adblock, "https://tracker.example/t.gif"));
    assert!(!blocks(&adblock, "https://ads.example/a.png"));

    // A background pass meanwhile keeps the staged engine.
    block_on(adblock.prepare()).expect("prepare");

    // Applying is immediate: the engine is already compiled, nothing loads.
    adblock.set_settings(next);
    assert!(matches!(adblock.status(), Status::Ready { lists: 3, .. }));
    assert!(blocks(&adblock, "https://ads.example/a.png"));
    assert!(blocks(&adblock, "https://extra.example/x.png"));
    assert!(blocks(&adblock, "https://tracker.example/t.gif"));
}

#[test]
fn a_failed_download_is_reported_and_the_old_lists_keep_blocking() {
    let (adblock, server, _clock, _dir) = ready_blocker(privacy_only());
    server.serve(EXTRA_URL, Serve::Status(503));
    let report = block_on(adblock.stage(wider(&adblock)));
    let extra = Source::custom(EXTRA_URL).id;
    assert!(!report.is_complete());
    assert_eq!(report.missing, std::slice::from_ref(&extra));
    assert!(
        report
            .results
            .iter()
            .any(|(id, outcome)| *id == extra && matches!(outcome, UpdateOutcome::Failed { .. }))
    );
    assert_eq!(adblock.level(), privacy_only());
    assert!(blocks(&adblock, "https://tracker.example/t.gif"));
}

#[test]
fn a_list_added_by_address_is_checked_first() {
    let (adblock, server, _clock, _dir) = ready_blocker(privacy_only());
    for bad in [
        "http://lists.test/extra.txt",
        "lists.test/extra.txt",
        "https://",
        "https://a b",
    ] {
        assert!(block_on(adblock.check_list(bad)).is_err(), "{bad}");
    }
    let page = "https://lists.test/page.html";
    server.serve(
        page,
        Serve::Body("<!doctype html><title>Lists</title>".into()),
    );
    let error = block_on(adblock.check_list(page)).expect_err("a web page is not a list");
    assert!(error.contains("HTML"), "{error}");
    let empty = "https://lists.test/empty.txt";
    server.serve(empty, Serve::Body("! Title: nothing\n".into()));
    assert!(block_on(adblock.check_list(empty)).is_err());
    let gone = "https://lists.test/gone.txt";
    assert!(block_on(adblock.check_list(gone)).is_err());

    // A real list: its rules are counted, including the ones this engine cannot apply (the
    // fixture's scriptlet and CSS injection rules), and the stored copy is used afterwards.
    let check = block_on(adblock.check_list(ADS_URL)).expect("the ad list checks");
    assert_eq!(check.source.id, SourceId::custom(ADS_URL));
    assert_eq!(check.source.title, "Day test ads");
    assert_eq!(check.rules.network, 6);
    assert_eq!(check.rules.unsupported, 2);
    let before = server.requests_to(ADS_URL).len();
    let mut settings = adblock.settings();
    settings.level = Level::Custom {
        sources: vec![check.source.id.clone()],
        document_blocking: false,
        same_site_blocking: false,
    };
    let report = block_on(adblock.stage(settings));
    assert!(report.is_complete(), "{report:?}");
    assert!(report.results.is_empty(), "{report:?}");
    assert_eq!(server.requests_to(ADS_URL).len(), before);
}
