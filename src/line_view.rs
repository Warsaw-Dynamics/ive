//! Scrolling and font metrics of line-based views: the document view and the scratch editor.

use std::cell::Cell;
use std::rc::Rc;

use gpui::{App, Bounds, IntoElement, Pixels, ScrollWheelEvent, Styled, Window, canvas};

use crate::saved_tabs::ScrollPosition;
use crate::scrollbar::{Axis, Extent, Scrollable, wheel_axes};
use crate::theme::{self, Theme};

/// Lines drawn before the first layout reports the real viewport height.
const FIRST_FRAME_LINES: f64 = 120.;
/// Characters of space after the longest visible line, so its end is not glued to the scrollbar.
const RIGHT_PADDING_CHARS: usize = 4;

/// Where a line view is scrolled and how big its viewport and content are; shared with the scrollbars.
/// It does its own scrolling instead of using GPUI's list: with 10M lines the list's f32 pixel positions
/// (200M px) are only accurate to 16 px; here the position is a line number plus a fraction.
pub struct LineScroll {
    line_count: Cell<usize>,
    /// first visible line, with the fraction of it scrolled past
    top: Cell<f64>,
    /// pixels scrolled to the right
    left: Cell<f64>,
    /// `None` until the first layout
    viewport_lines: Cell<Option<f64>>,
    viewport_width: Cell<f64>,
    content_width: Cell<f64>,
}

impl LineScroll {
    pub fn new(line_count: usize) -> Self {
        Self {
            line_count: Cell::new(line_count),
            top: Cell::new(0.),
            left: Cell::new(0.),
            viewport_lines: Cell::new(None),
            viewport_width: Cell::new(0.),
            content_width: Cell::new(0.),
        }
    }

    pub fn top(&self) -> f64 {
        self.top.get()
    }

    pub fn left(&self) -> f64 {
        self.left.get()
    }

    pub fn viewport_lines(&self) -> f64 {
        self.viewport_lines.get().unwrap_or(FIRST_FRAME_LINES)
    }

    pub fn viewport_width(&self) -> f64 {
        self.viewport_width.get()
    }

    pub fn position(&self) -> ScrollPosition {
        ScrollPosition { top_line: self.top.get(), left: self.left.get() }
    }

    /// Scrolls to a saved position. Before the first layout the viewport is unknown, so the top line is only kept
    /// inside the text here and settles once the viewport is measured; the next frame's widest line bounds `left`.
    pub fn set_position(&self, position: ScrollPosition) {
        self.left.set(position.left.max(0.));
        match self.viewport_lines.get() {
            Some(_) => self.scroll_to(Axis::Vertical, position.top_line),
            None => self.top.set(position.top_line.clamp(0., self.line_count.get().saturating_sub(1) as f64)),
        }
    }

    /// The viewport's height after layout; a taller viewport may leave room below the last line, so the top moves
    /// up to fill it.
    fn viewport_measured(&self, lines: f64) {
        self.viewport_lines.set(Some(lines));
        self.scroll_to(Axis::Vertical, self.top.get());
    }

    /// For views whose text changes; keeps the position valid.
    pub fn set_line_count(&self, line_count: usize) {
        self.line_count.set(line_count);
        self.scroll_to(Axis::Vertical, self.top.get());
    }

    fn max_top(&self) -> f64 {
        (self.line_count.get() as f64 - self.viewport_lines()).max(0.)
    }

    fn max_left(&self) -> f64 {
        (self.content_width.get() - self.viewport_width.get()).max(0.)
    }

    /// Widest line on screen; when it fits, horizontal scrolling resets, so the scrollbar shows only while a
    /// visible line overflows.
    pub fn set_content_width(&self, width: f64) {
        self.content_width.set(width);
        self.left.set(self.left.get().min(self.max_left()));
    }

    pub fn scroll_by_wheel(&self, event: &ScrollWheelEvent, line_height: Pixels, window: &mut Window) {
        let delta = event.delta.pixel_delta(line_height);
        let (dx, dy) = wheel_axes(f32::from(delta.x), f32::from(delta.y), event.modifiers.shift);
        self.scroll_to(Axis::Vertical, self.top.get() - f64::from(dy / f32::from(line_height)));
        self.scroll_to(Axis::Horizontal, self.left.get() - f64::from(dx));
        window.refresh();
    }

    /// Scrolls the least so that `line` is fully visible.
    pub fn reveal_line(&self, line: usize) {
        let (line, top, viewport) = (line as f64, self.top.get(), self.viewport_lines());
        if line < top {
            self.scroll_to(Axis::Vertical, line);
        } else if line + 1. > top + viewport {
            self.scroll_to(Axis::Vertical, line + 1. - viewport);
        }
    }

    /// Scrolls the least so that `x` (pixels from the text's start) is at least `margin` inside the viewport.
    pub fn reveal_x(&self, x: f64, margin: f64) {
        let (left, width) = (self.left.get(), self.viewport_width.get());
        // before the first layout the width is unknown, and any position would look out of view
        if width <= 0. {
            return;
        }
        if x < left + margin {
            self.scroll_to(Axis::Horizontal, x - margin);
        } else if x > left + width - margin {
            self.scroll_to(Axis::Horizontal, x - width + margin);
        }
    }
}

impl Scrollable for LineScroll {
    fn extent(&self, axis: Axis) -> Extent {
        match axis {
            Axis::Vertical => {
                Extent { viewport: self.viewport_lines(), max_scroll: self.max_top(), scrolled: self.top.get() }
            }
            Axis::Horizontal => {
                Extent { viewport: self.viewport_width.get(), max_scroll: self.max_left(), scrolled: self.left.get() }
            }
        }
    }

