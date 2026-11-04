# Scheduled commands

## Why

An agent working in IVE often has to come back later: CI takes two hours, a review is due Monday, a deploy lands
tonight. Today it has to keep a session waiting, or the user has to remember. IVE should let an agent schedule a
command (usually `claude --resume <session> "<what to check>"`) for a time, in a project and branch. IVE starts
the command then, in a new terminal, in a worktree it creates when needed.

A scheduled command runs after the agent is gone, so it is a way to run code later. **IVE never runs one unasked
the first time**: when a command comes due it goes to the Scheduled commands tab, where the user reads the whole
command and ticks what runs. For a repeating command the user can allow that exact command to run unasked from
then on.

The mock (`mock/checkouts.html#tab=scheduled`, `mock/schedule.js`, DESIGN.md "Scheduled commands") shows the
design.

## MCP

### `schedule_command`

| Argument | |
|---|---|
| `name` | required, 1–80 characters, one line; the terminal's first title |
| `description` | optional, up to 2000 characters: why, and what to check |
| `command` | required, 1–8000 bytes, typed into the new terminal's shell, followed by Enter |
| `at` | RFC 3339 with an offset (`2026-11-04T16:00:00+01:00`) |
| `in` | a delay: `90s`, `30m`, `2h`, `1d`, or a sum such as `1h30m` |
| `repeat` | a 5-field cron expression in local time (`0 7 * * 1-5`) |
| `project` | the project's folder (any folder in the repository); absolute |
| `directory` | instead of `project` and `branch`: a folder in an open project; its checkout's project and branch |
| `branch` | the branch to run on |
| `worktree` | optional: where a new worktree for `branch` goes (absolute, or relative to the main checkout) |
| `source_branch` | where `branch` starts when it does not exist yet |

- Exactly one of `at` and `in`, except with `repeat`, which may have neither: its first time is then the next one
  the cron gives.
- `project` must be a git repository's folder when scheduled. The main checkout's path is what is kept.
- The answer is `{ id, at, at_local, replaced }`. `at` is UTC and `at_local` the local time with its offset, so the
  agent sees what its time meant. `replaced` says whether it took the place of an existing command.
- **Same name, same project**: a pending or due command is replaced and keeps its id. The permission to run unasked
  goes when the command text changes. A retrying agent does not schedule twice.

**Bounds** (refused with the value that broke them):
- `at` within a year, and not more than a day in the past;
- at most 100 pending and due commands;
- a cron with fields out of range, or two of its next 24 times under 15 minutes apart;
- `name` empty or with line breaks;
- the folder missing, or not in a git repository;
- `directory` not in an open project, or in a checkout without a branch.

### `list_scheduled`

Pending and due commands, then the history of the last 7 days. Each entry has:
- `id`, `name`, `description`, `command`;
- `project`, `branch`, `worktree`, `source_branch`;
- `at` and `at_local`, `repeat`;
- `state`: `pending`, `due`, `started`, `skipped`, `failed` or `cancelled`;
- `runs_unasked`, `missed`;
- `process` (the id `list_projects` uses, once started) and `error` (failed).

### `cancel_scheduled`

`{ id }`: a pending or due command is cancelled (a repeating one with every later time). Answers `"ok"`, or why not.

## Kept

`~/.config/ive/<profile>/schedule.toml`, written whole through a staging file and a rename. On Unix it is mode 0600,
since it holds commands. It holds:
- `[[command]]` entries with every field above;
- `state`, `missed`, `trusted` (the command text allowed to run unasked);
- `created`, `done`, `process`, `error`, `unasked`.

The history is kept 7 days, then dropped on the next write. A file that does not load is moved to
`schedule.toml.bad`, with a notice; IVE starts with an empty schedule.

## Coming due

IVE checks every 30 seconds, and when it starts, against the wall clock (a sleep would drift across suspend and
clock changes). For each pending command whose time came:

- **Allowed for this exact command and runnable**: it starts unasked (see Starting), then a repeating one goes back
  to pending at its next time.
- **Otherwise** it becomes **due**. Due commands go to the Scheduled commands tab:
  - While IVE runs, the tab opens behind the shown one (or stays where it is). Its icon carries the attention dot,
    and the tooltip says `N commands are due`.
  - At start, the tab is shown, and each row says `IVE was closed then`.
  - With no IVE window active, one desktop notification `N scheduled commands are due`, body the names, then
    `nothing runs until you pick it`. A newer one replaces it. A click raises IVE and shows the tab. It is withdrawn
    when IVE comes to the front, as the attention ones are.
- **A repeating command still due** when its next time passes stays one row: its `missed` count grows, and its time
  moves to the latest one that passed.
- **A one-off command** past its time while IVE was closed is due like any other: missed commands run anyway, once
  the user picks them.

## The Scheduled commands tab

At most one; kind `scheduled`, icon `schedule.svg`, title `Scheduled commands`. It is not saved in `state.toml`: at
start it opens when something is due.

