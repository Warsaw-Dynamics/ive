//! Key and mouse bindings as plain data the gpui thread can read: no Rune values, only what a trigger does.

use gpui::{Keystroke, Modifiers};

use crate::sidebar::folders::FolderKind;

/// Where a binding applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// keys anywhere in the window
    Window,
    /// keys while a terminal has the keyboard; the mouse over a terminal's grid
    Terminal,
    /// keys while the scratch editor has the keyboard
    Editor,
    /// the mouse on a tab of the tab strip
    Tab,
    /// the mouse on a row of the sidebar tree
    Tree,
    /// keys while the hub (Ctrl+Shift+E) has the keyboard
    Hub,
}

impl Scope {
    pub fn name(self) -> &'static str {
        match self {
            Self::Window => "window",
            Self::Terminal => "terminal",
            Self::Editor => "editor",
            Self::Tab => "tab",
            Self::Tree => "tree",
            Self::Hub => "hub",
        }
    }
}

/// Where a key was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Terminal,
    Editor,
    Hub,
    /// the sidebar has the keyboard
    Tree,
    Elsewhere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseKind {
    Down,
    Up,
    Drag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// A mouse gesture as WezTerm names it: `Down`/`Up`/`Drag` of a button, the click count, the modifiers held, and
/// whether it applies while the terminal's program tracks the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseTrigger {
    pub kind: MouseKind,
    pub button: MouseButton,
    /// 1 to 3: single, double, triple click
    pub streak: u8,
    pub mods: Modifiers,
    pub mouse_reporting: bool,
}

impl MouseTrigger {
    fn describe(&self) -> String {
        let mods = Keystroke { modifiers: self.mods, key: String::new(), key_char: None }.unparse();
        format!("{mods}{:?} {:?} x{}", self.button, self.kind, self.streak).to_lowercase()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trigger {
    Key(Keystroke),
    Mouse(MouseTrigger),
}

impl Trigger {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Key(a), Self::Key(b)) => same_key(a, b),
            (Self::Mouse(a), Self::Mouse(b)) => a == b,
            _ => false,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Self::Key(keystroke) => keystroke.unparse(),
            Self::Mouse(mouse) => mouse.describe(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Cell,
    Word,
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardTarget {
    Clipboard,
    Primary,
    Both,
}

/// What a terminal paste takes from the clipboard: `TextElseImage` is `act::PasteFrom`'s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteKind {
    TextElseImage,
    Text,
    Image,
    /// the text, a space, the image's path
    TextAndImage,
}

impl PasteKind {
    /// The `act::` action that pastes this kind.
    pub fn action_name(self) -> &'static str {
        match self {
            Self::TextElseImage => "PasteFrom",
            Self::Text => "PasteTextFrom",
            Self::Image => "PasteImageFrom",
            Self::TextAndImage => "PasteTextAndImageFrom",
        }
    }
}

/// What IVE does itself, on the gpui thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    Nop,
    OpenSettings,
    OpenTerminal,
    ActivateTab,
    CloseTab,
    /// makes a temporary tab stay when another is shown
    KeepTab,
    /// the program in a process tab's foreground, or its run: SIGTERM (a run's Stop)
    StopProgram,
    /// as `StopProgram`, SIGKILL at once
    KillProgram,
    CloseOtherTabs,
    CloseTabsToTheLeft,
    CloseTabsToTheRight,
    /// changed files flat or grouped by folder, in the tree's git folders and the Git sections
    ToggleChangesLayout,
    /// a worktree folder shown in every worktree, or hidden in every one (the tree header's folders menu)
    ToggleFolderEverywhere(FolderKind),
    /// the hub asking where a new terminal goes (Ctrl+Shift+T)
    OpenTerminalIn,
    ActivateRow,
    /// `ActivateRow`, its tab opened, or kept, to stay
    ActivateRowToKeep,
    ToggleRow,
    /// Ctrl+click: adds a file or folder row to the selection, or takes it out
    ToggleRowSelection,
    /// Shift+click: selects the files and folders from the anchor to the row
    ExtendRowSelection,
    SelectAll,
    Cut,
    Undo,
    Redo,
    CopyTo(ClipboardTarget),
    PasteFrom(ClipboardTarget),
    /// the terminal-only pastes besides `PasteFrom`
    Paste(PasteKind, ClipboardTarget),
    SelectTextAtMouseCursor(SelectionMode),
    ExtendSelectionToMouseCursor(SelectionMode),
    CompleteSelection(ClipboardTarget),
    /// shows the hub (Ctrl+Shift+E); in the hub, `HubToggleChanged` is its key
    OpenHub,
    HubDown,
    HubUp,
    HubPageDown,
    HubPageUp,
    /// opens the selected row, or asks its worktree
    HubOpen,
    /// opens the row behind the hub, which stays
    HubOpenBehind,
    /// as `HubOpen` on numbered row 1 to 9
    HubOpenRow(u8),
    /// types the row's path into the last terminal
    HubTypePath,
    HubCopyPath,
    /// the absolute paths of the hub's row, or of the tree's selected files and folders, one per line
    CopyFullPath,
    /// stops the selected row's process, as `StopProgram` a process tab's
    HubStop,
    HubCycleScope,
    /// back to all projects, on an empty query
    HubWiden,
    HubToggleExcluded,
    HubToggleChanged,
    /// the worktree question, else the query, else the hub
    HubBack,
}

