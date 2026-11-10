<!--
Copyright © The Daybrite Project
SPDX-License-Identifier: MPL-2.0
-->

# Ad and tracker blocking guide

`netskip-adblock` decides which web requests to block and which page elements to hide. This guide
covers the blocker's lifecycle, levels and per-site settings, how a web view host connects each
engine to it, the element hiding script and its messages, the WebKit export, statistics, and tests.

## Lifecycle

```rust,no_run
use netskip_adblock::{AdBlock, Settings, Status};

let stored: Settings = Settings::default(); // or what the app saved last time
let adblock = AdBlock::builder().settings(stored).build()?;
adblock.on_change({
    let adblock = adblock.clone();
    move || {
        // Runs on whichever thread made the change; send it to the UI with a reactive setter.
        let _ = adblock.status();
    }
});
adblock.start();
# Ok::<(), netskip_adblock::Error>(())
```

`build()` reads `state.json` and compiles nothing. `start()` runs one background cycle: it loads
each engine in use from its cache file (or compiles it on the blocking pool), downloads the lists
that are due, and arms a timer for the next due list. `stop()` ends the schedule.

The same steps are available as futures for code that wants to await them, for example under
`day::task` or in a test: `prepare()`, `update_due()` and `update_now()`. None of them blocks the
calling thread; file I/O and compiling run on `day_async::spawn_blocking`'s pool.

`status()` reports on the global level:

| Status | Meaning |
| --- | --- |
| `Off` | The level is `Off`. |
| `WaitingForLists` | No list of the level has been downloaded yet. Nothing is blocked. |
| `Loading` | Lists are stored and the engine is being loaded or compiled. |
| `Ready { lists, missing }` | Blocking with `lists` lists; `missing` names lists still to download. |
| `Failed { error }` | The last compile failed and no engine is loaded. |

## Levels and settings

`Settings` is one serde value the app stores where it keeps its preferences:

```json
{
  "level": { "level": "strong" },
  "extra_sources": ["easylist-germany"],
  "custom_sources": [],
  "sites": { "news.example": { "level": "off" } }
}
```

- `level` applies to every site without its own entry.
- `extra_sources` joins every level except `Off`: regional lists (`regional_sources("de")` finds
  them) and lists added by URL.
- `custom_sources` defines lists added by URL. `Source::custom(url)` gives a stable id,
  `custom-<hash>`, and `with_title`, `with_purpose` and `with_format` describe the list.
- `sites` maps a host to its level. An entry covers subdomains, and `www.` is ignored, so
  `news.example` also covers `www.news.example` and `m.news.example`.

`set_level`, `set_site_level` and `set_settings` apply at once to queries. On a started blocker
they also start loading the engines and lists the new settings need. Each level in use gets its
own engine; a site set to `Off` needs none.

A level whose engine is not built yet blocks nothing until it is. To change settings without that
gap, stage them first:

1. `stage(settings)` downloads the lists the new settings use that have no stored copy, and
   compiles each level's engine, while the current settings keep blocking. Its `StageReport`
   says how each download ended and which lists are still `missing`. A staged engine survives
   the background cycle until it is applied.
2. On WebKit, `content_rules_for(&settings, &level)` exports a staged level's rule lists, so a
   view can move from the old rules straight to the new ones.
3. `set_settings(settings)` applies them. With the engines already built, the change is
   immediate.

A host decides what an incomplete report means. Net Skip refuses a change when a list it newly
switches on could not be downloaded, and keeps the previous settings.

`check_list(url)` checks a list before it is added: the address must use `https`, and the
answer must be a filter list (not a web page or an empty file). The copy is stored, so staging
settings that use it needs no second download. The `ListCheck` carries the list's own title
(from its `! Title:` header) and its `RuleCounts`, including `unsupported`: rules this blocker
counts but cannot apply (scriptlets and other script injection, CSS injection, HTML filtering,
resource redirects and response rewriting). The count comes from each rule's syntax and is
approximate.

## Connecting a web view

### Engines with a request callback

