use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    Bounds, DispatchPhase, Hitbox, HitboxBehavior, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Point, ScrollHandle, ScrollWheelEvent, Styled, UniformListScrollHandle, Window, canvas,
    fill, point, px, rgba,
};

use crate::style::ViewStyle;

/// Rows assumed visible before the list's first layout tells us its real size.
const FIRST_FRAME_ROWS: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Vertical,
    Horizontal,
}

impl Axis {
    fn of_point(self, point: Point<Pixels>) -> f32 {
        f32::from(match self {
            Self::Vertical => point.y,
            Self::Horizontal => point.x,
        })
    }

    fn of_bounds(self, bounds: Bounds<Pixels>) -> f32 {
        f32::from(match self {
            Self::Vertical => bounds.size.height,
            Self::Horizontal => bounds.size.width,
        })
    }
}

/// How much of the content is visible and how far it is scrolled along one axis, in the content's own unit
/// (pixels for lists, lines for the editor); a scrollbar only needs the ratios.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extent {
    pub viewport: f64,
    pub max_scroll: f64,
    pub scrolled: f64,
}

/// Something a scrollbar can show and move.
pub trait Scrollable {
    fn extent(&self, axis: Axis) -> Extent;
    /// Scrolls to `scrolled` (same unit as `extent`), clamped to the valid range.
    fn scroll_to(&self, axis: Axis, scrolled: f64);
}

/// What a pair of scrollbars moves, plus their drag state; clone it into each scrollbar.
#[derive(Clone)]
pub struct ScrollState {
    target: Rc<dyn Scrollable>,
    /// set for GPUI lists, which scroll themselves; the horizontal bar then adds Shift+wheel
    list: Option<ScrollHandle>,
    /// While a thumb is dragged: where on the thumb it was grabbed, from the thumb's start.
    grab_vertical: Rc<Cell<Option<f32>>>,
    grab_horizontal: Rc<Cell<Option<f32>>>,
}

impl ScrollState {
    pub fn new(target: Rc<dyn Scrollable>) -> Self {
        Self { target, list: None, grab_vertical: Rc::default(), grab_horizontal: Rc::default() }
    }

    pub fn for_list(list: &ListScroll) -> Self {
        Self { list: Some(list.handle()), ..Self::new(Rc::new(list.clone())) }
    }

    fn grab(&self, axis: Axis) -> &Rc<Cell<Option<f32>>> {
        match axis {
            Axis::Vertical => &self.grab_vertical,
            Axis::Horizontal => &self.grab_horizontal,
        }
    }
}

/// Scrolling of a GPUI `uniform_list`, in pixels.
#[derive(Clone)]
pub struct ListScroll {
    list: UniformListScrollHandle,
}

impl Default for ListScroll {
    fn default() -> Self {
        Self::new()
    }
}

impl ListScroll {
    pub fn new() -> Self {
        Self { list: UniformListScrollHandle::new() }
    }

    pub fn list_handle(&self) -> UniformListScrollHandle {
        self.list.clone()
    }

    /// The rows of a `row_count` list that the last layout showed (a first screenful before any layout).
    pub fn visible_rows(&self, row_count: usize) -> Range<usize> {
        let sizes = self.list.0.borrow().last_item_size;
        let Some(sizes) = sizes.filter(|_| row_count > 0) else {
            return 0..row_count.min(FIRST_FRAME_ROWS);
        };
        let row_height = f32::from(sizes.contents.height) / row_count as f32;
        let scrolled = -f32::from(self.handle().offset().y);
        rows_in_view(row_count, row_height, scrolled, f32::from(sizes.item.height))
    }

    /// Scrolls to `offset` (negative, as gpui's), unclamped: before the first layout the list's size is unknown, and
    /// its next layout keeps the offset inside the rows.
    pub fn set_offset(&self, offset: Point<Pixels>) {
        self.handle().set_offset(offset);
    }

    fn handle(&self) -> ScrollHandle {
        self.list.0.borrow().base_handle.clone()
    }
}

