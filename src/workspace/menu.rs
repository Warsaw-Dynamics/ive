//! The menus of tabs and tree rows: what `config.tab_menus` and `config.tree_menus` declare, opened by
//! `act::ShowTabMenu(name)` and `act::ShowTreeMenu(name)` on what was clicked, and what their items do.

use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gpui::{Context, KeyDownEvent, Pixels, Point, Window};
use serde_json::json;

use super::Workspace;
use super::processes::ProgramEnding;
use crate::editor::Tab;
use crate::file_tree::EntryKind;
use crate::menu::nav::Nav;
use crate::menu::resolve::{self, Applies, ItemId, Line, LineAction, LineState};
use crate::menu::{Home, PathKind, Subject, SubjectPath, Within};
use crate::path_text::checkout_of;
use crate::script::keymap::Builtin;
use crate::script::menus::{ItemAction, MenuKind, Menus};
use crate::script::state::{ScriptState, invoke};
use crate::sidebar::rows::{TreeItem, TreeKey};
use crate::sidebar::selection;
use crate::tabs::{Beside, TabId};

/// The menu MCP's `tab-menu:<tab>` and `tree-menu:<row>` targets open.
pub const DEFAULT_MENU: &str = "main";
/// A line that moved under a still pointer this recently is not chosen by a press.
const MOVED_GUARD: Duration = Duration::from_millis(300);

/// What a menu was opened on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuOn {
    Tab(TabId),
    /// a tree row, by its `list_targets` name
    Tree { row: String },
}

/// The open menu.
#[derive(Debug, Clone)]
pub struct OpenMenu {
    pub on: MenuOn,
    pub kind: MenuKind,
    pub name: String,
    /// where it opened, in window coordinates
    pub position: Point<Pixels>,
    pub subject: Subject,
    pub lines: Vec<Line>,
    pub nav: Nav,
    /// predicates asked and not answered yet, by request
    waiting: Vec<(u64, ItemId)>,
    /// when lines last went, for [`MOVED_GUARD`]
    changed_at: Instant,
}

impl OpenMenu {
    /// Whether a press now may choose a line: none went in the last [`MOVED_GUARD`].
    pub fn is_settled(&self) -> bool {
        self.changed_at.elapsed() >= MOVED_GUARD
    }
}

/// What a tab menu's tab actions depend on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabFacts {
    pub temporary: bool,
    /// whether it shows a process (terminal, Claude Code, run), which Stop and Kill end
    pub process: bool,
}

/// Whether a tab action applies to a tab: Keep Tab on a temporary one, Stop and Kill on a process, the Close ones
/// enabled when there are tabs to close; `beside` tells which sides have tabs.
pub fn tab_applies(builtin: Builtin, facts: TabFacts, beside: impl Fn(Beside) -> bool) -> Applies {
    let shown = |enabled| Applies::Shown { enabled };
    match builtin {
        Builtin::KeepTab if facts.temporary => shown(true),
        Builtin::KeepTab => Applies::Hidden,
        Builtin::StopProgram | Builtin::KillProgram if facts.process => shown(true),
        Builtin::StopProgram | Builtin::KillProgram => Applies::Hidden,
        Builtin::CloseOtherTabs => shown(beside(Beside::Others)),
        Builtin::CloseTabsToTheLeft => shown(beside(Beside::Left)),
        Builtin::CloseTabsToTheRight => shown(beside(Beside::Right)),
        _ => shown(true),
    }
}

/// Today's names of the tab actions' lines, kept for MCP besides the label paths.
pub fn tab_action_target(builtin: Builtin) -> Option<&'static str> {
    Some(match builtin {
        Builtin::KeepTab => "tab-menu:keep",
        Builtin::StopProgram => "tab-menu:stop",
        Builtin::KillProgram => "tab-menu:kill",
        Builtin::CloseTab => "tab-menu:close",
        Builtin::CloseOtherTabs => "tab-menu:close-others",
        Builtin::CloseTabsToTheLeft => "tab-menu:close-left",
        Builtin::CloseTabsToTheRight => "tab-menu:close-right",
        _ => return None,
    })
}

