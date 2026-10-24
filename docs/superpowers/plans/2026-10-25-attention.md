# Attention Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A process that sends a notification, a bell or the end of its progress is marked in its tab, its tree row,
the folded rows above it, a bell in the tab row and the hub, until the user looks at it; with IVE in the background
the desktop shows a notification.

**Architecture:** A `CallScanner` beside the hosted terminal's parser (as `ProgressScanner` and `DirectoryScanner`)
turns live output into numbered `CallReport`s on the connection's shared state. The workspace reads the latest report
after each batch of terminal events, decides with a pure `counts(Looking)` whether the user is looking, and stores an
`Attention` on the `Process`. Tree rows, tabs, the bell, the hub and MCP read it. `desktop_notice` sends and withdraws
desktop notifications through `notify-send`/`gdbus` (Linux) and `osascript` (macOS).

**Tech Stack:** Rust, gpui (pgui), alacritty_terminal's `vte` parser.

**Spec:** `docs/superpowers/specs/2026-10-25-attention-design.md`

## Global Constraints

- Only live output of hosted terminals raises attention; replayed output never does.
- Texts: control characters dropped, whitespace runs made one space, trimmed, at most 200 characters, cut ones end
  in `…`; an empty notification reads `notified`; bell `rang the bell`; progress end `finished`.
- One attention per process; a newer call replaces the text and keeps the first call's time.
- Palette key `attention`: dark `#F0A732`, light `#B86E00`.
- Desktop: title `<process title> — <worktree label>`; Linux `notify-send --app-name=IVE --print-id
  [--replace-id=<id>] --action=default=Show <title> <body>`; first failure logged once to `ive.log`.
- MCP names: bell `waiting`, list line `waiting:<process id>`; process JSON `attention: { text, since }` or null.
- Coding guideline (AGENTS.md): at most 3 parameters, no boolean parameters, no `!` in conditions where avoidable,
  tests `should_…` with given/when/then.

## Review Focus

1. `OSC 9;4;…` and `OSC 9;9;<path>` (ConEmu) must not read as notifications. Test:
   `should_not_take_conemu_commands_for_notifications` (Task 1).
2. A `BEL` that ends an OSC (`ESC ] 0;title BEL`) is not a bell. Test: `should_not_take_osc_terminator_for_a_bell`
   (Task 1).
3. A sequence split across two reads still counts once. Test: `should_find_a_call_split_across_reads` (Task 1).
4. A call while the user looks at the tab in an active window raises nothing, also in a detached window. Test:
   `should_not_count_a_call_the_user_sees` (Task 3).
5. A notification text with escape characters or 10 000 characters stays one short line. Test:
   `should_clean_and_bound_texts` (Task 1).

---

### Task 1: Calls in a program's output

**Files:**
- Create: `src/terminal/attention.rs`
- Modify: `src/terminal/mod.rs` (module, re-export `Call`, `CallReport`)
- Modify: `src/terminal/progress.rs` (`Progress::is_busy`)

**Interfaces:**
- Produces: `pub enum Call { Notification(String), Bell, Finished }` with `pub fn text(&self) -> String`;
  `pub struct CallReport { pub number: u64, pub call: Call }`; `pub struct CallScanner` with
  `pub fn scan(&mut self, bytes: &[u8]) -> Option<Call>` (the last call completed in `bytes`);
  `Progress::is_busy(self) -> bool`; `pub fn finished(before: Progress, after: Progress) -> bool`.

- [ ] Tests: `should_read_iterm2_notifications` (`\x1b]9;Claude needs you\x07` → `Notification("Claude needs you")`),
  `should_not_take_conemu_commands_for_notifications` (`9;4;1;5`, `9;9;/tmp` → None),
  `should_read_osc_777_body_else_title`, `should_read_kitty_payload_and_skip_base64` (`99;;Done` → "Done",
  `99;e=1;RG9uZQ==` → None), `should_read_a_bell` (`ab\x07c` → Bell),
  `should_not_take_osc_terminator_for_a_bell`, `should_find_a_call_split_across_reads`,
  `should_clean_and_bound_texts` (`"a\x1b[1m\tb\n"` → `"a [1m b"`, 10 000 × `x` → 199 `x` + `…`, `""` → `notified`),
  `should_finish_when_busy_progress_ends` (`Normal(5)→None` true, `None→None` false).
