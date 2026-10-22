//! An edited diff's rows before the host diffs the text again: the host's diff of the base with the edits made since
//! applied line by line. A line an edit touched shows as removed (when it was a context line) and added again as it
//! is now; the host's next diff replaces the guess. Free of gpui.

use std::ops::Range;

use super::data::{DiffLine, Hunk, LineKind};
use crate::panel::{Edit, LineSource};

/// Lines `base` of the base text became lines `now` of the edited one; both count from 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LineEdit {
    pub base: Range<usize>,
    pub now: Range<usize>,
}

/// The lines `edits` (sorted, non-overlapping, in `base`'s bytes) replace: each edit's whole lines, two edits on one
/// line joined.
pub(crate) fn line_edits(base: &dyn LineSource, edits: &[Edit]) -> Vec<LineEdit> {
    let mut joined: Vec<LineEdit> = Vec::new();
    let mut growth = 0isize;
    for edit in edits {
        let (first, last) = (base.line_of(edit.bytes.start), base.line_of(edit.bytes.end));
        // the start of the first line, the text, and the rest of the last line
        let lines = 1 + memchr::memchr_iter(b'\n', &edit.text).count();
        let start = first.saturating_add_signed(growth);
        let line_edit = LineEdit { base: first..last + 1, now: start..start + lines };
        growth += lines as isize - (last + 1 - first) as isize;
        match joined.last_mut() {
            Some(previous) if previous.base.end > first => {
                previous.base.end = line_edit.base.end;
                previous.now.end = line_edit.now.end;
            }
            _ => joined.push(line_edit),
        }
    }
    joined
}

/// The base line that line `now` of the edited text still is; `None` for a line an edit made.
pub(crate) fn base_line(edits: &[LineEdit], now: usize) -> Option<usize> {
    let mut growth = 0isize;
    for edit in edits {
        if now < edit.now.start {
            break;
        }
        if now < edit.now.end {
            return None;
        }
        growth = edit.now.end as isize - edit.base.end as isize;
    }
    now.checked_add_signed(-growth)
}

/// `hunks`, a diff of the base, with `edits` applied; `line(n)` is line `n` of the edited text as a row shows it.
pub(crate) fn patched(hunks: &[Hunk], edits: &[LineEdit], line: &dyn Fn(usize) -> String) -> Vec<Hunk> {
    let mut patch = Patch { edits, line, next: 0, growth: 0, seen: 0, flushed_end: 0, hunks: Vec::new() };
    for hunk in hunks {
        patch.hunks.push(Hunk { header: hunk.header.clone(), lines: Vec::new() });
        for diff_line in &hunk.lines {
            patch.take(diff_line);
        }
        // an edit of the hunk's lines shows in the hunk, not at the start of the next one
        patch.flush_started();
    }
    patch.flush_until(usize::MAX);
    patch.hunks.retain(|hunk| !hunk.lines.is_empty());
    patch.hunks
}

/// A patch being built: the hunks so far and the edits not applied yet.
struct Patch<'a> {
    edits: &'a [LineEdit],
    line: &'a dyn Fn(usize) -> String,
    /// the first edit whose lines are not added yet
    next: usize,
    /// lines the edits added so far made, minus the lines they replaced
    growth: isize,
    /// one past the last base line taken
    seen: usize,
    /// one past the last base line the edits added so far replace
    flushed_end: usize,
    hunks: Vec<Hunk>,
}

