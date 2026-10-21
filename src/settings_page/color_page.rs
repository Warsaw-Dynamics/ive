use gpui::{Context, Div, Stateful, div, prelude::*, px, rgb};

use super::{KeyState, SettingsView, preview};
use crate::click_targets::target;
use crate::colors::{ColorKey, Page, ResolvedScheme};
use crate::theme::{Appearance, Theme};
use crate::workspace::Workspace;

/// The colours are set in `init.rn`; the Color Scheme pages only show them.
const NOTE: &str = "Colours are set in init.rn under config.theme.dark.syntax and config.theme.light.syntax; the \
                    defaults are in theme.toml next to it.";

/// A Color Scheme page: the page's keys, the selected key's attributes, and a preview, all read-only.
pub fn render(view: &SettingsView, page: Page, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let state = KeyState::of(view.selected(), theme.appearance, cx);
    let scheme = cx.global::<ResolvedScheme>().clone();
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap(theme.sizes.settings.colors_gap.px())
        .child(header(page, theme))
        .child(div().text_size(theme.sizes.settings.text_size.px()).text_color(rgb(theme.muted)).child(NOTE))
        .child(
            div()
                .flex()
                .gap(theme.sizes.settings.keys_gap.px())
                .h(theme.sizes.settings.keys_height.px())
                .child(key_list(page, view.selected(), theme, cx))
                .child(attributes_panel(&state, theme, cx)),
        )
        .child(preview::render(view.preview(), &scheme, view.selected(), theme, cx))
}

fn header(page: Page, theme: Theme) -> Div {
    let scheme = match theme.appearance {
        Appearance::Dark => "Dark scheme",
        Appearance::Light => "Light scheme",
    };
    div()
        .flex()
        .justify_between()
        .child(div().font_weight(theme.sizes.settings.title_weight.font_weight()).child(format!("Editor › Color Scheme › {}", page.title())))
        .child(div().text_size(theme.sizes.settings.text_size.px()).text_color(rgb(theme.muted)).child(scheme))
}

fn key_list(page: Page, selected: ColorKey, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let rows: Vec<_> = ColorKey::on_page(page).into_iter().map(|key| key_row(key, key == selected, theme, cx)).collect();
    div()
        .id("color-keys")
        .flex()
        .flex_col()
        .flex_1()
        .overflow_y_scroll()
        .py(theme.sizes.settings.keys_padding_y.px())
        .rounded(theme.sizes.settings.keys_radius.px())
        .border(theme.sizes.settings.keys_border.px())
        .border_color(rgb(theme.border))
        .children(rows)
}

fn key_row(key: ColorKey, selected: bool, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let row = div()
        .id(key.id())
        .px(theme.sizes.settings.key_padding_x.px())
        .py(theme.sizes.settings.key_padding_y.px())
        .text_size(theme.sizes.settings.text_size.px())
        .cursor_pointer()
        .child(key.name())
        .on_click(target(
            format!("color-key:{}", key.id()),
            cx.listener(move |workspace, _event, _window, cx| workspace.select_color_key(key, cx)),
            cx,
        ));
    if selected {
        return row.bg(rgb(theme.selected));
    }
    row.hover(|style| style.bg(rgb(theme.hover)))
}

fn attributes_panel(state: &KeyState, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let shown = state.shown;
    div()
        .flex()
        .flex_col()
        .gap(theme.sizes.settings.colors_gap.px())
        .w(theme.sizes.settings.panel_width.px())
        .child(div().flex().gap(theme.sizes.settings.keys_gap.px()).child(checkbox("Bold", shown.bold, theme)).child(checkbox("Italic", shown.italic, theme)))
        .child(color_row("Foreground", shown.foreground, theme))
        .child(color_row("Background", shown.background, theme))
        .children(state.key.parent().map(|parent| inherit_row(state.inherits(), parent, theme, cx)))
}

fn color_row(label: &'static str, color: Option<u32>, theme: Theme) -> Div {
    let value = color.map_or_else(|| "none".to_owned(), |color| format!("{color:06X}"));
    div()
        .flex()
        .items_center()
        .gap(theme.sizes.settings.color_row_gap.px())
        .child(div().w(theme.sizes.settings.color_label_width.px()).text_size(theme.sizes.settings.text_size.px()).child(label))
        .child(div().w(theme.sizes.settings.color_label_width.px()).text_size(theme.sizes.settings.text_size.px()).text_color(rgb(theme.muted)).child(value))
        .child(swatch(color, theme))
}

fn swatch(color: Option<u32>, theme: Theme) -> Div {
    let swatch = div().size(theme.sizes.settings.swatch_size.px()).rounded(theme.sizes.settings.swatch_radius.px()).border(theme.sizes.settings.swatch_border.px()).border_color(rgb(theme.border));
    match color {
        Some(color) => swatch.bg(rgb(color)),
        None => swatch,
    }
}

fn inherit_row(inherits: bool, parent: ColorKey, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let link = div()
        .id("inherit-parent")
        .pl(px(theme.sizes.settings.box_size.0 + theme.sizes.settings.checkbox_gap.0))
        .text_size(theme.sizes.settings.text_size.px())
        .text_color(rgb(theme.accent))
        .cursor_pointer()
        .child(format!("{} ({})", parent.name(), parent.page().title()))
        .on_click(target(
            "inherit-parent".to_owned(),
            cx.listener(move |workspace, _event, _window, cx| workspace.select_color_key(parent, cx)),
            cx,
        ));
    div().flex().flex_col().gap(theme.sizes.settings.inherit_gap.px()).child(checkbox("Inherit values from:", inherits, theme)).child(link)
}

/// A ticked or unticked box with its label, which shows a value and takes no clicks.
fn checkbox(label: &'static str, checked: bool, theme: Theme) -> Div {
    let mark = div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(theme.sizes.settings.box_size.px())
        .rounded(theme.sizes.settings.box_radius.px())
        .border(theme.sizes.settings.box_border.px())
        .border_color(rgb(theme.border))
        .text_size(theme.sizes.settings.small_text_size.px())
        .when(checked, |mark| mark.bg(rgb(theme.accent)).text_color(rgb(theme.on_accent)).child("✓"));
    div().flex().items_center().gap(theme.sizes.settings.checkbox_gap.px()).text_size(theme.sizes.settings.text_size.px()).child(mark).child(label)
}
