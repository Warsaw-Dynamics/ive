# Selectable text, errors that say enough, and no silent failures

## Why

When something fails in IVE, the user wants to take the error elsewhere: paste it into a search engine, or hand it to
a coding agent to debug. Today that does not work, for three reasons:

1. Text outside files, diffs and terminals cannot be selected or copied: Settings, failed tabs, commit headers,
   prompts.
2. Errors often lack what pins the problem down. A shell that dies at start shows `exit status: 1` and throws away
   the output that held the shell's own error. A lost terminal does not say why. A deleted file says `deleted on
   disk` without the path.
3. About 15 failures go only to stderr, which nobody sees when IVE starts from the desktop file: lost edits,
   terminals that vanish at restart, clicks that do nothing.

The per-surface assessment is `docs/superpowers/specs/2026-10-23-selectable-text-inventory.md` (working file, in
the main checkout). Its calls are the defaults this design follows.

## Goals

### Part 1: selectable text

- **One element** for read-only text that can be selected like text on a web page: press and drag to select, double
  click selects a word, triple click the whole block, Shift+click extends. The selection is drawn with the theme's
  selection colour. Text wraps as before. The I-beam cursor shows over it.
- **One selection in the window at a time.** Pressing in another selectable block, or anywhere else, clears the
  previous selection.
- **Copying** uses the keys the user already has: while a text selection exists and nothing focused takes the key,
  the window treats the place as the editor scope, so `ive.rn`'s Copy (Ctrl+C, Cmd+C on macOS) copies the selected
  text and Select All selects the whole block. Rebinding Copy in `init.rn` therefore applies here too.
- **A block is the unit of selection.** A selection lives inside one block and does not run across blocks. A page
  that should copy as a whole (the Scripts instructions, a failure) also has a Copy button. *(Our call: selecting
  across several elements is a much larger change in gpui; per-block selection covers the uses described.)*
- **Where it applies** (from the inventory): Settings: Scripts build line, instructions, "settings go in" note, "Saved
  to" footer, settings error line, Updates version and status, Third-party versions, licence heading, URL and text;
  prompt detail paragraphs; commit tab header (sha, author and date, message, parents); diff tab header (path,
  source); Git log tab worktree rows, branch rows and stash messages; project overview location (the full path, not
  the `~` form); worktree list group hints; scratches toolbar hint; scratch tab header; run values tab (source,
  variable names, note); run configs tab place line; failed tabs and notices (Parts 2 and 3).
- **Not selectable:** controls and clickable rows (tabs, tree rows, menus, buttons, segmented options, hub rows, log
  rows), tooltips, short empty or loading states. The hub's preview stays as it is (the hub keeps its query focused).

### Part 2: errors that say enough

- **A failure is structured.** A failed tab shows a **summary** line (the error colour), then **details**: labelled
  values (`Command`, `Folder`, `Exit`, `File`, `Cause`…), then **output** when there is some (the last lines the
  program printed, monospace). All of it selectable. A **Copy error** button copies a plain-text report: the title,
  summary, details, output, and the IVE version line (`build_info::summary()`) and OS.
- **MCP:** `list_tabs` returns a failed tab's report as `error` (the same text Copy error copies), so an agent can
  read it without a screenshot.
- **Specific improvements** (from the inventory's worst cases):
  1. **Shell exits right after start:** the failure keeps the terminal's last 40 output lines, and says the shell
     command, the folder, and the exit code or the signal by name (`SIGSEGV (11)`).
  2. **Shell killed by a signal at any time** (not one IVE sent): shows the same failure instead of closing the tab.
     A shell that exits with a code after the first 2 s still closes its tab, as now (`exit 1` typed by the user).
  3. **Lost terminal or run:** says why: the connection error IVE got (today dropped as `Err(_)`), keeping the
     `ive --pty-host attach <id>` command as a detail.
  4. **Hosted terminal ended right after start / pty host did not start:** exit status, the command and folder,
     and which file failed (host executable, registry folder) instead of a bare OS error.
  5. **File deleted while open:** the banner names the path and says whether the tab still holds unsaved edits.
  6. **Run fails to start:** command line, folder, and which `.run.toml` (path relative to the checkout).
  7. **`.run.toml` parse errors** name the file relative to the checkout, not `.run.toml`.
  8. **`vars.toml` and `settings.toml` errors** keep line and column.
  9. **Git errors** keep git2's code and class (`… (class Reference, code NotFound)`).
  10. **Hub "new worktree"** shows the whole error (operation and folder), not only git2's message.
  11. **Run exit** in the sidebar names a signal (`✕ SIGKILL`) instead of `✕ exit 137`.

### Part 3: no silent failures

- **Notices.** A failure that happens away from a tab shows as a notice: a small panel stacked at the bottom right
  of the window, in the error colour, with selectable text, Copy and ×. Notices stay until closed; at most 5, the
  oldest goes first. Each has the time it happened.
- **App log.** Every notice, and every failure that only reached stderr before, is also appended to
  `ive.log` next to `scripts.log` (`~/.local/state/ive/<profile>/ive.log`; macOS next to `init.rn`), one
  timestamped line each, kept to 1 MB like the script log. stderr keeps getting them too.
- **MCP:** a `notices` tool returns the open notices, newest first, and the log file's path.
- **Which failures become notices** (from the inventory): terminal reattach at restart; terminals cannot outlive IVE
  (registry); edits not saved on close or failed save; Run from the tree menu; git actions, diff load, Add to
  .gitignore; hub scratch open/create/reopen; hub file listing; state save; file-watch backend; MCP server stopped;
  open with the OS / in a new window; pasted image not saved; typing paths into a new terminal. Failures raised off
  the UI thread reach the UI through the existing channels, or go to the app log only where no channel exists (the
  plan says which).

## Decided details *(our calls, from the builder check)*

- **Focus and copy.** Pressing in a selectable block takes the keyboard (a focus handle of its own), so a code view
  or terminal loses focus as it does when the user clicks anywhere else; its own selection is left as it is.
  Pressing in a code view or terminal again clears the text selection. Copy therefore always copies what the user
  selected last.
- **Keys while text is selected.** The window uses a new place, `Place::Text`, which matches `editor`-scope
  bindings. Of the built-in actions only Copy (`act::CopyTo`) and Select All act there; the other editing ones
  (Cut, Paste, Undo, Find…) do nothing, as a shown file already ignores the ones it cannot do. An `Invoke` bound in
  the editor scope runs as it does with a file shown (its context is the shown tab).
- **Notices are app-wide:** one list in global state, shown in the main window only (not in detached tab windows),
  and the MCP `notices` tool returns that list. Failures raised off the UI thread go through the existing channels
  to it.
- **Restart:** failed tabs and notices last for the current run only, as failed tabs do today; the app log keeps
  the history. `Failure` is not persisted.
- **"A signal IVE did not send":** IVE already marks a process it stops (`Processes::mark_stopping`, Stop, Kill,
  close, quit). A shell that ends by a signal while not marked shows the failure; that covers reattached shells
  too. A marked one closes as now.

## Non-goals

- Selection across several blocks or a whole page.
- Making clickable rows, tab titles, menus or tooltips selectable. A Copy item on row menus (branch, sha, path) is a
  separate change.
- Changing how the code view or terminals select.
- A notice centre, history UI or filtering; the app log is the history.
- Rewording every message in IVE; only the listed ones change.

## Anti-goals

- Selection must not break clicks: a click without a drag on a selectable link-like element still does what it did.
  Selectable text is never placed inside a clickable row.
- A notice must never steal focus or keys.
- Nothing that today closes quietly as normal behaviour (a shell the user exits) turns into an error.
- No error loses information it has today.

## Left to the builder

- The selectable element's internals (gpui `StyledText` + `TextLayout::index_for_position`, highlight runs for the
  selection, element state keyed by id) and where the "one selection" state lives.
- The `Failure` type's shape and how existing `Tab::Failed { error }` call sites move to it; whether `Copy error` and
  notices share a renderer.
- How the terminal view keeps its last output lines for the failure.
- Notice layout sizes, as theme sizes under `config.theme.sizes` (new keys documented in the README and
  `theme.toml`).
- Tests: word and line ranges, selection text, failure report text, signal naming, each improved message's content,
  notices cap and log append, MCP `list_tabs` error and `notices`.
