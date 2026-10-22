//! What the hub's preview shows of the selected row: a file's lines, a change's diff, a process's last lines, a
//! run configuration's command, a worktree's summary.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Lines a file preview shows.
pub const PREVIEW_LINES: usize = 200;
/// Lines shown above a `:line` preview's line, so it is in view in a pane of any height.
pub const PREVIEW_LEAD: usize = 8;
/// Bytes of them at most: a preview never holds a big file.
pub const PREVIEW_BYTES: usize = 256 * 1024;
/// A process's last lines.
pub const PROCESS_PREVIEW_LINES: usize = 30;
pub const BINARY: &str = "Binary file";
/// Bytes looked at for a NUL, as git does to tell binary files.
const SNIFF_BYTES: usize = 8 * 1024;
/// How far into a file a `:line` preview reads to reach its line.
const SCAN_BYTES: u64 = 32 * 1024 * 1024;

/// The first line shown (from 1) and up to [`PREVIEW_LINES`] lines: from the top, or from [`PREVIEW_LEAD`] lines above
/// `line`. Reads the disk: run it on the background executor.
pub fn read_text(path: &Path, line: Option<u32>) -> Result<(usize, Vec<String>), String> {
    let failed = |error: std::io::Error| format!("cannot read {}: {error}", path.display());
    let mut reader = BufReader::new(File::open(path).map_err(failed)?.take(SCAN_BYTES));
    if reader.fill_buf().map_err(failed)?.iter().take(SNIFF_BYTES).any(|byte| *byte == 0) {
        return Err(BINARY.to_owned());
    }
    let wanted = line.map_or(1, |line| line as usize);
    let first = wanted.saturating_sub(PREVIEW_LEAD).max(1);
    let mut lines = Vec::new();
    let mut kept = 0;
    let mut raw = Vec::new();
    for number in 1.. {
        raw.clear();
        if lines.len() == PREVIEW_LINES || kept >= PREVIEW_BYTES || reader.read_until(b'\n', &mut raw).map_err(failed)? == 0 {
            break;
        }
        if number < first {
            continue;
        }
        let text = String::from_utf8_lossy(&raw);
        let mut text = text.trim_end_matches(['\n', '\r']).to_owned();
        truncate_to(&mut text, PREVIEW_BYTES - kept);
        kept += text.len();
        lines.push(text);
    }
    Ok((first, lines))
}

/// Cuts `text` to at most `bytes`, on a char boundary.
fn truncate_to(text: &mut String, bytes: usize) {
    if text.len() <= bytes {
        return;
    }
    let mut end = bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn file_of(text: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.txt");
        fs::write(&path, text).unwrap();
        (dir, path)
    }

    fn numbered(count: usize) -> Vec<u8> {
        (1..=count).map(|line| format!("line {line}\n")).collect::<String>().into_bytes()
    }

    #[test]
    fn should_read_the_first_lines() {
        // given
        let (_dir, path) = file_of(&numbered(300));

        // when
        let (first, lines) = read_text(&path, None).unwrap();

        // then
        assert_eq!((first, lines.len(), lines[0].as_str()), (1, PREVIEW_LINES, "line 1"));
    }

    #[test]
    fn should_center_on_the_requested_line() {
        // given
        let (_dir, path) = file_of(&numbered(300));

        // when
        let (first, lines) = read_text(&path, Some(250)).unwrap();

        // then
        assert!(first <= 250 && 250 < first + lines.len(), "first {first}, {} lines", lines.len());
        assert_eq!(lines[250 - first], "line 250");
    }

    #[test]
    fn should_show_the_requested_line_among_the_first_lines() {
        // given
        let (_dir, path) = file_of(&numbered(300));

        // when
        let (first, _) = read_text(&path, Some(152)).unwrap();

        // then
        assert_eq!(first, 152 - PREVIEW_LEAD);
    }

    #[test]
    fn should_say_binary_for_a_binary_file() {
        // given
        let (_dir, path) = file_of(b"PK\x03\x04\0\0binary");

        // when
        let result = read_text(&path, None);

        // then
        assert_eq!(result, Err(BINARY.to_owned()));
    }

    #[test]
    fn should_stop_at_the_byte_limit() {
        // given
        let (_dir, path) = file_of(&vec![b'x'; 1024 * 1024]);

        // when
        let (_, lines) = read_text(&path, None).unwrap();

        // then
        assert_eq!(lines.len(), 1);
        assert!(lines[0].len() <= PREVIEW_BYTES);
    }

    #[test]
    fn should_name_a_file_that_cannot_be_read() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("gone.txt");

        // when
        let error = read_text(&missing, None).unwrap_err();

        // then
        assert!(error.contains("gone.txt"), "{error}");
    }
}
