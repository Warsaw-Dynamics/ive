//! The file tree's git status per shown checkout: read on the background executor when a checkout's files are
//! first shown, and again at most once a tick after its files or its git dir changed.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::Context;

use super::Workspace;
use crate::git::status::{self, TreeStatus};

/// What is known, changed since, and being read, by checkout path.
#[derive(Debug, Default)]
pub(super) struct TreeStatuses {
    known: HashMap<PathBuf, TreeStatus>,
    dirty: HashSet<PathBuf>,
    reading: HashSet<PathBuf>,
}

impl TreeStatuses {
    pub(super) fn known(&self) -> &HashMap<PathBuf, TreeStatus> {
        &self.known
    }

    pub(super) fn mark_dirty(&mut self, checkout: PathBuf) {
        self.dirty.insert(checkout);
    }

    /// The checkouts of `shown` to read now: never read, or changed since their last read, and not being read; they
    /// count as being read from now on. What is not shown any more is forgotten, so it is read when shown again.
    pub(super) fn take_due(&mut self, shown: &[PathBuf]) -> Vec<PathBuf> {
        self.known.retain(|checkout, _| shown.contains(checkout));
        let due: Vec<PathBuf> = shown
            .iter()
            .filter(|checkout| !self.reading.contains(*checkout))
            .filter(|checkout| self.dirty.contains(*checkout) || !self.known.contains_key(*checkout))
            .cloned()
            .collect();
        for checkout in &due {
            self.dirty.remove(checkout);
            self.reading.insert(checkout.clone());
        }
        due
    }

    /// A read of `checkout` ended; a failed one (`None`) keeps what was known. Whether the status changed.
    pub(super) fn finish(&mut self, checkout: PathBuf, read: Option<TreeStatus>) -> bool {
        self.reading.remove(&checkout);
        let Some(status) = read else {
            return false;
        };
        // compared before it is stored, so a large status is never cloned on the window's thread
        if self.known.get(&checkout) == Some(&status) {
            return false;
        }
        self.known.insert(checkout, status);
        true
    }
}

impl Workspace {
    /// Checkouts whose files the tree shows.
    fn file_checkouts(&self) -> Vec<PathBuf> {
        let shown = self.projects.iter().flat_map(|open| open.shown_checkouts().into_iter().filter(|checkout| open.files(&checkout.path).is_some()));
        shown.map(|checkout| checkout.path.clone()).collect()
    }

    /// The shown checkout `path` is in, if any, as changed: its status is read again on the next tick.
    pub(super) fn tree_files_changed(&mut self, path: &Path) {
        let checkouts = self.file_checkouts();
        // the innermost: a worktree can sit inside its main checkout
        let checkout = checkouts.into_iter().filter(|checkout| path.starts_with(checkout)).max_by_key(|checkout| checkout.as_os_str().len());
        if let Some(checkout) = checkout {
            self.tree_git_changed_at(&checkout);
            self.tree_statuses.mark_dirty(checkout);
        }
    }

    /// Every shown checkout of the project whose git dir is `dir`, as changed.
    pub(super) fn tree_git_changed(&mut self, dir: &Path) {
        let Some(open) = self.projects.iter().find(|open| open.project.git_dir.as_deref() == Some(dir)) else {
            return;
        };
        let checkouts: Vec<PathBuf> = open.shown_checkouts().into_iter().map(|checkout| checkout.path.clone()).collect();
        for checkout in checkouts {
            self.tree_git_changed_at(&checkout);
            self.tree_statuses.mark_dirty(checkout);
        }
    }

    /// Reads, on the background executor, the status of the shown checkouts that need it; the tree redraws when one
    /// changed.
    pub(super) fn read_tree_statuses(&mut self, cx: &mut Context<Self>) {
        let since = self.status_since();
        for checkout in self.tree_statuses.take_due(&self.file_checkouts()) {
            let dir = checkout.clone();
            let reading = cx.background_executor().spawn(async move { status::shared(&dir, since).ok().map(|read| Arc::unwrap_or_clone(read.tree)) });
            cx.spawn(async move |this, cx| {
                let read = reading.await;
                this.update(cx, |workspace, cx| {
                    if workspace.tree_statuses.finish(checkout, read) {
                        workspace.tree_changed(cx);
                    }
                })
                .ok();
            })
            .detach();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::*;

    fn path(name: &str) -> PathBuf {
        PathBuf::from(format!("/r/{name}"))
    }

    fn ignoring(name: &str) -> TreeStatus {
        TreeStatus { ignored: HashSet::from([PathBuf::from(name)]), ..TreeStatus::default() }
    }

    #[test]
    fn should_read_unknown_shown_checkout_once() {
        // given
        let mut statuses = TreeStatuses::default();
        let shown = [path("a")];

        // when
        let (first, second) = (statuses.take_due(&shown), statuses.take_due(&shown));

        // then
        assert_eq!((first, second), (vec![path("a")], Vec::new()));
    }

    #[test]
    fn should_read_changed_checkout_again() {
        // given
        let mut statuses = TreeStatuses::default();
        statuses.take_due(&[path("a")]);
        statuses.finish(path("a"), Some(ignoring("x")));

        // when
        let unchanged = statuses.take_due(&[path("a")]);
        statuses.mark_dirty(path("a"));
        let changed = statuses.take_due(&[path("a")]);

        // then
        assert_eq!((unchanged, changed), (Vec::new(), vec![path("a")]));
    }

    #[test]
    fn should_read_again_when_changed_during_a_read() {
        // given
        let mut statuses = TreeStatuses::default();
        statuses.take_due(&[path("a")]);
        statuses.mark_dirty(path("a"));

        // when
        statuses.finish(path("a"), Some(ignoring("x")));
        let due = statuses.take_due(&[path("a")]);

        // then
        assert_eq!(due, vec![path("a")]);
    }

    #[test]
    fn should_keep_old_status_when_read_fails() {
        // given
        let mut statuses = TreeStatuses::default();
        statuses.take_due(&[path("a")]);
        statuses.finish(path("a"), Some(ignoring("x")));
        statuses.mark_dirty(path("a"));
        statuses.take_due(&[path("a")]);

        // when
        let changed = statuses.finish(path("a"), None);

        // then
        assert!(!changed);
        assert_eq!(statuses.known()[&path("a")], ignoring("x"));
    }

    #[test]
    fn should_forget_status_of_hidden_checkout() {
        // given
        let mut statuses = TreeStatuses::default();
        statuses.take_due(&[path("a")]);
        statuses.finish(path("a"), Some(ignoring("x")));

        // when
        statuses.take_due(&[]);

        // then
        assert!(statuses.known().is_empty());
        assert_eq!(statuses.take_due(&[path("a")]), vec![path("a")]);
    }
}
