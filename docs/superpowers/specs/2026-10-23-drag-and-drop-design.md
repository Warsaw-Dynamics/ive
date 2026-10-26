# Drag and drop, copy and paste in the tree

Mock: `mock/checkouts.html` (`mock/dnd.js`, `mock/components/checkouts/conflicts-tab.html`), design notes in
`mock/DESIGN.md` "Drag and drop, copy and paste". `checkouts.html#tab=conflicts` opens the Conflicts tab.

## Why

Work moves between worktrees all the time: a terminal opened in the wrong worktree, a file wanted on another branch,
a folder copied from one project into another. Today that means typing `cd` into the terminal or copying files in a
file manager. The tree already shows every project and worktree side by side, so it should be the place to move
terminals and copy or move files between them. Overwriting must never happen by accident.

## Goals

1. **Pick rows.** Ctrl+click and Shift+click pick several rows, as files and folders already can (`selection.rs`),
   and now also process rows (terminals, Claude Code, runs). Files and folders are picked together; processes
   together; a pick of the other kind starts a new selection.
2. **Move terminals.** Dropping terminal or Claude Code rows on a worktree or project moves them there.
3. **Copy and move files.** Dropping file and folder rows on a folder, worktree or project copies or moves them
   there, in any project.
4. **Copy and paste.** Ctrl+C and Ctrl+V in the tree copy picked files and folders, keeping their folders.
5. **Conflicts.** Anything that would land on an existing file or folder is asked about in a Conflicts tab, before
   anything is written.
6. **Agents can do all of it over MCP.**

## Non-goals

- Dropping files from the OS file manager onto tree rows (dropping on the window keeps opening them), and dragging
  tree rows out to other apps.
- Undo, Cut (Ctrl+X), Delete, Rename, New file.
- Git-aware moves (`git mv`): files are moved on disk, git sees a delete and an add.
- Moving runs. A run belongs to its worktree's `.run.toml`.
- Changing the pty host's protocol.

## Anti-goals

- A drop or paste must never overwrite a file without the user choosing Replace for that file.
- Moving a terminal must not touch the shell: no `cd` typed, no restart, no signal.
- A moved terminal must not jump back on its own: not on the next rescan, not on a restart of IVE.
- A copy that fails half way must not leave a half-written file under the target name.

## Behaviour (decided)

### Picking

- The existing tree mouse bindings stay: SECONDARY+click `ToggleRowSelection`, Shift+click `ExtendRowSelection`.
- `is_multi_selectable` covers process rows (`TreeItem::Process`) and run rows (`TreeItem::LiveRun`) too. A range
  from a file to a process, or a toggle that mixes kinds, picks only the clicked row (`select_only`), so a selection
  holds files and folders, or processes, never both.
- Escape in the tree drops the picks (a new `Selection::clear`, which keeps the anchor).
- A file's checkout is the deepest checkout holding its path (`checkout_of_path`), wherever its row was picked: a
  worktree inside the main checkout's folder owns its files, even under the main checkout's tree. *(my call)*
- What a drag carries: the picked rows of the pressed row's kind when the pressed row is picked, else the pressed row
  alone (as `Selection::dragged_paths` does now). Of picked files and folders, one inside another picked folder is
  left out (it goes with its folder).

### Where things drop

The row under the pointer is the drop row; it is highlighted while the drag is over it and the drop is allowed (theme:
a drop background and a 1 px accent outline, `tree_states.drop`). A refused drop shows no highlight and the "no"
cursor. A small hint follows the pointer (the existing drag chip, its text replaced while over a row):

| Dragged | Drop row | Goes to |
|---|---|---|
| files, folders | folder | that folder |
| | file | its folder |
| | worktree, its Files folder | the worktree's root |
| | project | its main checkout's root |
| terminals, Claude Code, runs | worktree, its Processes folder | that worktree |
| | project | its main checkout |
| anything | other rows (Uncommitted, a commit, a change…) | refused |

Dropping files on a terminal keeps typing their paths (`DraggedFiles::typed_into`). Processes dropped on a terminal
do nothing.

