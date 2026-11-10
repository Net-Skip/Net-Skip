// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
use netskip_model::fixtures::*;
use netskip_model::history::*;
use netskip_model::tabs::*;

const HOUR: i64 = 3_600_000;
const DAY: i64 = 86_400_000;
// 2026-11-09T15:00:00Z, a Monday.
const NOW: i64 = 1_794_236_400_000;

#[test]
fn civil_dates() {
    assert_eq!(civil(0), (1970, 1, 1));
    assert_eq!(civil(-1), (1969, 12, 31));
    assert_eq!(civil(19_723), (2024, 1, 1));
    assert_eq!(civil(19_782), (2024, 2, 29));
    assert_eq!(civil(local_day(NOW, 0)), (2026, 11, 9));
    assert_eq!(weekday(0), 3); // Thursday
    assert_eq!(weekday(local_day(NOW, 0)), 0); // Monday
}

#[test]
fn local_days_follow_the_offset() {
    // 15:00 UTC is 01:00 the next day in UTC+10 and 07:00 the same day in UTC-8.
    assert_eq!(local_day(NOW, 10 * 3600), local_day(NOW, 0) + 1);
    assert_eq!(local_day(NOW, -8 * 3600), local_day(NOW, 0));
}

#[test]
fn day_groups() {
    assert_eq!(day_group(NOW - HOUR, NOW, 0), DayGroup::Today);
    assert_eq!(day_group(NOW + HOUR, NOW, 0), DayGroup::Today);
    assert_eq!(day_group(NOW - DAY, NOW, 0), DayGroup::Yesterday);
    // Two days back from Monday is Saturday.
    assert_eq!(day_group(NOW - 2 * DAY, NOW, 0), DayGroup::Weekday(5));
    assert_eq!(day_group(NOW - 6 * DAY, NOW, 0), DayGroup::Weekday(1));
    let old = day_group(NOW - 7 * DAY, NOW, 0);
    assert_eq!(old, DayGroup::Date(local_day(NOW, 0) - 7));
    // Just after local midnight, a visit before it is yesterday.
    let tz = -15 * 3600; // local time is exactly 00:00
    assert_eq!(day_group(NOW - 1000, NOW, tz), DayGroup::Yesterday);
    assert!(DayGroup::Today < DayGroup::Yesterday);
}

#[test]
fn clear_ranges() {
    assert_eq!(ClearRange::LastHour.since(NOW, 0), Some(NOW - HOUR));
    let midnight = NOW - 15 * HOUR;
    assert_eq!(ClearRange::Today.since(NOW, 0), Some(midnight));
    assert_eq!(
        ClearRange::TodayAndYesterday.since(NOW, 0),
        Some(midnight - DAY)
    );
    assert_eq!(ClearRange::All.since(NOW, 0), None);
    // In UTC+2 local midnight is 22:00 UTC the day before.
    assert_eq!(
        ClearRange::Today.since(NOW, 2 * 3600),
        Some(midnight - 2 * HOUR)
    );
    let ids: Vec<_> = ClearRange::ALL.iter().map(|r| r.id()).collect();
    assert_eq!(ids, ["hour", "today", "two-days", "all"]);
}

#[test]
fn what_history_records() {
    assert!(records("https://example.com/", false));
    assert!(!records("https://example.com/", true));
    assert!(!records("about:blank", false));
    assert!(!records("data:text/html,hi", false));
    assert!(!records("", false));
    let v = visit("https://Example.com/a#frag", "  Title ", true, false).unwrap();
    assert_eq!(v.key, "https://example.com/a");
    assert_eq!(v.title.as_deref(), Some("Title"));
    assert!(v.typed);
    assert_eq!(
        visit("https://example.com/", "   ", false, false)
            .unwrap()
            .title,
        None
    );
    assert_eq!(visit("https://example.com/", "x", false, true), None);
}