impl Scrollable for ListScroll {
    fn extent(&self, axis: Axis) -> Extent {
        let handle = self.handle();
        let (viewport, max_scroll, offset) = match axis {
            Axis::Vertical => (handle.bounds().size.height, handle.max_offset().y, handle.offset().y),
            Axis::Horizontal => (handle.bounds().size.width, handle.max_offset().x, handle.offset().x),
        };
        Extent { viewport: f32::from(viewport).into(), max_scroll: f32::from(max_scroll).into(), scrolled: -f64::from(f32::from(offset)) }
    }

    fn scroll_to(&self, axis: Axis, scrolled: f64) {
        let handle = self.handle();
        let max_scroll = f64::from(f32::from(match axis {
            Axis::Vertical => handle.max_offset().y,
            Axis::Horizontal => handle.max_offset().x,
        }));
        let to = px(-(scrolled.clamp(0., max_scroll.max(0.)) as f32));
        let offset = handle.offset();
        handle.set_offset(match axis {
            Axis::Vertical => point(offset.x, to),
            Axis::Horizontal => point(to, offset.y),
        });
    }
}

/// Scrolling along one axis that a view applies itself, by offsetting what it draws; in pixels.
#[derive(Clone, Default)]
pub struct OffsetScroll {
    scrolled: Rc<Cell<f64>>,
    /// the viewport's and the content's length, as of the last frame
    size: Rc<Cell<(f64, f64)>>,
}

impl OffsetScroll {
    pub fn scrolled(&self) -> f64 {
        self.scrolled.get()
    }

    /// Sets the viewport's and the content's length, keeping the scroll inside them.
    pub fn set_size(&self, viewport: f64, content: f64) {
        self.size.set((viewport, content));
        self.scroll_to(Axis::Horizontal, self.scrolled());
    }

    fn max_scroll(&self) -> f64 {
        let (viewport, content) = self.size.get();
        (content - viewport).max(0.)
    }
}

impl Scrollable for OffsetScroll {
    fn extent(&self, _axis: Axis) -> Extent {
        Extent { viewport: self.size.get().0, max_scroll: self.max_scroll(), scrolled: self.scrolled() }
    }

    fn scroll_to(&self, _axis: Axis, scrolled: f64) {
        self.scrolled.set(scrolled.clamp(0., self.max_scroll()));
    }
}

/// The row in `rows` with the largest `width`: the one a list should measure to size its scrollable width.
pub fn widest_row(rows: Range<usize>, width: impl Fn(usize) -> usize) -> Option<usize> {
    rows.max_by_key(|&row| width(row))
}

fn rows_in_view(row_count: usize, row_height: f32, scrolled: f32, viewport: f32) -> Range<usize> {
    if row_height <= 0. {
        return 0..row_count.min(FIRST_FRAME_ROWS);
    }
    let first = ((scrolled / row_height).floor().max(0.) as usize).min(row_count);
    let last = (((scrolled + viewport) / row_height).ceil().max(0.) as usize).min(row_count);
    first..last
}

/// Thumb position and length along the track, in pixels from the track's start.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Thumb {
    start: f32,
    length: f32,
}

/// `None` when everything fits and no scrollbar is needed.
fn thumb(track: f32, extent: Extent, min_thumb: f32) -> Option<Thumb> {
    if extent.max_scroll <= 0. || track <= 0. {
        return None;
    }
    let visible_fraction = extent.viewport / (extent.viewport + extent.max_scroll);
    let length = (track * visible_fraction as f32).clamp(min_thumb.min(track), track);
    let start = (track - length) * (extent.scrolled / extent.max_scroll).clamp(0., 1.) as f32;
    Some(Thumb { start, length })
}

/// How far to scroll, in the content's unit, so the thumb starts at `thumb_start`.
fn scroll_for(thumb_start: f32, thumb: Thumb, track: f32, max_scroll: f64) -> f64 {
    let travel = track - thumb.length;
    if travel <= 0. {
        return 0.;
    }
    f64::from((thumb_start / travel).clamp(0., 1.)) * max_scroll
}

