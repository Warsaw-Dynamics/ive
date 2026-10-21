use std::ops::Range;

use gpui::{
    Context, Div, HighlightStyle, InteractiveText, Stateful, StyledText, UnderlineStyle, WeakEntity, div, prelude::*, rgb,
};

use crate::colors::{ColorKey, Page, ResolvedScheme};
use crate::document::DisplayLine;
use crate::syntax::{Cancel, Capture, Highlight, Language, PARSE_BUDGET, paint, parse, spans};
use crate::theme::{self, Theme};
use crate::workspace::Workspace;

/// A page's demo text and its highlights, built once when the page is shown.
#[derive(Debug, PartialEq)]
pub struct Preview {
    text: String,
    highlights: Vec<Highlight>,
}

const LANGUAGE_DEFAULTS_DEMO: &str = include_str!("../../assets/previews/language_defaults.txt");

impl Preview {
    /// The demo text of the language previewing `page`; Language Defaults (and a page without one) show a tagged
    /// text instead.
    pub fn of(page: Page) -> Self {
        match previewed_language(page) {
            Some((language, demo)) => Self { highlights: highlight(demo, language), text: demo.to_owned() },
            None => {
                let (text, highlights) = parse_tagged(LANGUAGE_DEFAULTS_DEMO);
                Self { text, highlights }
            }
        }
    }

    /// As [`Self::of`], with a language's demo text plain: its highlighting may first compile the language's
    /// queries, which can take long, so it runs on the background executor (`of`) while this shows.
    pub fn plain(page: Page) -> Self {
        match previewed_language(page) {
            Some((_, demo)) => Self { highlights: Vec::new(), text: demo.to_owned() },
            None => Self::of(page),
        }
    }

    pub fn highlights(&self) -> &[Highlight] {
        &self.highlights
    }

    pub fn lines(&self) -> Vec<DisplayLine> {
        let mut start = 0;
        self.text
            .split('\n')
            .map(|line| {
                let display = DisplayLine::of(line.as_bytes(), start);
                start += line.len() + 1;
                display
            })
            .collect()
    }
}

fn previewed_language(page: Page) -> Option<(Language, &'static str)> {
    Language::all().filter(|language| language.page() == page).find_map(|language| Some((language, language.spec().preview?)))
}

fn highlight(demo: &str, language: Language) -> Vec<Highlight> {
    match parse(demo.as_bytes(), language, &Cancel::default(), PARSE_BUDGET) {
        Ok(syntax) => spans(&syntax, demo.as_bytes(), 0..demo.len()),
        Err(error) => {
            eprintln!("ive: preview of {language:?} shown plain: {error}");
            Vec::new()
        }
    }
}

/// `<KEY_ID>text</KEY_ID>` marks `text` with that key; tags nest, the inner one wins. `<` not starting a tag of a
/// known key id stays text.
pub fn parse_tagged(tagged: &str) -> (String, Vec<Highlight>) {
    let mut text = String::with_capacity(tagged.len());
    let mut open: Vec<(ColorKey, usize)> = Vec::new();
    let mut captures = Vec::new();
    let mut rest = tagged;
    while let Some(at) = rest.find('<') {
        text.push_str(&rest[..at]);
        rest = &rest[at..];
        let consumed = match tag(rest) {
            Some((Tag::Open(key), length)) => {
                open.push((key, text.len()));
                length
            }
            Some((Tag::Close(key), length)) if open.last().is_some_and(|(last, _)| *last == key) => {
                let start = open.pop().map_or(text.len(), |(_, start)| start);
                // deeper tags get higher pattern numbers, so they win over enclosing ones of equal length
                captures.push(Capture { bytes: start..text.len(), depth: 0, pattern: open.len(), key });
                length
            }
            _ => {
                text.push('<');
                1
            }
        };
        rest = &rest[consumed..];
    }
    text.push_str(rest);
    let length = text.len();
    (text, paint(captures, 0..length))
}

enum Tag {
    Open(ColorKey),
    Close(ColorKey),
}

/// The tag at the start of `rest` and its length, when it names a known key.
fn tag(rest: &str) -> Option<(Tag, usize)> {
    let end = rest.find('>')?;
    let inside = &rest[1..end];
    let (closing, id) = match inside.strip_prefix('/') {
        Some(id) => (true, id),
        None => (false, inside),
    };
    let key = ColorKey::by_id(id)?;
    let tag = if closing { Tag::Close(key) } else { Tag::Open(key) };
    Some((tag, end + 1))
}

/// The page's preview: clicking a coloured span selects its key; the selected key's spans are underlined.
pub fn render(
    preview: &Preview,
    scheme: &ResolvedScheme,
    selected: ColorKey,
    theme: Theme,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    let workspace = cx.entity().downgrade();
    let plain = scheme.get(ColorKey::TEXT).foreground.unwrap_or(theme.text);
    let lines: Vec<Div> = preview
        .lines()
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let text = preview_line(index, line, preview.highlights(), scheme, selected, theme, workspace.clone());
            div().h(theme.sizes.settings.preview_line_height.px()).child(text)
        })
        .collect();
    div()
        .id("color-preview")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .overflow_scroll()
        .p(theme.sizes.settings.preview_padding.px())
        .rounded(theme.sizes.settings.preview_radius.px())
        .border(theme.sizes.settings.preview_border.px())
        .border_color(rgb(theme.border))
        .font_family(theme::mono_font(cx))
        .text_size(theme.sizes.settings.text_size.px())
        .whitespace_nowrap()
        .text_color(rgb(plain))
        .children(lines)
}

