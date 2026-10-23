# Drag and drop, copy and paste in the tree: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move terminals, and copy or move files and folders, between any worktrees and projects by dragging tree rows
or with Ctrl+C / Ctrl+V, asking about conflicts in a Conflicts tab before anything is written.

**Architecture:** Pure planning (where things go, what conflicts, names, hints) in `src/transfer/` and
`src/sidebar/drop_place.rs`, unit-tested without a file system; disk work (walk, classify, write) in
`src/transfer/{scan,write}.rs`, tested against temp folders and run off the UI thread; gpui glue (drags, drops, menus,
the sidebar line, the Conflicts tab, MCP) in the workspace. Moving a terminal re-files the process and pins it to its
directory; the pin and its checkout are saved so a restart keeps them.

**Tech Stack:** Rust 2024, gpui (pgui fork), Rune scripts (`assets/scripts/ive.rn`), rmcp, `tempfile` in tests.

**Spec:** `docs/superpowers/specs/2026-10-23-drag-and-drop-design.md` (mock: `mock/checkouts.html`, `mock/dnd.js`).

## Global Constraints

- AGENTS.md's coding guideline: tests `should_…` with `// given` / `// when` / `// then`; no boolean parameters; ≤ 3
  parameters; self-contained error messages naming the value.
- Gates for every task: `cargo test`, `cargo clippy --all-targets` (no warnings), `cargo build --release` at the end.
- Bounds: at most 20 000 files and 4 GiB per copy (or move across file systems).
- Drop modifiers are settings: `config.drop_modifiers = #{ force_copy, force_move }` declared in `ive.rn` (Linux/Windows
  `"CTRL"` / `"SHIFT"`, macOS `"ALT"` / `"SUPER"`); no code hard-codes a modifier, hints name the configured keys.
- Texts are the spec's, verbatim: `Already in dev`, `Already here; Ctrl+drop copies`, `Copy refused: 31 204 files, at
  most 20 000`, `A copy is running`, `copies 4, replaces 1, skips 1 · 1 to choose`, `as mock (2).css`, …
- Nothing is overwritten without Replace; temp-name-then-rename for every file written.
- No commit per task: all the work (mock, spec, plan, code) goes in one commit on branch `drag-and-drop` at the end
  (Task 10), message style `area: what changed`, as in `git log`.

## Review Focus

1. A dragged folder containing a nested worktree or `.git` → those are left out, not copied (Task 6 test).
2. A move within one worktree onto a subfolder of a dragged folder → refused, nothing removed (Task 5 test).
3. Two incoming files with one name in a flat drop → `Twice` conflict, Replace not offered (Task 5 test).
4. The target file changes between the Conflicts tab opening and Copy → that file untouched, reported failed (Task 7
   test).
5. A moved terminal and a rescan (`follow_all_directories`) → it stays (Task 2 test).

---

### Task 1: Picking processes, Escape, MCP pick targets

**Files:**
- Modify: `src/sidebar/selection.rs`, `src/sidebar/view.rs` (`RowSelect`, `bind_row`, `process_row`, the run row),
  `src/workspace/menu.rs` (`menu_rows`), `src/script/{config,declare,keymap}.rs`, `src/menu/resolve.rs`,
  `src/workspace/scripts.rs` (`run_window_builtin`), `assets/scripts/ive.rn`, `README.md` (click targets table)

**Interfaces:**
- Produces: `pub enum PickGroup { Files, Processes }`; `Listed { key, pick: Option<PickGroup> }` (replaces `multi`);
  `pub fn pick_group(row: &TreeRow) -> Option<PickGroup>` (Entry File/Folder → Files; Process, LiveRun → Processes);
  `Selection::toggle(&mut self, key: TreeKey, listed: &[Listed])`, `Selection::extend` unchanged signature,
  `Selection::clear(&mut self)`, `Selection::dragged_keys(&self, pressed: &TreeKey, listed: &[Listed]) -> Vec<TreeKey>`
  (`dragged_paths` keeps its signature, built on it); `Builtin::ClearRowSelection` (`act::ClearRowSelection`).

- [ ] **Step 1: Tests in `selection.rs`** (helpers `file`, `other`, plus `process(n)` → `Listed { key: TreeKey::Process(ProcessId(n)), pick: Some(Processes) }`):
  - `should_start_new_selection_when_toggling_other_group`: select file a, toggle process 1 → keys `[process 1]`.
  - `should_range_only_over_rows_of_anchor_group`: rows `[file a, process 1, file b]`, anchor a, extend to b → `[a, b]`.
  - `should_refuse_range_to_row_of_other_group`: anchor file a, extend to process 1 → `false`, nothing changed.
  - `should_clear_picks_and_keep_anchor`: select a, `clear()` → no keys, anchor `Some(a)`.
  - `should_drag_picked_processes_when_pressed_one_is_picked`: toggle process 1, 2 → `dragged_keys(&process 2)` = `[1, 2]`.
  - Existing tests keep passing with `multi` replaced by `pick`.
- [ ] **Step 2:** `cargo test sidebar::selection` → the new tests fail to compile.
- [ ] **Step 3: Implement** in `selection.rs`. `RowSelect.multi` becomes `pick: Option<PickGroup>`; `bind_row`'s
  Toggle/Extend arms apply when `pick.is_some()`. `process_row` and the run row get `pick_group(row)` so they are
  pickable. `menu_rows` keeps returning only `Entry` keys for menus (processes do not join a path menu).
- [ ] **Step 4: Escape.** Add `"ClearRowSelection"` to `UNIT_ACTIONS` (count 62), `declare.rs` →
  `Builtin::ClearRowSelection`, `supports` → `Scope::Tree`, label in `resolve.rs` "Clear Selection";
  `run_window_builtin` → `self.change_selection(|s, _| s.clear())` + `cx.notify()`. `ive.rn`:
  `#{ key: "Escape", action: act::ClearRowSelection, scope: "tree" }`. Test in `script/engine.rs` style:
  `should_bind_escape_to_clear_row_selection_in_tree` (the default keymap has that binding).