/// A mark on a vertical scrollbar's track: where a find match is, as a share of the content from its top.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    pub at: f32,
    pub current: bool,
}

/// The ticks' rows on a `track` pixels long track: one per pixel row, the current one winning.
pub(crate) fn tick_rows(ticks: &[Tick], track: f32) -> Vec<(f32, bool)> {
    let mut rows: Vec<(f32, bool)> = Vec::new();
    for tick in ticks {
        let row = (tick.at.clamp(0., 1.) * track).floor();
        match rows.last_mut() {
            Some((last, current)) if *last == row => *current |= tick.current,
            _ => rows.push((row, tick.current)),
        }
    }
    rows
}

/// A vertical scrollbar overlaid on the right edge of its parent, which must be `relative()`.
pub fn vertical_scrollbar(state: &ScrollState, style: &ViewStyle) -> impl IntoElement {
    vertical_scrollbar_with_ticks(state, style, Rc::from([]))
}

/// A vertical scrollbar with `ticks` (in content order) on its track, under the thumb.
pub fn vertical_scrollbar_with_ticks(state: &ScrollState, style: &ViewStyle, ticks: Rc<[Tick]>) -> impl IntoElement {
    scrollbar(state, Bar { axis: Axis::Vertical, ticks }, style).top_0().right_0().h_full().w(px(style.scrollbar_thickness))
}

/// A horizontal scrollbar overlaid on the bottom edge of its parent, which must be `relative()`.
pub fn horizontal_scrollbar(state: &ScrollState, style: &ViewStyle) -> impl IntoElement {
    // stops short of the right edge so it never covers the vertical scrollbar's thumb
    let thickness = px(style.scrollbar_thickness);
    scrollbar(state, Bar { axis: Axis::Horizontal, ticks: Rc::from([]) }, style).bottom_0().left_0().right(thickness).h(thickness)
}

/// Which scrollbar, and the ticks on its track.
struct Bar {
    axis: Axis,
    ticks: Rc<[Tick]>,
}

fn scrollbar(state: &ScrollState, bar: Bar, style: &ViewStyle) -> gpui::Canvas<Hitbox> {
    let (state, style) = (state.clone(), style.clone());
    canvas(
        |bounds, window, _cx| window.insert_hitbox(bounds, HitboxBehavior::BlockMouseExceptScroll),
        move |bounds, hitbox, window, _cx| {
            paint_ticks(&bar.ticks, bounds, &style, window);
            paint(&state, bar.axis, Track { bounds, hitbox, style: style.clone() }, window);
        },
    )
    .absolute()
}

/// Tick height, in pixels.
const TICK: f32 = 2.;

/// A tick's colour: the match colours of the text, opaque enough to see on the track.
fn tick_colour(style: &ViewStyle, (_, current): (f32, bool)) -> u32 {
    if current { style.find_current | 0xff } else { (style.find_match & !0xff) | 0xcc }
}

/// The find box's matches on a vertical track: the current one solid, the others fainter.
fn paint_ticks(ticks: &[Tick], track: Bounds<Pixels>, style: &ViewStyle, window: &mut Window) {
    let across = px(style.scrollbar_thickness - 2. * style.scrollbar_inset);
    let room = (f32::from(track.size.height) - TICK).max(0.);
    for (row, current) in tick_rows(ticks, room) {
        let colour = tick_colour(style, (row, current));
        let at = track.origin + point(px(style.scrollbar_inset), px(row));
        window.paint_quad(fill(Bounds::new(at, gpui::size(across, px(TICK))), rgba(colour)));
    }
}

struct Track {
    bounds: Bounds<Pixels>,
    hitbox: Hitbox,
    style: ViewStyle,
}

