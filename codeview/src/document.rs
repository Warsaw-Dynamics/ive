use std::borrow::Cow;
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use gpui::SharedString;
use memmap2::Mmap;

use crate::colors::ColorKey;
use crate::panel::LineSource;
use crate::parallel::in_parallel;
use crate::syntax::{Highlight, MAX_HIGHLIGHT_BYTES};

/// Beyond this the line index alone could reach hundreds of MB; refuse instead.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
/// The index keeps the start of every Nth line; reaching any line scans at most N-1 newlines.
const LINES_PER_CHECKPOINT: usize = 1024;
/// Files up to this size are read into memory rather than mapped; see [`Document::open`].
const MAX_COPIED_BYTES: u64 = MAX_HIGHLIGHT_BYTES as u64;
/// Indexing work unit; small enough to spread a few hundred MB over all cores.
const PARALLEL_CHUNK_BYTES: usize = 8 * 1024 * 1024;
/// Text layout of a longer line would stall the UI (think minified JSON); the rest is cut off.
pub const MAX_LINE_CHARS: usize = 10_000;
/// Like git, a NUL byte in the first 8000 bytes marks the file as binary.
const BINARY_SNIFF_BYTES: usize = 8000;
const TAB_AS_SPACES: &str = "    ";

/// A text file opened for viewing: memory-mapped, with an index of line starts, so opening
/// costs one newline scan and only the lines on screen are ever decoded.
pub struct Document {
    path: PathBuf,
    text: Text,
}

/// Text held in memory and read by line ranges, like a [`Document`]: a scratch's text.
pub struct TextSource(Text);

/// Bytes and the index of their line starts.
struct Text {
    bytes: Arc<Bytes>,
    index: LineIndex,
}

/// The document's bytes, shared with a background parse without copying the file.
#[derive(Clone)]
pub struct Source(Arc<Bytes>);

impl Source {
    /// Takes `bytes`; copies them only to drop spare capacity.
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self(Arc::new(Bytes::Owned(bytes.into_boxed_slice())))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

enum Bytes {
    Empty,
    Mapped(Mmap),
    Owned(Box<[u8]>),
}

impl Bytes {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Empty => &[],
            Self::Mapped(map) => map,
            Self::Owned(bytes) => bytes,
        }
    }

    /// Indexing touches every page, which would show the whole file in the process's resident memory
    /// (876 MB for a 10M-line log) although the kernel can reclaim it; unmap `range` from this process.
    /// The pages stay in the page cache, so later reads are still fast.
    fn release_pages(&self, range: Range<usize>) {
        #[cfg(unix)]
        if let Self::Mapped(map) = self {
            // SAFETY: DontNeed on a read-only shared file mapping only drops this process's page table
            // entries; the next access maps the same file contents back in.
            let _ignored_hint = unsafe {
                map.unchecked_advise_range(memmap2::UncheckedAdvice::DontNeed, range.start, range.len())
            };
        }
    }
}

struct LineIndex {
    /// byte offset of lines 0, N, 2N, ... where N is `LINES_PER_CHECKPOINT`
    checkpoints: Vec<usize>,
    line_count: usize,
}

impl LineIndex {
    /// Scans in parallel chunks: first counting newlines per chunk, then (knowing each chunk's first
    /// line number) collecting the checkpoints that fall in it.
    /// `release` is called with each byte range once it is scanned.
    fn build(bytes: &[u8], release: &(dyn Fn(Range<usize>) + Sync)) -> Self {
        Self::build_in_chunks(bytes, PARALLEL_CHUNK_BYTES, release)
    }

    fn build_in_chunks(bytes: &[u8], chunk_bytes: usize, release: &(dyn Fn(Range<usize>) + Sync)) -> Self {
        let chunks: Vec<&[u8]> = bytes.chunks(chunk_bytes.max(1)).collect();
        let counts: Vec<usize> = in_parallel(&chunks, |chunk| memchr::memchr_iter(b'\n', chunk).count());
        let first_newline_numbers: Vec<usize> = counts
            .iter()
            .scan(0, |before, count| {
                let first = *before;
                *before += count;
                Some(first)
            })
            .collect();
        let jobs: Vec<(usize, &[u8], usize)> = chunks
            .iter()
            .enumerate()
            .map(|(index, chunk)| (index * chunk_bytes, *chunk, first_newline_numbers[index]))
            .collect();
        let parts = in_parallel(&jobs, |&(chunk_start, chunk, newlines_before)| {
            let checkpoints = checkpoints_in(chunk, chunk_start, newlines_before);
            release(chunk_start..chunk_start + chunk.len());
            checkpoints
        });
        let newlines: usize = counts.iter().sum();
        // a trailing newline ends the last line rather than starting an empty one
        let ends_with_newline = bytes.last() == Some(&b'\n');
        let line_count = if ends_with_newline { newlines } else { newlines + 1 };
        let checkpoints = std::iter::once(0).chain(parts.into_iter().flatten()).collect();
        Self { checkpoints, line_count: line_count.max(1) }
    }
}

