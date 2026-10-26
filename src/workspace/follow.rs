//! Terminals follow their working directory: a process whose directory lies in another checkout moves there.

use std::path::{Path, PathBuf};

use gpui::Context;

use super::Workspace;
use crate::editor::Tab;
use crate::process::{ProcessId, ProcessKind, Processes};
use crate::project::CheckoutRef;

impl Workspace {
    /// What the foreground poll found: the processes whose directory changed, and where they are now.
    pub(super) fn apply_directories(&mut self, directories: Vec<(ProcessId, PathBuf)>, cx: &mut Context<Self>) {
        let ids = directories.iter().map(|(id, _)| *id).collect();
        for (id, directory) in directories {
            self.processes.set_directory(id, directory);
        }
        self.follow_directories(ids, cx);
    }

    /// After a scan: a directory seen before its worktree was listed may be in a checkout now.
    pub(super) fn follow_all_directories(&mut self, cx: &mut Context<Self>) {
        let ids = self.processes.iter().filter(|process| process.directory().is_some()).map(|process| process.id).collect();
        self.follow_directories(ids, cx);
    }

    /// Moves each of `ids` into the checkout its directory is in; the tree then finds a moved one whose tab is
    /// shown, as Select Opened File does.
    fn follow_directories(&mut self, ids: Vec<ProcessId>, cx: &mut Context<Self>) {
        let active = match self.tabs.active() {
            Some(Tab::Process(id)) => Some(*id),
            _ => None,
        };
        let (moved, select) = moves(&self.processes, &ids, |path| self.checkout_of_path(path), active);
        if moved.is_empty() {
            return;
        }
        for (id, checkout) in moved {
            self.processes.move_to(id, checkout);
        }
        if select {
            self.select_opened(cx);
        } else {
            self.tree_changed(cx);
        }
    }
}

