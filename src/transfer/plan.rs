//! Where copied or moved files go, and what is in the way there: pure, from what the disk was found to hold.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::Verb;
use crate::project::CheckoutRef;
use crate::sidebar::drop_place::DropPlace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    File,
    Folder,
}

/// A dragged or pasted file or folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// absolute
    pub path: PathBuf,
    pub checkout: CheckoutRef,
    pub kind: SourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// every source straight into the target folder, by its name
    Flat,
    /// every source at its path below the sources' deepest common folder
    KeepFolders,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    pub verb: Verb,
    pub sources: Vec<Source>,
    pub target: DropPlace,
    pub layout: Layout,
}

/// How a drop lays its sources out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutChoice {
    /// all from one folder: they go straight in
    Straight,
    /// from folders of one checkout: the drop asks, keeping folders below `common` or flat
    Ask { common: PathBuf },
    /// from several checkouts: no common folder means anything
    Flat,
}

pub fn layout_choice(sources: &[Source]) -> LayoutChoice {
    let checkouts: HashSet<&Path> = sources.iter().map(|source| source.checkout.path.as_path()).collect();
    if checkouts.len() > 1 {
        return LayoutChoice::Flat;
    }
    let parents: HashSet<&Path> = sources.iter().filter_map(|source| source.path.parent()).collect();
    match parents.len() {
        0 | 1 => LayoutChoice::Straight,
        _ => LayoutChoice::Ask { common: common_folder(sources) },
    }
}

/// The deepest folder holding every source.
pub fn common_folder(sources: &[Source]) -> PathBuf {
    let mut parents = sources.iter().filter_map(|source| source.path.parent());
    let first = parents.next().map(Path::to_path_buf).unwrap_or_default();
    parents.fold(first, |common, parent| common.ancestors().find(|ancestor| parent.starts_with(ancestor)).map(Path::to_path_buf).unwrap_or_default())
}

/// Moving within one checkout, copying into another; `forced` by the drop's modifiers wins.
pub fn verb_for(sources: &[Source], target: &DropPlace, forced: Option<Verb>) -> Verb {
    let same = sources.iter().all(|source| source.checkout.path == target.checkout.path);
    forced.unwrap_or(if same { Verb::Move } else { Verb::Copy })
}

/// Why a drop cannot happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    IntoItself { name: String },
    AlreadyHere,
}

impl Refusal {
    /// As the hint says it; `copy_name` is the copy modifier's.
    pub fn text(&self, copy_name: &str) -> String {
        match self {
            Self::IntoItself { name } => format!("Cannot put {name} inside itself"),
            Self::AlreadyHere => format!("Already here; {copy_name}+drop copies"),
        }
    }
}

/// Refuses a drop into one of the dragged folders, and a move of what already sits in the target folder.
pub fn refusal(sources: &[Source], target: &DropPlace, verb: Verb) -> Option<Refusal> {
    if let Some(folder) = sources.iter().find(|source| source.kind == SourceKind::Folder && target.folder.starts_with(&source.path)) {
        return Some(Refusal::IntoItself { name: name_of(&folder.path) });
    }
    let staying = sources.iter().all(|source| source.path.parent() == Some(target.folder.as_path()));
    (verb == Verb::Move && staying).then_some(Refusal::AlreadyHere)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomingKind {
    File,
    Symlink,
    EmptyFolder,
}

/// A file (or symlink, or empty folder) a source carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incoming {
    /// its source, an index into the operation's sources
    pub top: usize,
    /// absolute
    pub path: PathBuf,
    /// below its source; empty for the source itself
    pub relative: PathBuf,
    pub kind: IncomingKind,
    pub size: u64,
}

/// What is at a destination already.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Existing {
    File { size: u64, modified: SystemTime },
    Folder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conflict {
    /// a file is there, not compared yet
    Exists,
    Same,
    /// lines only in the incoming file, lines only in the one there
    Differs { added: usize, removed: usize },
    DiffersBinary,
    /// another incoming file, from `first`, lands on the same name
    Twice { first: PathBuf },
    FolderThere,
    FileThere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Replace,
    Skip,
    KeepBoth,
}