/// Byte offsets of the line starts after newline number `LINES_PER_CHECKPOINT`, `2 * LINES_PER_CHECKPOINT`, ...
/// that fall in `chunk`, given how many newlines precede it.
fn checkpoints_in(chunk: &[u8], chunk_start: usize, newlines_before: usize) -> Vec<usize> {
    let mut newline_number = newlines_before;
    let mut checkpoints = Vec::new();
    for at in memchr::memchr_iter(b'\n', chunk) {
        newline_number += 1;
        if newline_number.is_multiple_of(LINES_PER_CHECKPOINT) {
            checkpoints.push(chunk_start + at + 1);
        }
    }
    checkpoints
}

#[derive(Debug)]
pub enum DocumentError {
    TooLarge { path: PathBuf, bytes: u64 },
    NotText(PathBuf),
    Unreadable { path: PathBuf, source: io::Error },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { path, bytes } => write!(
                f,
                "{} is too large to open: {bytes} bytes (limit {MAX_FILE_BYTES})",
                path.display()
            ),
            Self::NotText(path) => write!(f, "{} looks like a binary file", path.display()),
            Self::Unreadable { path, source } => write!(f, "cannot read {}: {source}", path.display()),
        }
    }
}

impl Document {
    pub fn open(path: &Path) -> Result<Self, DocumentError> {
        Self::open_stamped(path).map(|(document, _)| document)
    }

    /// Opens the file at `path`, with the stamp the open file had both before and after it was read; `None` when
    /// the two differ (another program wrote it meanwhile) or the system keeps no modification times.
    pub fn open_stamped(path: &Path) -> Result<(Self, Option<Stamp>), DocumentError> {
        let unreadable = |source| DocumentError::Unreadable { path: path.to_path_buf(), source };
        let file = File::open(path).map_err(unreadable)?;
        let before = stamp_of(&file.metadata().map_err(unreadable)?);
        let document = Self::read(path, &file)?;
        let after = file.metadata().ok().and_then(|metadata| stamp_of(&metadata));
        Ok((document, before.filter(|_| before == after)))
    }

    /// The text of `file`, opened at `path`: copied, or mapped when large.
    fn read(path: &Path, file: &File) -> Result<Self, DocumentError> {
        let unreadable = |source| DocumentError::Unreadable { path: path.to_path_buf(), source };
        let size = file.metadata().map_err(unreadable)?.len();
        if size > MAX_FILE_BYTES {
            return Err(DocumentError::TooLarge { path: path.to_path_buf(), bytes: size });
        }
        let bytes = if size == 0 {
            // mapping an empty file fails on some platforms
            Bytes::Empty
        } else if size <= MAX_COPIED_BYTES {
            // a copy: another program rewriting the file in place (`>` in a shell) changes neither the text an
            // editor's edits apply to nor its length
            Bytes::Owned(read_all(file, size).map_err(unreadable)?.into_boxed_slice())
        } else {
            // SAFETY: the map is read-only. If another process truncates the file while it is open,
            // reading past the new end raises SIGBUS; accepted for files this large, as other editors do.
            Bytes::Mapped(unsafe { Mmap::map(file) }.map_err(unreadable)?)
        };
        if looks_binary(bytes.as_slice()) {
            return Err(DocumentError::NotText(path.to_path_buf()));
        }
        let index = LineIndex::build(bytes.as_slice(), &|range| bytes.release_pages(range));
        Ok(Self { path: path.to_path_buf(), text: Text { bytes: Arc::new(bytes), index } })
    }

    pub fn bytes(&self) -> &[u8] {
        self.text.bytes()
    }

