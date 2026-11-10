// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
use netskip_model::complete::*;

const NOW: i64 = 1_800_000_000_000;
const DAY: i64 = 86_400_000;

fn page(url: &str, title: &str, visits: u32, typed: u32, days_ago: i64) -> Candidate {
    Candidate {
        url: url.into(),
        title: title.into(),
        visits,
        typed,
        last_visit: NOW - days_ago * DAY,
        bookmarked: false,
    }
}

fn history() -> Vec<Candidate> {
    vec![
        page(
            "https://github.com/rust-lang/rust",
            "rust-lang/rust",
            12,
            0,
            1,
        ),
        page("https://github.com/", "GitHub", 30, 10, 0),
        page("https://gitlab.com/explore", "Explore · GitLab", 2, 0, 20),
        page("https://www.wikipedia.org/", "Wikipedia", 8, 4, 2),
        page(
            "https://en.wikipedia.org/wiki/Rust_(programming_language)",
            "Rust (programming language) - Wikipedia",
            3,
            0,
            5,
        ),
        page("http://localhost:3000/dashboard", "Dev dashboard", 5, 5, 0),
        page("https://news.ycombinator.com/", "Hacker News", 50, 20, 0),
        page(
            "https://docs.rs/serde/latest/serde/",
            "serde - Rust",
            4,
            0,
            3,
        ),
    ]
}

#[test]
fn frecency_prefers_recent_and_typed_visits() {
    let recent = page("https://a.example/", "", 5, 0, 1);
    let old = page("https://b.example/", "", 5, 0, 200);
    let typed = page("https://c.example/", "", 5, 3, 1);
    assert!(frecency(&recent, NOW) > frecency(&old, NOW));
    assert!(frecency(&typed, NOW) > frecency(&recent, NOW));
    let mut marked = recent.clone();
    marked.bookmarked = true;
    assert!(frecency(&marked, NOW) > frecency(&recent, NOW));
    // A bookmark never visited still scores.
    let saved = Candidate {
        url: "https://d.example/".into(),
        bookmarked: true,
        ..Default::default()
    };
    assert!(frecency(&saved, NOW) > 0.0);
}

#[test]
fn frecency_buckets_step_down_with_age() {
    let at = |days| frecency(&page("https://a.example/", "", 1, 0, days), NOW);
    assert!(at(0) > at(5));
    assert!(at(5) > at(20));
    assert!(at(20) > at(60));
    assert!(at(60) > at(400));
    assert_eq!(at(0), at(3));
}

#[test]
fn completes_to_the_host() {
    let c = inline("git", &history(), NOW).unwrap();
    assert_eq!(c.text, "github.com");
    assert_eq!(c.typed_chars, 3);
    assert_eq!(c.suffix(), "hub.com");
    assert_eq!(c.url, "https://github.com/");
}

#[test]
fn completion_keeps_the_typed_case() {
    let c = inline("GiT", &history(), NOW).unwrap();
    assert_eq!(c.text, "GiThub.com");
    assert_eq!(c.suffix(), "hub.com");
}

#[test]
fn the_busier_site_wins() {
    // gitlab has far fewer visits than github.
    assert_eq!(inline("gi", &history(), NOW).unwrap().text, "github.com");
    assert_eq!(inline("gitl", &history(), NOW).unwrap().text, "gitlab.com");
}

#[test]
fn www_may_be_typed_or_left_out() {
    let c = inline("wiki", &history(), NOW).unwrap();
    assert_eq!(c.text, "wikipedia.org");
    assert_eq!(c.url, "https://www.wikipedia.org/");
    let c = inline("www.wi", &history(), NOW).unwrap();
    assert_eq!(c.text, "www.wikipedia.org");
}

#[test]
fn subdomains_complete_from_their_start() {
    let c = inline("news.y", &history(), NOW).unwrap();
    assert_eq!(c.text, "news.ycombinator.com");
    let c = inline("en.", &history(), NOW).unwrap();
    assert_eq!(c.text, "en.wikipedia.org");
}

#[test]
fn ports_and_http_are_kept() {
    let c = inline("localh", &history(), NOW).unwrap();
    assert_eq!(c.text, "localhost:3000");
    assert_eq!(c.url, "http://localhost:3000/");
}

