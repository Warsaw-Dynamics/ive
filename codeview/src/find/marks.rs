//! The matches a body draws: their bytes in the text it searched, and which one is current.

use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use crate::scrollbar::Tick;

/// How a match is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Match,
    Current,
}

/// The matches of the find box, by bytes of the searched text (sorted, not overlapping), and the line each starts
/// on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindMarks {
    pub ranges: Arc<[Range<usize>]>,
    pub lines: Arc<[usize]>,
    pub current: Option<usize>,
}

impl FindMarks {
    /// The parts of the matches within `bytes`, in order.
    pub fn on_bytes(&self, bytes: Range<usize>) -> Vec<(Range<usize>, Mark)> {
        // matches do not overlap, so their ends are sorted as their starts are
        let first = self.ranges.partition_point(|range| range.end <= bytes.start);
        let mark = |index| if Some(index) == self.current { Mark::Current } else { Mark::Match };
        (first..self.ranges.len())
            .take_while(|&index| self.ranges[index].start < bytes.end)
            .map(|index| (self.ranges[index].start.max(bytes.start)..self.ranges[index].end.min(bytes.end), mark(index)))
            .collect()
    }

    /// A scrollbar tick for each match, placed by `at`: from the line a match starts on to its share of the content.
    pub fn ticks(&self, at: impl Fn(usize) -> f32) -> Rc<[Tick]> {
        self.lines.iter().enumerate().map(|(index, line)| Tick { at: at(*line), current: Some(index) == self.current }).collect()
    }

    /// The current match's bytes.
    pub fn current_range(&self) -> Option<Range<usize>> {
        self.current.and_then(|current| self.ranges.get(current).cloned())
    }
}

/// The character columns of `text`, starting at byte `start` of the searched text, that `marks` cover; past its end
/// (the line break) is the column after it.
pub(crate) fn text_columns(text: &str, start: usize, marks: &FindMarks) -> Vec<(Range<usize>, Mark)> {
    let starts: Vec<usize> = text.char_indices().map(|(at, _)| at).collect();
    let column = |byte: usize| match byte - start {
        at if at > text.len() => starts.len() + 1,
        at => starts.partition_point(|first| *first < at),
    };
    let found = marks.on_bytes(start..start + text.len() + 1).into_iter().map(|(bytes, mark)| (column(bytes.start)..column(bytes.end), mark));
    found.filter(|(columns, _)| !columns.is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_mark_the_columns_of_a_long_text_quickly() {
        // given
        let text = "é ".repeat(100_000);
        let ranges: Vec<Range<usize>> = (0..100_000).map(|at| 10 + 3 * at..10 + 3 * at + 2).collect();
        let marks = FindMarks { lines: vec![0; ranges.len()].into(), ranges: ranges.into(), current: Some(1) };

        // when
        let started = std::time::Instant::now();
        let columns = text_columns(&text, 10, &marks);

        // then
        assert_eq!((columns.len(), columns[1].clone()), (100_000, (2..3, Mark::Current)));
        assert!(started.elapsed() < std::time::Duration::from_millis(100), "{:?}", started.elapsed());
    }

    #[test]
    fn should_clip_the_matches_to_the_bytes_and_mark_the_current_one() {
        // given
        let marks = FindMarks { ranges: Arc::from([0..2, 4..9, 12..13]), lines: Arc::from([0, 0, 1]), current: Some(1) };

        // when
        let on_line = marks.on_bytes(5..12);

        // then
        assert_eq!(on_line, [(5..9, Mark::Current)]);
        assert_eq!(marks.on_bytes(0..5), [(0..2, Mark::Match), (4..5, Mark::Current)]);
    }
}
