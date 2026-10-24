//! Writing a copy or move: each file under a temporary name beside its destination, then renamed into place, never
//! over a file that was not chosen to be replaced. Blocking I/O: run it off the UI thread.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::Verb;
use super::plan::{Action, IncomingKind, Layout, Operation, Plan, PlanRow, SourceKind, common_folder, counted, place_name, top_destination};
use super::scan::existing;
use crate::project::CheckoutRef;
use crate::rename::rename_new;

/// Makes temporary names unique within this IVE.
static TEMPORARY: AtomicUsize = AtomicUsize::new(0);

/// What the line under the tree counts: the files written, or the folders and files a move renamed whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counted {
    Files,
    Items,
}

/// What a copy or move did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub written: Vec<PathBuf>,
    pub skipped: usize,
    /// the first file that could not be written, and why; nothing after it was tried
    pub failed: Option<(PathBuf, String)>,
    /// nested checkouts a folder held, not copied
    pub left_out: usize,
    /// what was to be written: the plan's rows less the skipped ones
    pub total: usize,
    pub counted: Counted,
}

/// Writes every row of `plan` but the skipped ones, in order, stopping at the first failure; a move removes what it
/// moved, and the folders it left empty.
pub fn execute(plan: &Plan) -> Report {
    let skipped = plan.rows.iter().filter(|row| is_skipped(row)).count();
    let mut report = Report { written: Vec::new(), skipped, failed: None, left_out: 0, total: plan.rows.len() - skipped, counted: Counted::Files };
    let kept = plan.kept_names(|path| fs::symlink_metadata(path).is_ok());
    for (index, row) in plan.rows.iter().enumerate().filter(|(_, row)| !is_skipped(row)) {
        let destination = kept.get(&index).cloned().unwrap_or_else(|| row.destination.clone());
        match write_row(row, &destination, plan.operation.verb) {
            Ok(()) => report.written.push(destination),
            Err(why) => {
                report.failed = Some((destination, why));
                break;
            }
        }
    }
    if plan.operation.verb == Verb::Move {
        plan.operation.sources.iter().filter(|source| source.kind == SourceKind::Folder).for_each(|source| prune_empty(&source.path));
    }
    report
}

/// Skipped by choice, or a conflict left undecided.
fn is_skipped(row: &PlanRow) -> bool {
    row.action == Some(Action::Skip) || (row.conflict.is_some() && row.action.is_none())
}

