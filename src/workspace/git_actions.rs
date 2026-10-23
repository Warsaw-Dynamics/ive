//! The Git items of menus: a file's diff of uncommitted changes or against the base branch, the history of a path,
//! and adding paths to `.gitignore`.

use std::path::PathBuf;

use gpui::{Context, Window};

use super::Workspace;
use crate::git::diff::FilePaths;
use crate::git::ignore_file;
use crate::git::types::GitPath;
use crate::overview::diff_tab::{DiffRequest, DiffSource, DiffTabSpec};
use crate::project::CheckoutRef;
use crate::menu::resolve::Applies;
use crate::menu::{Home, PathKind, Subject, Within};
use crate::script::keymap::Builtin;
use crate::sidebar::folders::ChangeSource;

impl Workspace {
    /// What Git action `builtin` does for `subject`; `None` when `builtin` is not one of them.
    pub(super) fn git_applies(&self, builtin: Builtin, subject: &Subject) -> Option<Applies> {
        let shown = match builtin {
            Builtin::GitShowDiff => self.changed_file(subject, &ChangeSource::Uncommitted).is_some(),
            Builtin::GitCompareWithBase => self.changed_file(subject, &ChangeSource::SinceBase).is_some(),
            Builtin::GitHistory => matches!(&subject.paths[..], [path] if matches!(path.within, Within::Checkout(_))),
            Builtin::CompareFiles => self.compared_files(subject).is_some(),
            Builtin::GitIgnore => {
                let inside = |within: &Within| matches!(within, Within::Checkout(relative) if relative != ".");
                matches!(subject.home, Home::Checkout(_)) && !subject.paths.is_empty() && subject.paths.iter().all(|path| inside(&path.within))
            }
            _ => return None,
        };
        Some(if shown { Applies::Shown { enabled: true } } else { Applies::Hidden })
    }

    /// Runs Git action `builtin` on `subject`; one that does not apply does nothing.
    pub(super) fn run_git_action(&mut self, builtin: Builtin, subject: &Subject, window: &mut Window, cx: &mut Context<Self>) {
        match builtin {
            Builtin::GitShowDiff => self.open_changed_file(subject, &ChangeSource::Uncommitted, window, cx),
            Builtin::GitCompareWithBase => self.open_changed_file(subject, &ChangeSource::SinceBase, window, cx),
            Builtin::GitHistory => self.open_history_of(subject, window, cx),
            Builtin::GitIgnore => self.add_to_gitignore(subject),
            Builtin::CompareFiles => self.compare_files(subject, window, cx),
            _ => {}
        }
    }

    /// The one file of `subject`, when the checkout's `source` folder lists it as IVE last read it.
    fn changed_file(&self, subject: &Subject, source: &ChangeSource) -> Option<(crate::project::CheckoutRef, crate::git::status::Change)> {
        let [path] = &subject.paths[..] else {
            return None;
        };
        let (Within::Checkout(relative), PathKind::File) = (&path.within, path.kind) else {
            return None;
        };
        let checkout = self.checkout_ref_of(&path.path)?;
        let change = self.known_change(&checkout.path, relative, source)?;
        Some((checkout, change))
    }

    fn open_changed_file(&mut self, subject: &Subject, source: &ChangeSource, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((checkout, change)) = self.changed_file(subject, source) {
            self.open_change(&checkout, source, &change, window, cx);
        }
    }

    fn open_history_of(&mut self, subject: &Subject, window: &mut Window, cx: &mut Context<Self>) {
        let [path] = &subject.paths[..] else {
            return;
        };
        let (Within::Checkout(relative), Some(checkout)) = (&path.within, self.checkout_ref_of(&path.path)) else {
            return;
        };
        // the checkout's root: the whole history of HEAD
        let relative = if relative == "." { PathBuf::new() } else { PathBuf::from(relative) };
        self.open_path_history(&checkout, relative, window, cx);
    }

    /// The two files of `subject`, left the first in the tree's order, with the checkout the tab is listed under.
    fn compared_files(&self, subject: &Subject) -> Option<(CheckoutRef, PathBuf, PathBuf)> {
        let [left, right] = &subject.paths[..] else {
            return None;
        };
        if left.kind != PathKind::File || right.kind != PathKind::File {
            return None;
        }
        let checkout = self.checkout_ref_of(&left.path).or_else(|| self.checkout_ref_of(&right.path))?;
        Some((checkout, left.path.clone(), right.path.clone()))
    }

    /// Compare Files: a diff tab of the two files, read in the background.
    fn compare_files(&mut self, subject: &Subject, window: &mut Window, cx: &mut Context<Self>) {
        let Some((checkout, left, right)) = self.compared_files(subject) else {
            return;
        };
        let file = FilePaths { path: GitPath::of_path(&right), old_path: Some(GitPath::of_path(&left)) };
        let request = DiffRequest { dir: checkout.path.clone(), file, source: DiffSource::Files { left, right } };
        let label = self.checkout_label(&checkout);
        let loading = cx.background_executor().spawn({
            let request = request.clone();
            async move { request.load() }
        });
        cx.spawn_in(window, async move |this, cx| {
            let loaded = loading.await;
            this.update_in(cx, |workspace, window, cx| match loaded {
                Ok(Some(file)) => {
                    workspace.open_diff_tab(&DiffTabSpec { checkout, label, request, file }, window, cx);
                }
                Ok(None) => {}
                Err(error) => eprintln!("ive: {error}"),
            })
            .ok();
        })
        .detach();
    }

    fn add_to_gitignore(&self, subject: &Subject) {
        let Home::Checkout(root) = &subject.home else {
            return;
        };
        let file = root.join(".gitignore");
        if let Err(error) = ignore_file::append(&file, &ignore_file::ignore_lines(subject)) {
            eprintln!("ive: cannot add to {}: {error}", file.display());
        }
    }
}
