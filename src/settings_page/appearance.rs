use gpui::{Context, Div, Stateful, div, prelude::*, rgb};

use crate::click_targets::target;
use crate::settings::{SettingsStore, ThemePreference};
use crate::theme::Theme;
use crate::workspace::Workspace;

/// The Appearance page: the theme picker.
pub fn render(theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let current = cx.global::<SettingsStore>().current().theme;
    div()
        .flex()
        .flex_col()
        .gap(theme.sizes.settings.appearance_gap.px())
        .child(div().text_size(theme.sizes.settings.title_text_size.px()).font_weight(theme.sizes.settings.title_weight.font_weight()).child("Appearance"))
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.sizes.settings.appearance_gap.px())
                .child(div().w(theme.sizes.settings.appearance_label_width.px()).child("Theme"))
                .child(render_theme_picker(current, theme, cx)),
        )
}

fn render_theme_picker(current: ThemePreference, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    div()
        .flex()
        .rounded(theme.sizes.settings.segmented_radius.px())
        .border(theme.sizes.settings.segmented_border.px())
        .border_color(rgb(theme.border))
        .overflow_hidden()
        .children(ThemePreference::ALL.map(|option| render_option(option, option == current, theme, cx)))
}

fn render_option(option: ThemePreference, selected: bool, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let button = div()
        .id(option.label())
        .px(theme.sizes.settings.segment_padding_x.px())
        .py(theme.sizes.settings.segment_padding_y.px())
        .cursor_pointer()
        .child(option.label())
        .on_click(target(format!("theme:{}", option.label()), cx.listener(move |workspace, _event, window, cx| workspace.set_theme(option, window, cx)), cx));
    if selected {
        return button.bg(rgb(theme.accent)).text_color(rgb(theme.on_accent));
    }
    button.hover(|style| style.bg(rgb(theme.hover)))
}
