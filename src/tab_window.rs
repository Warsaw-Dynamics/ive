use gpui::{App, Context, Entity, FocusHandle, IntoElement, Render, Subscription, WeakEntity, Window, div, prelude::*, rgb};

use crate::editor;
use crate::frame;
use crate::tabs::TabId;
use crate::theme;
use crate::window_drag::WindowDrag;
use crate::window_state::WindowPlace;
use crate::workspace::Workspace;

/// A window showing one tab pulled out of the main window's tab row. The tab stays the workspace's: it is found,
/// reloaded, saved and closed as any other; this window only draws it.
pub struct TabWindow {
    workspace: WeakEntity<Workspace>,
    tab: TabId,
    drag: WindowDrag,
    /// the size a drag out of fullscreen returns the window to
    place: WindowPlace,
    /// keys go here while the tab takes none, e.g. a document
    focus_handle: FocusHandle,
    title: String,
    _subscriptions: Vec<Subscription>,
}

impl TabWindow {
    /// `focus` is what the tab takes the keyboard with, if anything.
    pub fn new(workspace: WeakEntity<Workspace>, tab: TabId, focus: Option<FocusHandle>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(focus.as_ref().unwrap_or(&focus_handle), cx);
        let subscriptions = vec![cx.observe_window_activation(window, |tab_window, window, cx| {
            // the tab chip shows whether this window is active
            cx.notify();
            let active = window.is_window_active();
            let _ = tab_window.workspace.update(cx, |workspace, cx| match active {
                true => workspace.refresh_on_activation(cx),
                // edits in the tab reach the disk when the user turns to another window
                false => workspace.flush_edits(cx),
            });
        })];
        let closing = workspace.clone();
        // closing the window closes the tab, as its × in the row would
        window.on_window_should_close(cx, move |window, cx| close_tab(&closing, tab, window, cx));
        Self { workspace, tab, drag: WindowDrag::default(), place: WindowPlace::default(), focus_handle, title: String::new(), _subscriptions: subscriptions }
    }

    /// The × of the window's controls.
    pub fn close(tab_window: &Entity<Self>, window: &mut Window, cx: &mut App) {
        let (workspace, tab) = {
            let tab_window = tab_window.read(cx);
            (tab_window.workspace.clone(), tab_window.tab)
        };
        if close_tab(&workspace, tab, window, cx) {
            window.remove_window();
        }
    }

    fn set_title(&mut self, title: String, window: &mut Window) {
        if title != self.title {
            window.set_window_title(&format!("{title} — IVE"));
            self.title = title;
        }
    }
}

/// Closes the tab through the workspace, which then closes the window; true when the workspace is gone and the
/// window has to close itself.
fn close_tab(workspace: &WeakEntity<Workspace>, tab: TabId, window: &mut Window, cx: &mut App) -> bool {
    workspace.update(cx, |workspace, cx| workspace.close_tab(tab, window, cx)).is_err()
}

impl Render for TabWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.place.observe(window);
        self.drag.follow(&self.place, window);
        let theme = crate::theme::set_tabs(window, cx);
        crate::theme::apply_to_window(&theme, window);
        let drawn = self.workspace.upgrade().and_then(|workspace| {
            workspace.update(cx, |workspace, cx| editor::render_detached(workspace, self.tab, &self.drag, window, cx))
        });
        let content = match drawn {
            Some((title, content)) => {
                self.set_title(title, window);
                content
            }
            // the tab closed; the workspace closes this window next
            None => div(),
        };
        let mut content = div()
            .flex()
            .flex_col()
            .size_full()
            .child(content.flex_1().min_h_0())
            // input here counts against the update restart's idle time as the main window's does
            .child(crate::workspace::activity_watcher(self.workspace.clone()))
            .track_focus(&self.focus_handle)
            // the user's Copy and Select All on selected text; the workspace's other bindings stay in the main window
            .on_key_down(|event, window, cx| {
                if crate::workspace::run_text_key(event, window, cx) {
                    cx.stop_propagation();
                }
            })
            .bg(rgb(theme.background))
            .font_family(theme::ui_font(cx))
            .text_color(rgb(theme.text));
        content.text_style().font_fallbacks = Some(theme::emoji_fallbacks());
        frame::render(content, theme, window)
    }
}

