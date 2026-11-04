//! A repeating command's times: a 5-field cron expression (`minute hour day-of-month month day-of-week`), read in
//! the zone it is given (local time in IVE).

use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDate, TimeZone, Timelike};

/// How far ahead [`Cron::next_after`] looks: past this, an expression such as `0 0 30 2 *` never matches.
const SEARCH_DAYS: i64 = 4 * 366;
/// Times closer than this are refused, so a mistaken `* * * * *` cannot start a terminal every minute.
pub const MIN_GAP_MINUTES: i64 = 15;
/// The times [`Cron::check_gaps`] looks at.
const CHECKED_TIMES: usize = 24;

/// A parsed cron expression; `text` as it was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cron {
    text: String,
    minutes: u64,
    hours: u64,
    days: u64,
    months: u64,
    weekdays: u64,
    /// whether day of month and day of week were each `*`: cron matches either when both are restricted
    any_day: bool,
    any_weekday: bool,
}

/// One field's name and range, for parsing and for messages.
struct Field {
    name: &'static str,
    min: u32,
    max: u32,
}

const MINUTE: Field = Field { name: "minute", min: 0, max: 59 };
const HOUR: Field = Field { name: "hour", min: 0, max: 23 };
const DAY: Field = Field { name: "day of month", min: 1, max: 31 };
const MONTH: Field = Field { name: "month", min: 1, max: 12 };
/// 0 and 7 are both Sunday
const WEEKDAY: Field = Field { name: "day of week", min: 0, max: 7 };