/// `paths` with their kinds, each placed in the innermost of `checkouts` that holds it.
pub fn subject_of(paths: Vec<(PathBuf, PathKind)>, checkouts: &[PathBuf]) -> Subject {
    let roots = || checkouts.iter().map(PathBuf::as_path);
    let only: Vec<PathBuf> = paths.iter().map(|(path, _)| path.clone()).collect();
    let home = checkout_of(&only, roots()).map_or(Home::Mixed, Home::Checkout);
    let placed = paths.into_iter().map(|(path, kind)| {
        let within = match checkout_of(std::slice::from_ref(&path), roots()) {
            Some(root) => Within::Checkout(crate::path_text::relative(&path, &root).to_string_lossy().into_owned()),
            None => Within::Outside,
        };
        SubjectPath { path, within, kind }
    });
    Subject { paths: placed.collect(), home }
}

/// The rows a tree menu opened on `pressed` applies to: the whole selection of files and folders when `pressed` is
/// one of them, else `pressed` alone.
pub fn menu_rows(pressed: &TreeKey, is_selected: impl Fn(&TreeKey) -> bool, listed: &[selection::Listed]) -> Vec<TreeKey> {
    if matches!(pressed, TreeKey::Entry(_)) && is_selected(pressed) {
        return listed.iter().filter(|row| row.multi && is_selected(&row.key)).map(|row| row.key.clone()).collect();
    }
    vec![pressed.clone()]
}

/// The name of a line for MCP's `click`: `<kind>-menu:` and the labels down to it, joined by `/`; a built-in's choice
/// by its key.
pub fn line_target(kind: MenuKind, labels: &[String]) -> String {
    let prefix = match kind {
        MenuKind::Tab => "tab-menu",
        MenuKind::Tree => "tree-menu",
    };
    format!("{prefix}:{}", labels.join("/"))
}

impl Workspace {
    pub fn menu(&self) -> Option<&OpenMenu> {
        self.menu.as_ref().filter(|menu| match menu.on {
            MenuOn::Tab(tab) => self.tabs.row().any(|(id, _)| id == tab),
            MenuOn::Tree { .. } => true,
        })
    }

    /// Opens menu `name` of tab `tab` at the pointer; the tab is not shown, so a temporary one shown stays. A menu
    /// with nothing for the tab, or one the scripts have not declared (yet), does not open.
    pub fn open_tab_menu(&mut self, tab: TabId, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        let subject = self.tab_subject(tab);
        self.open_menu(MenuKind::Tab, name, MenuOn::Tab(tab), subject, window, cx);
    }

    /// Opens tree menu `name` on the row of `pressed`, named `row` for MCP; a row outside the selection becomes the
    /// selection first.
    pub fn open_tree_menu(&mut self, pressed: TreeKey, row: String, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        if !self.tree.selection.contains(&pressed) {
            self.tree.selection.select_only(pressed.clone());
        }
        let subject = self.tree_subject(&pressed);
        self.open_menu(MenuKind::Tree, name, MenuOn::Tree { row }, subject, window, cx);
    }

    /// What a menu or action on the row of `pressed` applies to, see [`menu_rows`].
    pub(super) fn tree_subject(&self, pressed: &TreeKey) -> Subject {
        let listed = selection::listed(&self.tree.rows);
        let rows = menu_rows(pressed, |key| self.tree.selection.contains(key), &listed);
        subject_of(rows.iter().filter_map(|key| self.row_path(key)).collect(), &self.all_checkouts())
    }

