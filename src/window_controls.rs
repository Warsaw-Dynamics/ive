use gpui::{
    App, Bounds, ClickEvent, Decorations, Div, MouseButton, Pixels, Stateful, TitlebarOptions, Window, WindowBounds, WindowControlArea,
    WindowDecorations, WindowOptions, div, point, prelude::*, px, rgb, svg,
};

use crate::click_targets::target;
use crate::icons;
use crate::tab_window::TabWindow;
use crate::theme::Theme;
use crate::workspace::Workspace;

/// Room the macOS traffic lights take at the left of the sidebar header, which they sit in.
pub const TRAFFIC_LIGHTS_WIDTH: f32 = 80.;

/// Window options that hide the OS title bar: IVE's tab row and sidebar header take its place.
/// Linux falls back to the OS title bar when there is no compositor; [`window_controls`] then draws no buttons.
/// The window's app id (WM_CLASS on X11): a desktop entry with `StartupWMClass=ive` gives the running window its
/// icon in the task bar.
pub const APP_ID: &str = "ive";

pub fn window_options(bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        app_id: Some(APP_ID.to_owned()),
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("IVE".into()),
            appears_transparent: true,
            // centred in the 36 px sidebar header
            traffic_light_position: Some(point(px(12.), px(11.))),
        }),
        window_decorations: Some(WindowDecorations::Client),
        app_owns_titlebar_drag: true,
        ..Default::default()
    }
}

/// Minimize, maximize or restore, and close; none on macOS, which keeps its traffic lights, nor when the OS
/// draws the title bar.
pub fn window_controls(theme: Theme, window: &Window, cx: &App) -> Option<Div> {
    if !ive_draws_window_buttons(window) {
        return None;
    }
    let supported = window.window_controls();
    let maximize = if window.is_maximized() { icons::WINDOW_RESTORE } else { icons::WINDOW_MAXIMIZE };
    let controls = div()
        .flex()
        .h_full()
        .when(supported.minimize, |controls| {
            controls.child(
                window_button("minimize", icons::WINDOW_MINIMIZE, WindowControlArea::Min, theme)
                    .hover(|style| style.bg(rgb(theme.hover)))
                    .on_click(|_event, window, _cx| window.minimize_window()),
            )
        })
        .when(supported.maximize, |controls| {
            controls.child(
                window_button("maximize", maximize, WindowControlArea::Max, theme)
                    .hover(|style| style.bg(rgb(theme.hover)))
                    .on_click(|_event, window, _cx| window.zoom_window()),
            )
        })
        .child(
            window_button("close", icons::WINDOW_CLOSE, WindowControlArea::Close, theme)
                .hover(|style| style.bg(rgb(theme.close_hover)))
                .on_click(target("window:close".to_owned(), request_quit, cx)),
        );
    Some(controls)
}

/// Whether IVE draws the window's buttons in `window`: not on macOS, which keeps its traffic lights, nor where the OS
/// decorates the window.
fn ive_draws_window_buttons(window: &Window) -> bool {
    draws_window_buttons(std::env::consts::OS, matches!(window.window_decorations(), Decorations::Client { .. }))
}

/// [`ive_draws_window_buttons`] for `os` (as `std::env::consts::OS`) and whether the client decorates the window.
fn draws_window_buttons(os: &str, client_decorations: bool) -> bool {
    os != "macos" && (os == "windows" || client_decorations)
}

/// Above and right of the close button in a maximised window with rounded panels: the frame there closes the window
/// too, so the screen's very corner still hits it. Drawn below the panels, so the button keeps its own clicks; none
/// where IVE draws no close button.
pub fn corner_close(theme: Theme, window: &Window, cx: &App) -> Option<Stateful<Div>> {
    if !ive_draws_window_buttons(window) {
        return None;
    }
    let reach = theme.sizes.window.control_width.0 + theme.panels.gap.0;
    let height = theme.sizes.tabs.row_height.0 + theme.panels.gap.0;
    let corner = div().id("window-corner-close").absolute().top_0().right_0().w(px(reach)).h(px(height));
    Some(corner.on_click(target("window:corner-close".to_owned(), request_quit, cx)))
}

/// The × asks the workspace, which may ask the user first; a detached tab's closes its tab; any other window just
/// closes.
fn request_quit(_event: &ClickEvent, window: &mut Window, cx: &mut App) {
    if let Some(workspace) = window.root::<Workspace>().flatten() {
        return workspace.update(cx, |workspace, cx| workspace.request_quit(window, cx));
    }
    match window.root::<TabWindow>().flatten() {
        Some(tab_window) => TabWindow::close(&tab_window, window, cx),
        None => window.remove_window(),
    }
}

/// A window button; its icon is drawn as a mask so it can turn white over the red close hover.
fn window_button(id: &'static str, name: &str, area: WindowControlArea, theme: Theme) -> Stateful<Div> {
    let group = format!("window-{id}");
    div()
        .id(id)
        .group(group.clone())
        .flex()
        .justify_center()
        .items_center()
        .w(theme.sizes.window.control_width.px())
        .h_full()
        .window_control_area(area)
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
        .child(
            svg()
                .path(icons::path(theme.appearance, name))
                .size(theme.sizes.window.control_icon_size.px())
                .text_color(rgb(theme.text))
                .when(id == "close", |icon| icon.group_hover(group, |style| style.text_color(rgb(theme.on_accent)))),
        )
}

#[cfg(test)]
mod tests {
    use gpui::size;

    use super::*;

    #[test]
    fn should_draw_window_buttons_only_where_the_os_leaves_them_to_ive() {
        // given
        let cases = [("linux", true), ("linux", false), ("macos", true), ("windows", false)];

        // when
        let drawn = cases.map(|(os, client_decorations)| draws_window_buttons(os, client_decorations));

        // then
        assert_eq!(drawn, [true, false, false, true]);
    }

    #[test]
    fn should_name_window_as_the_desktop_entry() {
        // given
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(1000.), px(700.)));

        // when
        let options = window_options(bounds);

        // then
        assert_eq!(options.app_id.as_deref(), Some(APP_ID));
        assert_eq!(APP_ID, "ive");
    }
}
