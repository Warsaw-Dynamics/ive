# Selectable Text and Errors Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Read-only text the user may paste elsewhere becomes selectable. Failures carry the details that pin them
down, and can be copied and read over MCP. Failures that went only to stderr show as notices and are written to
`ive.log`.

**Architecture:** A new `selectable` module holds the selection logic (pure) and a gpui element that draws a
`StyledText` with a highlight for the selection. One app-wide `TextSelection` global holds the selected block's id
and text and a focus handle. A new `Place::Text` routes the user's editor-scope Copy and Select All to it. A
`Failure` type replaces `Tab::Failed`'s plain error string and has one renderer, which notices reuse. A
`notices` module holds an app-wide `Notices` global (max 5) and an `AppLog`, which writes `ive.log` in the same way
as `ScriptLog`.

**Tech Stack:** Rust, gpui (`StyledText`, `TextLayout::index_for_position`, `HighlightStyle`), alacritty_terminal,
git2.

**Spec:** `docs/superpowers/specs/2026-10-23-selectable-text-and-errors-design.md`

## Global Constraints

- One selection in the window. Pressing elsewhere clears it. A selection never spans blocks.
- Mouse: double click selects a word, triple click the block, Shift+click extends.
- Copy and Select All come from the user's `editor`-scope bindings through `Place::Text`. No other built-in acts
  there.
- Failed-tab report and MCP `error`: title, summary, `Label: value` details, output, then `build_info::summary()`
  and `std::env::consts::OS`.
- Shell failures keep the last 40 output lines. Signals are named as `SIGSEGV (11)`.
- Notices: at most 5, oldest dropped, shown in the main window only, never take focus. The log is
  `<script log folder>/ive.log`, capped at 1 MB like `scripts.log`.
- No new theme keys. The selection uses the palette's `selection` (terminal) colour. Notices use the existing
  `editor.notice_*` sizes and palette colours.
- Coding guideline (AGENTS.md): at most 3 parameters, no boolean parameters, tests named `should_…` with
  given/when/then.

## Review Focus

1. A click with no drag on selectable text that sits beside a link or button must not swallow that control's click.
   Test: `should_not_select_on_click_without_drag` (Task 1, pure: a press and release at the same index give an
   empty selection).
2. Non-ASCII text: word and line ranges must land on char boundaries. Test: `should_find_word_in_multibyte_text`
   (Task 1).
3. A shell that IVE stops itself (Stop, close, quit) must still close quietly. Test:
   `should_close_quietly_when_ive_stopped_the_shell` (Task 5).
4. A notice must not move the keyboard. Covered by Task 7's design: the notice has no focus handle. Checked in the
   app (Task 9).
5. The report text must stay useful when output is empty or details are missing. Test:
   `should_write_report_without_output` (Task 4).

---

### Task 1: Selection logic and the selectable element

**Files:** Create `src/selectable.rs` (pure logic and tests) and `src/selectable/element.rs` (gpui element). Modify
`src/main.rs` (`mod selectable;`).

**Interfaces (produces):**
- `pub struct Span { pub start: usize, pub end: usize }` (byte offsets, `start <= end`)
- `pub fn word_at(text: &str, index: usize) -> Span`: the run of alphanumerics, `_`, `.`, `/`, `-` around `index`
  (so paths and shas select whole); one char otherwise.
- `pub fn spanned(anchor: usize, head: usize) -> Span`
- `pub fn selected_text(text: &str, span: Span) -> &str`
- `pub struct TextSelection` (gpui `Global`): `id: Option<ElementId>`, `text: String`, `span: Span`,
  `focus: FocusHandle`. Methods: `select(&mut self, id, text, span)`, `clear(&mut self)`,
  `copy_text(&self) -> Option<&str>` (none when empty), `select_all(&mut self)`.