Android's `shouldInterceptRequest`, WebView2's `WebResourceRequested`, Qt's
`QWebEngineUrlRequestInterceptor` and ArkWeb's scheme handler each see every request. Build a
`RequestInfo` and call `decide`; it is safe on any thread and takes a few microseconds.

```rust,no_run
# use netskip_adblock::{AdBlock, RequestInfo, ResourceKind};
# fn host(adblock: &AdBlock, url: &str, page: &str, is_main_frame: bool) -> bool {
let request = RequestInfo {
    url,
    source_url: page,          // the top-level page, or the frame's URL where the engine has it
    kind: ResourceKind::Unknown, // Android gives no type; the URL's extension decides
    main_frame: is_main_frame,
};
adblock.decide(&request).block
# }
```

| Engine | Thread | Type | Blocked answer |
| --- | --- | --- | --- |
| Android System WebView | background | none: pass `Unknown` | a `WebResourceResponse` with status 403 and an empty body |
| WebView2 | UI | `ResourceContext`: `Document`, `Script`, `Image`, `Fetch`, ... | a 403 response from `CreateWebResourceResponse` |
| QtWebEngine | UI | `resourceType()` | `info.block(true)` |
| ArkWeb scheme handler | IO | the request's resource type | fail the request |

For a top-level navigation use `RequestInfo::navigation(url)`. It is blocked only when the level
has document blocking on (`Aggressive`), and the host then shows its own interstitial page.

`Decision::matched_list` names the list that blocked the request, for a host that wants to show
it. `Decision::redirect` is always `None` in this release.

### WebKit (iOS, macOS, WebKitGTK)

WebKit has no per-request callback. It matches compiled JSON rule lists inside the engine.

```rust,no_run
# async fn compile(adblock: netskip_adblock::AdBlock) -> Result<(), netskip_adblock::Error> {
let rules = adblock.content_rules(&adblock.level()).await?;
for list in &rules.lists {
    let json = list.read_json().map_err(|e| netskip_adblock::Error::Io(e.to_string()))?;
    // WKContentRuleListStore.compileContentRuleList(forIdentifier: list.identifier,
    //                                               encodedContentRuleList: json)
    // then WKUserContentController.add(_:) for each compiled list.
    let _ = (&list.identifier, json);
}
// Store rules.version; on the next launch, compile again only when it differs and
// otherwise look the lists up by identifier.
# Ok(()) }
```

- Each list holds at most 150,000 rules, WebKit's limit (`webkit_rule_limit` lowers it). An
  exception rule only cancels earlier rules in the same list, so every list ends with all the
  exception rules.
- With same-site blocking off, each list ends with a rule that exempts first-party loads.
- The lists carry site-specific element hiding only. Generic hiding (a selector for every site,
  tens of thousands of them in EasyList) is left to the cosmetic script: as WebKit rules it becomes
  one style sheet on every page, which stalls heavy pages (dailymail.co.uk and weather.com never
  finished loading at Strong). Procedural selectors (`:has-text()` and the like) are left out
  too: they are not CSS. So a WebKit host runs the cosmetic script as well.
- Identifiers are `day-adblock-<level>-<n>`; the export is cached under
  `webkit-<level>-<version>/` in the cache directory.
- WebKitGTK reads the same JSON through `WebKitUserContentFilterStore`.
- WebKit reports no blocked loads to the app, so the statistics stay at zero on these engines.

EasyList and EasyPrivacy export to 129,420 rules in one list. `WKContentRuleListStore` compiles it
in about 4 seconds on a Mac; compile once per version and keep the store.

## The cosmetic script

`cosmetic_script()` returns JavaScript to inject at document start into every frame
(`WKUserScript` with `forMainFrameOnly: false`, Android's `addDocumentStartJavaScript`,
WebView2's `AddScriptToExecuteOnDocumentCreatedAsync`, a Qt `QWebEngineScript` at
`DocumentCreation` on sub-frames). It hides elements with a `<style>` element and watches the DOM
for new classes and ids.