#[test]
fn new_tabs_without_an_opener_go_last() {
    let order = [1, 2, 3];
    assert_eq!(insertion_index(&order, None, None), 3);
    assert_eq!(insertion_index::<i32>(&[], None, None), 0);
    // An opener that is gone also appends.
    assert_eq!(insertion_index(&order, Some(&9), None), 3);
}

#[test]
fn link_tabs_follow_their_opener_in_order() {
    let mut order = vec![1, 2, 3];
    let mut run = OpenerRun::new();
    // Three links opened in the background from tab 1.
    for child in [10, 11, 12] {
        let at = insertion_index(&order, Some(&1), run.last_child(&1));
        order.insert(at, child);
        run.opened(Some(&1), &child, false);
    }
    assert_eq!(order, [1, 10, 11, 12, 2, 3]);
    // Selecting another tab ends the run: the next link goes right after the opener again.
    run.selected(&2);
    run.selected(&1);
    let at = insertion_index(&order, Some(&1), run.last_child(&1));
    assert_eq!(at, 1);
}

#[test]
fn a_foreground_link_tab_ends_the_run() {
    let mut run = OpenerRun::new();
    run.opened(Some(&1), &10, false);
    assert_eq!(run.last_child(&1), Some(&10));
    run.opened(Some(&1), &11, true);
    assert_eq!(run.last_child(&1), None);
    run.opened(Some(&1), &12, false);
    run.opened(None, &13, false);
    assert_eq!(run.last_child(&1), None);
    // Reselecting the opener keeps it.
    run.opened(Some(&1), &14, false);
    run.selected(&1);
    assert_eq!(run.last_child(&1), Some(&14));
}

#[test]
fn a_child_moved_before_its_opener_is_ignored() {
    let order = [10, 1, 2];
    assert_eq!(insertion_index(&order, Some(&1), Some(&10)), 2);
}

#[test]
fn closed_tabs_are_bounded_and_skip_private_and_blank() {
    let mut closed = ClosedTabs::new();
    let tab = |i: usize| ClosedTab {
        url: format!("https://e.example/{i}"),
        title: format!("{i}"),
        index: i,
    };
    assert!(!closed.push(tab(0), true));
    assert!(!closed.push(
        ClosedTab {
            url: String::new(),
            title: String::new(),
            index: 0
        },
        false
    ));
    assert!(!closed.push(
        ClosedTab {
            url: "about:blank".into(),
            title: String::new(),
            index: 0
        },
        false
    ));
    assert!(closed.is_empty());
    for i in 0..30 {
        assert!(closed.push(tab(i), false));
    }
    assert_eq!(closed.len(), ClosedTabs::CAPACITY);
    assert_eq!(closed.recent().next().unwrap().index, 29);
    assert_eq!(closed.take(1).unwrap().index, 28);
    assert_eq!(closed.pop().unwrap().index, 29);
    assert_eq!(closed.pop().unwrap().index, 27);
    assert_eq!(closed.take(100), None);
    closed.clear();
    assert_eq!(closed.pop(), None);
}

#[test]
fn sessions_leave_private_tabs_out() {
    let tabs = [
        ("https://a.example/", "A", false, 0),
        ("https://secret.example/", "S", true, 0),
        ("https://b.example/", "B", false, 0),
    ];
    let s = Session::capture(tabs, Some(2));
    assert_eq!(s.tabs.len(), 2);
    assert_eq!(s.tabs[1].url, "https://b.example/");
    assert_eq!(s.selected, 1);
    // A selected private tab hands the selection to the tab before it.
    assert_eq!(Session::capture(tabs, Some(1)).selected, 0);
    assert_eq!(Session::capture(tabs, None).selected, 0);
    assert_eq!(
        Session::capture([("x", "", true, 0)], Some(0)).tabs.len(),
        0
    );
}