    /// Whether the bytes are the file's own, mapped, rather than a copy: another program writing the file in place
    /// changes them, and truncating it makes reading past its new end raise SIGBUS.
    pub fn is_mapped(&self) -> bool {
        matches!(*self.text.bytes, Bytes::Mapped(_))
    }

    pub fn byte_len(&self) -> usize {
        self.bytes().len()
    }

    pub fn source(&self) -> Source {
        Source(self.text.bytes.clone())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map_or_else(|| self.path.display().to_string(), |name| name.to_string_lossy().into_owned())
    }

    pub fn line_count(&self) -> usize {
        self.text.index.line_count
    }

    /// Lines `range` (clamped to the document) ready to display.
    pub fn lines(&self, range: Range<usize>) -> Vec<DisplayLine> {
        self.text.lines(range)
    }
}

impl TextSource {
    pub fn new(text: &str) -> Self {
        Self::from_bytes(text.as_bytes())
    }

    /// Bytes as they are stored, valid UTF-8 or not.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::from_source(Source(Arc::new(Bytes::Owned(bytes.into()))))
    }

    /// Shares `source`'s bytes without copying them.
    pub fn from_source(source: Source) -> Self {
        let index = LineIndex::build(source.as_bytes(), &|_| {});
        Self(Text { bytes: source.0, index })
    }
}

impl Text {
    fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    fn lines(&self, range: Range<usize>) -> Vec<DisplayLine> {
        let line_count = self.index.line_count;
        let range = range.start.min(line_count)..range.end.min(line_count);
        let bytes = self.bytes();
        let mut start = self.line_start(range.start);
        range
            .map(|_| {
                let end = memchr::memchr(b'\n', &bytes[start..]).map_or(bytes.len(), |at| start + at);
                let line = DisplayLine::of(&bytes[start..end], start);
                start = (end + 1).min(bytes.len());
                line
            })
            .collect()
    }

    fn line_start(&self, line: usize) -> usize {
        let bytes = self.bytes();
        let checkpoint = line / LINES_PER_CHECKPOINT;
        let mut start = self.index.checkpoints[checkpoint];
        for _ in 0..line % LINES_PER_CHECKPOINT {
            start = memchr::memchr(b'\n', &bytes[start..]).map_or(bytes.len(), |at| start + at + 1);
        }
        start
    }

    /// Like [`Self::line_start`], but `line_count` and beyond map to the end of the text.
    fn clamped_line_start(&self, line: usize) -> u64 {
        if line >= self.index.line_count {
            return self.bytes().len() as u64;
        }
        self.line_start(line) as u64
    }

    fn line_of(&self, byte: u64) -> usize {
        let bytes = self.bytes();
        let byte = usize::try_from(byte).unwrap_or(usize::MAX).min(bytes.len());
        let checkpoint = self.index.checkpoints.partition_point(|&start| start <= byte) - 1;
        let start = self.index.checkpoints[checkpoint];
        checkpoint * LINES_PER_CHECKPOINT + memchr::memchr_iter(b'\n', &bytes[start..byte]).count()
    }

    /// The whole text, when it is small enough to highlight.
    fn highlightable(&self) -> Option<Source> {
        (self.bytes().len() <= MAX_HIGHLIGHT_BYTES).then(|| Source(self.bytes.clone()))
    }

    /// Bytes `range`, clamped to the text.
    fn slice(&self, range: Range<u64>) -> &[u8] {
        let bytes = self.bytes();
        let clamp = |at: u64| usize::try_from(at).unwrap_or(usize::MAX).min(bytes.len());
        let start = clamp(range.start);
        &bytes[start..clamp(range.end).max(start)]
    }
}

impl LineSource for Document {
    fn line_count(&self) -> usize {
        self.text.index.line_count
    }

    fn lines(&self, range: Range<usize>) -> Vec<DisplayLine> {
        self.text.lines(range)
    }

    fn source(&self) -> Option<Source> {
        self.text.highlightable()
    }

    fn bytes(&self, range: Range<u64>) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.text.slice(range))
    }

    fn byte_len(&self) -> u64 {
        self.text.bytes().len() as u64
    }

    fn line_start(&self, line: usize) -> u64 {
        self.text.clamped_line_start(line)
    }

    fn line_of(&self, byte: u64) -> usize {
        self.text.line_of(byte)
    }
}