/// What can be done about `conflict`: no Replace where nothing could be replaced by one file.
pub fn offered(conflict: &Conflict) -> &'static [Action] {
    match conflict {
        Conflict::Twice { .. } | Conflict::FolderThere | Conflict::FileThere => &[Action::Skip, Action::KeepBoth],
        _ => &[Action::Replace, Action::Skip, Action::KeepBoth],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRow {
    pub incoming: Incoming,
    /// absolute
    pub destination: PathBuf,
    pub there: Option<Existing>,
    pub conflict: Option<Conflict>,
    /// what is done with it; `None` for a file going in plainly, and for a conflict not decided yet
    pub action: Option<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub operation: Operation,
    pub rows: Vec<PlanRow>,
    /// the folder kept below, when keeping folders
    pub common: Option<PathBuf>,
    /// nested checkouts the walk left out, for the line saying how it went
    pub left_out: usize,
}

/// Where each of `incoming` goes for `operation`, and what `existing` says is there. A source copied onto itself
/// goes beside itself as one copy (`src (2)`); a file a move leaves where it is keeps its name.
pub fn plan(operation: Operation, incoming: Vec<Incoming>, existing: impl Fn(&Path) -> Option<Existing>) -> Plan {
    let common = (operation.layout == Layout::KeepFolders).then(|| common_folder(&operation.sources));
    let tops: Vec<PathBuf> = operation
        .sources
        .iter()
        .map(|source| {
            let top = top_destination(&operation.target, source, common.as_deref());
            match operation.verb == Verb::Copy && top == source.path {
                true => free_name(&top, |path| existing(path).is_some()),
                false => top,
            }
        })
        .collect();
    let placed: Vec<(Incoming, PathBuf)> = incoming
        .into_iter()
        .map(|incoming| {
            let base = tops[incoming.top].clone();
            let destination = if incoming.relative.as_os_str().is_empty() { base } else { base.join(&incoming.relative) };
            (incoming, destination)
        })
        .collect();
    // what stays where it is holds its name before anything else can land on it
    let mut planned: HashMap<PathBuf, PathBuf> =
        placed.iter().filter(|(incoming, destination)| incoming.path == *destination).map(|(incoming, destination)| (destination.clone(), incoming.path.clone())).collect();
    let rows = placed.into_iter().map(|(incoming, destination)| planned_row(Planned { incoming, destination }, &mut planned, &existing)).collect();
    Plan { operation, rows, common, left_out: 0 }
}

/// Where `source` itself goes in `target`: at its path below `common` when keeping folders, else by its name.
pub fn top_destination(target: &DropPlace, source: &Source, common: Option<&Path>) -> PathBuf {
    match common {
        Some(common) => target.folder.join(source.path.strip_prefix(common).unwrap_or(&source.path)),
        None => target.folder.join(name_of(&source.path)),
    }
}

struct Planned {
    incoming: Incoming,
    destination: PathBuf,
}

fn planned_row(planned: Planned, taken: &mut HashMap<PathBuf, PathBuf>, existing: &impl Fn(&Path) -> Option<Existing>) -> PlanRow {
    let Planned { incoming, destination } = planned;
    let row = |there, conflict, action| PlanRow { incoming: incoming.clone(), destination: destination.clone(), there, conflict, action };
    if incoming.path == destination {
        // moved onto itself: it is there already
        return row(None, None, Some(Action::Skip));
    }
    if let Some(first) = taken.get(&destination) {
        return row(None, Some(Conflict::Twice { first: first.clone() }), None);
    }
    taken.insert(destination.clone(), incoming.path.clone());
    let there = existing(&destination);
    let conflict = match (there, incoming.kind) {
        (None, _) | (Some(Existing::Folder), IncomingKind::EmptyFolder) => None,
        (Some(Existing::Folder), _) => Some(Conflict::FolderThere),
        (Some(Existing::File { .. }), IncomingKind::EmptyFolder) => Some(Conflict::FileThere),
        (Some(Existing::File { .. }), _) => Some(Conflict::Exists),
    };
    row(there, conflict, None)
}

impl Plan {
    pub fn conflicts(&self) -> impl Iterator<Item = &PlanRow> {
        self.rows.iter().filter(|row| row.conflict.is_some())
    }

    pub fn clean(&self) -> impl Iterator<Item = &PlanRow> {
        self.rows.iter().filter(|row| row.conflict.is_none())
    }

    pub fn undecided(&self) -> usize {
        self.conflicts().filter(|row| row.action.is_none()).count()
    }

    /// Every conflict that offers `action` gets it.
    pub fn set_all(&mut self, action: Action) {
        for row in &mut self.rows {
            if row.conflict.as_ref().is_some_and(|conflict| offered(conflict).contains(&action)) {
                row.action = Some(action);
            }
        }
    }

    /// The names Keep both gives, by row: `name (N).ext`, the lowest N from 2 that `exists` does not report and no
    /// incoming file or earlier kept copy takes. Worked out once, before anything is written, so the Conflicts tab
    /// shows what is written.
    pub fn kept_names(&self, exists: impl Fn(&Path) -> bool) -> HashMap<usize, PathBuf> {
        let mut taken: HashSet<PathBuf> = self.rows.iter().map(|row| row.destination.clone()).collect();
        let mut kept = HashMap::new();
        for (index, row) in self.rows.iter().enumerate().filter(|(_, row)| row.action == Some(Action::KeepBoth)) {
            let name = free_name(&row.destination, |path| taken.contains(path) || exists(path));
            taken.insert(name.clone());
            kept.insert(index, name);
        }
        kept
    }

    /// `copies 4, replaces 1, skips 1 · 1 to choose`.
    pub fn summary(&self) -> String {
        let count = |test: &dyn Fn(&PlanRow) -> bool| self.rows.iter().filter(|row| test(row)).count();
        let plain = if self.operation.verb == Verb::Move { "moves" } else { "copies" };
        let parts = [
            (count(&|row| row.conflict.is_none() && row.action != Some(Action::Skip)), plain),
            (count(&|row| row.conflict.is_some() && row.action == Some(Action::Replace)), "replaces"),
            (count(&|row| row.conflict.is_some() && row.action == Some(Action::KeepBoth)), "keeps both of"),
            (count(&|row| row.action == Some(Action::Skip)), "skips"),
        ];
        let said: Vec<String> = parts.iter().filter(|(count, _)| *count > 0).map(|(count, verb)| format!("{verb} {count}")).collect();
        let said = if said.is_empty() { "nothing".to_owned() } else { said.join(", ") };
        match self.undecided() {
            0 => said,
            undecided => format!("{said} · {undecided} to choose"),
        }
    }

    /// `Copy 7 files from GEN-817-ive, IDP-30 to dev · internal-apps/ive/mock`, checkouts named by `label`.
    pub fn title(&self, label: impl Fn(&CheckoutRef) -> String) -> String {
        let verb = if self.operation.verb == Verb::Move { "Move" } else { "Copy" };
        let mut from: Vec<String> = Vec::new();
        for source in &self.operation.sources {
            let name = label(&source.checkout);
            if !from.contains(&name) {
                from.push(name);
            }
        }
        let files = counted(self.rows.len(), "file", "files");
        format!("{verb} {files} from {} to {}", from.join(", "), place_name(&self.operation.target, &label))
    }

    /// `flat: every file straight into mock/`, or `keeping folders, from internal-apps/ive/`.
    pub fn layout_text(&self) -> String {
        let target = &self.operation.target;
        match &self.common {
            None => format!("flat: every file straight into {}", folder_name(&target.folder, &target.checkout.path)),
            Some(common) => {
                let root = self.operation.sources.first().map(|source| source.checkout.path.as_path()).unwrap_or(common);
                format!("keeping folders, from {}", relative_folder(common, root))
            }
        }
    }
}

/// `dev · internal-apps/ive/mock`, `dev · /` for its root.
pub fn place_name(place: &DropPlace, label: impl Fn(&CheckoutRef) -> String) -> String {
    let inside = place.folder.strip_prefix(&place.checkout.path).ok().filter(|inside| !inside.as_os_str().is_empty());
    let inside = inside.map_or_else(|| "/".to_owned(), |inside| inside.display().to_string());
    format!("{} · {inside}", label(&place.checkout))
}

/// `name (N).ext`, the lowest N from 2 that `taken` refuses.
pub fn free_name(path: &Path, taken: impl Fn(&Path) -> bool) -> PathBuf {
    let name = name_of(path);
    let (stem, extension) = match name.rfind('.') {
        Some(dot) if dot > 0 => (&name[..dot], &name[dot..]),
        _ => (name.as_str(), ""),
    };
    let parent = path.parent().unwrap_or(Path::new("/"));
    (2..).map(|number| parent.join(format!("{stem} ({number}){extension}"))).find(|candidate| !taken(candidate)).unwrap_or_else(|| path.to_path_buf())
}

pub fn counted(count: usize, one: &str, many: &str) -> String {
    match count {
        1 => format!("1 {one}"),
        count => format!("{count} {many}"),
    }
}

fn name_of(path: &Path) -> String {
    path.file_name().map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned())
}