    /// Send to Terminal from a binding: its list of terminals alone, as a menu at the pointer.
    pub(super) fn open_terminal_list(&mut self, subject: Subject, on: MenuOn, window: &mut Window, cx: &mut Context<Self>) {
        let Applies::Choices(choices) = self.terminal_choices(&subject) else {
            return;
        };
        let lines = resolve::choice_lines(Builtin::SendPathsToTerminal, choices, &[]);
        let position = window.mouse_position();
        self.menu = Some(OpenMenu { on, kind: MenuKind::Tree, name: String::new(), position, subject, lines, nav: Nav::new(), waiting: Vec::new(), changed_at: Instant::now() - MOVED_GUARD });
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    /// Runs path action `builtin` from a mouse binding on the row of `pressed`; a row outside the selection becomes
    /// the selection first.
    pub fn run_row_path_action(&mut self, pressed: TreeKey, builtin: Builtin, window: &mut Window, cx: &mut Context<Self>) {
        if !self.tree.selection.contains(&pressed) {
            self.tree.selection.select_only(pressed.clone());
        }
        let subject = self.tree_subject(&pressed);
        if builtin == Builtin::SendPathsToTerminal {
            return self.open_terminal_list(subject, MenuOn::Tree { row: "selection".to_owned() }, window, cx);
        }
        self.run_path_action(builtin, &subject, window, cx);
    }

    /// Runs path action `builtin` from a binding: on the selection when the tree has the keyboard, else on the shown
    /// tab's file.
    pub(super) fn run_bound_path_action(&mut self, builtin: Builtin, in_tree: bool, window: &mut Window, cx: &mut Context<Self>) {
        let (subject, on) = match (in_tree, self.tree.selection.anchor().cloned(), self.tabs.active_id()) {
            (true, Some(anchor), _) => (self.tree_subject(&anchor), MenuOn::Tree { row: "selection".to_owned() }),
            (false, _, Some(shown)) => (self.tab_subject(shown), MenuOn::Tab(shown)),
            _ => return,
        };
        if builtin == Builtin::SendPathsToTerminal {
            return self.open_terminal_list(subject, on, window, cx);
        }
        self.run_path_action(builtin, &subject, window, cx);
    }

    /// Opens tree menu `name` on the selection, from a key; nothing when no row is selected.
    pub(super) fn open_tree_menu_on_selection(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(pressed) = self.tree.selection.anchor().cloned() else {
            return;
        };
        self.open_tree_menu(pressed, "selection".to_owned(), name, window, cx);
    }

    fn open_menu(&mut self, kind: MenuKind, name: &str, on: MenuOn, subject: Subject, window: &mut Window, cx: &mut Context<Self>) {
        let items = cx.try_global::<Menus>().and_then(|menus| menus.get(kind, name)).map(<[_]>::to_vec).unwrap_or_default();
        let resolved = resolve::resolve(&items, &subject, &|builtin| self.applies(builtin, &on, &subject));
        if resolved.lines.is_empty() {
            return;
        }
        let mut waiting = Vec::new();
        for test in resolved.tests {
            self.menu_requests += 1;
            let context = self.menu_context(name, &on, &subject);
            if let Some(state) = cx.try_global::<ScriptState>() {
                state.test(test.generation, test.function, context, self.menu_requests);
            }
            waiting.push((self.menu_requests, test.id));
        }
        let position = window.mouse_position();
        self.menu = Some(OpenMenu { on, kind, name: name.to_owned(), position, subject, lines: resolved.lines, nav: Nav::new(), waiting, changed_at: Instant::now() - MOVED_GUARD });
        // so the menu's keys reach the workspace, not a focused terminal
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    /// Closes the menu and gives the keyboard back to the shown tab.
    pub fn close_context_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.menu.take().is_some() {
            self.focus_active_tab(window, cx);
            cx.notify();
        }
    }

    /// A predicate asked as `request` answered; one of a closed menu changes nothing.
    pub(super) fn menu_answered(&mut self, request: u64, answer: bool, cx: &mut Context<Self>) {
        let Some(menu) = self.menu.as_mut() else {
            return;
        };
        let Some(at) = menu.waiting.iter().position(|(asked, _)| *asked == request) else {
            return;
        };
        let (_, id) = menu.waiting.remove(at);
        let before = std::mem::take(&mut menu.lines);
        menu.lines = resolve::answered(before.clone(), &id, answer);
        menu.nav = menu.nav.settle(&before, &menu.lines);
        if !answer {
            menu.changed_at = Instant::now();
        }
        if menu.lines.is_empty() {
            self.menu = None;
        }
        cx.notify();
    }

    pub fn hover_menu_line(&mut self, id: ItemId, cx: &mut Context<Self>) {
        if let Some(menu) = self.menu.as_mut() {
            menu.nav = menu.nav.hover(id);
            // hovering a submenu's line opens it
            menu.nav = menu.nav.right(&menu.lines);
            cx.notify();
        }
    }

    /// The menu's keys: ↑ ↓ move, → opens a submenu, ← and Escape close one, Enter chooses; whether it took the key.
    pub(super) fn menu_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(menu) = self.menu.as_mut() else {
            return false;
        };
        let nav = &menu.nav;
        menu.nav = match event.keystroke.key.as_str() {
            "down" => nav.down(&menu.lines),
            "up" => nav.up(&menu.lines),
            "right" => nav.right(&menu.lines),
            "left" | "escape" if nav.is_at_top() => {
                self.close_context_menu(window, cx);
                return true;
            }
            "left" | "escape" => nav.left(),
            "enter" => {
                let id = nav.highlighted().cloned();
                if let Some(id) = id {
                    self.choose_menu_line(&id, window, cx);
                }
                return true;
            }
            _ => return false,
        };
        cx.notify();
        true
    }

