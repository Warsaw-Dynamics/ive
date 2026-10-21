use std::collections::HashSet;
use std::fmt;
use std::fs::{self, DirBuilder, File, Metadata, OpenOptions};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Larger scratches open read-only in the document view.
pub const MAX_SCRATCH_BYTES: u64 = 1024 * 1024;
const MAX_NAME_CHARS: usize = 100;
/// The first non-empty line shown in the listing is looked for in this much of the file.
const FIRST_LINE_PROBE_BYTES: u64 = 4096;
const FIRST_LINE_CHARS: usize = 80;

/// The scratches folder of one project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScratchStore {
    dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScratchInfo {
    pub name: String,
    pub modified: SystemTime,
    pub first_line: String,
}

/// What a file held when IVE last read or wrote it: its size and a hash of its bytes. Unlike a modification
/// time, another program's write of the same size within one clock tick changes it, and a `touch` does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileStamp {
    size: u64,
    digest: u64,
}

impl FileStamp {
    pub fn of_bytes(bytes: &[u8]) -> Self {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        Self { size: bytes.len() as u64, digest: hasher.finish() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    /// CRLF when most line breaks are CRLF.
    pub fn detect(text: &str) -> Self {
        let breaks = text.matches('\n').count();
        let crlf = text.matches("\r\n").count();
        if crlf * 2 > breaks { Self::Crlf } else { Self::Lf }
    }

    pub fn newline_bytes(self) -> usize {
        match self {
            Self::Lf => 1,
            Self::Crlf => 2,
        }
    }

    /// Bytes of `text` (LF only) as written with this ending.
    pub fn written_len(self, text: &str) -> usize {
        text.len() + text.matches('\n').count() * (self.newline_bytes() - 1)
    }

    fn apply(self, text: &str) -> String {
        match self {
            Self::Lf => text.to_owned(),
            Self::Crlf => text.replace('\n', "\r\n"),
        }
    }
}

pub enum ScratchContent {
    /// `text` has LF line breaks; `ending` is what to write back
    Editable { text: String, ending: LineEnding, stamp: FileStamp },
    TooLarge,
    NotUtf8,
}

impl ScratchStore {
    /// The folder `<ive_config>/scratches/<project_id>`; the id must be a plain folder name.
    pub fn new(ive_config: &Path, project_id: &str) -> Result<Self, ScratchError> {
        let special = project_id.is_empty() || project_id == "." || project_id == ".." || project_id.contains(['/', '\\']);
        if special {
            return Err(ScratchError::BadProjectId(project_id.to_owned()));
        }
        Ok(Self { dir: ive_config.join("scratches").join(project_id) })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// The scratches, newest first; a folder not created yet holds none.
    pub fn list(&self) -> Result<Vec<ScratchInfo>, ScratchError> {
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => return Err(ScratchError::io("list", &self.dir, source)),
        };
        let mut scratches: Vec<ScratchInfo> = entries.filter_map(Result::ok).filter_map(|entry| info(&entry)).collect();
        scratches.sort_by(|a, b| b.modified.cmp(&a.modified).then_with(|| a.name.cmp(&b.name)));
        Ok(scratches)
    }

    /// Creates the folder when missing, open to the user only, so it can be watched before the first scratch
    /// exists.
    pub fn ensure_dir(&self) -> Result<(), ScratchError> {
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        builder.mode(0o700);
        builder.create(&self.dir).map_err(|source| ScratchError::io("create", &self.dir, source))
    }

    /// Creates an empty scratch.
    pub fn create(&self, name: &str) -> Result<(), ScratchError> {
        validate_name(name)?;
        self.ensure_dir()?;
        let path = self.path(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Err(ScratchError::Exists(name.to_owned())),
            Err(source) => Err(ScratchError::io("create", &path, source)),
        }
    }

    /// Never replaces another file: a taken `to` is `ScratchError::Exists`, unless it is the same file (a
    /// case-only rename on a case-insensitive file system).
    pub fn rename(&self, from: &str, to: &str) -> Result<(), ScratchError> {
        validate_name(to)?;
        let (source, target) = (self.path(from), self.path(to));
        let renamed = match rename_new(&source, &target) {
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists && is_same_file(&source, &target) => fs::rename(&source, &target),
            other => other,
        };
        match renamed {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Err(ScratchError::Exists(to.to_owned())),
            Err(error) => Err(ScratchError::Rename { from: source, to: to.to_owned(), source: error }),
        }
    }

    pub fn delete(&self, name: &str) -> Result<(), ScratchError> {
        let path = self.path(name);
        fs::remove_file(&path).map_err(|source| ScratchError::io("delete", &path, source))
    }

    /// Too large also when writing it back would pass the limit: the first save gives every line break the
    /// ending most of them had.
    pub fn read(&self, name: &str) -> Result<ScratchContent, ScratchError> {
        let path = self.path(name);
        let bytes = read_capped(&path).map_err(|source| ScratchError::io("read", &path, source))?;
        if bytes.len() as u64 > MAX_SCRATCH_BYTES {
            return Ok(ScratchContent::TooLarge);
        }
        let stamp = FileStamp::of_bytes(&bytes);
        let Ok(text) = String::from_utf8(bytes) else {
            return Ok(ScratchContent::NotUtf8);
        };
        let (ending, text) = (LineEnding::detect(&text), text.replace("\r\n", "\n"));
        if ending.written_len(&text) as u64 > MAX_SCRATCH_BYTES {
            return Ok(ScratchContent::TooLarge);
        }
        Ok(ScratchContent::Editable { text, ending, stamp })
    }

    /// Replaces the scratch atomically and durably: a hidden temporary file in the same folder with the scratch's
    /// permissions, synced, then a rename. Creates the folder again when it was removed.
    pub fn write(&self, name: &str, text: &str, ending: LineEnding) -> Result<FileStamp, ScratchError> {
        let path = self.path(name);
        let bytes = ending.apply(text).into_bytes();
        self.ensure_dir()?;
        // short and dot-prefixed: fits any name length, and the listing ignores it
        let staging = self.dir.join(format!(".ive-save-{}.tmp", std::process::id()));
        if let Err(source) = replace(&staging, &path, &bytes) {
            let _ = fs::remove_file(&staging);
            return Err(ScratchError::io("save", &path, source));
        }
        // from the bytes, not the file: another program's write right after the rename must not count as ours
        Ok(FileStamp::of_bytes(&bytes))
    }

    /// What the scratch holds now; a file over the limit is told apart by its first 1 MiB.
    pub fn stamp(&self, name: &str) -> Result<FileStamp, ScratchError> {
        let path = self.path(name);
        read_capped(&path).map(|bytes| FileStamp::of_bytes(&bytes)).map_err(|source| ScratchError::io("check", &path, source))
    }
}

/// At most one byte past the limit, enough to tell that a file is over it.
fn read_capped(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?.take(MAX_SCRATCH_BYTES + 1).read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Writes `bytes` to `staging` with `path`'s permissions, syncs it, renames it over `path` and syncs the folder.
fn replace(staging: &Path, path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(staging)?;
    if let Ok(metadata) = fs::metadata(path) {
        file.set_permissions(metadata.permissions())?;
    }
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(staging, path)?;
    // the new content is in place; a folder that cannot be synced only leaves the rename less durable
    if let Some(dir) = path.parent() {
        let _ = File::open(dir).and_then(|dir| dir.sync_all());
    }
    Ok(())
}

/// rename(2) that fails with `AlreadyExists` instead of replacing `target`, in one step where the OS offers it.
#[cfg(target_os = "linux")]
fn rename_new(source: &Path, target: &Path) -> io::Result<()> {
    let (from, to) = (c_path(source)?, c_path(target)?);
    // SAFETY: both pointers are NUL-terminated paths that outlive the call
    let result = unsafe { libc::renameat2(libc::AT_FDCWD, from.as_ptr(), libc::AT_FDCWD, to.as_ptr(), libc::RENAME_NOREPLACE) };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    match error.raw_os_error() {
        // a file system without RENAME_NOREPLACE
        Some(libc::EINVAL | libc::ENOSYS) => checked_rename(source, target),
        _ => Err(error),
    }
}

#[cfg(target_os = "macos")]
fn rename_new(source: &Path, target: &Path) -> io::Result<()> {
    let (from, to) = (c_path(source)?, c_path(target)?);
    // SAFETY: both pointers are NUL-terminated paths that outlive the call
    let result = unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_EXCL) };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    match error.raw_os_error() {
        // a file system without RENAME_EXCL
        Some(libc::ENOTSUP) => checked_rename(source, target),
        _ => Err(error),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn rename_new(source: &Path, target: &Path) -> io::Result<()> {
    checked_rename(source, target)
}

/// Checks, then renames: another program can still create `target` in between.
fn checked_rename(source: &Path, target: &Path) -> io::Result<()> {
    if fs::symlink_metadata(target).is_ok() {
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    fs::rename(source, target)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn c_path(path: &Path) -> io::Result<std::ffi::CString> {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("{} holds a NUL byte", path.display())))
}

#[cfg(unix)]
fn is_same_file(a: &Path, b: &Path) -> bool {
    match (fs::symlink_metadata(a), fs::symlink_metadata(b)) {
        (Ok(a), Ok(b)) => (a.dev(), a.ino()) == (b.dev(), b.ino()),
        _ => false,
    }
}

#[cfg(not(unix))]
fn is_same_file(_a: &Path, _b: &Path) -> bool {
    false
}

/// Entries that are not regular files, start with `.`, or have names IVE cannot show are not scratches.
fn info(entry: &fs::DirEntry) -> Option<ScratchInfo> {
    let name = entry.file_name().into_string().ok().filter(|name| name.chars().next().is_some_and(|first| first != '.'))?;
    // on Unix this does not follow symlinks, so links are skipped too
    let metadata = entry.metadata().ok().filter(Metadata::is_file)?;
    Some(ScratchInfo { first_line: first_line(&entry.path()), modified: metadata.modified().ok()?, name })
}

fn first_line(path: &Path) -> String {
    let mut probe = Vec::new();
    if File::open(path).and_then(|file| file.take(FIRST_LINE_PROBE_BYTES).read_to_end(&mut probe)).is_err() {
        return String::new();
    }
    let text = String::from_utf8_lossy(&probe);
    let line = text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or_default();
    cut(line, FIRST_LINE_CHARS)
}

fn cut(text: &str, chars: usize) -> String {
    match text.char_indices().nth(chars) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text.to_owned(),
    }
}

pub fn validate_name(name: &str) -> Result<(), ScratchError> {
    let reason = if name.is_empty() {
        Some("empty")
    } else if name.chars().count() > MAX_NAME_CHARS {
        Some("over 100 characters")
    } else if name.starts_with('.') {
        Some("starts with .")
    } else if name.contains(['/', '\\']) {
        Some("contains / or \\")
    } else if name.chars().any(char::is_control) {
        Some("contains a control character")
    } else {
        None
    };
    match reason {
        Some(reason) => Err(ScratchError::InvalidName { name: name.to_owned(), reason }),
        None => Ok(()),
    }
}

/// `scratch.md`, or `scratch-2.md`, `scratch-3.md`, … when taken.
pub fn free_name(taken: &[&str]) -> String {
    let taken: HashSet<&str> = taken.iter().copied().collect();
    std::iter::once("scratch.md".to_owned())
        .chain((2..).map(|n| format!("scratch-{n}.md")))
        .find(|name| !taken.contains(name.as_str()))
        .expect("an endless sequence of names has a free one")
}

/// Byte length of the name without its extension: the part a name input selects.
pub fn stem_len(name: &str) -> usize {
    Path::new(name).file_stem().map_or(name.len(), |stem| stem.len())
}

#[derive(Debug)]
pub enum ScratchError {
    InvalidName { name: String, reason: &'static str },
    BadProjectId(String),
    Exists(String),
    Io { action: &'static str, path: PathBuf, source: io::Error },
    Rename { from: PathBuf, to: String, source: io::Error },
}

impl ScratchError {
    fn io(action: &'static str, path: &Path, source: io::Error) -> Self {
        Self::Io { action, path: path.to_path_buf(), source }
    }
}

impl fmt::Display for ScratchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { name, reason } => write!(f, "invalid scratch name {name:?}: {reason}"),
            Self::BadProjectId(id) => write!(f, "cannot keep scratches for project id {id:?}: not a plain folder name"),
            Self::Exists(name) => write!(f, "a scratch named {name} already exists"),
            Self::Io { action, path, source } => write!(f, "cannot {action} {}: {source}", path.display()),
            Self::Rename { from, to, source } => write!(f, "cannot rename {} to {to}: {source}", from.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn store() -> (tempfile::TempDir, ScratchStore) {
        let config = tempfile::tempdir().unwrap();
        let store = ScratchStore::new(config.path(), "atlassian").unwrap();
        (config, store)
    }

    fn write_aged(store: &ScratchStore, name: &str, text: &str, age_secs: u64) {
        fs::create_dir_all(store.dir()).unwrap();
        fs::write(store.path(name), text).unwrap();
        let file = File::options().write(true).open(store.path(name)).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(age_secs)).unwrap();
    }

    fn names(store: &ScratchStore) -> Vec<String> {
        store.list().unwrap().into_iter().map(|scratch| scratch.name).collect()
    }

    #[test]
    fn should_keep_scratches_in_a_folder_per_project_id() {
        // given
        let config = tempfile::tempdir().unwrap();

        // when
        let store = ScratchStore::new(config.path(), "atlassian").unwrap();

        // then
        assert_eq!(store.path("idp-users.sql"), config.path().join("scratches/atlassian/idp-users.sql"));
    }

    #[test]
    fn should_reject_project_ids_that_are_not_a_plain_folder_name() {
        // given
        let config = tempfile::tempdir().unwrap();
        let ids = ["", ".", "..", "a/b", "a\\b"];

        // when
        let refused = ids.map(|id| ScratchStore::new(config.path(), id).is_err());

        // then
        assert_eq!(refused, [true; 5]);
    }

    #[test]
    fn should_create_the_empty_folder_on_request() {
        // given
        let (_config, store) = store();

        // when
        store.ensure_dir().unwrap();

        // then
        assert!(store.dir().is_dir() && store.list().unwrap().is_empty());
        #[cfg(unix)]
        assert_eq!(fs::metadata(store.dir()).unwrap().mode() & 0o777, 0o700, "the folder is the user's only");
    }

    #[test]
    fn should_rename_to_another_case_of_the_same_name() {
        // given
        let (_config, store) = store();
        store.create("notes.md").unwrap();

        // when
        store.rename("notes.md", "Notes.md").unwrap();

        // then
        assert_eq!(names(&store), ["Notes.md"]);
    }

    #[test]
    fn should_create_rename_and_delete_scratches() {
        // given
        let (_config, store) = store();
        store.create("a.sql").unwrap();

        // when
        store.rename("a.sql", "b.sql").unwrap();
        let after_rename = names(&store);
        store.delete("b.sql").unwrap();

        // then
        assert_eq!((after_rename, names(&store)), (vec!["b.sql".to_owned()], Vec::<String>::new()));
    }

    #[test]
    fn should_refuse_a_name_that_exists_on_create_and_rename() {
        // given
        let (_config, store) = store();
        store.create("idp-users.sql").unwrap();
        store.create("other.md").unwrap();

        // when
        let created = store.create("idp-users.sql").unwrap_err().to_string();
        let renamed = store.rename("other.md", "idp-users.sql").unwrap_err().to_string();

        // then
        assert_eq!(created, "a scratch named idp-users.sql already exists");
        assert_eq!(renamed, created);
    }

    #[test]
    fn should_reject_invalid_names_naming_the_value() {
        // given
        let long = "a".repeat(101);
        let names = ["", ".", "..", ".hidden", "a/b", "a\\b", "a\0b", long.as_str(), "\u{15}touch pwned #.sh"];

        // when
        let errors: Vec<String> = names.iter().map(|name| validate_name(name).unwrap_err().to_string()).collect();

        // then
        assert_eq!(errors[0], r#"invalid scratch name "": empty"#);
        assert_eq!(errors[3], r#"invalid scratch name ".hidden": starts with ."#);
        assert_eq!(errors[4], r#"invalid scratch name "a/b": contains / or \"#);
        assert_eq!(errors[6], r#"invalid scratch name "a\0b": contains a control character"#);
        assert!(errors[7].ends_with(": over 100 characters"), "{}", errors[7]);
        assert_eq!(errors[8], r#"invalid scratch name "\u{15}touch pwned #.sh": contains a control character"#);
    }

    #[test]
    fn should_accept_100_characters_of_any_width() {
        // given
        let name = "é".repeat(96) + ".sql";

        // when
        let result = validate_name(&name);

        // then
        assert!(result.is_ok());
    }

    #[test]
    fn should_list_only_regular_files_not_starting_with_a_dot() {
        // given
        let (_config, store) = store();
        store.create("notes.md").unwrap();
        fs::write(store.dir().join(".notes.md.swp"), "x").unwrap();
        fs::create_dir(store.dir().join("folder")).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(store.path("notes.md"), store.dir().join("link.md")).unwrap();

        // when
        let listed = names(&store);

        // then
        assert_eq!(listed, ["notes.md"]);
    }

    #[test]
    fn should_list_newest_first_with_first_non_empty_line_cut_at_80_characters() {
        // given
        let (_config, store) = store();
        write_aged(&store, "old.sql", "\n  \nSELECT 1;\n", 3_600);
        write_aged(&store, "new.md", &"x".repeat(100), 60);

        // when
        let listed = store.list().unwrap();

        // then
        let summary: Vec<(&str, &str)> = listed.iter().map(|s| (s.name.as_str(), s.first_line.as_str())).collect();
        let cut = format!("{}…", "x".repeat(80));
        assert_eq!(summary, [("new.md", cut.as_str()), ("old.sql", "SELECT 1;")]);
    }

    #[test]
    fn should_list_nothing_when_the_folder_does_not_exist_yet() {
        // given
        let (_config, store) = store();

        // when
        let listed = store.list().unwrap();

        // then
        assert!(listed.is_empty());
    }

    #[test]
    fn should_write_atomically_leaving_no_temporary_file() {
        // given
        let (_config, store) = store();
        store.create("a.md").unwrap();

        // when
        store.write("a.md", "one\ntwo", LineEnding::Lf).unwrap();

        // then
        let entries: Vec<String> =
            fs::read_dir(store.dir()).unwrap().map(|entry| entry.unwrap().file_name().into_string().unwrap()).collect();
        assert_eq!(entries, ["a.md"]);
        assert_eq!(fs::read_to_string(store.path("a.md")).unwrap(), "one\ntwo");
    }

    #[cfg(unix)]
    #[test]
    fn should_keep_the_permissions_of_the_scratch_when_saving() {
        // given
        let (_config, store) = store();
        store.create("secret.md").unwrap();
        fs::set_permissions(store.path("secret.md"), std::os::unix::fs::PermissionsExt::from_mode(0o600)).unwrap();

        // when
        store.write("secret.md", "token", LineEnding::Lf).unwrap();

        // then
        assert_eq!(fs::metadata(store.path("secret.md")).unwrap().mode() & 0o777, 0o600);
    }

    #[test]
    fn should_create_the_folder_again_when_saving_after_it_was_removed() {
        // given
        let (_config, store) = store();
        store.create("a.md").unwrap();
        fs::remove_dir_all(store.dir()).unwrap();

        // when
        let saved = store.write("a.md", "kept", LineEnding::Lf);

        // then
        assert!(saved.is_ok(), "{saved:?}");
        assert_eq!(fs::read_to_string(store.path("a.md")).unwrap(), "kept");
    }

    #[test]
    fn should_stamp_a_save_with_what_it_wrote() {
        // given
        let (_config, store) = store();
        store.create("a.md").unwrap();

        // when
        let saved = store.write("a.md", "one\ntwo", LineEnding::Crlf).unwrap();

        // then
        assert_eq!(store.stamp("a.md").unwrap(), saved);
    }

    #[test]
    fn should_read_crlf_as_lf_and_write_it_back_as_crlf() {
        // given
        let (_config, store) = store();
        fs::create_dir_all(store.dir()).unwrap();
        fs::write(store.path("w.bat"), "a\r\nb\r\nc\n").unwrap();

        // when
        let ScratchContent::Editable { text, ending, .. } = store.read("w.bat").unwrap() else {
            panic!("w.bat should be editable");
        };
        store.write("w.bat", &format!("{text}d"), ending).unwrap();

        // then
        assert_eq!((text.as_str(), ending), ("a\nb\nc\n", LineEnding::Crlf));
        assert_eq!(fs::read_to_string(store.path("w.bat")).unwrap(), "a\r\nb\r\nc\r\nd");
    }

    #[test]
    fn should_keep_lf_when_most_line_breaks_are_lf() {
        // given
        let text = "a\r\nb\nc\n";

        // when
        let ending = LineEnding::detect(text);

        // then
        assert_eq!(ending, LineEnding::Lf);
    }

    #[test]
    fn should_open_non_utf8_and_over_1_mib_files_read_only() {
        // given
        let (_config, store) = store();
        fs::create_dir_all(store.dir()).unwrap();
        fs::write(store.path("latin1.txt"), [0x63, 0x61, 0x66, 0xe9]).unwrap();
        fs::write(store.path("big.log"), vec![b'a'; MAX_SCRATCH_BYTES as usize + 1]).unwrap();
        fs::write(store.path("limit.txt"), vec![b'a'; MAX_SCRATCH_BYTES as usize]).unwrap();
        // at the limit, but its LF line becomes CRLF when written back
        let mixed = "a\r\nb\r\nc\n".to_owned() + &"x".repeat(MAX_SCRATCH_BYTES as usize - 8);
        fs::write(store.path("mixed.txt"), mixed).unwrap();

        // when
        let kinds = ["latin1.txt", "big.log", "limit.txt", "mixed.txt"].map(|name| match store.read(name).unwrap() {
            ScratchContent::Editable { .. } => "editable",
            ScratchContent::TooLarge => "too large",
            ScratchContent::NotUtf8 => "not utf-8",
        });

        // then
        assert_eq!(kinds, ["not utf-8", "too large", "editable", "too large"]);
    }

    #[test]
    fn should_see_a_new_stamp_after_another_program_writes_even_the_same_size_at_the_same_time() {
        // given
        let (_config, store) = store();
        store.create("a.md").unwrap();
        store.write("a.md", "mine", LineEnding::Lf).unwrap();
        let (before, modified) = (store.stamp("a.md").unwrap(), fs::metadata(store.path("a.md")).unwrap().modified().unwrap());

        // when
        fs::write(store.path("a.md"), "ours").unwrap();
        File::options().write(true).open(store.path("a.md")).unwrap().set_modified(modified).unwrap();

        // then
        assert_ne!(store.stamp("a.md").unwrap(), before);
    }

    #[test]
    fn should_keep_the_stamp_when_only_the_modification_time_changes() {
        // given
        let (_config, store) = store();
        store.create("a.md").unwrap();
        let before = store.write("a.md", "mine", LineEnding::Lf).unwrap();

        // when
        let file = File::options().write(true).open(store.path("a.md")).unwrap();
        file.set_modified(SystemTime::now() + Duration::from_secs(60)).unwrap();

        // then
        assert_eq!(store.stamp("a.md").unwrap(), before);
    }

    #[test]
    fn should_name_errors_with_the_file_and_the_reason() {
        // given
        let (_config, store) = store();
        fs::create_dir_all(store.dir()).unwrap();

        // when
        let error = store.rename("a.sql", "b.sql").unwrap_err().to_string();

        // then
        let expected = format!("cannot rename {} to b.sql: ", store.path("a.sql").display());
        assert!(error.starts_with(&expected), "{error}");
    }

    #[test]
    fn should_name_a_new_scratch_after_the_taken_ones() {
        // given
        let taken = ["scratch.md", "scratch-2.md", "notes.md"];

        // when
        let names = (free_name(&[]), free_name(&taken));

        // then
        assert_eq!(names, ("scratch.md".to_owned(), "scratch-3.md".to_owned()));
    }

    #[test]
    fn should_measure_the_stem_in_bytes_for_selecting_it() {
        // given
        let names = ["scratch-2.md", "notes", "a.tar.gz", "zażółć.md"];

        // when
        let lengths = names.map(stem_len);

        // then
        assert_eq!(lengths, [9, 5, 5, 10]);
    }
}
