use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::run;
use crate::script::hidden::Hidden;
use crate::run::file::{RUN_FILE, RunFile, RunFileError};

/// Caps one folder's listing so opening a huge directory (e.g. `/`) cannot stall the UI.
const MAX_ENTRIES: usize = 10_000;
/// The name of the row a folder's hidden entries (dot files and [`Hidden`]'s names) are shown under, once it has at least [`MIN_HIDDEN_GROUP`].
pub const HIDDEN_GROUP: &str = "@hidden";
const MIN_HIDDEN_GROUP: usize = 2;

/// The files of one checkout, shown as a tree whose folders are read when first expanded. The tree does not read
/// the disk itself: [`read_folder`] does, on the background executor, and [`FileTree::apply`] takes what it found.
pub struct FileTree {
    root: PathBuf,
    children: HashMap<PathBuf, Vec<Entry>>,
    expanded: HashSet<PathBuf>,
    /// folders whose hidden group is expanded
    expanded_hidden: HashSet<PathBuf>,
    hidden: Hidden,
    rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub kind: EntryKind,
    /// a folder's `.run.toml`, or a `.run.toml` file's own, read with the listing
    pub run: RunState,
}

/// A `.run.toml` as the listing found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunState {
    None,
    Loaded(Arc<RunFile>),
    Broken(RunFileError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Folder,
    /// a folder's hidden entries, under one row named [`HIDDEN_GROUP`]; its path is that name in the folder, which
    /// it shares with an entry of the same name, if any
    HiddenGroup,
}

impl Entry {
    /// Folders and hidden groups expand; files open.
    pub fn expands(&self) -> bool {
        self.kind != EntryKind::File
    }
}

/// One visible line of the tree: an entry, how deep it is below the project root and whether it is expanded.
#[derive(Debug, Clone)]
pub struct Row {
    pub entry: Entry,
    pub depth: usize,
    pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileTreeError {
    Unreadable { path: PathBuf, reason: String },
}

impl fmt::Display for FileTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, reason } => write!(f, "cannot read {}: {reason}", path.display()),
        }
    }
}

/// What reading one folder found.
#[derive(Debug, Clone)]
pub struct Listing {
    pub dir: PathBuf,
    pub entries: Result<Vec<Entry>, FileTreeError>,
}

/// What of a file tree is expanded, as `state.toml` keeps it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Expansion {
    pub folders: Vec<PathBuf>,
    /// the folders whose hidden group is expanded
    pub hidden: Vec<PathBuf>,
}

impl Expansion {
    pub fn is_empty(&self) -> bool {
        self.folders.is_empty() && self.hidden.is_empty()
    }
}

/// Lists `dir`: folders first, by name, at most 10,000 entries. Reads the disk; run it on the background executor.
#[tracing::instrument(level = "trace", skip_all, fields(dir = %dir.display()))]
pub fn read_folder(dir: &Path) -> Listing {
    Listing { dir: dir.to_path_buf(), entries: read_entries(dir).map_err(|source| unreadable(dir, &source)) }
}

impl FileTree {
    /// The tree of folder `root` (canonical), showing `entries`, its listing; only dot files go under [`HIDDEN_GROUP`]
    /// until [`Self::set_hidden`].
    pub fn new(root: PathBuf, entries: Vec<Entry>) -> Self {
        let mut tree = Self {
            children: HashMap::from([(root.clone(), entries)]),
            root,
            expanded: HashSet::new(),
            expanded_hidden: HashSet::new(),
            hidden: Hidden::default(),
            rows: Vec::new(),
        };
        tree.rebuild_rows();
        tree
    }

    /// The tree of `root` showing `entries`, with what `expansion` names expanded again; the folders it shows are
    /// read as [`Self::unread_folders`] names them.
    pub fn restored(root: PathBuf, entries: Vec<Entry>, expansion: &Expansion) -> Self {
        let mut tree = Self::new(root, entries);
        tree.expanded.extend(expansion.folders.iter().cloned());
        tree.expanded_hidden.extend(expansion.hidden.iter().cloned());
        tree.rebuild_rows();
        tree
    }