    /// Runs line `id` of the open menu, closing the menu; a submenu's line opens it instead. A pending or disabled
    /// line does nothing.
    pub fn choose_menu_line(&mut self, id: &ItemId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(menu) = self.menu.as_mut() else {
            return;
        };
        let chosen = menu.nav.hover(id.clone()).chosen(&menu.lines).cloned();
        match chosen {
            Some(Line::Submenu { state: LineState::Ready, .. }) => {
                menu.nav = menu.nav.hover(id.clone()).right(&menu.lines);
                cx.notify();
            }
            Some(Line::Item { action, state: LineState::Ready, enabled: true, .. }) => {
                let Some(menu) = self.menu.take() else {
                    return;
                };
                self.focus_active_tab(window, cx);
                self.run_menu_action(action, &menu, window, cx);
                cx.notify();
            }
            _ => {}
        }
    }

    fn run_menu_action(&mut self, action: LineAction, menu: &OpenMenu, window: &mut Window, cx: &mut Context<Self>) {
        match (action, &menu.on) {
            (LineAction::Script(ItemAction::Invoke { generation, function }), on) => invoke(generation, function, Some(self.menu_context(&menu.name, on, &menu.subject)), cx),
            (LineAction::Script(ItemAction::Builtin(builtin)), MenuOn::Tab(tab)) if builtin.is_tab_command() => self.run_tab_command(builtin, *tab, window, cx),
            (LineAction::Script(ItemAction::Builtin(builtin)), _) => self.run_menu_builtin(builtin, &menu.subject, window, cx),
            (LineAction::Choice { builtin: Builtin::SendPathsToTerminal, choice }, _) => self.send_paths_to(&choice, &menu.subject, window, cx),
            (LineAction::Choice { builtin: Builtin::RunFile, choice }, _) => self.run_from_menu(Some(&choice), &menu.subject, window, cx),
            (LineAction::Choice { .. }, _) => {}
        }
    }

    /// A built-in chosen from a menu, on the menu's subject.
    fn run_menu_builtin(&mut self, builtin: Builtin, subject: &Subject, window: &mut Window, cx: &mut Context<Self>) {
        match super::path_actions::path_applies(builtin, subject) {
            Some(_) => self.run_path_action(builtin, subject, window, cx),
            None if builtin == Builtin::RunFile => self.run_from_menu(None, subject, window, cx),
            None if self.git_applies(builtin, subject).is_some() => self.run_git_action(builtin, subject, window, cx),
            None => self.run_window_builtin(builtin, window, cx),
        }
    }

    /// What `builtin` does on `subject`, opened on `on`.
    fn applies(&self, builtin: Builtin, on: &MenuOn, subject: &Subject) -> Applies {
        match (builtin, on) {
            (tab_action, MenuOn::Tab(tab)) if tab_action.is_tab_command() => {
                let process = matches!(self.tabs.get(*tab), Some(Tab::Process(_)));
                let facts = TabFacts { temporary: self.tabs.is_temporary(*tab), process };
                tab_applies(tab_action, facts, |which| self.tabs.has_beside(*tab, which))
            }
            (Builtin::OpenHub | Builtin::OpenSettings | Builtin::OpenTerminal | Builtin::OpenTerminalIn | Builtin::ToggleChangesLayout, _) => Applies::Shown { enabled: true },
            (Builtin::SendPathsToTerminal, _) => self.terminal_choices(subject),
            (Builtin::RunFile, _) => self.run_file_applies(subject),
            (git_action, _) if self.git_applies(git_action, subject).is_some() => self.git_applies(git_action, subject).unwrap_or(Applies::Hidden),
            (path_action, _) => super::path_actions::path_applies(path_action, subject).unwrap_or(Applies::Hidden),
        }
    }

    /// The file a tab shows: a file, a diff's file, a scratch; nothing for the rest.
    pub(super) fn tab_subject(&self, tab: TabId) -> Subject {
        let path = match self.tabs.get(tab) {
            Some(Tab::Document(editor)) => Some(editor.path().to_path_buf()),
            Some(Tab::Scratch(scratch)) => Some(scratch.path().to_path_buf()),
            Some(Tab::Diff(diff)) => Some(diff.checkout.path.join(Path::new(std::ffi::OsStr::from_bytes(diff.request.file.path.as_bytes())))),
            _ => None,
        };
        subject_of(path.map(|path| (path, PathKind::File)).into_iter().collect(), &self.all_checkouts())
    }