fn paint(state: &ScrollState, axis: Axis, track: Track, window: &mut Window) {
    if let (Axis::Horizontal, Some(list)) = (axis, &state.list) {
        listen_for_shift_wheel(state, list.bounds(), window);
    }
    let extent = state.target.extent(axis);
    let style = &track.style;
    let Some(thumb) = thumb(axis.of_bounds(track.bounds), extent, style.scrollbar_min_thumb) else {
        return;
    };
    let active = state.grab(axis).get().is_some() || track.hitbox.is_hovered(window);
    let color = if active { style.scrollbar_thumb_hover } else { style.scrollbar_thumb };
    let (origin, inset, along) = (track.bounds.origin, px(style.scrollbar_inset), px(thumb.start));
    // the thumb fills the track's thickness but for the inset on both sides, with fully round ends
    let across = px(style.scrollbar_thickness - 2. * style.scrollbar_inset);
    let thumb_bounds = match axis {
        Axis::Vertical => Bounds::new(origin + point(inset, along), gpui::size(across, px(thumb.length))),
        Axis::Horizontal => Bounds::new(origin + point(along, inset), gpui::size(px(thumb.length), across)),
    };
    window.paint_quad(fill(thumb_bounds, rgba(color)).corner_radii(across / 2.));
    listen_for_drag(state, axis, track.bounds, Drag { thumb, max_scroll: extent.max_scroll }, window);
}

#[derive(Clone, Copy)]
struct Drag {
    thumb: Thumb,
    max_scroll: f64,
}

fn listen_for_drag(state: &ScrollState, axis: Axis, track: Bounds<Pixels>, drag: Drag, window: &mut Window) {
    let track_start = axis.of_point(track.origin);
    let track_length = axis.of_bounds(track);
    let (grab, target) = (state.grab(axis).clone(), state.target.clone());
    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
        let pressed_on_track =
            phase == DispatchPhase::Bubble && event.button == MouseButton::Left && track.contains(&event.position);
        if pressed_on_track {
            let at = axis.of_point(event.position) - track_start;
            let on_thumb = (drag.thumb.start..drag.thumb.start + drag.thumb.length).contains(&at);
            // a click on the track centres the thumb there, then drags from its middle
            let grabbed_at = if on_thumb { at - drag.thumb.start } else { drag.thumb.length / 2. };
            grab.set(Some(grabbed_at));
            target.scroll_to(axis, scroll_for(at - grabbed_at, drag.thumb, track_length, drag.max_scroll));
            cx.stop_propagation();
            window.refresh();
        }
    });
    let (grab, target) = (state.grab(axis).clone(), state.target.clone());
    window.on_mouse_event(move |event: &MouseMoveEvent, _phase, window, _cx| {
        if let Some(grabbed_at) = grab.get() {
            let thumb_start = axis.of_point(event.position) - track_start - grabbed_at;
            target.scroll_to(axis, scroll_for(thumb_start, drag.thumb, track_length, drag.max_scroll));
            window.refresh();
        }
    });
    let grab = state.grab(axis).clone();
    window.on_mouse_event(move |_: &MouseUpEvent, _phase, window, _cx| {
        if grab.take().is_some() {
            window.refresh();
        }
    });
}

/// X11, Windows and macOS already deliver Shift+wheel as horizontal; this covers the rest (e.g. Wayland).
fn listen_for_shift_wheel(state: &ScrollState, list_bounds: Bounds<Pixels>, window: &mut Window) {
    let target = state.target.clone();
    window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
        let delta = event.delta.pixel_delta(window.line_height());
        let (horizontal, _) = wheel_axes(f32::from(delta.x), f32::from(delta.y), event.modifiers.shift);
        let converted = delta.x == px(0.) && horizontal != 0.;
        if phase == DispatchPhase::Capture && converted && list_bounds.contains(&event.position) {
            let scrolled = target.extent(Axis::Horizontal).scrolled - f64::from(horizontal);
            target.scroll_to(Axis::Horizontal, scrolled);
            // keep the list from also scrolling vertically
            cx.stop_propagation();
            window.refresh();
        }
    });
}

