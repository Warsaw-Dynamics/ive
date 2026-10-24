//! Processes that called for the user: raised when the user does not see them, cleared once they do.

use std::time::SystemTime;

use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use gpui::{AnyWindowHandle, Context, Task, Window};

use super::Workspace;
use crate::attention::{Sight, WindowFocus, counts};
use crate::sidebar::view::HeaderMenu;
use crate::process::ProcessId;

/// Shows the process of each desktop notification the user clicks, bringing the main window to the front.
pub(super) fn show_clicked(mut clicks: UnboundedReceiver<ProcessId>, window: &mut Window, cx: &mut Context<Workspace>) -> Task<()> {
    cx.spawn_in(window, async move |this, cx| {
        while let Some(id) = clicks.next().await {
            if this.update_in(cx, |workspace, window, cx| workspace.show_waiting(id, window, cx)).is_err() {
                return;
            }
        }
    })
}

impl Workspace {
    /// Window `handle`, the main one or a detached tab's, became active or stopped being so; once active, the
    /// processes it shows stop waiting.
    pub(crate) fn window_activated(&mut self, handle: AnyWindowHandle, focus: WindowFocus, cx: &mut Context<Self>) {
        self.active_window.observe(handle, focus);
        if focus == WindowFocus::Active {
            self.look_at_shown(cx);
        }
    }

    /// Takes process `id`'s newest call, if any it has not taken: the process waits for the user unless they see it.
    pub(super) fn take_call(&mut self, id: ProcessId, cx: &mut Context<Self>) {
        let Some(report) = self.processes.get(id).and_then(|process| process.view.call()) else {
            return;
        };
        let text = report.call.text();
        let fresh = self.processes.take_call(id, report);
        let (sight, focus) = self.sight_of(id);
        if fresh && counts(sight, focus) {
            self.processes.raise(id, text.clone(), SystemTime::now());
            self.notify_desktop(id, &text);
            self.waiting_changed(cx);
        }
    }

    /// With no IVE window active, a desktop notification `<title> — <worktree>` saying `text`.
    fn notify_desktop(&mut self, id: ProcessId, text: &str) {
        if self.active_window.is_any() {
            return;
        }
        let Some(process) = self.processes.get(id) else {
            return;
        };
        let title = match &process.checkout {
            Some(checkout) => format!("{} — {}", process.title(), self.checkout_label(checkout)),
            None => process.title(),
        };
        self.desktop_notices.send(id, &title, text);
    }

    /// Takes process `id`'s desktop notification away, as it ends.
    pub(super) fn withdraw_notice(&mut self, id: ProcessId) {
        self.desktop_notices.withdraw(id);
    }

    /// The waiting processes the user now sees, their tab shown in an active window, stop waiting.
    pub(crate) fn look_at_shown(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<ProcessId> = self.processes.waiting().iter().map(|process| process.id).collect();
        let seen: Vec<ProcessId> = ids.into_iter().filter(|id| self.is_seen(*id)).collect();
        // in front, the bell lists what still waits: the desktop needs none of IVE's notifications
        if self.active_window.is_any() {
            self.desktop_notices.withdraw_all();
        }
        let mut cleared = false;
        for id in seen {
            cleared |= self.processes.clear_attention(id);
            self.desktop_notices.withdraw(id);
        }
        if cleared {
            self.waiting_changed(cx);
        }
    }

    fn is_seen(&self, id: ProcessId) -> bool {
        self.sight_of(id) == (Sight::Shown, WindowFocus::Active)
    }

    /// Whether process `id`'s tab is the one its window shows, and whether that window has the keyboard; a process
    /// without a tab is hidden in the main window.
    fn sight_of(&self, id: ProcessId) -> (Sight, WindowFocus) {
        let focus = |handle| self.active_window.focus_of(handle);
        let tab = self.tabs.find(|tab| tab.shows_process(id));
        match tab {
            Some(tab) => match self.tab_windows.handle_of(tab) {
                // a detached window shows its one tab
                Some(handle) => (Sight::Shown, focus(handle)),
                None if self.tabs.active_id() == Some(tab) => (Sight::Shown, focus(self.main_window)),
                None => (Sight::Hidden, focus(self.main_window)),
            },
            None => (Sight::Hidden, focus(self.main_window)),
        }
    }

    /// Redraws what shows waiting processes: the tree, the tab row, an open hub and the windows of detached tabs.
    fn waiting_changed(&mut self, cx: &mut Context<Self>) {
        // the bell goes with the last waiting process: its list must not open again with the next one
        if self.processes.waiting().is_empty() && self.header_menu == Some(HeaderMenu::Waiting) {
            self.header_menu = None;
        }
        self.tree_changed(cx);
        self.settle_open_hub(cx);
        self.redraw_tab_windows(cx);
        cx.notify();
    }
}