- [ ] **Step 5: MCP targets.** In `bind_row`, next to `tree-menu:{name}`, register `tree-pick:{name}` (toggle, as
  Ctrl+click) and `tree-pick-range:{name}` (extend; falls back to a click as the mouse does). README's click target
  table lists both.
- [ ] **Step 6:** `cargo test && cargo clippy --all-targets` → pass, no warnings.

### Task 2: Moved processes stay until their directory changes

**Files:** Modify `src/process.rs`, `src/workspace/follow.rs`

**Interfaces:**
- Produces: `Process::pinned(&self) -> Option<&Path>`; `Processes::move_pinned(&mut self, id: ProcessId, checkout: CheckoutRef) -> bool`
  (terminal or Claude Code only; sets checkout and pins the current directory; false for a run or the same
  checkout); `Processes::set_pinned(&mut self, id: ProcessId, directory: PathBuf)` (restore);
  `set_directory` clears the pin when the new directory differs from it.

- [ ] **Step 1: Tests** (`follow.rs` tests module, its helpers `terminal_in`, `checkout_of`):
  - `should_keep_moved_process_while_directory_unchanged`: terminal in `/a` at directory `/a/src`, `move_pinned` to
    `/a/wt` → `moves(...)` with `/a/wt` known is empty.
  - `should_follow_moved_process_once_directory_changes`: as above, then `set_directory(/a/src/x)` → moves to `/a`.
  - `should_keep_moved_process_when_directory_leaves_every_checkout`: moved, `set_directory(/tmp)` → `moves` empty and
    `pinned()` is `None`.
  - `process.rs`: `should_refuse_to_move_pin_a_run` (`move_pinned` on a run → false, checkout unchanged).
- [ ] **Step 2:** `cargo test follow process::` → fail to compile.
- [ ] **Step 3: Implement.** Field `pinned: Option<PathBuf>` on `Process` (None at start); `moves` skips a process
  whose `pinned() == directory()`.
- [ ] **Step 4:** `cargo test && cargo clippy --all-targets` → pass.

### Task 3: Moves survive a restart

**Files:** Modify `src/state.rs`, `src/workspace/persist.rs` (`saved_state`), `src/workspace/processes.rs`
(`adopt_sessions`, new `checkout_from_record`), `src/workspace.rs` (the `adopt_sessions` call)