/// Splits a wheel delta into (horizontal, vertical); with Shift held a purely vertical wheel scrolls sideways.
pub fn wheel_axes(dx: f32, dy: f32, shift: bool) -> (f32, f32) {
    if shift && dx == 0. {
        return (dy, 0.);
    }
    (dx, dy)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default minimum thumb length.
    const MIN_THUMB: f32 = 24.;

    fn extent(viewport: f64, max_scroll: f64, scrolled: f64) -> Extent {
        Extent { viewport, max_scroll, scrolled }
    }

    #[test]
    fn should_size_thumb_to_visible_fraction_and_place_it_by_scroll_position() {
        // given
        let track = 400.;

        // when
        let at_start = thumb(track, extent(400., 1200., 0.), MIN_THUMB).unwrap();
        let halfway = thumb(track, extent(400., 1200., 600.), MIN_THUMB).unwrap();

        // then
        assert_eq!(at_start, Thumb { start: 0., length: 100. });
        assert_eq!(halfway, Thumb { start: 150., length: 100. });
    }

    #[test]
    fn should_draw_close_ticks_once_and_keep_the_current_one() {
        // given
        let ticks = [Tick { at: 0.1, current: false }, Tick { at: 0.1001, current: true }, Tick { at: 0.5, current: false }];

        // when
        let rows = tick_rows(&ticks, 100.);

        // then
        assert_eq!(rows, [(10., true), (50., false)]);
    }

    #[test]
    fn should_colour_ticks_as_the_matches_in_the_text_only_more_opaque() {
        // given
        let style = ViewStyle { find_match: 0xffc43c47, find_current: 0xe8a33dcc, ..ViewStyle::for_test() };

        // when
        let colours = [(10., false), (20., true)].map(|row| tick_colour(&style, row));

        // then
        assert_eq!(colours, [0xffc43ccc, 0xe8a33dff]);
    }

    #[test]
    fn should_hide_scrollbar_when_content_fits() {
        // given
        let fits = extent(400., 0., 0.);

        // when
        let result = thumb(400., fits, MIN_THUMB);

        // then
        assert_eq!(result, None);
    }

    #[test]
    fn should_keep_thumb_grabbable_and_exact_for_ten_million_lines() {
        // given
        let lines = extent(50., 10_000_000. - 50., 10_000_000. - 50.);

        // when
        let result = thumb(400., lines, MIN_THUMB).unwrap();

        // then
        assert_eq!(result, Thumb { start: 400. - MIN_THUMB, length: MIN_THUMB });
    }

    #[test]
    fn should_map_thumb_position_back_to_scroll_and_clamp_at_ends() {
        // given
        let thumb = Thumb { start: 0., length: 100. };

        // when
        let positions = [-50., 150., 300., 999.].map(|start| scroll_for(start, thumb, 400., 1200.));

        // then
        assert_eq!(positions, [0., 600., 1200., 1200.]);
    }

    #[test]
    fn should_list_rows_in_view_including_partly_visible_ones() {
        // given
        let (row_count, row_height, viewport) = (1000, 20., 100.);

        // when
        let at_top = rows_in_view(row_count, row_height, 0., viewport);
        let mid_row = rows_in_view(row_count, row_height, 30., viewport);
        let at_end = rows_in_view(row_count, row_height, 19_900., viewport);

        // then
        assert_eq!((at_top, mid_row, at_end), (0..5, 1..7, 995..1000));
    }

    #[test]
    fn should_keep_an_offset_scroll_inside_its_content() {
        // given
        let scroll = OffsetScroll::default();
        scroll.set_size(100., 300.);
        scroll.scroll_to(Axis::Horizontal, 500.);
        let past_the_end = scroll.scrolled();

        // when
        scroll.set_size(100., 150.);

        // then
        assert_eq!((past_the_end, scroll.scrolled()), (200., 50.));
    }

    #[test]
    fn should_turn_vertical_wheel_sideways_only_with_shift() {
        // given
        let (dx, dy) = (0., -30.);

        // when
        let plain = wheel_axes(dx, dy, false);
        let shifted = wheel_axes(dx, dy, true);
        let already_horizontal = wheel_axes(-30., 0., true);

        // then
        assert_eq!((plain, shifted, already_horizontal), ((0., -30.), (-30., 0.), (-30., 0.)));
    }
}
