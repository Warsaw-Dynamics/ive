//! The tree header's menus besides Open project: the folders every worktree shows, and which projects the tree shows.

use std::collections::BTreeSet;
use std::path::PathBuf;

use gpui::{Context, Div, Stateful, div, prelude::*, px, rgb};

use super::{badge, closing_outside, header_button_press, menu_item, menu_panel, menu_section};
use crate::click_targets::target;
use crate::favicon_badge::favicon_or;
use crate::hub::key_label;
use crate::icons;
use crate::overview::file_list::FileLayout;
use crate::project::ProjectId;
use crate::sidebar::rows::initials;
use crate::script::keymap::{Action, Builtin, Keymap, Scope};
use crate::sidebar::folders::FolderKind;
use crate::theme::Theme;
use crate::tooltip;
use crate::workspace::{Workspace, icon};

/// What the folders menu shows: every worktree's folders, the changed files' layout, the keys of its rows, and the
/// worktrees set apart (`<project> · <worktree>`).
pub struct FoldersMenu {
    pub every: BTreeSet<FolderKind>,
    pub layout: FileLayout,
    pub keys: Vec<(FolderKind, Option<String>)>,
    pub layout_key: Option<String>,
    pub set_apart: Vec<String>,
}

/// Under the whole header: a tick per folder (switches it in every worktree), the Flat | Tree switch of changed
/// files, and Reset while some worktrees are set apart. Each row shows its key on its right; a press outside closes it.
pub(super) fn folders_menu(menu: &FoldersMenu, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let rows = menu.keys.iter().map(|(kind, key)| {
        let (kind, on) = (*kind, menu.every.contains(kind));
        let switch = target(format!("every-folder:{}", kind.name()), cx.listener(move |workspace, _event, _window, cx| workspace.show_folder_everywhere(kind, !on, cx)), cx);
        menu_item(("every-folder", kind as usize), theme)
            .child(tick(on, theme))
            .child(icon(theme, kind.icon()))
            .child(div().flex_1().child(kind.title()))
            .children(key.clone().map(|key| key_hint(key, theme)))
            .on_click(switch)
    });
    let reset = (!menu.set_apart.is_empty()).then(|| {
        let count = menu.set_apart.len();
        let label = format!("Reset {count} {} set apart", if count == 1 { "worktree" } else { "worktrees" });
        let reset = target("reset-folders".to_owned(), cx.listener(|workspace, _event, _window, cx| workspace.reset_folders(cx)), cx);
        let row = menu_item("reset-folders", theme).child(tick(false, theme)).child(label).tooltip(tooltip::text_tooltip(menu.set_apart.join("\n"))).on_click(reset);
        div().child(separator(theme)).child(row)
    });
    let panel = menu_panel("folders-menu", theme)
        .left(theme.sizes.tree.menu_offset.px())
        .right(theme.sizes.tree.menu_offset.px())
        .child(menu_section("In every worktree", theme))
        .children(rows)
        .child(separator(theme))
        .child(layout_row(menu, theme, cx))
        .children(reset);
    closing_outside(panel, cx)
}

/// What the title's picker shows and lists: the open projects, in tree order.
pub struct ProjectsMenu {
    pub projects: Vec<PickedProject>,
}

pub struct PickedProject {
    pub id: ProjectId,
    pub name: String,
    /// in the tree and the hub
    pub shown: bool,
    /// live processes
    pub running: usize,
    /// its favicon, drawn in place of its initials
    pub icon: Option<PathBuf>,
}

impl ProjectsMenu {
    fn all_shown(&self) -> bool {
        self.projects.iter().all(|project| project.shown)
    }
}

/// The header's title, drawn on every frame (the menu is built only while open): `All projects`, the one project
/// shown with its initials, or `2 of 4 projects`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerTitle {
    pub text: String,
    pub badge: Option<String>,
    /// the one project's favicon, drawn in place of its initials
    pub icon: Option<PathBuf>,
}

