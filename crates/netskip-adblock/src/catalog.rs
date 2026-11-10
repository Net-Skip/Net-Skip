// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! The filter list catalog, the blocking levels, and the settings an app persists.
//!
//! Every identifier here is stable: an app stores [`SourceId`]s and [`Level`]s in its
//! preferences and maps them to localized names itself. List titles are the lists' own names
//! and stay untranslated.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

// --- Sources ---

/// A filter list's stable identifier: `easylist`, `easyprivacy`, or `custom-<hash>` for a list
/// added by URL.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceId(Cow<'static, str>);

impl SourceId {
    /// EasyList, the main ad list.
    pub const EASYLIST: SourceId = SourceId::from_static("easylist");
    /// EasyPrivacy, trackers and analytics.
    pub const EASYPRIVACY: SourceId = SourceId::from_static("easyprivacy");
    /// Adblock Plus's anti-circumvention list.
    pub const ABP_ANTI_CV: SourceId = SourceId::from_static("abp-anti-cv");
    /// Adblock Warning Removal List.
    pub const ADBLOCK_WARNING_REMOVAL: SourceId = SourceId::from_static("adblock-warning-removal");
    /// Fanboy's Social Blocking List.
    pub const FANBOY_SOCIAL: SourceId = SourceId::from_static("fanboy-social");
    /// Easylist Cookie List (cookie banners).
    pub const EASYLIST_COOKIE: SourceId = SourceId::from_static("easylist-cookie");
    /// Fanboy's Annoyance List.
    pub const FANBOY_ANNOYANCE: SourceId = SourceId::from_static("fanboy-annoyance");
    /// Peter Lowe's ad and tracking server list, in hosts format.
    pub const PETER_LOWE: SourceId = SourceId::from_static("peter-lowe");
    /// EasyList Germany.
    pub const EASYLIST_GERMANY: SourceId = SourceId::from_static("easylist-germany");
    /// EasyList Dutch.
    pub const EASYLIST_DUTCH: SourceId = SourceId::from_static("easylist-dutch");
    /// EasyList Spanish.
    pub const EASYLIST_SPANISH: SourceId = SourceId::from_static("easylist-spanish");
    /// EasyList Italy.
    pub const EASYLIST_ITALY: SourceId = SourceId::from_static("easylist-italy");
    /// EasyList Portuguese.
    pub const EASYLIST_PORTUGUESE: SourceId = SourceId::from_static("easylist-portuguese");
    /// EasyList China.
    pub const EASYLIST_CHINA: SourceId = SourceId::from_static("easylist-china");
    /// EasyList Polish.
    pub const EASYLIST_POLISH: SourceId = SourceId::from_static("easylist-polish");
    /// Liste FR.
    pub const LISTE_FR: SourceId = SourceId::from_static("liste-fr");

    const fn from_static(id: &'static str) -> SourceId {
        SourceId(Cow::Borrowed(id))
    }

    /// An identifier from its stored string.
    pub fn new(id: impl Into<String>) -> SourceId {
        SourceId(Cow::Owned(id.into()))
    }

    /// The identifier of a custom list downloaded from `url`: `custom-` and a hash of the URL,
    /// so the same URL always gets the same id.
    pub fn custom(url: &str) -> SourceId {
        SourceId::new(format!("custom-{:016x}", fnv1a(url.trim().as_bytes())))
    }

    /// The identifier as stored.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// A file name for this source in the cache directory. Ids are ASCII by construction for
    /// built-in and custom lists; anything else is replaced so a stored id cannot name a path.
    pub(crate) fn file_stem(&self) -> String {
        self.0
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }
}

impl std::fmt::Display for SourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// What a list blocks. Stats count blocked requests per purpose, and an app can group lists by
/// it in its settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Purpose {
    /// Trackers, analytics, and fingerprinting.
    Privacy,
    /// Advertising.
    Ads,
    /// Cookie consent banners.
    Cookies,
    /// Pop-ups, newsletters, and other page clutter.
    Annoyances,
    /// Social media buttons and widgets.
    Social,
    /// Ads on sites in one language or country.
    Regional,
    /// Malware, phishing, and scam domains.
    Security,
}

impl Purpose {
    /// Every purpose, in declaration order. [`Purpose::index`] is a position in this array.
    pub const ALL: [Purpose; 7] = [
        Purpose::Privacy,
        Purpose::Ads,
        Purpose::Cookies,
        Purpose::Annoyances,
        Purpose::Social,
        Purpose::Regional,
        Purpose::Security,
    ];

