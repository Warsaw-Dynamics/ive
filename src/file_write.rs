//! Writing a panel's edits to its file: only the changed byte ranges, in place when no byte moves, else through a
//! temporary file renamed over the original. A file a panel may have mapped is never shrunk or rewritten in place
//! (SIGBUS); in-place writes keep every line break where it was, so a mapped file's line index stays right.

use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use codeview::document::Stamp;
use codeview::panel::Edit;

const SMALL_FILE_BYTES: u64 = 8 * 1024 * 1024;
const LARGE_FILE_BYTES: u64 = 256 * 1024 * 1024;

/// Temporary files of saves running at once in one folder get different names.
static STAGING_COUNT: AtomicU64 = AtomicU64::new(0);

/// A save: the file, the text the edits apply to, and what the file must be for the save to go ahead.
pub struct Target<'a> {
    pub path: &'a Path,
    /// the panel's base: the text as last read or written
    pub base: &'a [u8],
    pub expected: Expected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expected {
    /// the file as last read or written; any other state is a conflict
    Stamp(Stamp),
    /// whatever is there, or nothing: the user chose to overwrite it
    Anything,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteOutcome {
    InPlace,
    Rewritten,
}

/// A save that went through, and the file's stamp right after it, which marks the change on disk as IVE's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Written {
    pub outcome: WriteOutcome,
    pub stamp: Stamp,
}

#[derive(Debug)]
pub enum WriteError {
    /// the file changed or went away since its base was read
    Conflict,
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict => write!(f, "the file changed on disk"),
            Self::Io { path, source } => write!(f, "cannot save {}: {source}", path.display()),
        }
    }
}

/// The pause after the last edit before a file of `byte_len` bytes is saved: a rewrite costs more the larger it is.
pub fn save_delay(byte_len: u64) -> Duration {
    if byte_len <= SMALL_FILE_BYTES {
        return Duration::from_millis(500);
    }
    if byte_len <= LARGE_FILE_BYTES {
        return Duration::from_secs(2);
    }
    Duration::from_secs(10)
}

/// Applies `edits` (sorted, non-overlapping, in base coordinates) to the file. A file that is not as `expected`
/// is left untouched: [`WriteError::Conflict`].
pub fn apply_edits(target: &Target, edits: &[Edit]) -> Result<Written, WriteError> {
    let io = |source| WriteError::Io { path: target.path.to_path_buf(), source };
    let original = open_original(target)?;
    match (original, target.expected) {
        (Original::File(file), Expected::Stamp(_)) if keeps_every_line_break(target.base, edits) => {
            write_in_place(&file, edits).map_err(io)
        }
        (original, _) => rewrite(target, &original, edits).map_err(io),
    }
}

/// The file before the save, opened for writing so a read-only file fails here, before anything is written.
enum Original {
    File(File),
    Missing,
}

fn open_original(target: &Target) -> Result<Original, WriteError> {
    let io = |source| WriteError::Io { path: target.path.to_path_buf(), source };
    let file = match OpenOptions::new().read(true).write(true).open(target.path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return match target.expected {
                Expected::Stamp(_) => Err(WriteError::Conflict),
                Expected::Anything => Ok(Original::Missing),
            };
        }
        Err(error) => return Err(io(error)),
    };
    if let Expected::Stamp(expected) = target.expected
        && stamp_of(&file.metadata().map_err(io)?).map_err(io)? != expected
    {
        return Err(WriteError::Conflict);
    }
    Ok(Original::File(file))
}

fn stamp_of(metadata: &Metadata) -> io::Result<Stamp> {
    Ok((metadata.len(), metadata.modified()?))
}

/// Whether every edit replaces as many bytes as it removes, with line breaks at the same places.
fn keeps_every_line_break(base: &[u8], edits: &[Edit]) -> bool {
    edits.iter().all(|edit| {
        let replaced = slice(base, &edit.bytes);
        replaced.len() == edit.text.len() && memchr::memchr_iter(b'\n', replaced).eq(memchr::memchr_iter(b'\n', &edit.text))
    })
}