- `pub fn selectable(id: impl Into<ElementId>, text: impl Into<SharedString>) -> SelectableText` (an `IntoElement`;
  `.mono()` uses the code font; colour and size come from the parent's text style like `div().child(text)`).

- [ ] **Step 1:** Tests in `selectable.rs`: `should_select_word_around_index` (`"cannot read /a/b.rs: gone"`, index
  inside the path → `/a/b.rs`), `should_find_word_in_multibyte_text` (`"zażółć gęślą"`, index 3 → `zażółć`, end on a
  char boundary), `should_order_span_when_dragging_backwards` (`spanned(9, 2)` → `2..9`),
  `should_not_select_on_click_without_drag` (`spanned(4, 4)` is empty, so `copy_text` is `None` after
  `select(…, 4..4)`), `should_copy_selected_part`.
- [ ] **Step 2:** `cargo test --bin ive selectable` fails (module missing).
- [ ] **Step 3:** Implement the logic. Then the element: on prepaint, take `StyledText::with_highlights` with one
  `HighlightStyle { background_color: selection }` for the span when this id is selected. On paint, register mouse
  handlers on a hitbox. Mouse down (left): `click_count` 1 → anchor (Shift extends), 2 → `word_at`, 3 → whole
  block; focus `TextSelection.focus`; `cx.stop_propagation()` only when the press is on this text. Drag with the
  button held → head. Use `TextLayout::index_for_position` (an `Err(i)` counts as `i`). Use the I-beam cursor. A
  mouse down anywhere else in the window clears the selection: the workspace's root `on_mouse_down` capture calls
  `clear()` unless the press was on selectable text.
- [ ] **Step 4:** `cargo test --bin ive selectable` passes; `cargo clippy --all-targets` is clean.
- [ ] **Step 5:** Commit: `selectable: read-only text the mouse selects, one selection in the window`.

### Task 2: Copy and Select All through the user's keys

**Files:** Modify `src/script/keymap.rs` (`Place::Text`, which matches `Scope::Editor`), `src/workspace/scripts.rs`
(`place_of`: `Place::Text` when `TextSelection.focus` is focused; `CopyTo`: copy `copy_text()` when the place is
Text; `SelectAll`: `select_all()`; other editor built-ins do nothing in Text).

**Interfaces:** consumes Task 1's `TextSelection`.

- [ ] **Step 1:** Tests in keymap.rs: `should_match_editor_bindings_in_text_place` (ctrl-c in Scope::Editor is found
  at `Place::Text`) and `should_not_match_terminal_bindings_in_text_place`.
- [ ] **Step 2:** They fail.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** `cargo test --bin ive script::keymap`, clippy.
- [ ] **Step 5:** Commit: `keys: Copy and Select All act on selected text`.

### Task 3: Selectable text where the inventory says so

**Files:** `src/settings_page/{scripts,mod,updates,third_party}.rs`, `src/prompt.rs` (detail), `src/workspace/quit.rs`
(detail), `src/overview/{commit_tab,diff_tab,git_log_tab,section_row,worktree_list_view,scratches}.rs`,
`src/scratch/tab.rs` (header), `src/run/{values_tab,configs_tab}.rs`. Replace each listed text child with
`selectable(id, text)`, using ids such as `"scripts-instructions-2"` or `"commit-sha"`. The project overview location
shows the full path.

- [ ] **Step 1:** No new unit test: these are render calls. `cargo build` and clippy must stay clean.
- [ ] **Step 2:** Commit: `selectable text on Settings, headers, prompts and run tabs`.

### Task 4: Failures with details, Copy error, and MCP `error`

**Files:** Create `src/failure.rs`. Modify `src/editor.rs` (`Tab::Failed { title: String, failure: Failure }`,
rendered by `failure::render`), every `Tab::Failed` call site (wrap the existing message as `Failure::new(summary)`),
and `src/workspace/api_requests.rs` (`list_tabs` adds `"error": failure.report(title)` for failed tabs).

**Interfaces (produces):**
- `pub struct Failure { summary: String, details: Vec<(&'static str, String)>, output: Vec<String> }`, with
  `Failure::new(summary: impl Into<String>) -> Self`, `.with(label: &'static str, value: impl Into<String>) -> Self`,
  `.with_output(lines: Vec<String>) -> Self`, `summary(&self) -> &str`, and
  `report(&self, title: &str) -> String`.
- `pub fn render(id: &str, title: &str, failure: &Failure, theme: Theme, cx) -> Div`: the summary in the error
  colour, a details grid, the output in mono, each part selectable, and a `copy-error` button that copies `report`.

- [ ] **Step 1:** Tests in failure.rs: `should_write_report_with_details_and_output` (title, summary, `Folder: /p`,
  output lines, the version line and OS, in that order) and `should_write_report_without_output` (no empty output
  section). In api_requests tests: `should_list_failed_tab_error`, if the existing test harness can build a failed
  tab; otherwise unit-test the JSON helper that adds `error`.
- [ ] **Step 2:** They fail.
- [ ] **Step 3:** Implement and migrate the call sites. Keep the old message as the summary, so nothing loses
  information.
- [ ] **Step 4:** `cargo test`, clippy.
- [ ] **Step 5:** Commit: `failures: summary, details and output, selectable, Copy error, in list_tabs`.

### Task 5: Terminal failures that say why

**Files:** `src/terminal/view.rs` (`TerminalState::Failed(Failure)`; `exit_state_of` / `run_exit_state_of` take a
`ExitFacts { status, elapsed, start, lost: Option<LostTerminal> }` struct to stay within 3 parameters), and
`src/terminal/hosted.rs` (keep the connection error: `lost()` gives the reason instead of a bool).
`src/workspace/processes.rs`: a signal failure for a process IVE is stopping (`Processes::is_stopping`, new) closes
quietly. Shell start failure details: the shell command, folder, `$SHELL`, hosted or not. Also `ptyhost/src/host.rs`:
add context on `create_dir`, `File::create(log)` and `spawn` (`cannot start the terminal host <exe>: <io>`, etc.)
and the host's exit status.

**Interfaces (produces):** `pub fn signal_name(signal: i32) -> String` (`"SIGSEGV (11)"`; unknown numbers give
`"signal 77"`) in `src/terminal/exit.rs` (new), used by Task 6 for runs.

- [ ] **Step 1:** Tests (view.rs tests module): `should_keep_last_output_when_shell_fails_at_start` (the failure
  holds the 40 lines passed in, and the details include `Folder`), `should_name_signal_of_crashed_shell`
  (`ExitStatus::from_raw(11)` → summary or detail has `SIGSEGV (11)`; this applies at any elapsed time),
  `should_close_on_exit_code_after_start` (code 1 after 3 s → `Closed`),
  `should_close_quietly_when_ive_stopped_the_shell` (processes.rs: a signal failure for a stopping id → removed
  without a failed tab; a pure helper `quiet_exit(stopping, state) -> TerminalState` is fine),
  `should_say_why_terminal_was_lost` (the lost reason appears under `Cause`, and the attach command under `Attach`).
  In exit.rs: `should_name_known_signals`.
- [ ] **Step 2:** They fail.
- [ ] **Step 3:** Implement. The view passes `self.output_text(Some(40))` into the failure.
- [ ] **Step 4:** `cargo test`, clippy.
- [ ] **Step 5:** Commit: `terminals: failures keep the output, name the signal and say why the host was lost`.

### Task 6: The other errors that say too little

Each of these gets its own test, named after the improvement, asserting the new detail is in the message:
- `codeview/src/code_view.rs` `mark_deleted`: `deleted on disk: <path>` plus `; this tab keeps your unsaved edits`
  when dirty. Test: `should_name_deleted_path`.
- `src/workspace/runs.rs` start failure: `Failure` with `Command`, `Folder`, `Configuration` (`<rel>/.run.toml`).
  Test on a pure builder `run_failure(config, folder, error)`.
- `src/run/file.rs`: parse errors name `<rel path>/.run.toml:<line>:<col>`, where the relative path comes from
  the caller. Test: `should_name_run_file_in_error`.
- `src/run/values.rs` and `src/settings.rs`: TOML errors keep line and column (use the error's span, as
  `theme_config/user.rs` `line_of` does). Tests: `should_keep_line_of_vars_error`,
  `should_keep_line_of_settings_error`.
- `src/git/repo.rs` `GitError::new`: append ` (class <Class>, code <Code>)`. Test: `should_keep_git_code_and_class`.
- `src/workspace/hub.rs:533`: show `error` (Display), not `error.message`.
- `src/workspace/runs.rs` exit label: `✕ SIGKILL` for 128+n codes from a signal, using `signal_name`. Test:
  `should_name_signal_of_run_exit`.

- [ ] Steps as in Task 1: tests, see them fail, implement, `cargo test` + clippy, commit
  `errors: paths, commands, line numbers, git codes and signals in messages`.

### Task 7: Notices and the app log

**Files:** Create `src/notices.rs` (`Notices` global, `AppLog`, render) and `src/notices/view.rs` if large. Modify
the main window's workspace render (overlay at the bottom right) and `src/mcp/server.rs` + `src/api.rs` +
`src/workspace/api_requests.rs` (a `notices` tool: `{ notices: [{ time, text }], log_file }`).

**Interfaces (produces):**
- `pub fn notify(cx: &mut App, text: impl Into<String>)`: pushes a notice (timestamped, max 5) to `Notices`,
  appends it to `AppLog`, `eprintln!("ive: {text}")`, and refreshes windows.
- `pub fn log_only(text: &str)`: off-thread failures with no channel. It writes to stderr and the log through a
  process-wide `AppLog` (`OnceLock`).
- `Notices::dismiss(index)`. The panel's click targets are `notice-copy:<n>` and `notice-close:<n>`.

- [ ] **Step 1:** Tests: `should_keep_five_newest_notices`, `should_append_notice_to_log`,
  `should_describe_notices_for_mcp`.
- [ ] **Step 2:** They fail.
- [ ] **Step 3:** Implement. The panel uses `failure::render`-like styling with selectable text and is never
  focusable.
- [ ] **Step 4:** `cargo test`, clippy.
- [ ] **Step 5:** Commit: `notices: failures away from tabs show at the bottom right and go to ive.log`.

### Task 8: stderr-only failures become notices

Replace `eprintln!("ive: …")` with `notices::notify(cx, …)` at the sites the spec lists. Where there is no `App`
context, use `log_only`, or forward through the existing channel (reattach: `workspace/processes.rs:149`; registry:
`terminal/hosted.rs`; edits not saved: `file_tab.rs:568–591`; run from the tree menu: `path_actions.rs:129`; git
actions: `git_actions.rs`, `tree_git.rs`; hub scratch: `hub.rs`; hub listing: `hub_index.rs`; state save:
`persist.rs`; watch backend: `changes.rs`; MCP server stopped: `mcp/server.rs`; OS open / new window:
`os_open.rs`, `path_actions.rs:101`, `tab_windows.rs:84`; pasted image: `terminal/view.rs:663`; send paths:
`send_paths.rs:73`). Record in the commit which sites went to `log_only`.

- [ ] Build, clippy, `cargo test`. Commit: `failures that went to stderr show as notices`.

### Task 9: Docs, then check in the app

- [ ] README: "Selecting text", failed tabs (Copy error, `list_tabs` error), notices and `ive.log`, the `notices`
  MCP tool and click targets. Update the mock's `DESIGN.md` with a short section.
- [ ] Run `./run.sh -- --profile agent-select --background --stop-processes-on-quit .`. Over MCP: open Settings ›
  Scripts and screenshot, then select via xdotool-free means is not possible over MCP, so check with a screenshot
  that the text renders the same. Open a terminal with a failing command (`open_terminal` with `command: "exit 3"`
  is closed normally; use `sh -c 'echo boom >&2; kill -SEGV $$'`) and check the failed tab's details, output and
  `list_tabs` error. Trigger a notice (Run on a non-runnable path from the tree menu) and call `notices`. Quit the
  instance and remove the profile.
- [ ] Commit: `docs: selectable text, failures, notices`.