#[test]
fn sessions_round_trip_and_survive_bad_input() {
    let s = Session::capture(
        [
            ("https://a.example/", "A", false, 5),
            ("https://b.example/", "B", false, 0),
        ],
        Some(1),
    );
    assert_eq!(Session::decode(&s.encode()), s);
    assert_eq!(s.tabs[0].viewed, 5);
    assert_eq!(Session::decode(""), Session::default());
    assert_eq!(Session::decode("{not json"), Session::default());
    let clamped = Session::decode(r#"{"tabs":[{"url":"https://a.example/"}],"selected":7}"#);
    assert_eq!(clamped.selected, 0);
    assert_eq!(clamped.tabs[0].title, "");
    assert_eq!(
        clamped.tabs[0].viewed, 0,
        "an older session reads as never viewed"
    );
}

#[test]
fn sessions_expire_tabs_not_viewed_lately() {
    const DAY: i64 = 86_400_000;
    let now = 100 * DAY;
    let tabs = [
        ("https://old.example/", "Old", false, now - 9 * DAY),
        ("https://recent.example/", "Recent", false, now - DAY / 2),
        ("https://unknown.example/", "Unknown", false, 0),
        (
            "https://selected.example/",
            "Selected",
            false,
            now - 40 * DAY,
        ),
        ("https://stale.example/", "Stale", false, now - 8 * DAY),
    ];
    let mut s = Session::capture(tabs, Some(3));
    s.expire(now, 7 * DAY);
    let urls: Vec<_> = s.tabs.iter().map(|t| t.url.as_str()).collect();
    assert_eq!(
        urls,
        [
            "https://recent.example/",
            "https://unknown.example/",
            "https://selected.example/"
        ]
    );
    assert_eq!(s.selected, 2, "the selection follows its tab");
    // A week's limit keeps a tab viewed six days ago.
    let mut s = Session::capture([("https://a.example/", "A", false, now - 6 * DAY)], None);
    s.expire(now, 7 * DAY);
    assert_eq!(s.tabs.len(), 1);
}

#[test]
fn fixture_addresses_round_trip() {
    let f = FixtureUrls::new("day-resource://p1");
    assert_eq!(f.base(), "day-resource://p1/");
    let cases = [
        (
            "https://example.com/news/?x=1#top",
            "day-resource://p1/https/example.com/news/?x=1#top",
        ),
        (
            "http://example.com/a",
            "day-resource://p1/http/example.com/a",
        ),
        (
            "https://example.com",
            "day-resource://p1/https/example.com/",
        ),
        (
            "https://example.com?q=1",
            "day-resource://p1/https/example.com/?q=1",
        ),
    ];
    for (logical, physical) in cases {
        assert_eq!(f.to_physical(logical), physical);
        let back = f.to_logical(physical);
        assert_eq!(f.to_physical(&back), physical, "{logical}");
    }
    assert_eq!(
        f.to_logical("day-resource://p1/https/example.com/"),
        "https://example.com/"
    );
    // Outside the namespace, addresses pass through untouched.
    assert_eq!(f.to_physical("about:blank"), "about:blank");
    assert_eq!(
        f.to_logical("https://elsewhere.example/"),
        "https://elsewhere.example/"
    );
    assert_eq!(
        f.to_logical("day-resource://p1/ftp/x"),
        "day-resource://p1/ftp/x"
    );
}

#[test]
fn fixture_pages_are_rewritten() {
    let f = FixtureUrls::new("https://day-resource.invalid/3/");
    let html = r#"<a href="https://example.com/a">a</a><img src='http://img.example/x.png'><div style="background:url(https://cdn.example/b.png)">"#;
    let out = f.rewrite_html(html);
    assert!(out.contains(r#"href="https://day-resource.invalid/3/https/example.com/a""#));
    assert!(out.contains("src='https://day-resource.invalid/3/http/img.example/x.png'"));
    assert!(out.contains("url(https://day-resource.invalid/3/https/cdn.example/b.png)"));
    // Prose mentioning an address is left alone.
    assert_eq!(
        f.rewrite_html("see https://example.com"),
        "see https://example.com"
    );
}

/// WebView2 serves the namespace from an `http://` virtual host: the rewritten addresses must not
/// be rewritten again.
#[test]
fn fixture_pages_are_rewritten_once_under_an_http_base() {
    let f = FixtureUrls::new("http://day-resource.p3/");
    let out =
        f.rewrite_html(r#"<a href="https://example.com/a"><img src="http://img.example/x.png">"#);
    assert_eq!(
        out,
        r#"<a href="http://day-resource.p3/https/example.com/a"><img src="http://day-resource.p3/http/img.example/x.png">"#
    );
}

#[test]
fn fixture_requests_and_files() {
    let r = parse_request("https/WWW.Example.com/news/").unwrap();
    assert_eq!(r.host, "www.example.com");
    assert_eq!(r.path, "/news/");
    assert_eq!(candidate_files(&r), ["example.com/news/index.html"]);
    let r = parse_request("/http/example.com").unwrap();
    assert_eq!(r.scheme, "http");
    assert_eq!(r.path, "/");
    assert_eq!(candidate_files(&r), ["example.com/index.html"]);
    let r = parse_request("https/example.com/about").unwrap();
    assert_eq!(
        candidate_files(&r),
        [
            "example.com/about",
            "example.com/about.html",
            "example.com/about/index.html"
        ]
    );
    let r = parse_request("https/example.com/style.css").unwrap();
    assert_eq!(candidate_files(&r), ["example.com/style.css"]);
    assert_eq!(parse_request("ftp/example.com/"), None);
    assert_eq!(parse_request("https/"), None);
    assert_eq!(parse_request(""), None);
}

#[test]
fn fixture_mime_and_escaping() {
    assert_eq!(mime_for("a/b/index.HTML"), "text/html; charset=utf-8");
    assert_eq!(mime_for("x.svg"), "image/svg+xml");
    assert_eq!(mime_for("noext"), "application/octet-stream");
    assert_eq!(
        escape_html(r#"<a href="x">&'"#),
        "&lt;a href=&quot;x&quot;&gt;&amp;&#39;"
    );
}

#[test]
fn closing_a_link_tab_returns_to_its_family() {
    // 1 opened 10 and 11; 11 opened 20.
    let order = [1, 10, 11, 20, 2];
    let opener = |k: &i32| match k {
        10 | 11 => Some(1),
        20 => Some(11),
        _ => None,
    };
    // The next sibling first, then the previous one.
    assert_eq!(select_after_close(&order, &10, opener), Some(11));
    assert_eq!(select_after_close(&order, &11, opener), Some(10));
    // An only child returns to its opener.
    assert_eq!(select_after_close(&order, &20, opener), Some(11));
    // A tab without an opener prefers its own children, then its neighbors.
    assert_eq!(select_after_close(&order, &1, opener), Some(10));
    assert_eq!(select_after_close(&order, &2, opener), Some(20));
    assert_eq!(select_after_close(&[5, 6], &5, |_| None), Some(6));
    assert_eq!(select_after_close(&[5], &5, |_| None), None);
    assert_eq!(select_after_close(&[5], &9, |_| None), None);
    // An opener that has closed is ignored.
    assert_eq!(
        select_after_close(&[10, 3], &10, |k| (*k == 10).then_some(1)),
        Some(3)
    );
}

#[test]
fn load_progress_only_moves_forward() {
    let p = LoadProgress::default();
    let p = p.report(true, 0.0);
    assert_eq!(
        p,
        LoadProgress {
            value: LoadProgress::START,
            visible: true
        }
    );
    let p = p.report(true, 0.6);
    assert_eq!(p.value, 0.6);
    // A redirect restarts the engine's estimate; the line holds.
    let p = p.report(true, 0.2);
    assert_eq!(p.value, 0.6);
    let p = p.report(false, 0.2);
    assert_eq!(
        p,
        LoadProgress {
            value: 1.0,
            visible: false
        }
    );
    assert_eq!(p.report(false, 0.0), p);
    assert_eq!(
        LoadProgress::default().report(true, f64::NAN).value,
        LoadProgress::START
    );
    assert_eq!(LoadProgress::default().report(true, 7.0).value, 1.0);
}
