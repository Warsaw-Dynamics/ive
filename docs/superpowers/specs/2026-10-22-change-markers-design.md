# Change markers in the editor (IntelliJ-style)

Status: design, approved in conversation. Date: 2026-10-22. Builds on
[the codeview design](2026-10-22-codeview-crate-design.md).

## Intent

While editing a file under git, the user sees in the gutter which lines changed since the last commit, and from a
marker can look at, copy, roll back or commit that one change — as in IntelliJ (user's screenshots: a blue bar on a
changed line; clicking it opens a popup with ↑ ↓, rollback, show diff, copy, a "Commit this change" field, and the
previous text below).

## Behaviour (user's decisions unless marked "my call")

### Markers

- In the line-number gutter of editable file tabs (`Content::Lines` with `edit: Some`) for files tracked in a git
  repository.
  - green bar: added lines;
  - blue bar: changed lines (a hunk that replaced lines);
  - small grey triangle between two lines: lines deleted there.
- Base: the file in HEAD, in working-tree form (checkout filters applied — the same `git::blob_filter` the
  editable diff uses), so markers agree with the Uncommitted diff tab. Staged changes still show as changes (my call).
- Live: recomputed a moment after typing stops (and after a save, an outside change, or HEAD moving), off the UI
  thread. The panel never blocks on it; old markers stay, shifted by edits, until new ones land (same rule as
  colours).
- No markers: untracked files, files not in a repository, binary files, files over 8 MiB (`MAX_HIGHLIGHT_BYTES`),
  scratches, read-only views and diffs (my call).

### Popup (click a marker)

Opens under the changed block, over the text, and closes with Esc, a click outside, or editing elsewhere. Contains:

- ↑ / ↓: previous / next change (moves caret and popup).
- Rollback: replaces the block with the HEAD text — an ordinary edit, undoable, saved like typing.
- Show diff: opens the file's Uncommitted diff tab scrolled to this hunk.
- Copy: copies the previous (HEAD) text of the block.
- "Commit this change": a one-line message field and a send button (Enter sends).
- Below: the previous text of the block, coloured as code (deleted lines in the removed tint); nothing for pure
  additions.

Rollback, Copy and Commit act only on markers diffed from the current text: markers moved by edits since may hold a
line that is not the change's (one joined to the block's last line). While they are stale, the clicked action waits,
its buttons shown busy; the diff is asked for at once (no pause), and the action runs on the block as the new diff
finds it, or not at all when the block is gone (the popup closes). The popup also closes when the keyboard leaves
the editor and when the wheel scrolls the block out of view.

### Commit this change (user asked to include it)

Commits only this block, nothing else:

- The new content committed for the file = HEAD's version of the file with only this block replaced by the
  current block text (unsaved edits in the block count; other changes in the file and in other files are left out).
- Done through the `git` CLI so hooks, signing and the user's config apply: content into the object store with
  `git hash-object -w --path=<path> --stdin` (clean filters run), a temporary index built from the HEAD checked
  (`GIT_INDEX_FILE`, `git read-tree <head_id>`), the one entry updated (`git update-index --cacheinfo`), then
  `git commit -m <message>` with that index (reflog action `commit (ive <run>)`, so IVE finds its commit in HEAD's
  reflog: the oldest entry that reads `commit (ive <run>): <the commit's first line>`, whitespace squeezed as git
  does, and is not the HEAD checked; hooks inherit the action, and a pre-commit hook's `git reset` or `git checkout`
  writes `commit (ive <run>): updating HEAD` or the like on the HEAD checked). HEAD keeping no reflog at all: HEAD and
  its parent are taken. A reflog without IVE's entry: nothing is undone; HEAD is taken as IVE's commit only if it
  holds the tree and subject committed, and either way a bar says the commit was made, and how to find it.
  Afterwards the real index entry for the file is set to the file's entry in the new commit (as hooks left it) only
  if it equalled the old HEAD blob (the file had nothing staged); not when the commit was not identified. The
  variables IVE was started with that pick another repository or config are not passed on (`GIT_DIR`,
  `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_COMMON_DIR`, `GIT_CONFIG_PARAMETERS`, `GIT_CONFIG_COUNT` and
  `GIT_CONFIG_KEY_<n>`/`VALUE_<n>`, `GIT_NAMESPACE`, `GIT_CEILING_DIRECTORIES` and the like).
- IVE's commits run one at a time (process-wide lock). HEAD is checked before, and the new commit's parent after:
  when another commit landed in between (git put ours on top of it), ours is taken off with a compare-and-swap
  `git update-ref -m "ive: undo commit on moved HEAD" HEAD <parent> <ours>`, and a bar says so (ours, the commit it
  went on, nothing committed). Should that fail too (HEAD moved on again), the bar says so: our commit, the commit
  it went on, `git reset --soft <parent>` to undo it while HEAD is still ours, `git revert <ours>` once another commit
  is on top of it. The work tree is never touched.
- These (taken off, not taken off, made but not identified) show in a bar above the file's text, apart from the
  popup and whatever HEAD does next, until dismissed (✕, click target `commit-report-dismiss`) or a commit goes
  through; a commit that stays closes the popup as a success does, one taken off leaves it open with the message.
- Refused, with the reason shown in the popup: an empty message; the file has staged changes that differ from HEAD;
  a merge, rebase, cherry-pick or revert in progress; no HEAD (empty repository); a `filter` or
  `working-tree-encoding` attribute on the file (git's form of the text may differ from the one IVE compared); the
  `git` command missing or failing (its stderr shown); HEAD moved since the markers were computed (recompute and
  try again).
- Hooks run as with `git commit`: one that stashes or rewrites files (a formatter, `lint-staged`) changes them on
  disk during the commit, and IVE sees that as any outside change to the file.
- One commit in flight per editor: while it is, the field takes no typing, Enter and the send button do nothing
  (also after Esc and reopening the popup), and the send button shows busy. The answer applies to the change sent
  (its base lines and HEAD): a success closes that change's popup if it is open; a failure shows under that
  change's popup, now or when it is next opened, until the next commit is sent.
- After a commit the markers recompute against the new HEAD (that block's marker disappears) and IVE's git views
  refresh as they do for outside commits.

## Protocol additions

- `Event::NeedChangeBase` (panel → IVE) when a marked panel opens or wants a refresh; IVE answers
  `Cast::SetChangeBase { base: ChangeBase }` off the UI thread: `Tracked { head: Source, head_id }` or `Untracked`.
- The line diff (base vs the piece table's text) is computed inside `codeview` on a background job, as parsing is
  (`Event::NeedChangeDiff(job)` → IVE runs it → `Cast::ChangeDiff { revision, hunks }`), so the text never crosses
  to IVE. A diff library may be added to `codeview` (e.g. `imara-diff`, histogram) — builder's choice, pinned.
- `Event::CommitChange { head_id, block: Range<usize> (base lines), text: Vec<u8>, message }` →
  IVE answers `Cast::ChangeCommitted`, `Cast::ChangeCommitFailed(reason)` (a refusal, in the popup) or
  `Cast::ChangeCommitTrouble { report, left: CommitLeft::{Commit, Nothing} }` (the bar). No `path`: the host knows the panel's
  file. `text` is the whole file to commit (HEAD's text, from IVE, with only the block replaced), which
  `git hash-object` needs; of the edited text only the block's lines cross.
- Rollback, copy, navigation and show-diff stay inside the panel (show diff as `Event::OpenDiff { path, line }`).

## Non-goals

Staging hunks, committing several blocks at once, amending, choosing an author, markers against the index or another
branch, markers in diff tabs.