impl Builtin {
    /// Whether the action means something in `scope`.
    pub fn supports(&self, scope: Scope) -> bool {
        match self {
            Self::Nop => true,
            Self::OpenSettings | Self::OpenTerminal | Self::OpenTerminalIn | Self::ToggleChangesLayout | Self::ToggleFolderEverywhere(_) => scope == Scope::Window,
            Self::CloseTab | Self::KeepTab => matches!(scope, Scope::Window | Scope::Tab),
            Self::StopProgram | Self::KillProgram | Self::CloseOtherTabs | Self::CloseTabsToTheLeft | Self::CloseTabsToTheRight => {
                matches!(scope, Scope::Window | Scope::Tab)
            }
            Self::ActivateTab => scope == Scope::Tab,
            Self::ActivateRow | Self::ActivateRowToKeep | Self::ToggleRow | Self::ToggleRowSelection | Self::ExtendRowSelection => scope == Scope::Tree,
            Self::SelectAll | Self::Cut | Self::Undo | Self::Redo => scope == Scope::Editor,
            Self::CopyTo(_) | Self::PasteFrom(_) => matches!(scope, Scope::Terminal | Scope::Editor),
            Self::Paste(..) => scope == Scope::Terminal,
            Self::SelectTextAtMouseCursor(_) | Self::ExtendSelectionToMouseCursor(_) | Self::CompleteSelection(_) => scope == Scope::Terminal,
            Self::OpenHub => matches!(scope, Scope::Window | Scope::Hub),
            Self::CopyFullPath => matches!(scope, Scope::Hub | Scope::Tree),
            Self::HubDown
            | Self::HubUp
            | Self::HubPageDown
            | Self::HubPageUp
            | Self::HubOpen
            | Self::HubOpenBehind
            | Self::HubOpenRow(_)
            | Self::HubTypePath
            | Self::HubCopyPath
            | Self::HubStop
            | Self::HubCycleScope
            | Self::HubWiden
            | Self::HubToggleExcluded
            | Self::HubToggleChanged
            | Self::HubBack => scope == Scope::Hub,
        }
    }

    /// Whether the action is one of the hub's, which its query's keys run before the field sees them.
    pub fn is_hub_command(&self) -> bool {
        self.supports(Scope::Hub) && !matches!(self, Self::Nop | Self::OpenHub)
    }

    /// Whether the action can be an item of a tab menu.
    pub fn is_tab_command(&self) -> bool {
        matches!(
            self,
            Self::KeepTab | Self::StopProgram | Self::KillProgram | Self::CloseTab | Self::CloseOtherTabs | Self::CloseTabsToTheLeft | Self::CloseTabsToTheRight
        )
    }

    /// Whether the action needs a place the mouse points at, so a key cannot run it.
    pub fn is_mouse_only(&self) -> bool {
        matches!(
            self,
            Self::SelectTextAtMouseCursor(_)
                | Self::ExtendSelectionToMouseCursor(_)
                | Self::CompleteSelection(_)
                // a tree row's actions need the row under the pointer
                | Self::ActivateRow
                | Self::ActivateRowToKeep
                | Self::ToggleRow
                | Self::ToggleRowSelection
                | Self::ExtendRowSelection
        )
    }
}

