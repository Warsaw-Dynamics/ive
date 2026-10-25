# Tree file actions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The main checkout's row marked with a star; Rename and Delete in the tree (also on changed files); changed
files, projects and worktrees dragged onto a terminal; changed files picked together with Files rows.

**Architecture:** Pure helpers (name checks, protected paths, the delete mode, the selection's paths) carry the tests;
the workspace wires them to the menu, keys, tabs and the tree note. File operations run off the UI thread after every
open tab's edits have landed, as closing a tab waits for them (`workspace/saves.rs`).

**Tech Stack:** Rust, gpui, Rune scripts (`ive.rn`), the `trash` crate (MIT, default features off).

**Spec:** `docs/superpowers/specs/2026-10-25-tree-file-actions-design.md`

## Global Constraints

- AGENTS.md's coding guideline: no boolean parameters, ≤ 3 parameters, early returns, no negated names, `should_…`
  tests with `// given // when // then`.
- Error texts self-contained, naming the value: `Rename refused: main.rs is taken in src`.
- No new dependency but `trash = { version = "5.2.9", default-features = false }`; `THIRD-PARTY-NOTICES.md`
  regenerated with `tools/third-party-notices.sh`; `cargo deny check licenses` (deny.toml) passes.
- The star: IntelliJ `expui/nodes/star.svg` (light, `#FFAF0F`) and `star_dark.svg` (dark, `#F2C55C`), copied unmodified
  as `assets/icons/{light,dark}/star.svg`, credited as the other IntelliJ icons are (`assets/icons/NOTICE`).
- `config.delete`: `"trash"` (default in `ive.rn`) or `"permanent"`; anything else:
  `delete must be "trash" or "permanent", not <value>`.
- Keys: Linux F2 → `act::RenameFile`, Delete → `act::DeleteFiles`; macOS F2 and Enter → `RenameFile`,
  Cmd+Backspace → `DeleteFiles`; all `scope: "tree"`.
- Menu labels: `Rename…`, `Delete`; prompt `Delete <name> permanently?` / `Delete <n> items permanently?`, buttons
  `Delete`, `Cancel`.

## Review Focus

- A path picked twice (Files row and Uncommitted row of one file; Uncommitted and Since base) acts once: Delete must
  not fail on the second, the drag types it once. → Task 2 test `should_count_a_path_picked_twice_once`.
- Renaming or deleting a folder whose files have open tabs: the tabs inside follow or close, not only an exact path
  match. → Task 5 test `should_find_tabs_inside_a_renamed_folder` (pure `affected` helper).
- Delete on `.git`, the worktrees folder or a nested worktree from Files' `@hidden`: refused, the rest goes. → Task 4
  test `should_refuse_a_folder_holding_an_open_worktree`.
- A case-only rename on Linux and macOS: `foo.rs` → `Foo.rs` works and is not "taken". → Task 4 test
  `should_rename_only_the_case`.
- A rename while the file's tab holds edits that cannot be saved: nothing renamed, the tab shown, the note says why
  (no edits lost). → Task 5/6 manual check over MCP (Task 8), the stop rule in `edits_settled`.

---

### Task 1: The main checkout's star

**Files:**
- Create: `assets/icons/light/star.svg`, `assets/icons/dark/star.svg` (copies)
- Modify: `assets/icons/NOTICE`, `src/icons.rs` (`pub const STAR: &str = "star.svg";`, embedded list if there is one),
  `src/favicon_badge.rs`, `src/sidebar/row_icons.rs`, `src/sidebar/view.rs` (`checkout_row`)

**Interfaces:**
- Produces: `RowMark { favicon, rail, star: bool }` (a field, not a parameter);
  `favicon_badge::Badge { image: BadgeImage, ring: u32 }` with `enum BadgeImage { Favicon(PathBuf), Embedded(&'static str) }`.

- [ ] **Step 1: Failing test** in `row_icons.rs`: `should_star_the_main_checkout` — `row_mark` of a `TreeItem::Checkout`
  with `is_main: true` has `star == true` and `rail == None`; `should_not_star_another_worktree` the reverse
  (build a minimal `Workspace`-free variant: split `row_mark`'s checkout arm into `fn checkout_mark(is_main, colour: impl FnOnce() -> Option<u32>) -> RowMark` and test that).
- [ ] **Step 2:** `cargo test row_icons` → FAIL.
- [ ] **Step 3:** Implement; `checkout_row` draws `badged(icon(theme, BRANCH), star.then(|| Badge { image: Embedded(STAR), ring: row background }), theme)`.
  The ring colour: the row's background as `folder_mark` / tab badges pick it.
- [ ] **Step 4:** `cargo test row_icons` → PASS; `cargo clippy --all-targets` clean.
- [ ] **Step 5:** Commit `tree: the main checkout's branch icon carries IntelliJ's favourite-branch star`.

### Task 2: Changed files pick with Files rows

**Files:** Modify `src/sidebar/selection.rs`, `src/workspace/menu.rs` (`menu_rows`, `tree_subject`),
`src/workspace/projects.rs` (`dragged_paths`), `src/workspace/transfers.rs` (`copy_files`)

**Interfaces:**
- Produces: `Listed { key, pick, path: Option<PathBuf> }` (the work-tree path of a Files-group row);
  `pick_group`: `TreeItem::Change(row) if row.change.mark != Mark::Deleted => Some(PickGroup::Files)`;
  `Selection::dragged_paths(&self, pressed: &TreeKey, listed: &[Listed]) -> Vec<PathBuf>` (deduplicated, nested
  dropped); `Selection::listed_paths` deduplicated; `menu_rows` widens for any pressed row whose listed `pick` is
  `Files`; `tree_subject` dedups paths keeping the first.

- [ ] **Step 1: Failing tests** in `selection.rs`:
  - `should_pick_a_change_with_files` — toggling a Files row then a change row keeps both.
  - `should_count_a_path_picked_twice_once` — Entry `/p/a` and a change row whose `path` is `/p/a` both selected:
    `dragged_paths` = `[/p/a]`.
  - `should_drag_a_changed_file_inside_a_picked_folder_with_its_folder` — `/p/d` and change `/p/d/x` → `[/p/d]`.
  And in `menu.rs`: `should_widen_a_menu_on_a_selected_change_to_the_selection`.
- [ ] **Step 2:** `cargo test selection menu_rows` → FAIL.
- [ ] **Step 3:** Implement. `dragged_keys` keeps working on keys; the folder-nesting check uses listed paths.
  The change row's path: `checkout.path.join(OsStr::from_bytes(change.path.as_bytes()))`, as `row_path` does —
  move it into one `pub fn change_path(row: &ChangeRow) -> PathBuf` in `sidebar/folders.rs` and use it in both.
- [ ] **Step 4:** tests PASS, clippy clean.
- [ ] **Step 5:** Commit `tree: changed files pick together with files and folders`.

### Task 3: Drags onto a terminal

**Files:** Modify `src/dragged_files.rs`, `src/sidebar/view.rs` (`entry_row`, `render_row`), `src/sidebar/view/folder_view.rs`
(`change_row`), `src/workspace/tree_drop.rs`, `src/workspace/transfers.rs`, `src/terminal/view.rs`

**Interfaces:**
- Produces: `DraggedFiles { pressed: TreeKey, workspace }`; `Dragging::Files(TreeKey)`; `files_drop_hint`,
  `files_drop`, `drop_files` take `&TreeKey`; `Workspace::dragged_paths(&self, pressed: &TreeKey)`.
  New `pub struct DraggedRoot { pub path: PathBuf, pub workspace: WeakEntity<Workspace> }` with
  `fn typed(&self) -> String` (= `path_text::typed_paths(&[path], Base::Absolute)`) and `fn chip(&self) -> DragChip`
  (label: the root folder's name). Tree rows take no `DraggedRoot` drop.

- [ ] **Step 1: Failing test** in `dragged_files.rs`: `should_type_a_root_absolute` — `DraggedRoot` text for
  `/home/u/repo` equals `typed_paths(&[/home/u/repo], Base::Absolute)` (test the pure `fn root_text(path: &Path) -> String`).
- [ ] **Step 2:** `cargo test dragged_files` → FAIL.
- [ ] **Step 3:** Implement: `change_row` drags `DraggedFiles` when its pick is `Files`; `project_row` and
  `checkout_row` drag `DraggedRoot` (project: its main checkout's path); the terminal's `.on_drop` gets a second
  handler for `DraggedRoot` typing `typed()`; its drag-over styling accepts both.
- [ ] **Step 4:** tests PASS, clippy clean.
- [ ] **Step 5:** Commit `tree: changed files, projects and worktrees drag onto a terminal`.

### Task 4: `config.delete` and the file operations

**Files:**
- Create: `src/script/delete_mode.rs`, `src/file_ops.rs`
- Modify: `Cargo.toml`, `src/main.rs` (mod), `src/script/{mod,config,engine,thread,state}.rs`, `src/workspace/scripts.rs`
  (wherever `drop_modifiers` is carried, `delete` is carried the same way), `assets/scripts/ive.rn`

**Interfaces:**
- Produces: `pub enum DeleteMode { Trash, Permanent }` (gpui `Global`, default `Trash`), `delete_mode::parse(&rune::Value) -> Result<DeleteMode, String>`.
  `file_ops`:
  - `pub fn checked_name(old: &Path, new: &str) -> Result<PathBuf, String>` — the target path; refuses empty, `.`,
    `..`, `/`, `\0`, the same name. Taken is checked by the rename itself.
  - `pub fn stem_end(name: &str, kind: PathKind) -> usize` — where the selection ends: before the last `.` of a file
    (all of a folder or a dot-file).
  - `pub fn protected(path: &Path, checkouts: &[PathBuf]) -> Option<String>` — a checkout root, a folder holding an
    open checkout, or a `.git` (file or folder) named; reason text as Move's: `<path> is an open worktree`,
    `<path> is git's own folder`, `<path> is a checkout's root`.
  - `pub fn rename(from: &Path, to: &Path) -> Result<(), String>` — `rename::rename_new`; a case-only change (`to`
    differs from `from` only in case and `to`'s metadata is `from`'s inode) goes via `<name>.ive-rename-<pid>` in the
    same folder; `AlreadyExists` → `Rename refused: <name> is taken in <folder>`.
  - `pub fn delete(path: &Path, mode: DeleteMode) -> Result<(), String>` — `trash::delete` or `remove_file` /
    `remove_dir_all` (a symlink: `remove_file`, never followed).

- [ ] **Step 1: Failing tests**: `delete_mode`: `should_take_trash_and_permanent`, `should_refuse_another_value`
  (exact message above). `file_ops`: `should_refuse_names_with_a_slash`, `should_refuse_dot_and_dot_dot`,
  `should_refuse_the_same_name`, `should_select_the_stem_of_a_file`, `should_select_all_of_a_dot_file`,
  `should_refuse_a_folder_holding_an_open_worktree`, `should_refuse_git_folder`, `should_rename_only_the_case`
  (temp dir), `should_refuse_a_taken_name` (temp dir, both files intact), `should_delete_permanently` (temp dir).
  No test calls the real trash.
- [ ] **Step 2:** `cargo test delete_mode file_ops` → FAIL.
- [ ] **Step 3:** Implement; `ive.rn`: `config.delete = "trash";` with a comment (`"permanent": removed at once,
  after a question`).
- [ ] **Step 4:** tests PASS; `cargo test` (the script defaults tests) PASS; clippy clean.
- [ ] **Step 5:** Commit `files: rename and delete helpers, config.delete`.

### Task 5: Delete in the tree

**Files:** Create `src/workspace/file_actions.rs`; Modify `src/script/keymap.rs` (`Builtin::DeleteFiles`, Tree scope,
menu kind Tree), `src/script/declare.rs`, `src/script/config.rs` (`UNIT_ACTIONS` 68→69 with Rename), `src/menu/resolve.rs`
(label), `src/workspace/menu.rs` (applies, run), `src/workspace/scripts.rs` (window builtin on the anchor),
`assets/scripts/ive.rn` (menu, keys), `src/script/engine.rs` (key tests)

**Interfaces:**
- Consumes: `file_ops::{protected, delete}`, `DeleteMode`, `Workspace::tree_subject`, `saves::saves_landed`.
- Produces: `pub fn affected(tabs: impl Iterator<Item = (TabId, PathBuf)>, paths: &[PathBuf]) -> Vec<TabId>` (a tab
  whose path is one of `paths` or inside one); `Workspace::delete_files(&mut self, pressed: &TreeKey, window, cx)`;
  `fn deletable(subject: &Subject, checkouts: &[PathBuf]) -> (Vec<PathBuf>, Vec<String>)` (kept, refusals);
  applies: Hidden when nothing is deletable.

Flow: deletable (without_nested) → refusals noted → `Permanent` asks the prompt → `flush_edits`, `saves_landed` →
any affected tab still `is_unsettled`: stop, show it, note `Delete stopped: <name> has edits that could not be saved`
→ delete off the UI thread → close affected tabs (`tabs.close`), `sync_watches`, note failures
(`Could not delete <path>: <error>`) as `NoteKind::Staying`; success leaves no note.

- [ ] **Step 1: Failing tests**: `should_find_tabs_inside_a_renamed_folder` (for `affected`), `should_leave_roots_out_of_delete`,
  `should_hide_delete_without_deletable_paths`, engine: `should_bind_delete_and_rename_in_tree` (Linux keys).
- [ ] **Step 2:** FAIL. **Step 3:** implement. **Step 4:** PASS, clippy clean.
- [ ] **Step 5:** Commit `tree: Delete, to the trash or for good after a question`.

### Task 6: Rename in the row

**Files:** Modify `src/workspace/file_actions.rs`, `src/sidebar/view.rs` (`entry_row`) and
`src/sidebar/view/folder_view.rs` (`change_row`): the field in place of the name, `Builtin::RenameFile` wiring as Task 5,
the tree's state (`Tree { renaming: Option<Renaming> }` in `src/sidebar/view.rs` or wherever `Tree` lives)

**Interfaces:**
- Produces: `pub struct Renaming { pub key: TreeKey, pub path: PathBuf, pub input: Entity<TextInput>, pub error: Option<String>, _events: Subscription }`;
  `Workspace::start_rename(&mut self, pressed: &TreeKey, window, cx)`, `submit_rename`, `cancel_rename`;
  `fn rename_applies(subject: &Subject, checkouts: &[PathBuf]) -> Applies` — one path, not protected.
  The input's click target `tree-rename`; Escape (on the row's key-down, stopping propagation) and focus loss cancel.

Flow on Enter: `checked_name` (error under the row, field stays) → `flush_edits`, `saves_landed` → unsettled affected
tab: error `<name> has edits that could not be saved` → `file_ops::rename` off the UI thread → each affected
`Tab::Document` replaced in place (`tabs.replace`) by `FileTab::new(Document::open_stamped(new_path))`, each affected
`Tab::Diff` closed → `sync_watches`, the tree refreshes, the new row selected (`TreeKey::Entry(new)` when it is a Files
row, else the selection cleared).

- [ ] **Step 1: Failing tests**: `should_offer_rename_for_one_path`, `should_hide_rename_for_a_root`,
  `should_place_the_renamed_path_inside_a_renamed_folder` (pure `fn moved_path(path, from, to) -> PathBuf`).
- [ ] **Step 2:** FAIL. **Step 3:** implement. **Step 4:** PASS, clippy clean.
- [ ] **Step 5:** Commit `tree: Rename in the row, its tabs following`.

### Task 7: Docs and mock

**Files:** `README.md` (Use table: Rename, Delete, the drag rows, selection, the star; Scripting: `config.delete`),
`mock/` (the star on the main checkout's row, Rename… and Delete in the tree menu), `THIRD-PARTY-NOTICES.md`,
the spec (my calls as built).

- [ ] **Step 1:** Write; `tools/third-party-notices.sh`.
- [ ] **Step 2:** Commit `docs: rename, delete, drags and the main checkout's star`.

### Task 8: Check in the running app

- [ ] `./run.sh -- --profile agent-fileops --background --stop-processes-on-quit <scratch repo>` with a worktree;
  over MCP: the tree menu on a file shows Rename… and Delete; rename a file with an open tab (the tab follows);
  delete to trash (the file is in `~/.local/share/Trash/files`); a changed file drags (MCP drag if available, else
  check the selection over `click` with modifiers); screenshot the star. Stop the instance.
