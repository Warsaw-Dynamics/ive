//! What the disk holds for a copy or move: the files the sources carry, what is at their destinations, and how a
//! file in the way differs from the incoming one. Blocking I/O: run it off the UI thread.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::Verb;
use super::plan::{Action, Conflict, Existing, Incoming, IncomingKind, Plan, PlanRow, Source};

/// At most this many files in one copy, or one move across file systems.
pub const MAX_FILES: usize = 20_000;
/// At most this many bytes in one copy, or one move across file systems.
pub const MAX_BYTES: u64 = 4 << 30;
/// A file larger than this, or with a NUL in its first `SNIFF_BYTES`, is compared as binary.
const MAX_TEXT_BYTES: u64 = 1 << 20;
const SNIFF_BYTES: usize = 8 << 10;
/// The lines a Conflicts tab row shows of each side.
const PREVIEW_LINES: usize = 200;
/// git's own, never copied with a folder
const GIT: &str = ".git";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    pub files: usize,
    pub bytes: u64,
}

/// What the sources carry, and how many nested checkouts were left out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walked {
    pub incoming: Vec<Incoming>,
    pub left_out: usize,
}

/// Every file, symlink and empty folder `sources` carry, by name in each folder, without `.git` and the folders in
/// `other_checkouts`; refused over [`MAX_FILES`] or [`MAX_BYTES`].
pub fn walk(sources: &[Source], verb: Verb, other_checkouts: &[PathBuf]) -> Result<Walked, String> {
    walk_bounded(sources, verb, other_checkouts, Bounds { files: MAX_FILES, bytes: MAX_BYTES })
}

/// Stops as soon as a bound is passed, without walking the rest. A move carries git's own with its folder but refuses
/// a folder holding another open checkout; a copy leaves both out.
pub fn walk_bounded(sources: &[Source], verb: Verb, other_checkouts: &[PathBuf], bounds: Bounds) -> Result<Walked, String> {
    let mut walked = Walked { incoming: Vec::new(), left_out: 0 };
    let mut bytes = 0;
    for (top, source) in sources.iter().enumerate() {
        let mut walker = Walker { top, root: &source.path, other_checkouts, walked: &mut walked, rules: Rules { verb, bounds }, bytes: &mut bytes };
        walker.visit(&source.path)?;
    }
    Ok(walked)
}

#[derive(Clone, Copy)]
struct Rules {
    verb: Verb,
    bounds: Bounds,
}

struct Walker<'a> {
    top: usize,
    root: &'a Path,
    other_checkouts: &'a [PathBuf],
    walked: &'a mut Walked,
    rules: Rules,
    bytes: &'a mut u64,
}

impl Walker<'_> {
    fn visit(&mut self, path: &Path) -> Result<(), String> {
        let metadata = fs::symlink_metadata(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if !metadata.is_dir() {
            let kind = if metadata.file_type().is_symlink() { IncomingKind::Symlink } else { IncomingKind::File };
            return self.push(path, kind, metadata.len());
        }
        let copying = self.rules.verb == Verb::Copy;
        let mut children: Vec<PathBuf> = fs::read_dir(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|child| !copying || child.file_name().is_some_and(|name| name != GIT))
            .collect();
        children.sort();
        let before = (self.walked.incoming.len(), self.walked.left_out);
        for child in children {
            if self.other_checkouts.contains(&child) && !copying {
                return Err(format!("Move refused: {} is an open worktree; move it with git worktree move", child.display()));
            }
            if self.other_checkouts.contains(&child) {
                self.walked.left_out += 1;
                continue;
            }
            self.visit(&child)?;
        }
        // a folder holding only what was left out is not an empty folder to make
        if (self.walked.incoming.len(), self.walked.left_out) == before {
            return self.push(path, IncomingKind::EmptyFolder, 0);
        }
        Ok(())
    }

    fn push(&mut self, path: &Path, kind: IncomingKind, size: u64) -> Result<(), String> {
        let Rules { verb, bounds } = self.rules;
        let verb = if verb == Verb::Move { "Move" } else { "Copy" };
        *self.bytes += size;
        if self.walked.incoming.len() >= bounds.files {
            return Err(format!("{verb} refused: more than {} files", grouped(bounds.files as u64)));
        }
        if *self.bytes > bounds.bytes {
            return Err(format!("{verb} refused: more than {}", size_text(bounds.bytes)));
        }
        let relative = path.strip_prefix(self.root).map(Path::to_path_buf).unwrap_or_default();
        self.walked.incoming.push(Incoming { top: self.top, path: path.to_path_buf(), relative, kind, size });
        Ok(())
    }
}

