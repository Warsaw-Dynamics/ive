//! The Third-Party Software page: what IVE includes and under which licence, as IntelliJ's About dialog lists it.
//! A name opens the project's page in the browser; a licence shows its text here.

use std::ops::Range;

use gpui::{AnyElement, Context, Div, Stateful, div, prelude::*, rgb, uniform_list};

use crate::click_targets::target;
use crate::scrollbar::{ListScroll, ScrollState, vertical_scrollbar};
use crate::theme::{self, Theme};
use crate::third_party::{self, Software};
use crate::workspace::Workspace;

pub const TITLE: &str = "Third-Party Software";

/// The page's list position, kept while another page is shown, and the row whose licence is open.
pub struct ThirdPartyView {
    list: ListScroll,
    scroll: ScrollState,
    licence_of: Option<usize>,
}

impl ThirdPartyView {
    pub fn new() -> Self {
        let list = ListScroll::new();
        let scroll = ScrollState::for_list(&list);
        Self { list, scroll, licence_of: None }
    }

    /// Shows the licence of the `row`th software, or the list again for `None`.
    pub fn show_licence(&mut self, row: Option<usize>) {
        self.licence_of = row.filter(|&row| row < third_party::bundled().software().len());
    }
}

pub fn render(view: &ThirdPartyView, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let page = div().flex().flex_col().flex_1().min_h_0().gap(theme.sizes.settings.table_gap.px());
    match view.licence_of {
        Some(row) => page.child(render_licence(&third_party::bundled().software()[row], theme, cx)),
        None => page.child(div().text_size(theme.sizes.settings.title_text_size.px()).font_weight(theme.sizes.settings.title_weight.font_weight()).child(TITLE)).child(render_list(view, theme, cx)),
    }
}

/// Only the visible rows are built: the list has some 700 entries.
fn render_list(view: &ThirdPartyView, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let count = third_party::bundled().software().len();
    let rows = uniform_list("third-party-rows", count, cx.processor(|_workspace: &mut Workspace, range, _window, cx| render_rows(range, cx)))
        .track_scroll(&view.list.list_handle())
        .size_full();
    let header = div()
        .flex()
        .gap(theme.sizes.settings.table_gap.px())
        .pr(theme.sizes.settings.table_padding_right.px())
        .text_size(theme.sizes.settings.text_size.px())
        .font_weight(theme.sizes.settings.table_header_weight.font_weight())
        .text_color(rgb(theme.muted))
        .child(div().flex_1().child("Software"))
        .child(div().flex_none().w(theme.sizes.settings.licence_width.px()).child("License"));
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap(theme.sizes.settings.list_gap.px())
        .child(div().text_color(rgb(theme.muted)).child(format!("IVE includes these {count} libraries, fonts and icon sets; click a licence to read it.")))
        .child(header)
        .child(div().relative().flex_1().min_h_0().child(rows).child(vertical_scrollbar(&view.scroll, &theme::view_style(&theme))))
}

fn render_rows(range: Range<usize>, cx: &mut Context<Workspace>) -> Vec<AnyElement> {
    let theme = crate::theme::tabs(cx);
    let software = third_party::bundled().software();
    range.map(|row| render_row(row, &software[row], theme, cx).into_any_element()).collect()
}

fn render_row(row: usize, software: &'static Software, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let name = link(("third-party-name", row), software.name.clone(), theme).on_click(move |_event, _window, cx| cx.open_url(&software.url));
    let licence = link(("third-party-licence", row), software.license.clone(), theme).on_click(target(
        format!("third-party-licence:{}", software.name),
        cx.listener(move |workspace, _event, _window, cx| workspace.show_licence(Some(row), cx)),
        cx,
    ));
    let named = div()
        .flex()
        .flex_1()
        .min_w_0()
        .gap(theme.sizes.settings.list_gap.px())
        .overflow_hidden()
        .whitespace_nowrap()
        .child(name)
        .child(div().text_color(rgb(theme.muted)).child(software.version.clone()));
    div().w_full().flex().gap(theme.sizes.settings.table_gap.px()).py(theme.sizes.settings.row_padding_y.px()).pr(theme.sizes.settings.table_padding_right.px()).child(named).child(div().flex_none().w(theme.sizes.settings.licence_width.px()).overflow_hidden().whitespace_nowrap().child(licence))
}

fn link(id: (&'static str, usize), label: String, theme: Theme) -> Stateful<Div> {
    div().id(id).text_color(rgb(theme.accent)).cursor_pointer().hover(|style| style.underline()).child(label)
}

fn render_licence(software: &'static Software, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let back = div()
        .id("third-party-back")
        .text_color(rgb(theme.accent))
        .cursor_pointer()
        .hover(|style| style.underline())
        .child(format!("← {TITLE}"))
        .on_click(target("third-party-back".to_owned(), cx.listener(|workspace, _event, _window, cx| workspace.show_licence(None, cx)), cx));
    let texts = third_party::bundled().texts_of(software).into_iter().map(|text| {
        div().pb(theme.sizes.settings.licence_text_padding_bottom.px()).mb(theme.sizes.settings.licence_text_margin_bottom.px()).border_b(theme.sizes.settings.licence_text_border.px()).border_color(rgb(theme.border)).font_family(theme::mono_font(cx)).text_size(theme.sizes.settings.text_size.px()).child(text.to_owned())
    });
    let heading = format!("{} {}", software.name, software.version);
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap(theme.sizes.settings.list_gap.px())
        .child(back)
        .child(div().text_size(theme.sizes.settings.title_text_size.px()).font_weight(theme.sizes.settings.title_weight.font_weight()).child(heading.trim_end().to_owned()))
        .child(div().text_color(rgb(theme.muted)).child(format!("{} · {}", software.license, software.url)))
        .child(div().id("third-party-licence-text").flex_1().min_h_0().overflow_y_scroll().pr(theme.sizes.settings.table_padding_right.px()).children(texts))
}