    /// A tree row's path and kind: a file or folder, a checkout's root, a project's main checkout, a changed file.
    fn row_path(&self, key: &TreeKey) -> Option<(PathBuf, PathKind)> {
        let row = self.tree.rows.iter().find(|row| row.key() == *key)?;
        match &row.item {
            TreeItem::Entry(entry) => match entry.entry.kind {
                EntryKind::Folder => Some((entry.entry.path.clone(), PathKind::Folder)),
                EntryKind::File => Some((entry.entry.path.clone(), PathKind::File)),
                _ => None,
            },
            TreeItem::Checkout(checkout) => Some((checkout.checkout.path.clone(), PathKind::Folder)),
            TreeItem::Project(project) => self.project(&project.id).map(|project| (project.main_checkout().path.clone(), PathKind::Folder)),
            TreeItem::Change(change) => {
                let relative = Path::new(std::ffi::OsStr::from_bytes(change.change.path.as_bytes()));
                Some((change.checkout.path.join(relative), PathKind::File))
            }
            _ => None,
        }
    }

    /// Every checkout of the open projects.
    pub(super) fn all_checkouts(&self) -> Vec<PathBuf> {
        self.projects.iter().flat_map(|open| open.project.checkouts.iter().map(|checkout| checkout.path.clone())).collect()
    }

    /// A menu function's `context`: the menu, its paths, their checkout, and the row or tab.
    fn menu_context(&self, name: &str, on: &MenuOn, subject: &Subject) -> serde_json::Value {
        let paths: Vec<String> = subject.paths.iter().map(|path| path.path.display().to_string()).collect();
        let items: Vec<serde_json::Value> = subject.paths.iter().map(item_json).collect();
        let checkout = match &subject.home {
            Home::Checkout(path) => json!(path.display().to_string()),
            Home::Mixed => serde_json::Value::Null,
        };
        let mut context = json!({ "menu": name, "paths": paths, "items": items, "checkout": checkout });
        match on {
            MenuOn::Tab(tab) => context["tab"] = json!(tab.value()),
            MenuOn::Tree { row } => context["row"] = json!(row),
        }
        context
    }

    /// Stop and Kill: a process tab's process ends for good (its tab closes with it); any other tab just closes, so
    /// one key stops or closes whatever is shown.
    fn stop_tab(&mut self, tab: TabId, ending: ProgramEnding, window: &mut Window, cx: &mut Context<Self>) {
        match self.tabs.get(tab) {
            Some(Tab::Process(id)) => {
                let id = *id;
                self.stop_process(id, ending, window, cx);
            }
            _ => self.close_tab(tab, window, cx),
        }
    }

    /// Runs tab action `command` (see [`Builtin::is_tab_command`]) on tab `tab`.
    pub fn run_tab_command(&mut self, command: Builtin, tab: TabId, window: &mut Window, cx: &mut Context<Self>) {
        let beside = match command {
            Builtin::KeepTab => return self.keep_tab(tab, cx),
            Builtin::CloseTab => return self.close_tab(tab, window, cx),
            Builtin::StopProgram => return self.stop_tab(tab, ProgramEnding::Stop, window, cx),
            Builtin::KillProgram => return self.stop_tab(tab, ProgramEnding::Kill, window, cx),
            Builtin::CloseOtherTabs => Beside::Others,
            Builtin::CloseTabsToTheLeft => Beside::Left,
            Builtin::CloseTabsToTheRight => Beside::Right,
            _ => return,
        };
        self.tabs.close_beside(tab, beside);
        self.tab_shown(window, cx);
    }
}

