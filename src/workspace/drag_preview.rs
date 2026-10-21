use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnyWindowHandle, App, AppContext, Bounds, Context, Image, ImageFormat, Pixels, Point, SharedString, Size, Task, WindowBackgroundAppearance,
    WindowBounds, WindowKind, WindowOptions, point, px, size,
};

use crate::tab_preview::TabPreview;
use crate::theme::Theme;
use crate::tabs::TabId;
use crate::window_capture;
use crate::window_controls::APP_ID;

use super::Workspace;

/// Largest picture of a tab's content in a preview.
const MAX_PICTURE: Size<Pixels> = size(px(320.), px(220.));
/// Where the pointer is on the preview: on its title, as on the window's tab once it is detached.
const GRAB_OFFSET: Point<Pixels> = point(px(24.), px(12.));
/// A tab shown as the drag started is drawn before it is captured.
const DRAW_WAIT: Duration = Duration::from_millis(50);

/// The preview of the window a dragged tab would detach into, from its first move away from the tab row to the
/// end of the drag.
pub(super) struct DragPreview {
    tab: TabId,
    title: SharedString,
    picture: Picture,
    window: Option<AnyWindowHandle>,
    /// where the pointer is on the screen while it is away from the tab row
    pointer: Option<Point<Pixels>>,
}

enum Picture {
    None,
    /// the capture runs while its task is kept
    Capturing { _task: Task<()> },
    Ready { image: Arc<Image>, size: Size<Pixels> },
    /// no preview for this drag, the chip under the pointer stays
    Failed,
}

/// Where a dragged tab is in the main window's pane: its tab row, and the content below it.
#[derive(Clone, Copy)]
pub struct TabDragArea {
    pub row: Bounds<Pixels>,
    pub body: Bounds<Pixels>,
}

impl Workspace {
    /// Whether the chip of a dragged tab hides, while the preview stands for it.
    pub fn drag_chip_hidden(&self) -> Rc<Cell<bool>> {
        self.drag_chip_hidden.clone()
    }

    /// A new drag of tab `id`: it is shown, as pressing a tab in a browser does, so its content can be captured.
    pub(super) fn start_drag_preview(&mut self, id: TabId, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.end_drag_preview(cx);
        let Some(title) = self.tabs.get(id).map(|tab| SharedString::from(tab.title(self))) else {
            return;
        };
        if self.tabs.active_id() != Some(id) {
            self.activate_tab(id, window, cx);
        }
        self.drag_preview = Some(DragPreview { tab: id, title, picture: Picture::None, window: None, pointer: None });
    }

    /// Shows the preview at screen point `pointer`, capturing the content in `body` first; `None`, over the tab
    /// row, hides it.
    pub(super) fn move_drag_preview(&mut self, pointer: Option<Point<Pixels>>, body: Bounds<Pixels>, window: &gpui::Window, cx: &mut Context<Self>) {
        let Some(preview) = self.drag_preview.as_mut() else {
            return;
        };
        preview.pointer = pointer;
        if pointer.is_some() && matches!(preview.picture, Picture::None) {
            let task = capture(preview.tab, device_area(body, window.scale_factor()), window, cx);
            preview.picture = Picture::Capturing { _task: task };
        }
        self.place_drag_preview(cx);
    }

    /// The drag ended: the preview goes, and the chip shows again for the next drag.
    pub(super) fn end_drag_preview(&mut self, cx: &mut App) {
        if let Some(window) = self.drag_preview.take().and_then(|preview| preview.window) {
            close(window, cx);
        }
        self.drag_chip_hidden.set(false);
    }

    fn drag_preview_captured(&mut self, id: TabId, captured: Result<(Vec<u8>, Size<Pixels>), String>, cx: &mut Context<Self>) {
        let Some(preview) = self.drag_preview.as_mut().filter(|preview| preview.tab == id) else {
            return;
        };
        preview.picture = match captured {
            Ok((png, size)) => Picture::Ready { image: Arc::new(Image::from_bytes(ImageFormat::Png, png)), size },
            Err(error) => {
                eprintln!("ive: no preview of the dragged tab: {error}");
                Picture::Failed
            }
        };
        self.place_drag_preview(cx);
    }

