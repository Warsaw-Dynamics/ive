//! Declared items into the lines a menu shows for a subject: items that do not apply left out, built-ins that list
//! choices (terminals, run configurations) as submenus, items with a predicate pending until it answers.

use super::Subject;
use super::when::described;
use crate::script::keymap::Builtin;
use crate::script::menus::{ItemAction, Label, MenuItem, Test, When};

/// A declared item's place: its index in its menu, then in each submenu down to it. Stays while lines around it come
/// and go, so the highlight and pending answers follow the item, not its position.
pub type ItemId = Vec<usize>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineState {
    Ready,
    /// its predicate has not answered: drawn faded, not chosen
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineAction {
    Script(ItemAction),
    /// one of a built-in's choices, e.g. a terminal of Send to Terminal by its process id
    Choice { builtin: Builtin, choice: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Item { id: ItemId, label: String, action: LineAction, state: LineState, enabled: bool },
    Submenu { id: ItemId, label: String, lines: Vec<Line>, state: LineState },
    Separator,
}

impl Line {
    pub fn id(&self) -> Option<&ItemId> {
        match self {
            Self::Item { id, .. } | Self::Submenu { id, .. } => Some(id),
            Self::Separator => None,
        }
    }
}

/// A line of a built-in's submenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    Item { label: String, choice: String, enabled: bool },
    Separator,
}

/// What a built-in does for a subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applies {
    Hidden,
    Shown { enabled: bool },
    /// a submenu of these
    Choices(Vec<Choice>),
}

/// A predicate to run before its line can be chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTest {
    pub id: ItemId,
    pub generation: u64,
    pub function: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub lines: Vec<Line>,
    pub tests: Vec<PendingTest>,
}

/// The lines `items` show for `subject`, `applies` telling what each built-in does for it, and the predicates to ask.
pub fn resolve(items: &[MenuItem], subject: &Subject, applies: &dyn Fn(Builtin) -> Applies) -> Resolved {
    let mut tests = Vec::new();
    let lines = Resolver { subject, applies, tests: &mut tests }.lines(items, &[]);
    Resolved { lines: pruned(lines), tests }
}

/// `lines` once item `id`'s predicate answered: ready where it is, or gone.
pub fn answered(lines: Vec<Line>, id: &ItemId, answer: bool) -> Vec<Line> {
    let changed = lines.into_iter().filter_map(|line| match line {
        Line::Item { id: line_id, state: LineState::Pending, .. } if line_id == *id && !answer => None,
        Line::Submenu { id: line_id, state: LineState::Pending, .. } if line_id == *id && !answer => None,
        Line::Item { id: line_id, label, action, enabled, .. } if line_id == *id => Some(Line::Item { id: line_id, label, action, state: LineState::Ready, enabled }),
        Line::Submenu { id: line_id, label, lines, .. } if line_id == *id => Some(Line::Submenu { id: line_id, label, lines, state: LineState::Ready }),
        Line::Submenu { id: line_id, label, lines, state } if id.starts_with(&line_id) => Some(Line::Submenu { id: line_id, label, lines: answered(lines, id, answer), state }),
        other => Some(other),
    });
    pruned(changed.collect())
}

/// The default label of a built-in item.
pub fn label(builtin: Builtin) -> &'static str {
    match builtin {
        Builtin::KeepTab => "Keep Tab",
        Builtin::StopProgram => "Stop",
        Builtin::KillProgram => "Kill",
        Builtin::CloseTab => "Close",
        Builtin::CloseOtherTabs => "Close Other Tabs",
        Builtin::CloseTabsToTheLeft => "Close Tabs to the Left",
        Builtin::CloseTabsToTheRight => "Close Tabs to the Right",
        Builtin::SendPathsToTerminal => "Send to Terminal",
        Builtin::CopyFullPath => "Copy Absolute Path",
        Builtin::CopyRelativePath => "Copy Relative Path",
        Builtin::CopyFileName => "Copy File Name",
        Builtin::FindInFiles => "Find in Files…",
        Builtin::OpenFindInFiles => "Find in Files…",
        Builtin::OpenTerminalHere => "Open in Terminal",
        Builtin::NewTerminal => "New Terminal",
        Builtin::RevealInFileManager if cfg!(target_os = "macos") => "Reveal in Finder",
        Builtin::RevealInFileManager => "Reveal in Files",
        Builtin::OpenWithDefaultApp => "Open in Default App",
        Builtin::OpenInBrowser => "Open in Browser",
        Builtin::RunFile => "Run",
        Builtin::GitShowDiff => "Show Diff",
        Builtin::GitCompareWithBase => "Compare with Base Branch",
        Builtin::GitHistory => "Show History",
        Builtin::GitIgnore => "Add to .gitignore",
        Builtin::CompareFiles => "Compare Files",
        Builtin::OpenInNewWindow => "Open in New Window",
        Builtin::OpenHub => "Hub",
        Builtin::OpenSettings => "Settings",
        Builtin::OpenTerminal => "New Terminal",
        Builtin::OpenTerminalIn => "New Terminal In…",
        Builtin::ToggleChangesLayout => "Toggle Changes Layout",
        _ => "",
    }
}

