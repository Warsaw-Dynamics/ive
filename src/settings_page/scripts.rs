use std::path::Path;

use gpui::{ClipboardItem, Context, Div, FontWeight, Stateful, div, prelude::*, rgb};

use crate::click_targets::target;
use crate::script::defaults_file::THEME_NAME;
use crate::script::engine::IVE_RN_NAME;
use crate::script::state::ScriptState;
use crate::theme::Theme;
use crate::workspace::Workspace;

const TITLE: &str = "Customising IVE";
const README_URL: &str = "https://github.com/Warsaw-Dynamics/ive#scripting";
const README_LINE: &str = "Full reference: \"Scripting\" and \"Theme\" in IVE's README";

/// The Scripts page: which IVE runs the scripts (version, build time), how to customise it, for a person or an agent
/// to follow, and the files beside `init.rn` to read: IVE's default bindings and theme, and the settings the other
/// pages save.
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
    let log = cx.global::<ScriptState>().log().path().to_path_buf();
    page.child(render_instructions(&dir, &log, theme, cx))
        .child(render_file_link("open-ive-rn", &dir.join(IVE_RN_NAME), format!("IVE's default bindings ({IVE_RN_NAME}, read-only)"), theme, cx))
        .child(render_file_link("open-theme-toml", &dir.join(THEME_NAME), format!("IVE's default theme ({THEME_NAME}, read-only)"), theme, cx))
        .when_some(crate::settings::default_path(), |page, path| page.child(render_settings_link(&path, theme, cx)))
}

/// The instructions as headed paragraphs, each heading (empty for none) in bold before its text.
fn sections(dir: &Path, log: &Path) -> [(&'static str, String); 5] {
    let (dir, log) = (dir.display(), log.display());
    [
        (
            "",
            format!(
                "All your changes go in {dir}. IVE writes two reference files there at every start. They are read-only, and \
                 editing them changes nothing:\n- {IVE_RN_NAME}: every default key binding, mouse binding and menu\n- {THEME_NAME}: \
                 every default colour and size, for both the dark and light looks"
            ),
        ),
        (
            "To change keys, mouse clicks, menus or excluded folders:",
            format!(
                "write init.rn in that folder. IVE runs it after {IVE_RN_NAME}. An entry with the same scope and key replaces the \
                 default one. Look up the entry to override in {IVE_RN_NAME}. IVE reloads it on save."
            ),
        ),
        (
            "To make a theme:",
            format!(
                "create themes/<name>.toml in that folder, with the same sections as {THEME_NAME} ([sizes], [dark], [light]…) but \
                 only the values you change. Everything else stays at the default. Pick it under Settings › Appearance › Theme. \
                 IVE reloads it on save."
            ),
        ),
        (
            "To adjust any theme from a script:",
            "set config.theme.<path> in init.rn. This applies on top of the picked theme, for example \
             config.theme.dark.tree = \"#202124\"; for [dark] tree."
                .to_owned(),
        ),
        (
            "Checking it worked:",
            format!(
                "load errors, with file, line or key, and the bad value, go to {log}. IVE opens the log when a load fails. Agents \
                 connected over MCP can read it with script_status. A failed load keeps the previous settings."
            ),
        ),
    ]
}

/// The instructions as plain text, for the clipboard.
fn instructions(dir: &Path, log: &Path) -> String {
    let paragraphs = sections(dir, log).map(|(heading, body)| if heading.is_empty() { body } else { format!("{heading} {body}") });
    format!("{TITLE}\n\n{}\n\n{README_LINE}: {README_URL}\n", paragraphs.join("\n\n"))
}

/// The instructions with a Copy button (click target `copy-script-instructions`) and the README link.
fn render_instructions(dir: &Path, log: &Path, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let text = instructions(dir, log);
    let copy = div()
        .id("copy-script-instructions")
        .text_color(rgb(theme.accent))
        .cursor_pointer()
        .hover(|style| style.underline())
        .child("Copy instructions")
        .on_click(target("copy-script-instructions".to_owned(), cx.listener(move |_, _event, _window, cx| cx.write_to_clipboard(ClipboardItem::new_string(text.clone()))), cx));
    let heading = div().flex().items_center().justify_between().child(div().font_weight(FontWeight::SEMIBOLD).child(TITLE)).child(copy);
    let paragraphs = sections(dir, log).map(|(heading, body)| render_section(heading, body));
    let readme = div()
        .id("open-ive-readme")
        .text_color(rgb(theme.accent))
        .cursor_pointer()
        .hover(|style| style.underline())
        .child(README_LINE)
        .on_click(target("open-ive-readme".to_owned(), cx.listener(|_, _event, _window, cx| cx.open_url(README_URL)), cx));
    div().flex().flex_col().gap(theme.sizes.settings.page_gap.px()).child(heading).children(paragraphs).child(readme)
}

fn render_section(heading: &'static str, body: String) -> Div {
    if heading.is_empty() {
        return div().child(body);
    }
    div().child(div().font_weight(FontWeight::SEMIBOLD).child(heading)).child(body)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_fill_paths_into_instructions() {
        // given
        let (dir, log) = (Path::new("/p/ive/x"), Path::new("/s/scripts.log"));

        // when
        let text = instructions(dir, log);

        // then
        for needle in ["/p/ive/x", "themes/<name>.toml", "/s/scripts.log", "script_status"] {
            assert!(text.contains(needle), "{needle} missing from {text}");
        }
    }

    #[test]
    fn should_explain_init_override_example() {
        // given
        let (dir, log) = (Path::new("/p"), Path::new("/l"));

        // when
        let text = instructions(dir, log);

        // then
        assert!(text.contains(r##"config.theme.dark.tree = "#202124";"##), "{text}");
    }
}
