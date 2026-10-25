# Tree file actions: main checkout marker, rename, delete, more drags to a terminal

## Why

Four gaps in the project tree, met while working in it every day:

- The main checkout's row looks like any other worktree's: the same branch icon, and only the *other* worktrees carry
  a rail. Which row is the main checkout should show on the row itself.
- A file or folder cannot be renamed or deleted from the tree; the user goes to a terminal or a file manager for it.
  This includes the files listed under Uncommitted (an untracked file to throw away, a new file to rename).
- Only rows under Files drag onto a terminal. A changed file under Uncommitted, Since base or Commits, a project and
  a worktree are just as often wanted as a path in a terminal.

## Goals

### 1. The main checkout's marker

- In the tree only (not tabs, the hub or the Worktrees list), the main checkout's row draws a **small star badge in
  the bottom-right corner of its branch icon**.
- The star is IntelliJ's favourite-branch star (`platform/icons/src/expui/nodes/star.svg` and `star_dark.svg` in
  `/mnt/data/Projects/intellij-community`, Apache 2.0, as `branch.svg` already is): IntelliJ marks `main` / `master`
  as favourite branches with it. Its yellow (`#F2C55C` dark, the light file's colour in light) stays as IntelliJ draws it.
- Drawn as module favicons are badged on file icons (`favicon_badge.rs`): the same badge size and placement, ringed
  in the tree's background, as the attention dot on process rows is.
- A project with one checkout shows no worktree row, so nothing changes there.

### 2. Rename

- **Where:** a new tree menu item **Rename…** (`act::RenameFile`), and a key in the tree (`scope: "tree"` in
  `ive.rn`, changeable in `init.rn`; see "Keys per OS").
- **Applies to** one row: a file or folder under Files, or a changed file under Uncommitted, Since base or Commits
  that exists in the work tree now. Not with several rows selected, not on a deleted file's row, a change folder, a
  commit row, the group folders (Processes, Uncommitted, …), a worktree or a project row. Left out of the menu
  where it does not apply.
- **In the row:** the row's name becomes a text field (`text_input`) holding the current name, the part before the
  last `.` selected (all of it for a folder or a name starting with `.`). Enter renames, Escape or a click elsewhere
  cancels. The field takes the keys while it is open; the tree's own keys do not fire.
- **Refused**, with the reason shown in the line under the tree (as built: a tree row has a fixed height and no room
  beside the field) and the field kept open: an empty name, `.` or `..`, a name with
  `/` (or `\0`), the same name, or a name taken in that folder. The rename never replaces anything
  (`rename::rename_new`). An I/O error shows its message the same way.
- **After:** open tabs of the file (or of anything inside the folder) follow it to the new path, keeping their place
  and unsaved edits; a tab with unsaved edits saves to the new path. The renamed row is selected. Diff tabs of the old
  path are left to the git views, which see the change as any other change on disk.

### 3. Delete

- **Where:** a new tree menu item **Delete** (`act::DeleteFiles`), and a key in the tree (see "Keys per OS").
- **Applies to** the selection, as other tree menu items do (the pressed row alone when it is not selected): files
  and folders under Files, and changed files under Uncommitted, Since base and Commits that exist in the work tree.
  Rows it does not apply to (a deleted file's row, change folders, group folders, worktree and project rows, a
  checkout's root) are left out of what is deleted; when none is left, the item is left out of the menu.
- **How:** set in `init.rn` by a new `config.delete`, defaulting in `ive.rn` to `"trash"`:
  - `"trash"`: moved to the system trash (freedesktop trash on Linux, the Finder's Trash on macOS), no question asked.
  - `"permanent"`: removed for good after a prompt (IVE's `prompt.rs`), `Delete 3 items permanently?` / `Delete
    main.rs permanently?`, Delete or Cancel (Escape).
  - Any other value is a config error, with the value named, as `quit_countdown` reports one.
- **Open tabs:** tabs of a deleted file (or of a file inside a deleted folder) close. One with unsaved edits stops the
  deletion of that file first with a prompt (Delete anyway / Cancel); Cancel keeps it and its tab.
- **Failures:** a file that cannot be trashed or removed stays, and the reason shows as IVE shows other failed file
  actions (the transfer notices); the others still go.
- Terminals and processes working in a deleted folder are left alone.

### 4. Drags onto a terminal

- Rows under **Uncommitted, Since base and Commits** drag like rows under Files: the file's path in the work tree,
  the whole selection when the row is selected, the same chip. A drop on a terminal types the paths as Send to
  Terminal does (relative to the terminal's folder when it works in that worktree, else absolute), without Enter. A
  deleted file's row, change folders and commit rows do not drag.
- **Project and worktree rows** drag too: a project its main checkout's root, a worktree its root. On a terminal they
  type the **absolute** path, always, even for a terminal in that worktree.
- A drag that holds a project or worktree root does nothing when dropped on a tree row (no move or copy of a
  checkout); the chip's hint over tree rows says so or shows nothing. Changed files dropped on tree rows move or copy
  as rows from Files do. Project and worktree rows do not join the selection: they drag alone.

### 5. Changed files in the selection

- Changed-file rows under Uncommitted, Since base and Commits join the **Files pick group** (`sidebar/selection.rs`
  `pick_group`): Ctrl+click and Shift+click pick them together with Files rows, across sections and worktrees, as
  Files rows pick across worktrees today. Change folders, commit rows and a deleted file's row do not pick.
- What a selection stands for is its paths in the work tree: a path picked twice (as a Files row and a changed row,
  or under Uncommitted and Since base) counts once (`without_nested` and a dedup), for the drag, the menu items
  (Send to Terminal, Copy Path, Copy, Compare Files, Delete…) and Ctrl+Shift+C.
- The menu on a selected changed row applies to the whole selection (`menu_rows`), as on a selected Files row.
- The README's "files and folders pick together" lines say changed files pick with them.

### Settled by the designer

- **Checkouts are protected.** Rename and Delete refuse an open worktree, a folder holding one, and the main
  checkout's `.git`, with the reason, as Move does (`transfer/scan.rs`, "is an open worktree"). With Delete on a
  selection, those are left out and named; the rest goes.
- **Edits are written first.** Before a rename or delete, every open tab's pending edits reach the disk. Diff tabs of
  the file count as its tabs. When a tab of an affected file still holds edits that could not be saved, the rename or
  delete stops (nothing is touched), that tab is shown with its own bar, and the note under the tree (or the rename
  field) says why; the user saves or uses the tab's Close without saving, then tries again. This replaces the
  "Delete anyway" prompt above: it never loses edits. Rename then reopens each file tab at the new path in its place
  in the tab row (caret and scroll start afresh) and closes the file's diff tabs, so nothing writes to the old path.
- **Keys per OS.** Linux: F2 renames, Delete deletes. macOS: F2 or Enter renames, Cmd+Backspace deletes, as Finder.
- **Case-only rename** (`foo.rs` → `Foo.rs`) is allowed: it goes through a temporary name in the same folder, so it
  works on a case-insensitive file system; "taken" means taken by another entry.

### Everywhere

- README: the new menu items, F2 and Delete, `config.delete`, the drags, the main checkout's star.
- `ive.rn`: `config.delete = "trash";` with a comment, the two items in `config.tree_menus.main`, the two keys.
- MCP: the tree's rows already reach the menu items through the tree menu tools; rename needs the text field to be
  reachable as other text fields are, and the permanent-delete prompt as other prompts are.
- The mock (`mock/checkouts.html`, DESIGN.md) gets the star, the rename field and the menu items, as earlier
  features did.

## Non-goals

- Renaming or deleting worktrees, projects, branches, commits or scratches from these items.
- Undo of a delete inside IVE (the trash is the undo for `"trash"`).
- Rename by a slow second click on a row.
- Deleting a whole change folder (Uncommitted and the others) at once.
- Moving a file to another folder by rename (a `/` is refused; drag does moves).

## Anti-goals

- Anything that deletes without the user being able to get it back, unless they chose `"permanent"` and confirmed.
- A rename or delete that loses unsaved edits in an open tab.
- A project or worktree drop that moves or copies a checkout.

## Left to the builder

- The badge's exact pixel size and the ring, within the favicon badge's look.
- The trash crate (e.g. `trash`) or a hand-written freedesktop / macOS call; its licence must pass `deny.toml`.
- Whether rename and delete go through the transfer machinery (`src/transfer`) or sit beside it.
- How the rename field is keyed for MCP.