fn slice<'a>(bytes: &'a [u8], range: &Range<u64>) -> &'a [u8] {
    &bytes[range.start as usize..range.end as usize]
}

/// `pwrite`s each edit at its offset; the file keeps its length.
fn write_in_place(file: &File, edits: &[Edit]) -> io::Result<Written> {
    for edit in edits {
        write_at(file, &edit.text, edit.bytes.start)?;
    }
    file.sync_all()?;
    Ok(Written { outcome: WriteOutcome::InPlace, stamp: stamp_of(&file.metadata()?)? })
}

#[cfg(unix)]
fn write_at(file: &File, bytes: &[u8], offset: u64) -> io::Result<()> {
    std::os::unix::fs::FileExt::write_all_at(file, bytes, offset)
}

#[cfg(not(unix))]
fn write_at(mut file: &File, bytes: &[u8], offset: u64) -> io::Result<()> {
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::Start(offset))?;
    file.write_all(bytes)
}

/// Writes the new text to a temporary file in the same folder, then renames it over the original.
fn rewrite(target: &Target, original: &Original, edits: &[Edit]) -> io::Result<Written> {
    let dir = target.path.parent().unwrap_or(Path::new("."));
    let staging = dir.join(format!(".ive-save-{}-{}.tmp", std::process::id(), STAGING_COUNT.fetch_add(1, Ordering::Relaxed)));
    let renamed = write_staging(&staging, target, original, edits).and_then(|stamp| fs::rename(&staging, target.path).map(|()| stamp));
    let stamp = renamed.inspect_err(|_| drop(fs::remove_file(&staging)))?;
    // the new text is in place; a folder that cannot be synced only leaves the rename less durable
    let _ = File::open(dir).and_then(|dir| dir.sync_all());
    Ok(Written { outcome: WriteOutcome::Rewritten, stamp })
}

/// The new text in `staging`, with the original's permissions, synced; its stamp.
fn write_staging(staging: &Path, target: &Target, original: &Original, edits: &[Edit]) -> io::Result<Stamp> {
    let mut out = OpenOptions::new().write(true).create_new(true).open(staging)?;
    if let Original::File(file) = original {
        out.set_permissions(file.metadata()?.permissions())?;
    }
    let mut copier = Copier { out: &mut out, base: target.base, original: kept_source(original, target.expected) };
    let mut kept_from = 0;
    for edit in edits {
        copier.keep(kept_from..edit.bytes.start)?;
        copier.out.write_all(&edit.text)?;
        kept_from = edit.bytes.end;
    }
    copier.keep(kept_from..target.base.len() as u64)?;
    out.sync_all()?;
    stamp_of(&out.metadata()?)
}

/// The original file when it holds the base, so kept bytes may be copied inside the kernel.
fn kept_source(original: &Original, expected: Expected) -> Option<&File> {
    match (original, expected) {
        (Original::File(file), Expected::Stamp(_)) => Some(file),
        (Original::File(_) | Original::Missing, _) => None,
    }
}

/// Writes kept base bytes: cloned or copied from the original file where the system can, else from the base.
struct Copier<'a> {
    out: &'a mut File,
    base: &'a [u8],
    original: Option<&'a File>,
}

impl Copier<'_> {
    fn keep(&mut self, range: Range<u64>) -> io::Result<()> {
        let copied = match self.original {
            Some(original) => kernel_copy(original, self.out, range.clone()),
            None => 0,
        };
        self.out.write_all(slice(self.base, &(range.start + copied..range.end)))
    }
}

/// Copies what it can of `range` from `from` to the end of `to` without passing through IVE: a block-aligned start
/// of the file by cloning (btrfs, XFS), then by `copy_file_range`. Returns how many bytes it copied.
#[cfg(target_os = "linux")]
fn kernel_copy(from: &File, to: &mut File, range: Range<u64>) -> u64 {
    use std::io::{Seek, SeekFrom};
    let cloned = clone_start(from, to, &range);
    if cloned > 0 && to.seek(SeekFrom::Start(range.start + cloned)).is_err() {
        return 0;
    }
    cloned + copy_file_range(from, to, range.start + cloned..range.end)
}