    /// The stable id: `privacy`, `ads`, `cookies`, `annoyances`, `social`, `regional`,
    /// `security`.
    pub fn id(self) -> &'static str {
        match self {
            Purpose::Privacy => "privacy",
            Purpose::Ads => "ads",
            Purpose::Cookies => "cookies",
            Purpose::Annoyances => "annoyances",
            Purpose::Social => "social",
            Purpose::Regional => "regional",
            Purpose::Security => "security",
        }
    }

    /// The position in [`Purpose::ALL`].
    pub fn index(self) -> usize {
        self as usize
    }
}

/// The syntax a list is written in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ListFormat {
    /// Adblock Plus / uBlock Origin filter syntax (`||ads.example^`, `##.ad`).
    #[default]
    Adblock,
    /// A hosts file (`0.0.0.0 ads.example`): every host is blocked outright.
    Hosts,
}

/// A filter list: where it comes from, what it blocks, and under which license it is published.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// The stable identifier.
    pub id: SourceId,
    /// The list's own name, such as "EasyList". Not translated.
    pub title: Cow<'static, str>,
    /// What the list blocks.
    pub purpose: Purpose,
    /// The syntax of the downloaded file.
    #[serde(default)]
    pub format: ListFormat,
    /// Download URLs, tried in order: the first is the upstream address, the rest are mirrors
    /// the list's maintainers publish.
    pub urls: Vec<Cow<'static, str>>,
    /// The list's home page.
    #[serde(default)]
    pub homepage: Option<Cow<'static, str>>,
    /// The license the list is published under, as its maintainers name it.
    #[serde(default)]
    pub license: Option<Cow<'static, str>>,
    /// For a regional list, the BCP 47 language tags of the sites it covers.
    #[serde(default)]
    pub languages: Vec<Cow<'static, str>>,
}

impl Source {
    /// A list the app adds by URL. Its purpose defaults to [`Purpose::Ads`] and its format to
    /// [`ListFormat::Adblock`]; the title is the URL until the app sets one.
    pub fn custom(url: &str) -> Source {
        let url = url.trim().to_string();
        Source {
            id: SourceId::custom(&url),
            title: Cow::Owned(url.clone()),
            purpose: Purpose::Ads,
            format: ListFormat::Adblock,
            urls: vec![Cow::Owned(url)],
            homepage: None,
            license: None,
            languages: Vec::new(),
        }
    }

    /// This source with another title.
    pub fn with_title(mut self, title: impl Into<String>) -> Source {
        self.title = Cow::Owned(title.into());
        self
    }

    /// This source with another purpose.
    pub fn with_purpose(mut self, purpose: Purpose) -> Source {
        self.purpose = purpose;
        self
    }

    /// This source with another format.
    pub fn with_format(mut self, format: ListFormat) -> Source {
        self.format = format;
        self
    }
}

const EASYLIST_LICENSE: &str = "GPL-3.0 or CC BY-SA 3.0 (https://easylist.to/pages/licence.html)";
const FANBOY_LICENSE: &str = "CC BY 3.0 (https://creativecommons.org/licenses/by/3.0/)";

