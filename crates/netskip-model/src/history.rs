// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! History rules: what a visit records, how the history list groups by day, and how far back
//! each Clear History choice reaches.

use crate::address;

const HOUR_MS: i64 = 3_600_000;
const DAY_MS: i64 = 86_400_000;

/// Whether loading `url` is recorded in history. Private tabs record nothing, and neither do
/// blank and internal pages, data URLs, or the browser's own fixture namespace.
pub fn records(url: &str, private: bool) -> bool {
    !private && address::history_key(url).is_some()
}

/// The visit a page load records: the deduplication key, and whether the page title should
/// replace the stored one (an empty title never overwrites a real one).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Visit {
    pub key: String,
    pub title: Option<String>,
    pub typed: bool,
}

/// The visit loading `url` records, or `None` when it records nothing.
pub fn visit(url: &str, title: &str, typed: bool, private: bool) -> Option<Visit> {
    if !records(url, private) {
        return None;
    }
    let title = title.trim();
    Some(Visit {
        key: address::history_key(url)?,
        title: (!title.is_empty()).then(|| title.to_string()),
        typed,
    })
}

/// A local calendar day: days since 1970-01-01 in the user's time zone.
pub fn local_day(ms: i64, utc_offset_secs: i64) -> i64 {
    (ms + utc_offset_secs * 1000).div_euclid(DAY_MS)
}

/// The civil date (year, month 1–12, day 1–31) of a day number, by Howard Hinnant's
/// `civil_from_days`.
pub fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The day of the week, 0 = Monday … 6 = Sunday (1970-01-01 was a Thursday).
pub fn weekday(days: i64) -> u32 {
    (days + 3).rem_euclid(7) as u32
}

/// The heading a history entry is grouped under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DayGroup {
    Today,
    Yesterday,
    /// Within the last week: the weekday's name (0 = Monday).
    Weekday(u32),
    /// Older: the date, as a local day number ([`civil`] spells it).
    Date(i64),
}

/// Group a visit time against now, both in milliseconds, in the user's time zone.
pub fn day_group(visit: i64, now: i64, utc_offset_secs: i64) -> DayGroup {
    let day = local_day(visit, utc_offset_secs);
    let today = local_day(now, utc_offset_secs);
    match today - day {
        i64::MIN..=0 => DayGroup::Today,
        1 => DayGroup::Yesterday,
        2..=6 => DayGroup::Weekday(weekday(day)),
        _ => DayGroup::Date(day),
    }
}

/// The ranges Clear History offers, Safari's set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearRange {
    LastHour,
    Today,
    TodayAndYesterday,
    All,
}

impl ClearRange {
    pub const ALL: [ClearRange; 4] = [
        ClearRange::LastHour,
        ClearRange::Today,
        ClearRange::TodayAndYesterday,
        ClearRange::All,
    ];

    /// The earliest visit time the range removes; `None` removes everything.
    pub fn since(self, now: i64, utc_offset_secs: i64) -> Option<i64> {
        let start_of_today = local_day(now, utc_offset_secs) * DAY_MS - utc_offset_secs * 1000;
        match self {
            ClearRange::LastHour => Some(now - HOUR_MS),
            ClearRange::Today => Some(start_of_today),
            ClearRange::TodayAndYesterday => Some(start_of_today - DAY_MS),
            ClearRange::All => None,
        }
    }

    /// The id a dayscript and the preference store use.
    pub fn id(self) -> &'static str {
        match self {
            ClearRange::LastHour => "hour",
            ClearRange::Today => "today",
            ClearRange::TodayAndYesterday => "two-days",
            ClearRange::All => "all",
        }
    }
}
