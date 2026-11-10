<!--
Copyright © The Daybrite Project
SPDX-License-Identifier: MPL-2.0
-->

# netskip-adblock

Net Skip's ad and tracker blocking. The crate keeps a catalog of public filter lists, turns a
blocking level into a set of lists, downloads them into the app's cache directory and keeps them
current, and answers two questions from any thread: should this request be blocked, and which
elements should this page hide.

It has no UI and no web view code. The app (`src/adblock.rs`) connects it to each engine: a request
callback on Android, WebView2, Qt and ArkWeb, compiled content rule lists on WebKit, and a
document-start script for element hiding everywhere.

## Use

It lives in the Net Skip workspace and is not published:

```toml
[dependencies]
netskip-adblock = { path = "crates/netskip-adblock" }
```

```rust
use netskip_adblock::{AdBlock, Error, Level, RequestInfo, ResourceKind};

fn setup() -> Result<AdBlock, Error> {
    let adblock = AdBlock::builder().build()?; // lists live in <cache dir>/adblock/
    adblock.set_level(Level::Strong);
    adblock.start(); // load the cached engine, then download what is due, in the background
    Ok(adblock)
}

// In a web view's request callback, on any thread:
fn should_block(adblock: &AdBlock, url: &str, page: &str) -> bool {
    adblock
        .decide(&RequestInfo::new(url, page, ResourceKind::Script))
        .block
}
```

`AdBlock` is a cloneable `Send + Sync` handle. A decision takes about 3.5 µs with EasyList,
EasyPrivacy, the anti-circumvention list and the cookie list loaded (release build, Apple M-series).

## Levels

| Level | Lists | Document blocking | Same-site blocking |
| --- | --- | --- | --- |
| `Off` | none | | |
| `Light` (default) | EasyPrivacy, Fanboy's Social | off | off |
| `Strong` | Light, EasyList, Adblock Warning Removal, ABP anti-circumvention | off | off |
| `Aggressive` | Strong, Peter Lowe's list, Easylist Cookie List | on | on |
| `Custom` | the app's choice | the app's choice | the app's choice |

Document blocking blocks a navigation to a blocked page, so the host can show an interstitial
instead. Same-site blocking lets rules block requests to the page's own site; with it off, a
rule only blocks third-party requests. The level names and descriptions are the app's to write and
translate; the crate exposes stable ids (`Level::id()`, serde `{"level": "strong"}`).

Regional lists and lists added by URL join every level except `Off` through
`Settings::extra_sources`. A site can have its own level (`AdBlock::set_site_level`), which also
covers its subdomains. The app stores `Settings` with serde; the crate keeps no settings file.

## Lists

Nothing is compiled into the binary. Each list is downloaded from its upstream address the first
time a level needs it.

| Id | List | Purpose | License | URL |
| --- | --- | --- | --- | --- |
| `easylist` | EasyList | ads | GPL-3.0 or CC BY-SA 3.0 | https://easylist-downloads.adblockplus.org/easylist.txt |
| `easyprivacy` | EasyPrivacy | privacy | GPL-3.0 or CC BY-SA 3.0 | https://easylist-downloads.adblockplus.org/easyprivacy.txt |
| `abp-anti-cv` | ABP Anti-Circumvention Filter List | ads | GPL-3.0 | https://easylist-downloads.adblockplus.org/abp-filters-anti-cv.txt |
| `adblock-warning-removal` | Adblock Warning Removal List | ads | GPL-3.0 or CC BY-SA 3.0 | https://easylist-downloads.adblockplus.org/antiadblockfilters.txt |
| `fanboy-social` | Fanboy's Social Blocking List | social | CC BY 3.0 | https://secure.fanboy.co.nz/fanboy-social.txt |
| `easylist-cookie` | Easylist Cookie List | cookies | CC BY 3.0 | https://secure.fanboy.co.nz/fanboy-cookiemonster.txt |
| `fanboy-annoyance` | Fanboy's Annoyance List | annoyances | CC BY 3.0 | https://secure.fanboy.co.nz/fanboy-annoyance.txt |
| `peter-lowe` | Peter Lowe's Ad and tracking server list (hosts format) | ads | none published | https://pgl.yoyo.org/adservers/serverlist.php?hostformat=hosts&showintro=1&mimetype=plaintext |
| `easylist-germany` | EasyList Germany | regional (de) | GPL-3.0 or CC BY-SA 3.0 | https://easylist-downloads.adblockplus.org/easylistgermany.txt |
| `easylist-dutch` | EasyList Dutch | regional (nl) | GPL-3.0 or CC BY-SA 3.0 | https://easylist-downloads.adblockplus.org/easylistdutch.txt |
| `easylist-spanish` | EasyList Spanish | regional (es) | GPL-3.0 or CC BY-SA 3.0 | https://easylist-downloads.adblockplus.org/easylistspanish.txt |
| `easylist-italy` | EasyList Italy | regional (it) | GPL-3.0 or CC BY-SA 3.0 | https://easylist-downloads.adblockplus.org/easylistitaly.txt |
| `easylist-portuguese` | EasyList Portuguese | regional (pt) | GPL-3.0 or CC BY-SA 3.0 | https://easylist-downloads.adblockplus.org/easylistportuguese.txt |
| `easylist-china` | EasyList China | regional (zh) | GPL-3.0 or CC BY-SA 3.0 | https://raw.githubusercontent.com/easylist/easylistchina/refs/heads/master/easylistchina.txt |
| `easylist-polish` | EasyList Polish | regional (pl) | GPL-3.0 or CC BY-SA 3.0 | https://easylist-downloads.adblockplus.org/easylistpolish.txt |
| `liste-fr` | Liste FR | regional (fr) | CC BY-SA 3.0 | https://easylist-downloads.adblockplus.org/liste_fr.txt |