/// What is at `path`, a symlink taken as a file; `None` when nothing is.
pub fn existing(path: &Path) -> Option<Existing> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if metadata.is_dir() {
        return Some(Existing::Folder);
    }
    Some(Existing::File { size: metadata.len(), modified: metadata.modified().unwrap_or(std::time::UNIX_EPOCH) })
}

/// Tells each file in the way apart: the same bytes (Skip from the start), text differing by so many lines, or
/// binary.
pub fn classify(plan: &mut Plan) {
    for row in plan.rows.iter_mut().filter(|row| row.conflict == Some(Conflict::Exists)) {
        let conflict = compared(row);
        if conflict == Conflict::Same {
            row.action = Some(Action::Skip);
        }
        row.conflict = Some(conflict);
    }
}

fn compared(row: &PlanRow) -> Conflict {
    let (Ok(incoming), Ok(there)) = (read_bounded(&row.incoming.path), read_bounded(&row.destination)) else {
        return Conflict::DiffersBinary;
    };
    match (incoming, there) {
        (Some(incoming), Some(there)) if incoming == there => Conflict::Same,
        (Some(incoming), Some(there)) if !is_binary(&incoming) && !is_binary(&there) => {
            let (incoming, there) = (String::from_utf8_lossy(&incoming), String::from_utf8_lossy(&there));
            Conflict::Differs { added: lines_only_in(&incoming, &there), removed: lines_only_in(&there, &incoming) }
        }
        _ => Conflict::DiffersBinary,
    }
}

/// The file's bytes; `None` when it is larger than [`MAX_TEXT_BYTES`].
fn read_bounded(path: &Path) -> std::io::Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(MAX_TEXT_BYTES + 1).read_to_end(&mut bytes)?;
    Ok((bytes.len() as u64 <= MAX_TEXT_BYTES).then_some(bytes))
}

/// The lines of `a` that `b` does not have, counted as multisets.
pub fn lines_only_in(a: &str, b: &str) -> usize {
    let mut left: HashMap<&str, usize> = HashMap::new();
    for line in b.lines() {
        *left.entry(line).or_default() += 1;
    }
    a.lines()
        .filter(|line| match left.get_mut(line) {
            Some(count) if *count > 0 => {
                *count -= 1;
                false
            }
            _ => true,
        })
        .count()
}

pub fn is_binary(bytes: &[u8]) -> bool {
    bytes.len() as u64 > MAX_TEXT_BYTES || bytes.iter().take(SNIFF_BYTES).any(|byte| *byte == 0)
}

/// The first [`PREVIEW_LINES`] lines of a text file, or `binary, 14 KB`.
pub fn preview(path: &Path) -> String {
    match read_bounded(path) {
        Ok(Some(bytes)) if !is_binary(&bytes) => String::from_utf8_lossy(&bytes).lines().take(PREVIEW_LINES).collect::<Vec<_>>().join("\n"),
        Ok(_) => format!("binary, {}", size_text(fs::metadata(path).map(|metadata| metadata.len()).unwrap_or_default())),
        Err(error) => format!("cannot read {}: {error}", path.display()),
    }
}

