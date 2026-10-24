//! Drags over tree rows: the hint under the pointer, and what a drop of processes does (files: `transfers`).

use std::path::PathBuf;

use gpui::{Context, Modifiers, Window};

use super::Workspace;
use super::process_drop::{Dragged, Held, Labelled, ProcessDrop, plan_process_drop};
use crate::process::{ProcessId, ProcessKind, RunId};
use crate::project::{CheckoutRef, ProjectId};
use crate::run::values_tab::RunRequest;
use crate::sidebar::drop_place::{DropHint, processes_place};
use crate::sidebar::rows::{TreeKey, TreeRow};
use crate::sidebar::selection;
use crate::transfer::{DropModifiers, Verb};

/// MCP's `drop_rows`, by the rows' names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowsDrop {
    pub rows: Vec<String>,
    pub onto: String,
    pub force: Option<Verb>,
}

/// Where a terminal running `claude` turns into Claude Code.
const CLAUDE_COMMAND: &str = "claude\r";

/// The row a `drop_rows` request names: `entry:/r/src`, `process:3`, as its `tree-menu:` target is named.
pub fn row_named(rows: &[TreeRow], name: &str) -> Option<TreeKey> {
    rows.iter().find(|row| row.target_names().first().is_some_and(|first| first == name)).map(TreeRow::key)
}

/// What is being dragged: the pressed row, whose picks go with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Dragging {
    Files(PathBuf),
    Processes(TreeKey),
}

impl Workspace {
    /// A drag over the row of `row`: the hint the chip shows, and whether the row is drawn as the drop row.
    pub(crate) fn hover_drop(&mut self, dragging: Dragging, row: TreeKey, window: &Window, cx: &mut Context<Self>) {
        let hint = match &dragging {
            Dragging::Files(pressed) => self.files_drop_hint(pressed, &row, &window.modifiers(), cx),
            Dragging::Processes(pressed) => self.process_drop_hint(pressed, &row, &window.modifiers(), cx),
        };
        if self.tree.drop_hint.as_ref() != Some(&hint) {
            self.tree.drop_hint = Some(hint);
            cx.notify();
        }
    }

    fn process_drop_hint(&self, pressed: &TreeKey, row: &TreeKey, modifiers: &Modifiers, cx: &Context<Self>) -> DropHint {
        match self.process_drop(pressed, row, modifiers, cx) {
            Some((drop, text, _)) => DropHint { row: row.clone(), text, allowed: drop != ProcessDrop::Refused },
            None => DropHint { row: row.clone(), text: "Drop on a worktree or a project".to_owned(), allowed: false },
        }
    }

    /// Processes dropped on the row of `onto`: terminals and Claude Code move there, or with the copy modifier the
    /// same things start there; then the hint goes.
    pub(crate) fn drop_processes(&mut self, (pressed, onto): (&TreeKey, TreeKey), modifiers: &Modifiers, window: &mut Window, cx: &mut Context<Self>) {
        self.tree.drop_hint = None;
        let planned = self.process_drop(pressed, &onto, modifiers, cx);
        match planned {
            Some((ProcessDrop::Move(ids), _, target)) => self.move_processes(&ids, &target.checkout, cx),
            Some((ProcessDrop::Start(ids), _, target)) => self.start_copies(&ids, &target.checkout, window, cx),
            Some((ProcessDrop::Refused, ..)) | None => cx.notify(),
        }
    }

    /// What dropping the drag of `pressed` on the row of `row` does, with `modifiers` held; `None` when the row takes
    /// no processes.
    fn process_drop(&self, pressed: &TreeKey, row: &TreeKey, modifiers: &Modifiers, cx: &Context<Self>) -> Option<(ProcessDrop, String, Labelled)> {
        let item = &self.tree.rows.iter().find(|listed| listed.key() == *row)?.item;
        let checkout = processes_place(item, |project| self.main_of(project))?;
        let target = Labelled { label: self.checkout_label(&checkout), checkout };
        let configured = cx.try_global::<DropModifiers>().copied().unwrap_or_default();
        let held = Held { forced: configured.forced(modifiers), copy_name: configured.copy_name() };
        let (drop, text) = plan_process_drop(&self.dragged_processes(pressed, &target.checkout), &target, &held);
        Some((drop, text, target))
    }

    /// The processes a drag of the row of `pressed` carries, as the drop sees them.
    fn dragged_processes(&self, pressed: &TreeKey, target: &CheckoutRef) -> Vec<Dragged> {
        let keys = self.tree.selection.dragged_keys(pressed, &selection::listed(&self.tree.rows));
        let id = |key: &TreeKey| match key {
            TreeKey::Process(id) | TreeKey::LiveRun(RunId::Process(id)) => Some(*id),
            _ => None,
        };
        let dragged = |id: ProcessId| {
            let process = self.processes.get(id)?;
            let config_in_target = process.run.as_ref().is_some_and(|origin| {
                let checkout = process.checkout.as_ref().map(|checkout| checkout.path.as_path());
                let relative = checkout.and_then(|checkout| origin.folder.strip_prefix(checkout).ok());
                relative.is_some_and(|relative| target.path.join(relative).join(crate::run::file::RUN_FILE).is_file())
            });
            Some(Dragged { id, kind: process.kind, name: process.title(), checkout: process.checkout.clone(), config_in_target })
        };
        keys.iter().filter_map(id).filter_map(dragged).collect()
    }

