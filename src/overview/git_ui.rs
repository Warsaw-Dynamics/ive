//! Small widgets shared by the git views, after the mock's `rows`, `seg`, `tag` and `toggle`.

use codeview::host::Command;
use codeview::panel::{Cast, DiffLayout, Panel};
use gpui::{App, ClickEvent, Div, ElementId, Entity, SharedString, Stateful, Window, div, prelude::*, rgb};

use crate::click_targets::target;
use crate::git::types::Mark;
use crate::icons;
use crate::theme::Theme;
use crate::workspace::icon;

/// Whether a row, segment or checkbox shows as picked (filled, ticked).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Picked {
    Yes,
    No,
}

impl From<bool> for Picked {
    fn from(picked: bool) -> Self {
        if picked {
            return Self::Yes;
        }
        Self::No
    }
}

pub use codeview::diff::counted;

/// A clickable list row; a picked one is filled.
pub fn list_row(id: impl Into<ElementId>, picked: Picked, theme: Theme) -> Stateful<Div> {
    let row = div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .gap(theme.sizes.git.row_gap.px())
        .h(theme.sizes.git.row_height.px())
        .px(theme.sizes.git.row_padding_x.px())
        .border_b(theme.sizes.git.row_border.px())
        .border_color(rgb(theme.border))
        .text_size(theme.sizes.git.text_size.px())
        .whitespace_nowrap()
        .cursor_pointer();
    if picked == Picked::Yes {
        return row.bg(rgb(theme.selected));
    }
    row.hover(|style| style.bg(rgb(theme.hover)))
}

pub fn mark_colour(mark: Mark, theme: Theme) -> u32 {
    match mark {
        Mark::Added | Mark::Untracked => theme.mark_added,
        Mark::Deleted | Mark::Conflicted => theme.mark_removed,
        Mark::Modified | Mark::Renamed | Mark::TypeChange => theme.mark_modified,
    }
}

pub fn mark_label(mark: Mark, theme: Theme, cx: &App) -> Div {
    div()
        .flex_none()
        .w(theme.sizes.git.mark_width.px())
        .font_family(crate::theme::mono_font(cx))
        .text_size(theme.sizes.git.mark_text_size.px())
        .font_weight(theme.sizes.git.mark_weight.font_weight())
        .text_color(rgb(mark_colour(mark, theme)))
        .child(mark.letter())
}

pub fn tag(text: impl Into<SharedString>, colour: u32, theme: Theme) -> Div {
    div().flex_none().px(theme.sizes.git.tag_padding_x.px()).rounded(theme.sizes.git.tag_radius.px()).bg(rgb(theme.hover)).text_size(theme.sizes.git.tag_text_size.px()).text_color(rgb(colour)).child(text.into())
}

/// One button of a segmented control.
pub fn segment(id: impl Into<ElementId>, label: impl Into<SharedString>, picked: Picked, theme: Theme) -> Stateful<Div> {
    let button = div()
        .id(id)
        .flex()
        .items_center()
        .gap(theme.sizes.git.segment_gap.px())
        .px(theme.sizes.git.segment_padding_x.px())
        .py(theme.sizes.git.segment_padding_y.px())
        .rounded(theme.sizes.git.segment_radius.px())
        .border(theme.sizes.git.segment_border.px())
        .text_size(theme.sizes.git.segment_text_size.px())
        .cursor_pointer()
        .child(label.into());
    if picked == Picked::Yes {
        return button.bg(rgb(theme.selected)).border_color(rgb(theme.accent));
    }
    button.border_color(rgb(theme.border)).hover(|style| style.bg(rgb(theme.hover)))
}

/// Inline | Side by side, for a diff panel; a click on the other one switches its layout (targets
/// `diff-layout:inline` and `diff-layout:side-by-side`).
pub fn layout_switch(panel: &Entity<Panel>, theme: Theme, cx: &App) -> Div {
    let shown = panel.read(cx).diff_layout(cx);
    let segments = [(DiffLayout::Inline, "inline", "Inline"), (DiffLayout::SideBySide, "side-by-side", "Side by side")].map(|(layout, name, label)| {
        let panel = panel.clone();
        let pick = target(format!("diff-layout:{name}"), move |_event, _window, cx| switch_layout(&panel, layout, cx), cx);
        segment(("diff-layout", layout as usize), label, Picked::from(shown == Some(layout)), theme).on_click(pick)
    });
    div().flex().flex_none().gap_1().children(segments)
}

fn switch_layout(panel: &Entity<Panel>, layout: DiffLayout, cx: &mut App) {
    if panel.read(cx).diff_layout(cx) == Some(layout) {
        return;
    }
    panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::ToggleLayout), cx));
}

/// An outlined button with an icon, like the mock's `toggle`.
pub fn toggle_button(id: impl Into<ElementId>, icon_name: &str, label: impl Into<SharedString>, theme: Theme) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .gap(theme.sizes.git.toggle_gap.px())
        .px(theme.sizes.git.toggle_padding_x.px())
        .py(theme.sizes.git.toggle_padding_y.px())
        .rounded(theme.sizes.git.toggle_radius.px())
        .border(theme.sizes.git.toggle_border.px())
        .border_color(rgb(theme.border))
        .text_size(theme.sizes.git.toggle_text_size.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(icon(theme, icon_name))
        .child(label.into())
}


pub fn muted(text: impl Into<SharedString>, theme: Theme) -> Div {
    div().text_size(theme.sizes.git.text_size.px()).text_color(rgb(theme.muted)).child(text.into())
}

pub fn group_title(title: impl Into<SharedString>, hint: Option<&'static str>, theme: Theme) -> Div {
    div()
        .flex()
        .items_center()
        .gap(theme.sizes.git.group_title_gap.px())
        .pt(theme.sizes.git.group_title_padding_top.px())
        .pb(theme.sizes.git.group_title_padding_bottom.px())
        .text_size(theme.sizes.git.group_title_text_size.px())
        .font_weight(theme.sizes.git.group_title_weight.font_weight())
        .child(title.into())
        .children(hint.map(|hint| div().text_size(theme.sizes.git.group_hint_text_size.px()).font_weight(theme.sizes.git.group_hint_weight.font_weight()).text_color(rgb(theme.muted)).child(hint)))
}

/// A failed load: the message, and Retry (click target `retry`).
pub fn error_with_retry(message: impl Into<SharedString>, on_retry: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static, cx: &App) -> Div {
    let theme = crate::theme::tabs(cx);
    div()
        .flex()
        .items_center()
        .gap(theme.sizes.git.error_gap.px())
        .py(theme.sizes.git.error_padding_y.px())
        .child(div().text_size(theme.sizes.git.text_size.px()).text_color(rgb(theme.error)).child(message.into()))
        .child(toggle_button("git-retry", icons::HISTORY, "Retry", theme).on_click(target("retry".to_owned(), on_retry, cx)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_count_one_and_many() {
        // given
        let counts = [0, 1, 2];

        // when
        let texts = counts.map(|count| counted(count, "file", "files"));

        // then
        assert_eq!(texts, ["0 files", "1 file", "2 files"]);
    }
}