/// The title for the projects `shown`, by name, of `open` open projects.
pub fn picker_title(shown: &[&str], open: usize) -> PickerTitle {
    match shown {
        _ if shown.len() == open => PickerTitle { text: "All projects".to_owned(), badge: None, icon: None },
        [one] => PickerTitle { text: (*one).to_owned(), badge: Some(initials(one)), icon: None },
        many => PickerTitle { text: format!("{} of {open} projects", many.len()), badge: None, icon: None },
    }
}

/// The header's title: `All projects ▾`, the one project shown with its badge, or `2 of 4 projects ▾`; a click opens
/// the picker.
pub(super) fn picker_button(title: &PickerTitle, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let sizes = &theme.sizes.tree;
    let open = target("project-picker".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.toggle_menu(super::HeaderMenu::Projects, window, cx)), cx);
    header_button_press(div().id("project-picker"), cx)
        .flex()
        .min_w_0()
        .items_center()
        .gap(sizes.menu_item_gap.px())
        .px(sizes.header_button_padding.px())
        .rounded(sizes.header_button_radius.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .text_size(sizes.header_title_size.px())
        .font_weight(sizes.header_title_weight.font_weight())
        .children(title.badge.as_ref().map(|initials| match &title.icon {
            Some(path) => {
                let initials = initials.clone();
                favicon_or(path, theme.sizes.icon.px(), move || badge(&initials, theme.accent, theme.on_accent, theme).into_any_element()).into_any_element()
            }
            None => badge(initials, theme.accent, theme.on_accent, theme).into_any_element(),
        }))
        .child(div().overflow_hidden().text_ellipsis().whitespace_nowrap().child(title.text.clone()))
        .child(icon(theme, icons::CHEVRON_DOWN))
        .tooltip(tooltip::text_tooltip("Show all projects, or one"))
        .on_click(open)
}

/// Under the whole header: All projects with how many are open, then each open project with a tick while it shows;
/// a click shows it or leaves it out (the last one shown stays), and the menu stays open.
pub(super) fn projects_menu(menu: &ProjectsMenu, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let all = target("show-projects:all".to_owned(), cx.listener(|workspace, _event, _window, cx| workspace.show_all_projects(cx)), cx);
    let all = menu_item("show-projects-all", theme)
        .child(tick(menu.all_shown(), theme))
        .child(div().flex_1().child("All projects"))
        .child(key_hint(menu.projects.len().to_string(), theme))
        .on_click(all);
    let rows = menu.projects.iter().enumerate().map(|(index, project)| {
        let id = project.id.clone();
        let show = target(format!("show-projects:{}", id.as_str()), cx.listener(move |workspace, _event, _window, cx| workspace.toggle_project_shown(id.clone(), cx)), cx);
        let running = (project.running > 0).then(|| key_hint(format!("● {} running", project.running), theme));
        menu_item(("show-project", index), theme)
            .child(tick(project.shown, theme))
            .child(match &project.icon {
                Some(path) => {
                    let letters = initials(&project.name);
                    favicon_or(path, theme.sizes.icon.px(), move || badge(&letters, theme.hover, theme.text, theme).into_any_element()).into_any_element()
                }
                None => badge(&initials(&project.name), theme.hover, theme.text, theme).into_any_element(),
            })
            .child(div().flex_1().overflow_hidden().text_ellipsis().child(project.name.clone()))
            .children(running)
            .on_click(show)
    });
    let panel = menu_panel("projects-menu", theme)
        .left(theme.sizes.tree.menu_offset.px())
        .right(theme.sizes.tree.menu_offset.px())
        .child(all)
        .child(separator(theme))
        .children(rows);
    closing_outside(panel, cx)
}

