use gpui::{Context, Div, Stateful, div, prelude::*, rgb};

use crate::click_targets::target;
use crate::selectable::selectable;
use crate::settings::{AutoUpdate, SettingsStore};
use crate::theme::Theme;
use crate::update::RELEASE_TAG;
use crate::workspace::Workspace;

pub const TITLE: &str = "Updates";

/// The Updates page: the running release, how updating goes, and the switch for it. A build from source shows only
/// that it updates by rebuilding.
pub fn render(status: String, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let page = div()
        .flex()
        .flex_col()
        .gap(theme.sizes.settings.appearance_gap.px())
        .child(div().text_size(theme.sizes.settings.title_text_size.px()).font_weight(theme.sizes.settings.title_weight.font_weight()).child(TITLE))
        .child(row("Version", div().child(selectable("updates-version", RELEASE_TAG.unwrap_or("built from source"), cx)), theme))
        .child(row("Status", div().child(selectable("updates-status", status, cx)), theme));
    let Some(_) = RELEASE_TAG else {
        return page;
    };
    let current = cx.global::<SettingsStore>().current().auto_update;
    page.child(row("Auto-update", render_switch(current, theme, cx), theme))
}

fn row(label: &'static str, value: Div, theme: Theme) -> Div {
    // the value wraps: a status can be a long error
    div().flex().items_center().gap(theme.sizes.settings.appearance_gap.px()).child(div().flex_none().w(theme.sizes.settings.appearance_label_width.px()).child(label)).child(value.flex_1().min_w_0())
}

fn render_switch(current: AutoUpdate, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    div()
        .flex()
        .rounded(theme.sizes.settings.segmented_radius.px())
        .border(theme.sizes.settings.segmented_border.px())
        .border_color(rgb(theme.border))
        .overflow_hidden()
        .children(AutoUpdate::ALL.map(|option| render_option(option, option == current, theme, cx)))
}

fn render_option(option: AutoUpdate, selected: bool, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let button = div()
        .id(option.label())
        .px(theme.sizes.settings.segment_padding_x.px())
        .py(theme.sizes.settings.segment_padding_y.px())
        .cursor_pointer()
        .child(option.label())
        .on_click(target(format!("auto-update:{}", option.label()), cx.listener(move |workspace, _event, _window, cx| workspace.set_auto_update(option, cx)), cx));
    if selected {
        return button.bg(rgb(theme.accent)).text_color(rgb(theme.on_accent));
    }
    button.hover(|style| style.bg(rgb(theme.hover)))
}