struct Resolver<'a> {
    subject: &'a Subject,
    applies: &'a dyn Fn(Builtin) -> Applies,
    tests: &'a mut Vec<PendingTest>,
}

impl Resolver<'_> {
    fn lines(&mut self, items: &[MenuItem], parent: &[usize]) -> Vec<Line> {
        let id = |index: usize| [parent, &[index]].concat();
        items.iter().enumerate().filter_map(|(index, item)| self.line(item, id(index))).collect()
    }

    fn line(&mut self, item: &MenuItem, id: ItemId) -> Option<Line> {
        match item {
            MenuItem::Separator => Some(Line::Separator),
            MenuItem::Submenu { label, items, when } => {
                let state = self.state(when, &id)?;
                let lines = self.lines(items, &id);
                Some(Line::Submenu { id, label: label.clone(), lines, state })
            }
            MenuItem::Action { label: declared, action, when } => {
                let state = self.state(when, &id)?;
                let ItemAction::Builtin(builtin) = action else {
                    return Some(Line::Item { id, label: text(declared, ""), action: LineAction::Script(*action), state, enabled: true });
                };
                let label = text(declared, label(*builtin));
                match (self.applies)(*builtin) {
                    Applies::Hidden => None,
                    Applies::Shown { enabled } => Some(Line::Item { id, label, action: LineAction::Script(*action), state, enabled }),
                    Applies::Choices(choices) => Some(Line::Submenu { lines: choice_lines(*builtin, choices, &id), id, label, state }),
                }
            }
        }
    }

    /// `None` when `when`'s description fails; else pending when it has a predicate, which is then asked.
    fn state(&mut self, when: &When, id: &ItemId) -> Option<LineState> {
        if !described(when, self.subject) {
            return None;
        }
        match when.test {
            Test::Absent => Some(LineState::Ready),
            Test::Function { generation, function } => {
                self.tests.push(PendingTest { id: id.clone(), generation, function });
                Some(LineState::Pending)
            }
        }
    }
}

fn text(label: &Label, default: &str) -> String {
    match label {
        Label::Custom(label) => label.clone(),
        Label::Default => default.to_owned(),
    }
}

/// `choices` of `builtin` as the lines of a submenu at `parent`.
pub fn choice_lines(builtin: Builtin, choices: Vec<Choice>, parent: &[usize]) -> Vec<Line> {
    let line = |(index, choice)| match choice {
        Choice::Separator => Line::Separator,
        Choice::Item { label, choice, enabled } => {
            Line::Item { id: [parent, &[index]].concat(), label, action: LineAction::Choice { builtin, choice }, state: LineState::Ready, enabled }
        }
    };
    choices.into_iter().enumerate().map(line).collect()
}

