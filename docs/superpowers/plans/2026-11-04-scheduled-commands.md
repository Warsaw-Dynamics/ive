# Scheduled commands: plan

**Spec:** `docs/superpowers/specs/2026-11-04-scheduled-commands-design.md`

1. **Dependency**: `chrono` (already built for gpui; `clock`, `std`) for RFC 3339, local time and the cron's days.
2. **Cron** (`schedule/cron.rs`): `Cron::parse` (5 fields: `*`, numbers, `a-b`, lists, `/step`; day of week 0–7,
   Sunday both 0 and 7; day of month and day of week combined as cron does: either matches when both are set),
   `next_after(DateTime<Local>)` (minute by minute over days that match, at most 4 years ahead), and the 15-minute
   check over the next 24 times. Tests: each field form, refusals with the field named, weekdays at 07:00 over a
   weekend, a DST change, the too-frequent refusal.
3. **Delays and times** (`schedule/when.rs`): `parse_delay` (`90s`, `1h30m`, at most a year), `parse_at`
   (RFC 3339), and `first_time`: `at`, `in` or the cron, with the bounds. Tests.
4. **Model** (`schedule/mod.rs`): `Entry`, `State`, `Target`, `Schedule` (entries, next id).
   - `add`: replaces by (project, name) and drops the trust on a changed command; the 100 cap.
   - `come_due(now)`: returns what to start unasked and what became due, counting `missed`.
   - `finish(id, Outcome, now)`: a repeating entry leaves a history copy and moves to its next time.
   - `cancel`, `ask_again`, `remove_failed`; `due()` and `pending()` in order; `history(now)`; `prune(now)`;
     `to_json` for `list_scheduled`.
   - Tests per rule.
5. **File** (`schedule/file.rs`): load (a bad file moves to `.bad`), and save through a staging file and a rename,
   0600 on Unix. Round-trip and bad-file tests.
6. **Target** (`schedule/target.rs`, background): `plan(target) -> Plan` (`In(path)`, `AddWorktree(BranchName,
   dir)`, `NewBranch { source, dir }`, `Cannot(reason)`) from git2 (worktrees with their branches, local and
   `origin` branches); `prepare(plan)` runs fetch (git CLI, 2-minute timeout), branch and worktree; `steps(plan)`
   for the row. `git/worktree_add.rs` gains `add_worktree_at` (a given folder) and `add_worktree_from` (a new
   branch at a start commit, deleted again on failure). Tests on temp repositories: each plan, a new branch from
   the source, a missing source, the main checkout's branch.
7. **Desktop notices** (`desktop_notice.rs`): keyed by `NoticeKey` (`Process(ProcessId)` or `Scheduled`); clicks
   carry the key; `workspace/attention.rs` shows the tab for `Scheduled`.
8. **Workspace** (`workspace/schedules.rs`):
   - load at start; a 30-second tick; `come_due` handling (open the tab behind, or shown at start; the desktop
     notice; unasked starts);
   - plans read off the UI thread;
   - `start_scheduled`: open the project if needed, prepare off the thread, open the folder, show a hidden worktree,
     a terminal behind with the env and the typed command, title, history;
   - row picks, Always, Run selected, Skip, Remove, Cancel, Ask again;
   - save after each change.
   - `processes.rs` gains a terminal with extra env opened behind.
9. **Tab** (`schedule/tab.rs`, `editor.rs` `Tab::Scheduled`): render as the spec; attention text; icon
   `SCHEDULE`; not saved.
10. **Settings page** (`settings_page/scheduled.rs`): Due, Coming, Last 7 days, and the saved-to line.
11. **MCP** (`mcp/server.rs`, `api/mod.rs`, `workspace/api_requests.rs`): `schedule_command`, `list_scheduled`,
    `cancel_scheduled`, bounds checked in the server and in the model.
12. **Docs**: README (MCP table, a "Scheduled commands" section, the click targets); mock DESIGN.md (in the app).
13. **Check**: `cargo test`, `cargo clippy --all-targets`. Then the running app with `--profile agent-schedule`:
    - schedule over MCP: one due in 1 minute on an existing worktree, one on a new branch from `main`, one
      repeating, one on a missing folder (refused);
    - the tab coming due, Always, Run selected, Settings;
    - restart with a missed one.
