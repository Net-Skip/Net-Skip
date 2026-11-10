<!--
Copyright © The Daybrite Project
SPDX-License-Identifier: MPL-2.0
-->

# Changelog

## Unreleased

The first version.

- New: the list catalog (`catalog`, `SourceId`, `Purpose`): EasyList, EasyPrivacy, the ABP
  anti-circumvention list, Adblock Warning Removal List, Fanboy's Social and Annoyance lists,
  Easylist Cookie List, Peter Lowe's list in hosts format, and eight regional lists, each with its
  upstream URL and license. Lists added by URL with `Source::custom`.
- New: blocking levels (`Level`): `Off`, `Light`, `Strong`, `Aggressive` and `Custom`, with
  document blocking and same-site blocking; per-site levels (`SiteLevels`); one serde `Settings`
  value for the app to store.
- New: the updater: conditional downloads through day-part-http, the `! Expires:` schedule clamped
  to 1 hour through 14 days with jitter, backoff from 15 minutes to 12 hours, mirrors, the last
  good copy kept on failure, and `state.json` in the app's cache directory.
- New: the compiler: one adblock-rust engine per list, built on the blocking pool and cached as
  `engine-<level>-<hash>.bin`, so a blocked request names its list.
- New: `AdBlock::decide`, `cosmetic`, `generic_selectors`, and the document-start element hiding
  script with its message handler (`cosmetic_script`, `handle_script_message`).
- New: the WebKit content rule export (`content_rules`), split under WebKit's 150,000-rule limit
  and versioned so a host recompiles only when it changes.
- New: blocked-request statistics per page host and purpose.
- New: `AdBlock::stage` downloads and compiles new settings while the current ones keep
  blocking, `content_rules_for` exports a staged level for WebKit, and `check_list` checks a
  list added by address before it is used. `RuleCounts::unsupported` counts the rules the blocker
  cannot apply.
