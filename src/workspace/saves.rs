//! Edits reach the disk without the user saving: the workspace asks every editing panel to save now when the user
//! turns away from it (another tab, another window), before a run, and before quitting or closing its tab, which
//! wait for the saves.

use std::time::Duration;

use codeview::host::Command;
use codeview::panel::{Cast, Panel};
use gpui::{App, AsyncWindowContext, Context, Entity, WeakEntity, Window};

use super::Workspace;
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
    pub(super) fn unsaved_tab(&self, cx: &App) -> Option<TabId> {
        self.editing_panels(cx).into_iter().find(|(_, panel)| panel.read(cx).has_unsaved(cx)).map(|(id, _)| id)
    }

    /// Quits once every save landed; edits that could not be saved keep IVE open, their tab shown with the reason.
    pub(super) fn quit_when_saved(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.flush_edits(cx);
        cx.spawn_in(window, async move |workspace, cx| {
            saves_landed(&workspace, cx).await;
            let _ = workspace.update_in(cx, |workspace, window, cx| workspace.quit_unless_unsaved(window, cx));
        })
        .detach();
    }

    fn quit_unless_unsaved(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tab) = self.unsaved_tab(cx) else {
            return cx.quit();
        };
        self.quit_prompt = None;
        cx.cancel_quit();
        self.activate_tab(tab, window, cx);
        cx.notify();
    }

    /// Closes tabs `ids` as quitting does: a file tab with edits is flushed and closes once its saves landed; one
    /// whose edits cannot be saved stays, shown with the reason. Other tabs close at once.
    pub fn close_tabs(&mut self, ids: Vec<TabId>, window: &mut Window, cx: &mut Context<Self>) {
        let (waiting, settled): (Vec<_>, Vec<_>) = ids.into_iter().partition(|id| self.is_unsettled(*id, cx));
        for id in settled {
            self.tabs.close(id);
        }
        // flushes the waiting tabs' edits
        self.tab_shown(window, cx);
        self.close_when_saved(waiting, window, cx);
    }

    /// Whether tab `id` is a file tab whose edits are not on disk yet: waiting for a pause, in flight, or failed.
    pub(super) fn is_unsettled(&self, id: TabId, cx: &App) -> bool {
        match self.tabs.get(id) {
            Some(Tab::Document(file)) => file.panel.read(cx).has_unsaved(cx) || file.panel.read(cx).is_saving(),
            _ => false,
        }
    }

    /// Closes the flushed file tabs `ids` once their saves landed, unless edits stay unsaved.
    pub(super) fn close_when_saved(&mut self, ids: Vec<TabId>, window: &mut Window, cx: &mut Context<Self>) {
        if ids.is_empty() {
            return;
        }
        cx.spawn_in(window, async move |workspace, cx| {
            tabs_saved(&workspace, &ids, cx).await;
            let _ = workspace.update_in(cx, |workspace, window, cx| workspace.close_saved_tabs(&ids, window, cx));
        })
        .detach();
    }

    /// Closes the tabs of `ids` whose edits are all saved; the first of the others is shown, with the reason.
    fn close_saved_tabs(&mut self, ids: &[TabId], window: &mut Window, cx: &mut Context<Self>) {
        let (unsaved, saved): (Vec<TabId>, Vec<TabId>) = ids.iter().partition(|id| self.is_unsettled(**id, cx));
        for id in saved {
            self.tabs.close(id);
        }
        match unsaved.first() {
            Some(tab) => self.activate_tab(*tab, window, cx),
            None => self.tab_shown(window, cx),
        }
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
