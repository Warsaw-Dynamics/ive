//! A piece's text runs: its looks (bold, code, links, colours) and what lies over it (the selection, find's matches),
//! cut where any of them starts or ends, since gpui's runs cannot overlap.

use std::ops::Range;
use std::rc::Rc;

use gpui::{Font, Hsla, StrikethroughStyle, TextRun, px};

use super::document::{Look, Run};
use crate::colors::ColorKey;

/// The fonts and colours a piece is drawn with.
#[derive(Clone)]
pub(crate) struct Paint {
    pub font: Font,
    pub mono: Font,
    pub color: Hsla,
    pub link: Hsla,
    pub code_background: Hsla,
    pub selected: Hsla,
    pub found: Hsla,
    pub current: Hsla,
    /// a code block colour's foreground, from the colour scheme
    pub syntax: Rc<dyn Fn(ColorKey) -> Option<Hsla>>,
}

/// What lies over text, the later over the earlier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Overlay {
    Match,
    Selected,
    Current,
}

/// The runs of a text `len` bytes long: `runs` and `overlays` are by bytes of that text.
pub(crate) fn text_runs(len: usize, runs: &[Run], overlays: &[(Range<usize>, Overlay)], paint: &Paint) -> Vec<TextRun> {
    let ends = runs.iter().map(|run| &run.bytes).chain(overlays.iter().map(|(bytes, _)| bytes)).flat_map(|bytes| [bytes.start, bytes.end]);
    let mut cuts: Vec<usize> = [0, len].into_iter().chain(ends).map(|at| at.min(len)).collect();
    cuts.sort_unstable();
    cuts.dedup();
    cuts.windows(2).map(|cut| run_of(cut[0]..cut[1], runs, overlays, paint)).collect()
}

/// The run of `bytes`, over which no look or overlay starts or ends.
fn run_of(bytes: Range<usize>, runs: &[Run], overlays: &[(Range<usize>, Overlay)], paint: &Paint) -> TextRun {
    let look = runs.iter().find(|run| run.bytes.contains(&bytes.start)).map(|run| &run.look);
    let over = overlays.iter().filter(|(over, _)| over.contains(&bytes.start)).map(|(_, overlay)| *overlay).max();
    let background = match (over, look) {
        (Some(Overlay::Current), _) => Some(paint.current),
        (Some(Overlay::Selected), _) => Some(paint.selected),
        (Some(Overlay::Match), _) => Some(paint.found),
        (None, Some(Look::Code)) => Some(paint.code_background),
        (None, _) => None,
    };
    let strike = matches!(look, Some(Look::Strike)).then(|| StrikethroughStyle { thickness: px(1.), color: None });
    TextRun {
        len: bytes.len(),
        font: font_of(look, paint),
        color: colour_of(look, paint),
        background_color: background,
        underline: None,
        strikethrough: strike,
    }
}

fn font_of(look: Option<&Look>, paint: &Paint) -> Font {
    match look {
        Some(Look::Strong) => paint.font.clone().bold(),
        Some(Look::Emphasis) => paint.font.clone().italic(),
        Some(Look::StrongEmphasis) => paint.font.clone().bold().italic(),
        Some(Look::Code) => paint.mono.clone(),
        Some(Look::Strike | Look::Link(_) | Look::Syntax(_)) | None => paint.font.clone(),
    }
}

fn colour_of(look: Option<&Look>, paint: &Paint) -> Hsla {
    match look {
        Some(Look::Link(_)) => paint.link,
        Some(Look::Syntax(key)) => (paint.syntax)(*key).unwrap_or(paint.color),
        _ => paint.color,
    }
}

#[cfg(test)]
mod tests {
    use gpui::{FontStyle, FontWeight, black, blue, green, red, white, yellow};

    use super::*;

    fn paint() -> Paint {
        Paint {
            font: gpui::font("Sans"),
            mono: gpui::font("Mono"),
            color: white(),
            link: blue(),
            code_background: black(),
            selected: green(),
            found: yellow(),
            current: red(),
            syntax: Rc::new(|_| Some(red())),
        }
    }

    fn lengths(runs: &[TextRun]) -> Vec<usize> {
        runs.iter().map(|run| run.len).collect()
    }

    #[test]
    fn should_split_runs_at_looks_and_overlays() {
        // given
        let looks = [Run { bytes: 2..6, look: Look::Strong }];
        let overlays = [(0..3, Overlay::Selected)];

        // when
        let runs = text_runs(10, &looks, &overlays, &paint());

        // then
        assert_eq!(lengths(&runs), vec![2, 1, 3, 4]);
        let weights: Vec<FontWeight> = runs.iter().map(|run| run.font.weight).collect();
        assert_eq!(weights, vec![FontWeight::NORMAL, FontWeight::BOLD, FontWeight::BOLD, FontWeight::NORMAL]);
        let backgrounds: Vec<Option<Hsla>> = runs.iter().map(|run| run.background_color).collect();
        assert_eq!(backgrounds, vec![Some(green()), Some(green()), None, None]);
    }

    #[test]
    fn should_draw_the_current_match_over_the_selection() {
        // given
        let overlays = [(0..4, Overlay::Selected), (2..3, Overlay::Current), (1..3, Overlay::Match)];

        // when
        let runs = text_runs(4, &[], &overlays, &paint());

        // then
        let backgrounds: Vec<Option<Hsla>> = runs.iter().map(|run| run.background_color).collect();
        assert_eq!(lengths(&runs), vec![1, 1, 1, 1]);
        assert_eq!(backgrounds, vec![Some(green()), Some(green()), Some(red()), Some(green())]);
    }

    #[test]
    fn should_colour_code_links_and_syntax() {
        // given
        let looks = [
            Run { bytes: 0..2, look: Look::Code },
            Run { bytes: 2..4, look: Look::Syntax(ColorKey::TEXT) },
            Run { bytes: 4..6, look: Look::Link("u".into()) },
            Run { bytes: 6..8, look: Look::Strike },
            Run { bytes: 8..10, look: Look::StrongEmphasis },
        ];

        // when
        let runs = text_runs(10, &looks, &[], &paint());

        // then
        assert_eq!(lengths(&runs), vec![2, 2, 2, 2, 2]);
        assert_eq!((runs[0].font.family.as_ref(), runs[0].background_color), ("Mono", Some(black())));
        assert_eq!(runs[1].color, red());
        assert_eq!(runs[2].color, blue());
        assert!(runs[3].strikethrough.is_some());
        assert_eq!((runs[4].font.weight, runs[4].font.style), (FontWeight::BOLD, FontStyle::Italic));
    }

    #[test]
    fn should_cover_an_empty_text_with_no_runs() {
        // given
        let overlays = [(0..0, Overlay::Selected)];

        // when
        let runs = text_runs(0, &[], &overlays, &paint());

        // then
        assert!(runs.is_empty());
    }
}
