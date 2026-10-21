//! Long lists show 10 rows, then "Show 10 more · N of M" and "Show all".

use gpui::{App, ClickEvent, Div, ElementId, Stateful, Window, div, prelude::*, rgb};

use crate::theme::Theme;

pub const PAGE_SIZE: usize = 10;

/// How many rows of a long list are shown: 10, then 10 more per "Show 10 more", or all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paging {
    shown: usize,
}

impl Default for Paging {
    fn default() -> Self {
        Self { shown: PAGE_SIZE }
    }
}

impl Paging {
    pub fn more(&mut self) {
        self.shown = self.shown.saturating_add(PAGE_SIZE);
    }

    pub fn all(&mut self) {
        self.shown = usize::MAX;
    }

    /// Shows pages until row `index` (from 0) is shown, as if "Show 10 more" was pressed until it was.
    pub fn reveal(&mut self, index: usize) {
        let pages = index / PAGE_SIZE + 1;
        self.shown = self.shown.max(pages.saturating_mul(PAGE_SIZE));
    }

    /// Rows to show of a list of `total`.
    pub fn visible(&self, total: usize) -> usize {
        self.shown.min(total)
    }
}

/// "Show 10 more · 10 of 37"; `None` once all `total` rows are shown.
pub fn more_label(total: usize, shown: usize) -> Option<String> {
    (shown < total).then(|| format!("Show {} more · {shown} of {total}", PAGE_SIZE.min(total - shown)))
}

/// The footer of a list showing `shown` of `total` rows: "Show 10 more · 10 of 37" and "Show all"; `None` when
/// nothing is hidden.
pub fn more_row(
    id: impl Into<ElementId>,
    total: usize,
    shown: usize,
    theme: Theme,
    on_more: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_all: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Option<Stateful<Div>> {
    let label = more_label(total, shown)?;
    let link = |id: &'static str, text: String| {
        div().id(id).text_color(rgb(theme.accent)).cursor_pointer().hover(|style| style.underline()).child(text)
    };
    Some(
        div()
            .id(id)
            .flex()
            .items_center()
            .gap(theme.sizes.controls.paging_gap.px())
            .text_size(theme.sizes.controls.paging_text_size.px())
            .whitespace_nowrap()
            .child(link("more", label).on_click(on_more))
            .child(link("all", "Show all".to_owned()).on_click(on_all)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_show_ten_then_ten_more_then_all() {
        // given
        let mut paging = Paging::default();
        let first = paging.visible(37);

        // when
        paging.more();
        let second = paging.visible(37);
        paging.all();

        // then
        assert_eq!((first, second, paging.visible(37), paging.visible(3)), (10, 20, 37, 3));
    }

    #[test]
    fn should_page_to_a_revealed_row_without_hiding_rows_already_shown() {
        // given
        let mut paging = Paging::default();
        paging.more();

        // when
        paging.reveal(9);
        let kept = paging.visible(50);
        paging.reveal(30);

        // then
        assert_eq!((kept, paging.visible(50)), (20, 40));
    }

    #[test]
    fn should_offer_only_the_rows_left() {
        // given
        let lists = [(37, 10), (37, 30), (10, 10)];

        // when
        let labels = lists.map(|(total, shown)| more_label(total, shown));

        // then
        assert_eq!(labels, [Some("Show 10 more · 10 of 37".to_owned()), Some("Show 7 more · 30 of 37".to_owned()), None]);
    }
}