    /// Opens, moves or closes the preview's window to match where the pointer is, and hides the chip while the
    /// preview shows.
    fn place_drag_preview(&mut self, cx: &mut App) {
        let Some(preview) = self.drag_preview.as_mut() else {
            return;
        };
        let shown = match (&preview.picture, preview.pointer) {
            (Picture::Ready { image, size }, Some(pointer)) => {
                let origin = pointer - GRAB_OFFSET;
                match preview.window {
                    // at every pointer event, as GTK and Swing move their drag images: waiting for a frame trails
                    Some(window) => window.update(cx, |_, window, _| window.move_to(origin)).is_ok(),
                    None => {
                        preview.window = open(TabPreview::new(preview.title.clone(), image.clone(), *size), origin, *size, cx);
                        preview.window.is_some()
                    }
                }
            }
            _ => {
                if let Some(window) = preview.window.take() {
                    close(window, cx);
                }
                false
            }
        };
        self.drag_chip_hidden.set(shown);
    }
}

/// `body` in the window's device pixels, as a capture of it has them.
fn device_area(body: Bounds<Pixels>, scale: f32) -> Bounds<u32> {
    let device = |value: Pixels| (f32::from(value) * scale).max(0.).round() as u32;
    Bounds::new(point(device(body.origin.x), device(body.origin.y)), size(device(body.size.width), device(body.size.height)))
}

/// Draws `window` to an image once the tab is drawn, and shrinks `area` of it to a preview's picture off the main
/// thread.
fn capture(id: TabId, area: Bounds<u32>, window: &gpui::Window, cx: &mut Context<Workspace>) -> Task<()> {
    let scale = window.scale_factor();
    let max = (f32::from(MAX_PICTURE.width) * scale, f32::from(MAX_PICTURE.height) * scale);
    cx.spawn_in(window, async move |workspace, cx| {
        cx.background_executor().timer(DRAW_WAIT).await;
        let image = cx.update(|window, _cx| window_capture::capture(window)).unwrap_or_else(|_| Err("the window is closed".to_owned()));
        let captured = cx
            .background_executor()
            .spawn(async move {
                let picture = window_capture::thumbnail(&image?, area, (max.0 as u32, max.1 as u32));
                let size = size(px(picture.width() as f32 / scale), px(picture.height() as f32 / scale));
                Ok((window_capture::encode_png(&picture)?, size))
            })
            .await;
        let _ = workspace.update(cx, |workspace, cx| workspace.drag_preview_captured(id, captured, cx));
    })
}

/// A drag image: no window manager decorates, places or focuses it, nor animates its moves, so it follows the
/// pointer exactly.
fn open(preview: TabPreview, origin: Point<Pixels>, picture: Size<Pixels>, cx: &mut App) -> Option<AnyWindowHandle> {
    let sizes = &cx.global::<Theme>().sizes.tabs;
    // the preview's border on both sides
    let border = px(2. * sizes.preview_border.0);
    let size = size(picture.width + border, picture.height + sizes.preview_header_height.px() + border);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(origin, size))),
        kind: WindowKind::DragImage,
        focus: false,
        is_movable: false,
        is_resizable: false,
        window_background: WindowBackgroundAppearance::Transparent,
        app_id: Some(APP_ID.to_owned()),
        ..Default::default()
    };
    match cx.open_window(options, |_, cx| cx.new(|_| preview)) {
        Ok(handle) => Some(handle.into()),
        Err(error) => {
            eprintln!("ive: cannot open the preview of the dragged tab: {error}");
            None
        }
    }
}

fn close(window: AnyWindowHandle, cx: &mut App) {
    let _ = window.update(cx, |_, window, _| window.remove_window());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_scale_the_content_area_to_device_pixels() {
        // given
        let body = Bounds::new(point(px(10.5), px(36.)), size(px(100.), px(50.)));

        // when
        let area = device_area(body, 2.);

        // then
        assert_eq!(area, Bounds::new(point(21, 72), size(200, 100)));
    }
}

