<!--
Copyright © The Daybrite Project
SPDX-License-Identifier: MPL-2.0
-->

# Fixture web pages

Net Skip serves these pages instead of the network when it starts with `NETSKIP_FIXTURES=1`
(src/fixtures.rs). A file's path is its address: `example.com/news/index.html` answers
`https://example.com/news/`. Absolute links between fixture pages stay inside the fixtures, and
an address with no file gets a generated page titled with its host. Search engines' results
pages are generated from the typed terms.

The dayscripts rely on what these pages contain, so change them together:

- `example.com/news/` holds the word "harbor" exactly 5 times (find in page).
- `example.com/` links to `/news/`, `/about.html`, and opens `https://example.org/` in a new tab.
- Each site has a `favicon.png`, which the start page and the tab strip show.
- `files.example/` (*Dropbay*) has a button for each script dialog (`#alert`, `#confirm`,
  `#prompt`, answering into `#result`), a file served as an attachment
  (`download/tide-table.txt`, 115 bytes; anything under a `download/` directory is), a
  generated `blob:` download, a file input that reports into `#picked`, and links to `mailto:`
  and to an app scheme no app handles (`dropbay-app:`).

The pages and icons were written for these tests and carry no third-party content.

## Showcase sites

Five fuller sites give the walkthroughs and store screenshots something worth looking at. Each
one works on a phone, a tablet and a desktop, in light and dark mode, sets `theme-color`, and
has a `favicon.svg`, a 128 × 128 `favicon.png` and a 180 × 180 `apple-touch-icon.png`.

| Host | Site | Pages |
| --- | --- | --- |
| `harbor.example` | *Leeward*, a sailing and coastal travel magazine (deep blue) | `/`, the long feature `/stories/lighthouse-coast.html` (drop cap, pull quote, figures, reading progress bar, "Read next"), and five shorter stories in `/stories/` |
| `kitchen.example` | *Larder*, a recipe site (terracotta) | `/` and eight recipes in `/recipes/`; `/recipes/lemon-orzo.html` is the full one, with a servings stepper, ingredient checkboxes, timed steps and a nutrition table |
| `skies.example` | *Cirrus*, a weather site for Port Avelin | `/`: current conditions, a wind advisory, a 26-hour strip, a 10-day forecast, a rain radar map and detail tiles |
| `field.example` | *Fieldwork*, an illustrator's portfolio | `/`: a masonry gallery with filter chips and a lightbox; two of its works are JPEGs, the rest SVG |
| `news.example` | *The Daily Lantern*, an ad-heavy local news front page | `/`, for the ad and tracker blocking tests below |

Useful ids: `#page-title` on every home page (`#headline` on news.example), `#article-title`
on harbor stories, `#recipe-title`, `#recipe`, `#servings`, `#serv-plus` and `#nutrition` on
recipes, `#link-feature` for each home page's lead link, `#current-temp` and `#forecast` on
skies.example, and `#gallery`, `#work-count` and `#lightbox` on field.example.

Write absolute `https://host/...` addresses in HTML. Fixture mode loads pages from a
resource namespace, so a root-relative `/path` leaves the site. CSS and JavaScript files are
served as they are, without that rewriting, so they use relative addresses only.

## Blocking test markers (news.example)

The page loads first-party and third-party resources, and each one reports what happened:

- **`window.__loaded`**: the scripts that ran, by name.
- **`window.__events`**: `"<name>:load"` or `"<name>:error"` from each script, image and frame
  element's `onload` and `onerror`.
- **`window.__visibleAds()`** (defined by `app.js`): the ids of the ad placeholders still
  visible, so it returns `[]` when cosmetic filtering hides them all.
- **`document.documentElement.dataset.app == "ready"`** and `#app-status` reading "Live" once
  the first-party script has run.

| Name | Resource | Kind | Records |
| --- | --- | --- | --- |
| `app` | `https://news.example/assets/app.js` | first-party script, must always load | `__loaded`, `__events` |
| `ads-banner` | `https://ads.example/adserver/banner.js` | script; also fills the leaderboard's text and sets `data-filled-by="ads-banner"` on it | `__loaded`, `__events` |
| `tracker-collect` | `https://tracker.example/collect.js` | script | `__loaded`, `__events` |
| `analytics-ga` | `https://analytics.example/ga.js` | script | `__loaded`, `__events` |
| `ads-frame` | `https://ads.example/adserver/frame.html` in `iframe#google_ads_iframe_1` | frame; its script posts a message that the page records | `__loaded` (via `postMessage`), `__events` |
| `ads-pixel` | `https://ads.example/pixel.gif` in `img#ads-pixel` | 1 × 1 GIF | `__events` |
| `tracker-beacon` | `https://tracker.example/beacon?e=pageview&p=front` in `img#tracker-beacon` | 1 × 1 GIF with no extension | `__events` |

A frame hidden by a cosmetic rule still loads, and some engines fire `load` on a frame whose
request was blocked, so assert on `__loaded` containing `ads-frame` rather than on its
`__events` entry.

These elements look like obvious "AD" placeholders and take up space until a filter hides them.
Each carries `data-test-ad` (which `__visibleAds()` reads) and wraps its own label, so a hidden
slot leaves no stray text:

| Selector it matches | Element |
| --- | --- |
| `div[id^="div-gpt-ad"]` | `div#div-gpt-ad-1`, the leaderboard under the masthead |
| `.ad-banner` | `div#ad-banner-mid`, the wide banner in the main column |
| `.sponsored` | `article#sponsored-1`, the paid story in "Business & climate" |
| `ins.adsbygoogle` | `ins#ad-adsense`, the 300 × 250 slot in the sidebar |
| `iframe[id^="google_ads_iframe"]` | `iframe#google_ads_iframe_1` |
| `[data-ad-slot]` | `div#ad-slot-mpu`, inside the culture story |
| `#taboola-below-article` | the "You may like" grid under the culture story |
| `.OUTBRAIN` | `div#outbrain-widget`, "Recommended for you" |

The headline `h1#headline`, the stories and `app.js` sit outside all of these, so the page still
reads well with every ad hidden.

All names, places, stories, recipes, brands and advertisers on these sites are invented. The
illustrations, icons and the two JPEGs were drawn or generated for these fixtures and are
original work under the same license.

## Added filter list (lists.example)

`lists.example/harbor-extras.txt` is the list `dayscript/filter-lists.yaml` adds by address in
Settings > Filter Lists. Its `! Title:` is "Harbor Extras", and it holds one network rule, one
hiding rule and one scriptlet, so the added list reports 3 rules of which 1 can't be applied.
It was written for these tests.
