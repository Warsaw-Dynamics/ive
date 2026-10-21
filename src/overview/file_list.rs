//! How the Uncommitted and Since lists order their files: by path or newest edit first, flat or as a folder tree.

use std::cmp::Reverse;
use std::collections::{BTreeSet, HashMap};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::git::status::Change;
use crate::git::types::GitPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSort {
    #[default]
    Name,
    /// newest first; deleted files, which have no time, last
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileLayout {
    #[default]
    Flat,
    Tree,
}

impl FileLayout {
    pub fn toggled(self) -> Self {
        match self {
            Self::Flat => Self::Tree,
            Self::Tree => Self::Flat,
        }
    }
}

/// The user's choice for every git file list, saved in the settings as `[git_files]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FileOrder {
    pub sort: FileSort,
    pub layout: FileLayout,
}

impl FileOrder {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// When each listed file was last written in the work tree; a deleted file has no entry.
pub type Modified = HashMap<GitPath, SystemTime>;

/// One row of a file list: a file by its index into the listed changes, or a tree folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEntry {
    /// `depth` is `None` in the flat list, which shows the folder beside the name
    File { index: usize, depth: Option<usize> },
    Folder(FolderRow),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderRow {
    /// `src/overview`: the key a collapse is kept under
    pub path: String,
    /// `overview`, or `src/overview` when `src` holds nothing else
    pub label: String,
    pub depth: usize,
    /// changed files below it, at any depth
    pub files: usize,
    pub open: bool,
}

/// The indexes of `files` in `sort` order.
pub fn sorted(files: &[Change], modified: &Modified, sort: FileSort) -> Vec<usize> {
    let mut order: Vec<usize> = (0..files.len()).collect();
    match sort {
        FileSort::Name => order.sort_by(|a, b| files[*a].path.cmp(&files[*b].path)),
        // `None` sorts below every time, so `Reverse` puts deleted files last
        FileSort::Modified => order.sort_by_key(|index| (Reverse(modified.get(&files[*index].path)), &files[*index].path)),
    }
    order
}

/// The rows for the files `order` lists. In a tree, a folder sits where its first file in `order` would, but
/// under [`FileSort::Name`] folders come before files; the rows of a `collapsed` folder's contents are left out.
pub fn entries(files: &[Change], order: &[usize], order_by: FileOrder, collapsed: &BTreeSet<String>) -> Vec<FileEntry> {
    if order_by.layout == FileLayout::Flat {
        return order.iter().map(|index| FileEntry::File { index: *index, depth: None }).collect();
    }
    let mut root = Folder::default();
    for index in order {
        root.insert(&folders_of(&files[*index].path), *index);
    }
    let mut rows = Vec::new();
    root.emit(Emit { prefix: "", depth: 0, sort: order_by.sort, collapsed }, &mut rows);
    rows
}

/// The folders of the tree [`entries`] would list to collapse so that it shows at most `budget` rows where it can.
/// Folders in `toggled` stay as the user left them (`true`: open); the others open shallowest first, in list
/// order, each when its rows still fit.
pub fn collapsed_within(files: &[Change], order: &[usize], order_by: FileOrder, toggled: &HashMap<String, bool>, budget: usize) -> BTreeSet<String> {
    let all = entries(files, order, order_by, &BTreeSet::new());
    let depth = |entry: &FileEntry| match entry {
        FileEntry::File { depth, .. } => depth.unwrap_or(0),
        FileEntry::Folder(folder) => folder.depth,
    };
    // each row's folder, and how many rows each folder holds directly
    let (mut parent, mut children) = (vec![None; all.len()], vec![0; all.len()]);
    let mut enclosing: Vec<usize> = Vec::new();
    for (index, entry) in all.iter().enumerate() {
        while enclosing.last().is_some_and(|folder| depth(&all[*folder]) >= depth(entry)) {
            enclosing.pop();
        }
        if let Some(folder) = enclosing.last() {
            parent[index] = Some(*folder);
            children[*folder] += 1;
        }
        if matches!(entry, FileEntry::Folder(_)) {
            enclosing.push(index);
        }
    }
    let mut folders: Vec<(usize, &FolderRow)> = all
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| match entry {
            FileEntry::Folder(folder) => Some((index, folder)),
            FileEntry::File { .. } => None,
        })
        .collect();
    folders.sort_by_key(|(index, folder)| (folder.depth, *index));
    let mut shown = all.iter().filter(|entry| depth(entry) == 0).count();
    let mut open = vec![false; all.len()];
    for (index, folder) in &folders {
        if parent[*index].is_some_and(|above| !open[above]) {
            continue;
        }
        open[*index] = toggled.get(&folder.path).copied().unwrap_or(shown + children[*index] <= budget);
        if open[*index] {
            shown += children[*index];
        }
    }
    folders.into_iter().filter(|(index, _)| !open[*index]).map(|(_, folder)| folder.path.clone()).collect()
}