**Interfaces:**
- Consumes: Task 2's `pinned`, `set_pinned`.
- Produces: `pub struct SavedTerminal { pub session: String, pub project: String, pub checkout: PathBuf, #[serde(default, skip_serializing_if = "Option::is_none")] pub pinned: Option<PathBuf> }`;
  `SavedState.terminals: Vec<SavedTerminal>` (`#[serde(default, skip_serializing_if = "Vec::is_empty")]`);
  `fn checkout_from_record(record: &SavedTerminal, is_open: impl Fn(&ProjectId) -> bool) -> Option<CheckoutRef>`
  (Some when the project is open and `checkout.is_dir()`); `adopt_sessions(&mut self, tabs: AdoptedTabs, records: &[SavedTerminal], window, cx)`.

- [ ] **Step 1: Tests:**
  - `state.rs`: `should_round_trip_saved_terminals` (save/load a state with one pinned record, equal after).
  - `processes.rs`: `should_place_session_by_record` (tempdir as checkout, project open → that checkout);
    `should_ignore_record_of_closed_project`; `should_ignore_record_whose_folder_is_gone`.
- [ ] **Step 2:** run → fail. **Step 3: Implement:** `saved_state` records every process with `view.hosted_id()` and
  a checkout; `adopt_sessions` looks a record up by session id, prefers `checkout_from_record`, else the labels as
  today, and calls `set_pinned` with the record's pin.
- [ ] **Step 4:** `cargo test && cargo clippy --all-targets` → pass.

### Task 4: Dragging processes onto worktrees and projects

**Files:**
- Create: `src/sidebar/drop_place.rs`, `src/transfer/mod.rs` (only `Verb` and `forced` here), `src/workspace/process_drop.rs`
- Modify: `src/dragged_files.rs` (add `DraggedProcesses`), `src/sidebar/view.rs` (drag on process/run rows, `on_drop`,
  `drag_over` highlight, `on_drag_move` hint on every row), `src/theme_config/mod.rs` + `assets/theme/default.toml`
  (`tree_states.drop` in dark, light and both unfocused), `src/workspace.rs` (field `drop_hint: Option<DropHint>`)

