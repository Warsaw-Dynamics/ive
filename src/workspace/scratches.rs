//! Workspace actions on scratches: where they run, opening, creating, renaming, deleting and running them.

use std::path::Path;

use gpui::{Context, Window};

use super::Workspace;
use crate::editor::Tab;
use crate::overview::scratches::{self, ScratchesTarget};
use crate::project::{CheckoutRef, ProjectId};
use crate::scratch::store::{ScratchError, ScratchStore};
use crate::scratch::{ScratchRef, ScratchTab, ScratchTarget, log_error, run, store_of};
use crate::tabs::TabId;

impl Workspace {
    /// The project's last focused checkout, else its main checkout (for a plain project, its folder).
    pub fn scratch_checkout(&self, project: &ProjectId) -> Option<CheckoutRef> {
        self.last_focused_checkout(project).or_else(|| {
            let project = self.project(project)?;
            Some(project.checkout_ref(project.main_checkout()))
        })
    }

    pub fn scratch_run_label(&self, project: &ProjectId) -> Option<String> {
        self.scratch_checkout(project).map(|checkout| self.checkout_label(&checkout))
    }

    /// Shows the scratch's tab, opening one when needed.
    pub fn open_scratch(&mut self, scratch: &ScratchRef, window: &mut Window, cx: &mut Context<Self>) -> Result<TabId, String> {
        if let Some(open) = self.scratch_tab(scratch) {
            self.activate_tab(open, window, cx);
            return Ok(open);
        }
        let tab = self.scratch_tab_of(scratch, cx)?;
        Ok(self.open_tab(Tab::Scratch(tab), window, cx))
    }

    /// A tab for `scratch`, not shown yet; fails when its project is not open or the file cannot be read.
    pub(super) fn scratch_tab_of(&self, scratch: &ScratchRef, cx: &mut Context<Self>) -> Result<ScratchTab, String> {
        let project = self.project(&scratch.project).ok_or_else(|| format!("no open project {}", scratch.project))?;
        let target = ScratchTarget {
            scratch: scratch.clone(),
            project_name: project.name.clone(),
            store: store_of(&scratch.project)?,
            workspace: cx.weak_entity(),
        };
        ScratchTab::open(target, cx).map_err(reported)
    }

    pub fn create_scratch(&mut self, scratch: &ScratchRef, window: &mut Window, cx: &mut Context<Self>) -> Result<TabId, String> {
        store_of(&scratch.project)?.create(&scratch.name).map_err(reported)?;
        self.open_scratch(scratch, window, cx)
    }

    /// Saves an open tab first, so pending edits move with the file.
    pub fn rename_scratch(&mut self, scratch: &ScratchRef, to: &str, cx: &mut Context<Self>) -> Result<(), String> {
        let store = store_of(&scratch.project)?;
        let open = self.scratch_tab(scratch);
        if let Some(Tab::Scratch(tab)) = open.and_then(|id| self.tabs.get(id)) {
            tab.save(cx);
        }
        store.rename(&scratch.name, to).map_err(reported)?;
        if let Some(Tab::Scratch(tab)) = open.and_then(|id| self.tabs.get_mut(id)) {
            tab.rename(to, cx);
        }
        self.sync_watches();
        cx.notify();
        Ok(())
    }

    /// Deletes the file, then closes its tab without saving; a failed delete keeps the tab and its edits.
    pub fn delete_scratch(&mut self, scratch: &ScratchRef, window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        store_of(&scratch.project)?.delete(&scratch.name).map_err(reported)?;
        if let Some(open) = self.scratch_tab(scratch) {
            if let Some(Tab::Scratch(tab)) = self.tabs.get(open) {
                tab.forget(cx);
            }
            self.close_tab(open, window, cx);
        }
        Ok(())
    }

    /// Closes the scratch's tab; what it writes on release is up to its view.
    pub fn close_scratch(&mut self, scratch: &ScratchRef, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(open) = self.scratch_tab(scratch) {
            self.close_tab(open, window, cx);
        }
    }

