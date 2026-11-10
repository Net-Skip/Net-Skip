// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! The clock and the local time zone, for history's day headings and visit times. The host
//! already carries the zone rules (`localtime_r` on Unix, the registry behind Win32's
//! converters), so the offset is asked of it rather than bundled.

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// The local offset from UTC at `at_unix` seconds, in seconds east of UTC, with daylight
/// saving applied at that instant. `None` when the host cannot say; treat it as UTC.
#[cfg(unix)]
pub fn local_offset_seconds(at_unix: i64) -> Option<i64> {
    let t = at_unix as _;
    // SAFETY: an all-zero `struct tm` is a valid value of the type; `localtime_r` overwrites it.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    // SAFETY: `t` is a valid `time_t` and `tm` a writable `struct tm` for the whole call; a null
    // return (failure) is checked before any field is read.
    let filled = unsafe { libc::localtime_r(&t, &mut tm) };
    if filled.is_null() {
        return None;
    }
    Some(tm.tm_gmtoff as i64)
}

/// Windows has no `tm_gmtoff`: convert the instant to local time with the machine's zone rules
/// and measure the difference.
#[cfg(windows)]
pub fn local_offset_seconds(at_unix: i64) -> Option<i64> {
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct SystemTime {
        year: u16,
        month: u16,
        day_of_week: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        milliseconds: u16,
    }
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    unsafe extern "system" {
        fn FileTimeToSystemTime(ft: *const FileTime, st: *mut SystemTime) -> i32;
        fn SystemTimeToTzSpecificLocalTime(
            zone: *const core::ffi::c_void,
            utc: *const SystemTime,
            local: *mut SystemTime,
        ) -> i32;
        fn SystemTimeToFileTime(st: *const SystemTime, ft: *mut FileTime) -> i32;
    }
    // FILETIME counts 100 ns ticks from 1601-01-01, 11 644 473 600 seconds before 1970.
    const TICKS_PER_SEC: i128 = 10_000_000;
    const EPOCH_DELTA_SECS: i128 = 11_644_473_600;
    let ticks = u64::try_from((i128::from(at_unix) + EPOCH_DELTA_SECS) * TICKS_PER_SEC).ok()?;
    let utc_ft = FileTime {
        low: ticks as u32,
        high: (ticks >> 32) as u32,
    };
    let mut utc = SystemTime::default();
    let mut local = SystemTime::default();
    let mut local_ft = FileTime::default();
    // SAFETY: every pointer is to a live local of the declared shape, and each call's result is
    // checked before what it wrote is used.
    unsafe {
        if FileTimeToSystemTime(&utc_ft, &mut utc) == 0
            || SystemTimeToTzSpecificLocalTime(core::ptr::null(), &utc, &mut local) == 0
            || SystemTimeToFileTime(&local, &mut local_ft) == 0
        {
            return None;
        }
    }
    let local_ticks = u64::from(local_ft.low) | (u64::from(local_ft.high) << 32);
    i64::try_from((local_ticks as i128 - ticks as i128) / TICKS_PER_SEC).ok()
}

#[cfg(not(any(unix, windows)))]
pub fn local_offset_seconds(_at_unix: i64) -> Option<i64> {
    None
}

/// The local offset now.
pub fn utc_offset_secs() -> i64 {
    local_offset_seconds(now_ms() / 1000).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_are_within_a_day() {
        let off = utc_offset_secs();
        assert!(off.abs() <= 14 * 3600, "{off}");
        assert!(now_ms() > 1_700_000_000_000);
    }
}