### Moving terminals

- A plain drop **moves** every dragged terminal and Claude Code (not runs) to the target checkout:
  `Processes::move_to`, then the tree redraws, the moved rows are picked and the target's Processes unfold.
  Everything that goes by a process's checkout follows: its tree row, its tab tooltip, Send to Terminal's relative
  paths, worktree use. The shell is not touched.
- **Pinned until its directory changes.** The move records the process's directory at the time of the move. Following
  (`follow.rs` `moves`) skips a process while its directory equals that recorded directory; the first directory
  report that differs clears the record and follows as today: into another checkout it moves there; outside every
  checkout it stays where it is. This applies to both `apply_directories` and `follow_all_directories` (after a scan).
- **Survives a restart.** For every hosted terminal and Claude Code, the saved state records its session id, project
  id, checkout path and pinned directory, if any. `adopt_sessions` uses the recorded project and checkout instead of
  the session's `ive.project` / `ive.checkout` labels when the record's project is open and its checkout folder exists
  (the check the labels' path gets today: worktrees are not listed yet at adopt time; one that turns out unlisted is
  handled by `move_unlisted_to` after the scan), and restores the pin. Records of sessions that are gone are not saved
  again. In-process (not hosted) terminals are not restored at all today, so nothing is recorded for them.
- A moved process whose target worktree is later removed goes to the main checkout, as any process does
  (`move_unlisted_to`); its record goes with it.
- Runs in the dragged set stay. Hint: `Move 2 processes to dev · 1 run stays (Ctrl+drop starts its copy there)`;
  only runs dragged: refused, `A run stays in its worktree; Ctrl+drop starts its copy in dev`.
- **Ctrl+drop starts the same things there instead**, the originals keep running: a terminal starts a new terminal;
  Claude Code starts a new terminal running `claude`; a run starts the target checkout's own copy of that
  configuration (`RunRequest` for the same folder and key), refused for a run whose `.run.toml` is not in the target
  (named in the hint). Hint: `Start 2 processes in dev · the originals keep running`.
- Already in the target: refused, `Already in dev`.

### Copying and moving files

- **Verb:** same checkout → move; another checkout or project → copy. A modifier held at the drop forces it. The
  modifiers are settings, `config.drop_modifiers = #{ force_copy: "CTRL", force_move: "SHIFT" }` (`move` is a Rune
  keyword, so it cannot be a key), declared in `ive.rn` with the
  platform's file-manager defaults (Linux and Windows `CTRL` / `SHIFT`; macOS `ALT` (Option) / `SUPER` (Cmd)), written
  like key bindings' `mods` (`parse_mods`: `NONE, CTRL, SHIFT, ALT, SUPER, SECONDARY`, combined with `|`), so
  `init.rn` can change them. An invalid or `NONE` value, or force_copy equal to force_move, fails the load like any
  bad config value, naming the key and value (`init.rn: drop_modifiers.force_move: "CTRL" is the same as force_copy`). The modifiers are read
  when the drop happens (`window.modifiers()`, matched exactly), and the hint follows them while dragging. The copy
  modifier also starts copies of processes. Hints name the configured keys (`Ctrl+drop`, `Option+drop`).
- All other modifiers of this feature are already bindings in `ive.rn` and stay so: Ctrl+click / Shift+click
  (`config.mouse_bindings`, `ToggleRowSelection` / `ExtendRowSelection`), Ctrl+C / Ctrl+V and Escape (`config.keys`,
  tree scope).
- Refused, with the reason in the hint: into itself or one of its own folders; a move whose every source already sits
  in the target folder (`Already here; Ctrl+drop copies`).
