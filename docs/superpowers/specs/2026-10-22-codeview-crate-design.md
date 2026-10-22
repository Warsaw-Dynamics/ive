# codeview: one code component for viewing, editing and diffing

Status: design, not yet planned. Date: 2026-10-22.

## Intent

IVE has three separate ways of showing code today:

- `EditorView` (in `src/editor.rs`, next to the tab enum and tab bar): read-only, memory-mapped `Document`,
  tree-sitter colours.
- `TextEditor` (`src/text_editor/`): the scratch editor, plain text, no colours.
- `DiffView` (`src/diff_view.rs`): inline unified diff of git hunks, no colours.

The user wants:

1. A **two-panel (side-by-side) diff** next to the existing inline one, switchable by the user.
2. **Syntax colours in diffs**, in both layouts.
3. **Editing in the same component**, so scratches use it, and diffs can be editable where the context allows.
4. All of it, with tree-sitter and colouring, **extracted into its own crate** beside `ptyhost/`.
5. A clear interface between IVE (the host) and the tab content.

## Goals

- One crate, `codeview/`, owning: text model and edit history, syntax (tree-sitter parse, injections, highlight
  queries), colour keys, line scrolling/metrics, and the gpui views for the three modes.
- Three modes of one component:
  - **View**: read-only text. Must stay fast on enormous files (100M lines): the component never holds the whole
    file, it asks the host for line ranges.
  - **Edit**: editable text with syntax colours that keep up while typing (incremental re-parse).
  - **Diff**: `Inline` or `SideBySide` layout, switchable at runtime; **read-only or editable decided by the host**
    per use (git history and commit tabs: read-only; worktree diff: right side editable).
- Diffs coloured by parsing the **full** old and new texts, then mapping each diff row to its line in them, so
  multi-line strings and block comments colour correctly in hunks.
- IVE keeps everything app-specific: tabs, saved tabs, scratch storage and saving, file watching, git, MCP, Rune
  scripts, settings.

## Non-goals

- A separate process. ptyhost is a process because terminals outlive IVE; code views do not and draw inside IVE's
  gpui window. `codeview` is a **library crate** (path dependency), like ptyhost's library side.
- Highlighting beyond the existing limits (`MAX_HIGHLIGHT_BYTES` = 8 MiB, `PARSE_BUDGET` = 2 s). Big files show
  plain text.
- Editing huge files. Edit mode works on text held in memory (the scratch size limit stays).
- Language servers, completion, folding, minimap: out of scope.
- Changing what git computes (`src/git/diff.rs` stays in IVE).

## Anti-goals (a failure even if it works)

- Opening or scrolling a 10M–100M-line file gets slower than today (today: one newline scan, only visible lines
  decoded, line-number-plus-fraction scrolling in `LineScroll`).
- A 200,000-row diff stops scrolling cheaply (today only visible rows are built).
- `codeview` reaching back into IVE (`crate::workspace`, `crate::script`, `crate::theme`…). Dependencies point one
  way: IVE → codeview.
- Typing lag in a scratch because highlighting runs on the UI thread.

## Constraints

- gpui from the same pinned `pgui` rev as IVE; tree-sitter and all `tree-sitter-*` grammar crates move from IVE's
  `Cargo.toml` to `codeview/Cargo.toml` with the same versions (one copy builds). `streaming-iterator`, `memchr`,
  `unicode-segmentation` as needed.
- `[profile.dev.package."*"] opt-level = 3` already covers a path crate's dependencies; keep `cargo run`
  representative.
- Tests follow the repo style: `should_…` names with `// given`, `// when`, `// then`. Existing tests in
  `src/syntax/tests`, `src/editor.rs`, `src/text_editor/`, `src/diff_view/` move with their code.
- Licence header/metadata like `ptyhost/Cargo.toml` (`publish = false`, proprietary licence).

## The interface (decided in conversation, details are the design's)

Communication is plain in-process Rust: data in through types and traits, events out through gpui's
`EventEmitter`. No messages, no protocol.

### Data in

```rust
/// Where View mode reads lines; the host owns the file. Ranges only: never the whole text.
pub trait LineSource: Send + Sync {
    fn line_count(&self) -> usize;
    /// lines `range` (clamped), already decoded for display (tabs, CRLF, MAX_LINE_CHARS cut)
    fn lines(&self, range: Range<usize>) -> Vec<DisplayLine>;
    /// the whole text when it is small enough to highlight (<= MAX_HIGHLIGHT_BYTES), else None
    fn source(&self) -> Option<Source>;
}
```

