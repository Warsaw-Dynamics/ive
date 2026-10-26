//! Processes that called for the user: raised when the user does not see them, cleared once they do.

use std::time::SystemTime;

use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use gpui::{AnyWindowHandle, Context, Task, Window};

use super::Workspace;
use crate::attention::{Sight, TabAt, WindowFocus, counts, sight};
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
    /// Window `handle`, the main one or a detached tab's, became active or stopped being so; programs learn its
    /// focus state, and processes it shows while active stop waiting.
    pub(crate) fn window_activated(&mut self, handle: AnyWindowHandle, focus: WindowFocus, cx: &mut Context<Self>) {
        self.active_window.observe(handle, focus);
        self.look_at_shown(cx);
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
        // the program sets its title: cleaned and bounded as what it said
        let title = match &process.checkout {
            Some(checkout) => crate::terminal::one_line(&format!("{} — {}", process.title(), self.checkout_label(checkout))),
            None => crate::terminal::one_line(&process.title()),
        };
        self.desktop_notices.send(id, &title, text);
    }

    /// Takes process `id`'s desktop notification away, as it ends.
    pub(super) fn withdraw_notice(&mut self, id: ProcessId) {
        self.desktop_notices.withdraw(id);
    }

    /// The waiting processes the user now sees, their tab shown in an active window, stop waiting.
    pub(crate) fn look_at_shown(&mut self, cx: &mut Context<Self>) {
        self.report_terminal_focus();
        let ids: Vec<ProcessId> = self.processes.waiting().iter().map(|process| process.id).collect();
        let seen: Vec<ProcessId> = ids.into_iter().filter(|id| self.is_seen(*id)).collect();
        // in front, the bell lists what still waits: the desktop needs none of IVE's notifications
        if self.active_window.is_any() {
            self.desktop_notices.withdraw_all();
        }
        for id in &seen {
            self.desktop_notices.withdraw(*id);
        }
        let cleared = seen.into_iter().filter(|id| self.processes.clear_attention(*id)).count();
        if cleared > 0 {
            self.waiting_changed(cx);
        }
    }

    fn is_seen(&self, id: ProcessId) -> bool {
        self.sight_of(id) == (Sight::Shown, WindowFocus::Active)
    }

    /// Updates all terminals after a tab or window changes visibility.
    fn report_terminal_focus(&self) {
        for process in self.processes.iter() {
            process.view.report_focus(self.is_seen(process.id));
        }
    }

    /// A program may enable focus reporting after its terminal was already hidden.
    pub(super) fn report_process_focus(&self, id: ProcessId) {
        if let Some(process) = self.processes.get(id) {
            process.view.report_focus(self.is_seen(id));
        }
    }

    /// Whether process `id`'s tab is the one its window shows, and whether that window has the keyboard.
    fn sight_of(&self, id: ProcessId) -> (Sight, WindowFocus) {
        sight(self.tab_place(id), self.main_window, &self.active_window)
    }

    fn tab_place(&self, id: ProcessId) -> TabAt<AnyWindowHandle> {
        let Some(tab) = self.tabs.find(|tab| tab.shows_process(id)) else {
            return TabAt::Nowhere;
        };
        match self.tab_windows.handle_of(tab) {
            Some(window) => TabAt::Detached(window),
            None if self.tabs.active_id() == Some(tab) => TabAt::Shown,
            None => TabAt::Hidden,
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
