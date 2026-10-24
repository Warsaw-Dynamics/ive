//! Processes that called for the user: raised when the user does not see them, cleared once they do.

use std::time::SystemTime;

use gpui::{App, Context};

use super::Workspace;
use crate::attention::{Sight, WindowFocus, counts};
use crate::process::ProcessId;

impl Workspace {
    /// Takes process `id`'s newest call, if any it has not taken: the process waits for the user unless they see it.
    pub(super) fn take_call(&mut self, id: ProcessId, cx: &mut Context<Self>) {
        let Some(report) = self.processes.get(id).and_then(|process| process.view.call()) else {
            return;
        };
        let text = report.call.text();
        let fresh = self.processes.take_call(id, report);
        let (sight, focus) = self.sight_of(id, cx);
        if fresh && counts(sight, focus) {
            self.processes.raise(id, text, SystemTime::now());
            self.waiting_changed(cx);
        }
    }

    /// The waiting processes the user now sees, their tab shown in an active window, stop waiting.
    pub(crate) fn look_at_shown(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<ProcessId> = self.processes.waiting().iter().map(|process| process.id).collect();
        let seen: Vec<ProcessId> = ids.into_iter().filter(|id| self.is_seen(*id, cx)).collect();
        let mut cleared = false;
        for id in seen {
            cleared |= self.processes.clear_attention(id);
        }
        if cleared {
            self.waiting_changed(cx);
        }
    }

    fn is_seen(&self, id: ProcessId, cx: &App) -> bool {
        self.sight_of(id, cx) == (Sight::Shown, WindowFocus::Active)
    }

    /// Whether process `id`'s tab is the one its window shows, and whether that window has the keyboard; a process
    /// without a tab is hidden in the main window.
    fn sight_of(&self, id: ProcessId, cx: &App) -> (Sight, WindowFocus) {
        let active = cx.active_window();
        let focus = |handle| if active == Some(handle) { WindowFocus::Active } else { WindowFocus::Inactive };
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
        self.tree_changed(cx);
        self.settle_open_hub(cx);
        self.redraw_tab_windows(cx);
        cx.notify();
    }
}
