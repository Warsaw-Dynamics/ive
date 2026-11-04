//! When a scheduled command first comes due: `at` (RFC 3339), `in` (a delay), or a cron's next time.

use chrono::{DateTime, Duration, TimeZone};

use super::cron::Cron;

const MAX_AHEAD_DAYS: i64 = 366;
/// An `at` this far in the past is a mistake, not a command to run at once.
const MAX_PAST_HOURS: i64 = 24;

/// What the agent said about time; each `None` when not given.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Timing {
    pub at: Option<String>,
    pub delay: Option<String>,
    pub repeat: Option<String>,
}

/// The first time, in seconds since the epoch, and the cron of a repeating command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstTime {
    pub at: i64,
    pub repeat: Option<Cron>,
}

/// The first time `timing` gives after `now`, within the bounds: exactly one of `at` and `in`, or a cron with at
/// most one of them.
pub fn first_time<Tz: TimeZone>(timing: &Timing, now: &DateTime<Tz>) -> Result<FirstTime, String> {
    let repeat = timing.repeat.as_deref().map(Cron::parse).transpose()?;
    if let Some(cron) = &repeat {
        cron.check_gaps(now)?;
    }
    let at = match (&timing.at, &timing.delay, &repeat) {
        (Some(_), Some(_), _) => return Err("give `at` or `in`, not both".to_owned()),
        (Some(at), None, _) => parse_at(at)?,
        (None, Some(delay), _) => now.timestamp() + parse_delay(delay)?.num_seconds(),
        (None, None, Some(cron)) => cron.next_after(now).map(|time| time.timestamp()).ok_or_else(|| format!("repeat {:?} never comes", cron.text()))?,
        (None, None, None) => return Err("say when: `at` (RFC 3339), `in` (\"2h\") or `repeat` (a cron)".to_owned()),
    };
    check_bounds(at, now.timestamp())?;
    Ok(FirstTime { at, repeat })
}

/// `2026-11-04T16:00:00+01:00` as seconds since the epoch; an offset (or `Z`) is required.
pub fn parse_at(text: &str) -> Result<i64, String> {
    DateTime::parse_from_rfc3339(text.trim())
        .map(|time| time.timestamp())
        .map_err(|error| format!("at {text:?} is not an RFC 3339 time with an offset, e.g. 2026-11-04T16:00:00+01:00: {error}"))
}

/// `90s`, `30m`, `2h`, `1d`, or a sum such as `1h30m`.
pub fn parse_delay(text: &str) -> Result<Duration, String> {
    let wrong = || format!("in {text:?} is not a delay such as 90s, 30m, 2h, 1d or 1h30m");
    let mut total = Duration::zero();
    let mut digits = String::new();
    for c in text.trim().chars() {
        if c.is_ascii_digit() {
            digits.push(c);
            continue;
        }
        let amount: i64 = digits.parse().map_err(|_| wrong())?;
        digits.clear();
        let unit = match c {
            's' => Duration::try_seconds(amount),
            'm' => Duration::try_minutes(amount),
            'h' => Duration::try_hours(amount),
            'd' => Duration::try_days(amount),
            _ => None,
        };
        total = total.checked_add(&unit.ok_or_else(wrong)?).ok_or_else(wrong)?;
    }
    if !digits.is_empty() || total <= Duration::zero() {
        return Err(wrong());
    }
    Ok(total)
}

fn check_bounds(at: i64, now: i64) -> Result<(), String> {
    if at > now + Duration::days(MAX_AHEAD_DAYS).num_seconds() {
        return Err(format!("the time is more than {MAX_AHEAD_DAYS} days ahead"));
    }
    if at < now - Duration::hours(MAX_PAST_HOURS).num_seconds() {
        return Err(format!("the time is more than {MAX_PAST_HOURS} hours in the past"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::FixedOffset;

    use super::*;

    fn now() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2026-10-16T14:00:00+02:00").unwrap()
    }

    fn timing(at: Option<&str>, delay: Option<&str>, repeat: Option<&str>) -> Timing {
        Timing { at: at.map(str::to_owned), delay: delay.map(str::to_owned), repeat: repeat.map(str::to_owned) }
    }

    #[test]
    fn should_read_delays_and_their_sums() {
        // given
        let texts = ["90s", "30m", "2h", "1d", "1h30m"];
        // when
        let seconds: Vec<i64> = texts.iter().map(|text| parse_delay(text).unwrap().num_seconds()).collect();
        // then
        assert_eq!(seconds, [90, 1800, 7200, 86_400, 5400]);
    }

    #[test]
    fn should_refuse_delays_that_are_not_ones() {
        // given
        let wrong = ["", "2", "h", "2w", "0m", "-5m", "1h 30"];
        // when
        let refused: Vec<bool> = wrong.iter().map(|text| parse_delay(text).is_err()).collect();
        // then
        assert_eq!(refused, [true; 7]);
    }

    #[test]
    fn should_take_the_first_time_from_at_in_or_the_cron() {
        // given
        let base = now().timestamp();
        // when
        let at = first_time(&timing(Some("2026-10-16T16:00:00+02:00"), None, None), &now()).unwrap();
        let delay = first_time(&timing(None, Some("2h"), None), &now()).unwrap();
        let cron = first_time(&timing(None, None, Some("0 7 * * 1-5")), &now()).unwrap();
        // then
        assert_eq!((at.at - base, delay.at - base), (7200, 7200));
        assert_eq!(cron.at, DateTime::parse_from_rfc3339("2026-10-19T07:00:00+02:00").unwrap().timestamp());
        assert_eq!(cron.repeat.map(|cron| cron.text().to_owned()), Some("0 7 * * 1-5".to_owned()));
    }

    #[test]
    fn should_refuse_times_out_of_bounds() {
        // given
        let cases = [timing(Some("2026-10-16T16:00:00+02:00"), Some("2h"), None), timing(None, None, None), timing(None, Some("400d"), None), timing(Some("2026-10-14T14:00:00+02:00"), None, None), timing(Some("2026-10-16 16:00"), None, None)];
        // when
        let errors: Vec<String> = cases.iter().map(|case| first_time(case, &now()).unwrap_err()).collect();
        // then
        assert!(errors[0].contains("not both"));
        assert!(errors[1].contains("say when"));
        assert!(errors[2].contains("days ahead"));
        assert!(errors[3].contains("in the past"));
        assert!(errors[4].contains("RFC 3339"));
    }
}