    /// What is expanded, sorted; leaves out what its folder's listing no longer has.
    pub fn expansion(&self) -> Expansion {
        let listed = |path: &&PathBuf| match path.parent().and_then(|parent| self.children.get(parent)) {
            Some(entries) => entries.iter().any(|entry| entry.path == **path),
            None => true,
        };
        let mut folders: Vec<PathBuf> = self.expanded.iter().filter(listed).cloned().collect();
        let mut hidden: Vec<PathBuf> = self.expanded_hidden.iter().filter(|dir| **dir == self.root || listed(dir)).cloned().collect();
        folders.sort();
        hidden.sort();
        Expansion { folders, hidden }
    }

    /// Shown folders never read: those expanded again by [`Self::restored`].
    pub fn unread_folders(&self) -> Vec<PathBuf> {
        self.shown_folders().into_iter().filter(|folder| !self.children.contains_key(folder)).collect()
    }

    /// Shows the entries `hidden` names under [`HIDDEN_GROUP`] from now on; returns the folders to read now, those
    /// it shows that were never read.
    pub fn set_hidden(&mut self, hidden: &Hidden) -> Vec<PathBuf> {
        if self.hidden == *hidden {
            return Vec::new();
        }
        self.hidden = hidden.clone();
        self.rebuild_rows();
        self.unread_folders()
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The entry at `path`, from its folder's last listing; `None` while that folder was not read.
    pub fn entry(&self, path: &Path) -> Option<&Entry> {
        self.children.get(path.parent()?)?.iter().find(|entry| entry.path == path)
    }

    /// Expands a collapsed folder or collapses an expanded one, and returns the folders to read now: an expanded
    /// folder shows its last listing until the new one comes. Folders expanded below a collapsed one stay expanded
    /// when it is opened again, and are read again too, since nothing watched them while hidden.
    pub fn toggle(&mut self, dir: &Path) -> Vec<PathBuf> {
        if self.expanded.remove(dir) {
            self.rebuild_rows();
            return Vec::new();
        }
        self.expanded.insert(dir.to_path_buf());
        self.rebuild_rows();
        self.shown_folders().into_iter().filter(|folder| folder.starts_with(dir)).collect()
    }

    /// Expands or collapses hidden group `group`, as [`Self::toggle`] does a folder; returns the hidden folders to
    /// read now.
    pub fn toggle_hidden(&mut self, group: &Path) -> Vec<PathBuf> {
        let Some(dir) = group.parent() else {
            return Vec::new();
        };
        if self.expanded_hidden.remove(dir) {
            self.rebuild_rows();
            return Vec::new();
        }
        self.expanded_hidden.insert(dir.to_path_buf());
        self.rebuild_rows();
        self.shown_folders().into_iter().filter(|folder| self.is_hidden_below(folder, dir)).collect()
    }

    /// Expands every folder, and every hidden group, between the root and `path`; returns the shown folders not read yet, which the caller
    /// reads and applies, then calls this again until `path` has a row or nothing is left to read.
    pub fn expand_to(&mut self, path: &Path) -> Vec<PathBuf> {
        let below_root: Vec<&Path> = path.ancestors().take_while(|dir| *dir != self.root && dir.starts_with(&self.root)).collect();
        self.expanded.extend(below_root.iter().skip(1).map(|dir| dir.to_path_buf()));
        // a group appears only with two hidden entries, so its folder can be marked whether it has one or not
        let hidden = below_root.iter().filter(|path| path.file_name().is_some_and(|name| self.hidden.hides(&name.to_string_lossy())));
        self.expanded_hidden.extend(hidden.filter_map(|path| path.parent()).map(Path::to_path_buf));
        self.rebuild_rows();
        self.unread_folders()
    }

    /// Folders whose entries the rows show: the root, and each expanded folder whose parents are expanded.
    pub fn shown_folders(&self) -> Vec<PathBuf> {
        let expanded = self.rows.iter().filter(|row| row.entry.kind == EntryKind::Folder && row.expanded);
        std::iter::once(self.root.clone()).chain(expanded.map(|row| row.entry.path.clone())).collect()
    }

    /// Takes what reading folders found and rebuilds the rows once. A listing of a folder the rows do not show is
    /// dropped; a shown subfolder that could not be read is collapsed, the root keeps its last entries. Returns
    /// the folders that could not be read.
    pub fn apply(&mut self, listings: &[Listing]) -> Vec<FileTreeError> {
        let shown: HashSet<PathBuf> = self.shown_folders().into_iter().collect();
        let listings: Vec<&Listing> = listings.iter().filter(|listing| shown.contains(&listing.dir)).collect();
        if listings.is_empty() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for listing in listings {
            match &listing.entries {
                Ok(entries) => self.set_entries(&listing.dir, entries.clone()),
                Err(error) if listing.dir == self.root => errors.push(error.clone()),
                Err(error) => {
                    self.forget(std::slice::from_ref(&listing.dir));
                    errors.push(error.clone());
                }
            }
        }
        self.rebuild_rows();
        errors
    }

    /// Replaces the entries of `dir`, forgetting what was known below entries that are gone.
    fn set_entries(&mut self, dir: &Path, entries: Vec<Entry>) {
        // the folder's own row shows its .run.toml too: what this listing read of it is newer than the parent's
        let run = entries.iter().find(|entry| entry.name == RUN_FILE && entry.kind == EntryKind::File).map_or(RunState::None, |file| file.run.clone());
        if let Some(own) = dir.parent().and_then(|parent| self.children.get_mut(parent)).and_then(|siblings| siblings.iter_mut().find(|entry| entry.path == dir)) {
            own.run = run;
        }
        let kept: HashSet<PathBuf> = entries.iter().map(|entry| entry.path.clone()).collect();
        let before = self.children.insert(dir.to_path_buf(), entries).unwrap_or_default();
        let gone: Vec<PathBuf> = before.into_iter().map(|entry| entry.path).filter(|path| !kept.contains(path)).collect();
        self.forget(&gone);
    }

    /// Drops what is known about `paths` and everything below them.
    fn forget(&mut self, paths: &[PathBuf]) {
        let below = |path: &PathBuf| paths.iter().any(|gone| path.starts_with(gone));
        self.expanded.retain(|path| !below(path));
        self.expanded_hidden.retain(|path| !below(path));
        self.children.retain(|path, _| !below(path));
    }

    /// Whether `path` is in `dir`'s hidden group: a hidden entry of `dir` or below one.
    fn is_hidden_below(&self, path: &Path, dir: &Path) -> bool {
        let first = path.strip_prefix(dir).ok().and_then(|rest| rest.components().next());
        first.is_some_and(|name| self.hidden.hides(&name.as_os_str().to_string_lossy()))
    }

    fn rebuild_rows(&mut self) {
        let mut rows = Vec::new();
        self.collect_rows(&self.root, 0, &mut rows);
        self.rows = rows;
    }

    fn collect_rows(&self, dir: &Path, depth: usize, rows: &mut Vec<Row>) {
        let entries = self.children.get(dir).map(Vec::as_slice).unwrap_or_default();
        let (hidden, shown): (Vec<&Entry>, Vec<&Entry>) = entries.iter().partition(|entry| self.hidden.hides(&entry.name));
        if hidden.len() < MIN_HIDDEN_GROUP {
            return self.push_entries(entries.iter(), depth, rows);
        }
        let expanded = self.expanded_hidden.contains(dir);
        rows.push(Row { entry: hidden_group(dir), depth, expanded });
        if expanded {
            self.push_entries(hidden.into_iter(), depth + 1, rows);
        }
        self.push_entries(shown.into_iter(), depth, rows);
    }

    fn push_entries<'a>(&self, entries: impl Iterator<Item = &'a Entry>, depth: usize, rows: &mut Vec<Row>) {
        for entry in entries {
            let expanded = entry.kind == EntryKind::Folder && self.expanded.contains(&entry.path);
            rows.push(Row { entry: entry.clone(), depth, expanded });
            if expanded {
                self.collect_rows(&entry.path, depth + 1, rows);
            }
        }
    }
}