#[test]
fn a_typed_scheme_completes_within_it() {
    let c = inline("https://gith", &history(), NOW).unwrap();
    assert_eq!(c.text, "https://github.com");
    assert_eq!(c.url, "https://github.com/");
    // No https localhost was visited.
    assert_eq!(inline("https://localh", &history(), NOW), None);
    assert_eq!(inline("https://", &history(), NOW), None);
}

#[test]
fn a_slash_completes_whole_addresses() {
    let c = inline("github.com/ru", &history(), NOW).unwrap();
    assert_eq!(c.text, "github.com/rust-lang/rust");
    assert_eq!(c.url, "https://github.com/rust-lang/rust");
    let c = inline("docs.rs/s", &history(), NOW).unwrap();
    assert_eq!(c.text, "docs.rs/serde/latest/serde/");
}

#[test]
fn nothing_to_complete() {
    assert_eq!(inline("", &history(), NOW), None);
    assert_eq!(inline("git hub", &history(), NOW), None);
    assert_eq!(inline("?git", &history(), NOW), None);
    assert_eq!(inline("zzz", &history(), NOW), None);
    // Already complete: nothing to add.
    assert_eq!(inline("github.com", &history(), NOW), None);
    // Completion is a prefix match only.
    assert_eq!(inline("hub", &history(), NOW), None);
    assert_eq!(inline("git", &[], NOW), None);
}

#[test]
fn non_web_pages_never_complete() {
    let pages = vec![page("file:///Users/me/notes.html", "notes", 10, 10, 0)];
    assert_eq!(inline("fil", &pages, NOW), None);
}

#[test]
fn deleting_does_not_complete() {
    assert!(should_complete("gi", "git"));
    assert!(should_complete("", "g"));
    assert!(!should_complete("git", "gi"));
    assert!(!should_complete("github.com", "git"));
    assert!(!should_complete("git", "git"));
    // Replacing the text with something new completes.
    assert!(should_complete("git", "wik"));
    assert!(should_complete("git", "gix"));
}