/// Which of `ids` move, and where: a terminal or Claude Code session whose directory lies in a checkout other than
/// its own (`checkout_of` names it), unless it was moved there by hand from that very directory. Anywhere else it
/// stays. Also whether `active`, the shown process, is among them.
fn moves<V>(
    processes: &Processes<V>,
    ids: &[ProcessId],
    checkout_of: impl Fn(&Path) -> Option<CheckoutRef>,
    active: Option<ProcessId>,
) -> (Vec<(ProcessId, CheckoutRef)>, bool) {
    let moved: Vec<_> = ids
        .iter()
        .filter_map(|id| processes.get(*id))
        .filter(|process| process.kind != ProcessKind::Run)
        // moved by hand: it stays until its directory changes
        .filter(|process| process.pinned().is_none() || process.pinned() != process.directory())
        .filter_map(|process| {
            let target = checkout_of(process.directory()?)?;
            (process.checkout.as_ref() != Some(&target)).then_some((process.id, target))
        })
        .collect();
    let select = active.is_some_and(|shown| moved.iter().any(|(id, _)| *id == shown));
    (moved, select)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::project::ProjectId;

    fn checkout(path: &str) -> CheckoutRef {
        CheckoutRef { project: ProjectId::new("a"), path: PathBuf::from(path) }
    }

    /// Checkouts `/a` and those `known` names, the deepest containing one winning, as `checkout_of_path` picks.
    fn checkout_of<'a>(known: &'a [&'a str]) -> impl Fn(&Path) -> Option<CheckoutRef> + 'a {
        move |path| {
            known.iter().chain(&["/a"]).filter(|root| path.starts_with(root)).max_by_key(|root| root.len()).map(|root| checkout(root))
        }
    }

    fn terminal_in(processes: &mut Processes<()>, at: &str, directory: &str) -> ProcessId {
        let id = processes.start(Some(checkout(at)), |_| Ok::<_, ()>(())).unwrap();
        processes.set_directory(id, PathBuf::from(directory));
        id
    }

    #[test]
    fn should_move_process_into_deepest_checkout() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = terminal_in(&mut processes, "/a", "/a/.claude/worktrees/x/src");

        // when
        let (moved, _) = moves(&processes, &[id], checkout_of(&["/a/.claude/worktrees/x"]), None);

        // then
        assert_eq!(moved, [(id, checkout("/a/.claude/worktrees/x"))]);
    }

    #[test]
    fn should_keep_process_outside_every_checkout() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let outside = terminal_in(&mut processes, "/a/wt", "/tmp");
        let inside = terminal_in(&mut processes, "/a", "/a/src");

        // when
        let (moved, _) = moves(&processes, &[outside, inside], checkout_of(&["/a/wt"]), None);

        // then
        assert!(moved.is_empty());
    }

    #[test]
    fn should_select_only_a_shown_moved_process() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let background = terminal_in(&mut processes, "/a", "/a/wt");
        let shown = terminal_in(&mut processes, "/a", "/a/src");

        // when
        let selects = [background, shown].map(|active| moves(&processes, &[background, shown], checkout_of(&["/a/wt"]), Some(active)).1);

        // then
        assert_eq!(selects, [true, false]);
    }

    #[test]
    fn should_move_process_once_its_directory_becomes_a_checkout() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = terminal_in(&mut processes, "/b", "/b/.claude/worktrees/new");
        let before_scan = moves(&processes, &[id], |_| None, None).0;

        // when
        let (after_scan, _) = moves(&processes, &[id], checkout_of(&["/b/.claude/worktrees/new"]), None);

        // then
        assert!(before_scan.is_empty());
        assert_eq!(after_scan, [(id, checkout("/b/.claude/worktrees/new"))]);
    }

    #[test]
    fn should_leave_runs_where_they_started() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let origin = crate::process::RunOrigin {
            folder: PathBuf::from("/a/misc"),
            key: "app".to_owned(),
            name: "app".to_owned(),
            url: None,
            life: crate::process::RunLife::Running,
        };
        let run = processes.start_run(Some(checkout("/a")), origin, |_| Ok::<_, ()>(())).unwrap();
        processes.set_directory(run, PathBuf::from("/a/wt"));

        // when
        let (moved, _) = moves(&processes, &[run], checkout_of(&["/a/wt"]), None);

        // then
        assert!(moved.is_empty());
    }

    #[test]
    fn should_keep_moved_process_while_directory_unchanged() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = terminal_in(&mut processes, "/a", "/a/src");
        processes.move_pinned(id, checkout("/a/wt"));

        // when
        let (moved, _) = moves(&processes, &[id], checkout_of(&["/a/wt"]), None);

        // then
        assert!(moved.is_empty());
        assert_eq!(processes.get(id).unwrap().checkout, Some(checkout("/a/wt")));
    }

    #[test]
    fn should_follow_moved_process_once_directory_changes() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = terminal_in(&mut processes, "/a", "/a/src");
        processes.move_pinned(id, checkout("/a/wt"));

        // when
        processes.set_directory(id, PathBuf::from("/a/src/x"));
        let (moved, _) = moves(&processes, &[id], checkout_of(&["/a/wt"]), None);

        // then
        assert_eq!(moved, [(id, checkout("/a"))]);
    }

    #[test]
    fn should_keep_moved_process_when_directory_leaves_every_checkout() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = terminal_in(&mut processes, "/a", "/a/src");
        processes.move_pinned(id, checkout("/a/wt"));

        // when
        processes.set_directory(id, PathBuf::from("/tmp"));
        let (moved, _) = moves(&processes, &[id], checkout_of(&["/a/wt"]), None);

        // then
        assert!(moved.is_empty());
        assert_eq!(processes.get(id).unwrap().pinned(), None);
    }

    #[test]
    fn should_keep_moved_process_whose_directory_was_not_known_yet() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = processes.start(Some(checkout("/a")), |_| Ok::<_, ()>(())).unwrap();
        processes.move_pinned(id, checkout("/a/wt"));

        // when
        processes.set_directory(id, PathBuf::from("/a/src"));
        let (moved, _) = moves(&processes, &[id], checkout_of(&["/a/wt"]), None);

        // then
        assert!(moved.is_empty());
        assert_eq!(processes.get(id).unwrap().pinned(), Some(std::path::Path::new("/a/src")));
    }
}
