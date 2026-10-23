use std::path::Path;

use gpui::{Context, Div, Stateful, div, prelude::*, rgb};

use crate::click_targets::target;
use crate::script::defaults_file::THEME_NAME;
use crate::script::engine::{INIT, IVE_RN_NAME};
use crate::theme::Theme;
use crate::workspace::Workspace;

/// The Scripts page: which IVE runs the scripts (version, build time), where `init.rn` lives, and the files beside it
/// to read: IVE's default bindings and theme, and the settings the other pages save.
pub fn render(theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let page = div()
        .flex()
        .flex_col()
        .gap(theme.sizes.settings.page_gap.px())
        .child(div().text_size(theme.sizes.settings.title_text_size.px()).font_weight(theme.sizes.settings.title_weight.font_weight()).child("Scripts"))
        .child(div().text_color(rgb(theme.muted)).child(crate::build_info::summary()));
    let Some(dir) = crate::script::default_dir() else {
        return page.child(div().text_color(rgb(theme.muted)).child("The OS reports no config folder, so only IVE's default bindings apply."));
    };
    page.child(div().child(format!("Your bindings and functions go in {}", dir.join(INIT).display())))
        .child(render_file_link("open-ive-rn", &dir.join(IVE_RN_NAME), format!("IVE's default bindings ({IVE_RN_NAME}, read-only)"), theme, cx))
        .child(render_file_link("open-theme-toml", &dir.join(THEME_NAME), format!("IVE's default theme ({THEME_NAME}, read-only)"), theme, cx))
        .when_some(crate::settings::default_path(), |page, path| page.child(render_settings_link(&path, theme, cx)))
}

/// Click target `open-settings-toml`; a muted line until the first change on a settings page writes the file.
fn render_settings_link(path: &Path, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    if path.exists() {
        return div().child(render_file_link("open-settings-toml", path, "The settings these pages save (settings.toml)".to_owned(), theme, cx));
    }
    div().text_color(rgb(theme.muted)).child(format!("The settings these pages save go in {}, written on the first change", path.display()))
}

/// Click target `id`: the file at `path` in a tab, or that tab when it is open.
fn render_file_link(id: &'static str, path: &Path, label: String, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let path = path.to_path_buf();
    div()
        .id(id)
        .text_color(rgb(theme.accent))
        .cursor_pointer()
        .hover(|style| style.underline())
        .child(label)
        .on_click(target(id.to_owned(), cx.listener(move |workspace, _event, window, cx| workspace.open_document(&path, window, cx)), cx))
}