**Interfaces:**
- Produces:
  - `pub struct DropPlace { pub checkout: CheckoutRef, pub folder: PathBuf }`;
    `pub fn files_place(item: &TreeItem, main_of: impl Fn(&ProjectId) -> Option<CheckoutRef>) -> Option<DropPlace>`
    (rules: the spec's table); `pub fn processes_place(item: &TreeItem, main_of: …) -> Option<CheckoutRef>`.
  - `pub enum Verb { Copy, Move }`; `pub struct DropModifiers { pub copy: Modifiers, pub moving: Modifiers }`
    (from the script config, see Step 0); `DropModifiers::forced(&self, held: &Modifiers) -> Option<Verb>` (exact
    match); `DropModifiers::copy_name(&self) -> String` (`Ctrl`, `Option`, `Ctrl+Alt`: macOS names Option/Cmd).
    Hint functions take the copy name as their text's key name.
  - `pub struct Dragged { pub id: ProcessId, pub kind: ProcessKind, pub checkout: Option<CheckoutRef>, pub config_in_target: bool }`;
    `pub enum ProcessDrop { Move(Vec<ProcessId>), Start { started: Vec<ProcessId> }, Refused }`;
    `pub struct Labelled { pub checkout: CheckoutRef, pub label: String }` (the label the tree shows);
    `pub fn plan_process_drop(dragged: &[Dragged], target: &Labelled, forced: Option<Verb>) -> (ProcessDrop, String)`
    (hint texts from the spec).
  - `pub struct DraggedProcesses { pub pressed: TreeKey, pub workspace: WeakEntity<Workspace> }` with `chip`.
  - `pub struct DropHint { pub row: TreeKey, pub text: String, pub allowed: bool }`;
    `Workspace::drop_processes(&mut self, dragged: &DraggedProcesses, onto: TreeKey, forced: Option<Verb>, window, cx)`.

- [ ] **Step 0: `config.drop_modifiers`.** As `config.quit_countdown` / `config.open_folders` are carried
  (`src/script/config.rs` `Carried`, `state.rs` `ScriptEvent::Loaded`, a parser module `src/script/drop_modifiers.rs`
  using `declare::parse_mods`): `ive.rn` declares
  `config.drop_modifiers = if ive::os() == "macos" { #{ copy: "ALT", move: "SUPER" } } else { #{ copy: "CTRL", move: "SHIFT" } };`
  with a comment. Tests: `should_read_drop_modifiers`, `should_refuse_drop_modifier_none`
  (`config.drop_modifiers.copy: "NONE" is no modifier`), `should_refuse_same_copy_and_move`
  (`config.drop_modifiers.move: "CTRL" is the same as copy`), `should_refuse_unknown_drop_modifier_key`. The
  workspace keeps the loaded value as it does `quit_countdown`.
- [ ] **Step 1: Tests:**
  - `drop_place.rs`: `should_drop_files_into_folder_row`, `should_drop_files_on_file_into_its_folder`,
    `should_drop_files_on_project_into_main_checkout_root`, `should_refuse_files_on_processes_folder`,
    `should_drop_processes_on_processes_folder_into_its_checkout`, `should_refuse_processes_on_entry`.
  - `process_drop.rs`: `should_move_terminals_and_leave_runs` (hint `Move 2 processes to dev · 1 run stays (Ctrl+drop
    starts its copy there)`), `should_refuse_only_runs`, `should_refuse_already_there` (`Already in dev`),
    `should_start_copies_when_copy_forced` (hint ends `· the originals keep running`), `should_name_runs_missing_in_target`.
  - `transfer/mod.rs`: `should_force_copy_with_configured_copy_modifier`, `should_force_nothing_with_other_modifiers`
    (copy CTRL, held CTRL|SHIFT → None), `should_name_copy_modifier` (CTRL → `Ctrl`).
- [ ] **Step 2:** run → fail. **Step 3: Implement** pure parts, then the glue: process and run rows `on_drag` with
  `DraggedProcesses`; every row `on_drag_move::<DraggedProcesses>` / `::<DraggedFiles>` sets `drop_hint` for the row
  under the pointer (only redraw when it changed), `drag_over` styles the drop row with `tree_states.drop` when
  allowed; `on_drop::<DraggedProcesses>` → `drop_processes`: Move → `move_pinned` each, pick them, unfold the target's
  Processes, `tree_changed`; Start → `open_terminal_in` per terminal (Claude Code: the new terminal gets `claude\r`
  typed), `RunRequest` per run; then clear `drop_hint`. The chips show `drop_hint.text` while it is set.
- [ ] **Step 4:** `cargo test && cargo clippy --all-targets` → pass.
- [ ] **Step 5: Check in the app** (AGENTS.md): `./run.sh -- --profile agent-dnd --background --stop-processes-on-quit .`,
  open a terminal in one worktree; a real drag is not available over MCP until Task 10, so check here by hand only if
  a display is available, else defer to Task 10's check.

### Task 5: Transfer planning

**Files:** Create `src/transfer/plan.rs`; modify `src/transfer/mod.rs`

**Interfaces:**
- Consumes: `DropPlace`, `Verb`.
- Produces:
  - `pub struct Source { pub path: PathBuf, pub checkout: CheckoutRef }`; `pub enum Layout { Flat, KeepFolders }`;
    `pub struct Operation { pub verb: Verb, pub sources: Vec<Source>, pub target: DropPlace, pub layout: Layout }`.
  - `pub enum LayoutChoice { Straight, Ask { common: PathBuf }, Flat }`; `pub fn layout_choice(sources: &[Source]) -> LayoutChoice`.
  - `pub fn verb_for(sources: &[Source], target: &DropPlace, forced: Option<Verb>) -> Verb`.
  - `pub fn refusal(sources: &[Source], target: &DropPlace, verb: Verb) -> Option<String>`.
  - `pub enum IncomingKind { File, Symlink, EmptyFolder }`;
    `pub struct Incoming { pub top: usize /* index into sources */, pub path: PathBuf, pub relative: PathBuf, pub kind: IncomingKind, pub size: u64 }`.
  - `pub enum Existing { File { size: u64, modified: SystemTime }, Folder }`.
  - `pub enum Conflict { Exists, Same, Differs { added: usize, removed: usize }, DiffersBinary, Twice { first: PathBuf }, FolderThere, FileThere }`;
    `pub enum Action { Replace, Skip, KeepBoth }`; `pub fn offered(conflict: &Conflict) -> &'static [Action]`.
  - `pub struct PlanRow { pub incoming: Incoming, pub destination: PathBuf, pub there: Option<Existing>, pub conflict: Option<Conflict>, pub action: Option<Action> }`;
    `pub struct Plan { pub operation: Operation, pub rows: Vec<PlanRow>, pub common: Option<PathBuf> }`.
  - `pub fn plan(operation: Operation, incoming: Vec<Incoming>, existing: impl Fn(&Path) -> Option<Existing>) -> Plan`.
  - `Plan::{conflicts(), clean(), undecided() -> usize, set_all(action), kept_name(index) -> PathBuf, summary() -> String, title(labels) -> String}`.

- [ ] **Step 1: Tests** (`plan.rs`), with the mock's scenario (GEN-817's `mock.css`, `index.html`, IDP-30's `mock.css`,
  `run.sh`, `Cargo.toml`, `src/` copied flat into dev's `internal-apps/ive/mock`):
  - `should_go_straight_in_from_one_folder`, `should_ask_from_folders_of_one_checkout` (common = deepest common folder),
    `should_go_flat_from_several_checkouts`.
  - `should_move_within_one_checkout_and_copy_across`, `should_force_verb_with_modifier`.
  - `should_refuse_drop_into_itself` (`Cannot put src inside itself`), `should_refuse_move_onto_own_folder`
    (`Already here; Ctrl+drop copies`), `should_refuse_move_into_subfolder_of_dragged_folder`.
  - `should_keep_folders_below_common_folder` (destination paths), `should_place_flat_by_name`.
  - `should_mark_existing_file_as_conflict`, `should_mark_two_incoming_files_of_one_name_twice` (Replace not offered),
    `should_mark_folder_in_the_way`, `should_keep_both_onto_itself_without_conflict` (paste onto own folder → action
    KeepBoth, no conflict).
  - `should_name_kept_copy_after_earlier_rows` (`mock (2).css`, then `mock (3).css`), `should_sum_up_actions`
    (`copies 4, replaces 1, skips 1 · 1 to choose`), `should_count_undecided`.
- [ ] **Step 2:** run → fail. **Step 3: Implement.** `Same` starts on `Skip`; every other conflict starts undecided.
- [ ] **Step 4:** `cargo test transfer && cargo clippy --all-targets` → pass.

### Task 6: Walking and classifying on disk

**Files:** Create `src/transfer/scan.rs`

**Interfaces:**
- Consumes: Task 5 types.
- Produces: `pub const MAX_FILES: usize = 20_000; pub const MAX_BYTES: u64 = 4 << 30;`
  `pub struct Walked { pub incoming: Vec<Incoming>, pub left_out: usize }`;
  `pub fn walk(sources: &[Source], verb: Verb, other_checkouts: &[PathBuf]) -> Result<Walked, String>`;
  `pub fn existing(path: &Path) -> Option<Existing>`;
  `pub fn classify(plan: &mut Plan)` (turns `Exists` into `Same` / `Differs` / `DiffersBinary` / `FolderThere` /
  `FileThere`, sets `Same` to Skip); `pub fn lines_only_in(a: &str, b: &str) -> usize`; `pub fn is_binary(bytes: &[u8]) -> bool`
  (over 1 MiB, or a NUL in the first 8 KiB); `pub fn preview(path: &Path) -> String` (first 200 lines, or `binary, 14 KB`).

- [ ] **Step 1: Tests** with `tempfile::tempdir()`:
  - `should_walk_folder_with_hidden_files`, `should_keep_symlink_as_symlink`, `should_list_empty_folder`.
  - `should_leave_out_git_and_nested_checkouts` (a `.git` folder and a folder in `other_checkouts` → not in
    `incoming`, `left_out` 2 — counting the nested checkout as one, `.git` not counted).
  - `should_refuse_too_many_files` (use a test-only bound: `walk_bounded(…, Bounds { files: 3, bytes: u64::MAX })`;
    message `Copy refused: 4 files, at most 3`).
  - `should_classify_same_content_as_skip`, `should_count_differing_lines` (`+1 −0`), `should_classify_binary`.
- [ ] **Step 2–4:** fail, implement (`walk` = `walk_bounded` with the constants), pass with clippy.

### Task 7: Writing

**Files:** Create `src/transfer/write.rs`; modify `src/scratch/store.rs` (make `rename_new` `pub(crate)`, moved to
`src/transfer/write.rs` or a shared `src/rename.rs` — one copy only)

**Interfaces:**
- Consumes: `Plan`, `scan::existing`.
- Produces: `pub struct Report { pub written: Vec<PathBuf>, pub skipped: usize, pub failed: Option<(PathBuf, String)>, pub left_out: usize }`;
  `pub fn execute(plan: &Plan) -> Report`; `pub fn move_whole(operation: &Operation) -> Option<Report>` (same file
  system, every top-level destination free: rename each; `None` when not possible, the caller then walks);
  `Report::line(&self, operation: &Operation, labels) -> String` (`Copied 7 files to dev · internal-apps/ive/mock · skipped 1`,
  failure `Copied 5 of 7 files to dev · lib: lib/a.rs: Permission denied (os error 13)`).

- [ ] **Step 1: Tests** (tempdirs):
  - `should_copy_files_keeping_permissions`, `should_copy_symlink`, `should_not_leave_temp_file_on_failure`
    (target folder read-only → failed, no `.ive-copy` file left).
  - `should_replace_only_when_chosen`, `should_skip`, `should_keep_both_with_free_name`.
  - `should_not_touch_file_changed_since_plan` (modify target after planning → failed, contents unchanged).
  - `should_move_and_remove_emptied_folders`, `should_keep_folders_of_skipped_files_on_move`.
  - `should_move_whole_folder_by_rename`, `should_refuse_whole_move_when_name_taken` (`None`).
- [ ] **Step 2–4:** fail, implement, pass. Temp names: `.<name>.ive-copy-<pid>-<n>` in the target folder.

### Task 8: Dropping files, Copy and Paste, the sidebar line

**Files:**
- Create: `src/workspace/transfers.rs`
- Modify: `src/sidebar/view.rs` (`on_drop::<DraggedFiles>` on rows, the line at the sidebar's bottom),
  `src/workspace.rs` (field `transfer: Transfers`), `src/workspace/menu.rs` (`row_path`: `Folder(Files)` → checkout
  root; `run_menu_action` choice arm), `src/script/{config,declare,keymap}.rs`, `src/menu/resolve.rs`,
  `src/workspace/path_actions.rs`, `assets/scripts/ive.rn`

**Interfaces:**
- Consumes: Tasks 4–7.
- Produces: `pub struct Transfers { running: bool, line: Option<SidebarLine>, pending: Option<Operation> }`;
  `pub struct SidebarLine { pub text: String, pub sticky: bool }`;
  `Workspace::drop_files(&mut self, dragged: &DraggedFiles, onto: TreeKey, forced: Option<Verb>, window, cx)`;
  `Workspace::start_transfer(&mut self, operation: Operation, cx)` (background: `move_whole` or walk → plan →
  classify; conflicts → `open_conflicts(plan)` (Task 9; until then: line `N conflicts` and nothing written), else
  `execute` → line, reveal written); `pub fn clipboard_paths(text: &str) -> Option<Vec<PathBuf>>`;
  `Builtin::CopyFiles`, `Builtin::PasteFiles` (`act::CopyFiles`, `act::PasteFiles`, tree scope, tree menus, labels
  `Copy` / `Paste`; the layout menu's lines are `LineAction::Choice { builtin: Builtin::PasteFiles, choice: "keep-folders" | "flat" | "cancel" }`).

- [ ] **Step 1: Tests:** `transfers.rs`: `should_read_existing_absolute_paths_from_clipboard`,
  `should_refuse_clipboard_with_relative_or_missing_path`, `should_ignore_blank_clipboard_lines`;
  `script/engine.rs`: `should_bind_copy_and_paste_files_in_tree` (Ctrl+C/V on Linux); `path_actions.rs`:
  `should_grey_paste_without_files_on_clipboard` (applies → `Shown { enabled: false }`).
- [ ] **Step 2–3:** fail, implement. `UNIT_ACTIONS` count 64. Keys in `ive.rn`: `#{ key: "c", mods: "SECONDARY", action: act::CopyFiles, scope: "tree" }`,
  `"v"` → `act::PasteFiles`; tree menu after `act::FindInFiles`: `act::CopyFiles, act::PasteFiles`. The line shows
  under the tree for 5 s (sticky on failure, a click dismisses it); while `running`, a drop or paste sets the line `A
  copy is running` and does nothing. Reveal: `locating = TreeKey::Entry(first written)` + `continue_locating`, then
  the written files are picked.
- [ ] **Step 4:** `cargo test && cargo clippy --all-targets` → pass.

### Task 9: The Conflicts tab

**Files:** Create `src/conflicts_tab.rs`; modify `src/editor.rs` (every `Tab` match the checklist names: `kind` →
`"conflicts"`, `title`, `icon_name` → `icons::WARNING`, `saved` → `None`, `identity` → `None`, `checkout` → the
target's, `shown_statuses`, `poll_interval`, `watched`, `take_reread`, `tooltip`, `focus_handle`, `render`),
`src/workspace.rs` (`tree_key_of` → `None`), `src/workspace/transfers.rs`, `README.md` (`list_tabs` kinds, targets)

**Interfaces:**
- Consumes: `Plan` and its methods, `scan::preview`, `write::execute`.
- Produces: `pub struct ConflictsTab { pub plan: Plan, pub unfolded: BTreeSet<usize>, pub clean_unfolded: bool }`;
  `Tab::Conflicts(Box<ConflictsTab>)`; `Workspace::open_conflicts(&mut self, plan: Plan, window, cx)`;
  `Workspace::apply_conflicts(&mut self, tab: TabId, cx)` (closes the tab, runs `execute` off-thread, line, reveal);
  click targets `conflicts-all:<replace|skip|both>`, `conflict:<n>:<replace|skip|both>`, `conflict-row:<n>`,
  `conflicts-apply`, `conflicts-cancel`.

- [ ] **Step 1: Tests** (`conflicts_tab.rs`, on the tab's model, no drawing): `should_offer_only_skip_and_keep_both_for_twice`,
  `should_disable_apply_until_every_conflict_decided`, `should_set_all_where_offered`, `should_name_header_as_spec`
  (`Copy 7 files from GEN-817-ive, IDP-30-reserved-subdomain to dev · internal-apps/ive/mock` and the line under it).
- [ ] **Step 2–3:** fail, implement. Layout as the mock (`mock/components/checkouts/conflicts-tab.html`): header,
  All row, table (File, From with the checkout colour stripe, Incoming, There now, Why, Action), unfolding previews
  side by side, the folded clean list, Copy/Move (disabled until decided) + Cancel + summary. Closing the tab = Cancel.
- [ ] **Step 4:** `cargo test && cargo clippy --all-targets` → pass.

### Task 10: MCP `drop_rows`, docs, end-to-end check

**Files:** Modify `src/mcp/server.rs`, `src/api/mod.rs` (`Request::DropRows { rows, onto, force }`),
`src/workspace/api_requests.rs`, `README.md`, `mock/DESIGN.md` + `mock/README.md` (drop "not in the app yet" from
the section; note what differs from the mock, if anything)

**Interfaces:**
- Consumes: `drop_processes`, `drop_files`.
- Produces: tool `drop_rows { rows: Vec<String>, onto: String, force: Option<"copy"|"move"> }` → the hint, or the
  refusal; rows found by their `tree-menu:` names; bounded like `click` (`MAX_TARGET_BYTES`, at most 1000 rows).

- [ ] **Step 1: Test** in `api_requests.rs` style: `should_find_rows_by_their_menu_names` (pure lookup over
  `TreeRow`s). **Step 2–3:** fail, implement; README documents `drop_rows`, `tree-pick*`, the Conflicts targets.
- [ ] **Step 4: End to end in the app:** `./run.sh -- --profile agent-dnd --background --stop-processes-on-quit .`;
  with the `ive` MCP tools (`list_targets`, `click`, `drop_rows`, `list_tabs`, `screenshot`), on two worktrees of this
  repo (e.g. `.claude/worktrees/user-themes` and this one):
  open a terminal in one, `drop_rows` it onto the other's worktree row → `screenshot` shows it under the target;
  `cd` it into a third folder → it follows. Copy a folder between worktrees with a conflict → the Conflicts tab
  lists it; choose Skip; Copy → line says `skipped 1`. Ctrl+C a file, `tree-pick` a folder, Ctrl+V (`window_keys`) →
  `name` pasted. Stop the instance afterwards.
- [ ] **Step 5:** `cargo test && cargo clippy --all-targets && cargo build --release` → pass.
- [ ] **Step 6:** One commit of all the work (`git add -A` minus the untracked `progress.md`, which is not in this
  worktree): `tree: drag and drop, copy and paste between worktrees and projects; terminals move; Conflicts tab`.