    /// MCP's `drop_rows`: `rows` picked and dragged (the first pressed) onto `onto`, as if `force`'s modifier were
    /// held; the drop's hint, or why it was refused.
    pub(super) fn drop_rows(&mut self, request: RowsDrop, window: &mut Window, cx: &mut Context<Self>) -> Result<serde_json::Value, String> {
        let RowsDrop { rows, onto, force } = request;
        let named = |name: &String| row_named(&self.tree.rows, name).ok_or_else(|| format!("no tree row {name:?} on screen; see list_targets (tree-menu:<row>)"));
        let keys = rows.iter().map(named).collect::<Result<Vec<_>, _>>()?;
        let onto = named(&onto)?;
        let Some((pressed, others)) = keys.split_first() else {
            return Err("drop_rows needs at least one row".to_owned());
        };
        let configured = cx.try_global::<DropModifiers>().copied().unwrap_or_default();
        let modifiers = match force {
            Some(Verb::Copy) => configured.copy,
            Some(Verb::Move) => configured.moving,
            None => Modifiers::default(),
        };
        let listed = selection::listed(&self.tree.rows);
        self.tree.selection.select_only(pressed.clone());
        for key in others {
            self.tree.selection.toggle(key.clone(), &listed);
        }
        let hint = match pressed {
            TreeKey::Entry(path) => self.files_drop_hint(path, &onto, &modifiers, cx),
            _ => self.process_drop_hint(pressed, &onto, &modifiers, cx),
        };
        if !hint.allowed {
            return Err(hint.text);
        }
        match pressed {
            TreeKey::Entry(path) => self.drop_files((&path.clone(), onto), &modifiers, window, cx),
            _ => self.drop_processes((&pressed.clone(), onto), &modifiers, window, cx),
        }
        Ok(serde_json::json!(hint.text))
    }

    pub(super) fn main_of(&self, project: &ProjectId) -> Option<CheckoutRef> {
        let main = self.project(project)?.main_checkout();
        Some(CheckoutRef { project: project.clone(), path: main.path.clone() })
    }

    /// Lists `ids` under `checkout`, pinned to their directories, picks them and unfolds the checkout's Processes.
    fn move_processes(&mut self, ids: &[ProcessId], checkout: &CheckoutRef, cx: &mut Context<Self>) {
        let moved: Vec<ProcessId> = ids.iter().copied().filter(|id| self.processes.move_pinned(*id, checkout.clone())).collect();
        self.tree_changed(cx);
        self.reveal_processes(&moved, cx);
        self.schedule_save(cx);
    }

    /// Starts in `checkout` what `ids` run: a terminal, Claude Code (a terminal typing `claude`), or the checkout's own
    /// copy of a run's configuration.
    fn start_copies(&mut self, ids: &[ProcessId], checkout: &CheckoutRef, window: &mut Window, cx: &mut Context<Self>) {
        for id in ids {
            let Some(process) = self.processes.get(*id) else { continue };
            let (kind, run) = (process.kind, process.run.clone());
            let source = process.checkout.clone();
            match (kind, run, source) {
                (ProcessKind::Run, Some(origin), Some(source)) => {
                    let Ok(relative) = origin.folder.strip_prefix(&source.path) else { continue };
                    let request = RunRequest { checkout: checkout.clone(), folder: checkout.path.join(relative), key: origin.key.clone(), show: crate::run::values_tab::ShowRun::Tab };
                    self.request_run(request, window, cx);
                }
                (ProcessKind::ClaudeCode, ..) => {
                    let tab = self.open_terminal_in(Some(checkout.clone()), window, cx);
                    let _ = self.send_text_to_tab(tab, CLAUDE_COMMAND, cx);
                }
                _ => {
                    self.open_terminal_in(Some(checkout.clone()), window, cx);
                }
            }
        }
    }

    /// Unfolds the tree down to the first of `ids` and picks them all.
    fn reveal_processes(&mut self, ids: &[ProcessId], cx: &mut Context<Self>) {
        let Some((first, rest)) = ids.split_first() else {
            return cx.notify();
        };
        self.locating = Some(TreeKey::Process(*first));
        self.continue_locating(cx);
        let listed = selection::listed(&self.tree.rows);
        for id in rest {
            self.tree.selection.toggle(TreeKey::Process(*id), &listed);
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::file_tree::{Entry, EntryKind, RunState};
    use crate::terminal::Progress;
    use crate::sidebar::rows::{EntryRow, ProcessRow, TreeItem, TreeRow};
    use crate::sidebar::states::EntryState;

    fn rows() -> Vec<TreeRow> {
        let entry = Entry { name: "a.rs".to_owned(), path: PathBuf::from("/p/a.rs"), kind: EntryKind::File, run: RunState::None };
        let checkout = CheckoutRef { project: ProjectId::new("p"), path: PathBuf::from("/p") };
        let file = TreeItem::Entry(EntryRow { checkout, entry, expanded: false, warning: None, run_file: false, state: EntryState::Plain, mark: None });
        let process = TreeItem::Process(ProcessRow { id: ProcessId::for_tests(3), kind: ProcessKind::Terminal, title: "zsh".to_owned(), progress: Progress::None, attention: None });
        vec![TreeRow { depth: 1, item: file, target: "/p/a.rs".to_owned() }, TreeRow { depth: 1, item: process, target: "3".to_owned() }]
    }

    #[test]
    fn should_find_rows_by_their_menu_names() {
        // when
        let found = ["entry:/p/a.rs", "process:3", "entry:/p/gone.rs"].map(|name| row_named(&rows(), name));

        // then
        assert_eq!(found, [Some(TreeKey::Entry(PathBuf::from("/p/a.rs"))), Some(TreeKey::Process(ProcessId::for_tests(3))), None]);
    }
}
