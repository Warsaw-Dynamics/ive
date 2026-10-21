//! Date groups of scratches and the name filter; "now" and the time zone come from part 1's `age::Clock`.

use super::store::ScratchInfo;
use crate::age::{Clock, unix_seconds};

const DAY: i64 = 24 * 60 * 60;
const WEEK_DAYS: i64 = 7;
const MONTH_DAYS: i64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
    Today,
    ThisWeek,
    ThisMonth,
    Older,
}

impl Group {
    pub const ALL: [Self; 4] = [Self::Today, Self::ThisWeek, Self::ThisMonth, Self::Older];

    pub fn title(self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::ThisWeek => "This week",
            Self::ThisMonth => "This month",
            Self::Older => "Older",
        }
    }

    /// Relative to local midnight; anything from midnight on, the future included, is today.
    pub fn of(modified: i64, clock: Clock) -> Self {
        let midnight = clock.midnight();
        match modified {
            _ if modified >= midnight => Self::Today,
            _ if modified >= midnight - WEEK_DAYS * DAY => Self::ThisWeek,
            _ if modified >= midnight - MONTH_DAYS * DAY => Self::ThisMonth,
            _ => Self::Older,
        }
    }
}

/// Non-empty groups in display order, each newest first.
pub fn grouped<'a>(scratches: impl IntoIterator<Item = &'a ScratchInfo>, clock: Clock) -> Vec<(Group, Vec<&'a ScratchInfo>)> {
    let mut sorted: Vec<&ScratchInfo> = scratches.into_iter().collect();
    sorted.sort_by_key(|scratch| std::cmp::Reverse(scratch.modified));
    Group::ALL
        .into_iter()
        .map(|group| {
            let members: Vec<&ScratchInfo> =
                sorted.iter().copied().filter(|scratch| Group::of(unix_seconds(scratch.modified), clock) == group).collect();
            (group, members)
        })
        .filter(|(_, members)| !members.is_empty())
        .collect()
}

/// What the section lists: everything without a query; name matches while the content search runs; then its
/// result (`found`, names matching by name or content).
pub fn visible<'a>(scratches: &'a [ScratchInfo], query: &str, found: Option<&[String]>) -> Vec<&'a ScratchInfo> {
    match (query.is_empty(), found) {
        (true, _) => scratches.iter().collect(),
        (false, Some(found)) => scratches.iter().filter(|scratch| found.contains(&scratch.name)).collect(),
        (false, None) => scratches.iter().filter(|scratch| name_matches(&scratch.name, query)).collect(),
    }
}

/// Case-insensitive substring match.
pub fn name_matches(name: &str, query: &str) -> bool {
    name.to_lowercase().contains(&query.to_lowercase())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::*;
    use crate::paging::Paging;
    use Group::{Older, ThisMonth, ThisWeek, Today};

    /// 2026-10-16 12:00 UTC, 14:00 in Warsaw
    const NOW: i64 = 1_792_152_000;
    const WARSAW: i64 = 2 * 3600;
    /// 2026-10-16 00:00 in Warsaw
    const MIDNIGHT: i64 = 1_792_101_600;

    fn clock() -> Clock {
        Clock::new(NOW, WARSAW)
    }

    fn info(name: &str, modified: i64) -> ScratchInfo {
        ScratchInfo { name: name.to_owned(), modified: UNIX_EPOCH + Duration::from_secs(modified as u64), first_line: String::new() }
    }

    #[test]
    fn should_group_by_local_midnight_and_7_and_30_days_before_it() {
        // given
        let times = [MIDNIGHT, MIDNIGHT - 1, MIDNIGHT - 7 * DAY, MIDNIGHT - 7 * DAY - 1, MIDNIGHT - 30 * DAY, MIDNIGHT - 30 * DAY - 1];

        // when
        let groups = times.map(|time| Group::of(time, clock()));

        // then
        assert_eq!(groups, [Today, ThisWeek, ThisWeek, ThisMonth, ThisMonth, Older]);
    }

    #[test]
    fn should_use_the_local_day_not_the_utc_day() {
        // given
        let one_am_utc = 1_792_108_800 + 3600;
        let new_york = Clock::new(NOW, -4 * 3600);

        // when
        let groups = (Group::of(one_am_utc, Clock::new(NOW, 0)), Group::of(one_am_utc, new_york));

        // then
        assert_eq!(groups, (Today, ThisWeek));
    }

    #[test]
    fn should_put_future_times_in_today_as_now() {
        // given
        let tomorrow = NOW + DAY;

        // when
        let seen = (Group::of(tomorrow, clock()), clock().age(tomorrow));

        // then
        assert_eq!(seen, (Today, "now".to_owned()));
    }

    #[test]
    fn should_hide_empty_groups_and_sort_each_newest_first() {
        // given
        let scratches = [info("old", MIDNIGHT - 40 * DAY), info("a", MIDNIGHT + 10), info("b", MIDNIGHT + 20)];

        // when
        let groups: Vec<(Group, Vec<&str>)> = grouped(&scratches, clock())
            .into_iter()
            .map(|(group, members)| (group, members.iter().map(|scratch| scratch.name.as_str()).collect()))
            .collect();

        // then
        assert_eq!(groups, [(Today, vec!["b", "a"]), (Older, vec!["old"])]);
    }

    #[test]
    fn should_page_each_group_by_ten() {
        // given
        let scratches: Vec<ScratchInfo> = (0..25).map(|i| info(&format!("s{i}"), MIDNIGHT + i)).collect();
        let today = grouped(&scratches, clock())[0].1.len();
        let mut paging = Paging::default();

        // when
        let first = paging.visible(today);
        paging.more();
        let second = paging.visible(today);
        paging.more();
        let third = paging.visible(today);

        // then
        assert_eq!((today, first, second, third), (25, 10, 20, 25));
    }

    #[test]
    fn should_show_name_matches_until_the_content_search_answers() {
        // given
        let scratches = [info("idp-users.sql", NOW), info("notes.md", NOW)];
        let found = ["notes.md".to_owned()];

        // when
        let names = |list: Vec<&ScratchInfo>| list.iter().map(|scratch| scratch.name.clone()).collect::<Vec<_>>();
        let searching = names(visible(&scratches, "USERS", None));
        let answered = names(visible(&scratches, "USERS", Some(&found)));
        let cleared = names(visible(&scratches, "", Some(&found)));

        // then
        assert_eq!(searching, ["idp-users.sql"]);
        assert_eq!(answered, ["notes.md"]);
        assert_eq!(cleared, ["idp-users.sql", "notes.md"]);
    }
}
