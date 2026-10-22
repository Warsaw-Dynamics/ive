//! Edits reach the disk without the user saving: the workspace asks every editing panel to save now when the user
//! turns away from it (another tab, another window), before a run, and before quitting or closing its tab, which
//! wait for the saves; edits that cannot be saved can be given up with the tab's "Close without saving". What a
//! close or quit waits for is kept in [`super::waits::Waits`].

use std::time::Duration;

use codeview::host::Command;
use codeview::panel::{Cast, Event, Panel};
use gpui::{App, AsyncWindowContext, Context, Entity, WeakEntity, Window};

use super::Workspace;
use super::waits::{AfterDiscard, Forgot, Ticket};
use crate::editor::Tab;
use crate::file_tab;
use crate::tabs::TabId;

/// How often waiting for saves looks again.
const SAVE_POLL: Duration = Duration::from_millis(20);

impl Workspace {
    /// Every editing panel saves its edits now instead of after its pause.
    pub fn flush_edits(&self, cx: &mut App) {
        for (_, panel) in self.editing_panels(cx) {
            panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::Flush), cx));
        }
    }

    /// The panels of file and scratch tabs that edit text, with their tabs.
    fn editing_panels(&self, cx: &App) -> Vec<(TabId, Entity<Panel>)> {
        let panel = |tab: &Tab| match tab {
            Tab::Document(file) => Some(file.panel.clone()),
            Tab::Scratch(scratch) => scratch.editing_panel(cx),
            _ => None,
        };
        self.tabs.iter().filter_map(|(id, tab)| panel(tab).map(|panel| (id, panel))).collect()
    }

    /// A save is in flight in some panel, or one of a closed file tab is still being written.
    fn is_saving(&self, cx: &App) -> bool {
        file_tab::is_writing_any() || self.editing_panels(cx).iter().any(|(_, panel)| panel.read(cx).is_saving())
    }

    /// The first tab whose edits are not saved: a save failed or waits for the user.
    #[cfg(target_os = "macos")]
    pub(super) fn unsaved_tab(&self, cx: &App) -> Option<TabId> {
        self.unsaved_tabs(cx).into_iter().next()
    }

    fn unsaved_tabs(&self, cx: &App) -> Vec<TabId> {
        self.editing_panels(cx).into_iter().filter(|(_, panel)| panel.read(cx).has_unsaved(cx)).map(|(id, _)| id).collect()
    }

    /// Quits once every save landed; edits that could not be saved keep IVE open, their tab shown with the reason.
    /// It replaces any close or quit waiting.
    pub(super) fn quit_when_saved(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let ticket = self.waits.quit();
        self.withdraw_stale_asks(cx);
        self.quit_after_saves(ticket, window, cx);
    }

    /// Quit `ticket` goes on once every save landed, unless the user went on working meanwhile.
    fn quit_after_saves(&mut self, ticket: Ticket, window: &mut Window, cx: &mut Context<Self>) {
        self.flush_edits(cx);
        cx.spawn_in(window, async move |workspace, cx| {
            saves_landed(&workspace, cx).await;
            let _ = workspace.update_in(cx, |workspace, window, cx| workspace.quit_unless_unsaved(ticket, window, cx));
        })
        .detach();
    }

    fn quit_unless_unsaved(&mut self, ticket: Ticket, window: &mut Window, cx: &mut Context<Self>) {
        if !self.waits.is_live(ticket) {
            return;
        }
        let unsaved = self.unsaved_tabs(cx);
        let Some(first) = unsaved.first().copied() else {
            return cx.quit();
        };
        self.quit_prompt = None;
        cx.cancel_quit();
        self.waits.quit_stopped(ticket, unsaved.clone());
        self.ask_close(&unsaved, window, cx);
        self.activate_tab(first, window, cx);
        cx.notify();
    }

    /// The user showed tab `id`: a close or quit not waiting for it is forgotten.
    pub(super) fn went_to_tab(&mut self, id: TabId, cx: &mut App) {
        if self.waits.activated(id) == Forgot::Request {
            self.withdraw_stale_asks(cx);
        }
    }

    /// The user edited tab `id`: a close or quit waiting for it is forgotten.
    fn edited(&mut self, id: TabId, cx: &mut App) {
        if self.waits.edited(id) == Forgot::Request {
            self.withdraw_stale_asks(cx);
        }
    }

    /// Panels asked to close for a request no longer live stop offering closing without saving.
    fn withdraw_stale_asks(&self, cx: &mut App) {
        let stale = self.editing_panels(cx).into_iter().filter(|(id, _)| !self.waits.waits_on(*id));
        for (_, panel) in stale {
            panel.update(cx, |panel, cx| panel.handle(Cast::CloseWithdrawn, cx));
        }
    }

    /// Tells the panels of tabs `ids` that a close waits for their edits, so their bars offer closing without
    /// saving, and closes the tab or goes on quitting when the user does; an edit there forgets the close.
    fn ask_close(&mut self, ids: &[TabId], window: &mut Window, cx: &mut Context<Self>) {
        let panels = self.editing_panels(cx);
        self.waits.watched.retain(|watched| panels.iter().any(|(_, panel)| panel.entity_id() == *watched));
        for (id, panel) in panels.into_iter().filter(|(id, _)| ids.contains(id)) {
            panel.update(cx, |panel, cx| panel.handle(Cast::CloseAsked, cx));
            if self.waits.watched.contains(&panel.entity_id()) {
                continue;
            }
            self.waits.watched.push(panel.entity_id());
            cx.subscribe_in(&panel, window, move |workspace, _panel, event: &Event, window, cx| match event {
                Event::DiscardRequested => workspace.discarded(id, window, cx),
                Event::Edited { .. } => workspace.edited(id, cx),
                _ => {}
            })
            .detach();
        }
    }

    /// The user chose closing tab `id` without saving: its panel dropped the edits, so a file tab closes (a scratch
    /// tab stays and reloads) and the close or quit waiting for it goes on.
    fn discarded(&mut self, id: TabId, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(Tab::Document(_)) = self.tabs.get(id) {
            self.tabs.close(id);
        }
        self.tab_shown(window, cx);
        match self.waits.discarded(id) {
            AfterDiscard::Quit(ticket) => self.quit_after_saves(ticket, window, cx),
            AfterDiscard::CloseProjects => self.finish_closing_projects(window, cx),
            AfterDiscard::Nothing => {}
        }
        cx.notify();
    }

    /// Closes tabs `ids` as quitting does: a file tab with edits is flushed and closes once its saves landed; one
    /// whose edits cannot be saved stays, shown with the reason. Other tabs close at once. Returns the close.
    pub fn close_tabs(&mut self, ids: Vec<TabId>, window: &mut Window, cx: &mut Context<Self>) -> Ticket {
        let ticket = self.begin_close(&ids, cx);
        let (waiting, settled): (Vec<_>, Vec<_>) = ids.into_iter().partition(|id| self.is_unsettled(*id, cx));
        for id in settled {
            self.tabs.close(id);
        }
        // flushes the waiting tabs' edits
        self.tab_shown(window, cx);
        self.close_when_saved(waiting, ticket, window, cx);
        // a project waiting for these tabs closes with the last of them
        self.finish_closing_projects(window, cx);
        ticket
    }

    /// A close of tabs `ids`: it goes on with the close waiting for them, or replaces any close or quit waiting.
    pub(super) fn begin_close(&mut self, ids: &[TabId], cx: &mut App) -> Ticket {
        let ticket = self.waits.close(ids);
        self.withdraw_stale_asks(cx);
        ticket
    }

    /// Whether tab `id` is a file tab whose edits are not on disk yet: waiting for a pause, in flight, or failed.
    pub(super) fn is_unsettled(&self, id: TabId, cx: &App) -> bool {
        match self.tabs.get(id) {
            Some(Tab::Document(file)) => file.panel.read(cx).has_unsaved(cx) || file.panel.read(cx).is_saving(),
            _ => false,
        }
    }

    /// Closes the flushed file tabs `ids` of close `ticket` once their saves landed, unless edits stay unsaved.
    pub(super) fn close_when_saved(&mut self, ids: Vec<TabId>, ticket: Ticket, window: &mut Window, cx: &mut Context<Self>) {
        if ids.is_empty() {
            return;
        }
        cx.spawn_in(window, async move |workspace, cx| {
            tabs_saved(&workspace, &ids, cx).await;
            let _ = workspace.update_in(cx, |workspace, window, cx| workspace.close_saved_tabs(&ids, ticket, window, cx));
        })
        .detach();
    }

    /// Closes the tabs of `ids` whose edits are all saved; the first of the others is shown, with the reason, unless
    /// close `ticket` was forgotten or replaced meanwhile: they then stay as they are.
    fn close_saved_tabs(&mut self, ids: &[TabId], ticket: Ticket, window: &mut Window, cx: &mut Context<Self>) {
        let (unsaved, saved): (Vec<TabId>, Vec<TabId>) = ids.iter().partition(|id| self.is_unsettled(**id, cx));
        for id in saved {
            self.tabs.close(id);
        }
        let shown = unsaved.first().copied().filter(|_| self.waits.is_live(ticket));
        match shown {
            Some(tab) => {
                self.ask_close(&unsaved, window, cx);
                self.activate_tab(tab, window, cx);
            }
            None => self.tab_shown(window, cx),
        }
        self.finish_closing_projects(window, cx);
        cx.notify();
    }

    /// Whether a save of one of tabs `ids` is in flight.
    fn is_saving_any_of(&self, ids: &[TabId], cx: &App) -> bool {
        self.editing_panels(cx).iter().any(|(id, panel)| ids.contains(id) && panel.read(cx).is_saving())
    }
}

/// Waits until no panel of `workspace` has a save in flight, or the workspace is gone.
pub(super) async fn saves_landed(workspace: &WeakEntity<Workspace>, cx: &mut AsyncWindowContext) {
    while workspace.read_with(cx, |workspace, cx| workspace.is_saving(cx)).unwrap_or(false) {
        cx.background_executor().timer(SAVE_POLL).await;
    }
}

/// Waits until no save of tabs `ids` is in flight, or the workspace is gone.
async fn tabs_saved(workspace: &WeakEntity<Workspace>, ids: &[TabId], cx: &mut AsyncWindowContext) {
    while workspace.read_with(cx, |workspace, cx| workspace.is_saving_any_of(ids, cx)).unwrap_or(false) {
        cx.background_executor().timer(SAVE_POLL).await;
    }
}
