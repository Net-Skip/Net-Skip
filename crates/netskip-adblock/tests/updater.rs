// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! The updater against a simulated list server and a clock the test moves: first download,
//! conditional requests and `304`, failures that keep the last good copy, backoff, mirrors,
//! the `! Expires:` schedule, and the waiting status before any list arrives.

mod common;

use std::borrow::Cow;
use std::time::Duration;

use common::*;
use netskip_adblock::{
    AdBlock, FIRST_RETRY, Level, ListFormat, MAX_REFRESH, MIN_REFRESH, Purpose, RequestInfo,
    ResourceKind, Source, SourceId, Status, UpdateOutcome,
};

const HOUR: u64 = 3_600;
const DAY: u64 = 24 * HOUR;

fn outcome(report: &netskip_adblock::UpdateReport, id: &SourceId) -> UpdateOutcome {
    report
        .results
        .iter()
        .find(|(i, _)| i == id)
        .map(|(_, o)| o.clone())
        .unwrap_or_else(|| panic!("no result for {id}: {report:?}"))
}

fn state(adblock: &AdBlock, id: &SourceId) -> netskip_adblock::ListState {
    adblock
        .lists()
        .into_iter()
        .find(|l| &l.source.id == id)
        .map(|l| l.state)
        .unwrap_or_else(|| panic!("no list {id}"))
}

fn ads_blocked(adblock: &AdBlock) -> bool {
    adblock
        .decide(&RequestInfo::new(
            "https://ads.example/a.png",
            "https://news.example/",
            ResourceKind::Image,
        ))
        .block
}

#[test]
fn first_download_schedules_by_expires() {
    let dir = scratch_dir("first");
    let server = ListServer::with_fixtures();
    let clock = FakeClock::new();
    let adblock = blocker(&dir, &server, &clock, fixture_level(false, false));
    assert_eq!(adblock.status(), Status::WaitingForLists);
    assert!(!ads_blocked(&adblock));

    let report = block_on(adblock.update_due());
    assert!(report.recompiled);
    for source in [ads_source(), privacy_source(), hosts_source()] {
        assert_eq!(outcome(&report, &source.id), UpdateOutcome::Updated);
        assert!(dir.join(format!("{}.txt", source.id)).is_file());
    }
    assert_eq!(
        adblock.status(),
        Status::Ready {
            lists: 3,
            missing: vec![]
        }
    );
    assert!(ads_blocked(&adblock));

    let now = clock.secs();
    let ads = state(&adblock, &ads_source().id);
    assert_eq!(ads.url.as_deref(), Some(ADS_URL));
    assert!(ads.etag.is_some());
    assert_eq!(ads.fetched_at, Some(now));
    assert_eq!(ads.refresh_secs, Some(2 * DAY));
    let next = ads.next_fetch.unwrap();
    assert!(
        (now + 2 * DAY..=now + 2 * DAY + 2 * DAY / 10).contains(&next),
        "{next}"
    );
    assert_eq!(ads.rules.network, 6);
    assert!(ads.rules.cosmetic >= 4);
    assert_eq!(
        state(&adblock, &privacy_source().id).refresh_secs,
        Some(6 * HOUR)
    );
    // A hosts file has no Expires line: one day.
    assert_eq!(state(&adblock, &hosts_source().id).refresh_secs, Some(DAY));

    // state.json carries the same facts.
    let json = std::fs::read_to_string(dir.join("state.json")).unwrap();
    assert!(json.contains("\"etag\""), "{json}");
    assert!(json.contains(&ads_source().id.to_string()));

    // The earliest list (privacy, six hours) sets the next update.
    let next_update = adblock.next_update().unwrap();
    let in_secs = next_update
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - now;
    assert!(
        (6 * HOUR..=6 * HOUR + 6 * HOUR / 10).contains(&in_secs),
        "{in_secs}"
    );
}

#[test]
fn due_lists_revalidate_with_their_etag() {
    let (adblock, server, clock, _dir) = ready_blocker(fixture_level(false, false));
    let requests = server.log.lock().unwrap().len();

    // Nothing is due yet: no request at all.
    let report = block_on(adblock.update_due());
    assert!(
        report
            .results
            .iter()
            .all(|(_, o)| *o == UpdateOutcome::NotDue)
    );
    assert_eq!(server.log.lock().unwrap().len(), requests);

    // Seven hours on, only the six-hour list is due, and it is unchanged.
    clock.advance(Duration::from_secs(7 * HOUR));
    let report = block_on(adblock.update_due());
    assert_eq!(
        outcome(&report, &privacy_source().id),
        UpdateOutcome::NotModified
    );
    assert_eq!(outcome(&report, &ads_source().id), UpdateOutcome::NotDue);
    assert!(!report.recompiled);
    let sent = server.requests_to(PRIVACY_URL);
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0], None);
    assert!(
        sent[1].is_some(),
        "the second request carries If-None-Match"
    );
    let privacy = state(&adblock, &privacy_source().id);
    assert_eq!(privacy.fetched_at, Some(clock.secs()));
    assert_eq!(privacy.changed_at, Some(clock.secs() - 7 * HOUR));

    // update_now asks for everything, still conditionally.
    let report = block_on(adblock.update_now());
    assert!(
        report
            .results
            .iter()
            .all(|(_, o)| *o == UpdateOutcome::NotModified),
        "{report:?}"
    );
    assert!(server.requests_to(ADS_URL).last().unwrap().is_some());
}

