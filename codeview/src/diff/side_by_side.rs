//! Old and new side by side: a diff's lines paired into rows, old on the left, new on the right. Free of gpui.

use super::data::{DiffLine, Hunk, LineKind};

/// One of the two panels of a side-by-side diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Side {
    Old,
    #[default]
    New,
}

/// Line `line` of hunk `hunk` of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiffLineRef {
    pub hunk: usize,
    pub line: usize,
}

/// A row of a side-by-side diff: the line each side shows, `None` for a filler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SideRow {
    pub old: Option<DiffLineRef>,
    pub new: Option<DiffLineRef>,
}

impl SideRow {
    pub fn on(&self, side: Side) -> Option<DiffLineRef> {
        match side {
            Side::Old => self.old,
            Side::New => self.new,
        }
    }
}

/// The rows of `hunks`: a context line on both sides; within a hunk each run of removals paired line by line with
/// the run of additions right after it, the shorter run padded with fillers.
pub fn side_rows(hunks: &[Hunk]) -> Vec<SideRow> {
    hunks.iter().enumerate().flat_map(|(index, hunk)| hunk_rows(index, &hunk.lines)).collect()
}

/// The rows of hunk `hunk`, whose lines are `lines`; see [`side_rows`].
pub fn hunk_rows(hunk: usize, lines: &[DiffLine]) -> Vec<SideRow> {
    let mut pairing = Pairing::default();
    let mut last = LineKind::Context;
    for (line, content) in lines.iter().enumerate() {
        let at = DiffLineRef { hunk, line };
        // "\ No newline at end of file" goes with the line before it
        let kind = if content.kind == LineKind::NoNewline { last } else { content.kind };
        match kind {
            LineKind::Removed => pairing.removed(at),
            LineKind::Added => pairing.added.push(at),
            LineKind::Context | LineKind::NoNewline => pairing.both(at),
        }
        last = kind;
    }
    pairing.finish()
}

/// The runs of removals and additions not yet paired, and the rows made so far.
#[derive(Default)]
struct Pairing {
    removed: Vec<DiffLineRef>,
    added: Vec<DiffLineRef>,
    rows: Vec<SideRow>,
}

impl Pairing {
    fn removed(&mut self, at: DiffLineRef) {
        // a removal after additions starts a new run
        if self.added.last().is_some() {
            self.pair();
        }
        self.removed.push(at);
    }

    fn both(&mut self, at: DiffLineRef) {
        self.pair();
        self.rows.push(SideRow { old: Some(at), new: Some(at) });
    }

    fn pair(&mut self) {
        let count = self.removed.len().max(self.added.len());
        let rows = (0..count).map(|index| SideRow { old: self.removed.get(index).copied(), new: self.added.get(index).copied() });
        self.rows.extend(rows);
        self.removed.clear();
        self.added.clear();
    }

    fn finish(mut self) -> Vec<SideRow> {
        self.pair();
        self.rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hunk(kinds: &[LineKind]) -> Hunk {
        let lines = kinds.iter().map(|kind| DiffLine { kind: *kind, old: None, new: None, text: String::new() }).collect();
        Hunk { header: "@@ -1 +1 @@".to_owned(), lines }
    }

    fn at(hunk: usize, line: usize) -> Option<DiffLineRef> {
        Some(DiffLineRef { hunk, line })
    }

    fn row(old: Option<DiffLineRef>, new: Option<DiffLineRef>) -> SideRow {
        SideRow { old, new }
    }

    #[test]
    fn should_pair_removals_with_following_additions() {
        // given
        let hunks = [hunk(&[LineKind::Context, LineKind::Removed, LineKind::Removed, LineKind::Added, LineKind::Context])];

        // when
        let rows = side_rows(&hunks);

        // then
        assert_eq!(rows, [row(at(0, 0), at(0, 0)), row(at(0, 1), at(0, 3)), row(at(0, 2), None), row(at(0, 4), at(0, 4))]);
    }

    #[test]
    fn should_pad_extra_additions_with_fillers() {
        // given
        let hunks = [hunk(&[LineKind::Removed, LineKind::Added, LineKind::Added]), hunk(&[LineKind::Added, LineKind::Context])];

        // when
        let rows = side_rows(&hunks);

        // then
        assert_eq!(rows, [row(at(0, 0), at(0, 1)), row(None, at(0, 2)), row(None, at(1, 0)), row(at(1, 1), at(1, 1))]);
    }

    #[test]
    fn should_start_a_new_run_at_a_removal_after_additions() {
        // given
        let hunks = [hunk(&[LineKind::Added, LineKind::Removed, LineKind::Added])];

        // when
        let rows = side_rows(&hunks);

        // then
        assert_eq!(rows, [row(None, at(0, 0)), row(at(0, 1), at(0, 2))]);
    }

    #[test]
    fn should_keep_a_missing_newline_marker_on_the_side_of_the_line_before_it() {
        // given
        let hunks = [hunk(&[LineKind::Removed, LineKind::NoNewline, LineKind::Added, LineKind::Context, LineKind::NoNewline])];

        // when
        let rows = side_rows(&hunks);

        // then
        assert_eq!(rows, [row(at(0, 0), at(0, 2)), row(at(0, 1), None), row(at(0, 3), at(0, 3)), row(at(0, 4), at(0, 4))]);
    }
}
