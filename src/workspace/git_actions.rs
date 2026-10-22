//! The Git items of menus: a file's diff of uncommitted changes or against the base branch, the history of a path,
//! and adding paths to `.gitignore`.

use std::path::PathBuf;

use gpui::{Context, Window};

use super::Workspace;
use crate::git::ignore_file;
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