- **Layout:**
  - Sources all in one folder: they go straight in.
  - From different folders of **one** checkout: on drop, a menu at the pointer asks (styled as the tree menu):
    `Keep folders, from <common folder>/` (each source goes in at its path below the sources' deepest common folder)
    or `Flat, all into <target>/`, then a separator and `Cancel`. Escape cancels.
  - From **several** checkouts or projects: always flat, no menu. The hint says `· flat: from 2 worktrees`.
- A folder always carries its whole content, hidden and git-ignored files included. A **copy** leaves git's own out:
  a `.git` file or folder inside it, and any folder that is another open checkout's root (a worktree under
  `.claude/worktrees`); the sidebar line says so (`· left out 2 worktrees`). A **move** carries a nested `.git` with
  its folder, as `mv` would, and is refused for a folder holding another open checkout (`Move refused: … is an open
  worktree; move it with git worktree move`). *(my call, after review)* Symlinks are copied as symlinks (not
  followed). File permissions are kept.
- A source copied onto its own place (pasted or Ctrl+dropped into the folder it is in) goes beside itself as one copy:
  `a (2).rs`, or a folder as `src (2)/` with its content inside. A file a move leaves where it is keeps its name, and
  no other incoming file can take it.
- **Writing:** every file is written under a temporary name in the target folder and renamed into place, so a failed
  copy never leaves a half-written file under its real name. Replace renames over the old file; any other write uses
  a no-replace rename (`renameat2(RENAME_NOREPLACE)`, as `scratch/store.rs` does) so a file that appeared meanwhile
  is not overwritten (it is reported as failed). A move on one file system is a rename of each top-level source when
  nothing inside it conflicts, else file by file; across file systems it is a copy, then the sources are removed. A
  move removes the folders it emptied; skipped files keep theirs.
- **Bounds:** at most 20 000 files and 4 GiB in one copy (or a move across file systems); larger is refused before
  anything is written, with the counts (`Copy refused: more than 20 000 files`; the walk stops at the bound). A move on one file system whose
  top-level sources' names are free in the target is a rename of each and walks nothing, so has no bound; only a
  source whose name is taken is walked to find the conflicts. *(my call)*
- **One at a time:** while a copy or move writes, another drop or paste is refused (`A copy is running`). Several
  Conflicts tabs may be open; each re-checks its files when it writes. *(my call)*
- The work runs off the UI thread. A line at the bottom of the sidebar says what is going on and what happened, and
  goes 5 s after the end: `Copying 120 files to dev · internal-apps/ive…`, then `Copied 7 files to dev ·
  internal-apps/ive/mock · skipped 1`. A failure keeps the line until clicked, naming the first failed path and the OS
  error (`Copied 5 of 7 files to dev · lib: lib/a.rs: Permission denied (os error 13)`); nothing is rolled back.
- After it ends, the target folders unfold in the tree and the written files are picked. The tree's file watching
  picks up the new files as it does for any change on disk.
- Open tabs of moved files behave as for any file removed on disk (`take_removed`); they do not follow the file.

### Copy and paste

- New tree-scope actions `act::CopyFiles` (Ctrl+C, macOS Cmd+C) and `act::PasteFiles` (Ctrl+V, macOS Cmd+V) in
  `ive.rn`, and items in `config.tree_menus.main` after Find in Files: `Copy` and `Paste` (greyed when the clipboard
  holds no files), labelled like the other built-ins (no counts). *(my call)*
- Escape in the tree scope is `act::ClearRowSelection`, declared in `ive.rn` like the others.
- Copy writes the picked files' and folders' absolute paths to the clipboard, one per line, as `act::CopyFullPath`
  (Copy Path ▸ Copy Absolute Path; Ctrl+Shift+C, macOS Cmd+Alt+C) does for the selection. IVE keeps nothing of its own: Paste reads the clipboard.
- Paste applies when every non-empty clipboard line is an absolute path that exists; it then copies those into the
  target of the selection's anchor row (as a drop on that row would). Several picked rows: the anchor decides.
- Paste keeps the folders below the sources' common folder when they are all in one checkout; from several checkouts,
  or from outside every checkout, it is flat. No menu.
- Pasting a file onto its own place (target folder is its folder) writes `name (2).ext` without asking.

### Conflicts tab

When the plan has any conflict nothing is written; a tab opens instead (permanent, never saved, never temporary,
icon `warning`). Title `Copy conflicts` or `Move conflicts`; the tooltip has the full title.

