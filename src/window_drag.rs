use std::cell::Cell;
use std::rc::Rc;

use gpui::{Div, MouseButton, Pixels, Size, Stateful, Window, prelude::*};

use crate::frame;
use crate::window_state::{DEFAULT_SIZE, WindowPlace};

/// Makes empty space move the window, as an OS title bar does: drag to move, double-click to maximize,
/// right-click for the window menu. Children that handle clicks stop the mouse-down from reaching it.
#[derive(Clone, Default)]
pub struct WindowDrag {
    state: Rc<Cell<DragState>>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum DragState {
    #[default]
    Idle,
    /// the first move after the mouse down hands the drag to the window manager
    Pressed,
    /// a window manager does not move a fullscreen window: the drag waits until it has left fullscreen
    LeavingFullscreen,
}

impl WindowDrag {
    /// On Windows the row's `WindowControlArea::Drag` does all of it natively, and a double click here would fight
    /// the native one.
    pub fn apply(&self, area: Stateful<Div>) -> Stateful<Div> {
        if cfg!(windows) {
            return area;
        }
        let (on_down, on_up, on_up_out, on_out, on_move) =
            (self.state.clone(), self.state.clone(), self.state.clone(), self.state.clone(), self.state.clone());
        area.on_mouse_down(MouseButton::Left, move |event, window, _cx| {
            if event.click_count == 2 {
                on_down.set(DragState::Idle);
                toggle_maximized(window);
                return;
            }
            on_down.set(DragState::Pressed);
        })
        .on_mouse_up(MouseButton::Left, move |_event, _window, _cx| on_up.set(DragState::Idle))
        .on_mouse_up_out(MouseButton::Left, move |_event, _window, _cx| on_up_out.set(DragState::Idle))
        .on_mouse_down_out(move |_event, _window, _cx| on_out.set(DragState::Idle))
        .on_mouse_move(move |_event, window, _cx| {
            if on_move.get() == DragState::Pressed {
                on_move.set(start_move(window));
            }
        })
        .on_mouse_down(MouseButton::Right, |event, window, _cx| {
            if window.window_controls().window_menu {
                window.show_window_menu(event.position);
            }
        })
    }

    /// Moves a window dragged while fullscreen once it has left fullscreen, at the size it had before (`place`), or
    /// the default one. Called on every render of the window.
    pub fn follow(&self, place: &WindowPlace, window: &mut Window) {
        if self.state.get() == DragState::LeavingFullscreen && !window.is_fullscreen() {
            self.state.set(DragState::Idle);
            window.resize(frame::outer_size(windowed_size(place), window));
            window.start_window_move();
        }
    }
}

fn start_move(window: &mut Window) -> DragState {
    if window.is_fullscreen() {
        window.toggle_fullscreen();
        return DragState::LeavingFullscreen;
    }
    window.start_window_move();
    DragState::Idle
}

fn windowed_size(place: &WindowPlace) -> Size<Pixels> {
    place.saved().map_or(DEFAULT_SIZE, |saved| saved.bounds().size)
}

fn toggle_maximized(window: &Window) {
    if cfg!(target_os = "macos") {
        // follows the user's "double-click a window's title bar to" system setting
        window.titlebar_double_click();
        return;
    }
    window.zoom_window();
}
