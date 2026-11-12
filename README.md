<!--
Copyright © The Daybrite Project
SPDX-License-Identifier: MPL-2.0
-->

# Net Skip

Net Skip is a web browser for phones, tablets and computers. Each tab is the platform's own web
engine (WebKit on Apple devices, Android System WebView, WebView2 on Windows, WebKitGTK and
Qt WebEngine on Linux, ArkWeb on HarmonyOS), and the browser around it is built with
[Day](https://daybrite.dev) from each platform's native controls.

<div align="center">
  <a href="https://play.google.com/store/apps/details?id=org.appfair.app.Net_Skip"><img src="https://appfair.org/assets/badges/google-play-store.svg" alt="Get it on Google Play" height="60"></a>
  <a href="https://apps.apple.com/app/id1640618584"><img src="https://appfair.org/assets/badges/apple-app-store.svg" alt="Download on the App Store" height="60"></a>
</div>

## Features

- Tabs, with a tab strip on computers and tablets and a tab grid on phones. Links open in new
  tabs beside the page they came from, and Reopen Closed Tab brings back the last 25.
- An address bar that completes sites from your history as you type, the way Safari's does, with
  suggestions from your bookmarks, history, open tabs and your search engine.
- Search with Kagi by default, or DuckDuckGo, Google, Bing, Ecosia, Qwant, Startpage,
  Swisscows, Yahoo!, SearXNG, Yandex, Baidu, Dogpile, Gigablast, or any engine whose search
  address you enter.
- Bookmarks, shown as Favorites on the start page, and a searchable history grouped by day.
- Ad and tracker blocking, Off, Light, Strong or Aggressive for every site or just one. The
  filter lists come from their publishers and update on their own; Settings > Filter Lists
  switches single lists on and off and adds lists by address.
- Private tabs, find in page, page zoom, and the tabs you had open at your next launch.

## Building

Install the [Day CLI](https://daybrite.dev/docs/tutorials/first-app), then:

```sh
day launch -t macos-appkit      # or ios-uikit, android-material, linux-gtk, windows-winui …
```

## Testing

The browser's rules (address classification, search engines, completion ranking, tab
placement and history grouping) live in `crates/netskip-model` with their unit tests:

```sh
cargo test --workspace
```

The dayscripts under `dayscript/` drive the app itself. They run against the pages in
`tests/fixtures/web/` rather than the network: `NETSKIP_FIXTURES=1` serves every address from
those files, generates a results page for any search, and answers search suggestion and favicon
requests from them.

```sh
day launch -t macos-appkit --env NETSKIP_FIXTURES=1 --script dayscript/browse.yaml
```

Downloads over real HTTP, HTTP sign-in and certificate errors need a server, which the fixture
pages cannot stand in for. `tests/server/duties_server.py` is one for the local machine, and
`tests/server/server.yaml` drives the app against it:

```sh
python3 tests/server/duties_server.py &
day launch -t ios-uikit --env NETSKIP_FIXTURES=1 --script tests/server/server.yaml
```

## License

Net Skip is free software under the [Mozilla Public License 2.0](LICENSE). The interface icons
under `resource/vectors/` other than `app_mark.svg` are
[Material Symbols](https://github.com/google/material-design-icons) by Google, under the
[Apache License 2.0](LICENSES/Apache-2.0.txt).

Net Skip is published by [The App Fair Project](https://appfair.org).
# Net-Skip
