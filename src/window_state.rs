//! Where the main window is and whether it is maximized or fullscreen, saved in `state.toml` so the next IVE,
//! a restarted one too, opens the window as it was.

use gpui::{App, Bounds, Pixels, Size, Window, point, px, size};
use serde::{Deserialize, Serialize};

/// Sizes outside these, or a window off every screen, are not something IVE saved: it opens centred instead.
const MIN_SIZE: f32 = 100.;
const MAX_SIZE: f32 = 16384.;
const MAX_COORDINATE: f32 = 100_000.;
/// The size of a window with none saved.
pub const DEFAULT_SIZE: Size<Pixels> = Size { width: px(1000.), height: px(700.) };

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SavedWindow {
    /// the windowed bounds in logical pixels; a maximized or fullscreen window returns to them
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub mode: WindowMode,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowMode {
    #[default]
    Windowed,
    Maximized,
    Fullscreen,
}

impl SavedWindow {
    fn new(bounds: Bounds<Pixels>, mode: WindowMode) -> Self {
        let (x, y) = (f32::from(bounds.origin.x), f32::from(bounds.origin.y));
        Self { x, y, width: f32::from(bounds.size.width), height: f32::from(bounds.size.height), mode }
    }

    pub fn bounds(&self) -> Bounds<Pixels> {
        Bounds::new(point(px(self.x), px(self.y)), size(px(self.width), px(self.height)))
    }

    /// The bounds to open the window with, unless they are unreasonable or no longer on any screen (a monitor was
    /// unplugged).
    pub fn usable_bounds(&self, screens: &[Bounds<Pixels>]) -> Option<Bounds<Pixels>> {
        let sizes = [self.width, self.height];
        let sane = sizes.iter().all(|side| (MIN_SIZE..=MAX_SIZE).contains(side))
            && [self.x, self.y].iter().all(|coordinate| coordinate.abs() <= MAX_COORDINATE);
        let bounds = self.bounds();
        (sane && screens.iter().any(|screen| screen.intersects(&bounds))).then_some(bounds)
    }
}

/// Follows the main window. X11 reports a maximized window's maximized bounds, not those it returns to, so the
/// windowed bounds are kept from before it was maximized.
#[derive(Debug, Default)]
pub struct WindowPlace {
    current: Option<SavedWindow>,
    /// the saved mode, applied once the window is shown
    pending_mode: Option<WindowMode>,
    /// the windowed bounds before the latest ones: a window may report its maximized size before it reports being
    /// maximized, so the latest windowed bounds can be the maximized ones
    earlier: Option<Bounds<Pixels>>,
}

impl WindowPlace {
    pub fn new(saved: Option<SavedWindow>) -> Self {
        Self { current: saved, pending_mode: saved.map(|saved| saved.mode), earlier: None }
    }

    /// Maximizes or fullscreens the window as saved, the first time it is shown: a window manager ignores the
    /// request for a window it does not manage yet, as gpui sends it.
    pub fn shown(&mut self, window: &Window) {
        match self.pending_mode.take() {
            Some(WindowMode::Maximized) => window.zoom_window(),
            Some(WindowMode::Fullscreen) => window.toggle_fullscreen(),
            Some(WindowMode::Windowed) | None => {}
        }
    }

    pub fn saved(&self) -> Option<SavedWindow> {
        self.current
    }

    pub fn observe(&mut self, window: &Window) {
        let mode = match (window.is_fullscreen(), window.is_maximized()) {
            (true, _) => WindowMode::Fullscreen,
            (false, true) => WindowMode::Maximized,
            (false, false) => WindowMode::Windowed,
        };
        // without the transparent band of client decorations, which the window manager adds again when it opens
        self.update(window.inner_window_bounds().get_bounds(), mode);
    }

    fn update(&mut self, bounds: Bounds<Pixels>, mode: WindowMode) {
        let windowed = self.current.map(|current| current.bounds());
        let restore = match windowed {
            Some(windowed) if mode != WindowMode::Windowed && windowed == bounds => self.earlier.unwrap_or(bounds),
            Some(windowed) if mode != WindowMode::Windowed => windowed,
            _ => bounds,
        };
        if mode == WindowMode::Windowed && windowed != Some(bounds) {
            self.earlier = windowed;
        }
        self.current = Some(SavedWindow::new(restore, mode));
    }
}

pub fn screens(cx: &App) -> Vec<Bounds<Pixels>> {
    cx.displays().iter().map(|display| display.bounds()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(width), px(height)))
    }

    #[test]
    fn should_keep_the_windowed_bounds_of_a_maximized_window() {
        // given
        let mut place = WindowPlace::default();
        place.update(rect(1950., 40., 1000., 700.), WindowMode::Windowed);

        // when
        place.update(rect(1920., 0., 1920., 1080.), WindowMode::Maximized);

        // then
        assert_eq!(place.saved(), Some(SavedWindow { x: 1950., y: 40., width: 1000., height: 700., mode: WindowMode::Maximized }));
    }

    #[test]
    fn should_keep_the_windowed_bounds_when_the_maximized_size_comes_before_the_maximized_state() {
        // given
        let mut place = WindowPlace::default();
        place.update(rect(10., 20., 1000., 700.), WindowMode::Windowed);
        place.update(rect(0., 0., 1920., 1080.), WindowMode::Windowed);

        // when
        place.update(rect(0., 0., 1920., 1080.), WindowMode::Fullscreen);
        place.update(rect(0., 0., 1920., 1080.), WindowMode::Fullscreen);

        // then
        assert_eq!(place.saved().map(|saved| (saved.bounds(), saved.mode)), Some((rect(10., 20., 1000., 700.), WindowMode::Fullscreen)));
    }

    #[test]
    fn should_follow_a_window_moved_after_it_was_restored() {
        // given
        let mut place = WindowPlace::new(Some(SavedWindow { x: 10., y: 20., width: 1000., height: 700., mode: WindowMode::Maximized }));
        place.update(rect(10., 20., 1000., 700.), WindowMode::Windowed);

        // when
        place.update(rect(300., 200., 1000., 700.), WindowMode::Windowed);

        // then
        assert_eq!(place.saved(), Some(SavedWindow { x: 300., y: 200., width: 1000., height: 700., mode: WindowMode::Windowed }));
    }

    #[test]
    fn should_open_saved_bounds_only_on_a_screen_and_of_a_sane_size() {
        // given
        let screens = [rect(0., 0., 1920., 1080.), rect(1920., 0., 2560., 1440.)];
        let saved = |x, width| SavedWindow { x, y: 100., width, height: 700., mode: WindowMode::Windowed };

        // when
        let usable = [saved(2000., 1000.), saved(5000., 1000.), saved(100., 20.), saved(f32::NAN, 1000.)].map(|window| window.usable_bounds(&screens));

        // then
        assert_eq!(usable, [Some(rect(2000., 100., 1000., 700.)), None, None, None]);
    }
}