/// `mock/`, or `/` for the checkout's root.
fn folder_name(folder: &Path, root: &Path) -> String {
    if folder == root { "/".to_owned() } else { format!("{}/", name_of(folder)) }
}

/// `internal-apps/ive/`, or `/` for the root.
fn relative_folder(folder: &Path, root: &Path) -> String {
    match folder.strip_prefix(root) {
        Ok(inside) if !inside.as_os_str().is_empty() => format!("{}/", inside.display()),
        _ => "/".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;

    use super::*;
    use crate::project::ProjectId;

    const GEN: &str = "/a/.claude/worktrees/GEN-817-ive";
    const IDP: &str = "/a-wt/IDP-30";
    const DEV: &str = "/a";

    fn checkout(path: &str) -> CheckoutRef {
        CheckoutRef { project: ProjectId::new("atlassian"), path: PathBuf::from(path) }
    }

    fn file(at: &str, relative: &str) -> Source {
        Source { path: Path::new(at).join(relative), checkout: checkout(at), kind: SourceKind::File }
    }

    fn folder(at: &str, relative: &str) -> Source {
        Source { path: Path::new(at).join(relative), checkout: checkout(at), kind: SourceKind::Folder }
    }

    fn into(at: &str, relative: &str) -> DropPlace {
        DropPlace { checkout: checkout(at), folder: Path::new(at).join(relative) }
    }

    /// Each source as one incoming file, a folder as its files `children`.
    fn incoming(sources: &[Source], children: &[(usize, &str)]) -> Vec<Incoming> {
        let whole = sources.iter().enumerate().filter(|(_, source)| source.kind == SourceKind::File).map(|(top, source)| Incoming {
            top,
            path: source.path.clone(),
            relative: PathBuf::new(),
            kind: IncomingKind::File,
            size: 10,
        });
        let inside = children.iter().map(|(top, relative)| Incoming {
            top: *top,
            path: sources[*top].path.join(relative),
            relative: PathBuf::from(relative),
            kind: IncomingKind::File,
            size: 10,
        });
        whole.chain(inside).collect()
    }

    fn there(files: &'static [&'static str]) -> impl Fn(&Path) -> Option<Existing> {
        move |path| files.iter().any(|file| Path::new(file) == path).then_some(Existing::File { size: 1, modified: SystemTime::UNIX_EPOCH })
    }

    /// The mock's: GEN-817's mock.css and index.html, IDP-30's mock.css, run.sh, Cargo.toml and src/ copied flat into
    /// dev's internal-apps/ive/mock, where mock.css and index.html are.
    fn mock_plan() -> Plan {
        let sources = vec![
            file(GEN, "internal-apps/ive/mock/mock.css"),
            file(GEN, "internal-apps/ive/mock/index.html"),
            file(IDP, "internal-apps/ive/mock/mock.css"),
            file(GEN, "internal-apps/ive/run.sh"),
            file(GEN, "internal-apps/ive/Cargo.toml"),
            folder(GEN, "internal-apps/ive/src"),
        ];
        let arriving = incoming(&sources, &[(5, "main.rs"), (5, "theme.rs")]);
        let operation = Operation { verb: Verb::Copy, sources, target: into(DEV, "internal-apps/ive/mock"), layout: Layout::Flat };
        plan(operation, arriving, there(&["/a/internal-apps/ive/mock/mock.css", "/a/internal-apps/ive/mock/index.html"]))
    }

    #[test]
    fn should_go_straight_in_from_one_folder() {
        // when
        let choice = layout_choice(&[file(GEN, "src/a.rs"), file(GEN, "src/b.rs")]);

        // then
        assert_eq!(choice, LayoutChoice::Straight);
    }

    #[test]
    fn should_ask_from_folders_of_one_checkout() {
        // when
        let choice = layout_choice(&[file(GEN, "internal-apps/ive/src/a.rs"), file(GEN, "internal-apps/misc/pom.xml")]);

        // then
        assert_eq!(choice, LayoutChoice::Ask { common: Path::new(GEN).join("internal-apps") });
    }

    #[test]
    fn should_go_flat_from_several_checkouts() {
        // when
        let choice = layout_choice(&[file(GEN, "src/a.rs"), file(IDP, "lib/b.rs")]);

        // then
        assert_eq!(choice, LayoutChoice::Flat);
    }

    #[test]
    fn should_move_within_one_checkout_and_copy_across() {
        // when
        let verbs = [verb_for(&[file(GEN, "a.rs")], &into(GEN, "lib"), None), verb_for(&[file(GEN, "a.rs")], &into(DEV, "lib"), None)];

        // then
        assert_eq!(verbs, [Verb::Move, Verb::Copy]);
    }

    #[test]
    fn should_force_verb_with_modifier() {
        // when
        let verbs = [verb_for(&[file(GEN, "a.rs")], &into(GEN, "lib"), Some(Verb::Copy)), verb_for(&[file(GEN, "a.rs")], &into(DEV, "lib"), Some(Verb::Move))];

        // then
        assert_eq!(verbs, [Verb::Copy, Verb::Move]);
    }

    #[test]
    fn should_refuse_drop_into_itself() {
        // when
        let refused = refusal(&[folder(GEN, "src")], &into(GEN, "src/sidebar"), Verb::Copy);

        // then
        assert_eq!(refused, Some(Refusal::IntoItself { name: "src".to_owned() }));
        assert_eq!(refused.unwrap().text("Ctrl"), "Cannot put src inside itself");
    }

    #[test]
    fn should_refuse_move_onto_own_folder() {
        // when
        let refused = refusal(&[file(GEN, "src/a.rs"), file(GEN, "src/b.rs")], &into(GEN, "src"), Verb::Move);

        // then
        assert_eq!(refused.map(|refused| refused.text("Ctrl")), Some("Already here; Ctrl+drop copies".to_owned()));
    }

    #[test]
    fn should_refuse_move_into_subfolder_of_dragged_folder() {
        // when
        let refused = refusal(&[folder(GEN, "src"), file(GEN, "README.md")], &into(GEN, "src"), Verb::Move);

        // then
        assert_eq!(refused, Some(Refusal::IntoItself { name: "src".to_owned() }));
    }

    #[test]
    fn should_allow_copy_onto_own_folder() {
        // when
        let refused = refusal(&[file(GEN, "src/a.rs")], &into(GEN, "src"), Verb::Copy);

        // then
        assert_eq!(refused, None);
    }

    #[test]
    fn should_keep_folders_below_common_folder() {
        // given
        let sources = vec![file(GEN, "internal-apps/ive/src/a.rs"), file(GEN, "internal-apps/misc/pom.xml")];
        let arriving = incoming(&sources, &[]);
        let operation = Operation { verb: Verb::Copy, sources, target: into(DEV, "lib"), layout: Layout::KeepFolders };

        // when
        let planned = plan(operation, arriving, there(&[]));

        // then
        let destinations: Vec<&Path> = planned.rows.iter().map(|row| row.destination.as_path()).collect();
        assert_eq!(destinations, [Path::new("/a/lib/ive/src/a.rs"), Path::new("/a/lib/misc/pom.xml")]);
        assert_eq!(planned.common, Some(Path::new(GEN).join("internal-apps")));
    }

    #[test]
    fn should_place_flat_by_name() {
        // when
        let planned = mock_plan();

        // then
        let destinations: Vec<PathBuf> = planned.rows.iter().map(|row| row.destination.clone()).collect();
        let mock = Path::new("/a/internal-apps/ive/mock");
        let names = ["mock.css", "index.html", "mock.css", "run.sh", "Cargo.toml", "src/main.rs", "src/theme.rs"];
        assert_eq!(destinations, names.map(|name| mock.join(name)));
    }

    #[test]
    fn should_mark_existing_file_as_conflict() {
        // when
        let planned = mock_plan();

        // then
        assert_eq!(planned.rows[0].conflict, Some(Conflict::Exists));
        assert_eq!(planned.rows[1].conflict, Some(Conflict::Exists));
        assert_eq!(planned.clean().count(), 4);
    }

    #[test]
    fn should_mark_two_incoming_files_of_one_name_twice() {
        // when
        let planned = mock_plan();

        // then
        let twice = &planned.rows[2];
        assert_eq!(twice.conflict, Some(Conflict::Twice { first: Path::new(GEN).join("internal-apps/ive/mock/mock.css") }));
        assert_eq!(offered(twice.conflict.as_ref().unwrap()), [Action::Skip, Action::KeepBoth]);
    }

    #[test]
    fn should_mark_folder_in_the_way() {
        // given
        let sources = vec![file(GEN, "src")];
        let arriving = incoming(&sources, &[]);
        let operation = Operation { verb: Verb::Copy, sources, target: into(DEV, ""), layout: Layout::Flat };

        // when
        let planned = plan(operation, arriving, |path| (path == Path::new("/a/src")).then_some(Existing::Folder));

        // then
        assert_eq!(planned.rows[0].conflict, Some(Conflict::FolderThere));
    }

    #[test]
    fn should_copy_file_onto_itself_beside_it() {
        // given
        let sources = vec![file(GEN, "src/a.rs")];
        let arriving = incoming(&sources, &[]);
        let operation = Operation { verb: Verb::Copy, sources, target: into(GEN, "src"), layout: Layout::Flat };

        // when
        let planned = plan(operation, arriving, there(&["/a/.claude/worktrees/GEN-817-ive/src/a.rs"]));

        // then
        assert_eq!(planned.rows[0].destination, Path::new(GEN).join("src/a (2).rs"));
        assert_eq!((planned.rows[0].conflict.clone(), planned.rows[0].action), (None, None));
    }

    #[test]
    fn should_copy_folder_onto_itself_as_one_copy() {
        // given
        let sources = vec![folder(GEN, "src")];
        let arriving = incoming(&sources, &[(0, "a.rs"), (0, "b/c.rs")]);
        let operation = Operation { verb: Verb::Copy, sources, target: into(GEN, ""), layout: Layout::Flat };

        // when
        let planned = plan(operation, arriving, |path| (path == Path::new(GEN).join("src")).then_some(Existing::Folder));

        // then
        let destinations: Vec<PathBuf> = planned.rows.iter().map(|row| row.destination.clone()).collect();
        assert_eq!(destinations, [Path::new(GEN).join("src (2)/a.rs"), Path::new(GEN).join("src (2)/b/c.rs")]);
        assert_eq!(planned.conflicts().count(), 0);
    }

    #[test]
    fn should_keep_file_already_in_place_on_flat_move() {
        // given
        let sources = vec![file(GEN, "src/a/x.txt"), file(GEN, "src/z/x.txt")];
        let arriving = incoming(&sources, &[]);
        let operation = Operation { verb: Verb::Move, sources, target: into(GEN, "src/z"), layout: Layout::Flat };

        // when
        let planned = plan(operation, arriving, there(&["/a/.claude/worktrees/GEN-817-ive/src/z/x.txt"]));

        // then
        assert_eq!(planned.rows[0].conflict, Some(Conflict::Twice { first: Path::new(GEN).join("src/z/x.txt") }));
        assert_eq!((planned.rows[1].conflict.clone(), planned.rows[1].action), (None, Some(Action::Skip)));
    }

    #[test]
    fn should_name_kept_copy_after_earlier_rows() {
        // given
        let mut planned = mock_plan();
        planned.set_all(Action::KeepBoth);
        let taken = there(&["/a/internal-apps/ive/mock/mock.css", "/a/internal-apps/ive/mock/index.html"]);

        // when
        let names = planned.kept_names(|path| taken(path).is_some());

        // then
        assert_eq!([&names[&0], &names[&2]], [Path::new("/a/internal-apps/ive/mock/mock (2).css"), Path::new("/a/internal-apps/ive/mock/mock (3).css")]);
    }

    #[test]
    fn should_sum_up_actions() {
        // given
        let mut planned = mock_plan();
        planned.rows[0].action = Some(Action::Replace);
        planned.rows[1].action = Some(Action::Skip);

        // when
        let summary = planned.summary();

        // then
        assert_eq!(summary, "copies 4, replaces 1, skips 1 · 1 to choose");
    }

    #[test]
    fn should_count_undecided() {
        // given
        let mut planned = mock_plan();

        // when
        let before = planned.undecided();
        planned.set_all(Action::Skip);

        // then
        assert_eq!((before, planned.undecided()), (3, 0));
    }

    #[test]
    fn should_set_all_only_where_offered() {
        // given
        let mut planned = mock_plan();

        // when
        planned.set_all(Action::Replace);

        // then
        let actions: Vec<Option<Action>> = planned.conflicts().map(|row| row.action).collect();
        assert_eq!(actions, [Some(Action::Replace), Some(Action::Replace), None]);
    }

    #[test]
    fn should_title_as_the_mock() {
        // given
        let planned = mock_plan();
        let label = |checkout: &CheckoutRef| match checkout.path.to_str() {
            Some(GEN) => "GEN-817-ive".to_owned(),
            Some(IDP) => "IDP-30-reserved-subdomain".to_owned(),
            _ => "dev".to_owned(),
        };

        // when
        let (title, layout) = (planned.title(label), planned.layout_text());

        // then
        assert_eq!(title, "Copy 7 files from GEN-817-ive, IDP-30-reserved-subdomain to dev · internal-apps/ive/mock");
        assert_eq!(layout, "flat: every file straight into mock/");
    }
}