/// `["src", "overview"]` of `src/overview/view.rs`; an untracked folder `nested/` is a file of the top folder.
fn folders_of(path: &GitPath) -> Vec<String> {
    let folder = path.folder();
    if folder.is_empty() {
        return Vec::new();
    }
    folder.split('/').map(str::to_owned).collect()
}

#[derive(Default)]
struct Folder {
    /// in the order they were first seen
    children: Vec<Node>,
    files: usize,
}

enum Node {
    Folder(String, Folder),
    File(usize),
}

#[derive(Clone, Copy)]
struct Emit<'a> {
    prefix: &'a str,
    depth: usize,
    sort: FileSort,
    collapsed: &'a BTreeSet<String>,
}

impl Folder {
    fn insert(&mut self, folders: &[String], index: usize) {
        self.files += 1;
        let Some((first, rest)) = folders.split_first() else {
            self.children.push(Node::File(index));
            return;
        };
        let position = self.children.iter().position(|node| matches!(node, Node::Folder(name, _) if name == first));
        let position = position.unwrap_or_else(|| {
            self.children.push(Node::Folder(first.clone(), Folder::default()));
            self.children.len() - 1
        });
        if let Node::Folder(_, folder) = &mut self.children[position] {
            folder.insert(rest, index);
        }
    }

    /// The one folder this holds when it holds nothing else: shown on the same row.
    fn only_folder(&self) -> Option<(&str, &Folder)> {
        match self.children.as_slice() {
            [Node::Folder(name, folder)] => Some((name, folder)),
            _ => None,
        }
    }

    fn emit(&self, at: Emit<'_>, rows: &mut Vec<FileEntry>) {
        let folders = self.children.iter().filter(|node| matches!(node, Node::Folder(..)));
        let files = self.children.iter().filter(|node| matches!(node, Node::File(_)));
        let nodes: Vec<&Node> = match at.sort {
            FileSort::Name => folders.chain(files).collect(),
            FileSort::Modified => self.children.iter().collect(),
        };
        for node in nodes {
            match node {
                Node::File(index) => rows.push(FileEntry::File { index: *index, depth: Some(at.depth) }),
                Node::Folder(name, folder) => folder.emit_folder(name, at, rows),
            }
        }
    }

