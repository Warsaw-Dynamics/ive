//! Where the side-by-side view splits its width between the old panel, on the left, and the new one: a fraction of
//! the view's width, which the divider between the panels drags. Each view keeps its own; it is not saved.

use std::ops::Range;

use gpui::{Bounds, Pixels};

use super::side_by_side::Side;

/// The split of a new view, and the one a double-click on the divider goes back to: halves.
const EVEN: f32 = 0.5;
/// The least share of the view's width a panel keeps, so neither is dragged out of sight.
const MIN_SHARE: f32 = 0.1;

/// The share of the view's width the old panel takes; the new panel takes the rest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Split {
    old_share: f32,
}

impl Default for Split {
    fn default() -> Self {
        Self { old_share: EVEN }
    }
}

impl Split {
    /// The old panel taking `old_share` of the width, kept between [`MIN_SHARE`] and `1 - MIN_SHARE`; a share that
    /// is not a number keeps halves.
    pub(crate) fn of(old_share: f32) -> Self {
        if old_share.is_nan() {
            return Self::default();
        }
        Self { old_share: old_share.clamp(MIN_SHARE, 1. - MIN_SHARE) }
    }

    /// The split that puts the divider at window x `x` in a view at `bounds`.
    pub(crate) fn at(x: Pixels, bounds: Bounds<Pixels>) -> Self {
        Self::of((x - bounds.left()) / bounds.size.width)
    }

    /// The share of the width panel `side` takes.
    pub(crate) fn share(self, side: Side) -> f32 {
        match side {
            Side::Old => self.old_share,
            Side::New => 1. - self.old_share,
        }
    }

    /// The window x of the divider in a view at `bounds`.
    fn divider(self, bounds: Bounds<Pixels>) -> Pixels {
        bounds.left() + bounds.size.width * self.old_share
    }

    /// The window's horizontal range of panel `side` in a view at `bounds`.
    pub(crate) fn span(self, side: Side, bounds: Bounds<Pixels>) -> Range<Pixels> {
        match side {
            Side::Old => bounds.left()..self.divider(bounds),
            Side::New => self.divider(bounds)..bounds.right(),
        }
    }

    /// The panel under window x `x` in a view at `bounds`.
    pub(crate) fn side_at(self, x: Pixels, bounds: Bounds<Pixels>) -> Side {
        if x < self.divider(bounds) { Side::Old } else { Side::New }
    }
}

#[cfg(test)]
mod tests {
    use gpui::{point, px, size};

    use super::*;

    fn view() -> Bounds<Pixels> {
        Bounds::new(point(px(100.), px(0.)), size(px(1000.), px(500.)))
    }

    #[test]
    fn should_split_a_new_view_in_halves() {
        // given
        let split = Split::default();

        // when
        let spans = (split.span(Side::Old, view()), split.span(Side::New, view()));

        // then
        assert_eq!(spans, (px(100.)..px(600.), px(600.)..px(1100.)));
    }

    #[test]
    fn should_keep_each_panel_a_tenth_of_the_width_at_least() {
        // given
        let shares = [-1., 0., 0.05, 0.3, 0.95, 1., 2., f32::NAN];

        // when
        let split: Vec<f32> = shares.into_iter().map(|share| Split::of(share).share(Side::Old)).collect();

        // then
        assert_eq!(split, [0.1, 0.1, 0.1, 0.3, 0.9, 0.9, 0.9, 0.5]);
    }

    #[test]
    fn should_put_the_divider_where_it_was_dragged() {
        // given
        let x = px(400.);

        // when
        let split = Split::at(x, view());

        // then
        assert_eq!((split.share(Side::Old), split.share(Side::New)), (0.3, 0.7));
    }

    #[test]
    fn should_find_the_right_panel_past_a_divider_moved_left() {
        // given
        let split = Split::of(0.3);

        // when
        let sides = [px(399.), px(400.), px(550.)].map(|x| split.side_at(x, view()));

        // then
        assert_eq!(sides, [Side::Old, Side::New, Side::New]);
        assert_eq!(split.span(Side::New, view()), px(400.)..px(1100.));
    }
}
