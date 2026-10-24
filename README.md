# IVE - Integrated Vibe Editor

A desktop editor built on [gpui](https://crates.io/crates/gpui) (Zed's UI framework): several projects with their
git worktrees, processes, changes, commits and files in one tree; a project overview tab with read-only git views;
coloured code; scratches; terminals that know when they run Claude Code; and an MCP server so agents can drive it.

IVE is source-available, not open source: free for personal and noncommercial use, a paid licence per user for
commercial use (an employee using it for work included), and no redistribution. See [Licences](#licences).

## Run

```sh
./run.sh .                                   # open the current folder
./run.sh -- --no-mcp --background ~/code     # IVE's own flags go after --
./run.sh --release -- ~/code                 # flags before -- go to cargo
```

`run.sh` builds and starts IVE and names any missing system library. Building needs Rust (version in
`.tool-versions`) and, on Debian/Ubuntu:

```sh
sudo apt install libxkbcommon-dev libxkbcommon-x11-dev libfontconfig-dev libfreetype-dev libwayland-dev \
  libxcb1-dev libpipewire-0.3-dev libclang-dev
```

On macOS the Xcode Command Line Tools are enough (`xcode-select --install`): gpui compiles its Metal shaders at
startup, so the `metal` tool of a full Xcode is not needed. Settings, state and scratches live in
`~/Library/Application Support/ive/<profile>` there.

To install IVE for yourself, run `./install.sh` once from this checkout. It builds IVE, writes the launcher
`~/.local/bin/ive`, adds IVE to the desktop (Linux: an "IVE" entry in the application launcher of GNOME, KDE Plasma
and other freedesktop desktops; macOS: `~/Applications/IVE.app`, for Finder, Launchpad, Spotlight and the Dock) and
starts IVE as a profile named after your user, with the [Restart](#restart) button, which rebuilds this checkout.
Run it again to rebuild and reinstall.

| Flag | Meaning |
|---|---|
| `FOLDER` or `FILE` | open the folder's project (the repository's, for any worktree or subfolder) next to the saved ones; a file opens with it |
| `--mcp PORT` | serve MCP on `127.0.0.1:PORT` (1024-65535) from now on, instead of the remembered port, see [Drive IVE from an agent](#drive-ive-from-an-agent-mcp) |
| `--no-mcp` | do not serve MCP |
| `--background` | open the window without taking focus |
| `--stop-processes-on-quit` | quit without the quit prompt, stopping the running terminals and processes instead of leaving them running; being replaced by a [restart](#restart) still keeps them |
| `--restart SCRIPT` | show a Restart button left of Settings that rebuilds and restarts IVE with SCRIPT, see [Restart](#restart); Linux and macOS |
| `--profile NAME` | run as profile NAME (default `default`): its own projects, tabs, settings, scratches and terminals, see [Profiles](#profiles) |

## Use

| Action | How |
|---|---|
| Open a project | + in the sidebar header: a recent project, or Open Folder…; IVE reopens the projects you had open |
| Close a project | its row's × (hover); with processes running, IVE asks first; it moves to Recent |
| Overview | click a project row (Worktrees, Git, Scratches); a worktree row expands and collapses |
| Review a worktree's changes | its folders in the tree (Worktree folders below): Uncommitted, Since base, Commits; click a file for its diff tab |
| Browse history | project overview → Git: the last commits and the stashes; Open Git for the Git log tab (Log, Stashes, Worktrees, Branches); click a commit for its tab |
| Worktrees list | project overview → Worktrees: the main checkout, the worktrees by last use, the hidden ones and the branches without a worktree, as the hub lists them; the filter keeps the keyboard: type to filter all of them, ↑↓ select, Enter shows a worktree in the tree, Space hides or shows it, Escape drops the selection, then clears the filter |
| New worktree | in the Worktrees list, Enter, Space or + Worktree on a branch: it is checked out in `<worktrees>/<branch, / as ->` (no git hooks run), where `worktrees` comes from `.ive.toml` in the main checkout (a folder inside it; default `.worktree`); the new worktree shows collapsed in the tree, and Space leaves the selection on the next branch |
| Hide a worktree | the eye on its tree row (hover), or Space / Hide on its row in the Worktrees list: it leaves the tree, the hub and New terminal until Show (or Space) on its row under Hidden; its tabs and processes stay; the main checkout cannot be hidden; kept in `state.toml` |
| Scratches | project overview → Scratches: search names and contents, New scratch, hover a row to Rename or Delete; a scratch is a file in `~/.config/ive/<profile>/scratches/<project id>/`, kept when worktrees go |
| Edit a scratch | type in its tab; it saves itself half a second after you stop, and when the tab closes or IVE quits; a change made outside IVE reloads it, or asks (Reload / Overwrite) when you have unsaved edits |
| Run a shell scratch | Run in the header of a `.sh`, `.bash` or `.zsh` scratch: a new terminal in the project's last used worktree runs it with `sh`, `bash` or `zsh` |
| Open a file | click it in the tree; an open file shows its existing tab; or Ctrl+Shift+E (see [Hub](#hub-ctrlshifte)) |
| Edit an uncommitted change | in a diff tab of uncommitted changes, type on the new side (added and context lines; side by side, the right panel): the file in the work tree is edited and saved as a file tab saves it, and the rows are always the diff of the text you edit (HEAD as checked out against the text read or saved). A change made on disk outside IVE is taken in while nothing is unsaved; with unsaved edits the rows keep showing them and the next save meets the change. A file that cannot be read to edit leaves the diff read-only, saying why in its header (Reload tries again). Caret and undo outlive the new diff; when a change you undid disappears, the caret goes to the nearest line left. Since base and commit diffs, and the hub's preview, are read-only. While a file's tab and a diff tab of it are both open, one with unsaved edits makes the other read-only (it says why) until its edits are saved |
| Edit a file | type in its tab; there is no save: edits reach the disk half a second after you stop (longer for files over 8 MiB), and when you switch tabs or windows, run something, close the tab or quit; closing and quitting wait for the save, and a tab whose edits cannot be saved stays open with the reason. A change made outside IVE reloads the tab, unless it holds edits: then the save stops with a bar (Reload / Overwrite; Reload only for files over 8 MiB). A symbolic link stays: the file it points at is written. In a file tracked by git, the gutter marks the lines changed since HEAD (as the Uncommitted diff shows them): green bars on added lines, blue on changed ones, a grey triangle where lines were deleted; they follow your typing and catch up a moment after you stop, after a change on disk and after a commit (no markers for untracked files, files over 8 MiB, scratches and diffs). Click a marker for its popup: the lines as they are in HEAD, ↑ / ↓ to the previous / next change (round from the last to the first), Rollback (puts HEAD's lines back as one edit; Ctrl+Z undoes it), Show diff (the file's Uncommitted diff at that change), Copy (HEAD's lines) and a "Commit this change" field: type a message, Enter (or the send button) commits that change alone, as it is in the editor, on HEAD; other changes and other staged files stay out, and hooks and signing apply (IVE runs `git` for it). It is refused, with the reason in the popup, with an empty message, while the file has staged changes or is staged as new, for a file with a git `filter` or `working-tree-encoding`, during a merge, rebase, cherry-pick or revert, and when HEAD moved since the markers were found (a commit landing while IVE commits is taken off again; the work tree is never touched). What goes wrong during the commit itself (IVE's commit taken off again, or left on another commit because taking it off failed, or made but not found in HEAD's reflog) shows in a bar above the text with what to run, until you dismiss it (✕) or a commit goes through. One commit at a time per file: the field waits for the answer, also after Esc and reopening; a failure shows in that change's popup, now or when it is next opened. Hooks that stash or rewrite files change them on disk during the commit, which the tab sees as an outside change. Esc, a click elsewhere or typing outside the change closes it |
| Hub | Ctrl+Shift+E (Cmd+Shift+E on macOS): recent tabs, uncommitted changes, running processes, and a search over every file of every open project; see [Hub](#hub-ctrlshifte) |
| Select and copy in a file or a diff | drag, double-click (a word), triple-click (a line) or Shift+click (extend); Ctrl+A selects all, Ctrl+C copies (Cmd on macOS); a diff copies its code lines, without line numbers and +/- |
| Copy full paths | Ctrl+Shift+C (Cmd+Alt+C on macOS) in the tree copies the absolute paths of its selected files and folders (a click, Ctrl+click, Shift+click), one per line; in the hub, the selected row's |
| Find the shown tab in the tree | the crosshair left of + in the sidebar header expands the tree to its file, worktree or process and selects it |
| Open a terminal | Alt+F12 opens one in the current tab's checkout; + on that checkout's Processes row (on hover); Ctrl+Shift+T (Cmd+Shift+T on macOS) asks where: see "New terminal anywhere" below |
| Worktree folders | under a worktree (or a project with one checkout): Processes (terminals, Claude Code and every run; a click opens its Run configurations tab), Uncommitted, Since base, Commits, then Files. Its + (on hover) starts a terminal in that worktree; an empty Processes has no chevron. **Which folders show**: the folder button in the sidebar header opens a menu with a tick per folder (switches it in every worktree and in worktrees opened later; Processes and Files at first), each with its key (Ctrl+Alt+1 to 5 by default, in this order, as `init.rn` binds `act::ToggleFolderEverywhere`), a Changed files Flat \| Tree switch (Alt+Shift+G), and Reset N worktrees set apart while some worktree keeps a folder set of its own from before. A switched-on git folder with nothing in it stays, muted, without a chevron, with `· 0` (`· no base` without a base branch) and why in its tooltip. A changed file opens its diff tab; a commit folds its files out and in; Commits lists 10, then Load 20 more adds 20 each click. Since base includes the uncommitted files. Alt+Shift+G lists changed files flat or grouped by folder; grouped, a list opens its folders shallowest first while it shows at most 20 rows, and a folder you open or close stays so. Shown and folded folders, every worktree's set, open commits and change folders, Load more and the expanded folders of Files are kept in `state.toml` |
| Show some projects | the sidebar header's title, All projects ▾, picks what the tree and the hub show: a tick per open project (a click shows it or leaves it out; the last one shown stays) and All projects. It reads the one project shown, or `2 of 4 projects`. Projects left out stay open (tabs, terminals, runs); one opened shows; kept in `state.toml`. The hub's All scope is the projects shown, and its search, Recent, Changed, Running and New terminal leave the others out |
| Settings | Ctrl+, (Cmd+, on macOS), or the gear at the right of the tab row |
| Switch or close tabs | click a tab; middle click, its × or Ctrl+Shift+W (Cmd+W on macOS) closes it, and closing a process tab keeps the process running (in Processes); Ctrl+W stays the program's (a shell deletes a word) |
| Stop a process | Ctrl+Shift+Q (Cmd+Alt+W on macOS) on its tab, the tab menu's Stop, or the Stop button of its row: the tab closes at once and the row reads `stopping…`; the shell or run gets a hang-up and SIGTERM (to its process group and the foreground program's), SIGKILL 5 s later, and its row goes once it is gone (a run's configuration row goes back to ▷ Run). One still running 2 s after SIGKILL is left to itself: IVE forgets it and a tab names its pid. On any other tab Ctrl+Shift+Q just closes it |
| Tab menu | right click on a tab (other clicks and menus: `config.tab_menus` in [Scripting](#scripting)): Keep Tab (a temporary tab), Stop (a process tab: as Ctrl+Shift+Q), then for a file's tab (a file, a diff, a scratch) Send to Terminal, Find in Files, Copy Path, Open In and Git as in the tree menu, then Close, Close Other Tabs, Close Tabs to the Left / Right |
| Tree menu | right click on a row (`config.tree_menus` in [Scripting](#scripting)): on a selected file or folder it applies to the whole selection, on another row to that row alone (a worktree or project row: its root folder; a changed file: the file). New Terminal (a Processes folder: a terminal in its worktree, as its +), Stop (a process row: ends it for good, as its Stop button; a running run's row: stops the run), Send to Terminal ▸, Open in Terminal (a terminal in the folder), Run (a `.sh`, `.bash` or `.zsh` file as a shell scratch runs; a folder's `.run.toml` configurations), Find in Files…, Copy Path ▸ (absolute, relative to its worktree, file name; one per line), Open In ▸ (Reveal in Files / Finder, Open in Default App, Open in Browser for `.html`, `.htm`, `.svg`, Open in New Window), Git ▸ (Show Diff of an uncommitted file, Compare with Base Branch, Show History of a file or folder, following a renamed file, Add to .gitignore), Compare Files (two files). Items that do not apply are left out; ↑ ↓ Enter → ← Escape work in it |
| Send to Terminal | the menu's submenu lists the terminals, most recently used first (shown or Enter pressed in it), then New Terminal in the worktree: the paths are pasted into the chosen one, which is shown, without Enter. They are relative to the terminal's current folder (`../` included) when the terminal works in the worktree of every path, else absolute; a drag onto a terminal and the hub's Alt+Enter type paths the same way |
| Find in a file | Ctrl+F (Cmd+F on macOS) in a file, scratch, diff or commit tab: a box at the top right of the code with the selection (on one line) or the word at the caret, else the last query; every match marked (and on the scrollbar), `3 of 12`. Enter / Shift+Enter in the box, F3 / Shift+F3 anywhere in the code (macOS also Cmd+G / Cmd+Shift+G) and ▲ ▼ go to the next or previous match, round at the ends, and select it; Alt+C, Alt+W, Alt+R in the box (macOS Cmd+Alt+C, W, R) or Aa, W, .* match case, whole words, regex; Esc or × closes it and leaves the match selected. A side-by-side diff searches the side last clicked; a match in a collapsed commit file opens it. F3 with the box closed opens it with the last query |
| Find in files | Ctrl+Shift+F (Cmd+Shift+F on macOS), or Find in Files… in a menu for the picked files and folders: the hub searching file contents, see [Hub](#hub-ctrlshifte) |
| Temporary tabs | a project or a worktree's Processes clicked in the tree, Settings and the hub open in a temporary tab (italic title) that closes once another tab is shown; middle click on the row, or double click on the tab, keeps it; temporary tabs are not restored at the next start; see `config.temporary_tabs` in [Scripting](#scripting) |
| Reorder tabs | drag a tab; a bar between tabs marks where it lands (past the last tab moves it last) |
| Tab in its own window | drag a tab out of the tab row (a small picture of it follows the pointer) and release it: it opens in a window of its own under the pointer; the window's tab-row button (Move to main window) puts it back, closing the window closes the tab; detached tabs come back in the tab row at the next start |
| Select and paste in a terminal | drag, double-click (a word) or triple-click (a line) to select; the selection goes to the primary selection, which middle click pastes; Ctrl+Shift+C and Ctrl+Shift+V (Cmd+C, Cmd+V on macOS) copy and paste |
| Run configurations | a folder's `.run.toml`, see [Run configurations](#run-configurations): its runs are listed in the checkout's Processes; a click on Processes lists every configuration of the worktree |
| Change keys and mouse clicks | `init.rn` next to IVE's `ive.rn`, see [Scripting](#scripting) |
| Scroll a long tab strip | Shift+wheel, or a touchpad |
| Colours | files up to 8 MB in about 60 languages (see [Languages](#languages)) are coloured as in IntelliJ's New UI themes, including code embedded in other code; bigger or other files show plain text |

Colours are changed in `init.rn`, per theme under `config.theme.dark.syntax` and `config.theme.light.syntax` (see [Theme](#theme)).

Terminals run your login shell in their checkout's folder (the home folder outside projects), with
`TERM=xterm-256color`. Typing `exit` ends the process and closes its tab. Typing `claude` turns the terminal into a
Claude Code process (tab and tree row rename); Stop sends it SIGTERM and leaves the shell. Open projects, their
expansion and Recent are kept in `~/.config/ive/<profile>/state.toml`; a file IVE cannot read is moved to `state.toml.bad`.
So are the window's place and size and whether it is maximized or fullscreen: the next IVE, a restarted one too, opens
the window as it was, or centred when that place is no longer on any screen.

On Linux and macOS, terminals keep running when IVE closes, and come back the next time it starts: each shell runs in a small
host process of its own (`ive --pty-host`), and IVE reattaches to every one still running, under its worktree, each
in a tab, with its last output (up to 1 MiB) replayed; Claude Code and other full-screen programs redraw. Closing the
window (on macOS: Cmd+Q or the Dock's Quit; the window's × only hides the window there, IVE keeps running in the
Dock and a click on its icon shows the window again), or SIGTERM /
SIGINT, with terminals running asks **Stop N running processes?**: Keep running (Enter, and what happens when its
10 s countdown ends, `config.quit_countdown`), Stop all, or Cancel (Escape); with `--stop-processes-on-quit` it
stops them all without asking. SIGUSR1 quits at once and keeps them
running, as a [restart](#restart) does. A row's Close, closing a project, or `exit`
still ends a terminal. Only the first IVE running in a profile keeps terminals this way; a second one's terminals end with it.

On Linux, each terminal's host moves into a systemd scope of its own, `ptyhost-<id>.scope`, before its shell starts. Under
memory pressure systemd-oomd then ends the one terminal that filled memory (a build, a Claude Code session), not
IVE with every terminal. Without a user systemd the host logs why and the terminal stays in IVE's cgroup.

| Outside IVE | How |
|---|---|
| List running terminals | `ive --pty-host ls`: id, shell pid, folder, command |
| Use one in this terminal | `ive --pty-host attach <id>` while IVE is closed (a running IVE takes it straight back); Ctrl+] detaches and leaves it running |
| End one | `ive --pty-host kill <id>`: hangs it up, and kills it 2 s later if it is still there |
| Keep them elsewhere | `IVE_PTY_DIR=<dir>`; the default is `$XDG_RUNTIME_DIR/ive-pty/<profile>`, emptied at logout; on macOS `~/.ive-pty/<profile>`, since macOS deletes what sits in `$TMPDIR` for 3 days and socket paths must stay under 104 bytes |

The `--pty-host` commands use the `default` profile; for another one set `IVE_PTY_DIR` to that profile's folder.

### Hub (Ctrl+Shift+E)

Ctrl+Shift+E opens the hub as a temporary tab (italic `Hub`): opening anything from it replaces it. With an empty query it
lists **Recent** (every tab shown, newest first, open or closed: a closed one is greyed and reopens), **Changed**
(the uncommitted files of every worktree) and **Running** (live processes and runs; exited runs are in Processes only). Typing searches every file of
every open project, and their scratches (`scratches/<name>`), as fzf does: fuzzy, smart case, `'exact`, `^prefix`,
`suffix$`, `!not`, space for AND and `a | b` for OR, the matched characters highlighted; the counter at the right of
the query reads `matched/total · ms`. `file.rs:42` opens the file at line 42, marked. The preview on the right
shows the selected row: a file in the editor's view (coloured, read-only), a change's diff, a process's terminal as it
draws it (its last lines), a run configuration's command or its run's output, a worktree's summary.

A row of one worktree carries its colour and name and opens there; a file or run configuration in several says
`N worktrees`, and Enter asks which, most recently used first (a worktree is used when one of its tabs is shown,
Enter is pressed in one of its terminals or a run starts in it), so Enter, Enter opens in the last one used.

| Key | Does |
|---|---|
| ↑ ↓, Ctrl+J/K, Ctrl+N/P, PgUp PgDn | move the selection |
| Enter | open the row, or ask its worktree; in the question, open there |
| Shift+Enter | open behind: the hub stays, the tab opens right of it (to stay, even an overview) |
| Alt+1 … Alt+9 | open the numbered row |
| Alt+Enter | paste the row's path (`path:line` for a contents line) into the most recently used terminal, relative to its folder when it works in that worktree |
| Ctrl+Shift+Q (Cmd+Alt+W on macOS) | stop the process of a process row, or of a Recent row's open process tab, as Stop on its tab; the hub stays and the row reads `stopping…`; other rows do nothing |
| Ctrl+Y | copy the row's project-relative path |
| Ctrl+Shift+C (Cmd+Alt+C on macOS) | copy the row's absolute path |
| Tab | scope: all projects → the shown tab's project → its worktree → a scope picked with `:`; Backspace on an empty query goes back to all |
| Alt+I | search excluded (`config.excluded`) and gitignored files too; off at each Ctrl+Shift+E; while off, the key strip says how many are left out |
| Ctrl+Shift+E again | only the changed files; the query filters them |
| Ctrl+Shift+F (Cmd+Shift+F on macOS) | between searching file names and file contents |
| Alt+R, Alt+W | a contents search's query as a regex; whole words only |
| Alt+P | what a contents search covers: files and processes' output, files only, output only |
| Escape | from the worktree question back to the list; else clear the query; else back to the tab shown before |

**Contents.** Ctrl+Shift+F in the hub, or Find in Files from anywhere, searches the files' lines instead of their
names: plain text from 2 characters, smart case (any capital makes it exact), Alt+R for a regex and Alt+W for whole
words. Each row is `path:line` and the line with its matches marked; Enter opens the file at that line with the match
selected, and the preview shows the file around it, scrolled sideways to the match. It searches the hub's scope (the
worktrees of All, a project or a worktree, and their scratches), or the files and folders Find in Files was picked on,
named above the rows (`Contents of src +1`); Backspace on an empty query drops them for the scope. Gitignored and
excluded files are left out unless Alt+I, but a folder picked in the tree is searched even when it is ignored itself.
Files over 8 MB and binary files are skipped.

It also searches what the processes of the scope printed (running ones, and runs that ended, scrollback included),
listed under **Output** as `process:line`; a process in no worktree only in All. The preview draws the terminal
around the line, the match marked; Enter shows the process's tab scrolled to the line with the match selected, or at
its bottom when the line has left the scrollback. Alt+P cycles what is searched: files and output, files only, output
only; a search of files picked in the tree searches files only. The list stops at 2,000 lines, output first. The
counter reads `matches in files and processes · ms`. The lines stay as found: files changing on disk or processes
printing do not search again, a change to the query does.

Prefixes: `>` actions (New terminal, New scratch, Git log, project overview, Close Other Tabs, Settings, theme), `@`
processes and run configurations (Enter shows a configuration's live run, or starts one), `#` worktrees and the
branches without one (Enter: the Worktrees list with the branch typed and selected), `:` the scopes (projects, their worktrees
nested), `:42` line 42 of the file shown before the hub.

New terminal anywhere: Ctrl+Shift+T (`act::OpenTerminalIn`) opens the hub as a **New terminal** tab that only asks
where. Before typing, a section per open project lists its main checkout, its other worktrees (most recently used
first) and its 5 newest branches without a worktree (`local`, or only on a remote such as `origin`, with a faded
icon); typing ranks them all as the hub does and shows the 20 best. Enter on a checkout opens a terminal there; on a
branch, New worktree adds its worktree (in the folder `.ive.toml` names, default `.worktree`) and the terminal opens
in it, its worktree expanded in the tree. Ctrl+Shift+E switches to the full hub; Escape clears, then goes back. Over MCP:
`open_hub` with `{"new_terminal": true}`.

Closing a tab opened from the hub with Ctrl+Shift+W or Ctrl+Shift+Q within 10 s, before another tab is shown, brings
the hub back as it was: query, scope, worktree question, selected row. A click on ×, a middle click or the tab menu
close it plainly. The rule is `close_back_to_hub` and `stop_back_to_hub` in `ive.rn`, bound to those keys; see
[Scripting](#scripting) to change it.

A kept hub (double click on its title) stays open and is restored at the next start with an empty query. The hub's
history of shown tabs (500) and each worktree's last use are kept in `state.toml`.

### Terminals follow their directory

A terminal or Claude Code tab moves to the checkout it now works in, in any open project: `cd` into another
worktree, or Claude Code entering one with `EnterWorktree`, moves its row there within a second. If the tab is the
one shown, its row is also selected, as Select Opened File does. Outside every checkout (`cd /tmp`) it stays where it
was. A terminal started in the home folder joins the first checkout it enters. When a program exits, the shell's
directory counts again, so the tab goes back to where the shell is. Runs stay under their folder.

IVE reads the directory from the foreground program's `/proc/<pid>/cwd` (Linux) or from what it reports with
`OSC 7` (`ESC ] 7 ; file://<host>/<path> BEL`, hosted terminals). The last `OSC 7` wins until another program takes
the foreground. Shells can report it from the prompt. fish does already; for bash and zsh:

```sh
# ~/.bashrc
PROMPT_COMMAND='printf "\e]7;file://%s%s\a" "$HOSTNAME" "$PWD"'${PROMPT_COMMAND:+";$PROMPT_COMMAND"}
# ~/.zshrc
chpwd() { printf '\e]7;file://%s%s\a' "$HOST" "$PWD" }
```

Claude Code needs nothing: its process directory follows `EnterWorktree` and `ExitWorktree` (checked with 2.1.303).
As a fallback, this repository also reports it: the root `.claude/settings.json` runs `.claude/hooks/osc7.sh` after
`EnterWorktree` and `ExitWorktree` in every Claude Code session started in the repository. The hook writes `OSC 7`
to Claude's terminal itself, because hooks run without a terminal and Claude Code drops `OSC 7` from a hook's
`terminalSequence`. It needs only bash, and sends nothing when something is missing.

### Drag and drop, copy and paste

Ctrl+click (macOS Cmd+click) picks tree rows one by one, Shift+click from the last one picked; files and folders pick
together, terminals, Claude Code and runs together. Escape drops the picks. A hint under the pointer says what a drop
would do.

- **Terminals and Claude Code** dropped on a worktree, its Processes folder or a project (its main checkout) move
  there: the shell is not touched, only listed under the other worktree. It stays there until its directory changes
  again (into another worktree it follows as above; anywhere else it stays), and after a restart of IVE too. Runs
  stay with their worktree's `.run.toml`. With the copy modifier held (Ctrl, macOS Option), the same things start
  there instead and the originals keep running: a terminal, Claude Code, the worktree's own copy of a run.
- **Files and folders** dropped on a folder (or a file: its folder), a worktree, its Files folder or a project copy or
  move there, in any project: the same worktree moves, another copies; Ctrl copies and Shift moves (macOS Option and
  Cmd). From different folders of one worktree, a menu asks whether to keep their folders (below the folder they
  share) or put them all flat into the target; from several worktrees they always go flat. A copied folder carries
  all its files but git's own (`.git`, nested worktrees); a moved one carries a nested `.git` too, and a folder holding
  another open worktree is not moved. Pasted or Ctrl+dropped into its own folder, a file or folder goes beside
  itself as `name (2)`. At most 20 000 files and 4 GiB in one copy.
- **Copy and Paste**: Ctrl+C (macOS Cmd+C) in the tree puts the picked files' absolute paths on the clipboard; Ctrl+V
  on a row copies the files and folders on the clipboard into its folder, keeping their folders when they come from
  one worktree. The tree menu has Copy and Paste too.
- **Nothing is overwritten unasked.** When anything would land on an existing file or folder, nothing is written and
  a Conflicts tab lists them: where each comes from, both sizes, why (differs by so many lines, same content, two
  incoming files of one name, a folder in the way) and Replace | Skip | Keep both (`name (2).ext`); a click on a row
  shows both texts. Copy stays greyed until every row is decided; Cancel or closing the tab writes nothing. Each file is
  written under a temporary name and renamed into place. A line under the tree says what happened (`Copied 7 files to
  dev · internal-apps/ive/mock · skipped 1`); a failure stays until clicked.

The drop modifiers are `config.drop_modifiers = #{ force_copy: "CTRL", force_move: "SHIFT" }` in `ive.rn` (macOS
`ALT` / `SUPER`), the keys `act::CopyFiles`, `act::PasteFiles` and `act::ClearRowSelection` in the tree scope; change
them in `init.rn`.

### Selecting text, failures and notices

**Selecting text.** Text you may want to paste elsewhere can be selected like text on a web page: press and drag,
double click a word (a path or a sha counts as one), triple click the whole block, Shift+click to extend. Copy and
Select All are your editor keys (Ctrl+C and Ctrl+A, Cmd on macOS; whatever `init.rn` binds in the `editor` scope);
the other editing keys do nothing there. One selection at a time, inside one block; pressing anywhere else clears
it. Selectable: the Settings pages' texts (the Scripts instructions, the build line, versions, licence texts, where
settings are saved, errors), commit tab headers (sha, author and date, message, parents), diff tab headers (path,
source), the Git log tab's worktrees and branches, a project overview's location (the full path), worktree list
hints, the scratches folder, a scratch tab's header, the run values and run configurations tabs' texts, failed tabs
and notices. Buttons, tabs, tree rows, menus and tooltips are not: a click there does what it did.

**Failed tabs.** A tab that failed shows what failed, then details (`Command`, `Folder`, `Exit`, `Cause`, `Attach`,
`Configuration`, `Worktree`…) and the last lines the program printed, all selectable. **Copy error** copies it as
plain text with IVE's version and the OS, ready to paste into a search or hand to an agent; MCP's `list_tabs` gives
the same text as a failed tab's `error`. A shell that fails right after start, or dies of a signal IVE did not send
(Stop, closing the tab and quitting send theirs), keeps its tab with the command, the folder, the exit code or the
signal by name (`SIGSEGV (11)`) and its last 40 output lines; any other exit closes the tab, as before. A lost
terminal says why (the connection error) and how to reach it (`ive --pty-host attach <id>`). A run that cannot
start names its command as `.run.toml` writes it (never a secret), the folder, the configuration and its file. A
file deleted while open says which (`deleted on disk: <path>`); `.run.toml`, `vars.toml` and `settings.toml` errors
name the file and line; git errors keep libgit2's class and code; a run's exit by signal reads
`✕ exit 137 (SIGKILL)`.

**Notices.** A failure away from any tab (a terminal that cannot be re-attached at start, edits not saved, a git
action or Run from the tree menu that failed, a hub scratch that cannot open, IVE's state not saved) shows at the
bottom right of the main window, with its time, Copy and ×; at most five, the oldest goes first, and they never
take the keyboard. Every notice, and the failures no window can show (the terminal registry at start, file
watching, the MCP server stopping, handing a path to the OS, saving a pasted image), also go to
`~/.local/state/ive/<profile>/ive.log` (next to `scripts.log`; 1 MB, then its newer half) and stderr. MCP's
`notices` returns the open ones, newest first, and the log's path.

### Restart

To work on IVE in IVE, start it with `./restart.sh` (or a copy with your flags) from a terminal. The Restart button
left of Settings types `SCRIPT --replace PID` into a terminal tab of its own, PID being this IVE's, while IVE keeps
running. The script builds IVE; a failed build stays in that tab with its output. After a good one it sends IVE
SIGUSR1, on which IVE saves its tabs and quits at once, leaving the terminals running, and starts the new IVE with
`--restart` again; it comes back with the same tabs and terminals, the restart tab among them. A press while a
program still runs in that tab only shows it. The script decides how to build and which flags IVE gets: copy
`restart.sh` and add flags such as `--profile` or `--mcp` to its `IVE_FLAGS`.

On macOS the launcher `install.sh` writes starts the new IVE through `IVE.app`, so the Dock shows it
as IVE; `restart.sh` starts it directly.

### Desktop icon

On Linux the task bar shows IVE's icon (`desktop/icons/`) once it is installed for your user:
`desktop/install.sh` installs the icon, `desktop/install.sh COMMAND` also adds an "IVE" desktop entry that runs
COMMAND, e.g. `desktop/install.sh "$PWD/restart.sh"`. IVE's window has the app id `ive`, which is how the desktop
matches it to the icon. KDE Plasma keeps showing an icon it already loaded until `plasmashell` restarts.

### Profiles

A profile is a separate IVE: `ive --profile NAME` (letters, digits, `-` and `_`, up to 64) keeps everything apart
from other profiles. Without `--profile` IVE runs as `default`.

| What | Where |
|---|---|
| projects, tabs, Recent, window | `~/.config/ive/<profile>/state.toml` |
| settings | `~/.config/ive/<profile>/settings.toml` |
| scratches | `~/.config/ive/<profile>/scratches/` |
| terminals | `$XDG_RUNTIME_DIR/ive-pty/<profile>/` (macOS: `~/.ive-pty/<profile>/`) |

The tree, open files and git status follow changes made outside IVE (file notifications on what is shown: open
folders, open files' folders, each repository's git dir; a project overview's Worktrees list is also re-read every 5 s).
Where the OS refuses a watch (Linux: inotify's max_user_watches), a warning in the sidebar header names the paths IVE
polls instead.

Git is read with libgit2 and never written, except by New worktree. A worktree's git folders in the tree follow
changes on disk and re-read every 2 s while one is shown or its row is hovered; the Git log tab follows the repository's git dir; a commit tab shows its
commit as read once; diff tabs stay as they opened until Reload; an uncommitted diff tab edits its file and follows it on disk.

| In a terminal | How |
|---|---|
| Copy, paste | Ctrl+Shift+C, Ctrl+Shift+V (Cmd+C, Cmd+V on macOS); Shift+Insert pastes; a screenshot pastes as the path of a saved PNG, which Claude Code attaches; Ctrl+Alt+Shift+V (Cmd+Alt+V) pastes a clipboard's text and its image |
| Select | drag; double-click selects a word, triple-click a line; Shift+drag in programs that use the mouse |
| Scroll back | mouse wheel (10 000 lines); in full-screen programs like `less` the wheel sends arrow keys |
| Mouse in programs | programs that track the mouse (mc, Claude Code, htop, vim `mouse=a`) get clicks, drags and the wheel |

Not supported yet: IME input in terminals, split panes.

## Run configurations

A `.run.toml` in a folder declares what can run there: commands, and static servers. It is part of the repository,
so the team shares it; the values it needs (and secrets) stay with you. Any folder of a checkout may have one, but
the checkout's root: put repository-wide ones in a folder of their own (`run/.run.toml`).

```toml
# internal-apps/misc/.run.toml
[vars.zoho_secret]
description = "Zoho OAuth client secret"
secret = true                                   # kept in the OS wallet; usable in env only

[vars.ngrok_subdomain]
description = "ngrok subdomain"
default = "misc-${USER}"

[run.build]
name = "Build misc"
command = "mvn -q package -DskipTests"

[run.app]
name = "MiscApplication"
cwd = ".."                                      # relative to this folder, or absolute ("${REPO}/x", "/srv")
command = "mvn spring-boot:run -Dzoho.secret=$ZOHO_SECRET"   # $NAME is the shell's, ${name} IVE's
before = ["build"]                              # run first, in order; each must exit 0
env = { ZOHO_SECRET = "${zoho_secret}" }

[serve.docs]
name = "Docs preview"
root = "docs/site"
port = 4200                                     # or "${docs_port}"
live_reload = true                              # the default: CSS swaps in place, anything else reloads
fallback = "index.html"                         # for paths without a file (single-page apps)
rewrites = [{ from = "^/blog/([^/]+)$", to = "/blog/$1.html" }]
script = "serve.rn"                             # optional Rune hook, below
open = "/blog/"                                 # the page its link goes to, "/" by default
```

- Keys and variable names: 1-64 of `a-z`, `0-9`, `_`, `-`. `name` defaults to the key. Built-in variables: `USER`,
  `HOME`, `REPO` (the checkout's root), `DIR` (the folder of `.run.toml`); `$${` writes a literal `${`.
- A command runs as `$SHELL -lc '<command>'` in `cwd` (default: the folder), with `env` added.
- A broken file (bad TOML, an unknown field, `before` naming a missing key or a cycle, a secret outside `env`, an
  unknown `${name}`, a port outside 1024-65535) shows a warning on its row, with the line and column; its folder is
  then a plain folder.

**Values.** A run needs the variables its configuration and its `before` steps use. They come from
`~/.config/ive/<profile>/vars.toml` (a `[repo."<main checkout>"]` table first, then the top level), else a variable's
default. A secret's entry is a reference into the OS wallet (Secret Service on Linux, Keychain on macOS), service
`ive`, account `<profile>/<id>`:

```toml
ngrok_subdomain = "kb-misc"
zoho_secret = { wallet = "zoho_secret" }

[repo."/home/me/atlassian"]
ngrok_subdomain = "kb-atlassian"
```

Run with a value missing opens a **Run `<name>`** tab with a field per missing variable (masked for secrets);
its Run saves plain values to `vars.toml` (comments kept) and secrets to the wallet, then runs.

**In the tree.** A folder with a `.run.toml` has a run badge on its icon. Every run of a checkout, live or exited,
is a row in its Processes folder, after the terminals: `● running` (a server adds `· :<port>`),
`… waiting for <step>`, `exit 0`, `✕ exit 1`, `■ stopped`, `✕` and why a chain failed. Running: Restart and Stop on
hover, a static server also Open in the browser and Copy the link; exited: Start and Remove. Clicking a row shows the run's output, in a tab of its own.

**Run configurations tab.** A click on a worktree's Processes row (its chevron folds it) opens a temporary tab listing
every configuration of the worktree's `.run.toml` files, as the hub's index read them: a filter (words, all
matching name, folder or command), Flat (by name, the folder after it) or Tree (folders nested as in Files, a
folder with a `.run.toml` opens it on hover), and a row per configuration with its run's state. ▷ Run, Restart and
Start there start the run without leaving the list; a click on a row with a run shows its output. The layout and
folds last while the tab is open.

**Runs** are terminals running one program, and outlive IVE like terminals (Linux, macOS). One row per configuration:
starting one again removes its exited run. Stop sends SIGTERM to the program's process group, SIGKILL 5 s later.
Restart ends the run and starts it again, `before` steps first; a server waits up to 2 s for its old port, so it
keeps its link. A run keeps its output after it exits, until removed; quit and Close project count only live runs.
Removing a worktree ends its runs (its terminals move to the main checkout).

**Static server.** A `[serve.*]` configuration runs `ive --serve` (its setup in `IVE_SERVE`) on `127.0.0.1:<port>`,
or on a free port the OS picks when `<port>` is taken (the row and the link show the one used):
the files under `root` (never outside it), `index.html` for a folder, `Cache-Control: no-store`, a log line per
request. While it runs, a bar over its output (its tab) shows its link, to `open`: a click opens
the browser, Copy copies it; its row in Processes has both on hover. Live reload injects `<script src="/__ive/live.js">` into HTML and watches the files the server has sent, so a root wide enough for shared styles does not reload on edits to anything else under it; the log names the files behind each reload. The hook gets each
request first:

```rune
pub fn request(req) {                           // req: #{ method, path, query, headers }
    if req.path == "/health" {
        #{ status: 200, headers: #{ "Content-Type": "text/plain" }, body: "ok" }   // answer
    } else if req.path.starts_with("/v2/") {
        #{ path: req.path.replace("/v2/", "/") }                                    // rewrite, go on
    } else {
        ()                                                                          // go on
    }
}
```

It may use Rune's `fs`, `json` and `time`, and runs 50 million instructions per request at most; its errors are
logged and answered 500. End an `if` with `else { () }`: Rune reads `{ … } ()` as a call.

## Languages

Bash/Zsh, C, C++, C#, CMake, CSS, Dart, Diff, Dockerfile, DTD, EJS, Elixir, Elm, ERB, Erlang, F#, fish, Gleam, GLSL,
Go, go.mod, GraphQL, Groovy/Gradle, Haskell, HCL/Terraform, HEEx, HTML, INI, Java, JavaScript, JSON, Julia, KDL,
Kotlin, Lua, Makefile, Markdown, Nix, Objective-C, OCaml, PHP, PowerShell, Properties, Protocol Buffers, Python, R,
Ruby, Rune, Rust, Scala, SCSS, Solidity, SQL, Starlark/Bazel, Svelte, Swift, TOML, TypeScript/TSX, Vue, XML, YAML, Zig.

A file's language comes from its exact name (`Makefile`, `Dockerfile`, `.bashrc`), then its extension, then its
`#!` line (`#!/usr/bin/env python3`). Embedded code is coloured in its own language, up to three levels deep: CSS
and JavaScript in HTML, fenced code blocks in Markdown, JSDoc and regular expressions in JavaScript, HEEx in
Elixir's `~H`, HTML around PHP, ERB and EJS, script and style blocks in Vue and Svelte.

Adding a language: a row in `src/syntax/registry.rs` (matching, grammar, queries), a key
table in `src/colors/keys/` (listed in its `TABLES`), `assets/queries/<language>/highlights.scm` (capture names are key ids) and optionally
`injections.scm`, a sample file in `assets/previews/` (listed in `SAMPLES` of `src/syntax/tests/scaling.rs`), tests in `src/syntax/tests/`. Grammars without a crate that
works with IVE's tree-sitter are vendored in `vendor/grammars/` and compiled by `build.rs`. After adding
IntelliJ-keyed languages, regenerate the schemes: `tools/port_intellij_schemes.py /path/to/intellij-community`.

## Scripting

IVE's key and mouse bindings are Rune scripts ([Rune](https://rune-rs.github.io)), declared in
[WezTerm](https://wezterm.org/config/keys.html)'s shape. Two files take part, both in `~/.config/ive/<profile>/`:

- `ive.rn`: IVE's defaults, every one of them. IVE writes it there at each start, read-only, for you to read
  (Settings › Scripts opens it); it always runs its own built-in copy and never reads the file back, so editing or
  deleting it changes nothing, and the next start writes it afresh.
- `init.rn`: yours. IVE runs it after `ive.rn`, at start and again whenever a `.rn` file in that folder is saved; no
  restart needed. Other `.rn` files there are modules `init.rn` can `mod` in.

```rune
pub fn init(config) {
    config.keys = [
        // Claude Code reads ESC CR as "new line", Enter alone as "send"
        #{ key: "Enter", mods: "SHIFT", action: act::SendString("\x1b\r"), scope: "terminal" },
        #{ key: "r", mods: "CTRL|ALT", action: restart },
    ];
    config.mouse_bindings = [
        // middle click no longer closes a tab; ctrl + left click does
        #{ event: #{ Up: #{ streak: 1, button: "Middle" } }, action: act::DisableDefaultAssignment, scope: "tab" },
        #{ event: #{ Up: #{ streak: 1, button: "Left" } }, mods: "CTRL", action: act::CloseTab, scope: "tab" },
    ];
}

async fn restart(context) {
    // the Restart button's click target; IVE must run with --restart
    ive::click("restart").await?;
    Ok(())
}
```

- `config.keys` entries: `key`, `mods`, `action`, `scope`. `key` is one character (`"a"`, `","`), `"F1"`-`"F24"`,
  or `"Enter"`, `"Tab"`, `"Escape"`, `"Backspace"`, `"Delete"`, `"Insert"`, `"Home"`, `"End"`, `"PageUp"`,
  `"PageDown"`, `"UpArrow"`, `"DownArrow"`, `"LeftArrow"`, `"RightArrow"`, `"Space"`. `mods` is `"NONE"` (the
  default) or `|`-joined `CTRL`, `SHIFT`, `ALT` (`OPT`, `META`), `SUPER` (`CMD`, `WIN`) and IVE's `SECONDARY`: Cmd
  on macOS, Ctrl elsewhere. One key per binding; no chords.
- `config.mouse_bindings` entries: `event`, `mods`, `action`, `scope` (required), `mouse_reporting`. `event` is
  `#{ Down: ... }`, `#{ Up: ... }` or `#{ Drag: ... }` of `#{ streak, button }`: `button` `"Left"`, `"Middle"` or
  `"Right"`, `streak` 1 to 3 (single, double, triple click; a double click's first press is a `streak: 1`). A
  click with modifiers that no binding names runs the `mods: "NONE"` binding, so ctrl + click on a tab still
  shows it.
- `scope`, where a binding applies: `"window"` (keys anywhere; the default), `"terminal"` (keys while a terminal has
  the keyboard, the mouse over a terminal), `"editor"` (keys in a scratch, and Copy and Select All in an open file or a diff), `"tab"` (the mouse on a tab), `"tree"`
  (the mouse on a sidebar row, outside its chevron and buttons), `"hub"` (keys while the hub's query has the
  keyboard: the hub's own keys, before the query field's; the field keeps typing, ←/→, Home/End, Backspace on a
  non-empty query and the clipboard). `"tree"` is also a key scope: keys while the sidebar has the keyboard (a
  click on it gives it). In a terminal, a scratch, the hub or the tree, its own binding of a key comes before a
  window one.
- Overriding: an `init.rn` entry with the same scope and the same key and mods (or event and mods) as an `ive.rn`
  entry replaces it; `act::DisableDefaultAssignment` switches it off (a terminal then gets the key as if it were not
  bound). The same trigger twice in one file is an error.
- Text fields keep the keys that type text and their own editing keys (arrows, Backspace, Enter, the clipboard);
  others, such as Alt+F12, go on to the window's bindings. Typing a name never triggers a binding.
- While a program in a terminal tracks the mouse (vim, htop, Claude Code), clicks go to it, and only bindings with
  `mouse_reporting: true` apply; hold Shift to give the mouse to IVE, which then matches the binding without Shift.
- `config.temporary_tabs` (filled in with `ive.rn`'s, so `config.temporary_tabs.enabled = false;` alone works; a
  new object takes `ive.rn`'s values for the keys it leaves out): `enabled` (`true`), `kinds`, the tabs a click
  opens temporary, any of `"Project"`, `"Settings"`, `"Hub"` and `"RunConfigs"` (all four),
  and `marker`, `"Italic"` or `#{ Prefix: "*" }` (1 to 8 characters before the title). A temporary tab closes once
  another tab is shown; a new tab takes its place in the row.
- `config.quit_countdown` (`10`): the seconds the quit prompt counts down before IVE quits and leaves the running
  processes running, 1 to 60.
- `config.open_folders` (`["processes", "uncommitted", "since-base", "commits"]`): a worktree's folders in the project tree
  that start open, any of these and `"files"`; the others start folded. A folder you fold or unfold stays as you
  left it; `config.open_folders.push("files");` starts them all open.
- `config.tab_menus` and `config.tree_menus` (filled in with `ive.rn`'s, per menu name): menus that
  `act::ShowTabMenu(name)` opens on a tab and `act::ShowTreeMenu(name)` on a tree row (a key in the `"tree"` scope:
  on the selection); at most 32 menus of each, 32 items in a menu or submenu, submenus 4 deep. `ive.rn` declares
  `main` of both on the right press. An item is:
  - an action: the tab actions (`act::KeepTab`, `act::StopProgram`, `act::KillProgram`, Stop with SIGKILL at once,
    `act::CloseTab`, `act::CloseOtherTabs`, `act::CloseTabsToTheLeft`, `act::CloseTabsToTheRight`; tab menus only, except
    Stop and Kill, which in a tree menu end the process of a process row, and Stop a running run's), `act::NewTerminal`
    (tree menus: a terminal in the worktree of a Processes folder),
    the path actions in the table below, or `act::OpenHub`, `act::OpenSettings`, `act::OpenTerminal`,
    `act::OpenTerminalIn`, `act::OpenFindInFiles`, `act::ToggleChangesLayout`;
  - `"Separator"`;
  - `#{ label, action, when }`: `action` an action (relabelled) or a function, which gets the menu's `context`
    (below); `when` optional;
  - `#{ label, items, when }`: a submenu.

  A line shows on its right the key bound to its action in `config.keys` (a tree menu: the `"tree"` scope's first,
  then the window's; a tab menu: the window's), as `Ctrl+Shift+C`; a function item shows the key bound to the same
  function. Lines without a key show none.

  A menu leaves out what does not apply (Keep Tab on a permanent tab, Stop and Kill on a tab that is not a process, a
  path action on a terminal tab, Run on a file that does not run, Git Show Diff on an unchanged file), then the
  separators left dangling and the submenus left empty, and does not open when nothing is left. `when` is an object,
  a function, or both (`#{ kind: "folder", test: f }`, the function asked only where the rest holds):
  `kind` (`"file"` or `"folder"`), `name` (a glob or a list of globs against the file name), `path` (the same
  against the path relative to its worktree, absolute outside one), `count` (`"one"` or `"many"`) must all hold, the
  first three for every path; a terminal tab has no paths, so they never hold there. A `test` function (async or
  not) gets the `context` and holds when it answers `true` or `Ok(true)`: the menu opens at once with its item faded
  (`…`), and the item turns usable or goes when it answers; one that takes over 5 s, or errs, counts as `false` and
  the log says so. The same label twice with `kind` file and folder gives each its own action:

  ```rune
  config.tree_menus.main.push(#{ label: "Open in IntelliJ", action: idea_file, when: #{ kind: "file" } });
  config.tree_menus.main.push(#{ label: "Open in IntelliJ", action: idea_folder, when: #{ kind: "folder" } });
  config.tree_menus.main.push(#{ label: "Cargo", items: [#{ label: "Test", action: cargo_test }], when: #{ name: "Cargo.toml" } });
  ```

  A different tab menu with Shift:

  ```rune
  config.tab_menus.close = [act::CloseOtherTabs, act::CloseTabsToTheLeft, act::CloseTabsToTheRight];
  config.mouse_bindings = [
      #{ event: #{ Down: #{ streak: 1, button: "Right" } }, mods: "SHIFT", action: act::ShowTabMenu("close"), scope: "tab" },
  ];
  ```
- `ive::os()` is `"linux"` or `"macos"`, for keys that differ between them (as `ive.rn`'s terminal copy and paste).
- `ive::profile()` is the profile's name (`"default"` without `--profile`); `ive::config_dir()` is
  `Some(folder of init.rn)`, e.g. `Some("/home/me/.config/ive/default")`, or `None` when the OS reports no config
  folder.

| Action | Scope | Does |
|---|---|---|
| `act::SendString(text)` | window, terminal | types `text` into the shown terminal at once, in order with the keys around it |
| `act::CopyTo(target)` | terminal, editor | copies the selection to `"Clipboard"`, `"PrimarySelection"` or `"ClipboardAndPrimarySelection"` |
| `act::PasteFrom(source)` | terminal, editor | pastes from `"Clipboard"` or `"PrimarySelection"` (Linux; empty on macOS); in a terminal, a clipboard with an image and no text (a screenshot) pastes the path of the image saved to `~/.cache/ive/<profile>/pastes/` (kept 7 days), which Claude Code attaches |
| `act::PasteTextFrom(source)` | terminal | pastes only the text |
| `act::PasteImageFrom(source)` | terminal | pastes only the image, as its saved file's path |
| `act::PasteTextAndImageFrom(source)` | terminal | pastes the text, a space and the image's path (an office app's cells come as both) |
| `act::SelectTextAtMouseCursor(mode)` | terminal mouse | starts a selection by `"Cell"`, `"Word"` or `"Line"` |
| `act::ExtendSelectionToMouseCursor(mode)` | terminal mouse | extends it while dragging |
| `act::CompleteSelection(target)` | terminal mouse | ends it and copies it to `target`, as `CopyTo` |
| `act::OpenSettings`, `act::OpenTerminal` | window | the Settings tab; a new terminal in the current tab's checkout |
| `act::ToggleChangesLayout` | window | changed files flat or grouped by folder, in the tree's git folders and the Git sections (Alt+Shift+G by default) |
| `act::ToggleFolderEverywhere(folder)` | window | shows worktree folder `"processes"`, `"uncommitted"`, `"since-base"`, `"commits"` or `"files"` in every worktree, or hides it (Ctrl+Alt+1 to 5 by default, in this order; the folders menu shows the keys bound) |
| `act::ActivateTab`, `act::CloseTab` | tab (`CloseTab` also window) | shows or closes the clicked tab; a `CloseTab` key closes the shown one |
| `act::KeepTab` | tab, window | makes the clicked temporary tab stay (double click by default); a key keeps the shown one |
| `act::StopProgram`, `act::KillProgram` | tab, window | SIGTERM or SIGKILL to the program in the clicked (a key: the shown) process tab's foreground, the shell staying; a run is stopped as its Stop does (SIGKILL 5 s later), or killed at once |
| `act::CloseOtherTabs`, `act::CloseTabsToTheLeft`, `act::CloseTabsToTheRight` | tab, window | closes the row's other tabs, or those left or right of the clicked (a key: the shown) tab; detached tabs stay |
| `act::ShowTabMenu(name)` | tab | opens menu `name` of `config.tab_menus` on the clicked tab, at the pointer (`"main"` on a right press by default) |
| `act::ShowTreeMenu(name)` | tree | opens menu `name` of `config.tree_menus` on the clicked row (a key: on the selection), at the pointer (`"main"` on a right press by default) |
| `act::SendPathsToTerminal` | tree, window, menus | the list of terminals for the paths (the tree's selection; a window key: the shown tab's file) |
| `act::CopyRelativePath`, `act::CopyFileName` | tree, window, menus | paths relative to their worktree, or file names, one per line, to the clipboard |
| `act::FindInFiles` | tree, window, menus | the hub searching the contents of the paths |
| `act::OpenFindInFiles` | window, menus | the hub searching contents in its usual scope (Ctrl+Shift+F) |
| `act::OpenTerminalHere`, `act::RevealInFileManager`, `act::OpenInNewWindow` | tree, window, menus | a terminal in the folder (a file: its folder); the OS file manager with it selected; the file in a window of its own |
| `act::OpenWithDefaultApp`, `act::OpenInBrowser`, `act::RunFile`, `act::GitShowDiff`, `act::GitCompareWithBase`, `act::GitHistory`, `act::GitIgnore`, `act::CompareFiles` | menus | as the tree menu's items of those names |
| `act::HubToggleContent`, `act::HubToggleRegex`, `act::HubToggleWholeWord` | hub | file names or contents; a contents query as a regex; whole words |
| `act::HubCycleSearchIn` | hub | what a contents search covers: files and output, files only, output only (Alt+P) |
| `act::ActivateRow`, `act::ToggleRow` | tree | the row's click (a project's overview, open the file, toggle a worktree or folder); expand or collapse it |
| `act::ActivateRowToKeep` | tree | the row's click, its project tab kept rather than temporary (middle click by default); on other rows the same as `ActivateRow` |
| `act::ToggleRowSelection`, `act::ExtendRowSelection` | tree | Ctrl+click (Cmd on macOS) adds a file or folder to the selection or takes it out; Shift+click selects the files and folders from the last clicked row; dragging a selected row onto a terminal types all their paths |
| `act::SelectAll`, `act::Cut`, `act::Undo`, `act::Redo` | editor | the scratch editor's commands |
| `act::Find`, `act::FindNext`, `act::FindPrevious` | editor | a code view's find box: open it (or give it the keyboard back), the next and previous match (Ctrl+F, F3, Shift+F3; macOS Cmd+F, also Cmd+G, Cmd+Shift+G) |
| `act::ToggleDiffLayout` | editor | a focused diff inline or side by side (no key by default); new diff and commit tabs open in the layout last switched to |
| `act::OpenHub` | window | shows the hub (Ctrl+Shift+E) |
| `act::OpenTerminalIn` | window | the hub asking where a new terminal goes: a checkout, or a branch that first gets its worktree (Ctrl+Shift+T) |
| `act::HubDown`, `act::HubUp`, `act::HubPageDown`, `act::HubPageUp` | hub | move the selection |
| `act::HubOpen`, `act::HubOpenBehind`, `act::HubOpenRow(n)` | hub | open the selected row (asking its worktree), behind the hub, or numbered row `n` (1 to 9) |
| `act::HubTypePath`, `act::HubCopyPath` | hub | the row's path into the last terminal; to the clipboard |
| `act::CopyFullPath` | hub, tree | the absolute path of the hub's row, or of the tree's selected files and folders, one per line, to the clipboard |
| `act::HubStop` | hub | stops the selected process row's process, or a Recent row's process tab's, as `act::StopProgram` (the Stop key by default); the hub stays |
| `act::HubCycleScope`, `act::HubWiden` | hub | the next scope; all projects (on an empty query only) |
| `act::HubToggleExcluded`, `act::HubToggleChanged` | hub | search excluded and ignored files too; only the changed files |
| `act::HubBack` | hub | the worktree question → the list → clear the query → back to the tab shown before |
| `act::Nop` | any | takes the trigger and does nothing |
| `act::DisableDefaultAssignment` | any | removes the default for the trigger |
| a function | any | runs on IVE's script thread with `context` (below) |

- A function gets a `context`: for a key, the shown tab (`#{ tab, kind, checkout }`); for a click on a tab, that
  tab; on a tree row, `#{ row, checkout }` (`row` is its `list_targets` name); a menu's item or condition,
  `#{ menu, paths, items, checkout, row or tab }` (`paths` absolute; `items` `#{ path, relative, kind }`, `relative` to
  the worktree or `()`; `checkout` the one worktree of all paths or `()`); in a terminal, its tab plus the
  `column` and `line` clicked (from 0, as `read_screen`). It runs on IVE's script thread, so a slow one never holds
  up the window. Its `Err` goes to the log.
- Functions call IVE through the `ive` module, with the MCP tools' names and arguments (all `async`, returning
  `Ok(answer)` or `Err(reason)`): `list_tabs`, `list_projects`, `active_tab`, `open_file(path)`,
  `open_terminal(#{ command, directory })`, `activate_tab(tab)`, `close_tab(tab)`, `detach_tab(tab)`,
  `attach_tab(tab)`, `send_text(tab, text)`, `send_keys(tab, keys)`, `read_screen(tab)`, `read_output(tab, lines)` (`lines` an integer or `()` for all), `open_project(path)`,
  `show_process(process)`, `list_targets`, `click(name)`, `window_keys(keys)`, `window_text(text)`, `stop_tab(tab)` (as Ctrl+Shift+Q),
  `open_hub(hub)` (a tab's `opened_from_hub.hub`, or `#{}` for a fresh hub), and `send_to_active(text)` (the shown terminal).
- `active_tab` and `list_tabs` give a tab just opened from the hub `opened_from_hub: #{ seconds_ago, hub }`, until
  another tab is shown. `ive.rn` binds Ctrl+Shift+W and Ctrl+Shift+Q to `close_back_to_hub` and `stop_back_to_hub`,
  which close (stop) the tab and call `open_hub` when it came from the hub at most 10 s before. `init.rn` changes
  the time by binding its own copy to the keys, or drops the rule with `act::CloseTab` / `act::StopProgram`:

  ```rune
  pub fn init(config) {
      config.keys = [#{ key: "w", mods: "CTRL|SHIFT", action: close_back_to_hub }];
  }

  async fn close_back_to_hub(tab) {
      ive::close_tab(tab.tab).await?;
      if let Some(from) = tab.get("opened_from_hub") {
          if from.seconds_ago <= 30.0 {
              ive::open_hub(from.hub).await?;
          }
      }
      Ok(())
  }
  ```
- `config.font` sets the font of code, diffs and terminals (default `"JetBrains Mono"`), `config.ui_font` that of
  everything else (default `"Inter"`), as IntelliJ's New UI; IVE bundles both. Any installed family works; one that
  is not installed keeps the font IVE had, and the log says so.
- The OS is reachable through Rune's modules `fs`, `process`, `json`, `time`, `io`, and IVE's `http` and `toml`.
  Scripts run with your rights.
  - `http::get(url)` and `http::request(#{ method, url, headers, body, timeout })` (`method` defaults to `"GET"`,
    `timeout` to 30 seconds, at most 600; `body` is a string) answer `Ok(#{ status, headers, body })`, with header
    names in lowercase; a 404 is an `Ok` too, a failed connection an `Err`. They run the system's `curl`, so TLS,
    proxies (`https_proxy`) and certificates are the OS's; `~/.curlrc` is not read. Only `http://` and `https://`.
  - `toml::from_string(text)` reads TOML into a Rune object, `toml::to_string(object)` writes one back.

  ```rune
  let deploy = http::request(#{ method: "POST", url: "https://example.test/deploy", body: json::to_string(#{ env: "dev" })?, headers: #{ "Content-Type": "application/json" } }).await?;
  if deploy.status != 200 {
      println!("deploy: {} {}", deploy.status, deploy.body);
  }
  ```
- Everything scripts report goes to the log file `~/.local/state/ive/<profile>/scripts.log` (`$XDG_STATE_HOME/ive/<profile>`; on macOS
  next to `init.rn`), one timestamped line each: load results, errors with file, line and column, `println!` output
  and functions' errors. Past 1 MB it keeps its newer half. When a load or a function fails, IVE opens the log in an
  ordinary file tab and shows its end; agents read the same through the `script_status` tool.
- `init` must finish within 50 million Rune instructions; a load that fails keeps the previous bindings (at start,
  `ive.rn`'s alone).
- A function may run as long as it awaits now and then; one that runs 50 million instructions without awaiting is
  stopped, so a loop by mistake cannot block later keys or reloads.

### Excluded folders

`config.excluded` lists folder names whose content counts as excluded, besides what git ignores: the project tree
draws them as excluded. A name matches a folder of that name anywhere in a project. `init` gets IVE's defaults,
`target` and `node_modules`, so `init.rn` can add to them or take one out:

```rune
pub fn init(config) {
    config.excluded.push("dist");
    config.excluded = config.excluded.iter().filter(|name| name != "node_modules").collect::<Vec>();
}
```

Names are folder names, not paths: no `/`, at most 64 of them.

### Theme

Every colour and size IVE draws with is in `config.theme`. `init` gets it already filled in with IVE's defaults,
so `init.rn` sets only what it changes, and can read the defaults to compute from them. IVE writes the whole default
theme next to `ive.rn` as `theme.toml`, read-only, for you to read; like `ive.rn`, editing it changes nothing.

**User themes.** A theme is a file `themes/<name>.toml` next to `init.rn`, in `theme.toml`'s shape but holding only
what it changes; it has both looks, `[dark]` and `[light]`. Settings › Appearance › Theme picks one (Default is
IVE's own; the choice is kept in `settings.toml` as `user_theme`), and Mode picks the look. The theme is merged
over the default, tables key by key, anything else (arrays included) replaced whole, and `config.theme` starts from
the result, so `init.rn` changes it further. Saving, adding or removing a theme reloads it. A theme is checked like
`init.rn`'s theme, with the file in the message (`themes/nord.toml: theme.dark.tre: unknown field …`), larger than
256 KB is refused, and a failed load keeps the previous one; at start, a broken theme leaves the default theme with
`init.rn`'s bindings. A picked theme whose file is gone draws the default and stays picked, muted, until its file
is back. Settings › Scripts says where every customisation goes, with Copy instructions for an agent's prompt.

```toml
# themes/dim.toml
[sizes]
rem = 15.0

[dark]
tree = "#202124"

[light.git]
modified = "#1F5FBF"
```

```rune
pub fn init(config) {
    config.theme.dark.tree = "#202124";
    // the whole UI an eighth bigger: every size written in rem follows
    config.theme.sizes.rem = 18.0;
    config.theme.sizes.tree.row_height = "1.75rem";
    config.theme.sizes.tree.text_size = 14.0;
    // the code font a fifth bigger; it is in px, so arithmetic works
    config.theme.sizes.editor.text_size = config.theme.sizes.editor.text_size * 1.2;
    config.theme.dark.syntax["CSS.COLOR"] = #{ foreground: "FF0000", bold: true };
}
```

- Colours are per theme, in `config.theme.dark` and `config.theme.light`: `"#RRGGBB"`, or `"#RRGGBBAA"` where a
  colour is see-through (`diff_added`, `scrollbar_thumb`, `find_match` and `find_current` for the find box's matches, row
  backgrounds).
- Sizes are shared by both themes, in `config.theme.sizes`, grouped by where they are drawn: `tree`, `tabs`, `window`,
  `editor`, `diff`, `terminal`, `overview`, `git`, `settings`, `controls`, `menu`, and `icon` for every icon. A size is
  pixels, written with a decimal point (`26.0`: Rune does not mix whole and decimal numbers) or as `"26px"`, or rem,
  as `"1.625rem"`: that many times `config.theme.sizes.rem` (16 px by default), rounded to whole pixels. Most
  defaults are in rem, so changing `sizes.rem` zooms the UI, text without a size of its own included; lines and
  corners (borders, radii, insets, underlines, carets, scrollbars, handles) and the code font (`editor.text_size`,
  `terminal`, the diff's code rows) are in px and stay as they are. A default in rem is a string, so compute from
  it in px instead (`config.theme.sizes.tree.row_height = 26.0;`). On macOS the window buttons stay where they
  are, centred only at the default tab row height. Font weights are `"normal"`, `"medium"`, `"semibold"` or
  `"bold"`.
- Syntax and diff colours are `config.theme.dark.syntax` and `config.theme.light.syntax`, by colour key, as the
  Settings colour pages name them: `foreground`, `background` (`"RRGGBB"`, without `#`), `bold`, `italic`, or
  `inherit: true` for the key's parent's look.
- How thick text looks is `config.theme.dark.text_rendering` (and `light.text_rendering`), on Linux: `gamma` from
  1.0 to 2.2 and `contrast` (extra weight for text darker than 75% brightness) from 0.0 to 4.0; higher values draw
  text bolder. IVE draws with 1.0 and 0.0, the glyphs' coverage as rasterised, which is closer to IntelliJ than Zed's
  1.8 and 1.0 (`config.theme.dark.text_rendering.gamma = 1.4;`). A change applies to open windows at once.
- The project tree marks folders as excluded when git ignores them or `config.excluded` names them (see "Excluded
  folders"), and colours files by git status: `config.theme.dark.git` has `modified`,
  `added`, `deleted`, `unversioned`, `renamed`, `conflicted`. Rows in a state are drawn by
  `config.theme.dark.tree_states`: `hover`, `excluded`, `ignored`, `active` (the shown tab's file while another row
  is selected; plain by default), `selected`, `secondary` (grey extra text), each with an optional `text`,
  `background`, `weight` and `italic`. A file's git colour stays on selected and excluded rows. Folders get no git
  colour.
- Rows that are not on disk show their `@name` on a chip, styled like a row state by
  `config.theme.dark.virtual_folders`: `hidden` for `@hidden` (two or more
  hidden entries of a folder; weaker by default, as it holds real files); `chip` is unused since `@run` went (kept so
  themes that set it still load). `config.theme.sizes.tree.chip_padding_x`
  and `chip_radius` shape it; a chip without `background` is just the styled name.
- The sidebar and the tab area (the tab row and the shown tab) show which has the keyboard: a press in the sidebar
  gives it the keys, a press in a tab, a document or a terminal gives them to the tab area, and a file opened from
  the tree leaves them in the sidebar. The one without the keys, and both while the window is not active, draw with
  `config.theme.dark.unfocused` (and `light.unfocused`): any keys of the palette, over their usual values. By
  default, as IntelliJ, the selection (`selected`, `tree_states.selected`) and the shown tab (`tab_active`,
  `tab_active_border`) turn grey; `config.theme.dark.unfocused.selected = config.theme.dark.selected;` keeps a
  colour. Syntax colours do not change with focus.
- The tab and tree menus are drawn as IntelliJ's popup menus, by `config.theme.sizes.menu` (`min_width`,
  `padding_y`, `radius`, `border`, `text_size`, `weight`, `item_height`, `item_margin_x` (the highlight's distance
  from the sides), `item_padding_x`, `item_radius`, `shortcut_gap` (at least this between a label and its key),
  `separator_margin_y`, `separator_inset_x`, `pending_opacity`) and `config.theme.dark.menu` (and `light.menu`):
  `background`, `border`, `text`, `shortcut` (a line's key), `disabled`, `highlight` and `highlight_text` (the line
  under the pointer or the arrow keys), `separator`.
- `config.theme.sizes.tree.selection_radius` and `selection_inset` round a highlighted row and keep it off the
  panel's sides (IntelliJ: `8.0` and `12.0`).
- `config.theme.panels.gap` and `radius` (5 and 8 by default; 0 and 0 for the flat layout) draw the tree and the
  editor as rounded panels on the `config.theme.dark.frame` colour, as IntelliJ's Islands theme, with a glow behind
  the header rows at the top left in the colour of the worktree of the selected tree row (a project row: its main
  worktree; nothing selected: the shown tab's worktree). Each worktree has one of the nine `config.theme.dark.glow`
  colours, picked by its path; one whose colour an earlier worktree in the tree has takes the next free one, so up
  to nine open worktrees never share a colour; `glow_peak`, `glow_width`,
  `glow_height` and `glow_blend` shape it. In a maximised window the frame's corner closes the window too.
- A key IVE does not know, a value of the wrong type, a malformed colour, a size in another unit than px or rem, or
  a size out of range (0 to 2000 px, heights from 1 px, fonts 6 to 96 px, `sizes.rem` 8 to 32 px) fails the load with the key's path and the value, as `init.rn: theme.dark.tree: expected a colour
  "#RRGGBB", not "#12"`; IVE keeps the theme it had (at start, the default).

## Drive IVE from an agent (MCP)

IVE serves MCP on `127.0.0.1` unless started with `--no-mcp`, and prints how to register it with Claude Code:

```
ive: MCP server at http://127.0.0.1:7461/mcp
ive: connect with: claude mcp add --transport http ive http://127.0.0.1:7461/mcp --header "Authorization: Bearer 3f9c…"
```

Each profile keeps its port and token in `~/.config/ive/<profile>/mcp.toml` (readable by you only), so a
registration survives restarts; agents find a running IVE's URL and token in the same file.

IVE's terminals get `IVE_MCP_URL` and `IVE_MCP_KEY` (the token) in their environment, so one registration serves
every profile: a Claude Code started in an IVE terminal connects to the IVE it runs in. In this repository the
root `.mcp.json` registers it (Claude Code asks once to approve the `ive` server); elsewhere, register it for your
user once:

```sh
claude mcp add-json --scope user ive '{"type":"http","url":"${IVE_MCP_URL}","headers":{"Authorization":"Bearer ${IVE_MCP_KEY}"}}'
```

Outside IVE the variables are unset: Claude Code warns about them and `ive` fails to connect. Terminals started
before IVE served MCP, or with `--no-mcp`, go without them.

- The first run takes port 7461, or any free port when 7461 is taken (by another profile, say).
- `--mcp PORT` serves another port and remembers it. When the remembered port is taken, IVE says so and runs
  without MCP rather than move: a new port would break the registration.
- `IVE_MCP_TOKEN` (at least 16 characters) is served instead of the remembered token, and never written down.
- An unusable `mcp.toml` is moved to `mcp.toml.bad` and IVE picks a port and token afresh.

| Tool | What it does |
|---|---|
| `list_tabs` | tab ids, kinds (`document`, `process`, `project-overview`, `diff`, `commit`, `git-log`, `scratch`, `settings`, `failed`, `run-values`, `run-configs`, `conflicts`, `hub`), titles, checkout path, which one is shown, which one is temporary, `opened_from_hub` on a tab just opened from the hub, and a failed tab's `error` (the text its Copy error copies) |
| `active_tab` | the shown tab: id, kind, checkout path; nulls when no tab is shown |
| `script_status` | scripts from `~/.config/ive/<profile>`: whether `init.rn` loaded (`loaded`, `failed`, `no-init`), its errors with file and line, bindings made, the log file and its last 50 lines |
| `notices` | the failures shown at the bottom right, newest first (time, text), and the path of `ive.log`, which keeps them all |
| `list_projects` | open projects with their checkouts (label, branch, changes, ahead, behind; `null` until read) and processes (id, tab id, kind `terminal`, `claude-code` or `run`, title; a run also its `folder`, `config` and `live`) |
| `list_run_configs` | a folder's `.run.toml` read now: each configuration's key, name, kind (`run`, `serve`), command (secrets as `${name}`), live state and process id; or the file's error |
| `run_config` | runs a configuration (`folder`, `key`) as Run in the tree does; answers `{ process }`, or `{ missing: [names] }` without asking for them |
| `list_targets` | what `click` can press now, by name: every tree row and its actions (also rows scrolled out of view), overview sections and cards, Settings, prompt buttons (see the names below) |
| `click` | click a target from `list_targets`, as the mouse would; a tree, Worktrees or Scratches row out of view is scrolled to first; answers once the window has drawn the result |
| `drop_rows` | drag tree rows onto another row and drop them, as the mouse would: `rows` and `onto` by their names after `tree-menu:` (`entry:/r/src/a.rs`, `process:3`, `toggle:/r/.claude/worktrees/x`); the first row is the pressed one, the others are picked with it; `force`: `copy` or `move`, as if `config.drop_modifiers`' key were held. Returns the drop's hint (`Copy 2 items to dev · lib`), or why it was refused. Files from folders of one worktree open the layout menu: answer it with `click` on `tree-menu:keep-folders`, `tree-menu:flat` or `tree-menu:cancel` |
| `window_keys`, `window_text` | keys or text for whatever has the keyboard: a prompt (Enter, Escape), a terminal, a text field, the workspace's key bindings (`alt-f12`) |
| `show_process` | show a process's tab by its `list_projects` id, opening it again after its tab was closed |
| `open_project` | open a folder's project, as Open Folder… with it picked; answers with its id once it is open |
| `open_file` | open a file (absolute, or relative to the current checkout, as Alt+F12 picks it) or show its tab; returns the tab id |
| `open_terminal` | new terminal in the checkout holding `directory` (absolute or relative; default: as Alt+F12), optionally running `command`; returns the tab id |
| `activate_tab`, `close_tab` | show or close a tab by id |
| `stop_tab` | as Ctrl+Shift+Q: a process tab's process ends for good and the tab closes; another tab just closes |
| `open_hub` | show the hub, optionally in the state (`hub`) of a tab's `opened_from_hub`; answers `{ tab }` |
| `detach_tab`, `attach_tab` | open a tab in a window of its own, or put it back into the tab row |
| `send_text` | type text into a process, file, scratch or uncommitted diff tab (at the diff's caret); end with `\r` to press Enter (a line break in a file, scratch or diff) |
| `send_keys` | press keys in a process tab, written like key bindings: `["ctrl-c"]`, `["up", "enter"]` |
| `tab_mouse` | use the mouse on a tab of the tab row (`click`, `double_click`, `press`, `release`; button and Shift/Alt/Ctrl) through the tab mouse bindings, as a real click: a right click opens the tab's menu, a Shift+right click the one `init.rn` binds to it |
| `send_mouse` | use the mouse in a process tab on a screen cell: `click`, `double_click`, `triple_click`, `press`, `release`, `move`, `scroll`; says whether it went to the program or to IVE |
| `read_screen` | a process tab's visible text, one string per row, and the cursor position |
| `read_output` | what a process tab printed, scrollback included, one string per line (wrapped rows joined); `lines` keeps only the last ones |
| `screenshot` | the whole IVE window as a PNG |

Tips for agents:

- Prefer `read_screen` to `screenshot` for terminal output: it is exact and cheap. For a command's result, `read_output`
  also has what scrolled out of view.
- Send a key that interrupts a program (`ctrl-c`) in its own call, then check `read_screen` before typing more;
  keys sent while the program is still exiting reach the terminal raw and are echoed as `^[[A`.
- Input reaches IVE directly, not through the OS, so it works while the window is unfocused or behind others.
- `send_mouse` cells are the ones `read_screen` returns (column and line from 0). When the program tracks the mouse
  (mc, Claude Code, vim), clicks, drags and the wheel go to it; otherwise they select text and scroll history. Hold
  `shift` to select even then, as with a real mouse.

Click targets (`<id>` a project id, `<label>` a checkout label, `<n>` a process id from `list_projects`):

| Where | Targets |
|---|---|
| tab row | `restart` (with `--restart`: builds, then restarts IVE) |
| tree picks | `tree-pick:<row target>` as a Ctrl+click on the row (adds it to the picked rows, or takes it out), `tree-pick-range:<row target>` as a Shift+click (picks the rows from the last one picked); files and folders pick together, terminals, Claude Code and runs together |
| menus | `tab-menu:<tab>` and `tree-menu:<row target>` (e.g. `tree-menu:entry:/r/src`) open a tab's or a row's `main` menu; then each line is `tab-menu:` or `tree-menu:` and the labels down to it joined by `/` (`tree-menu:Copy Path/Copy Relative Path`; a click on a submenu's line opens it); Send to Terminal's lines by process id (`tree-menu:Send to Terminal/<n>`, `…/new`), Run's by configuration key; a line waiting for its condition is listed as `<name> (pending)`; the tab actions also as `tab-menu:keep`, `tab-menu:stop`, `tab-menu:kill`, `tab-menu:close`, `tab-menu:close-others`, `tab-menu:close-left`, `tab-menu:close-right` |
| sidebar header, welcome | `select-opened` (Select opened file: the shown tab's row), `open-project` (the + menu), `recent:<id>`, `open-folder` (the OS picker: use the `open_project` tool instead) |
| sidebar header menus | `project-picker` (the title), then `show-projects:all`, `show-projects:<id>`; `folders-menu`, then `every-folder:<folder>`, `changes-layout:flat`, `changes-layout:tree`, `reset-folders` |
| project row | `project:<id>` (overview), `toggle:<id>` (chevron), `close-project:<id>` |
| checkout row | `toggle:<id>/<label>` (`<label>@<path>` while two checkouts share a label, e.g. a worktree folder named like the main checkout's branch), `hide-worktree:<id>/<label>` (not on the main checkout) |
| worktree folders | `folder:<id>/<label>/<folder>` (fold/unfold; `<folder>` also `files`; Processes' opens its Run configurations tab), `toggle:<id>/<label>/processes` (Processes' chevron), `new-terminal:<id>/<label>/processes` (Processes' +) |
| process row | `process:<n>`, `stop-process:<n>`; a run in Processes: `live-run:<n>`; a running server's `open-link:<n>`, `copy-link:<n>`; its `restart-process:<n>`, `stop-process:<n>` (live) or `start-process:<n>`, `remove-process:<n>` (exited; `<n>` is `pending-<k>` while it waits for `before` steps) |
| git folder rows | `change:<id>/<label>/<source>:<path>` (its diff; `<source>`: `uncommitted`, `since-base` or a short sha), `change-folder:<id>/<label>/<source>:<path>/`, `commit:<id>/<label>/<short sha>` (folds its files), `commits-more:<id>/<label>` (Load 20 more) |
| Conflicts tab (`<n>` a row of the plan) | `conflicts-all:replace`, `conflicts-all:skip`, `conflicts-all:both` (every conflict where offered), `conflict:<n>:replace`, `conflict:<n>:skip`, `conflict:<n>:both`, `conflict-row:<n>` (unfolds both texts), `conflicts-clean` (the files without a conflict), `conflicts-apply` (Copy or Move, once every conflict is decided), `conflicts-cancel`; the line under the tree: `tree-note` (dismisses it) |
| Run configurations tab (`<folder>` checkout-relative) | `run-configs-filter` (focus the filter, then `window_text`), `run-configs-layout:flat`, `run-configs-layout:tree`, `run-configs-folder:<folder>` (folds a Tree folder), `open-run-file:<folder>` (its `.run.toml`), `run-config:<folder>#<key>` (shows its run), `run:<folder>#<key>` (▷ Run), and a run's `run-configs-restart:<n>`, `run-configs-stop:<n>`, `run-configs-start:<n>`, `run-configs-remove:<n>` |
| Run `<name>` tab | `value:<variable>` (focus its field, then `window_text`), `run-values:<tab>`, `cancel-values:<tab>` |
| file row | `entry:<absolute path>` (opens a file, toggles a folder); a file a worktree inside the main checkout shows twice is `entry:<path>@<label>` in the main checkout's tree |
| paging row | `more:<list>`, `all:<list>` (`<list>`: the project id for checkouts, the checkout path for processes) |
| tab row, Settings | `settings`, `theme:<label>`, `settings-page:<page>`, `open-ive-rn`, `open-theme-toml`, `open-settings-toml`, `user-theme:<name>` (`user-theme:Default` for IVE's own, `user-theme:Default.toml` for a theme file named `Default`), `copy-script-instructions`, `open-ive-readme`, `copy-error` (a failed tab), `notice-copy:<n>`, `notice-close:<n>` (notices, newest first from 0) |
| project overview | `section:<label>` |
| project Git section, Git log tab | `open-git`, `log:<sha>`, `git-log:Log`, `git-log:Stashes`, `git-log:Worktrees`, `git-log:Branches`, `load-more` |
| diff and commit tabs, the hub's diff preview | `reload-diff`, `diff-file:<path>`, `diff-layout:inline`, `diff-layout:side-by-side` |
| file, scratch, diff and commit tabs (shown) | `find:open` (the find box, as Ctrl+F; its query has the keyboard, so `send_text` or `window_text` types it). While it is open: `find:query` (gives it the keyboard back), `find:previous`, `find:next`, `find:match-case`, `find:whole-word`, `find:regex`, `find:close` |
| file tab | `change-marker:<line>`: a change marker in the gutter, by its first line (from 1); it opens the change's popup. While the popup is open: `change-popup:prev`, `change-popup:next`, `change-popup:rollback`, `change-popup:diff`, `change-popup:copy`, `change-popup:message` (gives the commit message field the keyboard; then `window_text` types in it and `window_keys` `["enter"]` commits) and `change-popup:commit` |
| Worktrees list | `worktree-filter` (focuses the filter: then `window_text`, `window_keys` `["down", "space"]`), `worktree-row:<path>`, `branch-row:<name>` (as Enter), `worktree-toggle:<path or name>` (Hide, Show, + Worktree), `worktree-hidden-group`, `branch-more`, `branch-all` |
| Scratches section | `new-scratch` (the name input opens focused and takes `window_text`, Enter / Escape), `scratch-search` (focus the search field), `scratch-name` (focus the open name input, New scratch's or Rename's) |
| scratch row | `scratch:<name>` (open it), `rename-scratch:<name>`, `delete-scratch:<name>` (its hover actions; Delete asks: `prompt:Delete`) |
| scratch groups | `scratch-group:<group>` (expand or collapse Today / This week / This month / Older), `scratch-more:<group>` (Show 10 more), `scratch-all:<group>` (Show all) |
| scratch tab (shown) | `run-scratch`, and its "changed on disk" / "deleted on disk" bar: `scratch-reload`, `scratch-overwrite`, `scratch-save-again`, `scratch-close` |
| file or scratch tab (shown) whose save found the file changed on disk | `save-reload` (drop the edits, show the file), `save-overwrite` (write the edits over it; not for files over 8 MiB); neither while a save is being written. While a close or quit waits for edits that could not be saved (the save failed, or a conflict is unanswered), also `save-discard` (Close without saving: the edits are lost, the tab closes, quitting or closing the project goes on) |
| prompt | `prompt:<button label>`; the quit prompt's are `prompt:Keep running`, `prompt:Stop all`, `prompt:Cancel` |
| window | `window:close` (the ×; with terminals running it shows the quit prompt) |

`<sha>` is the short sha (7 characters), `<path>` a path in the repository. Every action in the window is reachable
this way; window buttons are left out on purpose. gpui 0.2.2 cannot inject a mouse click, so clicks go by name, not by pixel.

The server listens on 127.0.0.1 only, needs the bearer token on every request, and refuses requests that carry a
browser `Origin`, so web pages cannot reach it. Anyone holding the token can run commands in your shell.

### Screenshots

`screenshot` has gpui draw the main window's last frame into an image (no OS screen capture), so:

- the window may be covered by other windows or minimized; it shows the last frame drawn before that;
- it shows IVE's main window only, its title bar included: not detached tabs, tooltips or menus.

## Releases

A push to the `release` branch builds IVE for Linux (x86_64) and macOS (Apple Silicon) and publishes both as one
[GitHub Release](https://github.com/Warsaw-Dynamics/ive/releases) (`.github/workflows/release.yml`); a failed build
publishes nothing.

- `ive-linux-x86_64-<commit>.tar.gz`: the `ive` binary, the licence files and the icons. It needs glibc 2.35 or newer
  (Ubuntu 22.04, Debian 12, Fedora 36 and later).
- `ive-macos-arm64-<commit>.zip`: `IVE.app` (written by `desktop/macos-bundle.sh`) and the licence files. The app is
  signed ad hoc, not by an Apple developer, nor notarized: at its first start macOS refuses it; System Settings →
  Privacy & Security → Open Anyway lets it run.

```sh
git push origin HEAD:release
```

The release also has `SHA256SUMS`, and each build knows its tag (`IVE_RELEASE_TAG` at build time).

### Updates

A release build updates itself, the way a browser does. A minute after it starts, and every 6 hours after that, it
looks at the [latest release](https://api.github.com/repos/Warsaw-Dynamics/ive/releases/latest). When that release
has a different tag, IVE downloads the release's asset for its OS and CPU, checks it against `SHA256SUMS`, and unpacks
it into `.ive-update/` beside the install (the `ive` binary, or `IVE.app`). Nothing changes while you work:

- **On quit**, the staged release replaces the install, so the next start runs it.
- **After 30 minutes without a key, click or MCP call**, IVE restarts onto it: tabs come back, terminals and
  their programs keep running. It waits while a file has unsaved edits, while another IVE runs from the same
  install, and while running processes would end (another IVE of the profile keeps its terminals).
- **Restart to update** in the tab row restarts at once, after saving edits.

An update whose terminals cannot carry over (the terminal host's protocol changed) is applied on quit only.
Settings → Updates shows the version and the state, and turns updates off (`auto_update = "off"` in
`settings.toml`, per profile); off, IVE does not look and drops what it staged. A release removed from GitHub rolls
builds back to the one that is latest again. Problems (no network, an install folder IVE cannot write) are printed
to stderr and shown on that page; IVE tries again at the next check. Builds from source never update themselves.

To try it without GitHub: `IVE_UPDATE_FEED` names another URL for the latest release's description (`file://`
works), `IVE_UPDATE_IDLE_SECONDS` another idle time (10 to 86400), and building with `IVE_RELEASE_TAG=<tag>` makes a
build from source act as that release.

## Troubleshooting

**Black band around the window (Linux X11).** IVE draws its own frame: a 10 px transparent band around the
window holds the shadow and the resize handles. Without a compositor, X11 cannot show transparency and the band
turns black. Full-screen games often switch compositing off while they run (KDE: they set
`_NET_WM_BYPASS_COMPOSITOR`). Press Shift+Alt+F12 to turn KDE compositing back on, or untick *System Settings →
Display & Monitor → Compositor → Allow applications to block compositing*.

**Slow start or high memory.** `IVE_LOG=ive=debug,warn ive …` prints to stderr how long each startup step and each
background read (project scan, git status, hub index) took, and when the first frame was drawn, with IVE's resident
memory then. The value is a [tracing filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html):
`info` adds gpui's own log, `ive=trace` every folder the tree lists. Without `IVE_LOG` nothing is logged.

## Licences

IVE is source-available under its own licence (`LICENSE`): you may read, build and change it, and use it free for
personal and noncommercial purposes; commercial use, including an employee's use for work, needs a paid licence per
user; nobody may redistribute it or changed versions of it. What it includes from others is listed in `NOTICE` (icons, fonts, colour schemes,
grammars, gpui, libgit2) and `THIRD-PARTY-NOTICES.md` (every Rust crate with its licence text); ship both with IVE.
Inside IVE, Settings → Third-Party Software lists the same, each with its licence text, as IntelliJ's About dialog does.
After changing a dependency, run `tools/third-party-notices.sh`: `cargo deny check` (`deny.toml`) fails on a licence
or source not reviewed yet, then the notices are rewritten from `Cargo.lock`; commit them with the change.
The script needs `cargo install --locked cargo-deny` and `cargo install --locked cargo-about --features cli`.

## Layout

| Path | Content |
|---|---|
| `src/workspace.rs`, `src/workspace/` | main window: projects, processes, tabs, saved state, actions |
| `src/sidebar/` | the tree: rows (pure), drawing, width |
| `src/editor.rs` | tab kinds, tab row, detached tabs' windows |
| `src/file_tab.rs`, `src/file_write.rs` | a file tab: edited in `codeview`'s panel, its edits written in place or through a temporary file |
| `src/line_view.rs` | scrolling and metrics shared by the document view and the editor |
| `src/overview/` | the project overview tab, its Worktrees list, its Git section, the Git log, commit and diff tabs |
| `src/git/` | reading repositories with git2 (libgit2): checkouts, status, changes, diffs, log, branches; New worktree, the one write |
| `src/diff_view.rs` | draws diffs (diff and commit tabs) |
| `src/workspace/git_views.rs` | the git views the workspace owns, when they read git, what their events open |
| `src/project.rs`, `src/project_list.rs`, `src/open_project.rs`, `src/process.rs`, `src/state.rs` | projects, checkouts, processes, `state.toml` |
| `src/scratch/` | scratches: store, groups, search, run, the scratch tab |
| `src/hub/`, `src/workspace/hub.rs`, `src/workspace/hub_index.rs` | the hub (Ctrl+Shift+E): fzf v1 matcher, per-checkout file index merged into one corpus, search, history of shown tabs, the state and rows (pure), the tab's drawing and preview; the workspace's side: opening rows, keeping the index current |
| `src/text_editor/` | the plain-text editor of scratch tabs |
| `src/text_input.rs` | single-line text field (IME through gpui's input handler); editing rules in `text_input/model.rs` |
| `src/syntax/` | language registry (`registry.rs`), parsing into embedded-code layers (`parse.rs`, `injection.rs`) and highlight spans (`highlight.rs`); queries in `assets/queries/` |
| `vendor/grammars/` | generated tree-sitter parsers without a usable crate, compiled by `build.rs` |
| `src/colors/` | colour keys, built-in schemes (`assets/schemes/`, from `tools/port_intellij_schemes.py`), user overrides; checkout colours |
| `src/settings_page/` | Settings tab: Appearance, Scripts, Updates and Third-Party Software |
| `src/terminal/` | terminal: PTY session (`alacritty_terminal`), foreground program (Claude Code), keyboard and mouse encoding, colours, drawing |
| `src/api/` | what MCP and scripts may ask of the workspace; `src/workspace/api_requests.rs` answers it on the UI thread |
| `src/mcp/` | MCP server, a caller of `src/api/` |
| `src/script/` | Rune scripts: `ive.rn` and `init.rn` loading, the `ive` and `act` modules, key and mouse bindings (plain-data keymap for the UI thread), the script thread, the script log |
| `assets/scripts/ive.rn` | IVE's default key and mouse bindings, compiled in |
| `src/run/` | run configurations: `.run.toml` (`file.rs`), values, `vars.toml` and the wallet (`values.rs`, `wallet.rs`), `before` chains, stopping, the Run `<name>` tab, the static server (`serve/`) |
| `src/workspace/runs.rs` | starting and showing runs |
| `src/mouse_binding.rs` | gpui mouse events as binding triggers, for tabs and tree rows |
| `mock/` | static HTML mock of the UI |
| `.github/workflows/` | the release build |
| `src/update/`, `src/workspace/updates.rs` | updating a release build: the latest release, download and checksum, staging beside the install, the swap, when to restart; the workspace's timers, the button and quit |
| `deny.toml`, `about.toml`, `tools/third-party-*`, `tools/third_party_json.py` | licence and source checks; the generators of `THIRD-PARTY-NOTICES.md` and of `assets/licenses/third-party.json`, which `src/third_party.rs` reads for Settings → Third-Party Software |
| `desktop/` | the app icon in task bar sizes and `install.sh`, which installs it (and a desktop entry) on Linux |