    fn emit_folder(&self, name: &str, at: Emit<'_>, rows: &mut Vec<FileEntry>) {
        let mut label = name.to_owned();
        let mut shown = self;
        while let Some((name, inner)) = shown.only_folder() {
            label = format!("{label}/{name}");
            shown = inner;
        }
        let path = if at.prefix.is_empty() { label.clone() } else { format!("{}/{label}", at.prefix) };
        let open = !at.collapsed.contains(&path);
        rows.push(FileEntry::Folder(FolderRow { path: path.clone(), label, depth: at.depth, files: self.files, open }));
        if open {
            shown.emit(Emit { prefix: &path, depth: at.depth + 1, ..at }, rows);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::git::types::Mark;

    fn changes(paths: &[&str]) -> Vec<Change> {
        paths.iter().map(|path| Change { mark: Mark::Modified, path: GitPath::from_bytes(path.as_bytes()), old_path: None }).collect()
    }

    fn at(seconds: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn tree(sort: FileSort) -> FileOrder {
        FileOrder { sort, layout: FileLayout::Tree }
    }

    /// A row as `name@depth`, folders as `label/ (files)`.
    fn shown(files: &[Change], rows: &[FileEntry]) -> Vec<String> {
        rows.iter()
            .map(|row| match row {
                FileEntry::File { index, depth } => format!("{}@{}", files[*index].path.file_name(), depth.map_or("-".to_owned(), |depth| depth.to_string())),
                FileEntry::Folder(folder) => format!("{}/ ({})@{}", folder.label, folder.files, folder.depth),
            })
            .collect()
    }

    #[test]
    fn should_sort_newest_first_with_deleted_files_last() {
        // given
        let files = changes(&["a.txt", "b.txt", "gone.txt", "c.txt"]);
        let modified = Modified::from([(files[0].path.clone(), at(10)), (files[1].path.clone(), at(30)), (files[3].path.clone(), at(20))]);

        // when
        let order = sorted(&files, &modified, FileSort::Modified);

        // then
        assert_eq!(order, [1, 3, 0, 2]);
    }

    #[test]
    fn should_list_folders_first_and_join_single_child_folders_when_sorted_by_name() {
        // given
        let files = changes(&["README.md", "src/overview/a.rs", "src/overview/b.rs", "src/main.rs", "tools/x/y/z.sh"]);
        let order = sorted(&files, &Modified::new(), FileSort::Name);

        // when
        let rows = entries(&files, &order, tree(FileSort::Name), &BTreeSet::new());

        // then
        assert_eq!(
            shown(&files, &rows),
            ["src/ (3)@0", "overview/ (2)@1", "a.rs@2", "b.rs@2", "main.rs@1", "tools/x/y/ (1)@0", "z.sh@1", "README.md@0"]
        );
    }

    #[test]
    fn should_put_the_folder_with_the_newest_file_first_when_sorted_by_modified() {
        // given
        let files = changes(&["old/a.rs", "new/b.rs", "top.txt", "old/c.rs"]);
        let modified = Modified::from([(files[0].path.clone(), at(1)), (files[1].path.clone(), at(9)), (files[2].path.clone(), at(5)), (files[3].path.clone(), at(7))]);
        let order = sorted(&files, &modified, FileSort::Modified);

        // when
        let rows = entries(&files, &order, tree(FileSort::Modified), &BTreeSet::new());

        // then
        assert_eq!(shown(&files, &rows), ["new/ (1)@0", "b.rs@1", "old/ (2)@0", "c.rs@1", "a.rs@1", "top.txt@0"]);
    }

    #[test]
    fn should_hide_the_contents_of_a_collapsed_folder() {
        // given
        let files = changes(&["src/a/x.rs", "src/b.rs", "c.rs"]);
        let order = sorted(&files, &Modified::new(), FileSort::Name);
        let collapsed = BTreeSet::from(["src/a".to_owned()]);

        // when
        let rows = entries(&files, &order, tree(FileSort::Name), &collapsed);

        // then
        assert_eq!(shown(&files, &rows), ["src/ (2)@0", "a/ (1)@1", "b.rs@1", "c.rs@0"]);
        assert!(matches!(&rows[1], FileEntry::Folder(FolderRow { open: false, .. })));
    }

    #[test]
    fn should_open_folders_shallowest_first_while_their_rows_fit() {
        // given
        let mut paths = vec!["src/a/1.rs", "src/a/2.rs", "src/b/1.rs", "docs/x.md", "top.txt"];
        let many: Vec<String> = (0..20).map(|n| format!("big/{n}.rs")).collect();
        paths.extend(many.iter().map(String::as_str));
        let files = changes(&paths);
        let order = sorted(&files, &Modified::new(), FileSort::Name);

        // when
        let collapsed = collapsed_within(&files, &order, tree(FileSort::Name), &HashMap::new(), 9);

        // then
        let rows = entries(&files, &order, tree(FileSort::Name), &collapsed);
        assert_eq!(collapsed, BTreeSet::from(["big".to_owned(), "src/b".to_owned()]));
        assert_eq!(shown(&files, &rows), ["big/ (20)@0", "docs/ (1)@0", "x.md@1", "src/ (3)@0", "a/ (2)@1", "1.rs@2", "2.rs@2", "b/ (1)@1", "top.txt@0"]);
    }

    #[test]
    fn should_keep_folders_the_user_opened_or_closed() {
        // given
        let files = changes(&["big/1.rs", "big/2.rs", "big/3.rs", "small/x.rs"]);
        let order = sorted(&files, &Modified::new(), FileSort::Name);
        let toggled = HashMap::from([("big".to_owned(), true), ("small".to_owned(), false)]);

        // when
        let collapsed = collapsed_within(&files, &order, tree(FileSort::Name), &toggled, 2);

        // then
        assert_eq!(collapsed, BTreeSet::from(["small".to_owned()]));
    }

    #[test]
    fn should_keep_the_flat_list_in_sort_order() {
        // given
        let files = changes(&["src/a.rs", "b.rs"]);
        let order = sorted(&files, &Modified::new(), FileSort::Name);

        // when
        let rows = entries(&files, &order, FileOrder::default(), &BTreeSet::new());

        // then
        assert_eq!(shown(&files, &rows), ["b.rs@-", "a.rs@-"]);
    }
}