#[test]
fn rows_lead_with_the_top_hit_then_search() {
    let pages = history();
    let hit = inline("git", &pages, NOW);
    let rows = rows(
        "git",
        hit.as_ref(),
        &pages,
        &[],
        &["git commands".into(), "GIT".into()],
        NOW,
        Limits::default(),
    );
    assert_eq!(
        rows[0],
        Row::Page {
            url: "https://github.com/".into(),
            title: "GitHub".into(),
            kind: RowKind::TopHit
        }
    );
    assert_eq!(
        rows[1],
        Row::Search {
            terms: "git".into(),
            suggested: false
        }
    );
    // The engine's echo of the typed text is dropped.
    assert_eq!(
        rows[2],
        Row::Search {
            terms: "git commands".into(),
            suggested: true
        }
    );
    // The top hit is not repeated among the history rows.
    let urls: Vec<_> = rows
        .iter()
        .filter_map(|r| match r {
            Row::Page { url, .. } => Some(url.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        urls.iter().filter(|u| **u == "https://github.com/").count(),
        1
    );
    assert!(urls.contains(&"https://github.com/rust-lang/rust"));
    assert!(urls.contains(&"https://gitlab.com/explore"));
}

#[test]
fn typed_addresses_offer_a_visit_row() {
    let rows = rows("example.org/x", None, &[], &[], &[], NOW, Limits::default());
    assert_eq!(
        rows[0],
        Row::Visit {
            url: "https://example.org/x".into()
        }
    );
    assert_eq!(
        rows[1],
        Row::Search {
            terms: "example.org/x".into(),
            suggested: false
        }
    );
}

#[test]
fn forced_searches_drop_the_question_mark() {
    let rows = rows("?rust", None, &history(), &[], &[], NOW, Limits::default());
    assert_eq!(
        rows[0],
        Row::Search {
            terms: "rust".into(),
            suggested: false
        }
    );
}

#[test]
fn every_word_must_match_title_or_address() {
    let rows = rows(
        "rust wikipedia",
        None,
        &history(),
        &[],
        &[],
        NOW,
        Limits::default(),
    );
    let pages: Vec<_> = rows
        .iter()
        .filter(|r| matches!(r, Row::Page { .. }))
        .collect();
    assert_eq!(pages.len(), 1);
    assert!(matches!(pages[0], Row::Page { url, .. } if url.contains("Rust_(programming")));
}

#[test]
fn open_tabs_and_bookmarks_are_labeled() {
    let mut pages = history();
    pages[3].bookmarked = true; // wikipedia
    let open =
        vec!["https://en.wikipedia.org/wiki/Rust_(programming_language)#History".to_string()];
    let rows = rows("wiki", None, &pages, &open, &[], NOW, Limits::default());
    let kinds: Vec<_> = rows
        .iter()
        .filter_map(|r| match r {
            Row::Page { url, kind, .. } => Some((url.clone(), kind.clone())),
            _ => None,
        })
        .collect();
    assert!(kinds.contains(&("https://www.wikipedia.org/".into(), RowKind::Bookmark)));
    assert!(kinds.contains(&(
        "https://en.wikipedia.org/wiki/Rust_(programming_language)".into(),
        RowKind::OpenTab
    )));
}

#[test]
fn limits_are_respected() {
    let pages: Vec<_> = (0..20)
        .map(|i| {
            page(
                &format!("https://site{i}.example/"),
                &format!("Example {i}"),
                i + 1,
                0,
                1,
            )
        })
        .collect();
    let remote: Vec<String> = (0..10).map(|i| format!("example {i}")).collect();
    let rows = rows(
        "example",
        None,
        &pages,
        &[],
        &remote,
        NOW,
        Limits {
            remote: 2,
            pages: 3,
        },
    );
    assert_eq!(
        rows.iter()
            .filter(|r| matches!(
                r,
                Row::Search {
                    suggested: true,
                    ..
                }
            ))
            .count(),
        2
    );
    assert_eq!(
        rows.iter()
            .filter(|r| matches!(r, Row::Page { .. }))
            .count(),
        3
    );
    // Best first: the most visited sites lead.
    assert!(matches!(&rows[3], Row::Page { url, .. } if url == "https://site19.example/"));
}

#[test]
fn empty_input_has_no_rows() {
    assert!(rows("  ", None, &history(), &[], &[], NOW, Limits::default()).is_empty());
}

#[test]
fn top_sites_group_by_site_and_skip_bookmarks() {
    let mut pages = history();
    pages.push(page(
        "https://github.com/explore",
        "Explore GitHub",
        1,
        0,
        0,
    ));
    pages[3].bookmarked = true; // wikipedia.org: shown under Favorites instead.
    let top = top_sites(&pages, NOW, 4);
    let urls: Vec<_> = top.iter().map(|c| c.url.as_str()).collect();
    assert_eq!(urls[0], "https://news.ycombinator.com/");
    assert_eq!(urls[1], "https://github.com/");
    assert!(!urls.iter().any(|u| u.contains("www.wikipedia.org")));
    assert_eq!(urls.iter().filter(|u| u.contains("github.com")).count(), 1);
    assert_eq!(top.len(), 4);
}

#[test]
fn top_sites_without_a_visited_front_page_use_the_best_page() {
    let pages = vec![page("https://docs.rs/serde/", "serde", 3, 0, 0)];
    assert_eq!(top_sites(&pages, NOW, 8)[0].url, "https://docs.rs/serde/");
}

#[test]
fn arrow_keys_step_through_the_rows_and_off_either_end() {
    assert_eq!(step_highlight(3, None, true), Some(0));
    assert_eq!(step_highlight(3, None, false), Some(2));
    assert_eq!(step_highlight(3, Some(0), true), Some(1));
    assert_eq!(step_highlight(3, Some(2), true), None);
    assert_eq!(step_highlight(3, Some(0), false), None);
    assert_eq!(step_highlight(3, Some(2), false), Some(1));
    assert_eq!(step_highlight(0, None, true), None);
    // A stale highlight beyond a shorter list leaves it.
    assert_eq!(step_highlight(2, Some(5), true), None);
    assert_eq!(step_highlight(2, Some(5), false), None);
}
