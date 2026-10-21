//! The left column: the "Projects" header and the tree of projects, worktrees, processes and files.

pub mod commit_label;
pub mod folders;
pub mod rows;
pub mod selection;
pub mod states;
pub mod view;

const MIN_WIDTH: f32 = 160.;
/// The editor keeps at least this much of the window, however wide the tree is dragged.
const MIN_EDITOR_WIDTH: f32 = 320.;

/// A width both the tree and the editor stay usable with in a window `window_width` wide; the saved width can
/// be wider than a smaller window allows.
pub fn fitted_width(width: f32, window_width: f32) -> f32 {
    let widest = (window_width - MIN_EDITOR_WIDTH).max(MIN_WIDTH);
    width.clamp(MIN_WIDTH, widest)
}

/// A drag of the tree's right edge, from where the mouse went down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resize {
    start_x: f32,
    start_width: f32,
}

impl Resize {
    pub fn start(x: f32, width: f32) -> Self {
        Self { start_x: x, start_width: width }
    }

    /// The tree width with the mouse at `x`: the edge follows the mouse, within [`fitted_width`].
    pub fn width_at(self, x: f32, window_width: f32) -> f32 {
        fitted_width(self.start_width + x - self.start_x, window_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_move_edge_by_mouse_distance() {
        // given
        let resize = Resize::start(300., 288.);

        // when
        let widths = [resize.width_at(350., 1200.), resize.width_at(250., 1200.)];

        // then
        assert_eq!(widths, [338., 238.]);
    }

    #[test]
    fn should_keep_tree_and_editor_usable_at_the_extremes() {
        // given
        let resize = Resize::start(300., 288.);

        // when
        let widths = [resize.width_at(-500., 1200.), resize.width_at(5000., 1200.)];

        // then
        assert_eq!(widths, [160., 880.]);
    }

    #[test]
    fn should_shrink_saved_width_to_a_narrow_window_but_not_below_minimum() {
        // given
        let saved = 700.;

        // when
        let widths = [fitted_width(saved, 800.), fitted_width(saved, 300.)];

        // then
        assert_eq!(widths, [480., 160.]);
    }
}