impl LineSource for TextSource {
    fn line_count(&self) -> usize {
        self.0.index.line_count
    }

    fn lines(&self, range: Range<usize>) -> Vec<DisplayLine> {
        self.0.lines(range)
    }

    fn source(&self) -> Option<Source> {
        self.0.highlightable()
    }

    fn bytes(&self, range: Range<u64>) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.slice(range))
    }

    fn byte_len(&self) -> u64 {
        self.0.bytes().len() as u64
    }

    fn line_start(&self, line: usize) -> u64 {
        self.0.clamped_line_start(line)
    }

    fn line_of(&self, byte: u64) -> usize {
        self.0.line_of(byte)
    }
}

/// A file's size and modification time: when both are as before, the file is not read again.
pub type Stamp = (u64, SystemTime);

/// What re-reading an open document found.
pub enum Reread {
    /// size and modification time as when it was last read
    Unchanged,
    Changed(Document, Option<Stamp>),
    Gone,
    Failed(String),
}

/// Re-reads the document at `path` unless its stamp is still `known`. Reads the disk; run it on the background
/// executor.
pub fn reread(path: &Path, known: Option<Stamp>) -> Reread {
    let now = stamp(path);
    if now.is_some() && now == known {
        return Reread::Unchanged;
    }
    match Document::open_stamped(path) {
        Ok((document, stamp)) => Reread::Changed(document, stamp),
        Err(error) if path.exists() => Reread::Failed(error.to_string()),
        Err(_) => Reread::Gone,
    }
}

/// The stamp of the file at `path` now; `None` when it cannot be read.
pub fn stamp(path: &Path) -> Option<Stamp> {
    stamp_of(&fs::metadata(path).ok()?)
}

fn stamp_of(metadata: &fs::Metadata) -> Option<Stamp> {
    Some((metadata.len(), metadata.modified().ok()?))
}

/// The file's bytes; `size` is what it held when opened, a capacity hint only.
fn read_all(mut file: &File, size: u64) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(size as usize);
    io::Read::read_to_end(&mut file, &mut bytes)?;
    Ok(bytes)
}

fn looks_binary(bytes: &[u8]) -> bool {
    memchr::memchr(0, &bytes[..bytes.len().min(BINARY_SNIFF_BYTES)]).is_some()
}

/// A line ready to display: tabs expanded, a trailing `\r` dropped, invalid UTF-8 replaced, and cut at
/// `MAX_LINE_CHARS` with ` …`. It remembers where each file byte landed, so highlights can follow.
pub struct DisplayLine {
    pub text: SharedString,
    /// file offset of the line's first byte
    start: usize,
    /// file offset of its line break (or of the end of the file), past a `\r` and any cut-off rest
    end: usize,
    /// display offset of each kept file byte, plus one entry for the end of the kept bytes
    offsets: Vec<usize>,
}

impl DisplayLine {
    /// `raw` is the line without its `\n`; `start` its offset in the file.
    pub fn of(raw: &[u8], start: usize) -> Self {
        let end = start + raw.len();
        let raw = raw.strip_suffix(b"\r").unwrap_or(raw);
        // a line longer than this in bytes is longer than the char limit too
        let capped = &raw[..raw.len().min(MAX_LINE_CHARS * 4)];
        let (mut text, mut offsets) = decode(capped);
        let cut = offsets.len() < raw.len();
        // one entry past the decoded bytes, so `file_bytes` covers only what is shown
        offsets.push(text.len());
        if cut {
            text.push_str(" …");
        }
        Self { text: text.into(), start, end, offsets }
    }

    /// The file offset of the line's break, or of the file's end on the last line.
    pub fn end(&self) -> usize {
        self.end
    }

    /// The file offset shown at display column `column`: a column inside an expanded tab is the offset after
    /// the tab, one past the shown text is the end of the shown bytes.
    pub fn offset_at(&self, column: usize) -> usize {
        let kept = self.offsets.len() - 1;
        let display = self.text.char_indices().nth(column).map_or(usize::MAX, |(at, _)| at);
        self.start + self.offsets[..kept].partition_point(|offset| *offset < display)
    }

    /// The display column of file offset `offset`, clamped to the shown bytes.
    pub fn column_of(&self, offset: usize) -> usize {
        let index = offset.saturating_sub(self.start).min(self.offsets.len() - 1);
        let display = self.offsets[index];
        self.text.char_indices().take_while(|(at, _)| *at < display).count()
    }