/// Without submenus left empty, then without separators at either end or next to another.
fn pruned(lines: Vec<Line>) -> Vec<Line> {
    let kept = lines.into_iter().filter_map(|line| match line {
        Line::Submenu { id, label, lines, state } => {
            let lines = pruned(lines);
            (!lines.is_empty()).then_some(Line::Submenu { id, label, lines, state })
        }
        other => Some(other),
    });
    let mut result: Vec<Line> = Vec::new();
    for line in kept {
        let dangling = line == Line::Separator && result.last().is_none_or(|last| *last == Line::Separator);
        if !dangling {
            result.push(line);
        }
    }
    if result.last() == Some(&Line::Separator) {
        result.pop();
    }
    result
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::menu::{Home, PathKind, SubjectPath, Within};
    use crate::script::menus::KindIs;

    fn folder() -> Subject {
        let path = SubjectPath { path: PathBuf::from("/w/src"), within: Within::Checkout("src".to_owned()), kind: PathKind::Folder };
        Subject { paths: vec![path], home: Home::Checkout(PathBuf::from("/w")) }
    }

    fn builtin(builtin: Builtin) -> MenuItem {
        MenuItem::Action { label: Label::Default, action: ItemAction::Builtin(builtin), when: When::default() }
    }

    fn mine(label: &str, function: usize, when: When) -> MenuItem {
        MenuItem::Action { label: Label::Custom(label.to_owned()), action: ItemAction::Invoke { generation: 1, function }, when }
    }

    fn tested(function: usize) -> When {
        When { test: Test::Function { generation: 1, function }, ..When::default() }
    }

    fn shown(_: Builtin) -> Applies {
        Applies::Shown { enabled: true }
    }

    fn labels(lines: &[Line]) -> Vec<String> {
        lines
            .iter()
            .map(|line| match line {
                Line::Item { label, state: LineState::Pending, .. } => format!("{label}…"),
                Line::Item { label, .. } => label.clone(),
                Line::Submenu { label, lines, .. } => format!("{label} [{}]", labels(lines).join(", ")),
                Line::Separator => "-".to_owned(),
            })
            .collect()
    }

    #[test]
    fn should_show_two_same_label_items_by_kind() {
        // given
        let items = [
            mine("Open in IntelliJ", 0, When { kind: KindIs::File, ..When::default() }),
            mine("Open in IntelliJ", 1, When { kind: KindIs::Folder, ..When::default() }),
        ];

        // when
        let resolved = resolve(&items, &folder(), &shown);

        // then
        assert!(matches!(&resolved.lines[..], [Line::Item { action: LineAction::Script(ItemAction::Invoke { function: 1, .. }), .. }]), "{:?}", resolved.lines);
    }

    #[test]
    fn should_leave_out_hidden_builtins_and_dangling_separators() {
        // given
        let items = [builtin(Builtin::KeepTab), MenuItem::Separator, builtin(Builtin::CopyFullPath), MenuItem::Separator, MenuItem::Separator, builtin(Builtin::CloseTab), MenuItem::Separator];
        let applies = |builtin| if builtin == Builtin::KeepTab { Applies::Hidden } else { Applies::Shown { enabled: true } };

        // when
        let resolved = resolve(&items, &folder(), &applies);

        // then
        assert_eq!(labels(&resolved.lines), ["Copy Absolute Path", "-", "Close"]);
    }

    #[test]
    fn should_drop_a_submenu_left_empty() {
        // given
        let items = [MenuItem::Submenu { label: "Git".to_owned(), items: vec![builtin(Builtin::GitHistory)], when: When::default() }, builtin(Builtin::CloseTab)];
        let applies = |builtin| if builtin == Builtin::GitHistory { Applies::Hidden } else { Applies::Shown { enabled: true } };

        // when
        let resolved = resolve(&items, &folder(), &applies);

        // then
        assert_eq!(labels(&resolved.lines), ["Close"]);
    }

    #[test]
    fn should_list_choices_of_a_builtin_as_a_submenu() {
        // given
        let items = [builtin(Builtin::SendPathsToTerminal)];
        let choices = |_| Applies::Choices(vec![Choice::Item { label: "zsh · main".to_owned(), choice: "7".to_owned(), enabled: true }, Choice::Separator]);

        // when
        let resolved = resolve(&items, &folder(), &choices);

        // then
        assert_eq!(labels(&resolved.lines), ["Send to Terminal [zsh · main]"]);
        let Line::Submenu { lines, .. } = &resolved.lines[0] else { panic!() };
        assert!(matches!(&lines[0], Line::Item { id, action: LineAction::Choice { builtin: Builtin::SendPathsToTerminal, choice }, .. } if id == &vec![0, 0] && choice == "7"));
    }

    #[test]
    fn should_list_a_test_as_pending_and_ask_for_it() {
        // when
        let resolved = resolve(&[mine("Deploy", 0, tested(3))], &folder(), &shown);

        // then
        assert_eq!(labels(&resolved.lines), ["Deploy…"]);
        assert_eq!(resolved.tests, [PendingTest { id: vec![0], generation: 1, function: 3 }]);
    }

    #[test]
    fn should_not_ask_a_test_whose_description_fails() {
        // given
        let when = When { kind: KindIs::File, ..tested(3) };

        // when
        let resolved = resolve(&[mine("Deploy", 0, when)], &folder(), &shown);

        // then
        assert!(resolved.lines.is_empty() && resolved.tests.is_empty());
    }

    #[test]
    fn should_remove_a_pending_line_answered_false_and_its_dangling_separator() {
        // given
        let items = [builtin(Builtin::CopyFullPath), MenuItem::Separator, mine("Deploy", 0, tested(3))];
        let resolved = resolve(&items, &folder(), &shown);

        // when
        let lines = answered(resolved.lines, &vec![2], false);

        // then
        assert_eq!(labels(&lines), ["Copy Absolute Path"]);
    }

    #[test]
    fn should_make_a_pending_line_ready_answered_true() {
        // given
        let resolved = resolve(&[mine("Deploy", 0, tested(3))], &folder(), &shown);

        // when
        let lines = answered(resolved.lines, &vec![0], true);

        // then
        assert_eq!(labels(&lines), ["Deploy"]);
    }

    #[test]
    fn should_keep_a_pending_submenu_while_any_item_is_pending() {
        // given
        let items = [MenuItem::Submenu { label: "Deploy".to_owned(), items: vec![mine("Staging", 0, tested(3)), mine("Production", 1, tested(4))], when: When::default() }];
        let resolved = resolve(&items, &folder(), &shown);

        // when
        let one_answered = answered(resolved.lines, &vec![0, 0], false);
        let none_left = answered(one_answered.clone(), &vec![0, 1], false);

        // then
        assert_eq!(labels(&one_answered), ["Deploy [Production…]"]);
        assert!(none_left.is_empty());
    }

    #[test]
    fn should_use_a_custom_label_for_a_builtin() {
        // given
        let items = [MenuItem::Action { label: Label::Custom("Path".to_owned()), action: ItemAction::Builtin(Builtin::CopyFullPath), when: When::default() }];

        // when
        let resolved = resolve(&items, &folder(), &shown);

        // then
        assert_eq!(labels(&resolved.lines), ["Path"]);
    }
}