impl Patch<'_> {
    fn take(&mut self, line: &DiffLine) {
        let Some(number) = line.new else {
            // a "no newline" note follows the lines of the edits that reached it
            if line.kind == LineKind::NoNewline {
                self.flush_until(self.seen);
            }
            return self.push(line.clone());
        };
        let base = number as usize - 1;
        self.seen = base + 1;
        self.flush_until(base);
        let replaced = base < self.flushed_end || self.edits.get(self.next).is_some_and(|edit| edit.base.contains(&base));
        match (replaced, line.kind) {
            (true, LineKind::Context) => self.push(DiffLine { kind: LineKind::Removed, new: None, ..line.clone() }),
            // an added line an edit replaced is added again as it is now
            (true, _) => {}
            (false, _) => self.push(DiffLine { new: base.checked_add_signed(self.growth).map(|now| now as u32 + 1), ..line.clone() }),
        }
    }

    /// Adds the lines of the edits that end at base line `base` or before it.
    fn flush_until(&mut self, base: usize) {
        while let Some(edit) = self.edits.get(self.next).filter(|edit| edit.base.end <= base) {
            self.add(edit);
        }
    }

    /// Adds the lines of the edits that start before the last base line taken.
    fn flush_started(&mut self) {
        while let Some(edit) = self.edits.get(self.next).filter(|edit| edit.base.start < self.seen) {
            self.add(edit);
        }
    }

    /// Adds the lines `edit` made; the base lines it replaced are not shown as they were any more.
    fn add(&mut self, edit: &LineEdit) {
        for now in edit.now.clone() {
            self.push(DiffLine { kind: LineKind::Added, old: None, new: Some(now as u32 + 1), text: (self.line)(now) });
        }
        self.growth = edit.now.end as isize - edit.base.end as isize;
        self.flushed_end = edit.base.end;
        self.next += 1;
    }

    fn push(&mut self, line: DiffLine) {
        if self.hunks.is_empty() {
            self.hunks.push(Hunk { header: "@@ @@".to_owned(), lines: Vec::new() });
        }
        let last = self.hunks.len() - 1;
        self.hunks[last].lines.push(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::TextSource;

    fn line(kind: LineKind, old: Option<u32>, new: Option<u32>, text: &str) -> DiffLine {
        DiffLine { kind, old, new, text: text.to_owned() }
    }

    fn edit(bytes: Range<u64>, text: &str) -> Edit {
        Edit { bytes, text: text.as_bytes().to_vec() }
    }

    /// The lines of `hunks` as (sign, old, new, text).
    fn rows(hunks: &[Hunk]) -> Vec<(char, Option<u32>, Option<u32>, String)> {
        let sign = |kind| match kind {
            LineKind::Added => '+',
            LineKind::Removed => '-',
            LineKind::Context => ' ',
            LineKind::NoNewline => '\\',
        };
        hunks.iter().flat_map(|hunk| &hunk.lines).map(|line| (sign(line.kind), line.old, line.new, line.text.clone())).collect()
    }

    #[test]
    fn should_find_the_lines_each_edit_replaces_and_join_edits_on_one_line() {
        // given
        let base = TextSource::new("one\ntwo\nthree\nfour\n");
        let edits = [edit(0..0, "a"), edit(2..3, "\nb"), edit(8..14, ""), edit(19..19, "x\ny\n")];

        // when
        let lines = line_edits(&base, &edits);

        // then
        let expected = [LineEdit { base: 0..1, now: 0..2 }, LineEdit { base: 2..4, now: 3..4 }, LineEdit { base: 4..5, now: 4..7 }];
        assert_eq!(lines, expected);
    }

    #[test]
    fn should_show_an_edited_context_line_as_removed_and_added_and_number_the_lines_after_it_anew() {
        // given
        let hunk = Hunk {
            header: "@@ -1,3 +1,3 @@".to_owned(),
            lines: vec![
                line(LineKind::Context, Some(1), Some(1), "one"),
                line(LineKind::Removed, Some(2), None, "old"),
                line(LineKind::Added, None, Some(2), "new"),
                line(LineKind::Context, Some(3), Some(3), "three"),
            ],
        };
        // line 1 ("one") became two lines
        let edits = [LineEdit { base: 0..1, now: 0..2 }];
        let now = ["o", "ne", "new", "three"];

        // when
        let hunks = patched(&[hunk], &edits, &|n| now[n].to_owned());

        // then
        assert_eq!(
            rows(&hunks),
            [
                ('-', Some(1), None, "one".to_owned()),
                ('-', Some(2), None, "old".to_owned()),
                ('+', None, Some(1), "o".to_owned()),
                ('+', None, Some(2), "ne".to_owned()),
                ('+', None, Some(3), "new".to_owned()),
                (' ', Some(3), Some(4), "three".to_owned()),
            ]
        );
    }

    #[test]
    fn should_add_an_edited_added_line_again_as_it_is_now() {
        // given
        let lines = vec![line(LineKind::Removed, Some(1), None, "old"), line(LineKind::Added, None, Some(1), "new"), line(LineKind::Context, Some(2), Some(2), "end")];
        let hunk = Hunk { header: "@@ -1,2 +1,2 @@".to_owned(), lines };

        // when
        let hunks = patched(&[hunk], &[LineEdit { base: 0..1, now: 0..1 }], &|_| "newer".to_owned());

        // then
        let expected = [('-', Some(1), None, "old".to_owned()), ('+', None, Some(1), "newer".to_owned()), (' ', Some(2), Some(2), "end".to_owned())];
        assert_eq!(rows(&hunks), expected);
    }

    #[test]
    fn should_add_lines_made_past_the_last_hunk_at_its_end() {
        // given
        let hunk = Hunk { header: "@@ -1 +1 @@".to_owned(), lines: vec![line(LineKind::Added, None, Some(1), "a")] };

        // when
        let hunks = patched(&[hunk], &[LineEdit { base: 5..6, now: 5..6 }], &|_| "late".to_owned());

        // then
        assert_eq!(rows(&hunks), [('+', None, Some(1), "a".to_owned()), ('+', None, Some(6), "late".to_owned())]);
    }

    #[test]
    fn should_keep_an_edited_last_context_line_in_its_own_hunk() {
        // given
        let context = |n: u32, text: &str| line(LineKind::Context, Some(n), Some(n), text);
        let first = Hunk { header: "@@ -1,3 +1,3 @@".to_owned(), lines: vec![context(1, "a"), line(LineKind::Added, None, Some(2), "b"), context(3, "c")] };
        let second = Hunk { header: "@@ -20,2 +20,2 @@".to_owned(), lines: vec![context(20, "t"), line(LineKind::Added, None, Some(21), "u")] };
        // "c", the first hunk's last line, is edited
        let edits = [LineEdit { base: 2..3, now: 2..3 }];

        // when
        let hunks = patched(&[first, second], &edits, &|_| "c!".to_owned());

        // then
        let in_first = rows(&hunks[..1]);
        assert_eq!(in_first.last(), Some(&('+', None, Some(3), "c!".to_owned())));
        assert_eq!(rows(&hunks[1..]), [(' ', Some(20), Some(20), "t".to_owned()), ('+', None, Some(21), "u".to_owned())]);
    }

    #[test]
    fn should_tell_the_base_line_of_a_line_no_edit_made() {
        // given
        let edits = [LineEdit { base: 1..2, now: 1..4 }, LineEdit { base: 5..7, now: 7..8 }];

        // when
        let found: Vec<Option<usize>> = [0, 1, 3, 4, 6, 7, 8].into_iter().map(|now| base_line(&edits, now)).collect();

        // then
        assert_eq!(found, [Some(0), None, None, Some(2), Some(4), None, Some(7)]);
    }
}
