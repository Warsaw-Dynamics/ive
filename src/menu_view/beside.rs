//! A submenu beside its parent menu, as IntelliJ places it: right of the parent, left of it where the right has no
//! room, its top at its line, and kept inside the window.

use gpui::{
    AnyElement, App, Bounds, Display, Edges, Element, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Length, Pixels, Point,
    Position, Size, Style, Window, point,
};

/// `child` beside the parent menu, laid out over the parent: `over` is how far the parent's outer edges are outside
/// the element this is put in (its line), left and right, and top.
pub fn beside(child: AnyElement, over: Point<Pixels>) -> Beside {
    Beside { child, over }
}

pub struct Beside {
    child: AnyElement,
    over: Point<Pixels>,
}

/// Where a submenu of `size` goes beside a parent menu of `parent` bounds, in a window of `viewport`.
pub fn placed(parent: Bounds<Pixels>, size: Size<Pixels>, viewport: Size<Pixels>) -> Point<Pixels> {
    let zero = Pixels::ZERO;
    let x = match () {
        _ if parent.right() + size.width <= viewport.width => parent.right(),
        _ if parent.left() - size.width >= zero => parent.left() - size.width,
        _ => (viewport.width - size.width).max(zero),
    };
    let y = match parent.top() + size.height <= viewport.height {
        true => parent.top(),
        false => (viewport.height - size.height).max(zero),
    };
    point(x, y)
}

impl Element for Beside {
    type RequestLayoutState = LayoutId;
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(&mut self, _id: Option<&GlobalElementId>, _inspector_id: Option<&InspectorElementId>, window: &mut Window, cx: &mut App) -> (LayoutId, LayoutId) {
        let child = self.child.request_layout(window, cx);
        let inset = |by: Pixels| Length::from(-by);
        let style = Style {
            position: Position::Absolute,
            display: Display::Flex,
            inset: Edges { top: inset(self.over.y), left: inset(self.over.x), right: inset(self.over.x), bottom: Length::Auto },
            ..Style::default()
        };
        (window.request_layout(style, [child], cx), child)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        child: &mut LayoutId,
        window: &mut Window,
        cx: &mut App,
    ) {
        let child_bounds = window.layout_bounds(*child);
        let at = placed(bounds, child_bounds.size, window.viewport_size());
        let offset = at - child_bounds.origin;
        window.with_element_offset(point(offset.x.round(), offset.y.round()), |window| self.child.prepaint(window, cx));
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _child: &mut LayoutId,
        _prepaint: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

impl IntoElement for Beside {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use gpui::{px, size};

    use super::*;

    fn parent(left: f32, top: f32) -> Bounds<Pixels> {
        Bounds { origin: point(px(left), px(top)), size: size(px(200.), px(300.)) }
    }

    #[test]
    fn should_place_submenu_right_of_parent() {
        // when
        let at = placed(parent(100., 50.), size(px(150.), px(80.)), size(px(1000.), px(700.)));

        // then
        assert_eq!(at, point(px(300.), px(50.)));
    }

    #[test]
    fn should_place_submenu_left_of_parent_without_room_on_the_right() {
        // when
        let at = placed(parent(700., 50.), size(px(150.), px(80.)), size(px(1000.), px(700.)));

        // then
        assert_eq!(at, point(px(550.), px(50.)));
    }

    #[test]
    fn should_keep_submenu_inside_the_window() {
        // given
        let narrow = size(px(400.), px(700.));

        // when
        let at = placed(parent(100., 650.), size(px(150.), px(80.)), narrow);

        // then
        assert_eq!(at, point(px(250.), px(620.)));
    }
}