From the top:
- The title: `N scheduled commands are due`, or `Nothing is due`.
- A hint: `Scheduled over MCP. Nothing runs until you tick it and press Run selected · each runs in a new terminal`.
- **All**, a checkbox ticking every runnable row.
- One row per due command, runnable ones first, each part oldest first. A row has:
  - a checkbox, off at first, and off and disabled when the row cannot run;
  - the name; `↻ <repeat>` for a repeating command; `due <age> (<local time>)`, then `· IVE was closed then` and
    `· missed N more times while waiting` where they apply;
  - the description;
  - where it runs: `<project> · ⎇ <branch>`, then `in <path>` (`(main checkout)` for the main one), or
    `Run first: fetch origin, then new branch X from origin/<source>, then new worktree <path>`, or why it cannot
    run, in the error colour;
  - **the whole command**, monospace, wrapped, never cut;
  - `scheduled <age> (<local time>)`;
  - a repeating, runnable command: **Always run this one**. Ticking it ticks the row.
  - A row that cannot run: **Remove**, which puts it in the history as failed.
  - A row being started says `starting…` and cannot be ticked.
- **Run selected (N)**, **Skip selected**, and `Unticked ones stay due`. Skip: this time does not run, and a
  repeating command goes on to its next time.
- **Started**: the commands started from this tab since it opened. A click shows the terminal.
- `N more scheduled · Settings › Scheduled` (a link).

The plan of a row (where it runs, what comes first) is read off the UI thread when the row becomes due and when
the tab is shown, and again when Run starts it.

## Starting

For one command, off the UI thread where it touches git:

1. **The project**: its folder is gone, or no longer a repository: failed, `The project <path> is gone`. Not open
   in IVE: it opens, as `open_project`.
2. **The checkout with the branch** (the main checkout or a worktree): it runs there. IVE never checks out anew.
3. **The branch exists**, local, or on `origin` only (it then gets a local branch tracking it): a new worktree at
   `worktree`, else the `.ive.toml` folder's `<branch, / as ->`.
4. **No such branch**, with a `source_branch`:
   - `git fetch origin <source_branch>` (the git CLI, so the user's credentials work; 2 minutes at most);
   - a new local branch at `origin/<source_branch>` (the local `<source_branch>` when there is no `origin`, or
     the fetch failed and it exists locally);
   - then its worktree, as in 3.
5. **No such branch and no source branch**, or the source is found nowhere: failed, saying which.
6. A hidden worktree is shown again.

Then a **new terminal** in that checkout:
- it opens behind the shown tab;
- it gets `IVE_SCHEDULE_ID`, `IVE_SCHEDULE_NAME` and `IVE_SCHEDULE_DESCRIPTION` in its environment;
- its first title is the name (the program may set another);
- the command is typed into it with Enter.

The command goes to the history as started, with the process.

- **From the tab** the terminal is not shown: the tab stays, and lists it under Started.
- **Unasked**, its process calls for the user as a program would (`attention.rs`), with the text
  `Scheduled: started unasked`. That gives the dot, the bell, and a desktop notification in the background; a
  click shows it.

A failure while starting:
- **From the tab**, the row stays due and says why in the error colour.
- **Unasked**, the command becomes due with that error, and comes to the tab as in Coming due.
- It is written to `ive.log` either way.

## Always run this one

Ticked on a row when Run selected starts it: `trusted` keeps that exact command text. Later, the command starts
unasked while `trusted` equals its command. An agent replacing the command drops it. Settings' **Ask again** drops
it too. One-off commands have no such option.

## Settings › Scheduled

A page after Scripts:
- the intro (what this is, that nothing runs unasked the first time);
- **Due (N)** with **Review**, which shows the tab: name, where, `due <age>`, the command on one line;
- **Coming (N)**, by time: name, `↻ repeat`, `runs unasked` (attention colour) with **Ask again**, where, local time
  and `(in …)`, **Cancel**, the description, the command on one line;
- **Last 7 days**, newest first: state (`started`, `started, unasked`, `skipped`, `failed`, `cancelled`), name and
  where, the error, the age. A click on a started one shows its terminal while it lives.

`Saved to …schedule.toml` replaces the settings line on this page.

## Click targets

- Tab:
  - `scheduled-all`, `scheduled-pick:<id>`, `scheduled-always:<id>`, `scheduled-remove:<id>`;
  - `scheduled-run`, `scheduled-skip`;
  - `scheduled-started:<id>`, `scheduled-settings`.
- Settings: `settings-page:Scheduled`, `scheduled-review`, `scheduled-cancel:<id>`, `scheduled-ask-again:<id>`,
  `scheduled-history:<id>`.

## Not in this

The tree marking a worktree with something coming; a key binding for the tab; editing a command in IVE. Agents
schedule, the user only picks, cancels or takes back.