- [ ] Run `cargo test terminal::attention` — fails; implement; passes.
- [ ] Commit `terminal: read notifications, bells and progress ends from a program's output`.

### Task 2: Calls reach the session

**Files:**
- Modify: `src/terminal/hosted.rs` (`Parsers.call`, `Shared.call: Mutex<Option<CallReport>>`, `Connection::call`)
- Modify: `src/terminal/session.rs` (`Session::call() -> Option<CallReport>`, None in-process)
- Modify: `src/terminal/view.rs` (`TerminalView::call()`)

**Interfaces:**
- Consumes: Task 1.
- Produces: `TerminalView::call(&self) -> Option<CallReport>`; numbers grow by one per call, from 1.

- [ ] In `Output::parse`, for `Origin::Live` only: a scanned call, or `finished(previous, new)` progress, stores the
  next numbered `CallReport`. Replayed output updates progress without a call.
- [ ] `cargo test` and `cargo clippy --all-targets` clean; commit `terminal: hosted sessions keep the last call`.

### Task 3: Attention on processes

**Files:**
- Create: `src/attention.rs` (`Attention`, `Looking`, `counts`)
- Modify: `src/process.rs` (`Process.attention: Option<Attention>`, `Process.seen_call: u64`, setters, `waiting`)
- Modify: `src/main.rs` (module)

**Interfaces:**
- Produces: `pub struct Attention { pub text: String, pub since: SystemTime }`;
  `pub enum Looking { Shown, Hidden }` and `pub fn counts(looking: Looking) -> bool`;
  `Process::attention(&self) -> Option<&Attention>`;
  `Processes::take_call(&mut self, id, report: CallReport) -> bool` (true when the report is new; remembers its
  number); `Processes::raise(&mut self, id, text: String, now: SystemTime) -> bool` (keeps `since` of an existing
  one); `Processes::clear_attention(&mut self, id) -> bool`; `Processes::waiting(&self) -> Vec<&Process<V>>` (oldest
  first); `Processes::waiting_in(&self, checkouts: &[&Path]) -> usize`.
- [ ] Tests: `should_not_count_a_call_the_user_sees`, `should_take_each_call_once`,
  `should_keep_the_first_time_and_the_newest_text`, `should_list_waiting_longest_first`,
  `should_count_waiting_in_checkouts`.
- [ ] Run, implement, pass; commit `process: attention, raised by calls the user does not see`.

### Task 4: The workspace raises and clears

