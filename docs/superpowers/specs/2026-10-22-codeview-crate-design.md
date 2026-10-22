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
  - **Edit**: editable text with syntax colours that keep up while typing (incremental re-parse). Works on files of
    any size (user's decision): the text is a piece table over the host's `LineSource`, never copied whole.
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
- Language servers, completion, folding, minimap: out of scope.
- Changing what git computes (`src/git/diff.rs` stays in IVE).

## Anti-goals (a failure even if it works)

- Opening or scrolling a 10M–100M-line file gets slower than today (today: one newline scan, only visible lines
  decoded, line-number-plus-fraction scrolling in `LineScroll`).
- A 200,000-row diff stops scrolling cheaply (today only visible rows are built).
- `codeview` reaching back into IVE (`crate::workspace`, `crate::script`, `crate::theme`…). Dependencies point one
  way: IVE → codeview.
- Typing lag in a scratch because highlighting runs on the UI thread.
- Typing or saving a big file holding the UI thread, or the whole file crossing the panel/IVE boundary on a save.

## Constraints

- gpui from the same pinned `pgui` rev as IVE; tree-sitter and all `tree-sitter-*` grammar crates move from IVE's
  `Cargo.toml` to `codeview/Cargo.toml` with the same versions (one copy builds). `streaming-iterator`, `memchr`,
  `unicode-segmentation` as needed.
- `[profile.dev.package."*"] opt-level = 3` already covers a path crate's dependencies; keep `cargo run`
  representative.
- Tests follow the repo style: `should_…` names with `// given`, `// when`, `// then`. Existing tests in
  `src/syntax/tests`, `src/editor.rs`, `src/text_editor/`, `src/diff_view/` move with their code.
- Licence header/metadata like `ptyhost/Cargo.toml` (`publish = false`, proprietary licence).

## The interface: actor-style messages (user's decision)

Each panel (a `codeview` view entity) behaves like an Elixir GenServer and IVE like its parent process: IVE **casts**
messages to the panel, **calls** it for answers, and the panel **sends** events back. Messages are delivered
synchronously on the UI thread (`entity.update(cx, |panel, cx| panel.handle(cast, cx))`, `cx.emit` /
`cx.subscribe`), not through a mailbox on another thread: the panel paints every frame from its own state, and an
async hop would add latency to typing and scrolling for no gain. The protocol is still pure messages: one entry
point, ordered, easy to log, test and drive from MCP or Rune.

### Casts: IVE → panel, no answer

```rust
pub enum Cast {
    Open(Content),
    Restore(PanelState),                 // held until the content is known, then clamped and applied
    SetFiles(Vec<FileDiff>),             // diff reload: keeps scroll, collapsed files, caret
    SetTexts { file: usize, old: Option<Source>, new: Option<Source> },  // answer to Event::NeedTexts
    Parsed { target: ParseTarget, revision: u64, syntax: Syntax },        // answer to Event::NeedParse
    Saved { revision: u64, source: Option<Arc<dyn LineSource>> },  // answer to SaveRequested; new source if rewritten
    SaveFailed { revision: u64, reason: SaveFailure },              // Conflict (disk changed) | Io(String)
    Command(Command),                    // keymap, menus, MCP, Rune: moves, copy, undo, toggle layout, Flush…
    RevealLine(usize),
    MarkDeleted,
}

pub enum Content {
    /// View or Edit mode, any size: scratches and files
    Lines { source: Arc<dyn LineSource>, language: Option<Language>, edit: Option<EditPolicy> },
    Diff(DiffInput),                                                     // Diff mode
}

/// Given by IVE, which knows what a write costs (file size, filesystem). The panel runs the timers.
pub struct EditPolicy {
    pub save_delay: Duration,            // pause after the last edit before SaveRequested
    pub limit: Option<SizeLimit>,        // scratches keep their size limit; files have none
}
```

`Restore` may arrive before or after `Open`; a shorter file just clamps the scroll.

### Events: panel → IVE

```rust
pub enum Event {
    Edited { revision: u64 },            // dirty marker; IVE does nothing else
    SaveRequested { base: u64, revision: u64, edits: Vec<Edit> },  // see Saving
    Refused(String),                     // edit over the size limit
    StateChanged,                        // scroll / caret / layout / collapsed changed: IVE marks saved tabs dirty
    LayoutToggled(DiffLayout),           // IVE also remembers it as the layout for the next new diff tab
    NeedParse(ParseJob),                 // IVE runs it on the background executor, answers Cast::Parsed
    NeedTexts { file: usize },           // a diff file came on screen: IVE fetches blobs, answers Cast::SetTexts
    OpenAt { path: PathBuf, line: usize }, // e.g. double-click a diff line → open the file there
}
```

### Calls: IVE asks, the panel answers at once

Plain methods, not a `Call`/`Reply` enum: in Rust a reply enum forces `match … _ => unreachable!()` at every call
site, methods give the same synchronous answer with typed results.

```rust
impl Panel {
    pub fn state(&self) -> PanelState;
    pub fn revision(&self) -> u64;
    pub fn has_unsaved(&self) -> bool;        // the quit prompt; Command::Flush then Saved before quitting
}
```

### State for restart

```rust
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]                             // older saved tabs still load
pub struct PanelState {
    pub scroll: ScrollPosition,               // top line + fraction, left offset (today's type, moved here)
    pub caret: Option<CaretPosition>,         // Edit mode, editable diffs
    pub layout: Option<DiffLayout>,           // Inline | SideBySide, per tab
    pub collapsed: Vec<String>,               // commit tabs: collapsed files, by path
}
```

- **Save:** on `Event::StateChanged` (debounced) and on quit, IVE calls `panel.state()` and stores it in
  `SavedTab { …, panel: PanelState }`. It replaces the separate `ScrollPosition`/`CaretPosition` fields there,
  reading old saved tabs into it.
- **Restore:** IVE creates the panel, casts `Open(content)` and `Restore(state)`.

### Data types behind the messages

```rust
/// Where View mode reads lines; the host owns the file. Ranges only: never the whole text (100M-line files).
pub trait LineSource: Send + Sync {
    fn line_count(&self) -> usize;
    /// lines `range` (clamped), already decoded for display (tabs, CRLF, MAX_LINE_CHARS cut)
    fn lines(&self, range: Range<usize>) -> Vec<DisplayLine>;
    /// the whole text when it is small enough to highlight (<= MAX_HIGHLIGHT_BYTES), else None
    fn source(&self) -> Option<Source>;
}

pub struct DiffInput {
    pub files: Vec<FileDiff>,                 // codeview's own hunk types (the data part of git/diff.rs)
    pub editable: Editable,                   // No | NewSide { limit: SizeLimit }
}
```

- `Document` (mmap + line index) moves into `codeview` and implements `LineSource`; IVE may implement others.
- The pure diff data types (`FileDiff`, `DiffBody`, `Hunk`, `DiffLine`, `LineKind`, `Cut`) move into `codeview`;
  IVE's `git::diff` builds them from libgit2 and fetches the old/new blobs.
- Full diff texts load **per file, lazily** (my call): `NeedTexts` only for files expanded and on screen, each file
  gets its own parse job, files above `MAX_HIGHLIGHT_BYTES` per side are never fetched and show uncoloured. A commit
  tab with hundreds of files costs only what is on screen.
- Language: `Language::for_path(path, content)` lives in `codeview`; the host may override in `Content`.

### Host services: the one direct callback

Key resolution needs an answer within the same keystroke, which would be a call from panel to IVE; it stays a
callback. Together with style it is one struct IVE provides (gpui global or passed in), replacing direct imports of
IVE modules:

```rust
pub struct Host {
    pub colours: ResolvedScheme,       // colour keys → attributes; moved into codeview with ColorKey
    pub style: Style,                  // fonts, sizes (today's DiffSizes and line metrics), UI colours used by views
    pub keys: Rc<dyn Fn(&KeyDownEvent, Place, &App) -> KeyOutcome>,
    pub clipboard: Rc<dyn Fn(ClipboardTarget, String, &mut App)>,
}

pub enum KeyOutcome {
    Command(Command),   // codeview's own enum, the same one Cast::Command carries
    Handled,            // IVE ran something (a Rune script binding); the panel does nothing
    Unbound,
}
```

IVE's adapter maps `script::keymap::Action`/`Builtin` to `Command` and runs `Action::Invoke` itself. `colors::apply`
keeps building `ResolvedScheme` from `ThemeSpec` in IVE (theme config stays in IVE); the scheme type and the built-in
schemes per key move with `ColorKey`.

### Threads

The panel never spawns threads. Parsing runs off the UI thread through the `NeedParse` → background → `Parsed` round
trip (today's `ParseJob` with `Cancel`); a stale `revision` is dropped. Edit mode applies `Tree::edit` and re-parses
incrementally after each edit, a newer job cancelling the older one; colours of the previous parse stay, shifted by
the edit, until the new one lands (never block typing).

### Editing model

The panel's text is a **piece table**: pieces pointing either into the host's `LineSource` (the mmap of the file, never
copied) or into an append-only buffer of typed text. Opening a 100M-line file for editing costs what viewing it
costs; edits cost their own size. Undo history records edits on the piece table. Line lookup keeps the `LineSource`
index plus per-piece line counts, so scrolling stays line-number-plus-fraction as today. Highlighting still applies
only up to `MAX_HIGHLIGHT_BYTES`.

### Saving: IntelliJ-style, no Ctrl+S (user's decision)

Edits reach the disk almost at once without the user doing anything; there is no save shortcut to press.

**Who does what:** the panel decides *when*, IVE decides *how and where*, and only the changes cross between them.

- **When (panel):** `EditPolicy::save_delay` after the last edit, and at once when focus leaves the panel and on
  `Command::Flush` (IVE sends it on tab switch or close, window deactivation, quit, and before a run). The panel then
  emits:

  ```rust
  pub struct Edit {
      pub bytes: Range<u64>,   // range in the text as of `base` (the last saved revision)
      pub text: String,        // what replaces it
  }
  Event::SaveRequested { base: u64, revision: u64, edits: Vec<Edit> }  // sorted, non-overlapping, merged since `base`
  ```

  A hundred keystrokes on one line become one `Edit`. One request is in flight at a time; edits typed meanwhile go
  into the next.

- **Delay (IVE):** IVE picks `save_delay` per panel from what a write costs: 500 ms (today's scratch `SAVE_DELAY`)
  for small files, growing with file size for files that need a rewrite (thresholds are the builder's; the rule is
  "typing never waits, a huge file is not rewritten on every pause"). IVE may update it with a new `EditPolicy`.

- **How (IVE), by edit:**
  - **Same length** (every `Edit` replaces exactly as many bytes as it removes): `pwrite` at each offset, in place.
    Cost = size of the edits, at any file size. The mmap sees the new bytes; the length is unchanged, so no SIGBUS.
  - **Length changes:** write a temp file in the same folder: the unchanged start (up to the first edit) is cloned
    with `FICLONERANGE`/`copy_file_range` where the filesystem can (btrfs, XFS, APFS `clonefile`: instant), copied
    otherwise (ext4, where the user's disks are: in-kernel copy, fast but linear); then the edits and shifted
    remainder are streamed from the old mmap; fsync, rename over the original. Crash-safe, memory stays small. Never
    shrink or rewrite a mapped file in place (SIGBUS).
  - Small files (scratches) may simply splice in memory and write the whole text: same result, simpler.
  - `fallocate` INSERT_RANGE/COLLAPSE_RANGE is not used: it shifts only whole 4 KiB blocks, text edits do not.

- **Answer (IVE):** `Cast::Saved { revision, source }`, with a new `LineSource` for the rewritten file; the panel
  re-points its pieces to it and drops the typed buffer up to `revision`. Failure: `Cast::SaveFailed`.

- **Conflicts:** IVE checks the `FileStamp` against the one it recorded at `base`. A changed file answers
  `SaveFailed { Conflict }` and IVE shows today's scratch conflict bar (reload / overwrite). Merging an outside change
  into unsaved edits automatically is out of scope for now.

- The same rule for scratches, files and editable diffs. IVE's own writes are ignored by its file watcher as today.

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
    through `NeedTexts`/`SetTexts`), so the undo history is over the whole file, not over rows.
  - **When it writes:** as in Saving above, no explicit save.
  - **After a write:** the host re-runs the diff (existing reload path) and casts `SetFiles`; the view keeps caret,
    selection and undo history by mapping them through new-side line numbers. A hunk an edit undid disappears; if
    the caret was in it, it goes to the nearest remaining row.
- Old side and all other diffs are always read-only.

### Big files

- View mode only reads `lines(range)` for what is on screen; no highlighting when `source()` is `None` or the parse
  fails/times out (plain text, as today). Diffs keep their 200,000-row view limit and 20,000-line file limit.

### Scratches

- `ScratchView`'s `Editing` body uses Edit mode (gaining colours by the scratch's file name); its `ReadOnly` body uses
  View mode. Save/conflict logic (`SaveState`, `FileStamp`) stays in IVE; saving follows the Saving rule above.

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

- Library crate, not a process; messages delivered synchronously on the UI thread, not via a mailbox thread.
- Calls are typed methods rather than a `Call`/`Reply` enum; key resolution stays a callback.
- Editable diffs: only the new side, only in single-file `Uncommitted` diff tabs (not `Since`, not the hub
  preview); hunk rows shown, full new text edited; saved per the Saving rule with the scratch conflict handling; caret and
  undo kept across re-diffs by new-side line numbers.
- Diff data types move into codeview; libgit2 stays in IVE.
- Scrollbar module moves into codeview.
- Diff full texts load per file, lazily, for what is on screen.
- One save request in flight at a time; scratches may splice in memory; outside changes give a conflict, no automatic
  merge.
- Rune script bindings stay in IVE behind the `keys` callback.

## Left to the builder

- Exact names, module boundaries inside the crate, how `Style` is shaped.
- Pairing algorithm details for side-by-side within the rule above.
- Whether `Host` is a gpui global or passed per view.
- How incremental re-parse is scheduled (debounce or not), within "never block typing".
