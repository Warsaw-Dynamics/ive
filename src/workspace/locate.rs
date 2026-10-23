//! Select Opened File: the tree expands down to the shown tab's row, selects it and scrolls to it.

use std::path::PathBuf;

use gpui::{Context, ScrollStrategy};

use super::Workspace;
use crate::sidebar::rows::TreeKey;

impl Workspace {
    /// The crosshair button: finds the shown tab's file, worktree, process or project in the tree. Folders not read
    /// yet are read first, one level at a time, so the row may appear a moment later.
    pub(crate) fn select_opened(&mut self, cx: &mut Context<Self>) {
        // a tab of a project the picker left out shows it again
        if let Some(project) = self.tabs.active().and_then(|tab| tab.project(self)) {
            self.hidden_projects.remove(&project);
        }
        self.locating = self.tabs.active().and_then(|tab| self.tree_key_of(tab));
        self.continue_locating(cx);
        self.schedule_save(cx);
    }

    /// One step towards the row being located: expands what leads to it and reads what that asks for; once the row
    /// is there, selects it and scrolls to it. Stops when nothing is left to read and the row is still missing.
    pub(super) fn continue_locating(&mut self, cx: &mut Context<Self>) {
        let Some(key) = self.locating.clone() else {
            return;
        };
        let folders = self.expand_towards(&key);
        self.tree_changed(cx);
        let index = self.tree.rows.iter().position(|row| row.key() == key);
        if index.is_some() || folders.is_empty() {
            self.locating = None;
        }
        if index.is_none() && folders.is_empty() {
            self.locating_picks.clear();
        }
        if let Some(index) = index {
            match std::mem::take(&mut self.locating_picks) {
                picks if picks.is_empty() => self.tree.selection.select_only(key),
                picks => self.tree.selection.pick_all(picks),
            }
            self.tree.list.list_handle().scroll_to_item(index, ScrollStrategy::Center);
        }
        // read even when the row is found: folders_to_read hands out trees shown again only once
        self.read_folders(folders, cx);
    }

    /// Expands the project, checkout, processes and folders above the row of `key`; returns the folders to read.
    fn expand_towards(&mut self, key: &TreeKey) -> Vec<PathBuf> {
        let checkout = match key {
            TreeKey::Entry(path) => self.checkout_of_path(path),
            TreeKey::Process(id) => self.processes.get(*id).and_then(|process| process.checkout.clone()),
            TreeKey::Checkout(path) => self.checkout_of_path(path),
            TreeKey::Project(_) | TreeKey::Folder(..) | TreeKey::More(_) => None,
            // the git folders' rows are not revealed: no tab opens one of them as its own
            TreeKey::LiveRun(_) | TreeKey::Change(..) | TreeKey::ChangeFolder(..) | TreeKey::Commit(..) | TreeKey::CommitsMore(_) | TreeKey::Note(..) => None,
        };
        let Some(checkout) = checkout else {
            return Vec::new();
        };
        let process_index = match key {
            TreeKey::Process(id) => self.processes.by_checkout(&checkout.path).iter().position(|process| process.id == *id),
            _ => None,
        };
        // a hidden worktree is not in the tree: nothing of it is unfolded or read
        let Some(open) = self.open_project_mut(&checkout.project).filter(|open| !open.is_hidden(&checkout.path)) else {
            return Vec::new();
        };
        open.reveal(&checkout.path);
        if let Some(index) = process_index {
            open.reveal_process(&checkout.path, index);
        }
        let mut folders = open.folders_to_read();
        if let TreeKey::Entry(path) = key {
            folders.extend(open.reveal_entry(&checkout.path, path));
        }
        folders
    }
}