/// A file as the Conflicts tab lists it: `12 lines · 189 B` for text, the size alone for binary, `folder`.
pub fn describe(path: &Path) -> String {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return "gone".to_owned();
    };
    if metadata.is_dir() {
        return "folder".to_owned();
    }
    match read_bounded(path) {
        Ok(Some(bytes)) if !is_binary(&bytes) => {
            let lines = String::from_utf8_lossy(&bytes).lines().count();
            format!("{} · {}", super::plan::counted(lines, "line", "lines"), size_text(metadata.len()))
        }
        _ => size_text(metadata.len()),
    }
}

/// `189 B`, `14 KB`, `5.1 MB`, `4.0 GB`.
pub fn size_text(bytes: u64) -> String {
    const KB: u64 = 1024;
    match bytes {
        bytes if bytes < KB => format!("{bytes} B"),
        bytes if bytes < KB * KB => format!("{} KB", bytes / KB),
        bytes if bytes < KB * KB * KB => format!("{:.1} MB", bytes as f64 / (KB * KB) as f64),
        bytes => format!("{:.1} GB", bytes as f64 / (KB * KB * KB) as f64),
    }
}

/// `31 204`: thousands set apart.
pub fn grouped(number: u64) -> String {
    let digits = number.to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::project::{CheckoutRef, ProjectId};
    use crate::sidebar::drop_place::DropPlace;
    use crate::transfer::plan::{Layout, Operation, SourceKind, plan};

    fn source(root: &Path, relative: &str, kind: SourceKind) -> Source {
        Source { path: root.join(relative), checkout: CheckoutRef { project: ProjectId::new("p"), path: root.to_path_buf() }, kind }
    }

    fn relatives(walked: &Walked) -> Vec<(PathBuf, IncomingKind)> {
        walked.incoming.iter().map(|incoming| (incoming.relative.clone(), incoming.kind)).collect()
    }

    #[test]
    fn should_walk_folder_with_hidden_files() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/inner")).unwrap();
        fs::write(dir.path().join("src/.env"), "A=1").unwrap();
        fs::write(dir.path().join("src/inner/a.rs"), "fn a() {}").unwrap();

        // when
        let walked = walk(&[source(dir.path(), "src", SourceKind::Folder)], Verb::Copy, &[]).unwrap();

        // then
        assert_eq!(relatives(&walked), [(PathBuf::from(".env"), IncomingKind::File), (PathBuf::from("inner/a.rs"), IncomingKind::File)]);
    }

    #[cfg(unix)]
    #[test]
    fn should_keep_symlink_as_symlink() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        std::os::unix::fs::symlink("/nowhere", dir.path().join("src/link")).unwrap();

        // when
        let walked = walk(&[source(dir.path(), "src", SourceKind::Folder)], Verb::Copy, &[]).unwrap();

        // then
        assert_eq!(relatives(&walked), [(PathBuf::from("link"), IncomingKind::Symlink)]);
    }

    #[test]
    fn should_list_empty_folder() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/empty")).unwrap();

        // when
        let walked = walk(&[source(dir.path(), "src", SourceKind::Folder)], Verb::Copy, &[]).unwrap();

        // then
        assert_eq!(relatives(&walked), [(PathBuf::from("empty"), IncomingKind::EmptyFolder)]);
    }

    #[test]
    fn should_leave_out_git_and_nested_checkouts() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("top/.git")).unwrap();
        fs::write(dir.path().join("top/.git/HEAD"), "ref").unwrap();
        fs::create_dir_all(dir.path().join("top/worktrees/x")).unwrap();
        fs::write(dir.path().join("top/worktrees/x/a.rs"), "a").unwrap();
        fs::write(dir.path().join("top/b.rs"), "b").unwrap();

        // when
        let walked = walk(&[source(dir.path(), "top", SourceKind::Folder)], Verb::Copy, &[dir.path().join("top/worktrees/x")]).unwrap();

        // then
        assert_eq!(relatives(&walked), [(PathBuf::from("b.rs"), IncomingKind::File)]);
        assert_eq!(walked.left_out, 1);
    }

    #[test]
    fn should_refuse_too_many_files() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        for name in ["a", "b", "c", "d"] {
            fs::write(dir.path().join("src").join(name), name).unwrap();
        }

        // when
        let error = walk_bounded(&[source(dir.path(), "src", SourceKind::Folder)], Verb::Copy, &[], Bounds { files: 3, bytes: u64::MAX }).unwrap_err();

        // then
        assert_eq!(error, "Copy refused: more than 3 files");
    }

    #[test]
    fn should_refuse_moving_folder_holding_another_checkout() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("top/worktrees/x")).unwrap();
        fs::write(dir.path().join("top/worktrees/x/a.rs"), "a").unwrap();

        // when
        let error = walk(&[source(dir.path(), "top", SourceKind::Folder)], Verb::Move, &[dir.path().join("top/worktrees/x")]).unwrap_err();

        // then
        assert!(error.starts_with("Move refused: ") && error.contains("worktrees/x"), "{error}");
    }

    #[test]
    fn should_carry_git_on_move() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("top/.git")).unwrap();
        fs::write(dir.path().join("top/.git/HEAD"), "ref").unwrap();

        // when
        let walked = walk(&[source(dir.path(), "top", SourceKind::Folder)], Verb::Move, &[]).unwrap();

        // then
        assert_eq!(relatives(&walked), [(PathBuf::from(".git/HEAD"), IncomingKind::File)]);
    }

    #[test]
    fn should_group_thousands_in_refusal() {
        // when
        let text = grouped(31_204);

        // then
        assert_eq!(text, "31 204");
    }

    fn classified(old: &str, new: &str) -> PlanRow {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::create_dir_all(root.join("from")).unwrap();
        fs::create_dir_all(root.join("to")).unwrap();
        fs::write(root.join("from/a.txt"), new).unwrap();
        fs::write(root.join("to/a.txt"), old).unwrap();
        let sources = vec![source(&root, "from/a.txt", SourceKind::File)];
        let walked = walk(&sources, Verb::Copy, &[]).unwrap();
        let target = DropPlace { checkout: sources[0].checkout.clone(), folder: root.join("to") };
        let mut planned = plan(Operation { verb: Verb::Copy, sources, target, layout: Layout::Flat }, walked.incoming, existing);
        classify(&mut planned);
        planned.rows.remove(0)
    }

    #[test]
    fn should_classify_same_content_as_skip() {
        // when
        let row = classified("same\n", "same\n");

        // then
        assert_eq!((row.conflict, row.action), (Some(Conflict::Same), Some(Action::Skip)));
    }

    #[test]
    fn should_count_differing_lines() {
        // when
        let row = classified("a\nb\n", "a\nb\nc\n");

        // then
        assert_eq!((row.conflict, row.action), (Some(Conflict::Differs { added: 1, removed: 0 }), None));
    }

    #[test]
    fn should_classify_binary() {
        // when
        let row = classified("a\0b", "a\0c");

        // then
        assert_eq!(row.conflict, Some(Conflict::DiffersBinary));
    }

    #[test]
    fn should_preview_first_lines_and_binary_size() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let text: String = (1..=300).map(|line| format!("{line}\n")).collect();
        fs::write(dir.path().join("a.txt"), text).unwrap();
        fs::write(dir.path().join("b.bin"), vec![0u8; 14 * 1024]).unwrap();

        // when
        let (shown, binary) = (preview(&dir.path().join("a.txt")), preview(&dir.path().join("b.bin")));

        // then
        assert_eq!(shown.lines().count(), 200);
        assert_eq!(binary, "binary, 14 KB");
    }

    #[test]
    fn should_describe_text_file_by_lines_and_size() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.css"), "a\nb\n").unwrap();

        // when
        let described = [describe(&dir.path().join("a.css")), describe(dir.path())];

        // then
        assert_eq!(described, ["2 lines · 4 B", "folder"]);
    }

    #[test]
    fn should_size_like_the_mock() {
        // when
        let sizes = [size_text(189), size_text(14 * 1024), size_text(5 * 1024 * 1024 + 100 * 1024)];

        // then
        assert_eq!(sizes, ["189 B", "14 KB", "5.1 MB"]);
    }
}