/// What a trigger does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Builtin(Builtin),
    /// types the text into the terminal, at once, on the gpui thread
    Send(String),
    /// runs function `function` of script load `generation` on the script thread
    Invoke { generation: u64, function: usize },
    /// `DisableDefaultAssignment`: the trigger does nothing in IVE
    Disable,
    /// opens the clicked tab's menu of that name, from `config.tab_menus`, at the pointer
    ShowTabMenu(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub trigger: Trigger,
    pub scope: Scope,
    pub action: Action,
}

impl Binding {
    fn same_trigger(&self, other: &Self) -> bool {
        self.scope == other.scope && self.trigger.same(&other.trigger)
    }
}

/// The bindings in effect, checked when loaded: no trigger twice in one scope.
#[derive(Debug, Clone, Default)]
pub struct Keymap {
    bindings: Vec<Binding>,
}

impl Keymap {
    pub fn new(bindings: Vec<Binding>) -> Result<Self, String> {
        for (index, binding) in bindings.iter().enumerate() {
            if bindings[..index].iter().any(|earlier| earlier.same_trigger(binding)) {
                return Err(format!("{} is bound twice in {} scope", binding.trigger.describe(), binding.scope.name()));
            }
        }
        Ok(Self { bindings })
    }

    pub fn empty() -> Self {
        Self::default()
    }

    /// `user`'s bindings over `defaults`: a user binding replaces the default with the same trigger and scope.
    pub fn merged(defaults: Keymap, user: Keymap) -> Keymap {
        let kept = defaults.bindings.into_iter().filter(|default| !user.bindings.iter().any(|mine| mine.same_trigger(default)));
        Self { bindings: user.bindings.iter().cloned().chain(kept).collect() }
    }

    /// Bindings that do something: disabled defaults are not counted.
    pub fn len(&self) -> usize {
        self.bindings.iter().filter(|binding| binding.action != Action::Disable).count()
    }

    /// The trigger bound to function `function`, for messages about it.
    pub fn invoked(&self, function: usize) -> Option<String> {
        let invokes = |binding: &&Binding| matches!(binding.action, Action::Invoke { function: bound, .. } if bound == function);
        self.bindings.iter().find(invokes).map(|binding| binding.trigger.describe())
    }

    /// The names of the tab menus the bindings open.
    pub fn tab_menu_names(&self) -> impl Iterator<Item = &str> {
        self.bindings.iter().filter_map(|binding| match &binding.action {
            Action::ShowTabMenu(name) => Some(name.as_str()),
            _ => None,
        })
    }

    /// The action for `keystroke` pressed at `place`: in a terminal, the editor or the hub, its binding before a
    /// window one.
    pub fn find_key(&self, keystroke: &Keystroke, place: Place) -> Option<&Action> {
        let in_scope = |scope| self.bindings.iter().find(|binding| binding.scope == scope && matches!(&binding.trigger, Trigger::Key(bound) if same_key(bound, keystroke)));
        let local = match place {
            Place::Terminal => in_scope(Scope::Terminal),
            Place::Editor => in_scope(Scope::Editor),
            Place::Hub => in_scope(Scope::Hub),
            Place::Tree => in_scope(Scope::Tree),
            Place::Elsewhere => None,
        };
        enabled(local.or_else(|| in_scope(Scope::Window)))
    }

    /// The keys bound to `action` in `scope`, in binding order, for the hub's key strip.
    pub fn keys_for(&self, scope: Scope, action: &Action) -> Vec<&Keystroke> {
        let bound = self.bindings.iter().filter(|binding| binding.scope == scope && binding.action == *action);
        bound.filter_map(|binding| match &binding.trigger {
            Trigger::Key(keystroke) => Some(keystroke),
            Trigger::Mouse(_) => None,
        })
        .collect()
    }

    /// The action for a mouse gesture in `scope`: the binding with exactly its modifiers, else the one without any.
    pub fn find_mouse(&self, scope: Scope, trigger: &MouseTrigger) -> Option<&Action> {
        let find = |wanted: &MouseTrigger| self.bindings.iter().find(|binding| binding.scope == scope && binding.trigger == Trigger::Mouse(*wanted));
        let without_mods = MouseTrigger { mods: Modifiers::default(), ..*trigger };
        enabled(find(trigger).or_else(|| find(&without_mods)))
    }
}

