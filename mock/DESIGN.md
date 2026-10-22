# IVE — projects, worktrees and overviews: design notes

Final design of the `checkouts.html` mock, written as input for the implementation spec.
Serve with `cd internal-apps/ive && python3 -m http.server 8767`, open
<http://localhost:8767/mock/checkouts.html>.

Since 2026-10-18 the mock shows the app as the Rust code draws it (last synced 2026-10-22: run configurations,
temporary tabs, the tree and tab menus, Send to Terminal, the hub's contents search). Parts of this design the app does not have yet are marked *(not in the app yet)*; the
mock does not show them (they are in its git history). Run configurations: the README, "Run configurations".

## Model

- **Project**: a folder the user opened (a git repository, or a plain folder: "no git").
- **Checkout** (worktree): a working directory of a project. Its id is set once
  (`project@branch` at creation) and never changes, even when the branch in it changes.
  The first checkout is the main one (`dev`, `main`). New worktrees go in `<main checkout>/<worktrees>/<branch, / as
  ->`, where `worktrees` is read from the main checkout's `.ive.toml` (a relative path, no `..`; default `.worktree`).
  The atlassian repo's `.ive.toml` says `worktrees = ".claude/worktrees"`.
- A project with one checkout, or without git, hides the worktree level: its content sits
  directly under the project.
- **Base** of a checkout: the main branch for a worktree (`dev`); the remote for the main
  checkout (`origin/dev`). "Ahead/behind" and "since base" are computed against it.

What belongs where:

| Thing | Belongs to | Stored |
|---|---|---|
| Processes (terminal, Claude Code, runs) | worktree they run in | in memory |
| Run configurations | the folder holding `.run.toml`, in each checkout (each reads its own copy) | `<folder>/.run.toml` in the checkout, any folder but the root; values in `~/.config/ive/<profile>/vars.toml`, secrets in the OS wallet |
| Databases (data sources) *(not in the app yet)* | project | team: `<module>/.ive/databases/`; personal: `~/.config/ive/projects/<repo>/<module>/databases/`, passwords in the OS wallet |
| Scratches | project (never a worktree) | `~/.config/ive/<profile>/scratches/<project>/`, outside the repo; kept when a worktree is removed, never archived |
| Git log, stashes, branches | project (stashes are shared by all worktrees) | git |

No generated run configurations: only what is in the files.

## Window

- **No title bar.** The editor tab row is the top of the window; settings, minimize,
  maximize and close sit at its right end. Empty tab-row space is the drag area.
- **Left: tree** (sidebar on the `tree` colour, header: the project picker (see "Projects in the tree") + the
  folders menu + Select opened file + "Open project" menu with recent projects and "Open Folder…"). **Right: editor tabs.** Both header rows are 36 px. Tree rows are 24 px,
  Inter 13 px.
- Alt+F12 opens (or focuses) a terminal in the current tab's checkout. Escape closes menus.

## Tree

Only projects, worktrees, processes and files:

```
▾ A atlassian            ● 3 running
  ▸ dev
  ▾ GEN-817-ive
    ▾ Processes  2              [+ new terminal on hover; a click opens Run configurations]
        Claude Code
        ive
    ▸ internal-apps
      CLAUDE.md
▸ FI forge-installer  ⎇ main   (single checkout: branch pill on the project row)
▸ N notes  no git
```

- Click a **project** row → project overview tab; a **worktree** row expands and collapses (there is no worktree
  tab: its processes, changes and commits are its folders, see "Worktree folders").
- **Worktree row**: branch icon + name only (no colour dot, no change count, no ahead/behind, no hover action).
  A worktree's name is its folder's; the main checkout's is its branch.
- **Processes** folder is always present (even empty: no chevron then); its hover action, +, starts a terminal in
  that worktree; a click opens its Run configurations tab, its chevron folds it. There is no "New Claude Code" action
  anywhere.
- **Process rows**: icon by kind (terminal, Claude Code). Hover: Stop (Claude Code) or Close (terminal).
  A process that exits goes away.
- **Project row**: hover action Close project.
- **Folder rows** with a `.run.toml`: the folder icon with a run badge in its
  bottom-right corner (`folder-run.svg`, as IntelliJ's `nodes/module.svg` marks modules), no count; a click
  expands it (its runs are in Processes, see "Run configurations").
- **Virtual folders** (not on disk: `@hidden` for two or more hidden entries): icon + italic
  `@name` on a chip; `@hidden`'s is weaker (muted text, half-transparent chip).
- Middle click on a row: as a click, then the tab it showed is kept (see "Editor tabs").
- The tree's right edge drags to resize it (160 px at least, the editor keeps 320 px); double-click for 288 px.
- Not in the tree: databases, scratches, pins.
- Long child lists show 10 rows, then "Show 10 more · N of M".

## Editor tabs

- IntelliJ New UI style: the current tab is a rounded chip with a filled background and a thin
  blue outline (dark `#25324d` / `#35538f`, light `#e3ecfd` / `#a8c3f5`); others are flat with
  a rounded hover. The close × shows on the current tab and on hover only.
- Icon by kind (file type, overview, console kind, diff, commit, table, config). Scratches show
  their file-type icon with IntelliJ's scratch badge (clock) overlaid.
- No branch badge on tabs; the tooltip says `<branch or project> — <title>`.
- Diff tabs are underlined (2 px) in their worktree's colour.
- Tab kinds: file, diff, commit, scratch, git log, process (terminal, Claude Code, run), project overview,
  run values, settings.
- A new tab opens right of the shown one.
- **Temporary tabs**: a click opens a project overview, or Settings, temporary (title in
  italic). It goes when another tab is shown, and a newly opened tab takes its place; one at a time. Double click on
  the tab, middle click on its tree row, or Keep Tab keeps it. Files, processes and the rest open permanent. Restored tabs
  are permanent; a temporary tab is not saved.
- **Tab menu**: a right press on a tab opens it at the pointer, without showing the tab. Items, no icons:
  - Keep Tab: a temporary tab only.
  - Stop, Kill: a process tab only (terminal, Claude Code, run). Stop sends SIGTERM to the program in the foreground
    (a run's: SIGKILL 5 s later if it is still there), Kill SIGKILL at once. Greyed when nothing runs in the
    foreground (an idle shell, an exited run).
  - a separator, when any of the above shows.
  - a file's tab (a file, a diff): Send to Terminal ▸, Find in Files…, Copy Path ▸, Open In ▸, Git ▸, as in the tree's
    menu (see Menus), then a separator.
  - Close, Close Other Tabs, Close Tabs to the Left, Close Tabs to the Right; the last three greyed with nothing to
    close. When the shown tab closes, the right-pressed one shows.
  - Escape or a press outside closes it. Styled as the Open project menu, as wide as its items.
- Middle click on a tab closes it.

## Processes

- A **terminal** is the only thing the user starts by hand. Typing `claude` in it turns it into
  a **Claude Code** process (tab and tree row rename, prompt becomes "Ask Claude…"); `/exit`
  turns it back into a terminal. The real app detects the foreground process.
- Running a **run configuration** adds a run to its worktree's Processes, after the terminals and Claude Code, and it
  stays there until removed: running, waiting, exited, stopped or failed, with its state (see "Run configurations");
  its output is its tab.
- Shell prompt shows `user:dir (branch)$`.

## Overviews

Opened as an editor tab by clicking a project. No header block: the editor tab names
it. A row of overview tabs, with the location on the right end of that row (muted, monospace,
full path on hover). Each overview remembers its selected tab.

### Project overview — tabs: Worktrees · Git · Scratches (Databases: not in the app yet)

Right end: project path (`~/Projects/atlassian`, or `… · no git`).

- **Worktrees** (see "Hidden worktrees"; in the app since 2026-10-20): one filter
  box over three groups drawn as the hub's sections and rows, with its keys row under them. The filter keeps the
  keyboard: ↑↓ select a row (on theme.selected), Enter opens it, Space hides or shows the selected worktree or adds the
  selected branch's worktree, Escape drops the selection, then clears the filter. Typing clears the selection, so
  Space types a space again. After Space the selection stays in place: the row now at the same position in its
  group (the next branch), else the first row after the group. A worktree added from a branch (here, the hub, or a
  click) shows collapsed in the tree; New terminal's expands, as it opens a terminal there.
  - **Worktrees**: the main checkout, then the shown worktrees by last use. Row: the worktree's colour as a left
    stripe, branch icon, label (the branch too when it differs), folder, grey tags ("N changes"/"clean", ↑/↓,
    "● n running" in green), last use. A click shows it expanded in the tree; Hide on hover (not on the main checkout).
  - **Hidden** (collapsible, only when there are some): the same rows faded, Show on hover.
  - **Branches without a worktree**: newest first, 10 + paging; "local" or "only on origin" (faded icon). A click
    adds its worktree where `.ive.toml` says, as New worktree did; "+ Worktree" on hover.
- **Databases** *(not in the app yet)*: IntelliJ-style data-source tree.
  - Row: `[LOCAL|QA|PRD] <module>` + engine and notes (`read-only · personal · 🔑 wallet`).
  - Rows tinted by environment, children included: prod red, qa blue, local none.
  - Source → schema (`main` for SQLite, `public` for PostgreSQL) → `tables (n)` → table;
    clicking a table opens it in a tab (grid view).
  - Filter box; 102 sources page 10 at a time.
  - Data: per app module local (team), qa (team, over SSH), prod (personal, read-only).
- **Git**: summary (recent commits · stashes · branches without a worktree) with "Open Git"
  (log tab: log, stashes, worktrees, branches without a worktree); clickable **Log** (commit
  opens a commit tab); **Stashes** ("shared by all worktrees"). No pull requests — not a git
  concept.
- **Scratches**: search over names and contents; groups Today · This week · This month · Older
  (Older collapsed), 10 rows per group + paging. Row: icon, name, first line, age.
  Opening a scratch shows `scratch of <project> · <path> · runs in ⎇ <worktree>`, with Run for shell
  scratches; it runs in the current worktree (see "Where project-level things run"). Hover a row: Rename, Delete.

Worktrees have no overview tab (removed 2026-10-20): their git and processes are in the tree, see "Worktree folders".
For the main checkout the base is the remote: "Since base" against `origin/dev` means unpushed work.

## Run configurations

In the tree, a folder with a `.run.toml` has the folder icon with a run badge and nothing else: no `@run`, no
`● n`. Its runs are rows in the worktree's Processes, after the terminals and Claude Code (updated 2026-10-20):

```
▾ Processes  5                               [+]
    ⌨ Terminal
    ⧉ Build misc  exit 1                    hover: ▷ Start · × Remove
    ⧉ MiscApplication  ● running            hover: ↻ Restart · ■ Stop
    ⧉ IVE  ● running · :8767
    ⧉ Misc - Ngrok  … waiting for Build misc
```

- **Run row** (Processes): icon (command, or HTML for a static server), name, its run's state
  (`● running`, `● running · :8767` for a server, `… waiting for <step>`, `exit <code>` (red when not 0),
  `■ stopped`, `✕ <why the chain failed>`). Every run of the worktree is listed, live or not, until removed. Actions on
  hover: Restart and Stop while it is live, Start and Remove once it exited. A click shows the run's tab; Run,
  Restart and Start show it too and unfold the worktree's Processes. Configurations that never ran are only in the
  Run configurations tab (▷ Run).
- A broken file: its `.run.toml` row has a warning icon, the error as its tooltip; the folder is a plain one (no
  badge).
- There is no folder overview tab (removed 2026-10-20).
- Run with variables that have no value opens a **values tab**, `Run <name>`: one field per missing variable the
  run (and its `before` steps) uses, masked for secrets, with where it goes ("stored in the OS wallet; vars.toml
  keeps a reference", or "saved to vars.toml"); Run (closes the tab and starts the run) and Cancel.
- From the tree, a run's output is its tab (read-only, no Stop inside); Run, Restart and Start there show it.
  `before` steps run as their own runs (own rows and tabs); the run waits ("Waiting for Build misc…") and starts
  when each exits 0.
- One run per configuration: starting it again (Run, Restart, Start, a `before` step) removes its exited run, row
  and tab, and adds the new run on top.
- **Run configurations tab** (proposal, 2026-10-20): a click on a worktree's Processes row (its chevron still folds
  it) opens `Run configurations` as a temporary tab: every configuration of every `.run.toml` in that worktree.
  - Bar: a filter (name, folder, command), **Flat | Tree**, the worktree and its folder on the right; under it a
    summary (`8 configurations in 3 files · 2 running · 1 broken file`).
  - Rows are the tree's rows (24 px, 13 px, same chevrons, hover and actions), in both layouts, at most 600 px wide.
  - Flat: one row per configuration by name, its folder muted after it. Tree: folders as the Files tree nests them
    (a chain of single folders is one row), each with `● n` live, folding; a folder's `.run.toml` opens from its
    hover action; its configurations in file order.
  - A configuration row: icon, name, the run's state, ▷ Run before it ever ran, Restart + Stop or
    Start + Remove on hover; the command in its tooltip (name, `run.<key>`, the command with built-ins and plain values
    filled in and secrets left as `${name}`, `serve <root> → :<port>` for a server, then the run's status and start). Runs started here keep the list shown; a click on a row
    with a run shows its output (the list, being temporary, goes).
  - A broken file is a `.run.toml` row with a warning, the error as its tooltip (last in Flat, in its folder in
    Tree); a click opens it.
- Mock data: `run/.run.toml` (repository-wide: Build all, Dep Check with a wallet secret), `internal-apps/misc`
  (build, migrate, MiscApplication with `before` and a missing secret, ngrok with a default), `internal-apps/ive`
  (IVE, and the mock itself as a static server with live reload), `lib` (a broken file). At scale: the `platform`
  project (`checkouts-data.js`, Faker, seeded) has 1000 configurations in 250 folders (`services/<domain>/<name>`,
  `apps/`, `tools/`, `infra/<env>`), each folder's stack (Maven, npm, Go, Cargo, Compose + Terraform) giving 2 to 7.
  The tab lists a checkout's own `.run.toml` files only.

## Where project-level things run

Scratches and tables resolve to: the current tab's checkout, else the project's last focused
worktree, else its first open checkout.

## Scale rules

- Built for a monorepo: 102 data sources (not in the app yet), 600 scratches (`checkouts-data.js`, Faker, seeded).
  Run configurations scale by folder: the tab lists a checkout's `.run.toml` files by folder.
- Every long list shows **10** entries with "Show 10 more", "Show all" and "or refine the search".

## Worktree folders

Under a worktree (or a project with a single checkout), its content sits in folders: icon + name, as any folder,
no chip and no `@` (`@hidden` keeps its). In this order:

| Folder | Holds | Shown |
|---|---|---|
| Processes | terminals, Claude Code, then every run of the worktree with its state (`● running · :8767`, `exit 1`; Restart and Stop, or Start and Remove, on hover); a click opens the Run configurations tab; its + (on hover) starts a terminal | by default |
| Uncommitted | what `git status` reports | by the folders menu |
| Since base | every file the branch changed since it left its base: its commits' files and the uncommitted ones | by the folders menu |
| Commits | commits since the base, newest first: the first 10, then `Load 20 more · 10 of 32` adds 20 each click | by the folders menu |
| Files | every file, as above | always |

- **No badges** (removed 2026-10-22): the worktree row has none; the header's folders menu and Ctrl+Alt+1 to 5 switch
  folders, in every worktree at once. A switched-on git folder with nothing in it keeps its row, muted (icon faded), without a
  chevron, `· 0` after its name (`· no base` for Since base and Commits of a branch without a base) and why in its
  tooltip (`No commits since dev`). An empty Processes has no chevron; its + still starts a terminal. The folder still
  folds on its own, per worktree.
- **Changed files** (in Uncommitted, Since base and under a commit) show their mark after the name (M, A, D, R, U),
  a deleted one struck through; a click opens its diff (uncommitted, since base, or the commit's).
- **Commits**: the message without the prefix most of them share (`[GEN-817] IVE - Integrated Vibe Editor - `), then
  `sha · age`; a click (or its chevron) folds its files out and in.
- **Flat or grouped**: changed files are listed flat or grouped (the settings' `[git_files]` layout): Flat, a row per file by path, its
  folder after its name; Tree, folders as its Tree layout groups them (a chain of single folders is one row, each
  with its file count, none when it holds one file). Folders open shallowest first while the list shows at most 20
  rows; one you open or close stays so. **Alt+Shift+G** flips it from anywhere in the window.
- **Counts**: a folder's count follows its label as `· 4`, 11 px in the secondary colour at 75% (no pill), so it
  reads as a note, not a button.
- **Header**: the project picker (see "Projects in the tree"); its folder button sets the folders every
  worktree shows (see "Folders for every worktree").

Explored and dropped (2026-10-20): the header title switching whole views (Files / Changes / Commits / Processes,
Alt+1…5); then a header menu of layer ticks; `@name` chips, uppercase section headers and badge-opened headers for
the folders.

## Menus

A right press on a tree row, or on a tab, opens its menu at the pointer (`menus.js`; workspace/menu.rs). Both are
declared in Rune, `ive.rn`'s `config.tree_menus.main` and `config.tab_menus.main`, which `init.rn` edits, replaces,
or adds to: its own items run a Rune function, submenus nest, and `when` (kind, name and path globs, count, or an
async predicate that fills the open menu in) leaves items out. Items that do not apply are left out, then the
separators left dangling and the submenus left empty. A submenu opens beside its line (`▸`) on hover or click.

A row's menu applies to its file or folder (the selection in the app; the mock has no multi-select), a worktree or
project row to its root folder, a changed file to the file:

- **Send to Terminal ▸**: the terminals and Claude Code sessions, most recently used first (shown, or Enter pressed
  in them), each `title · worktree`, then New Terminal in the worktree. The path is pasted into the chosen one, which
  is shown; nothing is pressed. Relative to the terminal's folder (`../` included) when it works in the path's
  worktree, else absolute. A drag onto a terminal and the hub's Alt+Enter type paths the same way.
- **Open in Terminal**: a terminal in the folder (a file's folder).
- **Run**: a `.sh`, `.bash` or `.zsh` file runs as a shell scratch does; a folder with a `.run.toml` lists its
  configurations.
- **Find in Files…**: the hub searching the contents of the file or folder (see Hub, Contents).
- **Copy Path ▸**: Copy Absolute Path, Copy Relative Path (to the worktree), Copy File Name.
- **Open In ▸**: Reveal in Files (Finder), Open in Default App (files), Open in Browser (`.html`, `.htm`, `.svg`),
  Open in New Window (files). The mock does nothing for them.
- **Git ▸**: Show Diff (an uncommitted file), Compare with Base Branch (a file changed since the base), Show History
  (a Git log tab, `History · <name>`, of the commits that changed the path, following a renamed file; the mock lists
  the project's log), Add to .gitignore (tree only; anchored lines, `/path`, `/folder/`).
- **Compare Files**: two selected files in a diff tab (not in the mock: no multi-select).

## Folders for every worktree (in the app since 2026-10-22)

Each worktree's badges are its own, so showing Commits everywhere took a click on each worktree. Now there is a set
of folders **every worktree** shows (Processes and Files in a new profile; the mock starts with all five, changed files in Tree), which new worktrees get too:

- **Projects header**: a folder button left of Select opened file opens a menu, "In every worktree", with a tick per
  folder (Processes, Uncommitted, Since base, Commits, Files); a click switches that folder in every worktree. Under
  them, one row "Changed files" with a Flat | Tree switch (the run configurations tab's), the layout Alt+Shift+G
  flips (see "Worktree folders"). Each row shows its shortcut on its right, muted, as `init.rn` binds it:
  **Ctrl+Alt+1 to 5** (`act::ToggleFolderEverywhere`, window scope, as Alt+Shift+G), Alt+Shift+G after the
  switch. Not Alt+Shift+1…5: with Shift a digit key arrives as `!`, `@`, … depending on the keyboard layout.
- **Badges** on the worktree row (a click: this worktree only; Alt+click: every worktree; a dot when set apart) were
  removed on 2026-10-22: the menu is the one way. A worktree set apart earlier keeps its own set until Reset.
- Switching a folder for every worktree also ends the worktrees set apart on that
  folder, so every worktree then shows what the click showed. When some are set apart, the menu ends with
  "Reset N worktrees set apart" (the worktrees in its tooltip), which ends them all.
- In the app: the set is `shown_folders` in `state.toml`; a checkout keeps its own full set as before, and one nobody
  changed follows every worktree's.
- Explored and dropped (2026-10-22): Alt+click without the menu (hard to find, no reset); one set for all with no
  worktree set apart (`global`); the set in `init.rn` as `config.shown_folders`.

## Projects in the tree: all, one or a few (in the app since 2026-10-22)

With several projects open, the tree is long and the hub mixes them all. The header's title, which said
`Projects`, is a picker that works as the folders menu: **All projects ▾**, the one project shown with its badge
(`A atlassian ▾`), or `2 of 4 projects ▾`.

- A click opens a menu under the header, as wide as the tree: All projects (with how many are open, ticked while all
  show), then each open project with a tick while it shows, its badge and `● n running`.
- A project's row shows it or leaves it out; the menu stays open. The last project shown stays. All projects brings
  every one back. Projects left out stay open (their tabs, terminals and runs carry on); they are only out of view.
- **The hub follows it**: its "all" scope is the projects shown (the scope chip and the `:` list's first row read
  `All projects`, the one project, or `2 of 4 projects`); with one shown, Tab's stops and the `:` list skip its
  project row. Recent, Changed, Running, file search and New terminal leave the other projects out. A worktree scope
  still narrows within its project.
- Opening a project shows it; closing one forgets it; Select opened file on a tab of a project left out shows it.
  Alt+F12 with no tab opens a terminal in the tree's first project.
- Kept per profile as the projects left out (`hidden_projects` in `state.toml`), so a project opened later shows;
  the mock keeps them in the hash (`hidden=`).
- Explored and dropped (2026-10-22): one project at a time (picking it showed it alone).

## Hub (Ctrl+E)

IntelliJ's Ctrl+E (Recent Files), widened into one place for everything recent and for finding any file. Mocked
in `hub.js`, `hub.css` and `components/checkouts/hub.html`; it searches the monorepo's real 22k tracked paths
(`data/files.gz`, regenerated by `tools/hub-files.sh`), with made-up contents.

- **A temporary tab** (italic, `Hub`): Ctrl+E opens it in the temporary slot, opening an item replaces it. Kept
  (double click, Keep Tab), it stays as a dashboard.
- **Escape steps back**: from the worktree question to the list, then clears the search, then (search empty) leaves
  the hub for the tab shown before.
- **Keyboard first**: the query always has the keyboard. ↑↓ (or Ctrl+J/K, Ctrl+N/P, PgUp/PgDn) move, Enter opens,
  Alt+1…9 opens the numbered row, Shift+Enter opens behind and stays (opens several), Alt+Enter types the path into the
  last terminal (the most recently used worktree's copy), Ctrl+Y copies it, Tab cycles the scope, Ctrl+E again toggles "changed only" (IntelliJ's second Ctrl+E).
- **Scope** (chips right of the query): all projects, a project or a worktree. Tab cycles All → the back tab's project
  → its worktree (its own, else the most recently used) → a scope picked with `:`; `:` (or the `:…` button)
  lists the open projects with their worktrees nested, most recently used first, Enter or Tab picks one and clears
  the query; Backspace on an empty query goes back to All. In a worktree scope, files open there without asking, and
  Recent, Changed and Running show that worktree's rows plus the project's own (scratches, commits). Kept in the
  hash (`scope=`). `:42` instead goes to line 42 of the tab shown before the hub (as VS Code's Ctrl+P); `file:42` and
  `:42` scroll the editor to the line and mark it.
- **Worktrees**: a row in one worktree is marked with it (its colour and name) and opens there: Recent tabs, changes,
  processes, files of a single-checkout project. A row in several (a file or a run configuration of a project with
  worktrees) says `4 worktrees`, and Enter asks: the worktrees, most recently used first (age, or `never`), with
  their state for that row (`● open`, `running · :8080`), the preview showing the picked one's copy; typing filters,
  Enter (or the first row: Enter, Enter) opens there, Shift+Enter carried over opens behind.
- **Worktree recency** is the last time the worktree was used: any of its tabs shown (file, terminal, Claude Code,
  run output, overview, diff; read from the hub's history of shown tabs), input typed into one of its terminals or
  Claude Code sessions, or a run started in it (also when a tab elsewhere is shown, e.g. a session working
  in the background). In the app both go into state.toml (history bounded), so the order survives a restart.
- **Empty query**: three sections in one list.
  - *Recent*: every kind of tab (files, scratches, processes, overviews, diffs, commits, Git log,
    Settings), newest first, with its worktree chip and age; open tabs marked `● open`, closed ones greyed and
    reopened by Enter. Processes that ended drop out.
  - *Changed*: uncommitted files of every worktree of every open project, with their mark.
  - *Running*: live processes and runs.
- **Typing searches files** of all open projects (and scratches) with fzf's matching: v1 fuzzy scoring with
  path-scheme boundaries, smart case, extended syntax (`'exact`, `^prefix`, `suffix$`, `!not`, space = AND,
  `a | b`), matched characters highlighted, counter `matched/total · ms` (about 5 ms over 23k paths in the mock). One
  row per project path, not per worktree (see Worktrees). Up to five matching Recent rows come first; Files skips those. `file:line` opens at the line.
- **Excluded files** (below folders named in `init.rn`'s `config.excluded`, default `target`, `node_modules`,
  and gitignored ones) are not searched unless the `Excluded` switch (Alt+I) is on; while off, the key strip says how
  many were left out. Their rows carry an `excluded` or `ignored` tag in the tree's excluded colours. The mock has a
  sample of the repo's real ignored files (150 per ignored folder, listed in GEN-817-ive, so tagged with it).
- **Prefixes** switch what is listed: `>` actions (New terminal, New scratch, Git log, Close Other Tabs, theme…),
  `@` processes and run configurations (Enter asks the worktree, then shows its live run or starts one), `#` worktrees and branches (Enter
  on a branch without a worktree creates it).
- **Preview** on the right, as `fzf --preview`: the file's text (line highlighted for `file:line`, from a few lines
  above it), a change's diff, a process's last lines, a worktree's summary.
- **Contents** (Ctrl+Shift+F from anywhere, or in the hub; a menu's Find in Files… on its file or folder): the query
  searches the files' lines instead of their names. Plain text from 2 characters, smart case; Alt+R (`.*`) a regex,
  Alt+W (`W`) whole words; an invalid regex says why. One row per matching line, `path:line` and the line with its
  matches highlighted (a long line cut around its first match); Enter opens the file at the line, the preview shows
  the lines around it. It searches the scope's worktrees and, for All or a project, the scratches; or the files and
  folders picked in a menu, named in the section (`Contents of src +1`), which Backspace on an empty query drops.
  Gitignored and excluded files only with Alt+I, but a folder picked on purpose is searched even when ignored itself.
  Counter `n matches in m files`; the list stops at 2,000 lines. The mock searches the text its files and scratches
  have.

Further ideas, not mocked:

- Frecency instead of plain recency; pinned rows on top of Recent.
- Recent locations (IntelliJ's Ctrl+Shift+E): the places edited, with a line of context each.
- Jira keys: `GEN-817` finds the branch, its worktree, its commits and the ticket.
- Commits by subject (`~`), stashes; recently closed projects to reopen.
- Search runs off the UI thread over an index kept by the OpenPaths watcher; the list streams like fzf does.

## New terminal (Ctrl+Shift+T)

Starting work somewhere, often on a branch that has no worktree yet. Ctrl+Shift+T (window scope, so it works in a
terminal too; Ctrl+N stays the shell's next-history) opens the hub as a temporary **New terminal** tab that only asks
where the terminal goes. One list, filtered fzf-style as the hub's:

```
atlassian        worktrees by last use, then the newest branches without one          38/38
  dev                atlassian · main checkout
  GEN-817-ive        atlassian                 ● 2 running   now
  IDP-30-…           atlassian · IDP-30-Reserved-…            3 d
  OHQ-330-bandwidth  local · atlassian                        now
  GEN-747-monitor    origin · atlassian   (faded icon)        yesterday
  … (5 branches)
  … 25 more branches; type to search them
forge-installer
  main               forge-installer · main checkout
  live-view          local · forge-installer
```

- Before typing, a section per open project: its main checkout, its other worktrees most recently used first, then
  its 5 newest branches without a worktree (newest commit first) and "… n more branches; type to search them". A
  project without git has only its own row.
- Typing ranks every project, worktree and branch together with the hub's fzf matcher (same syntax: `'exact ^start
  end$ !not a | b`, over "name detail"), best first, one flat list of the 20 best ("… n more; type more to
  narrow"). The counter at the right of the query row says matched/all, as the files counter does.
- Branch rows say where the branch is: `local`, or the remote's name (`origin`) when only the remote has it; a
  remote-only row has a faded icon. A remote branch whose name a local one has is left out (the worktree would check
  out the local one). Typing `origin` narrows to remote-only branches.
- Enter on a worktree: a terminal there. Enter on a branch: New worktree (a local branch tracking the remote first,
  when remote-only; no fetch, no fast-forward), then a terminal in it. Either way the tab shows the terminal and the
  tree shows its worktree expanded, the terminal under its Processes. A failure (folder exists, git error) stays in
  the hub as a note and opens nothing.
- The preview says what Enter does: the processes already running there, or the worktree's folder and that it comes
  from `.ive.toml` or the default.
- Keys: ↑↓ move, Enter, type to filter, Escape clears then goes back, Ctrl+E switches to the full hub. The query
  prefixes (`>`, `@`, `#`, `:`) do not apply here.
- The mock also takes Alt+Shift+T (browsers keep Ctrl+Shift+T); `#tab=newterminal` opens it.

## Hidden worktrees

A worktree with work parked for later clutters the tree. Hiding it changes nothing on disk (files, branch, processes
stay); it leaves the tree, the hub (search scopes, changes, file choices) and New terminal. Only the project page's
Worktrees section lists it, under Hidden, with Show. Its open tabs (files, terminals, Claude Code) stay open. The main
checkout cannot be hidden. Hide is on the worktree's tree row (on hover) and on its project page row. When all but the
main checkout are hidden the project drops its worktree level, as with a single checkout. Kept per profile (in the app:
saved state); the mock starts with GEN-811 hidden and does not keep changes on refresh.

## Settings › Scripts (proposal)

Settings has Appearance, Scripts and Third-Party Software; the Editor › Color Scheme pages are gone (colours are set
in `init.rn`, the defaults are in `theme.toml`). Scripts is a reference to hand to a coding agent, which then
customises IVE by editing `init.rn`:

- **Prompt for a coding agent**: one block, the profile's real paths filled in: what IVE's script files are and
  which one to edit, the override rules, and how to check a load (MCP `script_status`, or `scripts.log`). Its last
  line is `Task: …`; starter chips (Bind a key, Mouse binding, Colours, Bigger UI, Function, Terminal keys) fill it
  with an example. Copy takes the block as shown.
- **Files**: `init.rn`, `ive.rn`, `theme.toml`, `scripting.md`, `scripts.log`, `settings.toml`: name (opens it),
  absolute path, role, Copy path.
- **Quick reference**: folding blocks (Keys and mouse open, Actions, Functions, Theme, Fonts/excluded/tabs), each
  Rune or a plain table, each with Copy.
- All text on the page is selectable (drag, double click a word) and Ctrl+C copies the selection.
- New: IVE writes `scripting.md` next to `ive.rn` at each start, read-only, as it does `ive.rn` and `theme.toml`:
  the README's Scripting, Excluded folders and Theme sections, so an agent has the reference of the running version
  without the repo.

## Removed during the design (do not implement)

Layouts 1–3 (git on rows, git folder, git tab) · title bar and its run widget · terminal icon
in the top bar · pins/stars · run configurations, databases and scratches in the tree ·
global scratches · branch-linked scratches and their auto-archive · pull requests / CI ·
worktree colour dots (tree, tabs, cards' dot) · change counts and ahead/behind in the tree ·
branch badge on tabs · overview header (icon, name, path, "N running") · "New Claude Code"
actions · Stop button inside the run tab · run configurations as `<module>/.ive/run/*.toml`, personal run
configurations, and the worktree overview's Run configurations tab (filter, kind chips, module groups, "changed
here"): replaced by one `.run.toml` per folder and its `@run` in the tree (dropped 2026-10-20: runs are in Processes) · a ▶ at a
run-configuration folder's right end: the run badge on its icon instead · the missing-values form inside the folder
overview, and as a prompt over the window: a values tab instead · `@processes` under `@run` and run rows apart from
their configurations (definitions with their command, runs with their status, one step left): one row per
configuration carrying its run instead · `@hidden` chip variants (the same chip, an outline, text only) · Settings'
Editor › Color Scheme pages (read-only once colours moved to `init.rn`).

## Open points

- Worktree colour still appears on worktree cards (left stripe), the overview's section underline and the
  diff tabs' underline.
- "● N running" remains on the project tree row and worktree cards.
- Databases: IntelliJ's toolbar and "1 of N" database counts are not mocked.