#[allow(clippy::too_many_arguments)] // one row of the catalog table
fn builtin(
    id: SourceId,
    title: &'static str,
    purpose: Purpose,
    format: ListFormat,
    urls: &[&'static str],
    homepage: &'static str,
    license: &'static str,
    languages: &[&'static str],
) -> Source {
    Source {
        id,
        title: Cow::Borrowed(title),
        purpose,
        format,
        urls: urls.iter().map(|u| Cow::Borrowed(*u)).collect(),
        homepage: Some(Cow::Borrowed(homepage)),
        license: Some(Cow::Borrowed(license)),
        languages: languages.iter().map(|l| Cow::Borrowed(*l)).collect(),
    }
}

/// The built-in lists, with their upstream download addresses. Nothing here is compiled into
/// the binary; each list is downloaded on first use.
pub fn catalog() -> &'static [Source] {
    static CATALOG: OnceLock<Vec<Source>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        use ListFormat::{Adblock, Hosts};
        use Purpose::*;
        let el = "https://easylist.to/";
        vec![
            builtin(
                SourceId::EASYLIST,
                "EasyList",
                Ads,
                Adblock,
                &[
                    "https://easylist-downloads.adblockplus.org/easylist.txt",
                    "https://easylist.to/easylist/easylist.txt",
                ],
                el,
                EASYLIST_LICENSE,
                &[],
            ),
            builtin(
                SourceId::EASYPRIVACY,
                "EasyPrivacy",
                Privacy,
                Adblock,
                &[
                    "https://easylist-downloads.adblockplus.org/easyprivacy.txt",
                    "https://easylist.to/easylist/easyprivacy.txt",
                ],
                el,
                EASYLIST_LICENSE,
                &[],
            ),
            builtin(
                SourceId::ABP_ANTI_CV,
                "ABP Anti-Circumvention Filter List",
                Ads,
                Adblock,
                &["https://easylist-downloads.adblockplus.org/abp-filters-anti-cv.txt"],
                "https://gitlab.com/eyeo/anti-cv/abp-filters-anti-cv",
                "GPL-3.0",
                &[],
            ),
            builtin(
                SourceId::ADBLOCK_WARNING_REMOVAL,
                "Adblock Warning Removal List",
                Ads,
                Adblock,
                &["https://easylist-downloads.adblockplus.org/antiadblockfilters.txt"],
                "https://github.com/easylist/antiadblockfilters",
                EASYLIST_LICENSE,
                &[],
            ),
            builtin(
                SourceId::FANBOY_SOCIAL,
                "Fanboy's Social Blocking List",
                Social,
                Adblock,
                &["https://secure.fanboy.co.nz/fanboy-social.txt"],
                el,
                FANBOY_LICENSE,
                &[],
            ),
            builtin(
                SourceId::EASYLIST_COOKIE,
                "Easylist Cookie List",
                Cookies,
                Adblock,
                &["https://secure.fanboy.co.nz/fanboy-cookiemonster.txt"],
                el,
                FANBOY_LICENSE,
                &[],
            ),
            builtin(
                SourceId::FANBOY_ANNOYANCE,
                "Fanboy's Annoyance List",
                Annoyances,
                Adblock,
                &["https://secure.fanboy.co.nz/fanboy-annoyance.txt"],
                el,
                FANBOY_LICENSE,
                &[],
            ),
            builtin(
                SourceId::PETER_LOWE,
                "Peter Lowe's Ad and tracking server list",
                Ads,
                Hosts,
                &[
                    "https://pgl.yoyo.org/adservers/serverlist.php?hostformat=hosts&showintro=1&mimetype=plaintext",
                ],
                "https://pgl.yoyo.org/adservers/",
                "No license published; terms at https://pgl.yoyo.org/adservers/",
                &[],
            ),
            builtin(
                SourceId::EASYLIST_GERMANY,
                "EasyList Germany",
                Regional,
                Adblock,
                &["https://easylist-downloads.adblockplus.org/easylistgermany.txt"],
                "https://easylist.to/",
                EASYLIST_LICENSE,
                &["de"],
            ),
            builtin(
                SourceId::EASYLIST_DUTCH,
                "EasyList Dutch",
                Regional,
                Adblock,
                &["https://easylist-downloads.adblockplus.org/easylistdutch.txt"],
                el,
                EASYLIST_LICENSE,
                &["nl"],
            ),
            builtin(
                SourceId::EASYLIST_SPANISH,
                "EasyList Spanish",
                Regional,
                Adblock,
                &["https://easylist-downloads.adblockplus.org/easylistspanish.txt"],
                el,
                EASYLIST_LICENSE,
                &["es"],
            ),
            builtin(
                SourceId::EASYLIST_ITALY,
                "EasyList Italy",
                Regional,
                Adblock,
                &["https://easylist-downloads.adblockplus.org/easylistitaly.txt"],
                el,
                EASYLIST_LICENSE,
                &["it"],
            ),
            builtin(
                SourceId::EASYLIST_PORTUGUESE,
                "EasyList Portuguese",
                Regional,
                Adblock,
                &["https://easylist-downloads.adblockplus.org/easylistportuguese.txt"],
                el,
                EASYLIST_LICENSE,
                &["pt"],
            ),
            builtin(
                SourceId::EASYLIST_CHINA,
                "EasyList China",
                Regional,
                Adblock,
                &["https://raw.githubusercontent.com/easylist/easylistchina/refs/heads/master/easylistchina.txt"],
                "https://github.com/easylist/easylistchina/",
                EASYLIST_LICENSE,
                &["zh"],
            ),
            builtin(
                SourceId::EASYLIST_POLISH,
                "EasyList Polish",
                Regional,
                Adblock,
                &["https://easylist-downloads.adblockplus.org/easylistpolish.txt"],
                el,
                EASYLIST_LICENSE,
                &["pl"],
            ),
            builtin(
                SourceId::LISTE_FR,
                "Liste FR",
                Regional,
                Adblock,
                &["https://easylist-downloads.adblockplus.org/liste_fr.txt"],
                "https://github.com/easylist/listefr",
                "CC BY-SA 3.0 (https://creativecommons.org/licenses/by-sa/3.0/)",
                &["fr"],
            ),
        ]
    })
}

