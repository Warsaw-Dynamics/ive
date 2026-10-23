//! Where the hub's file preview, and the tab Enter opens from it, scroll to.

use std::ops::Range;

use codeview::panel::Cast;

/// Where a file's preview, or the tab Enter opens, scrolls to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reveal {
    Top,
    /// a line from 1, marked
    Line(usize),
    /// a line from 1, marked, with `bytes` of it selected and scrolled into view
    Range { line: usize, bytes: Range<usize> },
}

impl Reveal {
    pub fn of_line(line: Option<usize>) -> Self {
        line.map_or(Self::Top, Self::Line)
    }
}

/// What the file's view is told to show `reveal`; nothing for the top, where it opens anyway.
pub fn reveal_cast(reveal: &Reveal) -> Option<Cast> {
    match reveal {
        Reveal::Top => None,
        Reveal::Line(line) => Some(Cast::RevealLine(*line)),
        Reveal::Range { line, bytes } => Some(Cast::RevealRange { line: *line, bytes: bytes.clone() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_reveal_a_hit_as_its_range() {
        // given
        let reveals = [Reveal::Range { line: 4, bytes: 2..5 }, Reveal::Line(7), Reveal::Top];

        // when
        let casts = reveals.map(|reveal| reveal_cast(&reveal));

        // then
        assert!(matches!(&casts[0], Some(Cast::RevealRange { line: 4, bytes }) if *bytes == (2..5)));
        assert!(matches!(casts[1], Some(Cast::RevealLine(7))));
        assert!(casts[2].is_none());
    }
}
