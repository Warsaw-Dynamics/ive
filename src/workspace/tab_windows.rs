use std::collections::HashMap;

use gpui::{AnyWindowHandle, App, AppContext, Bounds, Context, EntityId, FocusHandle, Pixels, Point, Size, WeakEntity, Window, point, px, size};

use crate::tab_window::TabWindow;
use crate::tabs::TabId;
use crate::window_controls;

use super::{TabDragArea, Workspace};

/// Smallest window a tab is detached into, whatever size the main window has.
const MIN_SIZE: Size<Pixels> = size(px(480.), px(320.));
/// Where the pointer lands in the new window: over its tab, as if the window came out of the row with the tab.
const GRAB_OFFSET: Point<Pixels> = point(px(60.), px(18.));
/// How far a window detached by an MCP or script call is from the main window's top left, so both show.
const REQUEST_OFFSET: Point<Pixels> = point(px(48.), px(48.));

/// The windows of detached tabs, by tab.
#[derive(Default)]
pub(super) struct TabWindows(HashMap<TabId, DetachedWindow>);

struct DetachedWindow {
    handle: AnyWindowHandle,
    /// the window's [`TabWindow`], notified to redraw only that window
    view: EntityId,
}

/// A tab being dragged in the main window: where the pointer is, and whether it is over the tab row.
#[derive(Clone, Copy)]
pub(super) struct TabDrag {
    id: TabId,
    position: Point<Pixels>,
    over_row: bool,
}

impl Workspace {
    /// Notes where dragged tab `id` is, so a release away from the tab row detaches it, and shows there the preview
    /// of the window it would detach into.
    pub fn track_tab_drag(&mut self, id: TabId, position: Point<Pixels>, area: TabDragArea, window: &mut Window, cx: &mut Context<Self>) {
        if self.tab_drag.is_none_or(|drag| drag.id != id) {
            self.start_drag_preview(id, window, cx);
        }
        let over_row = area.row.contains(&position);
        self.tab_drag = Some(TabDrag { id, position, over_row });
        let pointer = (!over_row).then(|| window.bounds().origin + position);
        self.move_drag_preview(pointer, area.body, window, cx);
    }

    /// A left button release in the main window: a tab dragged and released away from the tab row goes to a window
    /// of its own, under the pointer.
    pub fn finish_tab_drag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let drag = self.tab_drag.take();
        self.end_drag_preview(cx);
        let Some(drag) = drag.filter(|drag| cx.has_active_drag() && !drag.over_row) else {
            return;
        };
        let at = window.bounds().origin + drag.position;
        self.detach_tab(drag.id, at, window, cx);
    }

    /// Opens tab `id` of the tab row in a window of its own at screen point `at`; the tab stays the workspace's.
    pub fn detach_tab(&mut self, id: TabId, at: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let focus = self.tabs.get(id).and_then(|tab| tab.focus_handle(self, cx));
        if !self.tabs.detach(id) {
            return;
        }
        // the main window's size without the band its frame draws the shadow in, which the new window adds again
        let size = window.inner_window_bounds().get_bounds().size.max(&MIN_SIZE);
        let bounds = Bounds::new(at - GRAB_OFFSET, size);
        let workspace = cx.entity().downgrade();
        // a window draws once while it opens, reading the workspace, which is busy until this returns
        cx.defer(move |cx| open_tab_window(workspace, id, bounds, focus, cx));
        self.tab_shown(window, cx);
    }

    fn tab_window_opened(&mut self, id: TabId, opened: anyhow::Result<DetachedWindow>, cx: &mut Context<Self>) {
        match opened {
            Ok(detached) => {
                self.tab_windows.0.insert(id, detached);
                // the tab may have closed while the window opened
                self.close_stale_tab_windows(cx);
            }
            Err(error) => {
                eprintln!("ive: cannot open a window for tab {}: {error}", id.value());
                self.tabs.attach(id);
                cx.notify();
            }
        }
    }

    /// Detaches tab `id` into a window over the main window's top left.
    pub(super) fn detach_tab_by_request(&mut self, id: TabId, window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        if self.tabs.is_detached(id) {
            return Err(format!("tab {} is in a window of its own already", id.value()));
        }
        let at = window.bounds().origin + GRAB_OFFSET + REQUEST_OFFSET;
        self.detach_tab(id, at, window, cx);
        Ok(())
    }

    pub(super) fn attach_tab_by_request(&mut self, id: TabId, window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        if self.tabs.is_detached(id) {
            self.attach_tab(id, window, cx);
            return Ok(());
        }
        Err(format!("tab {} is in the tab row already", id.value()))
    }

    /// Puts detached tab `id` back into the main window's tab row and shows it there; its window closes.
    pub fn attach_tab(&mut self, id: TabId, window: &mut Window, cx: &mut Context<Self>) {
        if self.tabs.attach(id) {
            self.tab_shown(window, cx);
        }
    }

    /// Brings detached tab `id`'s window to the front; false when the tab is in the row.
    pub(super) fn activate_tab_window(&self, id: TabId, cx: &mut App) -> bool {
        let Some(detached) = self.tab_windows.0.get(&id) else {
            return false;
        };
        // a window being closed is gone already
        let _ = detached.handle.update(cx, |_, window, _| window.activate_window());
        true
    }

    /// Redraws the window of detached tab `id`, e.g. for new terminal output, without redrawing the main window.
    pub(super) fn redraw_tab_window(&self, id: TabId, cx: &mut App) {
        if let Some(detached) = self.tab_windows.0.get(&id) {
            cx.notify(detached.view);
        }
    }

    /// Closes the windows of tabs that closed or went back into the row. Later, since the window may be the one
    /// handling the current event.
    pub(super) fn close_stale_tab_windows(&mut self, cx: &mut App) {
        let stale: Vec<TabId> = self.tab_windows.0.keys().copied().filter(|id| !self.tabs.is_detached(*id)).collect();
        let handles: Vec<AnyWindowHandle> = stale.iter().filter_map(|id| self.tab_windows.0.remove(id)).map(|detached| detached.handle).collect();
        if handles.is_empty() {
            return;
        }
        cx.defer(move |cx| {
            for handle in handles {
                let _ = handle.update(cx, |_, window, _| window.remove_window());
            }
        });
    }
}

fn open_tab_window(workspace: WeakEntity<Workspace>, id: TabId, bounds: Bounds<Pixels>, focus: Option<FocusHandle>, cx: &mut App) {
    let options = gpui::WindowOptions { keep_position: true, ..window_controls::window_options(bounds) };
    let owner = workspace.clone();
    let mut view = None;
    let opened = cx.open_window(options, |window, cx| {
        let tab_window = cx.new(|cx| TabWindow::new(owner, id, focus, window, cx));
        view = Some(tab_window.entity_id());
        tab_window
    });
    let opened = opened.map(|handle| DetachedWindow { handle: handle.into(), view: view.expect("open_window builds the root view") });
    let _ = workspace.update(cx, |workspace, cx| workspace.tab_window_opened(id, opened, cx));
}