/// The built-in source with this id.
pub fn builtin_source(id: &SourceId) -> Option<&'static Source> {
    catalog().iter().find(|s| &s.id == id)
}

/// The built-in regional lists that cover `language` (a BCP 47 tag such as `de` or `fr-CA`;
/// only the primary subtag is compared).
pub fn regional_sources(language: &str) -> Vec<&'static Source> {
    let primary = language
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    catalog()
        .iter()
        .filter(|s| s.languages.iter().any(|l| l.eq_ignore_ascii_case(&primary)))
        .collect()
}

// --- Levels ---

/// How much a page is filtered. The app names and describes each level in its own language;
/// [`Level::id`] is the stable key.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "level", rename_all = "kebab-case")]
pub enum Level {
    /// Nothing is blocked.
    Off,
    /// Trackers and social widgets: EasyPrivacy and Fanboy's Social Blocking List.
    #[default]
    Light,
    /// Light plus ads: EasyList, Adblock Warning Removal List, and the ABP anti-circumvention
    /// list's network and element-hiding rules.
    Strong,
    /// Strong plus Peter Lowe's list and Easylist Cookie List, with document blocking and
    /// same-site blocking on.
    Aggressive,
    /// The app's own selection.
    Custom {
        /// The lists to use, built-in or custom.
        sources: Vec<SourceId>,
        /// Block navigations to a blocked page, so the host can show an interstitial.
        #[serde(default)]
        document_blocking: bool,
        /// Block requests to the page's own site.
        #[serde(default)]
        same_site_blocking: bool,
    },
}

impl Level {
    /// The stable id: `off`, `light`, `strong`, `aggressive`, or `custom`.
    pub fn id(&self) -> &'static str {
        match self {
            Level::Off => "off",
            Level::Light => "light",
            Level::Strong => "strong",
            Level::Aggressive => "aggressive",
            Level::Custom { .. } => "custom",
        }
    }

    /// The level's own lists, without the settings' extra sources.
    pub fn sources(&self) -> Vec<SourceId> {
        let light = [SourceId::EASYPRIVACY, SourceId::FANBOY_SOCIAL];
        let strong = [
            SourceId::EASYLIST,
            SourceId::ABP_ANTI_CV,
            SourceId::ADBLOCK_WARNING_REMOVAL,
        ];
        let aggressive = [SourceId::PETER_LOWE, SourceId::EASYLIST_COOKIE];
        match self {
            Level::Off => Vec::new(),
            Level::Light => light.to_vec(),
            Level::Strong => strong.iter().chain(&light).cloned().collect(),
            Level::Aggressive => strong
                .iter()
                .chain(&light)
                .chain(&aggressive)
                .cloned()
                .collect(),
            Level::Custom { sources, .. } => sources.clone(),
        }
    }

    /// Whether a navigation to a blocked page is itself blocked.
    pub fn document_blocking(&self) -> bool {
        match self {
            Level::Aggressive => true,
            Level::Custom {
                document_blocking, ..
            } => *document_blocking,
            _ => false,
        }
    }

    /// Whether requests to the page's own site can be blocked.
    pub fn same_site_blocking(&self) -> bool {
        match self {
            Level::Aggressive => true,
            Level::Custom {
                same_site_blocking, ..
            } => *same_site_blocking,
            _ => false,
        }
    }

    /// Whether this level blocks nothing.
    pub fn is_off(&self) -> bool {
        matches!(self, Level::Off)
    }

    /// The name used in cache file names: the id, or `custom-<hash>` for a custom level.
    pub(crate) fn cache_key(&self) -> String {
        match self {
            Level::Custom { .. } => {
                let mut h = Fnv::new();
                for s in self.sources() {
                    h.write(s.as_str().as_bytes());
                    h.write(&[0]);
                }
                h.write(&[
                    u8::from(self.document_blocking()),
                    u8::from(self.same_site_blocking()),
                ]);
                format!("custom-{:08x}", h.finish() as u32)
            }
            _ => self.id().to_string(),
        }
    }
}

// --- Settings ---