fn write_row(row: &PlanRow, destination: &Path, verb: Verb) -> Result<(), String> {
    let replace = row.action == Some(Action::Replace);
    if replace && existing(destination) != row.there {
        return Err("changed since the Conflicts tab opened; left as it is".to_owned());
    }
    let parent = destination.parent().ok_or_else(|| format!("{} has no folder", destination.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let source = &row.incoming.path;
    match (row.incoming.kind, verb) {
        (IncomingKind::EmptyFolder, _) => fs::create_dir_all(destination).map_err(|error| error.to_string()),
        (_, Verb::Move) => moved(source, destination, replace),
        (kind, Verb::Copy) => copied(source, destination, (kind, replace)),
    }
}

/// Renames `source` to `destination`; across file systems, copies it and removes `source`.
fn moved(source: &Path, destination: &Path, replace: bool) -> Result<(), String> {
    let renamed = if replace { fs::rename(source, destination) } else { rename_new(source, destination) };
    match renamed {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {
            let kind = if fs::symlink_metadata(source).is_ok_and(|metadata| metadata.file_type().is_symlink()) { IncomingKind::Symlink } else { IncomingKind::File };
            copied(source, destination, (kind, replace))?;
            fs::remove_file(source).map_err(|error| format!("copied, but cannot remove {}: {error}", source.display()))
        }
        Err(error) => Err(error.to_string()),
    }
}

/// Copies `source` under a temporary name beside `destination`, then renames it into place; the temporary file goes
/// when anything fails.
fn copied(source: &Path, destination: &Path, (kind, replace): (IncomingKind, bool)) -> Result<(), String> {
    let temporary = temporary_beside(destination);
    let written = match kind {
        IncomingKind::Symlink => copy_link(source, &temporary),
        _ => fs::copy(source, &temporary).map(|_| ()),
    };
    let placed = written.and_then(|()| if replace { fs::rename(&temporary, destination) } else { rename_new(&temporary, destination) });
    placed.map_err(|error| {
        let _ = fs::remove_file(&temporary);
        error.to_string()
    })
}

#[cfg(unix)]
fn copy_link(source: &Path, temporary: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(fs::read_link(source)?, temporary)
}

#[cfg(not(unix))]
fn copy_link(source: &Path, temporary: &Path) -> io::Result<()> {
    fs::copy(source, temporary).map(|_| ())
}

fn temporary_beside(destination: &Path) -> PathBuf {
    let name = destination.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let number = TEMPORARY.fetch_add(1, Ordering::Relaxed);
    destination.with_file_name(format!(".{name}.ive-copy-{}-{number}", std::process::id()))
}

/// Removes the folders under `folder`, and `folder` itself, that hold nothing.
fn prune_empty(folder: &Path) {
    let Ok(children) = fs::read_dir(folder) else { return };
    for child in children.filter_map(Result::ok) {
        if child.file_type().is_ok_and(|kind| kind.is_dir()) {
            prune_empty(&child.path());
        }
    }
    let _ = fs::remove_dir(folder);
}

/// A move on one file system whose top-level destinations are all free and distinct: each source renamed whole,
/// nothing walked. `None`, having changed nothing, when a name is taken or wanted twice, a source is on another file
/// system, or a source folder holds one of `other_checkouts` (the walk refuses that).
pub fn move_whole(operation: &Operation, other_checkouts: &[PathBuf]) -> Option<Report> {
    if operation.verb != Verb::Move || !same_file_system(operation) {
        return None;
    }
    let holds_checkout = |source: &super::plan::Source| other_checkouts.iter().any(|checkout| checkout.starts_with(&source.path) && *checkout != source.path);
    if operation.sources.iter().any(holds_checkout) {
        return None;
    }
    let common = (operation.layout == Layout::KeepFolders).then(|| common_folder(&operation.sources));
    let destinations: Vec<PathBuf> = operation.sources.iter().map(|source| top_destination(&operation.target, source, common.as_deref())).collect();
    let distinct: std::collections::HashSet<&PathBuf> = destinations.iter().collect();
    if distinct.len() < destinations.len() || destinations.iter().any(|destination| fs::symlink_metadata(destination).is_ok()) {
        return None;
    }
    let mut report = Report { written: Vec::new(), skipped: 0, failed: None, left_out: 0, total: destinations.len(), counted: Counted::Items };
    for (source, destination) in operation.sources.iter().zip(destinations) {
        let renamed = destination.parent().map_or(Ok(()), fs::create_dir_all).and_then(|()| rename_new(&source.path, &destination));
        match renamed {
            Ok(()) => report.written.push(destination),
            Err(error) => {
                report.failed = Some((destination, error.to_string()));
                break;
            }
        }
    }
    Some(report)
}

#[cfg(unix)]
fn same_file_system(operation: &Operation) -> bool {
    use std::os::unix::fs::MetadataExt;
    let device = |path: &Path| fs::symlink_metadata(path).map(|metadata| metadata.dev()).ok();
    let target = device(&operation.target.folder);
    target.is_some() && operation.sources.iter().all(|source| device(&source.path) == target)
}

#[cfg(not(unix))]
fn same_file_system(_operation: &Operation) -> bool {
    false
}

impl Report {
    /// `Copied 7 files to dev · internal-apps/ive/mock · skipped 1`, or the first failure:
    /// `Copied 5 of 7 files to dev · lib: lib/a.rs: Permission denied (os error 13)`.
    pub fn line(&self, operation: &Operation, label: impl Fn(&CheckoutRef) -> String) -> String {
        let verb = if operation.verb == Verb::Move { "Moved" } else { "Copied" };
        let place = place_name(&operation.target, label);
        let (one, many) = match self.counted {
            Counted::Files => ("file", "files"),
            Counted::Items => ("item", "items"),
        };
        if let Some((path, why)) = &self.failed {
            let shown = path.strip_prefix(&operation.target.checkout.path).unwrap_or(path);
            return format!("{verb} {} of {} {many} to {place}: {}: {why}", self.written.len(), self.total, shown.display());
        }
        let mut line = format!("{verb} {} to {place}", counted(self.written.len(), one, many));
        if self.skipped > 0 {
            line.push_str(&format!(" · skipped {}", self.skipped));
        }
        if self.left_out > 0 {
            line.push_str(&format!(" · left out {}", counted(self.left_out, "worktree", "worktrees")));
        }
        line
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::project::{CheckoutRef, ProjectId};
    use crate::sidebar::drop_place::DropPlace;
    use crate::transfer::plan::{Action, Layout, Operation, Source, SourceKind, plan};
    use crate::transfer::scan::{classify, existing, walk};

    fn checkout(root: &Path) -> CheckoutRef {
        CheckoutRef { project: ProjectId::new("p"), path: root.to_path_buf() }
    }

    fn source(root: &Path, relative: &str, kind: SourceKind) -> Source {
        Source { path: root.join(relative), checkout: checkout(root), kind }
    }

    /// A plan of `sources` into `folder` of `root`, as the drop would make it.
    fn planned(sources: Vec<Source>, root: &Path, folder: &str, verb: Verb) -> Plan {
        let walked = walk(&sources, verb, &[]).unwrap();
        let target = DropPlace { checkout: checkout(root), folder: root.join(folder) };
        let mut planned = plan(Operation { verb, sources, target, layout: Layout::Flat }, walked.incoming, existing);
        classify(&mut planned);
        planned
    }

    fn tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("from/sub")).unwrap();
        fs::create_dir_all(dir.path().join("to")).unwrap();
        fs::write(dir.path().join("from/a.txt"), "new a").unwrap();
        fs::write(dir.path().join("from/sub/b.txt"), "b").unwrap();
        dir
    }

    fn label(_checkout: &CheckoutRef) -> String {
        "dev".to_owned()
    }

    #[cfg(unix)]
    #[test]
    fn should_copy_files_keeping_permissions() {
        // given
        use std::os::unix::fs::PermissionsExt;
        let dir = tree();
        fs::set_permissions(dir.path().join("from/a.txt"), fs::Permissions::from_mode(0o750)).unwrap();
        let plan = planned(vec![source(dir.path(), "from/a.txt", SourceKind::File)], dir.path(), "to", Verb::Copy);

        // when
        let report = execute(&plan);

        // then
        let copied = dir.path().join("to/a.txt");
        assert_eq!((fs::read_to_string(&copied).unwrap(), report.written), ("new a".to_owned(), vec![copied.clone()]));
        assert_eq!(fs::metadata(&copied).unwrap().permissions().mode() & 0o777, 0o750);
        assert!(dir.path().join("from/a.txt").exists());
    }

    #[cfg(unix)]
    #[test]
    fn should_copy_symlink() {
        // given
        let dir = tree();
        std::os::unix::fs::symlink("a.txt", dir.path().join("from/link")).unwrap();
        let plan = planned(vec![source(dir.path(), "from/link", SourceKind::File)], dir.path(), "to", Verb::Copy);

        // when
        execute(&plan);

        // then
        assert_eq!(fs::read_link(dir.path().join("to/link")).unwrap(), PathBuf::from("a.txt"));
    }

    #[cfg(unix)]
    #[test]
    fn should_not_leave_temp_file_on_failure() {
        // given
        use std::os::unix::fs::PermissionsExt;
        let dir = tree();
        let plan = planned(vec![source(dir.path(), "from/a.txt", SourceKind::File)], dir.path(), "to", Verb::Copy);
        fs::set_permissions(dir.path().join("to"), fs::Permissions::from_mode(0o555)).unwrap();

        // when
        let report = execute(&plan);

        // then
        fs::set_permissions(dir.path().join("to"), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(report.failed.is_some());
        assert_eq!(fs::read_dir(dir.path().join("to")).unwrap().count(), 0);
    }

    #[test]
    fn should_replace_only_when_chosen() {
        // given
        let dir = tree();
        fs::write(dir.path().join("to/a.txt"), "old a").unwrap();
        let mut plan = planned(vec![source(dir.path(), "from/a.txt", SourceKind::File)], dir.path(), "to", Verb::Copy);
        plan.rows[0].action = Some(Action::Replace);

        // when
        execute(&plan);

        // then
        assert_eq!(fs::read_to_string(dir.path().join("to/a.txt")).unwrap(), "new a");
    }

    #[test]
    fn should_skip() {
        // given
        let dir = tree();
        fs::write(dir.path().join("to/a.txt"), "old a").unwrap();
        let mut plan = planned(vec![source(dir.path(), "from/a.txt", SourceKind::File)], dir.path(), "to", Verb::Copy);
        plan.rows[0].action = Some(Action::Skip);

        // when
        let report = execute(&plan);

        // then
        assert_eq!((fs::read_to_string(dir.path().join("to/a.txt")).unwrap().as_str(), report.skipped), ("old a", 1));
    }

    #[test]
    fn should_keep_both_with_free_name() {
        // given
        let dir = tree();
        fs::write(dir.path().join("to/a.txt"), "old a").unwrap();
        let mut plan = planned(vec![source(dir.path(), "from/a.txt", SourceKind::File)], dir.path(), "to", Verb::Copy);
        plan.rows[0].action = Some(Action::KeepBoth);

        // when
        execute(&plan);

        // then
        assert_eq!(fs::read_to_string(dir.path().join("to/a (2).txt")).unwrap(), "new a");
        assert_eq!(fs::read_to_string(dir.path().join("to/a.txt")).unwrap(), "old a");
    }

    #[test]
    fn should_not_touch_file_changed_since_plan() {
        // given
        let dir = tree();
        fs::write(dir.path().join("to/a.txt"), "old a").unwrap();
        let mut plan = planned(vec![source(dir.path(), "from/a.txt", SourceKind::File)], dir.path(), "to", Verb::Copy);
        plan.rows[0].action = Some(Action::Replace);
        fs::write(dir.path().join("to/a.txt"), "changed meanwhile").unwrap();

        // when
        let report = execute(&plan);

        // then
        assert_eq!(fs::read_to_string(dir.path().join("to/a.txt")).unwrap(), "changed meanwhile");
        assert!(report.failed.is_some_and(|(path, why)| path == dir.path().join("to/a.txt") && why.contains("changed")));
    }

    #[test]
    fn should_move_and_remove_emptied_folders() {
        // given
        let dir = tree();
        let plan = planned(vec![source(dir.path(), "from", SourceKind::Folder)], dir.path(), "to", Verb::Move);

        // when
        let report = execute(&plan);

        // then
        assert_eq!(report.failed, None);
        assert_eq!(fs::read_to_string(dir.path().join("to/from/sub/b.txt")).unwrap(), "b");
        assert!(!dir.path().join("from").exists());
    }

    #[test]
    fn should_keep_folders_of_skipped_files_on_move() {
        // given
        let dir = tree();
        fs::create_dir_all(dir.path().join("to/from/sub")).unwrap();
        fs::write(dir.path().join("to/from/sub/b.txt"), "other b").unwrap();
        let mut plan = planned(vec![source(dir.path(), "from", SourceKind::Folder)], dir.path(), "to", Verb::Move);
        plan.set_all(Action::Skip);

        // when
        execute(&plan);

        // then
        assert_eq!(fs::read_to_string(dir.path().join("from/sub/b.txt")).unwrap(), "b");
        assert!(!dir.path().join("from/a.txt").exists());
    }

    #[test]
    fn should_move_whole_folder_by_rename() {
        // given
        let dir = tree();
        let sources = vec![source(dir.path(), "from", SourceKind::Folder)];
        let operation = Operation { verb: Verb::Move, sources, target: DropPlace { checkout: checkout(dir.path()), folder: dir.path().join("to") }, layout: Layout::Flat };

        // when
        let report = move_whole(&operation, &[]).unwrap();

        // then
        assert_eq!(report.written, [dir.path().join("to/from")]);
        assert!(dir.path().join("to/from/sub/b.txt").exists() && !dir.path().join("from").exists());
    }

    #[test]
    fn should_refuse_whole_move_when_name_taken() {
        // given
        let dir = tree();
        fs::create_dir(dir.path().join("to/from")).unwrap();
        let sources = vec![source(dir.path(), "from", SourceKind::Folder)];
        let operation = Operation { verb: Verb::Move, sources, target: DropPlace { checkout: checkout(dir.path()), folder: dir.path().join("to") }, layout: Layout::Flat };

        // when
        let report = move_whole(&operation, &[]);

        // then
        assert!(report.is_none());
        assert!(dir.path().join("from/a.txt").exists());
    }

    #[test]
    fn should_not_move_whole_folder_holding_another_checkout() {
        // given
        let dir = tree();
        let sources = vec![source(dir.path(), "from", SourceKind::Folder)];
        let operation = Operation { verb: Verb::Move, sources, target: DropPlace { checkout: checkout(dir.path()), folder: dir.path().join("to") }, layout: Layout::Flat };

        // when
        let report = move_whole(&operation, &[dir.path().join("from/sub")]);

        // then
        assert!(report.is_none());
        assert!(dir.path().join("from/a.txt").exists());
    }

    #[test]
    fn should_not_move_whole_onto_one_name_twice() {
        // given
        let dir = tree();
        fs::write(dir.path().join("from/sub/a.txt"), "other a").unwrap();
        let sources = vec![source(dir.path(), "from/a.txt", SourceKind::File), source(dir.path(), "from/sub/a.txt", SourceKind::File)];
        let operation = Operation { verb: Verb::Move, sources, target: DropPlace { checkout: checkout(dir.path()), folder: dir.path().join("to") }, layout: Layout::Flat };

        // when
        let report = move_whole(&operation, &[]);

        // then
        assert!(report.is_none());
        assert!(dir.path().join("from/a.txt").exists() && dir.path().join("from/sub/a.txt").exists());
    }

    #[test]
    fn should_write_the_kept_names_the_tab_shows() {
        // given
        let dir = tree();
        fs::write(dir.path().join("from/sub/a.txt"), "other a").unwrap();
        fs::write(dir.path().join("to/a.txt"), "old a").unwrap();
        let sources = vec![source(dir.path(), "from/a.txt", SourceKind::File), source(dir.path(), "from/sub/a.txt", SourceKind::File)];
        let mut plan = planned(sources, dir.path(), "to", Verb::Copy);
        plan.set_all(Action::KeepBoth);
        let shown = plan.kept_names(|path| path.exists());

        // when
        execute(&plan);

        // then
        let written: Vec<String> = [0, 1].iter().map(|index| fs::read_to_string(&shown[index]).unwrap()).collect();
        assert_eq!(written, ["new a", "other a"]);
        assert_eq!(shown[&1], dir.path().join("to/a (3).txt"));
    }

    #[test]
    fn should_say_what_was_copied() {
        // given
        let dir = tree();
        fs::write(dir.path().join("to/a.txt"), "old a").unwrap();
        let mut plan = planned(vec![source(dir.path(), "from", SourceKind::Folder)], dir.path(), "to", Verb::Copy);
        fs::create_dir_all(dir.path().join("to/from")).unwrap();
        fs::write(dir.path().join("to/from/a.txt"), "old").unwrap();
        plan.rows[0].action = Some(Action::Skip);

        // when
        let report = Report { left_out: 2, ..execute(&plan) };

        // then
        assert_eq!(report.line(&plan.operation, label), "Copied 1 file to dev · to · skipped 1 · left out 2 worktrees");
    }

    #[test]
    fn should_say_first_failure() {
        // given
        let root = Path::new("/r");
        let target = DropPlace { checkout: checkout(root), folder: root.join("lib") };
        let operation = Operation { verb: Verb::Copy, sources: vec![source(root, "a.rs", SourceKind::File)], target, layout: Layout::Flat };
        let report = Report {
            written: vec![PathBuf::from("/r/lib/x.rs")],
            skipped: 0,
            failed: Some((PathBuf::from("/r/lib/a.rs"), "Permission denied (os error 13)".to_owned())),
            left_out: 0,
            total: 2,
            counted: Counted::Files,
        };

        // when
        let line = report.line(&operation, label);

        // then
        assert_eq!(line, "Copied 1 of 2 files to dev · lib: lib/a.rs: Permission denied (os error 13)");
    }
}
