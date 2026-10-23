//! The workspace's side of user scripts: applying what the script thread reports (showing the script log when
//! something failed), reloading when the scripts folder changes, and running the user's window-wide bindings for
//! keys no view took.

use std::path::{Path, PathBuf};

use codeview::host::Command;
use codeview::panel::Cast;
use codeview::text_editor::{Motion, Selecting};
use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use gpui::{App, ClipboardItem, Context, KeyDownEvent, Task, Window};

use super::Workspace;
use crate::editor::Tab;
use crate::script::excluded::Excluded;
use crate::script::open_folders::OpenFolders;
use crate::script::keymap::Place;
use crate::script::keymap::{Action, Builtin, MouseTrigger, Scope};
use crate::script::state::{ScriptState, bound_action, bound_mouse, invoke};
use crate::tabs::TabId;
use crate::theme_config::ThemeSpec;
use crate::theme_config::user::THEMES_DIR;
use crate::script::temporary_tabs::TemporaryTabs;
use crate::selectable::TextSelection;
use crate::script::{ScriptEvent, Scripts};
use crate::text_input;
use crate::theme::Fonts;
use crate::watch::{OpenPaths, Watched};

/// The scripts folder and its themes folder as `OpenPaths` reports them (canonical), and the task applying script
/// events.
pub(super) struct ScriptWatch {
    dirs: Vec<PathBuf>,
    _events: Task<()>,
}

impl ScriptWatch {
    /// Makes the scripts state global, watches `dir` and its `themes` (creating them: `dir` is IVE's config folder)
    /// and applies the script thread's events as they come.
    pub(super) fn start(scripts: Scripts, dir: Option<PathBuf>, paths: &mut OpenPaths, window: &mut Window, cx: &mut Context<Workspace>) -> Self {
        cx.set_global(ScriptState::new(scripts.commands, scripts.log));
        let dirs: Vec<PathBuf> = dir.iter().flat_map(|dir| [dir.clone(), dir.join(THEMES_DIR)]).filter_map(|dir| watched_dir(&dir)).collect();
        if !dirs.is_empty() {
            let owner = paths.new_owner();
            paths.set(owner, dirs.iter().cloned().map(Watched::Folder));
        }
        Self { dirs, _events: apply_events(scripts.events, window, cx) }
    }
}

fn watched_dir(dir: &Path) -> Option<PathBuf> {
    if let Err(error) = std::fs::create_dir_all(dir) {
        eprintln!("ive: cannot create {}: {error}; scripts there reload only when IVE restarts", dir.display());
        return None;
    }
    dir.canonicalize().ok()
}

fn apply_events(mut events: UnboundedReceiver<ScriptEvent>, window: &mut Window, cx: &mut Context<Workspace>) -> Task<()> {
    cx.spawn_in(window, async move |workspace, cx| {
        while let Some(event) = events.next().await {
            let applied = workspace.update_in(cx, |workspace, window, cx| workspace.apply_script_event(event, window, cx));
            if applied.is_err() {
                return;
            }
        }
    })
}

/// Draws with `theme` from now on, in the current appearance.
fn apply_theme(theme: ThemeSpec, cx: &mut App) {
    if cx.try_global::<ThemeSpec>() == Some(&theme) {
        return;
    }
    cx.set_global(theme);
    crate::theme::reapply(cx);
}

/// Uses `excluded` from now on; whether it changed, so the tree's rows, whose states come from it, must be rebuilt.
fn apply_excluded(excluded: Excluded, cx: &mut App) -> bool {
    let changed = is_new(cx.try_global::<Excluded>(), &excluded);
    cx.set_global(excluded);
    changed
}

/// Runs the user's binding for `event` on selected text in `window` when that text has the keyboard (Copy, Select
/// All); whether it took the key. A detached tab's window has no workspace bindings, so it asks this.
pub(crate) fn run_text_key(event: &KeyDownEvent, window: &Window, cx: &mut App) -> bool {
    let has_keys = cx.global::<TextSelection>().focus.is_focused(window);
    let action = has_keys.then(|| bound_action(event, Place::Text, cx)).flatten();
    match action {
        Some(Action::Builtin(builtin)) if takes_in_text(builtin) => {
            run_text_builtin(builtin, cx);
            true
        }
        _ => false,
    }
}

/// Whether selected read-only text takes `builtin`: the editor's own actions; window-wide ones (Alt+F12, the hub)
/// still act on the window.
fn takes_in_text(builtin: Builtin) -> bool {
    builtin.supports(Scope::Editor) && !builtin.supports(Scope::Window)
}

