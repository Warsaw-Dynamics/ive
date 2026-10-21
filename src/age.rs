//! Short ages and dates, as commit lists (part 2) and scratch lists (part 3) show them.

use std::time::{SystemTime, UNIX_EPOCH};

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
/// Older than this many days before today, an age shows as the date.
const COUNTED_DAYS: i64 = 6;

/// A moment as git stores it: seconds since the epoch and the author's offset from UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct When {
    pub seconds: i64,
    pub offset_minutes: i32,
}

/// "Now" in seconds since the epoch and the viewer's offset from UTC in seconds; given, so tests are exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    now: i64,
    offset: i64,
}

impl Clock {
    pub fn new(now: i64, offset: i64) -> Self {
        Self { now, offset }
    }

    /// The system clock and the local time zone's current offset.
    pub fn system() -> Self {
        let now = unix_seconds(SystemTime::now());
        Self { now, offset: local_offset(now) }
    }

    pub fn now(self) -> i64 {
        self.now
    }

    /// The moment "now", in the viewer's zone.
    pub fn when(self) -> When {
        When { seconds: self.now, offset_minutes: (self.offset / MINUTE) as i32 }
    }

    /// Local midnight that started today, in seconds since the epoch.
    pub fn midnight(self) -> i64 {
        let local = self.now.saturating_add(self.offset);
        (local - local.rem_euclid(DAY)).saturating_sub(self.offset)
    }

    /// `now`, `5 min`, `3 h` today (the future included); `yesterday`; `2 d` … `6 d`; then the local date.
    pub fn age(self, seconds: i64) -> String {
        let midnight = self.midnight();
        if seconds >= midnight {
            let minutes = self.now.saturating_sub(seconds).max(0) / MINUTE;
            return match minutes {
                0 => "now".to_owned(),
                1..=59 => format!("{minutes} min"),
                _ => format!("{} h", minutes / 60),
            };
        }
        match midnight.saturating_sub(seconds).saturating_sub(1) / DAY + 1 {
            1 => "yesterday".to_owned(),
            days @ 2..=COUNTED_DAYS => format!("{days} d"),
            _ => date(When { seconds, offset_minutes: (self.offset / MINUTE) as i32 }),
        }
    }
}

/// `2023-11-15`, in the zone of `when`.
pub fn date(when: When) -> String {
    let (year, month, day, _) = local(when);
    format!("{year:04}-{month:02}-{day:02}")
}

/// `2023-11-15 00:13 +0200`, in the zone of `when`.
pub fn date_time(when: When) -> String {
    let (year, month, day, seconds) = local(when);
    let sign = if when.offset_minutes < 0 { '-' } else { '+' };
    let offset = when.offset_minutes.unsigned_abs();
    let (hour, minute) = (seconds / HOUR, seconds % HOUR / MINUTE);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02} {sign}{:02}{:02}", offset / 60, offset % 60)
}

/// Seconds since the epoch; negative before it.
pub fn unix_seconds(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(since) => i64::try_from(since.as_secs()).unwrap_or(i64::MAX),
        Err(before) => i64::try_from(before.duration().as_secs()).map_or(i64::MIN, |seconds| -seconds),
    }
}

/// Year, month, day and seconds into that day, in the zone of `when`.
fn local(when: When) -> (i64, u32, u32, i64) {
    let seconds = when.seconds.saturating_add(i64::from(when.offset_minutes) * MINUTE);
    let (year, month, day) = civil_from_days(seconds.div_euclid(DAY));
    (year, month, day, seconds.rem_euclid(DAY))
}

/// Days since 1970-01-01 to a calendar date: Howard Hinnant's `civil_from_days`.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days.saturating_add(719_468);
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
    let month = (if month_index < 10 { month_index + 3 } else { month_index - 9 }) as u32;
    (year_of_era + era * 400 + i64::from(month <= 2), month, day)
}

/// Seconds east of UTC at `now`; 0 where the platform cannot tell.
#[cfg(unix)]
#[allow(clippy::useless_conversion)] // c_long is i32 on 32-bit targets
fn local_offset(now: i64) -> i64 {
    let time = now as libc::time_t;
    // SAFETY: an all-zero `tm` is valid; localtime_r writes only into it
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let converted = unsafe { libc::localtime_r(&time, &mut tm) };
    if converted.is_null() { 0 } else { i64::from(tm.tm_gmtoff) }
}

#[cfg(not(unix))]
fn local_offset(_now: i64) -> i64 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-16 12:00 UTC
    const NOW: i64 = 1_792_152_000;
    const MIDNIGHT: i64 = NOW - 12 * HOUR;

    fn utc() -> Clock {
        Clock::new(NOW, 0)
    }

    #[test]
    fn should_write_ages_as_minutes_hours_yesterday_days_then_the_date() {
        // given
        let times = [NOW - 30, NOW - 5 * MINUTE, NOW - 3 * HOUR, MIDNIGHT - 1, MIDNIGHT - DAY - 1, MIDNIGHT - 5 * DAY - 1, MIDNIGHT - 6 * DAY - 1];

        // when
        let ages = times.map(|time| utc().age(time));

        // then
        assert_eq!(ages, ["now", "5 min", "3 h", "yesterday", "2 d", "6 d", "2026-10-09"]);
    }

    #[test]
    fn should_show_future_times_as_now() {
        // given
        let tomorrow = NOW + DAY;

        // when
        let age = utc().age(tomorrow);

        // then
        assert_eq!(age, "now");
    }

    #[test]
    fn should_count_days_from_local_midnight() {
        // given
        let warsaw = Clock::new(NOW, 2 * HOUR);
        let new_york = Clock::new(NOW, -4 * HOUR);
        let one_am_utc = MIDNIGHT + HOUR;

        // when
        let ages = [warsaw.age(one_am_utc), new_york.age(one_am_utc)];

        // then
        assert_eq!(ages, ["11 h", "yesterday"]);
    }

    #[test]
    fn should_show_date_and_time_in_the_zone_of_the_moment() {
        // given
        let east = When { seconds: 1_700_000_000, offset_minutes: 120 };
        let west = When { seconds: 1_700_000_000, offset_minutes: -330 };

        // when
        let shown = [date_time(east), date_time(west), date(west)];

        // then
        assert_eq!(shown, ["2023-11-15 00:13 +0200", "2023-11-14 16:43 -0530", "2023-11-14"]);
    }

    #[test]
    fn should_stay_within_range_at_the_ends_of_time() {
        // given
        let clock = Clock::new(i64::MAX, 14 * HOUR);
        let first = When { seconds: i64::MIN, offset_minutes: -720 };

        // when
        let written = [clock.age(i64::MIN), clock.age(i64::MAX), date_time(first)];

        // then
        assert!(written.iter().all(|text| !text.is_empty()), "{written:?}");
    }

    #[test]
    fn should_read_a_plausible_local_offset() {
        // given
        let now = unix_seconds(SystemTime::now());

        // when
        let offset = local_offset(now);

        // then
        assert!(offset % 60 == 0 && offset.abs() <= 14 * HOUR, "{offset}");
    }
}
