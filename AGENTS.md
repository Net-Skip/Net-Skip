# Working on this Day app

This app uses native widgets through Day. It is Net Skip, a web browser: tabs of native web views, an address bar
with completion from history, bookmarks, history, and a choice of search engine.

## Day documentation

Start with [For AI Agents](https://daybrite.dev/docs/reference/for-agents), the concise reference for
Day's conventions, reactive patterns, APIs, and development workflow.

- [Machine-readable documentation index](https://daybrite.dev/llms.txt): discover the relevant
  guides and detailed references before implementing a feature.
- [API tour](https://daybrite.dev/docs/reference/api-tour): examples of pieces, state, layout, navigation,
  localization, and accessibility.
- [Testing with dayscript](https://daybrite.dev/docs/how-to/dayscript): drive the app, assert behavior,
  and capture screenshots.

Use examples with the API supported by this project's Day dependency version. Check
`day <command> --help` for the installed CLI's options.

## Project map

- `src/lib.rs`: the root, the menu bar and its shortcuts, and the history date formatting.
- `src/browser.rs`: a window's tabs (`Tab`, `Browser`), loading, history recording, the saved
  session, and the address bar's editing state (`Omnibox`: inline completion, suggestions,
  arrow keys, Escape, submit).
- `src/chrome.rs`: the toolbar and tab strip (computer, tablet), the bottom bar (phone), the
  address field, the suggestion list, find in page, and the tab grid.
- `src/library.rs`: the start page (Favorites, Frequently Visited), Bookmarks and History.
- `src/downloads.rs`: asking before a file is saved, the Downloads list and its storage.
- `src/store.rs`: history and bookmarks in SQLite through day-persistence, with tests.
- `src/settings.rs`: the preferences and the Settings groups.
- `src/fixtures.rs`: fixture mode (`NETSKIP_FIXTURES=1`), which serves `tests/fixtures/web/`
  instead of the network.
- `src/native.rs`: per-platform tweaks (the address field's frameless look).
- `crates/netskip-model/`: the UI-free rules (address classification, search engines,
  completion ranking, tab placement, history day groups, fixture address mapping) and their
  tests: `cargo test -p netskip-model`.
- `dayscript/`: the walkthroughs; every script runs with `NETSKIP_FIXTURES=1`.
- `tests/server/`: a local server and a dayscript for what needs real HTTP (downloads,
  sign-in, a self-signed certificate). CI does not run it.
- `resource/locales/<locale>/app.ftl`: app-owned UI strings.
- `Day.toml`, `Day-appfair.toml`: app identity (the existing store ids), targets, window.

## Conventions

- Add new user-facing strings, including errors and accessibility labels, to
  `resource/locales/<locale>/app.ftl` in every app locale. Reference them through auto-generated
  symbolic accessors such as `res::str::field_name()`, never raw unlocalized strings in UI code.
  Format messages on the UI thread when doing asynchronous work.
- Reference bundled resources through generated `res::images`, `res::vectors`, `res::fonts`,
  and `res::assets` constants. Mark monochrome UI glyphs with `.template()` so they follow the theme.
- Add accessibility information where native semantics are insufficient: localized `.a11y`
  labels for controls and meaningful images, heading roles for headings, and hints when an
  action needs explanation. Mark purely decorative images with `.decorative()`. Preserve native
  control roles and values; a scripting ID does not replace an accessible label.
- Give interactive pieces stable `.id(day::id!(snake_case_id))` names for scripting and
  accessibility.
- Use the `Space` scale (`Space::S`, `Space::L`, …) for gaps and padding, and semantic fonts
  (`Font::Headline`) for text, rather than point values.
- Keep shared state in `Scene`, accessed through `Scene::ambient()`. Use reactive bindings for
  controls and untracked reads in action callbacks when no subscription is intended.
- Prefer semantic pieces such as `form`, `section`, and `labeled` over custom layout.
- Add new pages to `src/pages.rs`, their typed routes and navigation to `src/lib.rs`, and
  portable steps to `dayscript/demo.yaml`, asserting text through localization keys.
- Keep test fixtures in the repository. Synthetic test text is separate from shipped UI text.

## Validation

Run `cargo fmt --all`, `cargo fmt --all -- --check`, `cargo test`, and
`cargo clippy --all-targets -- -D warnings`. Run `day lint` and build the affected targets.
Run the walkthrough with `day launch -t <target> --script dayscript/demo.yaml` and inspect the
screenshots of changed screens. Use `day drive -t <target> --steps-json '[…]'` to inspect a
running app. Available Day MCP tools provide equivalent launch and drive operations.

## Dependency versions

Before Day 1.0, dependencies on Day and other Daybrite-owned 0.x crates use `"0.*"`.
Keep package release versions concrete and third-party requirements separate. Commit Cargo.lock
and use --locked when the app needs a fixed dependency set. Updating the lock may require code
changes; compatibility between Day 0.x releases is not guaranteed.