fn enabled(binding: Option<&Binding>) -> Option<&Action> {
    binding.map(|binding| &binding.action).filter(|action| **action != Action::Disable)
}

fn same_key(a: &Keystroke, b: &Keystroke) -> bool {
    a.modifiers == b.modifiers && a.key == b.key
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: &str, scope: Scope, action: Action) -> Binding {
        Binding { trigger: Trigger::Key(Keystroke::parse(key).unwrap()), scope, action }
    }

    fn send(text: &str) -> Action {
        Action::Send(text.to_owned())
    }

    fn pressed(key: &str) -> Keystroke {
        Keystroke::parse(key).unwrap()
    }

    fn mouse(kind: MouseKind, button: MouseButton, modifiers: &str) -> MouseTrigger {
        let mods = if modifiers.is_empty() { Modifiers::default() } else { Keystroke::parse(&format!("{modifiers}-a")).unwrap().modifiers };
        MouseTrigger { kind, button, streak: 1, mods, mouse_reporting: false }
    }

    fn click(scope: Scope, trigger: MouseTrigger, action: Action) -> Binding {
        Binding { trigger: Trigger::Mouse(trigger), scope, action }
    }

    #[test]
    fn should_find_terminal_binding_before_window_binding() {
        // given
        let keymap = Keymap::new(vec![key("shift-enter", Scope::Window, send("window")), key("shift-enter", Scope::Terminal, send("terminal"))]).unwrap();

        // when
        let found = keymap.find_key(&pressed("shift-enter"), Place::Terminal);

        // then
        assert_eq!(found, Some(&send("terminal")));
    }

    #[test]
    fn should_find_editor_binding_before_window_binding() {
        // given
        let keymap = Keymap::new(vec![key("ctrl-c", Scope::Window, send("window")), key("ctrl-c", Scope::Editor, send("editor"))]).unwrap();

        // when
        let found = keymap.find_key(&pressed("ctrl-c"), Place::Editor);

        // then
        assert_eq!(found, Some(&send("editor")));
    }

    #[test]
    fn should_not_find_terminal_binding_elsewhere() {
        // given
        let keymap = Keymap::new(vec![key("shift-enter", Scope::Terminal, send("terminal"))]).unwrap();

        // when
        let found = keymap.find_key(&pressed("shift-enter"), Place::Elsewhere);

        // then
        assert_eq!(found, None);
    }

    #[test]
    fn should_reject_same_trigger_twice_in_one_scope() {
        // given
        let bindings = vec![key("shift-enter", Scope::Terminal, send("a")), key("shift-enter", Scope::Terminal, send("b"))];

        // when
        let keymap = Keymap::new(bindings);

        // then
        assert!(keymap.unwrap_err().contains("shift-enter"));
    }

    #[test]
    fn should_replace_default_with_user_binding_of_same_trigger() {
        // given
        let defaults = Keymap::new(vec![key("ctrl-shift-c", Scope::Terminal, Action::Builtin(Builtin::CopyTo(ClipboardTarget::Clipboard)))]).unwrap();
        let user = Keymap::new(vec![key("ctrl-shift-c", Scope::Terminal, send("x"))]).unwrap();

        // when
        let keymap = Keymap::merged(defaults, user);

        // then
        assert_eq!(keymap.find_key(&pressed("ctrl-shift-c"), Place::Terminal), Some(&send("x")));
        assert_eq!(keymap.len(), 1);
    }

    #[test]
    fn should_disable_default() {
        // given
        let defaults = Keymap::new(vec![key("alt-f12", Scope::Window, Action::Builtin(Builtin::OpenTerminal))]).unwrap();
        let user = Keymap::new(vec![key("alt-f12", Scope::Window, Action::Disable)]).unwrap();

        // when
        let keymap = Keymap::merged(defaults, user);

        // then
        assert_eq!(keymap.find_key(&pressed("alt-f12"), Place::Elsewhere), None);
        assert_eq!(keymap.len(), 0);
    }

    #[test]
    fn should_fall_back_to_none_mods_for_mouse() {
        // given
        let activate = Action::Builtin(Builtin::ActivateTab);
        let keymap = Keymap::new(vec![click(Scope::Tab, mouse(MouseKind::Up, MouseButton::Left, ""), activate.clone())]).unwrap();

        // when
        let found = keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Left, "ctrl"));

        // then
        assert_eq!(found, Some(&activate));
    }

    #[test]
    fn should_prefer_exact_mods_for_mouse() {
        // given
        let (activate, close) = (Action::Builtin(Builtin::ActivateTab), Action::Builtin(Builtin::CloseTab));
        let keymap = Keymap::new(vec![
            click(Scope::Tab, mouse(MouseKind::Up, MouseButton::Left, ""), activate.clone()),
            click(Scope::Tab, mouse(MouseKind::Up, MouseButton::Left, "ctrl"), close.clone()),
        ])
        .unwrap();

        // when
        let with_ctrl = keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Left, "ctrl"));
        let plain = keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Left, ""));

        // then
        assert_eq!((with_ctrl, plain), (Some(&close), Some(&activate)));
    }

    #[test]
    fn should_not_fall_back_past_a_disabled_exact_binding() {
        // given
        let keymap = Keymap::new(vec![
            click(Scope::Tab, mouse(MouseKind::Up, MouseButton::Left, ""), Action::Builtin(Builtin::ActivateTab)),
            click(Scope::Tab, mouse(MouseKind::Up, MouseButton::Left, "ctrl"), Action::Disable),
        ])
        .unwrap();

        // when
        let found = keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Left, "ctrl"));

        // then
        assert_eq!(found, None);
    }

    #[test]
    fn should_keep_reporting_and_local_mouse_bindings_apart() {
        // given
        let reporting = MouseTrigger { mouse_reporting: true, ..mouse(MouseKind::Up, MouseButton::Middle, "") };
        let keymap = Keymap::new(vec![click(Scope::Terminal, reporting, Action::Builtin(Builtin::Nop))]).unwrap();

        // when
        let found = keymap.find_mouse(Scope::Terminal, &mouse(MouseKind::Up, MouseButton::Middle, ""));

        // then
        assert_eq!(found, None);
    }

    #[test]
    fn should_not_find_mouse_binding_of_another_scope() {
        // given
        let keymap = Keymap::new(vec![click(Scope::Tree, mouse(MouseKind::Up, MouseButton::Left, ""), Action::Builtin(Builtin::ActivateRow))]).unwrap();

        // when
        let found = keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Left, ""));

        // then
        assert_eq!(found, None);
    }

    #[test]
    fn should_limit_builtins_to_their_scopes() {
        // when
        let supported = [
            Builtin::ActivateRow.supports(Scope::Tab),
            Builtin::ActivateRow.supports(Scope::Tree),
            Builtin::CloseTab.supports(Scope::Window),
            Builtin::OpenSettings.supports(Scope::Terminal),
            Builtin::ToggleChangesLayout.supports(Scope::Window),
            Builtin::ToggleChangesLayout.supports(Scope::Tree),
            Builtin::CopyFullPath.supports(Scope::Tree),
            Builtin::CopyFullPath.supports(Scope::Hub),
            Builtin::CopyFullPath.supports(Scope::Window),
        ];

        // then
        assert_eq!(supported, [false, true, true, false, true, false, true, true, false]);
    }

    #[test]
    fn should_limit_keeping_actions_to_their_scopes() {
        // when
        let supported = [
            Builtin::KeepTab.supports(Scope::Tab),
            Builtin::KeepTab.supports(Scope::Window),
            Builtin::KeepTab.supports(Scope::Tree),
            Builtin::ActivateRowToKeep.supports(Scope::Tree),
            Builtin::ActivateRowToKeep.supports(Scope::Tab),
        ];

        // then
        assert_eq!(supported, [true, true, false, true, false]);
    }

    #[test]
    fn should_name_bound_function_trigger() {
        // given
        let keymap = Keymap::new(vec![key("ctrl-alt-r", Scope::Window, Action::Invoke { generation: 1, function: 0 })]).unwrap();

        // when
        let named = keymap.invoked(0);

        // then
        assert_eq!(named.as_deref(), Some("ctrl-alt-r"));
    }
}
