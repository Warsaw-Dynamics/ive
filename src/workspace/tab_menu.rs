//! Tab actions (Keep Tab, Stop and Kill a process tab's process for good, the Close ones) and the menus of them that
//! `config.tab_menus` declares and `act::ShowTabMenu(name)` opens.

use gpui::{Context, Pixels, Point, Window};

use super::Workspace;
use super::processes::ProgramEnding;
use crate::editor::Tab;
use crate::script::keymap::Builtin;
use crate::script::menus::{ItemAction, MenuItem, MenuKind, Menus};
use crate::tabs::{Beside, TabId};

/// The menu MCP's `tab-menu:<tab>` target opens.
pub const DEFAULT_MENU: &str = "main";

/// The open menu: the tab it is for, where it opened, in window coordinates, and its lines.
#[derive(Debug, Clone, PartialEq)]
pub struct TabMenu {
    pub tab: TabId,
    pub position: Point<Pixels>,
    entries: Vec<MenuEntry>,
}

/// A line of a tab menu as declared: a built-in tab action or a separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntry {
    Item(Builtin),
    Separator,
}

/// `items` as the tab menu draws them today: built-in actions and separators; submenus and functions wait for the
/// shared menu view.
fn entries(items: &[MenuItem]) -> Vec<MenuEntry> {
    let entry = |item: &MenuItem| match item {
        MenuItem::Action { action: ItemAction::Builtin(builtin), .. } => Some(MenuEntry::Item(*builtin)),
        MenuItem::Separator => Some(MenuEntry::Separator),
        _ => None,
    };
    items.iter().filter_map(entry).collect()
}

/// A line of the open menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuLine {
    Item { command: Builtin, enabled: bool },
    Separator,
}

/// What a menu shows about its tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabFacts {
    pub temporary: bool,
    /// whether it shows a process (terminal, Claude Code, run), which Stop and Kill end
    pub process: bool,
}

/// The item's text; `None` for an action that is not a tab action.
pub fn label(command: Builtin) -> Option<&'static str> {
    Some(match command {
        Builtin::KeepTab => "Keep Tab",
        Builtin::StopProgram => "Stop",
        Builtin::KillProgram => "Kill",
        Builtin::CloseTab => "Close",
        Builtin::CloseOtherTabs => "Close Other Tabs",
        Builtin::CloseTabsToTheLeft => "Close Tabs to the Left",
        Builtin::CloseTabsToTheRight => "Close Tabs to the Right",
        _ => return None,
    })
}

