//! A process's `OSC 9;4` progress as a small bar before its title, in the tab row and the tree.

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use gpui::{Div, ParentElement, Styled, div, px, rgb};

use crate::terminal::Progress;
use crate::theme::Theme;

/// How often an indeterminate bar's segment moves.
pub const FRAME: Duration = Duration::from_millis(100);
/// Positions the segment steps through one way; it then steps back.
const STEPS: u64 = 10;

/// Every indeterminate bar moves in step, however often the window redraws.
static EPOCH: LazyLock<Instant> = LazyLock::new(Instant::now);

/// `None` while the program reports no progress.
pub fn progress_bar(progress: Progress, theme: Theme) -> Option<Div> {
    let sizes = &theme.sizes.controls;
    let track = Track { width: sizes.progress_width.0, segment: sizes.progress_segment.0 };
    let (colour, left, width) = match progress {
        Progress::None => return None,
        Progress::Normal(percent) => (theme.mark_added, 0., track.fill(percent)),
        Progress::Error(percent) => (theme.error, 0., track.fill(percent)),
        Progress::Paused(percent) => (theme.muted, 0., track.fill(percent)),
        Progress::Indeterminate => (theme.mark_added, track.segment_left(frame()), track.segment),
    };
    let radius = px(sizes.progress_height.0 / 2.);
    let bar = div().absolute().top_0().left(px(left)).w(px(width)).h_full().rounded(radius).bg(rgb(colour));
    Some(div().relative().flex_none().w(sizes.progress_width.px()).h(sizes.progress_height.px()).rounded(radius).bg(rgb(theme.border)).child(bar))
}

/// The bar's length and an indeterminate bar's moving segment, in pixels.
#[derive(Clone, Copy)]
struct Track {
    width: f32,
    segment: f32,
}

impl Track {
    fn fill(self, percent: u8) -> f32 {
        self.width * f32::from(percent) / 100.
    }

    /// The segment's left edge in `frame`: it slides across the track and back.
    fn segment_left(self, frame: u64) -> f32 {
        let phase = frame % (2 * STEPS);
        let step = if phase < STEPS { phase } else { 2 * STEPS - phase };
        (self.width - self.segment) * step as f32 / STEPS as f32
    }
}

fn frame() -> u64 {
    (EPOCH.elapsed().as_millis() / FRAME.as_millis()) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default 16 px bar with its 6 px segment.
    const TRACK: Track = Track { width: 16., segment: 6. };

    #[test]
    fn should_slide_segment_across_the_track_and_back() {
        // given
        let frames = [0, 5, 10, 15, 20];

        // when
        let lefts = frames.map(|frame| TRACK.segment_left(frame));

        // then
        assert_eq!(lefts, [0., 5., 10., 5., 0.]);
    }

    #[test]
    fn should_fill_track_by_percent() {
        // given
        let percents = [0, 50, 100];

        // when
        let fills = percents.map(|percent| TRACK.fill(percent));

        // then
        assert_eq!(fills, [0., 8., 16.]);
    }
}
