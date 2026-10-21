use gpui::{
    AnyElement, Bounds, BoxShadow, CursorStyle, Decorations, HitboxBehavior, MouseButton, Pixels, Point,
    ResizeEdge, Size, Tiling, Window, canvas, div, rgba, point, prelude::*, prelude::FluentBuilder, px, rgb, size,
};

use crate::theme::Theme;

/// Transparent band around the window that holds the shadow and is grabbed to resize it.
const RESIZE_BORDER: f32 = 10.;

/// Wraps the window content in the frame IVE draws when the OS leaves decorations to it (Linux with a
/// compositor): shadow, border, rounded top corners and resize handles. Edges tiled to the screen get none.
pub fn render(content: impl IntoElement, theme: Theme, window: &mut Window) -> AnyElement {
    let Decorations::Client { tiling } = window.window_decorations() else {
        return content.into_any_element();
    };
    window.set_client_inset(px(RESIZE_BORDER));
    let inner = div()
        .size_full()
        .overflow_hidden()
        .cursor(CursorStyle::Arrow)
        .border_color(rgb(theme.border))
        .map(|inner| round_top(inner, tiling, theme))
        .when(!tiling.top, |inner| inner.border_t(theme.sizes.window.border.px()))
        .when(!tiling.bottom, |inner| inner.border_b(theme.sizes.window.border.px()))
        .when(!tiling.left, |inner| inner.border_l(theme.sizes.window.border.px()))
        .when(!tiling.right, |inner| inner.border_r(theme.sizes.window.border.px()))
        .when(!tiling.is_tiled(), |inner| {
            inner.shadow(vec![BoxShadow {
                color: rgba(theme.shadow).into(),
                offset: point(px(0.), px(0.)),
                blur_radius: px(RESIZE_BORDER / 2.),
                spread_radius: px(0.),
                inset: false,
            }])
        })
        // the band below sets resize cursors on mouse move; inside the window the content decides
        .on_mouse_move(|_event, _window, cx| cx.stop_propagation())
        .child(content);
    div()
        .id("window-frame")
        .size_full()
        .map(|frame| round_top(frame, tiling, theme))
        .when(!tiling.top, |frame| frame.pt(px(RESIZE_BORDER)))
        .when(!tiling.bottom, |frame| frame.pb(px(RESIZE_BORDER)))
        .when(!tiling.left, |frame| frame.pl(px(RESIZE_BORDER)))
        .when(!tiling.right, |frame| frame.pr(px(RESIZE_BORDER)))
        .child(resize_cursors(tiling))
        .on_mouse_move(|_event, window, _cx| window.refresh())
        .on_mouse_down(MouseButton::Left, move |event, window, _cx| {
            let size = window.window_bounds().get_bounds().size;
            if let Some(edge) = resize_edge(event.position, size, tiling) {
                window.start_window_resize(edge);
            }
        })
        .child(inner)
        .into_any_element()
}

fn round_top<E: Styled + FluentBuilder>(element: E, tiling: Tiling, theme: Theme) -> E {
    let radius = theme.sizes.window.corner_radius.px();
    element
        .when(!(tiling.top || tiling.left), |element| element.rounded_tl(radius))
        .when(!(tiling.top || tiling.right), |element| element.rounded_tr(radius))
}

/// The window size for content of `inner` size: the frame adds its band where the window is free to resize.
pub fn outer_size(inner: Size<Pixels>, window: &Window) -> Size<Pixels> {
    match window.window_decorations() {
        Decorations::Client { tiling } => framed_size(inner, tiling),
        Decorations::Server => inner,
    }
}

fn framed_size(inner: Size<Pixels>, tiling: Tiling) -> Size<Pixels> {
    let bands = |sides: [bool; 2]| px(RESIZE_BORDER) * sides.iter().filter(|tiled| !**tiled).count() as f32;
    size(inner.width + bands([tiling.left, tiling.right]), inner.height + bands([tiling.top, tiling.bottom]))
}

/// Shows the resize cursor that matches the edge under the mouse.
fn resize_cursors(tiling: Tiling) -> impl IntoElement {
    canvas(
        |_bounds, window, _cx| {
            let size = window.window_bounds().get_bounds().size;
            window.insert_hitbox(Bounds::new(point(px(0.), px(0.)), size), HitboxBehavior::Normal)
        },
        move |_bounds, hitbox, window, _cx| {
            let size = window.window_bounds().get_bounds().size;
            if let Some(edge) = resize_edge(window.mouse_position(), size, tiling) {
                window.set_cursor_style(cursor_for(edge), &hitbox);
            }
        },
    )
    .absolute()
    .size_full()
}

fn cursor_for(edge: ResizeEdge) -> CursorStyle {
    match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    }
}

/// The window edge or corner `position` is on, if it is within the band of one; edges tiled to the screen, all of
/// them in fullscreen, have no band and do not resize.
fn resize_edge(position: Point<Pixels>, size: Size<Pixels>, tiling: Tiling) -> Option<ResizeEdge> {
    let border = px(RESIZE_BORDER);
    let top = !tiling.top && position.y < border;
    let bottom = !tiling.bottom && position.y > size.height - border;
    let left = !tiling.left && position.x < border;
    let right = !tiling.right && position.x > size.width - border;
    match (top, bottom, left, right) {
        (true, _, true, _) => Some(ResizeEdge::TopLeft),
        (true, _, _, true) => Some(ResizeEdge::TopRight),
        (_, true, true, _) => Some(ResizeEdge::BottomLeft),
        (_, true, _, true) => Some(ResizeEdge::BottomRight),
        (true, _, _, _) => Some(ResizeEdge::Top),
        (_, true, _, _) => Some(ResizeEdge::Bottom),
        (_, _, true, _) => Some(ResizeEdge::Left),
        (_, _, _, true) => Some(ResizeEdge::Right),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_edge_or_corner_within_border() {
        // given
        let window = size(px(800.), px(600.));
        let positions = [(5., 5.), (400., 3.), (795., 300.), (795., 595.), (2., 598.), (400., 300.), (15., 15.)];

        // when
        let edges: Vec<_> =
            positions.iter().map(|(x, y)| resize_edge(point(px(*x), px(*y)), window, Tiling::default())).collect();

        // then
        assert_eq!(
            edges,
            [
                Some(ResizeEdge::TopLeft),
                Some(ResizeEdge::Top),
                Some(ResizeEdge::Right),
                Some(ResizeEdge::BottomRight),
                Some(ResizeEdge::BottomLeft),
                None,
                None,
            ]
        );
    }

    #[test]
    fn should_not_resize_at_tiled_edges() {
        // given
        let window = size(px(800.), px(600.));
        let left_half = Tiling { left: true, top: true, bottom: true, right: false };

        // when
        let fullscreen = resize_edge(point(px(5.), px(5.)), window, Tiling::tiled());
        let tiled_corner = resize_edge(point(px(5.), px(5.)), window, left_half);
        let free_edge = resize_edge(point(px(795.), px(595.)), window, left_half);

        // then
        assert_eq!((fullscreen, tiled_corner, free_edge), (None, None, Some(ResizeEdge::Right)));
    }

    #[test]
    fn should_add_bands_only_on_free_edges() {
        // given
        let inner = size(px(1000.), px(700.));
        let maximized_vertically = Tiling { top: true, bottom: true, left: false, right: false };

        // when
        let framed = framed_size(inner, maximized_vertically);

        // then
        assert_eq!(framed, size(px(1020.), px(700.)));
    }
}
