use gpui::{Context, Div, SharedString, Stateful, div, prelude::*, rgb};

use crate::click_targets::target;
use crate::script::state::ScriptState;
use crate::settings::{SettingsStore, ThemeChoice, ThemePreference};
use crate::theme::Theme;
use crate::workspace::Workspace;

/// The Appearance page: dark or light, and the theme they come from (without a config folder, IVE's own).
pub fn render(theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let settings = cx.global::<SettingsStore>().current();
    let (current, user_theme) = (settings.theme, settings.user_theme.clone());
    div()
        .flex()
        .flex_col()
        .gap(theme.sizes.settings.appearance_gap.px())
        .child(div().text_size(theme.sizes.settings.title_text_size.px()).font_weight(theme.sizes.settings.title_weight.font_weight()).child("Appearance"))
        .child(render_row("Mode", render_theme_picker(current, theme, cx), theme))
        .when(crate::script::default_dir().is_some(), |page| page.child(render_row("Theme", render_user_theme_picker(&user_theme, theme, cx), theme)))
}

fn render_row(label: &'static str, picker: Div, theme: Theme) -> Div {
    div()
        .flex()
        .items_center()
        .gap(theme.sizes.settings.appearance_gap.px())
        .child(div().flex_none().w(theme.sizes.settings.appearance_label_width.px()).child(label))
        .child(picker)
}

/// `Default`, then each theme in `themes/`, then the saved one when its file is gone, muted.
fn render_user_theme_picker(current: &ThemeChoice, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let names = cx.global::<ScriptState>().themes().to_vec();
    let missing = match current {
        ThemeChoice::File(name) if !names.contains(name) => Some(current.clone()),
        _ => None,
    };
    let found = std::iter::once(ThemeChoice::Builtin).chain(names.into_iter().map(ThemeChoice::File));
    let buttons: Vec<_> = found.map(|option| render_user_theme(option.clone(), option == *current, theme, cx)).collect();
    let missing = missing.map(|option| render_user_theme(option, true, theme, cx).text_color(rgb(theme.muted)).bg(rgb(theme.hover)));
    div()
        .flex()
        .flex_wrap()
        .rounded(theme.sizes.settings.segmented_radius.px())
        .border(theme.sizes.settings.segmented_border.px())
        .border_color(rgb(theme.border))
        .overflow_hidden()
        .children(buttons)
        .children(missing)
}

/// Click target `user-theme:<name>`, `user-theme:Default` for IVE's own.
fn render_user_theme(option: ThemeChoice, selected: bool, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let id = user_theme_target(&option);
    let button = div()
        .id(SharedString::from(id.clone()))
        .px(theme.sizes.settings.segment_padding_x.px())
        .py(theme.sizes.settings.segment_padding_y.px())
        .cursor_pointer()
        .child(option.label().to_owned())
        .on_click(target(id, cx.listener(move |workspace, _event, _window, cx| workspace.set_user_theme(option.clone(), cx)), cx));
    if selected {
        return button.bg(rgb(theme.accent)).text_color(rgb(theme.on_accent));
    }
    button.hover(|style| style.bg(rgb(theme.hover)))
}

/// `user-theme:<name>` for a theme file, `user-theme:Default` for IVE's own; a file named `Default` is
/// `user-theme:Default.toml`, so the two stay apart.
fn user_theme_target(option: &ThemeChoice) -> String {
    match option {
        ThemeChoice::File(name) if name == ThemeChoice::Builtin.label() => format!("user-theme:{name}.toml"),
        option => format!("user-theme:{}", option.label()),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_give_theme_file_named_default_its_own_target() {
        // given
        let (builtin, file) = (ThemeChoice::Builtin, ThemeChoice::File("Default".to_owned()));

        // when
        let targets = (user_theme_target(&builtin), user_theme_target(&file));

        // then
        assert_eq!(targets, ("user-theme:Default".to_owned(), "user-theme:Default.toml".to_owned()));
    }

    #[test]
    fn should_name_theme_file_target_after_file() {
        // given
        let file = ThemeChoice::File("nord".to_owned());

        // when
        let target = user_theme_target(&file);

        // then
        assert_eq!(target, "user-theme:nord");
    }
}