/// Per-site levels, keyed by host. A host matches its own entry or the nearest parent domain's:
/// an entry for `news.example` also covers `www.news.example`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SiteLevels(BTreeMap<String, Level>);

impl SiteLevels {
    /// No per-site levels.
    pub fn new() -> SiteLevels {
        SiteLevels::default()
    }

    /// The level stored for `host` or its nearest parent domain.
    pub fn get(&self, host: &str) -> Option<&Level> {
        let host = normalize_host(host);
        let mut rest = host.as_str();
        loop {
            if let Some(level) = self.0.get(rest) {
                return Some(level);
            }
            rest = &rest[rest.find('.')? + 1..];
        }
    }

    /// Store `level` for `host`, or remove its entry with `None`.
    pub fn set(&mut self, host: &str, level: Option<Level>) {
        let host = normalize_host(host);
        if host.is_empty() {
            return;
        }
        match level {
            Some(level) => {
                self.0.insert(host, level);
            }
            None => {
                self.0.remove(&host);
            }
        }
    }

    /// Every entry, sorted by host.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Level)> {
        self.0.iter().map(|(h, l)| (h.as_str(), l))
    }

    /// Whether there are no entries.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Lowercase, without a trailing dot or a leading `www.`.
pub(crate) fn normalize_host(host: &str) -> String {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    match host.strip_prefix("www.") {
        Some(rest) if !rest.is_empty() => rest.to_string(),
        _ => host,
    }
}

/// Everything about blocking that the app persists, as one serde value. The crate keeps no
/// settings file of its own.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The level for every site without its own entry.
    pub level: Level,
    /// Lists added to every level except [`Level::Off`], such as the regional list for the
    /// person's language.
    pub extra_sources: Vec<SourceId>,
    /// The definitions of lists added by URL. A level or `extra_sources` refers to them by id.
    pub custom_sources: Vec<Source>,
    /// Per-site levels.
    pub sites: SiteLevels,
}

impl Settings {
    /// The level for a page on `host`: its site entry, or the global level.
    pub fn level_for(&self, host: &str) -> &Level {
        self.sites.get(host).unwrap_or(&self.level)
    }

    /// The lists `level` uses with these settings: its own and the extra sources, without
    /// duplicates, in a stable order.
    pub fn sources_for(&self, level: &Level) -> Vec<SourceId> {
        if level.is_off() {
            return Vec::new();
        }
        let mut out = level.sources();
        for s in &self.extra_sources {
            if !out.contains(s) {
                out.push(s.clone());
            }
        }
        out
    }

    /// The definition of `id`: a built-in list or one of `custom_sources`.
    pub fn source(&self, id: &SourceId) -> Option<Source> {
        builtin_source(id)
            .cloned()
            .or_else(|| self.custom_sources.iter().find(|s| &s.id == id).cloned())
    }

    /// Every level in use: the global one and each site's, without duplicates, with
    /// [`Level::Off`] left out.
    pub fn active_levels(&self) -> Vec<Level> {
        let mut out: Vec<Level> = Vec::new();
        for level in std::iter::once(&self.level).chain(self.sites.0.values()) {
            if !level.is_off() && !out.contains(level) {
                out.push(level.clone());
            }
        }
        out
    }
}

// --- Hashing ---

/// FNV-1a, 64 bits: the cache keys and the custom list ids. Not a security boundary: a
/// collision only means a needless rebuild or a cache file read back and rejected.
pub(crate) struct Fnv(u64);

impl Fnv {
    pub(crate) fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }

    pub(crate) fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 ^= u64::from(*b);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    pub(crate) fn finish(&self) -> u64 {
        self.0
    }
}

pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h = Fnv::new();
    h.write(bytes);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_levels_match_parent_domains_and_ignore_www() {
        let mut sites = SiteLevels::new();
        sites.set("WWW.News.Example.", Some(Level::Off));
        assert_eq!(sites.get("news.example"), Some(&Level::Off));
        assert_eq!(sites.get("a.b.news.example"), Some(&Level::Off));
        assert_eq!(sites.get("othernews.example"), None);
        sites.set("news.example", None);
        assert!(sites.is_empty());
    }

    #[test]
    fn custom_ids_are_stable_and_path_safe() {
        let a = SourceId::custom("https://lists.example/my.txt");
        assert_eq!(a, SourceId::custom(" https://lists.example/my.txt "));
        assert!(a.as_str().starts_with("custom-"));
        assert_eq!(SourceId::new("../x").file_stem(), "___x");
    }
}