/// Of the editor's actions, selected read-only text runs Copy and Select All; the editing ones do nothing.
fn run_text_builtin(builtin: Builtin, cx: &mut App) {
    match builtin {
        Builtin::CopyTo(_) => {
            let text = cx.global::<TextSelection>().selection.copy_text().map(str::to_owned);
            if let Some(text) = text {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
            }
        }
        Builtin::SelectAll => cx.global_mut::<TextSelection>().selection.select_all(),
        _ => {}
    }
}

fn is_new(old: Option<&Excluded>, new: &Excluded) -> bool {
    old != Some(new)
}

/// Uses `fonts`, keeping the current family in place of one the system lacks; whether one was missing, which the
/// script log then says.
fn apply_fonts(fonts: Fonts, cx: &mut App) -> bool {
    let installed = cx.text_system().all_font_names();
    let (fonts, missing) = fonts.installed_or(cx.global::<Fonts>(), &installed);
    for family in &missing {
        cx.global::<ScriptState>().log().append(&format!("font {family:?} is not installed; IVE keeps the font it had"));
    }
    let style = cx.global::<crate::theme::ViewStyle>().clone().with_mono_font(fonts.mono.clone());
    cx.set_global(fonts);
    cx.set_global(style);
    !missing.is_empty()
}

impl Workspace {
    fn apply_script_event(&mut self, event: ScriptEvent, window: &mut Window, cx: &mut Context<Self>) {
        if let ScriptEvent::Tested { request, answer } = event {
            return self.menu_answered(request, answer, window, cx);
        }
        let fonts_missing = match &event {
            ScriptEvent::Loaded { fonts, theme, excluded, temporary_tabs, menus, quit_countdown, open_folders, .. } => {
                // its items may name functions of the load before
                self.close_context_menu(window, cx);
                apply_theme(theme.as_ref().clone(), cx);
                let open_changed = cx.try_global::<OpenFolders>() != Some(open_folders);
                cx.set_global(open_folders.clone());
                if apply_excluded(excluded.clone(), cx) || open_changed {
                    self.tree_changed(cx);
                }
                self.apply_temporary_tabs(temporary_tabs.clone(), cx);
                cx.set_global(menus.as_ref().clone());
                cx.set_global(*quit_countdown);
                apply_fonts(fonts.clone(), cx)
            }
            _ => false,
        };
        let failed = fonts_missing || matches!(event, ScriptEvent::LoadFailed { .. } | ScriptEvent::RunFailed);
        cx.global_mut::<ScriptState>().apply(event);
        if failed {
            self.show_script_log(window, cx);
        }
        window.refresh();
    }

    /// Opens tabs temporary as `temporary_tabs` says from now on; a shown temporary tab it no longer makes
    /// temporary stays.
    fn apply_temporary_tabs(&mut self, temporary_tabs: TemporaryTabs, cx: &mut Context<Self>) {
        let shown = self.tabs.active_id().filter(|id| self.tabs.is_temporary(*id));
        let kind = shown.and_then(|id| self.tabs.get(id)).and_then(Tab::temporary_kind);
        if kind.is_some_and(|kind| !temporary_tabs.opens_temporary(kind)) {
            self.keep_shown_tab(cx);
        }
        cx.set_global(temporary_tabs);
    }

    /// Shows the script log in its file tab, opening it when needed, at its newest lines.
    fn show_script_log(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let path = cx.global::<ScriptState>().log().path().to_path_buf();
        let shown = match self.open_file(&path, window, cx) {
            Ok(shown) => shown,
            // the log could not be written either; its writer said why on stderr
            Err(reason) => return eprintln!("ive: cannot show the script log: {reason}"),
        };
        self.cast_to_document_tab(shown, Cast::Command(Command::Move(Motion::DocumentEnd, Selecting::Move)), cx);
    }

    /// A folder changed on disk; when it is the scripts or themes folder, the script thread reloads if a script or
    /// theme changed.
    pub(super) fn scripts_folder_changed(&self, dir: &Path, cx: &Context<Self>) {
        if self.scripts.dirs.iter().any(|watched| watched == dir) {
            cx.global::<ScriptState>().reload();
        }
    }