The script posts JSON strings on the channel `SCRIPT_CHANNEL` (`"dayAdblock"`). It uses the first
transport it finds:

1. `window.webkit.messageHandlers.dayAdblock.postMessage(text)` (WebKit)
2. `window.dayAdblock.postMessage(text)` (an Android web message listener named `dayAdblock`)
3. `window.chrome.webview.postMessage(text)` (WebView2)
4. `window.__dayAdblockSend(text)`, a function the host defines (Qt, ArkWeb, others)

Messages:

```json
{"v": 1, "type": "page", "url": "https://news.example/story"}
{"v": 1, "type": "ids", "url": "https://news.example/story", "classes": ["ad-banner"], "ids": ["top"]}
```

Pass each one to `handle_script_message`. It returns `Some(json)`, the answer to send back as the
message's reply, or `None` when there is nothing to hide:

```json
{"hide": [".sponsored"], "generichide": false}
```

With day-piece-webview's `on_script_message("dayAdblock", …)`, return it as the handler's reply:
the script sends through `window.dayMessage` and applies the reply itself. A host without replies
can evaluate `window.__dayAdblock.apply(<json>)` in the frame that sent the message.

The `page` answer carries the site's own selectors and the generic selectors that are not plain
class or id selectors. The `ids` answers carry generic class and id selectors as they appear.
A page with a `$generichide` exception gets `"generichide": true` and the script stops reporting.
The script batches reports every 50 ms, at most 500 names per message.

Hosts that cannot inject a script can call `cosmetic(url)` and `generic_selectors(...)` directly.

## Statistics

`decide` counts each blocked request under the page's host and the blocking list's purpose:

```rust,no_run
# use netskip_adblock::{AdBlock, Purpose};
# fn shield(adblock: &AdBlock) {
let counts = adblock.blocked("news.example");
let label = format!("{} blocked, {} trackers", counts.total, counts.get(Purpose::Privacy));
adblock.reset_stats(Some("news.example")); // when the page reloads, if the app counts per load
# let _ = label; }
```

`blocked_by_host()` and `blocked_total()` give the totals. The counts live in memory; an app that
wants them across launches stores them itself.

## Tests

Install a `day_part_http::simulation::Simulation` on a session, give the blocker a client on that
session, and give it a clock you can move:

```rust,no_run
use std::sync::Arc;
use netskip_adblock::{AdBlock, Clock};
use day_part_http::simulation::{Reply, Simulation};
use day_part_http::{Client, Session};

struct Fixed;
impl Clock for Fixed {
    fn now(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_800_000_000)
    }
}

let simulation = Simulation::new();
simulation.route("GET", "https://lists.test/ads.txt", |_| {
    Ok(Reply::new(200, b"||ads.example^\n".to_vec()).header("ETag", "\"1\""))
});
let session = Session::new();
session.set_simulation(Some(simulation));
let adblock = AdBlock::builder()
    .cache_dir(std::env::temp_dir().join("adblock-test"))
    .client(Client::builder().session(session).build())
    .clock(Arc::new(Fixed))
    .build()?;
# Ok::<(), netskip_adblock::Error>(())
```

The crate's own tests use small hand-written lists in `tests/data/`. `tests/real_lists.rs`
measures the real lists when `DAY_ADBLOCK_LISTS` points at a folder of them, and
`tests/webkit_compile.rs` compiles the WebKit export with `WKContentRuleListStore` on macOS.

## What this release does not do

- Scriptlets and snippets (`##+js(...)`, `#$#`, `#%#`) are dropped, and `$redirect` rules block
  without a replacement body: both need resource libraries the crate does not ship.
- Procedural element hiding (`:has()`, `:has-text()`, `:-abp-contains()`) is not applied by the
  script; WebKit's export skips those rules as well.
- `$removeparam` and `$csp` rules have no effect.
- A list's `! Redirect:` header is ignored; the catalog URLs are the upstream ones.
- `$badfilter` cancels rules only within the list that contains it.
- On the web target there is no file system, so the blocker needs a cache directory it cannot get.