- `Document` (mmap + line index) moves into `codeview` and implements `LineSource`; IVE may implement others.
- Edit mode takes a `String` plus `SizeLimit` (as `TextEditor::new` does now).
- Diff mode takes:

```rust
pub struct DiffInput {
    pub files: Vec<FileDiff>,          // codeview's own copy of the hunk types (moved from git/diff.rs's data part)
    pub editable: Editable,            // No | NewSide { limit: SizeLimit }
}
// full texts arrive later, per file, when the view asks for them:
view.set_texts(file_index, old: Option<Source>, new: Option<Source>)
```

  Full texts are loaded **per file, lazily** (my call): the view emits `NeedTexts { file }` for files that are
  expanded and scrolled into view; the host fetches the blobs on a background thread and calls `set_texts`; each file
  gets its own parse job. Files above `MAX_HIGHLIGHT_BYTES` per side are never fetched and show uncoloured. A commit
  tab with hundreds of files therefore costs only what is on screen.

  The pure data types (`FileDiff`, `DiffBody`, `Hunk`, `DiffLine`, `LineKind`, `Cut`) move into `codeview`; IVE's
  `git::diff` builds them from libgit2 and fetches the old/new blobs. Without full texts a file's diff shows uncoloured.

- Language: `Language::for_path(path, content)` lives in `codeview`; the host may override.

### Host services

One struct the host sets as a gpui global (or passes in), replacing direct imports of IVE modules:

```rust
pub struct Host {
    pub colours: ResolvedScheme,       // colour keys → attributes; moved into codeview with ColorKey
    pub style: Style,                  // fonts, sizes (today's DiffSizes and line metrics), UI colours used by views
    pub keys: Rc<dyn Fn(&KeyDownEvent, Place, &App) -> KeyOutcome>,
    pub clipboard: Rc<dyn Fn(ClipboardTarget, String, &mut App)>,
}

pub enum KeyOutcome {
    Command(Command),   // codeview's own enum: moves, selection, copy/cut/paste, undo/redo, toggle diff layout…
    Handled,            // the host ran something (a Rune script binding); the view does nothing
    Unbound,
}
```

IVE's adapter maps `script::keymap::Action`/`Builtin` to `Command` and runs `Action::Invoke` itself. `colors::apply`
keeps building `ResolvedScheme` from `ThemeSpec` in IVE (theme config stays in IVE); the scheme type and the
built-in schemes per key move with `ColorKey`.

### Events out

```rust
pub enum CodeViewEvent {
    Edited { revision: u64 },          // host schedules saves (scratch autosave, worktree file write)
    Refused(String),                   // edit over the size limit
    LayoutToggled(DiffLayout),         // host saves it with the tab and as the default for new diffs
    Scrolled,                          // host persists ScrollPosition / CaretPosition in saved tabs
    OpenAt { path: PathBuf, line: usize }, // e.g. double-click a diff line → open the file there
    NeedTexts { file: usize },         // diff: full old/new texts wanted for colouring this file
}
```

State the host reads/writes: `scroll_position()`, `caret()`, `restore(...)`, `text()` (edit modes),
`set_text` / `set_files` (reload keeping scroll and collapsed files, as `DiffView::set_files` does now),
`reveal_line`, `mark_deleted`. `ScrollPosition`/`CaretPosition` become codeview types; `saved_tabs` stores them.

### Threads