**Files:**
- Modify: `src/workspace/processes.rs` (`handle_process_events` reads `view.call()`)
- Modify: `src/workspace.rs` (`tab_shown` and window activation clear the shown process's attention)
- Modify: `src/tab_window.rs` (activation of a detached window clears its process's attention)
- Modify: `src/workspace/api_requests.rs` (`process_json` `attention`)

**Interfaces:**
- Consumes: Tasks 2, 3.
- Produces: `Workspace::is_looking_at(&self, id: ProcessId, window: &Window) -> Looking`;
  `Workspace::look_at_shown(&mut self, window: &Window, cx)`; `Workspace::waiting_changed(&mut self, cx)` (redraws
  tree, tab row, hub; later tasks hook desktop notices here).
- [ ] Check in the app over MCP (Task 9) that `printf '\e]9;hi\a'` in a background terminal sets `attention`.
- [ ] `cargo test`, clippy; commit `workspace: raise attention for calls away from the user, clear it on sight`.

### Task 5: Palette and tree rows

**Files:**
- Modify: `assets/theme/default.toml`, `src/theme_config/mod.rs`, `src/theme.rs` (`attention`)
- Modify: `src/sidebar/rows.rs` (`ProcessRow.attention: Option<String>`; `ProjectRow.waiting`, `CheckoutRow.waiting`,
  `FolderRow.waiting`: counted only for Processes)
- Modify: `src/sidebar/view.rs` (dot on the process icon, `● <text>`; `● n waiting` on folded rows)
- Create: `src/attention_dot.rs` (`pub fn dot(theme: Theme, ring: u32) -> Div`, `pub fn with_dot(icon, theme, ring)`)

- [ ] Tests in `rows.rs`: `should_show_what_a_process_said`, `should_count_waiting_on_folded_rows_only`
  (outline shows `waiting 1` on a folded project and checkout, not on expanded ones).
- [ ] Theme tests updated for the new key; commit `tree: what a process said, and how many wait under folded rows`.

### Task 6: Tabs and the bell

**Files:**
- Modify: `src/editor.rs` (`Tab::attention`, dot on the icon in `render_tab` and `render_detached`, tooltip adds
  the text; `waiting_button` left of Settings while `processes().waiting()` is not empty)
- Create: `src/waiting_menu.rs` (the list: icon, `title · worktree`, text, age; click shows the process)
- Modify: `src/workspace.rs` (`waiting_menu_open: bool` field toggled by the bell, closed by Escape, a press
  outside, or a choice)
- Copy: `notifications@20x20.svg` → `assets/icons/{dark,light}/notifications-20.svg` (done), `icons::NOTIFICATIONS_20`

- [ ] Check in the app: bell appears, list opens (MCP `click waiting`), `waiting:<id>` shows the process and clears.
- [ ] Commit `tabs: a dot on the tabs of processes that called, and the bell that lists them`.

### Task 7: The hub's Waiting section

**Files:**
- Modify: `src/hub/sources.rs` (`waiting: Vec<Item>`), `src/hub/rows.rs` (section `Waiting` first on an empty
  query), the workspace code that fills sources (process items' `state` is the text while waiting)

- [ ] Test in `src/hub/model.rs`: `should_list_waiting_first_on_an_empty_query` (sections `["Waiting", "Recent",
  "Changed", "Running"]`).
- [ ] Commit `hub: Waiting first`.

### Task 8: Desktop notifications

**Files:**
- Create: `src/desktop_notice.rs`

**Interfaces:**
- Produces: `pub struct DesktopNotices` (global): `send(&mut self, id: ProcessId, title: String, body: String, cx)`,
  `withdraw(&mut self, id: ProcessId)`, `withdraw_all(&mut self)`; clicks come back as `Workspace::show_waiting(id,
  window, cx)` that activates the main window.
- Pure helpers with tests: `fn notify_send_args(title: &str, body: &str, replace: Option<u32>) -> Vec<String>`,
  `fn parse_notify_send(output: &str) -> (Option<u32>, Option<&str>)` (id line, action line),
  `fn osascript_args(title: &str, body: &str) -> Vec<String>` (quotes and backslashes escaped).
- [ ] Tests: `should_build_notify_send_args`, `should_read_id_and_action`, `should_escape_osascript_strings`.
- [ ] Hook: raise with no active IVE window → `send`; clear → `withdraw`; any IVE window activates → `withdraw_all`.
- [ ] Commit `desktop: notify when IVE is in the background, withdraw once seen`.

### Task 9: README and the app

- [ ] README: section "Attention" (what raises it, the agent settings from the spec).
- [ ] `mock/DESIGN.md` "Attention": "(in the app since 2026-10-25)".
- [ ] Run `./run.sh -- --profile agent-attention --background --stop-processes-on-quit .`; over MCP: open two
  terminals, print `\e]9;hi\a` in the hidden one (`type_text`), `list_projects` shows `attention`; screenshot tab
  row, tree, bell list, hub; show the tab, attention clears. Stop the instance.
- [ ] `cargo test`, `cargo clippy --all-targets`; commit `docs: attention`.