#[test]
fn a_changed_list_recompiles() {
    let (adblock, server, clock, dir) = ready_blocker(fixture_level(false, false));
    let fresh =
        |url: &'static str| RequestInfo::new(url, "https://news.example/", ResourceKind::Script);
    assert!(
        !adblock
            .decide(&fresh("https://newtracker.example/t.js"))
            .block
    );
    let engines_before = engine_files(&dir);
    assert_eq!(engines_before.len(), 1);

    server.serve(
        PRIVACY_URL,
        Serve::Body(format!("{PRIVACY}||newtracker.example^\n")),
    );
    clock.advance(Duration::from_secs(8 * HOUR));
    let report = block_on(adblock.update_due());
    assert_eq!(
        outcome(&report, &privacy_source().id),
        UpdateOutcome::Updated
    );
    assert!(report.recompiled);
    assert!(
        adblock
            .decide(&fresh("https://newtracker.example/t.js"))
            .block
    );
    // The old engine file is replaced, not kept beside the new one.
    let engines_after = engine_files(&dir);
    assert_eq!(engines_after.len(), 1);
    assert_ne!(engines_before, engines_after);
}

fn engine_files(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("engine-") && n.ends_with(".bin"))
        .collect();
    names.sort();
    names
}

#[test]
fn failures_keep_the_last_copy_and_back_off() {
    let (adblock, server, clock, dir) = ready_blocker(fixture_level(false, false));
    let ads = ads_source().id;
    let path = dir.join(format!("{ads}.txt"));
    let stored = std::fs::read_to_string(&path).unwrap();

    server.serve(ADS_URL, Serve::Status(503));
    let report = block_on(adblock.update_now());
    assert!(
        matches!(outcome(&report, &ads), UpdateOutcome::Failed { error } if error.contains("503"))
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), stored);
    assert!(ads_blocked(&adblock), "the last good copy stays in use");
    let st = state(&adblock, &ads);
    assert_eq!(st.failures, 1);
    let first = FIRST_RETRY.as_secs();
    let wait = st.next_fetch.unwrap() - clock.secs();
    assert!((first..=first + first / 10).contains(&wait), "{wait}");

    // The second failure waits twice as long. An HTML error page is a failure too.
    clock.advance(Duration::from_secs(wait));
    server.serve(
        ADS_URL,
        Serve::Body("<!doctype html><title>Oops</title>".into()),
    );
    let report = block_on(adblock.update_due());
    assert!(
        matches!(outcome(&report, &ads), UpdateOutcome::Failed { error } if error.contains("HTML"))
    );
    let st = state(&adblock, &ads);
    assert_eq!(st.failures, 2);
    let wait = st.next_fetch.unwrap() - clock.secs();
    assert!(
        (2 * first..=2 * first + 2 * first / 10).contains(&wait),
        "{wait}"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), stored);

    // Recovery resets the count and returns to the list's own schedule.
    clock.advance(Duration::from_secs(wait));
    server.serve(ADS_URL, Serve::Body(ADS.into()));
    let report = block_on(adblock.update_due());
    assert_eq!(outcome(&report, &ads), UpdateOutcome::NotModified);
    let st = state(&adblock, &ads);
    assert_eq!(st.failures, 0);
    assert!(st.next_fetch.unwrap() - clock.secs() >= 2 * DAY);
}

#[test]
fn mirrors_are_tried_in_order() {
    let dir = scratch_dir("mirror");
    let server = ListServer::new();
    let clock = FakeClock::new();
    let mut source = Source::custom("https://lists.test/gone.txt").with_purpose(Purpose::Privacy);
    source
        .urls
        .push(Cow::Borrowed("https://lists.test/mirror.txt"));
    server.serve("https://lists.test/gone.txt", Serve::Status(404));
    server.serve("https://lists.test/mirror.txt", Serve::Body(PRIVACY.into()));
    let settings = netskip_adblock::Settings {
        level: Level::Custom {
            sources: vec![source.id.clone()],
            document_blocking: false,
            same_site_blocking: false,
        },
        custom_sources: vec![source.clone()],
        ..Default::default()
    };
    let adblock = AdBlock::builder()
        .cache_dir(&dir)
        .client(server.client())
        .clock(std::sync::Arc::new(clock.clone()))
        .settings(settings)
        .build()
        .unwrap();
    let report = block_on(adblock.update_now());
    assert_eq!(outcome(&report, &source.id), UpdateOutcome::Updated);
    assert_eq!(
        state(&adblock, &source.id).url.as_deref(),
        Some("https://lists.test/mirror.txt")
    );
}

