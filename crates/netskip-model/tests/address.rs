// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
use netskip_model::address::*;

fn nav(text: &str) -> String {
    match classify(text) {
        Some(Input::Navigate(u)) => u,
        other => panic!("{text:?} should navigate, got {other:?}"),
    }
}

fn search(text: &str) -> String {
    match classify(text) {
        Some(Input::Search(t)) => t,
        other => panic!("{text:?} should search, got {other:?}"),
    }
}

#[test]
fn empty_input_does_nothing() {
    assert_eq!(classify(""), None);
    assert_eq!(classify("   \t "), None);
    assert_eq!(classify("?"), None);
    assert_eq!(classify("?   "), None);
}

#[test]
fn explicit_schemes_navigate_as_typed() {
    assert_eq!(nav("https://example.com"), "https://example.com/");
    assert_eq!(
        nav("http://example.com/a?b=c#d"),
        "http://example.com/a?b=c#d"
    );
    assert_eq!(nav("HTTPS://Example.COM/Path"), "https://example.com/Path");
    assert_eq!(nav("about:blank"), "about:blank");
    assert_eq!(nav("file:///etc/hosts"), "file:///etc/hosts");
    assert_eq!(nav("  https://example.com/  "), "https://example.com/");
}

#[test]
fn system_schemes_navigate_and_are_flagged() {
    let mail = nav("mailto:someone@example.com");
    assert!(is_system_url(&mail));
    assert!(is_system_url(&nav("tel:+15551234567")));
    assert!(!is_system_url("https://example.com/"));
}

#[test]
fn unknown_and_script_schemes_search() {
    assert_eq!(search("javascript:alert(1)"), "javascript:alert(1)");
    assert_eq!(search("foo:bar"), "foo:bar");
    assert_eq!(search("note: buy milk"), "note: buy milk");
}

#[test]
fn http_without_a_host_searches() {
    assert_eq!(search("http://"), "http://");
    assert_eq!(search("https:"), "https:");
}

#[test]
fn bare_hosts_get_https() {
    assert_eq!(nav("example.com"), "https://example.com/");
    assert_eq!(nav("www.example.com"), "https://www.example.com/");
    assert_eq!(
        nav("example.com/news?id=4#top"),
        "https://example.com/news?id=4#top"
    );
    assert_eq!(
        nav("sub.domain.example.co.uk/x"),
        "https://sub.domain.example.co.uk/x"
    );
    assert_eq!(nav("kagi.com"), "https://kagi.com/");
    assert_eq!(nav("example.io"), "https://example.io/");
    assert_eq!(nav("example.dev"), "https://example.dev/");
    assert_eq!(nav("Example.Com"), "https://example.com/");
    assert_eq!(
        nav("example.com:8443/admin"),
        "https://example.com:8443/admin"
    );
    assert_eq!(
        nav("user:pass@example.com/inbox"),
        "https://user:pass@example.com/inbox"
    );
}

#[test]
fn email_addresses_and_bare_users_search() {
    // DuckDuckGo's classifier cases: a user without a password is an email address.
    assert_eq!(search("user@domain.com"), "user@domain.com");
    assert_eq!(search("user@localhost"), "user@localhost");
    assert_eq!(search("user: @domain.com"), "user: @domain.com");
    assert_eq!(search(":pass@domain.com"), ":pass@domain.com");
}

#[test]
fn duckduckgo_classifier_cases() {
    assert_eq!(
        nav("regular-domain.com/path/to/file "),
        "https://regular-domain.com/path/to/file"
    );
    assert_eq!(search("local "), "local");
    assert_eq!(search("localdomain"), "localdomain");
    assert_eq!(search("duckduckgo"), "duckduckgo");
    assert_eq!(search("1.2"), "1.2");
    assert_eq!(search("1.4/3.4"), "1.4/3.4");
    assert_eq!(search("16385-12228.72"), "16385-12228.72");
    assert_eq!(search("1+(3+4*2)"), "1+(3+4*2)");
    assert_eq!(search("define: foo"), "define: foo");
    assert_eq!(search(" duck duck go.c "), "duck duck go.c");
    assert_eq!(search("test://hello/"), "test://hello/");
    assert_eq!(search("http:// 💩.la:8080"), "http:// 💩.la:8080");
    assert_eq!(search("example.com/pa th"), "example.com/pa th");
    assert_eq!(nav("http://💩.la:8080 "), "http://xn--ls8h.la:8080/");
    assert_eq!(nav("http:/duckduckgo.com"), "http://duckduckgo.com/");
    assert_eq!(nav("file:/Users/x"), "file:///Users/x");
    assert_eq!(
        nav("https://duckduckgo.com/?q=a b&arg 2=v 2"),
        "https://duckduckgo.com/?q=a%20b&arg%202=v%202"
    );
    assert_eq!(nav("https://example.com/a b"), "https://example.com/a%20b");
    assert_eq!(nav("https://例子.测试"), "https://xn--fsqu00a.xn--0zwm56d/");
}

