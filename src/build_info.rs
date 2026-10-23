//! What this IVE is: its version, the release it was built as, and when it was built.

use std::time::SystemTime;

use crate::age::{Clock, When, date_time, unix_seconds};

/// `IVE 0.1.0 (ive-0.1.0-bbec17d0a1)` for a release, `IVE 0.1.0 from source` otherwise.
pub fn version_text(version: &str, tag: Option<&str>) -> String {
    match tag {
        Some(tag) => format!("IVE {version} ({tag})"),
        None => format!("IVE {version} from source"),
    }
}

/// `built 2026-10-23 09:41 +0200` in the zone of `clock`.
pub fn built_text(built: Option<SystemTime>, clock: Clock) -> String {
    let Some(built) = built else {
        return "build time unknown".to_owned();
    };
    let offset_minutes = clock.when().offset_minutes;
    format!("built {}", date_time(When { seconds: unix_seconds(built), offset_minutes }))
}

/// When the running executable was written: linking writes it, and a release's archive keeps the time CI built it.
pub fn built_at() -> Option<SystemTime> {
    std::env::current_exe().and_then(std::fs::metadata).and_then(|metadata| metadata.modified()).ok()
}

/// This IVE in one line: `IVE 0.1.0 (ive-0.1.0-bbec17d0a1) · built 2026-10-23 09:41 +0200`.
pub fn summary() -> String {
    format!("{} · {}", version_text(env!("CARGO_PKG_VERSION"), crate::update::RELEASE_TAG), built_text(built_at(), Clock::system()))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::*;

    #[test]
    fn should_name_the_release_a_build_is() {
        // given
        let tag = Some("ive-0.1.0-bbec17d0a1");

        // when
        let text = version_text("0.1.0", tag);

        // then
        assert_eq!(text, "IVE 0.1.0 (ive-0.1.0-bbec17d0a1)");
    }

    #[test]
    fn should_say_a_build_is_from_source() {
        // given
        let tag = None;

        // when
        let text = version_text("0.1.0", tag);

        // then
        assert_eq!(text, "IVE 0.1.0 from source");
    }

    #[test]
    fn should_show_the_build_time_in_the_local_zone() {
        // given
        let built = UNIX_EPOCH + Duration::from_secs(1_792_741_260);
        let clock = Clock::new(1_792_750_000, 2 * 3600);

        // when
        let text = built_text(Some(built), clock);

        // then
        assert_eq!(text, "built 2026-10-23 09:41 +0200");
    }

    #[test]
    fn should_say_when_the_build_time_is_unknown() {
        // given
        let built = None;

        // when
        let text = built_text(built, Clock::new(0, 0));

        // then
        assert_eq!(text, "build time unknown");
    }
}