/// `Changed files`, a Flat | Tree switch, the key that flips it.
fn layout_row(menu: &FoldersMenu, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let sizes = &theme.sizes.tree;
    let segment = |layout: FileLayout, label: &'static str, cx: &mut Context<Workspace>| {
        let pick = target(format!("changes-layout:{}", label.to_lowercase()), cx.listener(move |workspace, _event, _window, cx| workspace.set_changes_layout(layout, cx)), cx);
        let segment = div().id(label).px(sizes.menu_item_padding_x.px()).cursor_pointer().child(label).on_click(pick);
        match layout == menu.layout {
            true => segment.bg(rgb(theme.accent)).text_color(rgb(theme.on_accent)),
            false => segment.hover(|style| style.bg(rgb(theme.hover))),
        }
    };
    let switch = div()
        .flex()
        .flex_none()
        .rounded(theme.sizes.settings.segmented_radius.px())
        .border(theme.sizes.settings.segmented_border.px())
        .border_color(rgb(theme.border))
        .overflow_hidden()
        .text_size(sizes.count_text_size.px())
        .child(segment(FileLayout::Flat, "Flat", cx))
        .child(segment(FileLayout::Tree, "Tree", cx));
    div()
        .flex()
        .items_center()
        .gap(sizes.menu_item_gap.px())
        .h(sizes.menu_item_height.px())
        .px(sizes.menu_item_padding_x.px())
        .whitespace_nowrap()
        .child(div().flex_1().child("Changed files"))
        .child(switch)
        .children(menu.layout_key.clone().map(|key| key_hint(key, theme)))
}

/// A row's tick, or the room of one.
fn tick(on: bool, theme: Theme) -> Div {
    div().flex_none().w(theme.sizes.icon.px()).text_center().children(on.then_some("✓"))
}

fn key_hint(key: String, theme: Theme) -> Div {
    div().flex_none().text_size(theme.sizes.tree.count_text_size.px()).text_color(rgb(theme.muted)).child(key)
}

fn separator(theme: Theme) -> Div {
    div().h(theme.sizes.tree.border.px()).my(px(4.)).bg(rgb(theme.border))
}

/// The key each folders menu row runs, as bound in the window scope: the folders' `act::ToggleFolderEverywhere`
/// keys in [`FolderKind::ALL`] order, then `act::ToggleChangesLayout`'s; `None` for a row without a key.
pub fn menu_keys(keymap: &Keymap) -> (Vec<(FolderKind, Option<String>)>, Option<String>) {
    let key = |builtin: Builtin| keymap.keys_for(Scope::Window, &Action::Builtin(builtin)).first().map(|keystroke| key_label::label(keystroke));
    let folders = FolderKind::ALL.iter().map(|kind| (*kind, key(Builtin::ToggleFolderEverywhere(*kind)))).collect();
    (folders, key(Builtin::ToggleChangesLayout))
}

#[cfg(test)]
mod tests {
    use gpui::Keystroke;

    use super::*;
    use crate::script::keymap::{Binding, Trigger};

    fn bound(key: &str, builtin: Builtin) -> Binding {
        Binding { trigger: Trigger::Key(Keystroke::parse(key).unwrap()), scope: Scope::Window, action: Action::Builtin(builtin) }
    }

    #[test]
    fn should_title_the_picker_by_the_projects_shown() {
        // given
        let shown = [vec!["ive", "notes", "third"], vec!["notes"], vec!["ive", "third"]];

        // when
        let titles = shown.map(|names| picker_title(&names, 3));

        // then
        assert_eq!(titles, [
            PickerTitle { text: "All projects".to_owned(), badge: None, icon: None },
            PickerTitle { text: "notes".to_owned(), badge: Some("N".to_owned()), icon: None },
            PickerTitle { text: "2 of 3 projects".to_owned(), badge: None, icon: None },
        ]);
    }

    #[test]
    fn should_label_menu_rows_with_their_bound_keys() {
        // given
        let keymap = Keymap::new(vec![
            bound("alt-shift-c", Builtin::ToggleFolderEverywhere(FolderKind::Commits)),
            bound("alt-shift-g", Builtin::ToggleChangesLayout),
        ])
        .unwrap();

        // when
        let (folders, layout) = menu_keys(&keymap);

        // then
        assert_eq!(folders.iter().find(|(kind, _)| *kind == FolderKind::Commits).unwrap().1.as_deref(), Some("Alt+Shift+C"));
        assert_eq!(folders.iter().find(|(kind, _)| *kind == FolderKind::Files).unwrap().1, None);
        assert_eq!(layout.as_deref(), Some("Alt+Shift+G"));
    }
}