#[test]
fn internationalized_hosts_navigate() {
    assert_eq!(nav("münchen.de"), "https://xn--mnchen-3ya.de/");
    assert_eq!(nav("example.рф"), "https://example.xn--p1ai/");
}

#[test]
fn file_like_words_search() {
    assert_eq!(search("node.js"), "node.js");
    assert_eq!(search("notes.txt"), "notes.txt");
    assert_eq!(search("3.14"), "3.14");
    assert_eq!(search("1.2.3"), "1.2.3");
}

#[test]
fn single_words_search() {
    assert_eq!(search("github"), "github");
    assert_eq!(search("rust"), "rust");
    assert_eq!(search("C++"), "C++");
}

#[test]
fn spaces_search() {
    assert_eq!(search("rust borrow checker"), "rust borrow checker");
    assert_eq!(search("example.com is down"), "example.com is down");
    assert_eq!(
        search("https://example.com and more"),
        "https://example.com and more"
    );
}

#[test]
fn question_mark_forces_search() {
    assert_eq!(search("?example.com"), "example.com");
    assert_eq!(search("? why is the sky blue"), "why is the sky blue");
}

#[test]
fn local_addresses_use_http() {
    assert_eq!(nav("localhost"), "http://localhost/");
    assert_eq!(nav("localhost:3000"), "http://localhost:3000/");
    assert_eq!(
        nav("localhost:8080/api?x=1"),
        "http://localhost:8080/api?x=1"
    );
    assert_eq!(nav("app.localhost"), "http://app.localhost/");
    assert_eq!(nav("127.0.0.1"), "http://127.0.0.1/");
    assert_eq!(
        nav("192.168.1.10:8000/status"),
        "http://192.168.1.10:8000/status"
    );
    assert_eq!(nav("[::1]"), "http://[::1]/");
    assert_eq!(nav("[::1]:8080/x"), "http://[::1]:8080/x");
    assert_eq!(nav("devbox:8080"), "http://devbox:8080/");
    assert_eq!(nav("intranet/"), "http://intranet/");
    assert_eq!(nav("intranet/wiki"), "http://intranet/wiki");
}

#[test]
fn bad_ports_and_hosts_search() {
    assert_eq!(search("example.com:99999"), "example.com:99999");
    assert_eq!(search("-bad-.com"), "-bad-.com");
    assert_eq!(search("a..com"), "a..com");
    assert_eq!(search("256.1.1.1"), "256.1.1.1");
}

#[test]
fn https_upgrade_skips_local_and_ported_hosts() {
    assert_eq!(
        upgrade_to_https("http://example.com/a"),
        "https://example.com/a"
    );
    assert_eq!(
        upgrade_to_https("https://example.com/a"),
        "https://example.com/a"
    );
    assert_eq!(
        upgrade_to_https("http://localhost:3000/"),
        "http://localhost:3000/"
    );
    assert_eq!(upgrade_to_https("http://127.0.0.1/"), "http://127.0.0.1/");
    assert_eq!(upgrade_to_https("http://[::1]/"), "http://[::1]/");
    assert_eq!(upgrade_to_https("http://intranet/"), "http://intranet/");
    assert_eq!(
        upgrade_to_https("http://printer.local/"),
        "http://printer.local/"
    );
    assert_eq!(
        upgrade_to_https("http://example.com:8080/"),
        "http://example.com:8080/"
    );
    assert_eq!(upgrade_to_https("ftp://example.com/"), "ftp://example.com/");
    assert_eq!(upgrade_to_https("not a url"), "not a url");
}