    /// Runs the window-wide binding for a key nothing focused took; a `SendString` goes to the shown terminal. A
    /// text field keeps the keys that type text and the ones it handles: typing a name never triggers a binding.
    /// A shown file takes the editor scope's bindings first; of them, it runs Copy and Select All.
    pub(super) fn run_window_binding(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if text_input::holds_focus(window) && !text_input::passes_to_window(&event.keystroke) {
            return;
        }
        let place = match self.tabs.active() {
            _ if cx.global::<TextSelection>().focus.is_focused(window) => Place::Text,
            _ if super::sidebar_has_keys(window, cx) => Place::Tree,
            Some(Tab::Document(_)) => Place::Editor,
            _ => Place::Elsewhere,
        };
        let Some(action) = bound_action(event, place, cx) else {
            return;
        };
        match action {
            Action::Builtin(builtin) if place == Place::Text && takes_in_text(builtin) => run_text_builtin(builtin, cx),
            // a shown tab that is not a terminal takes no text
            Action::Send(text) => drop(self.send_to_active(&text)),
            Action::Invoke { generation, function } => invoke(generation, function, None, cx),
            // the keymap allows Copy Absolute Path in the tree only, where it has its own key
            Action::Builtin(builtin) if builtin != Builtin::CopyFullPath && super::path_actions::is_path_action(builtin) => {
                self.run_bound_path_action(builtin, place == Place::Tree, window, cx);
            }
            Action::Builtin(builtin) => self.run_window_builtin(builtin, window, cx),
            // the keymap allows it only in the tab scope
            Action::ShowTreeMenu(name) if place == Place::Tree => self.open_tree_menu_on_selection(&name, window, cx),
            Action::ShowTabMenu(_) | Action::ShowTreeMenu(_) | Action::Disable => {}
        }
        cx.stop_propagation();
        window.refresh();
    }

    /// Runs the tab strip's binding for a mouse gesture on tab `tab`.
    pub(crate) fn run_tab_mouse(&mut self, tab: TabId, trigger: MouseTrigger, window: &mut Window, cx: &mut Context<Self>) {
        match bound_mouse(Scope::Tab, &trigger, cx) {
            Some(Action::Builtin(Builtin::ActivateTab)) => self.activate_tab(tab, window, cx),
            Some(Action::Builtin(builtin)) if builtin.is_tab_command() => self.run_tab_command(builtin, tab, window, cx),
            Some(Action::ShowTabMenu(name)) => self.open_tab_menu(tab, &name, window, cx),
            Some(Action::Invoke { generation, function }) => invoke(generation, function, Some(self.tab_context(tab)), cx),
            // the keymap allows no other action in the tab scope
            _ => {}
        }
    }

    /// MCP's `tab_mouse`: `triggers` on tab `id`, in order, as a real mouse gives them; the rest are dropped once a
    /// trigger closed the tab.
    pub(super) fn tab_mouse_by_request(&mut self, id: TabId, triggers: Vec<MouseTrigger>, window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        if self.tabs.is_detached(id) {
            return Err(format!("tab {} is in a window of its own, not in the tab row", id.value()));
        }
        for trigger in triggers {
            if self.tabs.get(id).is_none() {
                break;
            }
            self.run_tab_mouse(id, trigger, window, cx);
        }
        window.refresh();
        Ok(())
    }