fn hidden_group(dir: &Path) -> Entry {
    Entry { name: HIDDEN_GROUP.to_owned(), path: dir.join(HIDDEN_GROUP), kind: EntryKind::HiddenGroup, run: RunState::None }
}

/// `folder`'s `.run.toml`, as the tree shows it.
fn run_state(folder: &Path) -> RunState {
    match run::file::load(folder) {
        None => RunState::None,
        Some(Ok(file)) => RunState::Loaded(Arc::new(file)),
        Some(Err(error)) => RunState::Broken(error),
    }
}

fn unreadable(path: &Path, source: &io::Error) -> FileTreeError {
    FileTreeError::Unreadable { path: path.to_path_buf(), reason: source.to_string() }
}

fn read_entries(dir: &Path) -> io::Result<Vec<Entry>> {
    let mut entries = fs::read_dir(dir)?
        .take(MAX_ENTRIES)
        .map(|entry| {
            let path = entry?.path();
            // follows symlinks, so a link to a folder lists as a folder
            let kind = if path.is_dir() { EntryKind::Folder } else { EntryKind::File };
            let name = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
            let run = match kind {
                EntryKind::Folder => run_state(&path),
                _ if name == RUN_FILE => run_state(dir),
                _ => RunState::None,
            };
            Ok(Entry { name, path, kind, run })
        })
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort_by(|a, b| (Reverse(a.expands()), &a.name).cmp(&(Reverse(b.expands()), &b.name)));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(root: &Path) -> FileTree {
        FileTree::new(root.to_path_buf(), read_folder(root).entries.unwrap())
    }

    fn tree(project: &FileTree) -> Vec<String> {
        project.rows().iter().map(|row| format!("{}{}", "  ".repeat(row.depth), row.entry.name)).collect()
    }

    /// Toggles `dir` and reads what that asks for at once.
    fn toggle(project: &mut FileTree, dir: &Path) -> Vec<FileTreeError> {
        let listings: Vec<Listing> = project.toggle(dir).iter().map(|folder| read_folder(folder)).collect();
        project.apply(&listings)
    }

    fn sample_project() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        fs::create_dir_all(root.join("src/model")).unwrap();
        fs::write(root.join("src/main.rs"), "").unwrap();
        fs::write(root.join("src/model/user.rs"), "").unwrap();
        fs::write(root.join("b.txt"), "").unwrap();
        fs::write(root.join("a.txt"), "").unwrap();
        (dir, root)
    }

    #[test]
    fn should_find_a_listed_entry_by_path() {
        // given
        let (_dir, root) = sample_project();
        let project = open(&root);

        // when
        let found = [project.entry(&root.join("src")).map(|entry| entry.name.clone()), project.entry(&root.join("src/main.rs")).map(|entry| entry.name.clone())];

        // then
        assert_eq!(found, [Some("src".to_owned()), None]);
    }

    #[test]
    fn should_refresh_a_folders_run_file_when_the_folder_is_read_again() {
        // given
        let (_dir, root) = sample_project();
        fs::write(root.join("src/.run.toml"), "[run.build]\ncommand = \"make\"\n").unwrap();
        let mut project = open(&root);
        toggle(&mut project, &root.join("src"));
        fs::write(root.join("src/.run.toml"), "[run.build]\ncommand = \"make\"\n\n[run.test]\ncommand = \"make test\"\n").unwrap();

        // when
        project.apply(&[read_folder(&root.join("src"))]);

        // then
        let RunState::Loaded(file) = &project.entry(&root.join("src")).unwrap().run else { panic!("src has a loaded .run.toml") };
        assert_eq!(file.configs.len(), 2);
    }

    #[test]
    fn should_forget_a_folders_run_file_once_it_is_gone() {
        // given
        let (_dir, root) = sample_project();
        fs::write(root.join("src/.run.toml"), "[run.build]\ncommand = \"make\"\n").unwrap();
        let mut project = open(&root);
        toggle(&mut project, &root.join("src"));
        fs::remove_file(root.join("src/.run.toml")).unwrap();

        // when
        project.apply(&[read_folder(&root.join("src"))]);

        // then
        assert_eq!(project.entry(&root.join("src")).unwrap().run, RunState::None);
    }

    #[test]
    fn should_load_run_file_of_a_child_folder() {
        // given
        let (_dir, root) = sample_project();
        fs::write(root.join("src/.run.toml"), "[run.build]\ncommand = \"make\"\n").unwrap();

        // when
        let listing = read_folder(&root);

        // then
        let src = listing.entries.unwrap().into_iter().find(|entry| entry.name == "src").unwrap();
        let RunState::Loaded(file) = src.run else { panic!("src has a loaded .run.toml: {:?}", src.run) };
        assert_eq!(file.configs[0].key, "build");
    }

    #[test]
    fn should_mark_a_broken_run_file() {
        // given
        let (_dir, root) = sample_project();
        fs::write(root.join("src/.run.toml"), "[run.Build]\ncommand = \"make\"\n").unwrap();

        // when
        let parent = read_folder(&root).entries.unwrap();
        let folder = read_folder(&root.join("src")).entries.unwrap();

        // then
        let src = parent.iter().find(|entry| entry.name == "src").unwrap();
        let file = folder.iter().find(|entry| entry.name == ".run.toml").unwrap();
        assert!(matches!(&src.run, RunState::Broken(error) if error.line == 1), "{:?}", src.run);
        assert!(matches!(&file.run, RunState::Broken(_)), "{:?}", file.run);
        assert_eq!(parent.iter().find(|entry| entry.name == "a.txt").unwrap().run, RunState::None);
    }

    /// Expands down to `path`, reading what that asks for, until nothing is left to read.
    fn expand_to(project: &mut FileTree, path: &Path) -> usize {
        let mut rounds = 0;
        loop {
            let reads = project.expand_to(path);
            if reads.is_empty() {
                return rounds;
            }
            let listings: Vec<Listing> = reads.iter().map(|folder| read_folder(folder)).collect();
            project.apply(&listings);
            rounds += 1;
        }
    }

    #[test]
    fn should_expand_every_folder_above_a_file_one_level_per_read() {
        // given
        let (_dir, root) = sample_project();
        let mut project = open(&root);

        // when
        let rounds = expand_to(&mut project, &root.join("src/model/user.rs"));

        // then
        assert_eq!(rounds, 2);
        assert_eq!(tree(&project), ["src", "  model", "    user.rs", "  main.rs", "a.txt", "b.txt"]);
    }

    #[test]
    fn should_expand_nothing_for_a_path_outside_the_root() {
        // given
        let (_dir, root) = sample_project();
        let mut project = open(&root);

        // when
        let reads = project.expand_to(Path::new("/elsewhere/src/a.rs"));

        // then
        assert!(reads.is_empty());
        assert_eq!(tree(&project), ["src", "a.txt", "b.txt"]);
    }

    #[test]
    fn should_list_directories_before_files_sorted_by_name() {
        // given
        let (_dir, root) = sample_project();

        // when
        let project = open(&root);

        // then
        assert_eq!(tree(&project), ["src", "a.txt", "b.txt"]);
    }

    #[test]
    fn should_show_children_of_expanded_folder_indented() {
        // given
        let (_dir, root) = sample_project();
        let mut project = open(&root);

        // when
        toggle(&mut project, &root.join("src"));
        toggle(&mut project, &root.join("src/model"));

        // then
        assert_eq!(tree(&project), ["src", "  model", "    user.rs", "  main.rs", "a.txt", "b.txt"]);
    }

    #[test]
    fn should_hide_children_of_collapsed_folder_and_keep_nested_expansion() {
        // given
        let (_dir, root) = sample_project();
        let mut project = open(&root);
        toggle(&mut project, &root.join("src"));
        toggle(&mut project, &root.join("src/model"));

        // when
        toggle(&mut project, &root.join("src"));
        let collapsed = tree(&project);
        toggle(&mut project, &root.join("src"));

        // then
        assert_eq!(collapsed, ["src", "a.txt", "b.txt"]);
        assert_eq!(tree(&project), ["src", "  model", "    user.rs", "  main.rs", "a.txt", "b.txt"]);
    }

    #[test]
    fn should_ask_for_the_expanded_folder_and_drop_listings_of_folders_no_longer_shown() {
        // given
        let (_dir, root) = sample_project();
        let mut project = open(&root);

        // when
        let reads = project.toggle(&root.join("src"));
        let listing = read_folder(&root.join("src"));
        project.toggle(&root.join("src"));
        project.apply(&[listing]);
        project.toggle(&root.join("src"));

        // then
        assert_eq!(reads, [root.join("src")]);
        assert_eq!(tree(&project), ["src", "a.txt", "b.txt"], "nothing listed for src yet");
    }

    #[test]
    fn should_list_only_the_folders_the_rows_show() {
        // given
        let (_dir, root) = sample_project();
        let mut project = open(&root);
        toggle(&mut project, &root.join("src"));
        toggle(&mut project, &root.join("src/model"));

        // when
        let expanded = project.shown_folders();
        toggle(&mut project, &root.join("src"));

        // then
        assert_eq!(expanded, [root.clone(), root.join("src"), root.join("src/model")]);
        assert_eq!(project.shown_folders(), [root]);
    }

    #[test]
    fn should_reread_a_folder_keeping_expansion_and_forgetting_gone_subfolders() {
        // given
        let (_dir, root) = sample_project();
        let mut project = open(&root);
        toggle(&mut project, &root.join("src"));
        toggle(&mut project, &root.join("src/model"));
        fs::remove_dir_all(root.join("src/model")).unwrap();
        fs::write(root.join("src/lib.rs"), "").unwrap();

        // when
        project.apply(&[read_folder(&root.join("src"))]);

        // then
        assert_eq!(tree(&project), ["src", "  lib.rs", "  main.rs", "a.txt", "b.txt"]);
        assert_eq!(project.shown_folders(), [root.clone(), root.join("src")]);
    }

    #[test]
    fn should_reread_folders_expanded_below_a_folder_expanded_again() {
        // given
        let (_dir, root) = sample_project();
        let mut project = open(&root);
        toggle(&mut project, &root.join("src"));
        toggle(&mut project, &root.join("src/model"));
        toggle(&mut project, &root.join("src"));
        fs::write(root.join("src/model/order.rs"), "").unwrap();

        // when
        let reads = project.toggle(&root.join("src"));

        // then
        assert_eq!(reads, [root.join("src"), root.join("src/model")]);
    }

    #[test]
    fn should_collapse_a_folder_that_cannot_be_read_and_say_why() {
        // given
        let (_dir, root) = sample_project();
        let mut project = open(&root);
        fs::remove_dir_all(root.join("src")).unwrap();

        // when
        let errors = toggle(&mut project, &root.join("src"));

        // then
        assert_eq!((project.rows()[0].expanded, errors.len()), (false, 1));
        assert!(errors[0].to_string().starts_with(&format!("cannot read {}:", root.join("src").display())));
    }

    fn dotfiles_project() -> (tempfile::TempDir, PathBuf) {
        let (dir, root) = sample_project();
        fs::create_dir_all(root.join(".git/refs")).unwrap();
        fs::write(root.join(".gitignore"), "").unwrap();
        (dir, root)
    }

    fn hidden_group(root: &Path) -> PathBuf {
        root.join(HIDDEN_GROUP)
    }

    #[test]
    fn should_collapse_two_hidden_entries_under_a_hidden_group_first() {
        // given
        let (_dir, root) = dotfiles_project();

        // when
        let project = open(&root);

        // then
        assert_eq!(tree(&project), ["@hidden", "src", "a.txt", "b.txt"]);
        assert_eq!(project.rows()[0].entry.kind, EntryKind::HiddenGroup);
    }

    #[test]
    fn should_keep_a_single_hidden_entry_in_place() {
        // given
        let (_dir, root) = sample_project();
        fs::write(root.join(".gitignore"), "").unwrap();

        // when
        let project = open(&root);

        // then
        assert_eq!(tree(&project), ["src", ".gitignore", "a.txt", "b.txt"]);
    }

    #[test]
    fn should_show_hidden_entries_indented_below_the_expanded_group() {
        // given
        let (_dir, root) = dotfiles_project();
        let mut project = open(&root);

        // when
        let reads = project.toggle_hidden(&hidden_group(&root));
        let expanded = tree(&project);
        project.toggle_hidden(&hidden_group(&root));

        // then
        assert!(reads.is_empty());
        assert_eq!(expanded, ["@hidden", "  .git", "  .gitignore", "src", "a.txt", "b.txt"]);
        assert_eq!(tree(&project), ["@hidden", "src", "a.txt", "b.txt"]);
    }

    #[test]
    fn should_reread_hidden_folders_expanded_below_a_group_expanded_again() {
        // given
        let (_dir, root) = dotfiles_project();
        let mut project = open(&root);
        project.toggle_hidden(&hidden_group(&root));
        toggle(&mut project, &root.join(".git"));
        project.toggle_hidden(&hidden_group(&root));

        // when
        let reads = project.toggle_hidden(&hidden_group(&root));

        // then
        assert_eq!(reads, [root.join(".git")]);
        assert_eq!(tree(&project), ["@hidden", "  .git", "    refs", "  .gitignore", "src", "a.txt", "b.txt"]);
    }

    #[test]
    fn should_expand_the_hidden_group_above_a_located_file() {
        // given
        let (_dir, root) = dotfiles_project();
        fs::write(root.join(".git/HEAD"), "").unwrap();
        let mut project = open(&root);

        // when
        expand_to(&mut project, &root.join(".git/HEAD"));

        // then
        assert_eq!(tree(&project), ["@hidden", "  .git", "    refs", "    HEAD", "  .gitignore", "src", "a.txt", "b.txt"]);
    }

    #[test]
    fn should_put_named_entries_under_the_hidden_group() {
        // given
        let (_dir, root) = sample_project();
        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join(".gitignore"), "").unwrap();
        let mut project = open(&root);

        // when
        let reads = project.set_hidden(&crate::script::hidden::of(&["target"]));
        project.toggle_hidden(&hidden_group(&root));

        // then
        assert!(reads.is_empty());
        assert_eq!(tree(&project), ["@hidden", "  target", "  .gitignore", "src", "a.txt", "b.txt"]);
    }

    #[test]
    fn should_read_an_expanded_folder_shown_again_once_no_longer_hidden() {
        // given
        let (_dir, root) = dotfiles_project();
        fs::create_dir_all(root.join("target/debug")).unwrap();
        let hidden = crate::script::hidden::of(&["target"]);
        let restored = Expansion { folders: vec![root.join("target")], hidden: Vec::new() };
        let mut project = FileTree::restored(root.clone(), read_folder(&root).entries.unwrap(), &restored);
        project.set_hidden(&hidden);

        // when
        let reads = project.set_hidden(&crate::script::hidden::Hidden::default());

        // then
        assert_eq!(reads, [root.join("target")]);
    }

    #[test]
    fn should_reject_missing_folder_with_its_path_in_message() {
        // given
        let missing = Path::new("/nonexistent/ive-project");

        // when
        let error = read_folder(missing).entries.err().unwrap();

        // then
        assert!(error.to_string().starts_with("cannot read /nonexistent/ive-project:"));
    }
}