    /// The file bytes this line shows, without the cut-off rest.
    pub fn file_bytes(&self) -> Range<usize> {
        self.start..self.start + self.offsets.len() - 1
    }

    /// `highlights` (file offsets) as ranges of `text`, clipped to what the line shows.
    pub fn place(&self, highlights: &[Highlight]) -> Vec<(Range<usize>, ColorKey)> {
        let shown = self.file_bytes();
        highlights
            .iter()
            .filter(|highlight| highlight.bytes.start < shown.end && highlight.bytes.end > shown.start)
            .map(|highlight| {
                let from = highlight.bytes.start.max(shown.start) - shown.start;
                let to = highlight.bytes.end.min(shown.end) - shown.start;
                (self.offsets[from]..self.offsets[to], highlight.key)
            })
            .filter(|(range, _)| !range.is_empty())
            .collect()
    }
}

/// Decodes up to `MAX_LINE_CHARS` characters (a tab counts as one, an invalid sequence as one U+FFFD, as
/// `String::from_utf8_lossy` does); returns the text and the display offset of each decoded file byte.
fn decode(raw: &[u8]) -> (String, Vec<usize>) {
    let mut text = String::new();
    let mut offsets = Vec::with_capacity(raw.len() + 1);
    let mut chars = 0;
    for chunk in raw.utf8_chunks() {
        let pieces = chunk.valid().chars().map(|ch| (ch, ch.len_utf8()));
        let invalid = (!chunk.invalid().is_empty()).then_some(('\u{fffd}', chunk.invalid().len()));
        for (ch, raw_len) in pieces.chain(invalid) {
            if chars == MAX_LINE_CHARS {
                return (text, offsets);
            }
            offsets.extend(std::iter::repeat_n(text.len(), raw_len));
            match ch {
                '\t' => text.push_str(TAB_AS_SPACES),
                _ => text.push(ch),
            }
            chars += 1;
        }
    }
    (text, offsets)
}