    /// Opens a terminal where the scratch runs and types `<shell> '<path>'` into it. The caller saves the scratch
    /// first.
    pub fn run_scratch(&mut self, scratch: &ScratchRef, window: &mut Window, cx: &mut Context<Self>) -> Result<TabId, String> {
        let path = store_of(&scratch.project)?.path(&scratch.name);
        let checkout = self.scratch_checkout(&scratch.project);
        self.run_shell_file(&path, checkout, window, cx)
    }

    /// Opens a terminal in `checkout` (the home folder for `None`) and types `<shell> '<path>'` into it, the shell
    /// that `path`'s extension names.
    pub fn run_shell_file(&mut self, path: &Path, checkout: Option<CheckoutRef>, window: &mut Window, cx: &mut Context<Self>) -> Result<TabId, String> {
        let shell = run::shell_for(&path.to_string_lossy()).ok_or_else(|| format!("{} is not a shell file (.sh, .bash, .zsh)", path.display()))?;
        let typed = run::command(shell, path)?;
        let terminal = self.open_terminal_in(checkout, window, cx);
        self.send_text_to_process(terminal, &typed)?;
        Ok(terminal)
    }

    /// Project overview `tab` shows its Scratches section: made on the first show, re-read on every later one.
    pub(super) fn scratches_shown(&mut self, tab: TabId, window: &mut Window, cx: &mut Context<Self>) {
        let workspace = cx.weak_entity();
        let Some(Tab::ProjectOverview(overview)) = self.tabs.get_mut(tab) else {
            return;
        };
        let project = overview.project.clone();
        let store = store_of(&project);
        // watched from now on, so it must exist before the first scratch does; without a store the section says why
        if let Some(Err(error)) = store.as_ref().ok().map(ScratchStore::ensure_dir) {
            log_error(&error);
        }
        let make = move || ScratchesTarget { workspace, store, project };
        scratches::shown(&mut overview.scratches, make, window, cx);
        self.sync_watches();
    }

    /// `Change::File` / `FileRemoved` for `path`: its scratch tab, if any, reloads or asks.
    pub(super) fn scratch_changed_on_disk(&mut self, path: &Path, cx: &mut Context<Self>) {
        if let Some(Tab::Scratch(tab)) = self.tabs.find(|tab| matches!(tab, Tab::Scratch(open) if open.path() == path)).and_then(|id| self.tabs.get(id)) {
            tab.view().update(cx, |view, cx| view.disk_changed(cx));
        }
    }

    /// `Change::Folder` for `dir`: the Scratches sections listing it re-read it.
    pub(super) fn scratch_folder_changed(&mut self, dir: &Path, cx: &mut Context<Self>) {
        for section in self.scratch_sections(|folder| folder == dir) {
            section.update(cx, |section, cx| section.reload(cx));
        }
    }

    /// Notifications were lost: every scratch tab checks its file, every Scratches section re-reads its folder.
    pub(super) fn rescan_scratches(&mut self, cx: &mut Context<Self>) {
        let views: Vec<_> = self.tabs.iter().filter_map(|(_, tab)| match tab {
            Tab::Scratch(scratch) => Some(scratch.view().clone()),
            _ => None,
        }).collect();
        for view in views {
            view.update(cx, |view, cx| view.disk_changed(cx));
        }
        for section in self.scratch_sections(|_| true) {
            section.update(cx, |section, cx| section.reload(cx));
        }
    }

    /// The Scratches sections of open project overviews whose folder passes `wanted`.
    fn scratch_sections(&self, wanted: impl Fn(&Path) -> bool) -> Vec<gpui::Entity<scratches::ScratchesSection>> {
        self.tabs
            .iter()
            .filter_map(|(_, tab)| match tab {
                Tab::ProjectOverview(overview) => overview.scratches.clone().filter(|_| store_of(&overview.project).is_ok_and(|store| wanted(store.dir()))),
                _ => None,
            })
            .collect()
    }

    fn scratch_tab(&self, scratch: &ScratchRef) -> Option<TabId> {
        self.tabs.find(|tab| tab.shows_scratch(scratch))
    }
}

/// The message to show where the user acted, once logged.
fn reported(error: ScratchError) -> String {
    log_error(&error);
    error.to_string()
}