fn item_json(path: &SubjectPath) -> serde_json::Value {
    let relative = match &path.within {
        Within::Checkout(relative) => json!(relative),
        Within::Outside => serde_json::Value::Null,
    };
    let kind = match path.kind {
        PathKind::File => "file",
        PathKind::Folder => "folder",
    };
    json!({ "path": path.path.display().to_string(), "relative": relative, "kind": kind })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::resolve::resolve;
    use crate::script::menus::{Label, MenuItem, When};
    use crate::sidebar::selection::Listed;

    fn item(builtin: Builtin) -> MenuItem {
        MenuItem::Action { label: Label::Default, action: ItemAction::Builtin(builtin), when: When::default() }
    }

    fn default_menu() -> Vec<MenuItem> {
        vec![
            item(Builtin::KeepTab),
            item(Builtin::StopProgram),
            MenuItem::Separator,
            item(Builtin::CloseTab),
            item(Builtin::CloseOtherTabs),
            item(Builtin::CloseTabsToTheLeft),
            item(Builtin::CloseTabsToTheRight),
        ]
    }

    fn labels(items: &[MenuItem], facts: TabFacts, beside: impl Fn(Beside) -> bool + Copy) -> Vec<String> {
        let lines = resolve(items, &Subject::empty(), &|builtin| tab_applies(builtin, facts, beside)).lines;
        lines
            .iter()
            .map(|line| match line {
                Line::Item { label, enabled: true, .. } => label.clone(),
                Line::Item { label, .. } => format!("({label})"),
                Line::Submenu { label, .. } => label.clone(),
                Line::Separator => "-".to_owned(),
            })
            .collect()
    }

    #[test]
    fn should_offer_only_the_close_actions_for_a_permanent_tab() {
        // given
        let facts = TabFacts { temporary: false, process: false };

        // when
        let lines = labels(&default_menu(), facts, |which| which != Beside::Left);

        // then
        assert_eq!(lines, ["Close", "Close Other Tabs", "(Close Tabs to the Left)", "Close Tabs to the Right"]);
    }

    #[test]
    fn should_offer_keep_tab_for_a_temporary_tab() {
        // given
        let facts = TabFacts { temporary: true, process: false };

        // when
        let lines = labels(&default_menu(), facts, |_| false);

        // then
        assert_eq!(lines, ["Keep Tab", "-", "Close", "(Close Other Tabs)", "(Close Tabs to the Left)", "(Close Tabs to the Right)"]);
    }

    #[test]
    fn should_offer_stop_for_any_process_tab() {
        // given
        let facts = TabFacts { temporary: false, process: true };

        // when
        let lines = labels(&default_menu(), facts, |_| true);

        // then
        assert_eq!(lines[..2], ["Stop", "-"]);
    }

    #[test]
    fn should_offer_kill_when_a_menu_names_it() {
        // given
        let facts = TabFacts { temporary: false, process: true };

        // when
        let lines = labels(&[item(Builtin::StopProgram), item(Builtin::KillProgram)], facts, |_| true);

        // then
        assert_eq!(lines, ["Stop", "Kill"]);
    }

    #[test]
    fn should_leave_a_menu_of_process_items_empty_on_a_file_tab() {
        // given
        let facts = TabFacts { temporary: false, process: false };

        // when
        let lines = labels(&[item(Builtin::StopProgram), MenuItem::Separator, item(Builtin::KillProgram)], facts, |_| true);

        // then
        assert!(lines.is_empty());
    }

    fn entry(path: &str) -> TreeKey {
        TreeKey::Entry(PathBuf::from(path))
    }

    fn listed(keys: &[TreeKey]) -> Vec<Listed> {
        keys.iter().map(|key| Listed { key: key.clone(), multi: matches!(key, TreeKey::Entry(_)) }).collect()
    }

    #[test]
    fn should_apply_to_the_whole_selection_when_the_pressed_row_is_in_it() {
        // given
        let rows = [entry("/w/a"), entry("/w/b"), entry("/w/c")];
        let selected = [entry("/w/a"), entry("/w/c")];

        // when
        let applied = menu_rows(&entry("/w/c"), |key| selected.contains(key), &listed(&rows));

        // then
        assert_eq!(applied, selected);
    }

    #[test]
    fn should_apply_a_worktree_row_alone() {
        // given
        let worktree = TreeKey::Checkout(PathBuf::from("/w"));
        let rows = [worktree.clone(), entry("/w/a")];

        // when
        let applied = menu_rows(&worktree, |_| true, &listed(&rows));

        // then
        assert_eq!(applied, [worktree]);
    }

    #[test]
    fn should_place_paths_in_their_checkouts() {
        // given
        let checkouts = [PathBuf::from("/w"), PathBuf::from("/w/.worktree/b")];

        // when
        let subject = subject_of(vec![(PathBuf::from("/w/.worktree/b/src"), PathKind::Folder), (PathBuf::from("/tmp/x"), PathKind::File)], &checkouts);

        // then
        assert_eq!(subject.paths[0].within, Within::Checkout("src".to_owned()));
        assert_eq!(subject.paths[1].within, Within::Outside);
        assert_eq!(subject.home, Home::Mixed);
    }

    #[test]
    fn should_name_targets_by_label_path() {
        // when
        let name = line_target(MenuKind::Tree, &["Copy Path".to_owned(), "Copy Relative Path".to_owned()]);

        // then
        assert_eq!(name, "tree-menu:Copy Path/Copy Relative Path");
    }
}