impl Cron {
    /// Parses `0 7 * * 1-5`: each field `*`, a number, `a-b`, a list of those, each optionally `/step`.
    pub fn parse(text: &str) -> Result<Self, String> {
        let fields: Vec<&str> = text.split_whitespace().collect();
        let [minute, hour, day, month, weekday] = fields[..] else {
            return Err(format!("repeat {text:?}: a cron needs 5 fields (minute hour day-of-month month day-of-week), not {}", fields.len()));
        };
        let weekdays = parse_field(weekday, &WEEKDAY, text)?;
        Ok(Self {
            text: fields.join(" "),
            minutes: parse_field(minute, &MINUTE, text)?,
            hours: parse_field(hour, &HOUR, text)?,
            days: parse_field(day, &DAY, text)?,
            months: parse_field(month, &MONTH, text)?,
            // Sunday as 7 is Sunday as 0
            weekdays: (weekdays | (weekdays >> 7)) & 0x7f,
            any_day: day == "*",
            any_weekday: weekday == "*",
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// The first time after `after` (to the minute) that the expression matches; `None` when none comes within 4
    /// years. A time that does not exist in the zone (a DST gap) is skipped; one that exists twice is the earlier.
    pub fn next_after<Tz: TimeZone>(&self, after: &DateTime<Tz>) -> Option<DateTime<Tz>> {
        let zone = after.timezone();
        let start = after.date_naive();
        (0..SEARCH_DAYS)
            .filter_map(|offset| start.checked_add_signed(Duration::days(offset)))
            .filter(|date| self.matches_day(*date))
            .find_map(|date| self.first_on(date, &zone, after))
    }

    /// Refuses an expression that never matches, or whose next times come closer than [`MIN_GAP_MINUTES`].
    pub fn check_gaps<Tz: TimeZone>(&self, from: &DateTime<Tz>) -> Result<(), String> {
        let mut times = Vec::with_capacity(CHECKED_TIMES);
        let mut last = from.clone();
        while times.len() < CHECKED_TIMES {
            let Some(next) = self.next_after(&last) else {
                break;
            };
            times.push(next.clone());
            last = next;
        }
        if times.is_empty() {
            return Err(format!("repeat {:?} never comes in the next 4 years", self.text));
        }
        let too_close = times.windows(2).any(|pair| pair[1].clone().signed_duration_since(pair[0].clone()) < Duration::minutes(MIN_GAP_MINUTES));
        if too_close {
            return Err(format!("repeat {:?} comes more often than every {MIN_GAP_MINUTES} minutes", self.text));
        }
        Ok(())
    }

    fn matches_day(&self, date: NaiveDate) -> bool {
        if !has(self.months, date.month()) {
            return false;
        }
        let day = has(self.days, date.day());
        let weekday = has(self.weekdays, date.weekday().num_days_from_sunday());
        match (self.any_day, self.any_weekday) {
            (false, false) => day || weekday,
            _ => day && weekday,
        }
    }

    fn first_on<Tz: TimeZone>(&self, date: NaiveDate, zone: &Tz, after: &DateTime<Tz>) -> Option<DateTime<Tz>> {
        let times = (0..24u32).filter(|hour| has(self.hours, *hour)).flat_map(|hour| (0..60u32).filter(|minute| has(self.minutes, *minute)).map(move |minute| (hour, minute)));
        times
            .filter_map(|(hour, minute)| date.and_hms_opt(hour, minute, 0))
            .filter_map(|naive| match zone.from_local_datetime(&naive) {
                LocalResult::Single(time) => Some(time),
                LocalResult::Ambiguous(earlier, _) => Some(earlier),
                LocalResult::None => None,
            })
            .find(|time| time > after && time.second() == 0)
    }
}

fn has(set: u64, value: u32) -> bool {
    set & (1 << value) != 0
}

/// A field's values as bits; `whole` is the expression, for messages.
fn parse_field(text: &str, field: &Field, whole: &str) -> Result<u64, String> {
    text.split(',').try_fold(0u64, |set, part| Ok(set | parse_part(part, field).map_err(|reason| format!("repeat {whole:?}: {} {part:?}: {reason}", field.name))?))
}

fn parse_part(part: &str, field: &Field) -> Result<u64, String> {
    let (range, step) = match part.split_once('/') {
        Some((range, step)) => (range, step.parse::<u32>().ok().filter(|step| *step > 0).ok_or_else(|| format!("the step must be a number above 0, not {step:?}"))?),
        None => (part, 1),
    };
    let (low, high) = match range {
        "*" => (field.min, field.max),
        _ => match range.split_once('-') {
            Some((low, high)) => (number(low, field)?, number(high, field)?),
            // `5/15` runs from 5 to the end, as cron reads it
            None if step > 1 => (number(range, field)?, field.max),
            None => {
                let value = number(range, field)?;
                (value, value)
            }
        },
    };
    if low > high {
        return Err(format!("{low} is after {high}"));
    }
    Ok((low..=high).step_by(step as usize).fold(0, |set, value| set | (1 << value)))
}

fn number(text: &str, field: &Field) -> Result<u32, String> {
    let value: u32 = text.parse().map_err(|_| format!("{text:?} is not a number"))?;
    if value < field.min || value > field.max {
        return Err(format!("{value} is outside {}–{}", field.min, field.max));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, Utc};

    use super::*;

    fn at(text: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(text).unwrap()
    }

    #[test]
    fn should_find_the_next_weekday_morning_over_a_weekend() {
        // given
        let cron = Cron::parse("0 7 * * 1-5").unwrap();
        // when: Friday 2026-10-16, 14:00
        let next = cron.next_after(&at("2026-10-16T14:00:00+02:00"));
        // then
        assert_eq!(next, Some(at("2026-10-19T07:00:00+02:00")));
    }

    #[test]
    fn should_take_a_later_time_the_same_day() {
        // given
        let cron = Cron::parse("30 9,17 * * *").unwrap();
        // when
        let next = cron.next_after(&at("2026-10-16T09:30:00+00:00"));
        // then: 09:30 itself is not after 09:30
        assert_eq!(next, Some(at("2026-10-16T17:30:00+00:00")));
    }

    #[test]
    fn should_read_steps_ranges_and_lists() {
        // given
        let cron = Cron::parse("*/20 8-9 1,15 * *").unwrap();
        // when
        let times: Vec<String> = std::iter::successors(cron.next_after(&at("2026-10-14T00:00:00+00:00")), |time| cron.next_after(time))
            .take(4)
            .map(|time| time.format("%d %H:%M").to_string())
            .collect();
        // then
        assert_eq!(times, ["15 08:00", "15 08:20", "15 08:40", "15 09:00"]);
    }

    #[test]
    fn should_match_either_day_when_both_are_restricted() {
        // given: the 1st, or any Monday
        let cron = Cron::parse("0 12 1 * 1").unwrap();
        // when: from Wednesday 2026-10-28
        let first = cron.next_after(&at("2026-10-28T13:00:00+00:00")).unwrap();
        let second = cron.next_after(&first).unwrap();
        // then: Sunday the 1st, then Monday the 2nd
        assert_eq!((first, second), (at("2026-11-01T12:00:00+00:00"), at("2026-11-02T12:00:00+00:00")));
    }

    #[test]
    fn should_read_sunday_as_seven() {
        // given
        let cron = Cron::parse("0 10 * * 7").unwrap();
        // when: from Friday
        let next = cron.next_after(&at("2026-10-16T14:00:00+00:00"));
        // then
        assert_eq!(next, Some(at("2026-10-18T10:00:00+00:00")));
    }

    #[test]
    fn should_refuse_a_wrong_field_naming_it() {
        // given
        let wrong = ["0 7 * *", "61 * * * *", "0 7 * * mon", "0 9-7 * * *", "*/0 * * * *"];
        // when
        let errors: Vec<String> = wrong.iter().map(|text| Cron::parse(text).unwrap_err()).collect();
        // then
        assert!(errors[0].contains("5 fields"), "{}", errors[0]);
        assert!(errors[1].contains("minute \"61\": 61 is outside 0–59"), "{}", errors[1]);
        assert!(errors[2].contains("day of week \"mon\""), "{}", errors[2]);
        assert!(errors[3].contains("9 is after 7"), "{}", errors[3]);
        assert!(errors[4].contains("step"), "{}", errors[4]);
    }

    #[test]
    fn should_refuse_times_closer_than_fifteen_minutes() {
        // given
        let every_ten = Cron::parse("*/10 * * * *").unwrap();
        let hourly = Cron::parse("0 * * * *").unwrap();
        let now = Utc::now();
        // when
        let refused = every_ten.check_gaps(&now);
        let allowed = hourly.check_gaps(&now);
        // then
        assert!(refused.unwrap_err().contains("more often than every 15 minutes"));
        assert_eq!(allowed, Ok(()));
    }

    #[test]
    fn should_refuse_a_date_that_never_comes() {
        // given
        let cron = Cron::parse("0 0 30 2 *").unwrap();
        // when
        let refused = cron.check_gaps(&Utc::now());
        // then
        assert!(refused.unwrap_err().contains("never comes"));
    }
}