#[test]
fn expires_is_clamped() {
    let dir = scratch_dir("clamp");
    let server = ListServer::new();
    let clock = FakeClock::new();
    let long = Source::custom("https://lists.test/long.txt");
    let short = Source::custom("https://lists.test/short.txt").with_format(ListFormat::Adblock);
    server.serve(
        "https://lists.test/long.txt",
        Serve::Body("! Expires: 90 days\n||long.example^\n".into()),
    );
    server.serve(
        "https://lists.test/short.txt",
        Serve::Body("! Expires: 0 hours\n||short.example^\n".into()),
    );
    let settings = netskip_adblock::Settings {
        level: Level::Custom {
            sources: vec![long.id.clone(), short.id.clone()],
            document_blocking: false,
            same_site_blocking: false,
        },
        custom_sources: vec![long.clone(), short.clone()],
        ..Default::default()
    };
    let adblock = AdBlock::builder()
        .cache_dir(&dir)
        .client(server.client())
        .clock(std::sync::Arc::new(clock.clone()))
        .settings(settings)
        .build()
        .unwrap();
    block_on(adblock.update_now());
    assert_eq!(
        state(&adblock, &long.id).refresh_secs,
        Some(MAX_REFRESH.as_secs())
    );
    assert_eq!(
        state(&adblock, &short.id).refresh_secs,
        Some(MIN_REFRESH.as_secs())
    );
}

#[test]
fn nothing_blocks_until_a_list_arrives() {
    let dir = scratch_dir("waiting");
    let server = ListServer::new(); // every list answers 404
    let clock = FakeClock::new();
    let adblock = blocker(&dir, &server, &clock, fixture_level(false, false));
    let report = block_on(adblock.update_due());
    assert!(
        report
            .results
            .iter()
            .all(|(_, o)| matches!(o, UpdateOutcome::Failed { .. }))
    );
    assert!(!report.recompiled);
    assert_eq!(adblock.status(), Status::WaitingForLists);
    assert!(!ads_blocked(&adblock));
    assert!(adblock.lists().iter().all(|l| !l.downloaded));

    // One list arrives: the blocker works with it and names the others as missing.
    server.serve(PRIVACY_URL, Serve::Body(PRIVACY.into()));
    block_on(adblock.update_now());
    match adblock.status() {
        Status::Ready { lists, missing } => {
            assert_eq!(lists, 1);
            assert_eq!(missing.len(), 2);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_new_launch_reuses_state_and_engine_without_the_network() {
    let (adblock, server, clock, dir) = ready_blocker(fixture_level(false, false));
    let decisions = sample_decisions(&adblock);
    let requests = server.log.lock().unwrap().len();
    drop(adblock);

    let offline = ListServer::new();
    let again = blocker(&dir, &offline, &clock, fixture_level(false, false));
    assert_eq!(again.status(), Status::Loading);
    block_on(again.prepare()).unwrap();
    assert_eq!(sample_decisions(&again), decisions);
    // Nothing is due, so the update makes no request either.
    let report = block_on(again.update_due());
    assert!(
        report
            .results
            .iter()
            .all(|(_, o)| *o == UpdateOutcome::NotDue)
    );
    assert!(offline.log.lock().unwrap().is_empty());
    assert_eq!(server.log.lock().unwrap().len(), requests);
}

pub fn sample_decisions(adblock: &AdBlock) -> Vec<(bool, Option<SourceId>)> {
    let page = "https://news.example/";
    [
        ("https://ads.example/a.png", ResourceKind::Image),
        ("https://ads.example/allowed.js", ResourceKind::Script),
        ("https://tracker.example/t.js", ResourceKind::Script),
        ("https://widgets.example/w.js", ResourceKind::Script),
        ("https://hosts-blocked.example/", ResourceKind::Xhr),
        ("https://cdn.example/banner/ad-2.js", ResourceKind::Script),
        ("https://example.com/", ResourceKind::Image),
    ]
    .iter()
    .map(|(url, kind)| {
        let d = adblock.decide(&RequestInfo::new(url, page, *kind));
        (d.block, d.matched_list)
    })
    .collect()
}

#[test]
fn start_runs_in_the_background() {
    let dir = scratch_dir("start");
    let server = ListServer::with_fixtures();
    let clock = FakeClock::new();
    let adblock = blocker(&dir, &server, &clock, fixture_level(false, false));
    let changes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = changes.clone();
    adblock.on_change(move || {
        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    });
    adblock.start();
    let wait_for = |what: &str, done: &dyn Fn() -> bool| {
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        while !done() {
            assert!(
                std::time::Instant::now() < deadline,
                "timed out waiting for {what}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    };
    wait_for("the engine", &|| {
        matches!(adblock.status(), Status::Ready { lists: 3, .. })
    });
    assert!(ads_blocked(&adblock));
    assert!(changes.load(std::sync::atomic::Ordering::Relaxed) > 0);

    // A site level that needs another engine is compiled in the background as well.
    let first_party = RequestInfo::new(
        "https://news.example/banner/ad-1.js",
        "https://news.example/",
        ResourceKind::Script,
    );
    assert!(!adblock.decide(&first_party).block);
    adblock.set_site_level("news.example", Some(fixture_level(true, true)));
    wait_for("the site engine", &|| adblock.decide(&first_party).block);
    adblock.stop();
}
