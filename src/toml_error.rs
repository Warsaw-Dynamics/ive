//! Where in a TOML file a parse error points, for messages that name the line.

use std::ops::Range;

/// The 1-based line that `span` (a TOML error's byte range in `text`) starts on; 1 when it points nowhere.
pub fn line_of(span: Option<Range<usize>>, text: &str) -> usize {
    let at = span.map_or(0, |span| span.start).min(text.len());
    text.as_bytes()[..at].iter().filter(|byte| **byte == b'\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_count_lines_before_span() {
        // given
        let text = "a = 1\nb = 2\nc = ?\n";

        // when
        let line = line_of(Some(16..17), text);

        // then
        assert_eq!(line, 3);
    }

    #[test]
    fn should_say_first_line_without_span() {
        // given
        let text = "a";

        // when
        let line = line_of(None, text);

        // then
        assert_eq!(line, 1);
    }
}