EasyList and EasyPrivacy also list `https://easylist.to/easylist/` as a mirror, tried when the
first address fails. Peter Lowe's site publishes no license; review its terms at
https://pgl.yoyo.org/adservers/ before shipping the `Aggressive` level in a commercial app.
DuckDuckGo's Tracker Radar list is left out because its license (CC BY-NC-SA 4.0) does not allow
commercial use.

The crate drops scriptlet and snippet rules (`##+js(...)`, `#$#`, `#%#`) because running them needs
a scriptlet library in the page.

## Updates

- Each list is fetched with `If-None-Match` and `If-Modified-Since`, so an unchanged list costs a
  `304`.
- A list is due again after its `! Expires:` interval, clamped to 1 hour through 14 days (one day
  when it has none), plus up to a tenth of that interval as jitter.
- A failed download is retried after 15 minutes, doubling per failure up to 12 hours. The last good
  copy stays in use, and an HTML error page counts as a failure.
- `AdBlock::update_now()` fetches everything immediately; `start()` keeps the schedule running.
- Before the first list arrives, `status()` is `WaitingForLists` and nothing is blocked.

The cache directory (`<app cache>/adblock/`) holds `<list id>.txt`, `state.json` (ETag,
Last-Modified, fetch times, next fetch, rule counts, last result), `engine-<level>-<hash>.bin` (the
compiled engine, reloaded in about 3 ms on the next launch when its lists are unchanged) and
`webkit-<level>-<hash>/` (the WebKit export).

## Documentation

- [Guide: connecting a web view, element hiding, WebKit rules, statistics, tests](docs/adblock.md)
- [Changelog](CHANGELOG.md)

Generate the Rust API reference with `cargo doc --no-deps --open`.

## Dependencies

Besides Day's own crates (`day-part-http` for downloads, `day-async` for the blocking pool and the
timer, `day-spec` for the cache directory) and `serde`/`serde_json`, the crate uses one third-party
crate: [adblock-rust](https://github.com/brave/adblock-rust) 0.13 (MPL-2.0), the engine Brave ships.

- Why: filter syntax is large. About 85% of EasyList's network rules are plain `||host^`, but the
  rest (options, `$domain=`, exceptions, `$important`, regular expressions, element hiding with
  site exceptions, `$generichide`) and the WebKit converter would be 3,000 to 4,000 lines to write
  and keep current.
- Features: `embedded-domain-resolver` and `full-regex-handling`, plus `content-blocking` for the
  WebKit export. `single-thread` is off so one engine serves request callbacks on background
  threads; the crate checks at compile time that the engine is `Send + Sync`.
- Crates it brings that Day's tree did not have: `adblock`, `addr`, `regex`, `regex-automata`,
  `regex-syntax`, `aho-corasick`, `flatbuffers`, `itertools`, `seahash`.
- Size: a stripped release binary on macOS arm64 grows by 2.2 MB of code (`__text` from 1.52 MB to
  3.69 MB) and 3.1 MB on disk, measured against the same program with only the Day crates above.
- Measured with EasyList, EasyPrivacy, ABP anti-circumvention and the Easylist Cookie List
  (5 MB of lists): 121 ms to compile, a 7.5 MB engine cache file, 2.5 ms to reload it, 3.5 µs per
  decision. The WebKit export of EasyList and EasyPrivacy is 129,420 rules (13.9 MB of JSON),
  written in 230 ms; `WKContentRuleListStore` compiles it in 4.2 s on macOS.

## License

[MPL-2.0](LICENSE). The filter lists are not part of this crate and carry their own licenses,
listed above.