    pub(super) fn run_window_builtin(&mut self, builtin: Builtin, window: &mut Window, cx: &mut Context<Self>) {
        match builtin {
            Builtin::OpenSettings => self.open_settings(window, cx),
            Builtin::OpenHub => {
                self.open_hub(None, window, cx);
            }
            Builtin::OpenTerminal => {
                let checkout = self.current_checkout();
                self.open_terminal_in(checkout, window, cx);
            }
            Builtin::ToggleChangesLayout => self.toggle_changes_layout(cx),
            Builtin::ToggleFolderEverywhere(kind) => self.toggle_folder_everywhere(kind, cx),
            Builtin::OpenTerminalIn => {
                self.open_terminal_picker(window, cx);
            }
            Builtin::CopyFullPath => self.copy_selected_paths(cx),
            Builtin::OpenFindInFiles => {
                let scope = self.hub_scope.clone();
                self.open_hub(Some(crate::hub::model::HubState { content: true, scope, ..Default::default() }), window, cx);
            }
            tab_command if tab_command.is_tab_command() => {
                if let Some(shown) = self.tabs.active_id() {
                    self.run_tab_command(tab_command, shown, window, cx);
                }
            }
            Builtin::SelectAll => {
                if let Some(shown) = self.tabs.active_id() {
                    self.cast_to_document_tab(shown, Cast::Command(Command::SelectAll), cx);
                }
            }
            Builtin::CopyTo(target) => {
                if let Some(shown) = self.tabs.active_id() {
                    self.cast_to_document_tab(shown, Cast::Command(Command::Copy(target)), cx);
                }
            }
            // the keymap allows no other action in the window scope; a shown file ignores the editing ones
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn excluded(names: serde_json::Value) -> Excluded {
        crate::script::excluded::parse(&serde_json::from_value(names).unwrap()).unwrap()
    }

    /// Selectable text in a panel that hands its keys to [`run_text_key`], as a detached tab's window does.
    struct Detached;

    impl gpui::Render for Detached {
        fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
            use gpui::prelude::*;
            let panel = gpui::div().size_full().on_key_down(|event, window, cx| {
                run_text_key(event, window, cx);
            });
            gpui::div().size_full().child(panel.child(crate::selectable::selectable("t", "hello world", cx)))
        }
    }

    #[gpui::test]
    fn should_copy_selected_text_with_the_users_copy_key(cx: &mut gpui::TestAppContext) {
        // given
        let dir = tempfile::tempdir().unwrap();
        cx.update(|cx| {
            cx.set_global(crate::theme::Theme::of(crate::theme::Appearance::Dark));
            let selection = TextSelection::new(cx);
            cx.set_global(selection);
            let copy = crate::script::keymap::Binding {
                trigger: crate::script::keymap::Trigger::Key(gpui::Keystroke::parse("ctrl-c").unwrap()),
                scope: Scope::Editor,
                action: Action::Builtin(Builtin::CopyTo(crate::script::keymap::ClipboardTarget::Clipboard)),
            };
            let mut state = ScriptState::new(futures::channel::mpsc::unbounded().0, crate::script::log::ScriptLog::new(dir.path().join("scripts.log"), 64 * 1024));
            state.apply(loaded(crate::script::keymap::Keymap::new(vec![copy]).unwrap()));
            cx.set_global(state);
        });
        let (_, cx) = cx.add_window_view(|_window, _cx| Detached);
        cx.run_until_parked();
        cx.simulate_event(gpui::MouseDownEvent { button: gpui::MouseButton::Left, position: gpui::point(gpui::px(5.), gpui::px(5.)), modifiers: gpui::Modifiers::default(), click_count: 2, first_mouse: false });
        cx.run_until_parked();

        // when
        cx.simulate_keystrokes("ctrl-c");

        // then
        assert_eq!(cx.read_from_clipboard().and_then(|item| item.text()).as_deref(), Some("hello"));
    }

    fn loaded(keymap: crate::script::keymap::Keymap) -> crate::script::ScriptEvent {
        crate::script::ScriptEvent::Loaded {
            keymap,
            fonts: Fonts::default(),
            theme: Box::new(crate::theme_config::defaults()),
            excluded: Default::default(),
            temporary_tabs: TemporaryTabs { enabled: true, kinds: Vec::new(), marker: crate::script::temporary_tabs::TabMarker::Italic },
            menus: Default::default(),
            quit_countdown: crate::script::quit_countdown::parse(&serde_json::from_value(serde_json::json!(10)).unwrap()).unwrap(),
            open_folders: Default::default(),
            has_init: true,
            themes: Vec::new(),
        }
    }

    #[test]
    fn should_leave_window_actions_to_the_window_while_text_has_the_keys() {
        // given
        let builtins = [Builtin::CopyTo(crate::script::keymap::ClipboardTarget::Clipboard), Builtin::SelectAll, Builtin::Undo, Builtin::OpenTerminal, Builtin::OpenHub, Builtin::OpenSettings];

        // when
        let taken = builtins.map(takes_in_text);

        // then
        assert_eq!(taken, [true, true, true, false, false, false]);
    }

    #[test]
    fn should_rebuild_tree_when_exclusions_change() {
        // given
        let old = excluded(serde_json::json!(["target"]));
        let new = excluded(serde_json::json!(["target", "dist"]));

        // when
        let rebuild = is_new(Some(&old), &new);

        // then
        assert!(rebuild);
    }

    #[test]
    fn should_not_rebuild_tree_for_the_same_exclusions() {
        // given
        let old = excluded(serde_json::json!(["target"]));
        let new = excluded(serde_json::json!(["target"]));

        // when
        let rebuild = is_new(Some(&old), &new);

        // then
        assert!(!rebuild);
    }

    #[test]
    fn should_rebuild_tree_on_first_load() {
        // given
        let new = excluded(serde_json::json!([]));

        // when
        let rebuild = is_new(None, &new);

        // then
        assert!(rebuild);
    }
}