#[cfg(test)]
// expected highlight positions are lists of one range on purpose
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use super::*;
    use crate::colors::ColorKey;
    use crate::syntax::Highlight;
    use std::fs;
    use std::time::Instant;

    fn open_text(text: &[u8]) -> (tempfile::TempDir, Document) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        fs::write(&path, text).unwrap();
        let document = Document::open(&path).unwrap();
        (dir, document)
    }

    #[test]
    fn should_keep_a_small_files_text_when_another_program_rewrites_it_in_place() {
        // given
        let (dir, document) = open_text(b"first line\nsecond line\n");

        // when
        fs::write(dir.path().join("file.txt"), b"x\n").unwrap();

        // then
        assert_eq!(document.bytes(), b"first line\nsecond line\n");
    }

    #[test]
    fn should_take_the_stamp_of_the_file_it_read() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        fs::write(&path, b"text\n").unwrap();

        // when
        let (document, stamp) = Document::open_stamped(&path).unwrap();

        // then
        assert_eq!((document.bytes(), stamp), (&b"text\n"[..], super::stamp(&path)));
    }

    #[test]
    fn should_map_only_files_too_large_to_copy() {
        // given
        let (_small_dir, small) = open_text(b"text\n");

        // when
        let (_large_dir, large) = open_text(&vec![b'\n'; MAX_COPIED_BYTES as usize + 1]);

        // then
        assert_eq!((small.is_mapped(), large.is_mapped()), (false, true));
    }

    fn texts(lines: Vec<DisplayLine>) -> Vec<String> {
        lines.into_iter().map(|line| line.text.to_string()).collect()
    }

    fn highlight(bytes: Range<usize>) -> Highlight {
        Highlight { bytes, key: ColorKey::TEXT }
    }

    fn placed(line: &DisplayLine, highlights: &[Highlight]) -> Vec<Range<usize>> {
        line.place(highlights).into_iter().map(|(range, _)| range).collect()
    }

    #[test]
    fn should_place_highlight_after_tab_expansion() {
        // given
        let line = DisplayLine::of(b"\tx = 1", 100);

        // when
        let ranges = placed(&line, &[highlight(101..102)]);

        // then
        assert_eq!((line.text.as_ref(), ranges), ("    x = 1", vec![4..5]));
    }

    #[test]
    fn should_place_highlight_after_invalid_byte() {
        // given
        let line = DisplayLine::of(b"caf\xe9 x", 0);

        // when
        let ranges = placed(&line, &[highlight(5..6)]);

        // then
        assert_eq!(&line.text[ranges[0].clone()], "x");
    }

    #[test]
    fn should_clip_highlight_to_line_without_carriage_return() {
        // given
        let line = DisplayLine::of(b"ab\r", 10);

        // when
        let ranges = placed(&line, &[highlight(5..20)]);

        // then
        assert_eq!((line.file_bytes(), ranges), (10..12, vec![0..2]));
    }

    #[test]
    fn should_report_only_decoded_bytes_of_cut_line() {
        // given
        let raw = "x".repeat(MAX_LINE_CHARS * 3);

        // when
        let line = DisplayLine::of(raw.as_bytes(), 0);

        // then
        assert_eq!(line.file_bytes(), 0..MAX_LINE_CHARS);
    }

    #[test]
    fn should_mark_line_cut_by_the_byte_cap_as_cut() {
        // given
        let raw = "😀".repeat(MAX_LINE_CHARS + 1);

        // when
        let line = DisplayLine::of(raw.as_bytes(), 0);

        // then
        assert!(line.text.ends_with(" …"), "{} chars, no ellipsis", line.text.chars().count());
    }

    #[test]
    fn should_drop_highlight_past_the_cut_and_clip_one_crossing_it() {
        // given
        let raw = "x".repeat(MAX_LINE_CHARS + 50);
        let line = DisplayLine::of(raw.as_bytes(), 0);

        // when
        let crossing = highlight(MAX_LINE_CHARS - 5..MAX_LINE_CHARS + 5);
        let ranges = placed(&line, &[crossing, highlight(MAX_LINE_CHARS + 10..MAX_LINE_CHARS + 20)]);

        // then
        assert_eq!(ranges, [MAX_LINE_CHARS - 5..MAX_LINE_CHARS]);
    }

    #[test]
    fn should_map_columns_to_file_offsets_across_tabs_and_wide_characters() {
        // given
        let line = DisplayLine::of("\tżb".as_bytes(), 100);

        // when
        let offsets = [0, 2, 4, 5, 6, 99].map(|column| line.offset_at(column));

        // then
        assert_eq!(offsets, [100, 101, 101, 103, 104, 104]);
    }

    #[test]
    fn should_map_file_offsets_to_columns_and_clamp_to_the_shown_text() {
        // given
        let line = DisplayLine::of("\tżb\r".as_bytes(), 100);

        // when
        let columns = [100, 101, 103, 104, 105, 0].map(|offset| line.column_of(offset));

        // then
        assert_eq!((columns, line.end()), ([0, 4, 5, 6, 6, 0], 105));
    }

    #[test]
    fn should_report_file_bytes_of_each_line() {
        // given
        let (_dir, document) = open_text(b"ab\ncd\n");

        // when
        let ranges: Vec<_> = document.lines(0..2).iter().map(DisplayLine::file_bytes).collect();

        // then
        assert_eq!(ranges, [0..2, 3..5]);
    }

    #[test]
    fn should_map_between_lines_and_bytes_past_checkpoints() {
        // given
        let line_count = LINES_PER_CHECKPOINT * 3 + 5;
        let source = TextSource::new(&"abc\n".repeat(line_count));
        let line = LINES_PER_CHECKPOINT * 2 + 7;

        // when
        let start = source.line_start(line);

        // then
        assert_eq!(start, line as u64 * 4);
        assert_eq!(source.line_of(start + 2), line);
        assert_eq!(source.line_of(start), line);
        assert_eq!(source.line_of(start - 1), line - 1);
        assert_eq!(source.line_start(line_count), source.byte_len());
        assert_eq!(source.line_of(u64::MAX), line_count);
    }

    #[test]
    fn should_split_lines_with_tabs_expanded_and_crlf_stripped() {
        // given
        let (_dir, document) = open_text(b"fn main() {\n\tprintln!();\r\n}\n");

        // when
        let lines = texts(document.lines(0..10));

        // then
        assert_eq!(document.line_count(), 3);
        assert_eq!(lines, ["fn main() {", "    println!();", "}"]);
    }

    #[test]
    fn should_show_empty_file_as_one_empty_line() {
        // given
        let (_dir, document) = open_text(b"");

        // when
        let lines = texts(document.lines(0..1));

        // then
        assert_eq!((document.line_count(), lines), (1, vec![String::new()]));
    }

    #[test]
    fn should_read_lines_across_checkpoints() {
        // given
        let text: String = (0..3 * LINES_PER_CHECKPOINT).map(|n| format!("line {n}\n")).collect();
        let (_dir, document) = open_text(text.as_bytes());

        // when
        let lines = texts(document.lines(LINES_PER_CHECKPOINT - 1..LINES_PER_CHECKPOINT + 2));

        // then
        assert_eq!(document.line_count(), 3 * LINES_PER_CHECKPOINT);
        assert_eq!(lines, ["line 1023", "line 1024", "line 1025"]);
    }

    #[test]
    fn should_build_same_index_whatever_the_chunking() {
        // given
        let text: String = (0..5 * LINES_PER_CHECKPOINT).map(|n| format!("{}\n", "x".repeat(n % 7))).collect();
        let single = LineIndex::build_in_chunks(text.as_bytes(), usize::MAX, &|_| {});

        // when
        let chunked = [1, 3, 64, 4096].map(|chunk_bytes| LineIndex::build_in_chunks(text.as_bytes(), chunk_bytes, &|_| {}));

        // then
        // line 0, then one after every 1024th newline (the last one is the end of the file)
        assert_eq!(single.checkpoints.len(), 6);
        for index in chunked {
            assert_eq!((index.checkpoints.clone(), index.line_count), (single.checkpoints.clone(), single.line_count));
        }
    }

    #[test]
    fn should_cut_very_long_line() {
        // given
        let (_dir, document) = open_text("x".repeat(MAX_LINE_CHARS * 3).as_bytes());

        // when
        let line = document.lines(0..1).remove(0).text;

        // then
        assert_eq!(line.chars().count(), MAX_LINE_CHARS + 2);
        assert!(line.ends_with(" …"));
    }

    #[test]
    fn should_replace_invalid_utf8_instead_of_refusing_the_file() {
        // given
        let (_dir, document) = open_text(b"caf\xe9 latin-1\n");

        // when
        let lines = texts(document.lines(0..1));

        // then
        assert_eq!(lines, ["caf\u{fffd} latin-1"]);
    }

    #[test]
    fn should_reject_binary_file() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("image.png");
        fs::write(&path, [0x89, b'P', b'N', b'G', 0x00, 0xff]).unwrap();

        // when
        let result = Document::open(&path);

        // then
        assert!(matches!(result, Err(DocumentError::NotText(_))));
    }

    #[test]
    fn should_not_read_a_file_whose_size_and_modification_time_are_unchanged() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.md");
        fs::write(&path, "one\n").unwrap();
        let known = stamp(&path);

        // when
        let reread = reread(&path, known);

        // then
        assert!(matches!(reread, Reread::Unchanged));
    }

    #[test]
    fn should_read_lines_and_bytes_of_text_in_memory() {
        // given
        let source = TextSource::new("one\n\ttwo\nthree");

        // when
        let lines = texts(LineSource::lines(&source, 1..9));
        let bytes = source.bytes(4..100);

        // then
        assert_eq!((source.line_count(), lines), (3, vec!["    two".to_owned(), "three".to_owned()]));
        assert_eq!((&*bytes, source.byte_len()), (&b"\ttwo\nthree"[..], 14));
    }

    /// Run with: IVE_BIG_FILE=/path/to/file cargo test --release -- --ignored --nocapture
    #[test]
    #[ignore = "needs a large file in IVE_BIG_FILE"]
    fn should_open_big_file_quickly() {
        // given
        let path = PathBuf::from(std::env::var("IVE_BIG_FILE").expect("IVE_BIG_FILE not set"));

        // when
        let started = Instant::now();
        let document = Document::open(&path).unwrap();
        let opened = started.elapsed();
        let last_screen = document.lines(document.line_count() - 60..document.line_count());
        let read_last_screen = started.elapsed() - opened;

        // then
        println!("{} lines; open {opened:?}; last 60 lines {read_last_screen:?}", document.line_count());
        assert_eq!(last_screen.len(), 60);
    }
}
