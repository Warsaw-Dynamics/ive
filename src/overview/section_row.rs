use gpui::{App, Div, SharedString, div, prelude::*, rgb, transparent_black};

use crate::click_targets::target;
use crate::selectable::selectable;
use crate::theme::{self, Theme};
use crate::tooltip;

/// A click handler that a section button can share between frames.
pub use crate::click_targets::Listener;

/// The right end of an overview's section row: where it is (muted, monospace, cut with an ellipsis).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub text: SharedString,
    /// the full path, in a tooltip
    pub full: SharedString,
    /// short tags after the text, e.g. `↑5`
    pub tags: Vec<SharedString>,
    /// underline of the selected section: the checkout's colour, or the accent
    pub colour: u32,
}

/// An overview's section buttons with the location at the right end; `sections` are (title, selected, on click).
pub fn section_row(sections: &[(SharedString, bool, Listener)], location: Location, theme: Theme, cx: &App) -> Div {
    let sizes = &theme.sizes.overview;
    let buttons = sections.iter().enumerate().map(|(index, (title, selected, on_click))| {
        let on_click = on_click.clone();
        div()
            .id(("overview-section", index))
            .flex()
            .items_center()
            .h(sizes.section_height.px())
            .px(sizes.section_padding_x.px())
            .text_size(sizes.section_text_size.px())
            .rounded_t(sizes.section_radius.px())
            .border_b(sizes.section_underline.px())
            .border_color(if *selected { rgb(location.colour).into() } else { transparent_black() })
            .when(*selected, |button| button.font_weight(sizes.section_selected_weight.font_weight()))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(theme.hover)))
            .child(title.clone())
            .on_click(target(format!("section:{title}"), move |event, window, cx| on_click(event, window, cx), cx))
    });
    let tags = location.tags.iter().map(|tag| {
        div()
            .flex_none()
            .ml(sizes.tag_margin_left.px())
            .px(sizes.tag_padding_x.px())
            .rounded(sizes.tag_radius.px())
            .bg(rgb(theme.hover))
            .text_size(sizes.tag_text_size.px())
            .text_color(rgb(theme.muted))
            .child(tag.clone())
    });
    let place = div()
        .id("overview-location")
        .ml_auto()
        .pl(sizes.location_padding_left.px())
        .min_w_0()
        .overflow_hidden()
        .text_ellipsis()
        .whitespace_nowrap()
        .font_family(theme::mono_font(cx))
        .text_size(sizes.location_text_size.px())
        .text_color(rgb(theme.muted))
        .child(selectable(format!("overview-location-{}", location.full), location.text.clone(), cx))
        .tooltip(tooltip::text_tooltip(location.full.clone()));
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(sizes.section_gap.px())
        .border_b(sizes.section_row_border.px())
        .border_color(rgb(theme.border))
        .children(buttons)
        .child(place)
        .children(tags)
}