fn preview_line(
    index: usize,
    line: &DisplayLine,
    highlights: &[Highlight],
    scheme: &ResolvedScheme,
    selected: ColorKey,
    theme: Theme,
    workspace: WeakEntity<Workspace>,
) -> InteractiveText {
    let placed = line.place(highlights);
    let offered = clickable(&placed);
    let ranges: Vec<Range<usize>> = offered.iter().map(|(range, _)| range.clone()).collect();
    let keys: Vec<ColorKey> = offered.iter().map(|(_, key)| *key).collect();
    let styles = placed.into_iter().map(|(range, key)| (range, preview_style(key, selected, scheme, theme)));
    let text = StyledText::new(line.text.clone()).with_highlights(styles);
    InteractiveText::new(("preview-line", index), text).on_click(ranges, move |clicked, _window, cx| {
        let key = keys[clicked];
        workspace.update(cx, |workspace, cx| workspace.select_color_key(key, cx)).ok();
    })
}

/// The spans a click may select: keys of a page the navigation lists. Plain text (`TEXT`) has no page of its own, and
/// selecting it would show a page the navigation cannot reach.
fn clickable(placed: &[(Range<usize>, ColorKey)]) -> Vec<(Range<usize>, ColorKey)> {
    placed.iter().filter(|(_, key)| Page::listed().contains(&key.page())).cloned().collect()
}

fn preview_style(key: ColorKey, selected: ColorKey, scheme: &ResolvedScheme, theme: Theme) -> HighlightStyle {
    let style = scheme.get(key).highlight_style();
    if key == selected {
        let underline = UnderlineStyle { thickness: theme.sizes.settings.preview_underline.px(), color: Some(rgb(theme.accent).into()), wavy: false };
        return HighlightStyle { underline: Some(underline), ..style };
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn should_offer_only_keys_of_listed_pages_for_clicking() {
        // given
        let placed = vec![(0..3, ColorKey::TEXT), (4..6, ColorKey::by_id("DEFAULT_KEYWORD").unwrap())];

        // when
        let offered = clickable(&placed);

        // then
        assert_eq!(offered, vec![(4..6, ColorKey::by_id("DEFAULT_KEYWORD").unwrap())]);
    }

    fn key(id: &str) -> ColorKey {
        ColorKey::by_id(id).unwrap()
    }

    #[test]
    fn should_strip_tags_and_let_nested_tag_win() {
        // given
        let tagged = "<DEFAULT_STRING>'a<DEFAULT_VALID_STRING_ESCAPE>\\n</DEFAULT_VALID_STRING_ESCAPE>'</DEFAULT_STRING>";

        // when
        let (text, highlights) = parse_tagged(tagged);

        // then
        assert_eq!(text, "'a\\n'");
        assert_eq!(
            highlights,
            [
                Highlight { bytes: 0..2, key: key("DEFAULT_STRING") },
                Highlight { bytes: 2..4, key: key("DEFAULT_VALID_STRING_ESCAPE") },
                Highlight { bytes: 4..5, key: key("DEFAULT_STRING") },
            ]
        );
    }

    #[test]
    fn should_keep_angle_brackets_that_are_not_key_tags() {
        // given
        let tagged = "a < b <code>x</code>";

        // when
        let (text, highlights) = parse_tagged(tagged);

        // then
        assert_eq!((text.as_str(), highlights.len()), ("a < b <code>x</code>", 0));
    }

    #[test]
    fn should_show_every_language_default_key_in_its_preview() {
        // given
        let preview = Preview::of(Page::LANGUAGE_DEFAULTS);

        // when
        let shown: HashSet<ColorKey> = preview.highlights().iter().map(|highlight| highlight.key).collect();

        // then
        let missing: Vec<&str> =
            ColorKey::on_page(Page::LANGUAGE_DEFAULTS).into_iter().filter(|key| !shown.contains(key)).map(ColorKey::id).collect();
        assert!(missing.is_empty(), "not in the preview: {missing:?}");
    }

    #[test]
    fn should_show_the_demo_text_plain_until_highlighted() {
        // given
        let page = Page::listed().iter().copied().find(|page| *page != Page::LANGUAGE_DEFAULTS).unwrap();

        // when
        let (plain, highlighted) = (Preview::plain(page), Preview::of(page));

        // then
        assert_eq!((plain.text == highlighted.text, plain.highlights.len()), (true, 0));
        assert_eq!(Preview::plain(Page::LANGUAGE_DEFAULTS), Preview::of(Page::LANGUAGE_DEFAULTS), "tagged, no parse");
    }

    #[test]
    fn should_highlight_every_language_preview() {
        // given
        let pages = Page::listed();

        // when
        let empty: Vec<&str> =
            pages.iter().filter(|page| Preview::of(**page).highlights().is_empty()).map(|page| page.title()).collect();

        // then
        assert!(empty.is_empty(), "previews without highlights: {empty:?}");
    }
}
