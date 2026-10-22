# IVE — UI mock

A static HTML + Alpine mock of the app as it is now: title bar, project tree, editor, Settings tab and
welcome screen. Fake data, no build step. Icons come from `../assets/icons`, so serve the `ive` folder:

```sh
cd internal-apps/ive && python3 -m http.server 8767
```

Open <http://localhost:8767/mock/>. The bar along the bottom is mock-only: welcome screen, a project,
the 10M-line file, a binary file, and a link to `checkouts.html` (which links back with `← index` at its bottom
right; IVE's `Mock` run configuration opens `checkouts.html`). Ctrl+, opens Settings.

`include.js` and `vendor/alpine.min.js` are copied from `forge-installer/mock`.

`checkouts.html` (design notes: [DESIGN.md](DESIGN.md)) mocks the app as it is now (synced with the Rust code on
2026-10-22): projects and worktrees in one sidebar, each worktree with its folders: Processes (terminals, Claude
Code, every run), Uncommitted, Since base, Commits and Files; the badges on the worktree row show or hide them
(DESIGN.md "Worktree folders"). Clicking a project opens its overview as a tab (Worktrees, Git, Scratches); a worktree
row expands. Git is read-only; Open Git opens the Git log tab, a log commit a commit tab, a changed file in the tree
a diff tab; a commit in the tree folds its files, Load 20 more lists more. Scratches live outside the repo and are
kept when a worktree is removed. A click opens a project overview, or Settings, as a temporary tab (italic title):
it goes when another tab is shown, and the next tab opens in its place; double-clicking the tab, middle-clicking the
tree row or the tab menu's Keep Tab keeps it. Other tabs open permanent, right of the shown one. A right press on a tab
opens its menu (the tab is not shown): Keep Tab (temporary tab), Stop and Kill (process tab: SIGTERM, SIGKILL; greyed with
nothing in the foreground), a file's items (Send to Terminal ▸, Find in Files…, Copy Path ▸, Open In ▸, Git ▸), then
Close, Close Other Tabs, Close Tabs to the Left, to the Right. A right press on a tree row opens its menu (`menus.js`,
DESIGN.md "Menus"): Send to Terminal ▸ (terminals most recently used first, then a new one; the path is typed into it,
relative in its own worktree), Open in Terminal, Run (shell files, `.run.toml` folders), Find in Files…, Copy Path ▸,
Open In ▸ and Git ▸ (Show Diff, Compare with Base Branch, Show History, Add to .gitignore); submenus open on hover. There is no title
bar: Settings and the window buttons sit at the right end of the tab row; the gear (or Ctrl+,) opens Settings,
whose Appearance page switches the theme. Alt+F12 opens a terminal in the current tab's checkout.
Ctrl+E opens the hub (a proposal, DESIGN.md "Hub"): recent items, changes in every worktree and running processes;
typing searches the monorepo's real file paths fzf-style (`data/files.gz`; regenerate with `tools/hub-files.sh`; it also samples the ignored files). Alt+I searches excluded and gitignored files too.
Ctrl+Shift+F (in the hub too) searches the files' contents instead: rows `path:line` with the matches marked, Alt+R
regex, Alt+W whole words; Find in Files… in a menu limits it to that file or folder (Backspace on an empty query drops
it). The mock searches the text its files and scratches have.
Ctrl+Shift+T (in a browser also Alt+Shift+T; DESIGN.md "New terminal") asks where a new terminal goes: a project, a
worktree, or a branch without one (local, or only on `origin` with a faded icon), which first gets its worktree in
the folder `.ive.toml` names. A Processes row's + (on hover) starts a terminal there.
The project page's Worktrees section is a list (DESIGN.md "Hidden worktrees"): worktrees by last use,
Hidden ones (Hide on a worktree's tree row or page row; out of the tree, hub and New terminal), branches without a worktree.
Colours come from `src/theme.rs` (`mock.css`), sizes from the Rust file each block in `checkouts.css` names. Data is in `checkouts.js`, partials in `components/checkouts/`. `checkouts-data.js` generates scratches and
branches with Faker (seeded): 600 scratches and 30 branches without a worktree; long lists show 10 rows, "Show 10
more" and "Show all". Faker's English locale is vendored in `vendor/faker/` (MIT). Icons are the app's, from
`../assets/icons`; fonts (Inter, JetBrains Mono) from `../assets/fonts`.

Run configurations: folders with a
`.run.toml` (`run`, `internal-apps/ive`, `internal-apps/misc`) have a run badge on their icon, nothing else (no `@run`, no `● n`). Every run of the worktree is a row in its
Processes, after the terminals: its state (`● running · :8767`, `… waiting for build`, `exit 1`), Restart and Stop while
it is live, Start and Remove once it exited; a click shows the run's output tab and Run unfolds Processes. MiscApplication asks for its missing secret in a `Run MiscApplication` tab, then runs its `before` steps;
the IVE mock config is a static server; `lib/.run.toml` is broken on purpose (a warning on the file, `lib` a plain
folder). A click on a worktree's Processes row opens its `Run configurations` tab (temporary, a proposal): every
configuration in the worktree (the only place a configuration that never ran shows, with ▷ Run), Flat (by name) or Tree (by folder), with a filter and each run's state and actions.
The tree's right edge drags to resize it.

The tree's virtual folders, which are not on disk (`@hidden` for two or more hidden entries), show
as icon + italic `@name` on a chip; `@hidden`'s is weaker: muted text on a half-transparent chip.

Mock-only: the hash keeps the mock's state (`route.js`), so a refresh or a shared link shows the same screen: the
open tabs in order (`t`, by key), the shown and temporary one (`at`, `temp`), the tree's expanded and collapsed nodes
(`tree`), overview sections (`o`), theme, tree width, Settings page and the hub's query. Processes
started later, runs, new worktrees and scratches, and the hub's history are not kept.
An older-style hash opens a screen, for comparing with the app, e.g.
`checkouts.html#theme=dark&tab=project&section=Git` (`tab`: project, file, scratch, terminal, agent,
settings, gitlog, commit, diff, runconfigs (the Processes row's tab over the `runs` screen; `rlayout=tree` groups it), runconfigs1k (the `platform` project's tab: 1000 configurations in 250 folders), run (`internal-apps/misc` expanded), runs (ive's and misc's runs in Processes: a
server, a running command and a finished build); hub (the hub; `q`: its query, `changed=1`: changed only); `section`: an overview section; `layout`: flat, tree;
`page`: a Settings page; `menu`: add, branches, tab (the tab menu on the shown tab); `gap`: pixels between the
tree and the editor, 5 by default (as the app), 0 for the flat layout; `radius`: the panels' corner radius, 8 by default;
`side`: the tree's width in pixels; `counts`: how folder counts look, dot (the app), plain, pill, faint or outline, also
switched at the bottom right).