#[cfg(not(target_os = "linux"))]
fn kernel_copy(_from: &File, _to: &mut File, _range: Range<u64>) -> u64 {
    0
}

/// Clones the whole blocks of `range` when it starts the file, sharing them with the original; 0 where the file
/// system cannot (ext4).
#[cfg(target_os = "linux")]
fn clone_start(from: &File, to: &File, range: &Range<u64>) -> u64 {
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::io::AsRawFd;
    let block = from.metadata().map_or(0, |metadata| metadata.blksize());
    if range.start != 0 || block == 0 {
        return 0;
    }
    let length = range.end / block * block;
    let clone = libc::file_clone_range { src_fd: i64::from(from.as_raw_fd()), src_offset: 0, src_length: length, dest_offset: 0 };
    // SAFETY: both descriptors are open for the call; the kernel reads `clone` only
    let result = unsafe { libc::ioctl(to.as_raw_fd(), libc::FICLONERANGE, &clone) };
    if length == 0 || result != 0 {
        return 0;
    }
    length
}

/// Copies `range` of `from` to `to`'s position inside the kernel for as long as that works; how much it copied.
#[cfg(target_os = "linux")]
fn copy_file_range(from: &File, to: &File, range: Range<u64>) -> u64 {
    use std::os::unix::io::AsRawFd;
    let mut offset = range.start as i64;
    while (offset as u64) < range.end {
        let left = (range.end - offset as u64) as usize;
        // SAFETY: both descriptors are open for the call; `offset` outlives it
        let copied = unsafe { libc::copy_file_range(from.as_raw_fd(), &mut offset, to.as_raw_fd(), std::ptr::null_mut(), left, 0) };
        if copied <= 0 {
            break;
        }
    }
    offset as u64 - range.start
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    fn edit(bytes: Range<u64>, text: &[u8]) -> Edit {
        Edit { bytes, text: text.to_vec() }
    }

    /// A file holding `bytes` in a new folder, and its stamp.
    fn file_of(bytes: &[u8]) -> (tempfile::TempDir, PathBuf, Stamp) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        fs::write(&path, bytes).unwrap();
        let stamp = stamp_of(&fs::metadata(&path).unwrap()).unwrap();
        (dir, path, stamp)
    }

    #[cfg(unix)]
    fn inode(path: &Path) -> u64 {
        fs::metadata(path).unwrap().ino()
    }

    #[test]
    #[cfg(unix)]
    fn should_write_same_length_edits_in_place() {
        // given
        let base = b"one\ntwo\nthree\n";
        let (_dir, path, stamp) = file_of(base);
        let before = inode(&path);

        // when
        let written = apply_edits(&Target { path: &path, base, expected: Expected::Stamp(stamp) }, &[edit(0..3, b"ONE"), edit(8..13, b"THREE")]);

        // then
        assert_eq!(written.unwrap().outcome, WriteOutcome::InPlace);
        assert_eq!(fs::read(&path).unwrap(), b"ONE\ntwo\nTHREE\n");
        assert_eq!(inode(&path), before);
    }

    #[test]
    fn should_rewrite_when_a_same_length_edit_moves_a_line_break() {
        // given
        let base = b"ab\ncd\n";
        let (_dir, path, stamp) = file_of(base);

        // when
        let written = apply_edits(&Target { path: &path, base, expected: Expected::Stamp(stamp) }, &[edit(1..3, b"\nb")]);

        // then
        assert_eq!(written.unwrap().outcome, WriteOutcome::Rewritten);
        assert_eq!(fs::read(&path).unwrap(), b"a\nbcd\n");
    }

    #[test]
    #[cfg(unix)]
    fn should_rewrite_through_a_temp_file_when_length_changes() {
        // given
        let base = b"one\ntwo\nthree\n";
        let (dir, path, stamp) = file_of(base);
        // a mode change leaves the modification time, so the stamp still holds
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        let before = inode(&path);

        // when
        let written = apply_edits(&Target { path: &path, base, expected: Expected::Stamp(stamp) }, &[edit(3..3, b"!"), edit(4..8, b"")]);

        // then
        let written = written.unwrap();
        assert_eq!(written.outcome, WriteOutcome::Rewritten);
        assert_eq!(fs::read(&path).unwrap(), b"one!\nthree\n");
        assert_ne!(inode(&path), before);
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o640);
        assert_eq!(written.stamp, stamp_of(&fs::metadata(&path).unwrap()).unwrap());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1, "no temporary file left");
    }

    #[test]
    fn should_refuse_when_the_file_changed_since_base() {
        // given
        let base = b"one\n";
        let (_dir, path, stamp) = file_of(base);
        fs::write(&path, b"theirs, longer\n").unwrap();

        // when
        let written = apply_edits(&Target { path: &path, base, expected: Expected::Stamp(stamp) }, &[edit(0..0, b"x")]);

        // then
        assert!(matches!(written, Err(WriteError::Conflict)), "{written:?}");
        assert_eq!(fs::read(&path).unwrap(), b"theirs, longer\n");
    }

    #[test]
    fn should_overwrite_a_changed_file_with_the_base_and_edits_when_asked() {
        // given
        let base = b"one\n";
        let (_dir, path, _stamp) = file_of(base);
        fs::write(&path, b"theirs\n").unwrap();

        // when
        let written = apply_edits(&Target { path: &path, base, expected: Expected::Anything }, &[edit(0..0, b"x")]);

        // then
        assert_eq!(written.unwrap().outcome, WriteOutcome::Rewritten);
        assert_eq!(fs::read(&path).unwrap(), b"xone\n");
    }

    #[test]
    fn should_keep_crlf_tabs_and_invalid_utf8_outside_the_edits() {
        // given
        let base = b"a\tb\r\n\xff\xfe\r\nend";
        let (_dir, path, stamp) = file_of(base);

        // when
        let written = apply_edits(&Target { path: &path, base, expected: Expected::Stamp(stamp) }, &[edit(2..2, b"\x80x"), edit(12..12, b"!")]);

        // then
        assert!(written.is_ok(), "{written:?}");
        assert_eq!(fs::read(&path).unwrap(), b"a\t\x80xb\r\n\xff\xfe\r\nend!");
    }

    #[test]
    fn should_copy_the_kept_bytes_of_a_large_file() {
        // given
        let base: Vec<u8> = (0..3 * 1024 * 1024u32).map(|at| b"abcdefghij\n"[(at % 11) as usize]).collect();
        let (_dir, path, stamp) = file_of(&base);
        let edits = [edit(1_000_000..1_000_010, b"x"), edit(2_000_000..2_000_000, b"inserted\n")];

        // when
        let written = apply_edits(&Target { path: &path, base: &base, expected: Expected::Stamp(stamp) }, &edits);

        // then
        assert!(written.is_ok(), "{written:?}");
        let mut expected = base.clone();
        expected.splice(2_000_000..2_000_000, b"inserted\n".iter().copied());
        expected.splice(1_000_000..1_000_010, *b"x");
        assert!(fs::read(&path).unwrap() == expected, "the file differs from the edited base");
    }

    #[test]
    #[cfg(unix)]
    fn should_report_a_read_only_file_as_io_error() {
        // given
        let base = b"one\n";
        let (_dir, path, stamp) = file_of(base);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
        // SAFETY: geteuid has no preconditions
        if unsafe { libc::geteuid() } == 0 {
            return; // root writes read-only files
        }

        // when
        let written = apply_edits(&Target { path: &path, base, expected: Expected::Stamp(stamp) }, &[edit(0..0, b"x")]);

        // then
        let message = written.as_ref().map_err(ToString::to_string).unwrap_err();
        assert!(message.starts_with(&format!("cannot save {}: ", path.display())), "{message}");
        assert_eq!(fs::read(&path).unwrap(), b"one\n");
    }

    #[test]
    fn should_grow_the_delay_with_file_size() {
        // given
        let sizes = [1024, 100 * 1024 * 1024, 1024 * 1024 * 1024];

        // when
        let delays = sizes.map(save_delay);

        // then
        assert_eq!(delays, [Duration::from_millis(500), Duration::from_secs(2), Duration::from_secs(10)]);
    }
}