#[test]
fn local_host_detection() {
    for host in [
        "localhost",
        "dev.localhost",
        "nas.local",
        "intranet",
        "10.0.0.1",
        "[::1]",
    ] {
        assert!(is_local_host(host), "{host}");
    }
    for host in ["example.com", "www.example.org", "10.0.0.example.com"] {
        assert!(!is_local_host(host), "{host}");
    }
}

#[test]
fn display_shows_the_host_without_www() {
    assert_eq!(
        display("https://www.example.com/a/b?c", false),
        "example.com"
    );
    assert_eq!(display("http://example.com/", false), "example.com");
    assert_eq!(display("http://localhost:3000/x", false), "localhost:3000");
    assert_eq!(display("about:blank", false), "about:blank");
    assert_eq!(display("not a url", false), "not a url");
    assert_eq!(display("https://xn--mnchen-3ya.de/", false), "münchen.de");
}

#[test]
fn display_full_drops_only_a_bare_trailing_slash() {
    assert_eq!(display("https://example.com/", true), "https://example.com");
    assert_eq!(
        display("https://example.com/a/", true),
        "https://example.com/a/"
    );
    assert_eq!(
        display("https://example.com/?q=1", true),
        "https://example.com/?q=1"
    );
}

#[test]
fn editable_decodes_readable_characters() {
    assert_eq!(
        editable("https://example.com/caf%C3%A9"),
        "https://example.com/café"
    );
    assert_eq!(
        editable("https://example.com/a%20b"),
        "https://example.com/a%20b"
    );
    assert_eq!(editable("about:blank"), "");
    assert_eq!(editable("mailto:a@b.c"), "mailto:a@b.c");
}

#[test]
fn security_follows_the_scheme() {
    assert!(is_secure("https://example.com/"));
    assert!(!is_secure("http://example.com/"));
    assert!(!is_secure("garbage"));
}

#[test]
fn site_and_history_keys() {
    assert_eq!(
        site("https://WWW.Example.com/a").as_deref(),
        Some("example.com")
    );
    assert_eq!(site("about:blank"), None);
    assert_eq!(
        history_key("https://Example.com:443/a#section").as_deref(),
        Some("https://example.com/a")
    );
    assert_eq!(
        history_key("https://example.com").as_deref(),
        Some("https://example.com/")
    );
    assert_eq!(history_key("about:blank"), None);
    assert_eq!(history_key("data:text/plain,hi"), None);
    assert_eq!(history_key("mailto:a@b.c"), None);
}

#[test]
fn strip_scheme_matches_what_people_type() {
    assert_eq!(
        strip_scheme("https://www.example.com/a?b"),
        "example.com/a?b"
    );
    assert_eq!(strip_scheme("http://example.com/"), "example.com/");
    assert_eq!(strip_scheme("example.com"), "example.com");
}

#[test]
fn home_page_setting() {
    assert_eq!(
        home_page("example.com").as_deref(),
        Some("https://example.com/")
    );
    assert_eq!(
        home_page("https://a.org/start").as_deref(),
        Some("https://a.org/start")
    );
    assert_eq!(home_page(""), None);
    assert_eq!(home_page("not an address"), None);
}

#[test]
fn monograms_are_stable() {
    assert_eq!(monogram("https://www.github.com/"), "G");
    assert_eq!(monogram("https://ñandu.example/"), "Ñ");
    assert_eq!(monogram("about:blank"), "•");
    assert_eq!(
        monogram_hue("https://github.com/a"),
        monogram_hue("https://www.github.com/b")
    );
    assert!(monogram_hue("https://example.com/") < 360);
}

/// RFC 2606's reserved names are sites, not searches: documentation and fixture pages use them.
#[test]
fn reserved_names_are_addresses() {
    assert_eq!(nav("news.example"), "https://news.example/");
    assert_eq!(
        nav("kitchen.example/recipes"),
        "https://kitchen.example/recipes"
    );
    assert_eq!(nav("api.test"), "https://api.test/");
    // A made-up name is still a search.
    assert_eq!(search("kitchen.notatld"), "kitchen.notatld");
}
