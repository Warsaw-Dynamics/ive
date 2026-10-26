# Attention: a process that wants the user

## Why

Claude Code, Codex or a build runs in a terminal while the user works elsewhere in IVE. When it finishes, or waits
for an answer or a permission, nothing tells the user: they find out by looking at each terminal. IVE should show
which processes called, until the user looks at them, and reach the user through the desktop when IVE is in the
background.

IVE stays neutral: it reads only standard terminal sequences and knows no agent. Each agent is set up to send them
(README).

The mock (`mock/checkouts.html`, DESIGN.md "Attention") shows the design.

## Goals

### What raises attention

From a hosted terminal's **live** output (not replayed output: a replay holds calls the user already saw):

- **Notification** with a text: `OSC 9;<text>` (iTerm2), except ConEmu's numbered commands (`9;4;…`, `9;9;…`: a first
  parameter of digits only); `OSC 777;notify;<title>;<body>` (Ghostty, VTE, urxvt), the body, else the title;
  `OSC 99;<metadata>;<payload>` (kitty), the payload, unless the metadata says it is base64 (`e=1`): ignored.
- **Bell**: `BEL` outside an escape sequence. Text: `rang the bell`.
- **Finished**: `OSC 9;4` going from a busy state (normal, error, paused, indeterminate) to `0`. Text: `finished`.

Texts are cleaned: control characters dropped, runs of whitespace made one space, trimmed, at most 200 characters
(longer ones cut, ending in `…`). A notification whose text cleans to nothing reads `notified`.

In-process terminals (no pty host) do not read these, as they do not read `OSC 9;4` and `OSC 7`.

### When it counts

- A call counts only while the user is **not looking** at the process: its tab is not the shown tab of an active IVE
  window (the main window, or the detached window that holds the tab).
- A process has at most one attention. A newer call replaces the text and keeps the time of the first call, so the
  list stays ordered by how long a process has waited.
- It clears when the process's tab is shown in an active window: showing the tab, or the window becoming active
  while the tab is shown. It goes with the process.

### Where it shows

One colour, a new palette key `attention` (dark `#F0A732`, light `#B86E00`), not the green of running or the red of
failed.

- **Tab**: a dot at the top right of its icon (6 px, ringed with the tab row's background); the tooltip adds the text.
- **Tree process row**: the same dot on its icon, then `● <text>` after the title.
- **Folded rows**: a folded project, worktree or Processes row says `● n waiting` after its name.
- **Tab row**: left of Settings, IntelliJ's Notifications bell (`notifications-20.svg`, copied unmodified) with the
  dot, only while a process waits. A click opens a list, longest waiting first: icon, `title · worktree`, the text,
  the age (`age.rs`). A click on a line shows that process. Escape or a press outside closes it. MCP: the bell is
  `waiting`, a line `waiting:<process id>`.
- **Hub** (Ctrl+E), empty query: **Waiting** first, above Recent, the same order; the process rows of Running and
  `@` show the text instead of their state.
- **MCP**: each process in `list_projects` has `attention`: `{ "text", "since" }` (RFC 3339), or null.

### IVE in the background

When no IVE window is active, a call that counts also sends a **desktop notification**: title
`<process title> — <worktree label>`, body the text. One per process: a newer call replaces it.

- **Linux**: `notify-send --app-name=IVE --print-id [--replace-id=<id>] --action=default=Show <title> <body>`. A click
  (the `default` action) brings the main window to front and shows the process. IVE withdraws its notifications
  (`org.freedesktop.Notifications.CloseNotification` through `gdbus`) when the attention clears and when an IVE
  window becomes active: the bell lists what still waits.
- **macOS**: `osascript` `display notification`; no click action, not withdrawn.
- Elsewhere, or with the command missing: none. The first failure goes to `ive.log` (`notices`), once a session.

## Out of scope

- Keeping a call made while IVE was closed (the pty host would have to hold it).
- The window's urgency hint (taskbar flash, dock bounce): gpui has no API for it.
- A key that shows the process waiting longest; a Settings switch for desktop notifications; silence after output
  as a signal (tmux `monitor-silence`).

## Agent setup (README)

- Claude Code: `claude config set --global preferredNotifChannel iterm2` (or `ghostty`, `kitty`, `terminal_bell`).
- Codex: `~/.codex/config.toml`: `[tui] notifications = true`.