/// The item's click target, as MCP's `click` names it.
pub fn target(command: Builtin) -> Option<&'static str> {
    Some(match command {
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

/// `entries` for a tab: items it has no use for left out (Keep Tab unless it is temporary, Stop and Kill unless it is
/// a process), then separators at either end or next to another; `beside` tells which sides have tabs to close.
pub fn menu_lines(entries: &[MenuEntry], facts: TabFacts, beside: impl Fn(Beside) -> bool) -> Vec<MenuLine> {
    let shown = entries.iter().filter_map(|entry| line_for(*entry, facts, &beside));
    let mut lines: Vec<MenuLine> = Vec::new();
    for line in shown {
        let dangling = line == MenuLine::Separator && lines.last().is_none_or(|last| *last == MenuLine::Separator);
        if dangling {
            continue;
        }
        lines.push(line);
    }
    if lines.last() == Some(&MenuLine::Separator) {
        lines.pop();
    }
    lines
}

fn line_for(entry: MenuEntry, facts: TabFacts, beside: &impl Fn(Beside) -> bool) -> Option<MenuLine> {
    let item = |command, enabled| Some(MenuLine::Item { command, enabled });
    match entry {
        MenuEntry::Separator => Some(MenuLine::Separator),
        MenuEntry::Item(Builtin::KeepTab) if facts.temporary => item(Builtin::KeepTab, true),
        MenuEntry::Item(Builtin::KeepTab) => None,
        MenuEntry::Item(Builtin::StopProgram | Builtin::KillProgram) if !facts.process => None,
        MenuEntry::Item(Builtin::CloseOtherTabs) => item(Builtin::CloseOtherTabs, beside(Beside::Others)),
        MenuEntry::Item(Builtin::CloseTabsToTheLeft) => item(Builtin::CloseTabsToTheLeft, beside(Beside::Left)),
        MenuEntry::Item(Builtin::CloseTabsToTheRight) => item(Builtin::CloseTabsToTheRight, beside(Beside::Right)),
        MenuEntry::Item(command) => item(command, true),
    }
}

impl Workspace {
    pub fn tab_menu(&self) -> Option<&TabMenu> {
        self.tab_menu.as_ref().filter(|menu| self.tabs.row().any(|(id, _)| id == menu.tab))
    }

    /// Opens menu `name` of tab `tab` at the pointer; the tab is not shown, so a temporary one shown stays. A menu
    /// with nothing for the tab, or one the scripts have not declared (yet), does not open.
    pub fn open_tab_menu(&mut self, tab: TabId, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        let entries = cx.try_global::<Menus>().and_then(|menus| menus.get(MenuKind::Tab, name)).map(entries).unwrap_or_default();
        let menu = TabMenu { tab, position: window.mouse_position(), entries };
        if self.tab_menu_lines(&menu).is_empty() {
            return;
        }
        self.tab_menu = Some(menu);
        // so Escape reaches the workspace, not a focused terminal
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    /// Closes the menu and gives the keyboard back to the shown tab.
    pub fn close_tab_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.tab_menu.take().is_some() {
            self.focus_active_tab(window, cx);
            cx.notify();
        }
    }

    /// The lines `menu` shows for its tab.
    pub fn tab_menu_lines(&self, menu: &TabMenu) -> Vec<MenuLine> {
        let process = matches!(self.tabs.get(menu.tab), Some(Tab::Process(_)));
        let facts = TabFacts { temporary: self.tabs.is_temporary(menu.tab), process };
        menu_lines(&menu.entries, facts, |which| self.tabs.has_beside(menu.tab, which))
    }

    /// Runs an item of the open menu on its tab, closing the menu.
    pub fn run_tab_menu_item(&mut self, command: Builtin, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tab) = self.tab_menu().map(|menu| menu.tab) else {
            return;
        };
        self.close_tab_menu(window, cx);
        self.run_tab_command(command, tab, window, cx);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT: [MenuEntry; 7] = [
        MenuEntry::Item(Builtin::KeepTab),
        MenuEntry::Item(Builtin::StopProgram),
        MenuEntry::Separator,
        MenuEntry::Item(Builtin::CloseTab),
        MenuEntry::Item(Builtin::CloseOtherTabs),
        MenuEntry::Item(Builtin::CloseTabsToTheLeft),
        MenuEntry::Item(Builtin::CloseTabsToTheRight),
    ];

    fn labels(lines: &[MenuLine]) -> Vec<String> {
        lines
            .iter()
            .map(|line| match line {
                MenuLine::Item { command, enabled: true } => label(*command).unwrap().to_owned(),
                MenuLine::Item { command, enabled: false } => format!("({})", label(*command).unwrap()),
                MenuLine::Separator => "-".to_owned(),
            })
            .collect()
    }

    #[test]
    fn should_offer_only_the_close_actions_for_a_permanent_tab() {
        // given
        let facts = TabFacts { temporary: false, process: false };

        // when
        let lines = menu_lines(&DEFAULT, facts, |which| which != Beside::Left);

        // then
        assert_eq!(labels(&lines), ["Close", "Close Other Tabs", "(Close Tabs to the Left)", "Close Tabs to the Right"]);
    }

    #[test]
    fn should_offer_keep_tab_for_a_temporary_tab() {
        // given
        let facts = TabFacts { temporary: true, process: false };

        // when
        let lines = menu_lines(&DEFAULT, facts, |_| false);

        // then
        assert_eq!(labels(&lines), ["Keep Tab", "-", "Close", "(Close Other Tabs)", "(Close Tabs to the Left)", "(Close Tabs to the Right)"]);
    }

    #[test]
    fn should_offer_stop_for_any_process_tab() {
        // given
        let facts = TabFacts { temporary: false, process: true };

        // when
        let lines = menu_lines(&DEFAULT, facts, |_| true);

        // then
        assert_eq!(labels(&lines)[..2], ["Stop", "-"]);
    }

    #[test]
    fn should_offer_kill_when_a_menu_names_it() {
        // given
        let entries = [MenuEntry::Item(Builtin::StopProgram), MenuEntry::Item(Builtin::KillProgram)];
        let facts = TabFacts { temporary: false, process: true };

        // when
        let lines = menu_lines(&entries, facts, |_| true);

        // then
        assert_eq!(labels(&lines), ["Stop", "Kill"]);
    }

    #[test]
    fn should_drop_separators_left_dangling() {
        // given
        let entries = [
            MenuEntry::Separator,
            MenuEntry::Item(Builtin::KeepTab),
            MenuEntry::Separator,
            MenuEntry::Item(Builtin::CloseTab),
            MenuEntry::Separator,
            MenuEntry::Item(Builtin::StopProgram),
        ];
        let facts = TabFacts { temporary: false, process: false };

        // when
        let lines = menu_lines(&entries, facts, |_| true);

        // then
        assert_eq!(labels(&lines), ["Close"]);
    }

    #[test]
    fn should_leave_a_menu_of_process_items_empty_on_a_file_tab() {
        // given
        let entries = [MenuEntry::Item(Builtin::StopProgram), MenuEntry::Separator, MenuEntry::Item(Builtin::KillProgram)];
        let facts = TabFacts { temporary: false, process: false };

        // when
        let lines = menu_lines(&entries, facts, |_| true);

        // then
        assert_eq!(lines, []);
    }
}