    fn scroll_to(&self, axis: Axis, scrolled: f64) {
        match axis {
            Axis::Vertical => self.top.set(scrolled.clamp(0., self.max_top())),
            Axis::Horizontal => self.left.set(scrolled.clamp(0., self.max_left())),
        }
    }
}

/// Records the viewport size after layout; redraws when it changed so the next frame fetches enough lines.
pub fn measure_viewport(scroll: Rc<LineScroll>, metrics: Metrics) -> impl IntoElement {
    canvas(
        move |bounds: Bounds<Pixels>, window, _cx| {
            let lines = f64::from(f32::from(bounds.size.height) / f32::from(metrics.line_height));
            let text_width = f64::from(f32::from(bounds.size.width - metrics.gutter_width()));
            let changed = scroll.viewport_lines.get() != Some(lines) || scroll.viewport_width.get() != text_width;
            scroll.viewport_measured(lines);
            scroll.viewport_width.set(text_width);
            if changed {
                scroll.set_content_width(scroll.content_width.get());
                window.refresh();
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full()
}

/// Sizes derived from the monospace font at the views' text size.
#[derive(Clone, Copy)]
pub struct Metrics {
    pub font_size: Pixels,
    pub line_height: Pixels,
    pub char_width: Pixels,
    pub number_chars: usize,
}

impl Metrics {
    pub fn measure(window: &Window, cx: &App, line_count: usize) -> Self {
        let text_system = window.text_system();
        let font_id = text_system.resolve_font(&gpui::font(theme::mono_font(cx)));
        let sizes = &cx.global::<Theme>().sizes.editor;
        let font_size = sizes.text_size.px();
        let char_width = text_system.ch_width(font_id, font_size).unwrap_or(font_size * 0.6);
        Self {
            font_size,
            line_height: (font_size * sizes.line_height_factor.0).round(),
            char_width,
            number_chars: line_count.to_string().len(),
        }
    }

    /// Line numbers plus one character of padding on each side.
    pub fn gutter_width(&self) -> Pixels {
        self.char_width * (self.number_chars + 2) as f32
    }

    pub fn text_width(&self, chars: usize) -> f64 {
        f64::from(f32::from(self.char_width * (chars + RIGHT_PADDING_CHARS) as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn laid_out(line_count: usize, viewport_lines: f64) -> LineScroll {
        let scroll = LineScroll::new(line_count);
        scroll.viewport_lines.set(Some(viewport_lines));
        scroll
    }

    #[test]
    fn should_clamp_a_restored_scroll_to_a_shorter_file() {
        // given
        let scroll = LineScroll::new(100);

        // when
        scroll.set_position(ScrollPosition { top_line: 500., left: -3. });

        // then
        assert_eq!(scroll.position(), ScrollPosition { top_line: 99., left: 0. });
    }

    #[test]
    fn should_keep_the_last_line_at_the_bottom_once_the_viewport_is_known() {
        // given
        let scroll = LineScroll::new(100);
        scroll.set_position(ScrollPosition { top_line: 90., left: 0. });

        // when
        scroll.viewport_measured(40.);

        // then
        assert_eq!(scroll.top(), 60.);
    }

    #[test]
    fn should_stop_scrolling_with_last_line_at_bottom_of_viewport() {
        // given
        let scroll = laid_out(10_000_000, 40.);

        // when
        scroll.scroll_to(Axis::Vertical, f64::MAX);

        // then
        assert_eq!(scroll.top(), 9_999_960.);
    }

    #[test]
    fn should_keep_exact_fractional_position_deep_in_huge_document() {
        // given
        let scroll = laid_out(10_000_000, 40.);
        scroll.scroll_to(Axis::Vertical, 9_000_000.);

        // when
        scroll.scroll_to(Axis::Vertical, scroll.top() + 0.25);

        // then
        assert_eq!(scroll.top(), 9_000_000.25);
    }

    #[test]
    fn should_reset_horizontal_scroll_when_visible_lines_fit() {
        // given
        let scroll = laid_out(100, 40.);
        scroll.viewport_width.set(500.);
        scroll.set_content_width(2000.);
        scroll.scroll_to(Axis::Horizontal, 900.);

        // when
        scroll.set_content_width(300.);

        // then
        assert_eq!((scroll.left(), scroll.extent(Axis::Horizontal).max_scroll), (0., 0.));
    }

    #[test]
    fn should_keep_the_top_line_valid_when_the_text_shrinks() {
        // given
        let scroll = laid_out(1000, 40.);
        scroll.scroll_to(Axis::Vertical, 900.);

        // when
        scroll.set_line_count(100);

        // then
        assert_eq!(scroll.top(), 60.);
    }

    #[test]
    fn should_scroll_least_to_reveal_a_line_below_or_above_the_viewport() {
        // given
        let scroll = laid_out(1000, 40.);
        scroll.scroll_to(Axis::Vertical, 100.);

        // when
        scroll.reveal_line(150);
        let below = scroll.top();
        scroll.reveal_line(20);
        let above = scroll.top();

        // then
        assert_eq!((below, above), (111., 20.));
    }

    #[test]
    fn should_scroll_sideways_to_keep_x_inside_the_viewport_with_a_margin() {
        // given
        let scroll = laid_out(10, 40.);
        scroll.viewport_width.set(500.);
        scroll.set_content_width(2000.);

        // when
        scroll.reveal_x(800., 20.);
        let right = scroll.left();
        scroll.reveal_x(100., 20.);
        let left = scroll.left();

        // then
        assert_eq!((right, left), (320., 80.));
    }

    #[test]
    fn should_not_scroll_sideways_before_the_first_layout() {
        // given
        let scroll = laid_out(10, 40.);
        scroll.set_content_width(2000.);

        // when
        scroll.reveal_x(800., 20.);

        // then
        assert_eq!(scroll.left(), 0.);
    }
}