Parsing runs off the UI thread (today's `ParseJob` pattern with `Cancel`). The view hands out a job, the host spawns it
on gpui's background executor, the result comes back through `set_syntax`. Edit mode applies `Tree::edit` and
re-parses incrementally after each edit, a newer edit cancelling an older parse. Colours of a stale parse are shown
shifted until the new one lands (never block typing).

## Behaviour

### Diff layouts

- **Inline**: today's view, plus colours. Removed/added lines coloured from old/new parses respectively.
- **SideBySide**: old on the left, new on the right, rows aligned per hunk: context rows on both sides, a run of
  removals paired with the following run of additions line by line, the shorter side padded with empty filler rows.
  One vertical scroll for both panels; horizontal scroll per panel. Line numbers per side. Selection and copy stay
  within one side.
- Toggle: a button in the diff header and a `Command`. The layout belongs to each tab: switching changes only that
  tab (saved with it, restored on restart). IVE remembers the last layout the user switched to and opens every new
  diff tab in it (user's decision).
- Commit tabs (several collapsible files): both layouts, file headers span both panels.
- Editable diff (worktree), my calls:
  - **Which diffs:** only a single-file diff tab with `DiffSource::Uncommitted` (old = HEAD/index, new = the file on
    disk). `Since { .. }`, commit tabs, the git log and the hub preview stay read-only.
  - **What is shown:** the same hunk rows as today. Edits apply to the **full new text** held by the view (fetched
    through `set_texts`), so the undo history is over the whole file, not over rows.
  - **When it writes:** like scratches: debounced autosave after `Edited`, plus explicit save. IVE reuses the scratch
    `SaveState`/`FileStamp` logic; with unsaved edits and an outside change on disk it shows the same conflict bar
    (reload / overwrite).
  - **After a write:** the host re-runs the diff (existing reload path) and calls `set_files`; the view keeps caret,
    selection and undo history by mapping them through new-side line numbers. A hunk an edit undid disappears; if
    the caret was in it, it goes to the nearest remaining row.
- Old side and all other diffs are always read-only.

### Big files

- View mode only reads `lines(range)` for what is on screen; no highlighting when `source()` is `None` or the parse
  fails/times out (plain text, as today). Diffs keep their 200,000-row view limit and 20,000-line file limit.

### Scratches

- `ScratchView`'s `Editing` body uses Edit mode (gaining colours by the scratch's file name); its `ReadOnly` body uses
  View mode. Save/conflict logic (`SaveState`, `FileStamp`) stays in IVE.

### File tabs become entities

`EditorView` is a plain struct inside `Tab::Document` today, drawn and fed keys by `Workspace`
(`render_selectable(.., &mut Context<Workspace>)`). Using `EventEmitter` means file tabs hold an `Entity<CodeView>`
like scratch, diff and commit tabs already do; this is part of step 2.

## Crate layout (suggested; builder may adjust)

```
codeview/
  Cargo.toml
  src/lib.rs
  src/syntax/        ← src/syntax/ (parse, highlight, injection, language, registry, vendored, tests)
  src/colors/        ← ColorKey, keys/*, scheme, attributes (checkout colours stay in IVE)
  src/document.rs    ← src/document.rs
  src/text/          ← text_editor model + history (edit model), text_selection
  src/line_view.rs   ← src/line_view.rs
  src/view/          ← EditorView body (out of editor.rs), TextEditor view, merged into one CodeView
  src/diff/          ← diff_view.rs, rows, selection; new side_by_side.rs
  src/host.rs        ← Host, KeyOutcome, Command, Style
```

`scrollbar.rs` is used by other IVE views too: either it moves into codeview and IVE uses it from there, or codeview
gets its own minimal copy. My call: move it (IVE already depends on codeview).

## Order of work (suggested)

1. Create the crate; move `syntax` + `colors` keys/scheme + `document`; IVE compiles against it. No behaviour change.
2. Introduce `Host`; move line view, scrollbar, `EditorView` body, `TextEditor`, `DiffView`; IVE adapters.
3. Merge view and edit into one `CodeView`; scratches gain colours; incremental re-parse.
4. Diff colours (full old/new texts from git).
5. Side-by-side layout and the toggle.
6. Editable worktree diff.

Each step ships on its own with `cargo test` and `cargo clippy --all-targets` clean, and is checked in a running IVE
over MCP with an `--profile agent-…` instance.

## Calls made by the designer (not discussed with the user)

- Library crate, not a process.
- Editable diffs: only the new side, only in single-file `Uncommitted` diff tabs (not `Since`, not the hub
  preview); hunk rows shown, full new text edited; debounced autosave with the scratch conflict handling; caret and
  undo kept across re-diffs by new-side line numbers.
- Diff data types move into codeview; libgit2 stays in IVE.
- Scrollbar module moves into codeview.
- Diff full texts load per file, lazily, for what is on screen.
- Rune script bindings stay in IVE behind the `keys` callback.

## Left to the builder

- Exact names, module boundaries inside the crate, how `Style` is shaped.
- Pairing algorithm details for side-by-side within the rule above.
- Whether `Host` is a gpui global or passed per view.
- How incremental re-parse is scheduled (debounce or not), within "never block typing".
