# Change markers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** IntelliJ-style change markers in the editor gutter with a popup to view, copy, roll back and commit one
change.

**Architecture:** IVE supplies the HEAD text (working-tree form) through the panel protocol; `codeview` diffs it
against the piece table on a background job and draws markers and the popup; committing one block goes back to IVE,
which commits through the `git` CLI with a temporary index.

**Tech Stack:** Rust, gpui, codeview crate, libgit2 (reads), `git` CLI (the commit), a line-diff crate in codeview.

**Spec:** `docs/superpowers/specs/2026-10-22-change-markers-design.md` (and the codeview design it builds on).

## Global Constraints

- `codeview` never imports IVE. The edited text never crosses to IVE except the one committed block.
- No UI-thread blocking: base reads, diffs and the commit run in the background; results carry a revision/generation
  and stale ones are dropped.
- Markers only for editable file tabs of tracked files ≤ `MAX_HIGHLIGHT_BYTES`, not binary.
- Tests: `should_…` with `// given`, `// when`, `// then`; `cargo test` and `cargo clippy --all-targets` clean.
- Coding guidelines: "Coding guidelines" section of AGENTS.md (main checkout copy).
- Running-app checks use profile `agent-codeview`, a temp git repo, `--background`; never the user's files.

## Review Focus

1. CRLF / filtered files: markers compare working-tree forms; a CRLF file with no edits shows no markers.
2. Editing while a diff job runs: markers never point at the wrong lines (shifted until the new result lands).
3. Commit with other staged files: they are not committed and stay staged.
4. Commit of a block with unsaved edits: the committed content contains exactly that block's current text.
5. HEAD moves (commit from a terminal) while the popup is open: the commit is refused, markers recompute.

---

### Task 1: Markers

**Files:** codeview: `panel/protocol.rs` (`NeedChangeBase`, `SetChangeBase`, `NeedChangeDiff`, `ChangeDiff`),
new `codeview/src/changes.rs` (diff job, hunks → markers, shifting by edits), `text_editor/view.rs` (gutter
drawing); IVE: `src/codeview_host.rs` / `src/file_tab.rs` (answer with HEAD text via `git::blob_filter`, refresh on
save / outside change / HEAD change), `src/git/` (HEAD text of a path, tracked or not).

**Interfaces:** `pub enum ChangeBase { Tracked { head: Source, head_id: String }, Untracked }`;
`pub struct ChangeHunk { pub base: Range<usize>, pub current: Range<usize> }` (line ranges);
`pub enum Marker { Added, Changed, Deleted }` per line / between lines.

- [ ] Tests: `should_mark_added_changed_and_deleted_lines`, `should_show_no_markers_for_an_unchanged_crlf_file`,
  `should_shift_markers_with_edits_until_the_new_diff_lands`, `should_drop_a_diff_of_an_older_revision`,
  `should_not_mark_untracked_or_large_files`; IVE: `should_answer_the_head_text_in_working_tree_form`.
- [ ] Implement; MCP check: edit a tracked file → green/blue/grey markers appear and follow edits.
- [ ] Commit `codeview: change markers in the editor gutter`.

### Task 2: Popup — previous text, navigation, rollback, copy, show diff

**Files:** codeview: new `codeview/src/changes/popup.rs` (or `change_popup.rs`), `text_editor/view.rs`,
`panel/protocol.rs` (`Event::OpenDiff { path, line }`); IVE: open the Uncommitted diff tab at a line.

- [ ] Tests: `should_open_the_popup_for_the_clicked_marker`, `should_roll_back_a_block_as_one_undoable_edit`,
  `should_copy_the_previous_text`, `should_move_to_the_next_and_previous_change`,
  `should_close_the_popup_on_escape_and_on_an_edit_elsewhere`.
- [ ] Implement; MCP check: click target for a marker (`change-marker:<line>`), rollback, Ctrl+Z, copy.
- [ ] Commit `codeview: change popup`.

### Task 3: Commit this change

**Files:** codeview: popup message field + send, `Event::CommitChange`, `Cast::ChangeCommitted` /
`ChangeCommitFailed`; IVE: new `src/git/commit_block.rs` (CLI steps from the spec), wiring, refresh of markers and
git views.

- [ ] Tests (real temp repos): `should_commit_only_the_block`, `should_leave_other_staged_files_staged`,
  `should_refuse_when_the_file_has_staged_changes`, `should_refuse_during_a_merge`,
  `should_refuse_when_head_moved`, `should_commit_unsaved_block_text`, `should_run_clean_filters`
  (CRLF/autocrlf repo), `should_update_the_index_entry_when_nothing_was_staged`.
- [ ] Implement; MCP check: commit one of two changed blocks; `git show` has only it; the other stays a marker.
- [ ] Commit `codeview: commit one change from the editor`.