- **Header:** `Copy 7 files from GEN-817-ive, IDP-30-reserved-subdomain to dev · internal-apps/ive/mock`; under it
  `3 files are in the way · flat: every file straight into mock/ · nothing is written until you press Copy` (or
  `keeping folders, from internal-apps/ive/`).
- **All** Replace | Skip | Keep both: sets every conflict where that action is offered.
- **Rows**, one per conflicting file, in plan order, each on two lines (the file and its actions; under it where it
  comes from, both sizes and why), at most 500 drawn (`N more conflicts not shown; All sets them too`):
  - File: its path below the target folder, with its icon.
  - From: the source checkout's colour stripe, `<checkout label> · <path in its checkout>`; full path in the tooltip.
  - Incoming / There now: `12 lines · 189 B` for text, the size alone for binary files (over 1 MiB, or with a NUL in
    the first 8 KiB), `folder` for a folder.
  - Why:
    - `differs: +1 −0 lines` (lines only in the incoming file, lines only in the existing one, as multisets) for text;
      `differs` for binary;
    - `same content` (byte-equal): muted, starts on **Skip**;
    - `also coming from <checkout> · <path>`: two incoming files land on one name (flat layout);
    - `a folder of that name is there` / `a file of that name is there`.
    The last two offer only Skip and Keep both.
  - Action: Replace | Skip | Keep both. Keep both shows the name it gives under it (`as mock (2).css`): `name (N).ext`
    with the lowest free N ≥ 2, counting names earlier rows' Keep both take.
  - A click on a row unfolds both texts side by side (first 200 lines each, monospace, scrolling); binary files say
    `binary, 14 KB`.
- `4 files go in without a conflict`, folded, lists them.
- **Copy** (Move) is greyed until every row has an action; beside it `copies 4, replaces 1, skips 1 · 1 to choose`.
  **Cancel** closes the tab, writing nothing. Closing the tab is Cancel.
- Copy re-checks each file when it writes it: a target that changed since the plan (appeared, vanished, other size or
  mtime) is not touched and is reported as failed in the sidebar line.
- The tab is drawn with the tree's row sizes and the theme's colours; widths and spacing are the builder's.

### MCP

- Every tree row also registers `tree-pick:<row>` (as Ctrl+click: toggles the row in the picks) and
  `tree-pick-range:<row>` (as Shift+click), next to its `tree-menu:<row>`; `<row>` is the row's name as there
  (`entry:/abs/path`, `process:3`). `click` itself is unchanged.
- New tool `drop_rows { rows: [row names], onto: row name, force: "copy" | "move" (optional) }`: does what dragging
  those rows (the first is the pressed one) onto that row and dropping does, as if the platform's copy or move key were
  held when `force` is given; the layout menu opens as for a real drop and agents answer it with `click` on its lines'
  targets. Returns the hint text, or why it was refused.
- The Conflicts tab's buttons and per-row actions are click targets: `conflicts-all:<action>`,
  `conflict:<n>:<action>` (`replace`, `skip`, `both`), `conflict-row:<n>`, `conflicts-apply`, `conflicts-cancel`.
- `list_tabs` reports the new kind as `conflicts`. README "Drive IVE from an agent (MCP)" documents all of the above.

## Constraints

- Follow AGENTS.md's coding guideline. The planning (what goes where, conflicts, names) is pure and unit-tested
  without a file system; the writing is a separate step tested against a temp folder.
- Keys and menus are declared in `assets/scripts/ive.rn` like the others, so `init.rn` can rebind or drop them.
- The mock shows the decided behaviour; where this document and the mock differ, this document wins.

## Left to the builder

- Module layout (e.g. a `transfer` module for plan and write, a `conflicts_tab` module), names of types and theme
  keys, exact colours and spacing of the drop highlight, hint and Conflicts tab.
- How the drop hint replaces the chip's text while over a row (gpui `on_drag_move` on the tree, or `drag_over`).
- How the layout menu reuses the tree menu's view.
- Where in the saved state the moved-terminal records live, and their format.
- The order of work, as long as each step leaves IVE working.
