# Markdown Text | Rendered Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Markdown file and scratch tabs switch between their source (Text) and the document formatted (Rendered),
by a floating pill or Ctrl+Shift+M, with find, selection, copy and a Copy button on code blocks working in Rendered.

**Architecture:** A new `codeview::markdown` module parses Markdown with `pulldown-cmark` into a flat document model
(pieces of visible text with styled runs, each block knowing its source lines), off the UI thread. A `RenderedView`
entity draws it with gpui's variable-height `list`, hit-tests its own `StyledText` layouts for a selection that runs
across blocks, and answers the find box through the same hooks the other bodies do. The `Panel` keeps its editor (or
viewer) alive and shows a `RenderedView` beside it when switched; IVE adds the key, the remembered view in
`settings.toml`, and hooks file and scratch tabs.

**Tech Stack:** Rust 2024, gpui (Warsaw-Dynamics/pgui), `pulldown-cmark` (new, MIT), codeview's syntax colouring.

**Spec:** `docs/superpowers/specs/2026-10-26-markdown-rendered-view-design.md` (read it first; the mock is
`mock/checkouts.html`, a README.md tab, Ctrl+Shift+M).

## Global Constraints

- Markdown = `Language::for_path(path, bytes)` is the `"Markdown"` language (`.md`, `.markdown`); only file and
  scratch tabs, never diff/commit tabs or a panel made `unselectable()` (the hub preview).
- Key: `#{ key: "m", mods: "SECONDARY|SHIFT", action: act::ToggleMarkdownView, scope: "editor" }` in `ive.rn`.
- Click targets: `markdown-view:text`, `markdown-view:rendered`, `markdown-copy:<n>` (n from 1, document order).
- Tooltips: `Edit the Markdown source (Ctrl+Shift+M)`, `Read it formatted (Ctrl+Shift+M)` (`Cmd+Shift+M` on macOS),
  `Over 8 MiB: shown as text only` (greyed Rendered segment), `Copy the code` (Copy button).
- Copy button label `Copy`, then `Copied` for 1.5 s.
- `settings.toml`: `markdown_view = "rendered"`; absent means Text.
- Rendered offered up to `MAX_HIGHLIGHT_BYTES` (8 MiB).
- Rendered layout: UI font (inherited, no font family set) 14 px, line height 1.6, prose max 760 px wide, 32 px left
  margin; code in `ViewStyle::mono_font`; colours only from `ViewStyle` / the resolved scheme.
- The pill copies the find box's frame: `find::bar`'s `MARGIN` (6 px from the top, scrollbar thickness + 6 px from
  the right), 1 px `border`, 4 px radius, `background`, `shadow_md`, `occlude`.
- `pulldown-cmark` with `default-features = false`; options tables, tasklists, strikethrough only.
- AGENTS.md coding guideline; tests named `should_…` with `// given`, `// when`, `// then`.
- `cargo test`, `cargo clippy --all-targets` clean after every task.

## Review Focus

- **An empty or whitespace-only Markdown file** switched to Rendered shows an empty page, no panic, find says "No
  results". → Task 1 `should_parse_an_empty_text_to_no_blocks`; Task 5 `should_render_an_empty_document`.
- **Editing in Text, then switching** shows the unsaved edits in Rendered (not the file on disk). → Task 6
  `should_render_the_unsaved_edits`.
- **A file reloaded from disk while Rendered is shown** re-renders and keeps the top block. → Task 6
  `should_rerender_on_new_text_keeping_the_top_block`.
- **Malformed Markdown** (unclosed fence, ragged table rows, a table row with more cells than the header) renders
  without panicking; the unclosed fence runs to the end. → Task 1 `should_close_an_unclosed_fence_at_the_end` and
  `should_pad_or_cut_ragged_table_rows`.
- **A saved tab from before this feature** (no `markdown` in its panel state) opens in the last-picked view, and a
  non-Markdown tab never gains one. → Task 8 `should_restore_a_tab_saved_without_a_markdown_view`.

---

### Task 1: The document model and parser

**Files:**
- Modify: `codeview/Cargo.toml` (add `pulldown-cmark = { version = "0.13", default-features = false }`)
- Modify: `codeview/src/lib.rs` (add `pub mod markdown;`)
- Create: `codeview/src/markdown/mod.rs`, `codeview/src/markdown/document.rs`, `codeview/src/markdown/document_tests.rs`
- Modify: `THIRD-PARTY-NOTICES.md`, `assets/licenses/third-party.json` (regenerated)

**Interfaces:**
- Produces (in `codeview::markdown`):
  ```rust
  pub struct Doc { pub text: String, pub blocks: Vec<Block> }   // text: every piece's text joined by '\n'
  pub struct Block { pub kind: BlockKind, pub nesting: Nesting, pub lines: Range<usize>, pub pieces: Vec<Piece> }
  pub struct Nesting { pub quotes: u8, pub lists: u8 }
  pub enum BlockKind { Heading(u8), Paragraph, ListItem(Marker), Code { language: Option<Language> }, Rule,
                       Table { columns: Vec<Align>, header_cells: usize }, Html }
  pub enum Marker { Bullet, Number(u64), Task(Tick) }
  pub enum Tick { Open, Done }
  pub enum Align { Left, Center, Right }
  pub struct Piece { pub text: Range<usize>, pub runs: Vec<Run> }  // ranges are bytes of Doc::text
  pub struct Run { pub bytes: Range<usize>, pub look: Look }
  pub enum Look { Strong, Emphasis, StrongEmphasis, Strike, Code, Link(SharedString), Syntax(ColorKey) }
  pub fn parse(source: &str) -> Doc;
  impl Doc { pub fn block_at_line(&self, line: usize) -> usize; pub fn piece_starts(&self) -> Vec<usize>; }
  ```
  `lines` are the block's 0-based source lines (end exclusive). `block_at_line` is the last block whose
  `lines.start <= line` (0 for an empty doc). A table's pieces are its cells, row by row, header first. A rule has one
  empty piece. A list item's piece is its first paragraph; further paragraphs in it are `Paragraph` blocks with the
  item's `nesting.lists`. Raw HTML is an `Html` block (source text); inline HTML is plain text; an image is the text
  `[image: <alt>](<target>)`; footnotes are their text.

- [ ] **Step 1: Write the failing tests** in `document_tests.rs`:
  - `should_parse_an_empty_text_to_no_blocks`: `parse("")` and `parse(" \n\n")` → `blocks.is_empty()`, `text == ""`.
  - `should_parse_headings_paragraphs_and_inline_looks`: `"# Title\n\nSome **bold** and *it* and `code` and [l](u).\n"`
    → kinds `[Heading(1), Paragraph]`; piece texts `"Title"`, `"Some bold and it and code and l."`; runs on
    `"bold"` Strong, `"it"` Emphasis, `"code"` Code, `"l"` `Link("u")`; `blocks[1].lines == 2..3`.
  - `should_nest_lists_and_tasks`: `"- [x] a\n- [ ] b\n  - c\n1. d\n"` → `ListItem(Task(Done))` "a" lists 1,
    `ListItem(Task(Open))` "b" lists 1, `ListItem(Bullet)` "c" lists 2, `ListItem(Number(1))` "d" lists 1.
  - `should_quote_nested_blocks`: `"> q\n> - i\n"` → Paragraph "q" quotes 1, ListItem "i" quotes 1 lists 1.
  - `should_make_table_cells_pieces`: `"| App | Stack |\n|:---|---:|\n| ive | Rust |\n"` → one `Table { columns:
    [Left, Right], header_cells: 2 }` with piece texts `["App", "Stack", "ive", "Rust"]`.
  - `should_pad_or_cut_ragged_table_rows`: a body row with 1 cell and one with 3 cells under a 2-column header → 2
    pieces per row (an empty one added, the third dropped).
  - `should_keep_code_lines_and_language`: "```rust\nfn a() {}\n\n```\n" → `Code { language: Some(Rust) }`, piece text
    `"fn a() {}\n"`; an unknown fence word gives `language: None`; an indented block is `Code { language: None }`.
  - `should_close_an_unclosed_fence_at_the_end`: "```\nx\ny" → one Code block, text `"x\ny"`, lines `0..3`.
  - `should_show_html_and_images_as_text`: `"<div>hi</div>\n\n![logo](a.png)\n"` → `Html` "<div>hi</div>",
    Paragraph `"[image: logo](a.png)"`.
  - `should_join_pieces_with_line_breaks`: for the table test, `doc.text == "App\nStack\nive\nRust"` and
    `piece_starts() == [0, 4, 10, 14]`.
  - `should_find_the_block_at_a_source_line`: `"# A\n\npara\n\n- x\n"` → `block_at_line(0) == 0`, `(1) == 0`,
    `(2) == 1`, `(4) == 2`, `(99) == 2`.
- [ ] **Step 2:** `cargo test -p codeview markdown::` → fails (module missing).
- [ ] **Step 3:** Implement `parse` over `pulldown_cmark::Parser::new_ext(source, options).into_offset_iter()`,
  tracking a stack of open containers; line numbers from a precomputed line-start table (`partition_point`). Trailing
  newline of a code block's last line is kept as pulldown gives it; strip one trailing `\n` only for the Copy button
  (Task 4), not here.
- [ ] **Step 4:** `cargo test -p codeview markdown::` passes; `cargo clippy --all-targets` clean.
- [ ] **Step 5:** Run `tools/third-party-notices.sh` (cargo-deny + cargo-about are installed); it must pass and
  rewrite the notices.
- [ ] **Step 6: Commit** `codeview: Markdown parsed to a document of pieces and blocks (pulldown-cmark)`.

### Task 2: Copying and selecting by word and block (pure)

**Files:**
- Create: `codeview/src/markdown/copy.rs` (+ tests in the same file's `mod tests`)

**Interfaces:**
- Consumes: Task 1's `Doc`, `Block`, `Piece`, `Marker`, `Tick`.
- Produces:
  ```rust
  impl Doc {
      pub fn copy_text(&self, range: Range<usize>) -> String;      // range: bytes of Doc::text
      pub fn code_of(&self, block: usize) -> Option<String>;        // a Code block's code as written, no final '\n'
      pub fn word_at(&self, offset: usize) -> Range<usize>;         // within its piece
      pub fn piece_at(&self, offset: usize) -> (usize, usize);      // (block, piece index in the block)
      pub fn piece_range(&self, offset: usize) -> Range<usize>;     // the whole piece holding offset (triple click)
  }
  ```
- [ ] **Step 1: Write the failing tests**:
  - `should_copy_blocks_separated_by_blank_lines`: `"# A\n\npara one\n"` all → `"A\n\npara one"`.
  - `should_copy_list_items_with_their_markers`: `"- a\n- [x] b\n1. c\n"` all → `"- a\n- [x] b\n1. c"`; nested items
    indented by two spaces per level.
  - `should_copy_table_rows_with_tabs`: the Task 1 table all → `"App\tStack\nive\tRust"`.
  - `should_copy_a_part_of_one_piece`: copying bytes 1..3 of `"para"` → `"ar"`.
  - `should_copy_code_as_written`: "```\nx\n  y\n```" → `code_of(0) == Some("x\n  y")`; a Paragraph → `None`.
  - `should_select_a_word_within_its_piece`: in `"App\nStack"`, `word_at(1) == 0..3` (never across the `\n`).
  - `should_select_the_whole_piece`: `piece_range(5) == 4..9` for `"App\nStack"`.
- [ ] **Step 2:** run → fail. **Step 3:** implement (markers only where the range starts at or before a list
  item's piece start; a word is letters, digits, `_`, as `text_selection::word_around` counts them).
- [ ] **Step 4:** pass, clippy clean. **Step 5: Commit** `codeview: Rendered Markdown copies as plain text`.

### Task 3: Code blocks coloured, the document built off the UI thread

**Files:**
- Create: `codeview/src/markdown/build.rs`

**Interfaces:**
- Consumes: Task 1's `parse`, `Look::Syntax`; `crate::syntax::{parse, highlight::spans, Cancel, PARSE_BUDGET}`.
- Produces: `pub fn build(source: &[u8], cancel: &Cancel) -> Doc` — `parse` (lossy UTF-8), then each `Code { language:
  Some(_) }` piece gets `Look::Syntax(key)` runs from `syntax::parse` + `highlight::spans` over the code text; a block
  over 256 KiB, a parse error or a cancel leaves it plain.
- [ ] **Step 1: Write the failing tests**: `should_colour_a_known_fence` ("```rust\nfn a() {}\n```" → a run with
  `Look::Syntax(_)` covering `fn`); `should_leave_an_unknown_fence_plain` (no `Syntax` runs);
  `should_stop_when_cancelled` (a cancelled `Cancel` → no `Syntax` runs, same text).
- [ ] **Step 2:** fail. **Step 3:** implement. **Step 4:** pass. **Step 5: Commit** `codeview: Rendered code blocks
  coloured as the editor colours them`.

### Task 4: `RenderedView`: drawing, scrolling, find hooks, Copy buttons

**Files:**
- Create: `codeview/src/markdown/view.rs`, `codeview/src/markdown/view_tests.rs`
- Modify: `codeview/src/scrollbar.rs` (a `Scrollable` for `gpui::ListState`: `BlockScroll`)

**Interfaces:**
- Consumes: Tasks 1–3; `FindText::Parts`, `FindMarks`, `scrollbar::{ScrollState, vertical_scrollbar_with_ticks}`,
  `Host::{click_target, write_clipboard, tooltip}`, `ViewStyle::drawing`, `ResolvedScheme`.
- Produces:
  ```rust
  pub struct RenderedView;   // gpui entity; Render + Focusable
  impl RenderedView {
      pub fn new(doc: Arc<Doc>, cx: &mut Context<Self>) -> Self;
      pub fn set_doc(&mut self, doc: Arc<Doc>, cx: &mut Context<Self>);   // keeps the top block (clamped)
      pub fn top_line(&self) -> usize;                                     // first source line of the top block
      pub fn reveal_source_line(&mut self, line: usize, cx: &mut Context<Self>); // block_at_line to the top
      pub fn find_text(&self) -> FindText;            // Parts { bytes: doc.text, starts: piece_starts }
      pub fn find_anchor(&self) -> Range<usize>;      // the top block's first piece start, or the selection
      pub fn show_match(&mut self, bytes: Range<usize>, cx: &mut Context<Self>);
      pub fn set_find_marks(&mut self, marks: Option<FindMarks>, cx: &mut Context<Self>);
      pub(crate) fn has_find_marks(&self) -> bool;
      pub fn doc_revision(&self) -> u64;              // +1 per set_doc
  }
  ```
  One list item per block (`ListState::new(blocks.len(), ListAlignment::Top, px(400.))`); each piece a `StyledText`
  whose highlights are its runs' looks, the selection (Task 5) and the find marks (`find_match` / `find_current`
  backgrounds, current over selection over match). Scrollbar ticks: a match on piece `p` at `block_of(p) / blocks.len()`.
  Code blocks: tinted box (`hover` over `background`), `whitespace_nowrap`, horizontal scroll inside, Copy button top
  right (`markdown-copy:<n>`; label not part of any piece).
  Headings 1 and 2 draw a `border` rule under them; links are `accent`, their target as a tooltip over the run
  (`InteractiveText::tooltip`), and do not open. The Copy button writes `ClipboardTarget::Both` on Linux,
  `Clipboard` elsewhere, as the editor's Copy does.
- [ ] **Step 1: Write the failing tests** (gpui `TestAppContext`, `ViewStyle::set_for_test`):
  - `should_render_an_empty_document` (no panic, `find_text` has no parts).
  - `should_search_pieces_apart`: doc of the Task 1 table; `FindQuery { text: "App", whole_word }` over
    `find_text()` → 1 match at `0..3`; query `"pStack"` → 0 matches.
  - `should_scroll_a_match_into_view`: 200 paragraphs, `show_match` on the 150th's bytes → its block is in
    `list_state.visible range` after a frame.
  - `should_reveal_a_source_line_at_the_top`: 200 paragraphs (2 lines each), `reveal_source_line(100)` → `top_line()
    == 100`.
  - `should_keep_the_top_block_on_a_new_doc`: scrolled to block 50, `set_doc` with the same text plus one paragraph
    at the end → top block still 50.
  - `should_show_a_link_target_as_its_tooltip` (hovering a link run shows `u` for `[l](u)`).
  - `should_copy_a_code_block_and_say_copied`: a Host whose clipboard records; clicking target `markdown-copy:1`
    writes `"x\n  y"`; the button reads `Copied`, and `Copy` again after advancing the clock 1.5 s.
- [ ] **Step 2:** fail. **Step 3:** implement. **Step 4:** pass, clippy. **Step 5: Commit** `codeview: RenderedView
  draws a Markdown document, scrolls by block, answers find, copies code blocks`.

### Task 5: Selection across blocks and the keys of Rendered

**Files:**
- Modify: `codeview/src/markdown/view.rs`, `codeview/src/markdown/view_tests.rs`

**Interfaces:**
- Consumes: Task 2's `copy_text`, `word_at`, `piece_range`; `text_selection::TextSelection`;
  `Host::key_outcome(event, KeyScope::Viewer, cx)`.
- Produces: `pub fn selected_text(&self) -> Option<String>` (copy format), `pub fn find_prefill(&self) ->
  Option<(String, usize)>` (the selection when on one line), `pub fn select_all(&mut self, cx)`, `pub fn
  copy(&self, target: ClipboardTarget, cx: &mut App)`. Hit testing: keep each drawn piece's `TextLayout` with its
  `Doc::text` start for the frame; a press/move at the list's level maps a point to the nearest piece (by bounds:
  above the first → its start, below the last → its end) then `index_for_position`. Drag past the top/bottom edge
  scrolls by one line height per move. Keys: Copy and Select All run here; every other key goes on (as `CodeView`).
- [ ] **Step 1: Write the failing tests**: `should_select_across_blocks_by_dragging` (press in block 0's piece,
  drag into block 2's → `selected_text()` covers both with the blank-line separator);
  `should_select_a_word_on_double_click`; `should_select_the_piece_on_triple_click`;
  `should_extend_with_shift_click`; `should_select_all_and_copy` (key bound to `CopyTo(Clipboard)` through a test
  Host → clipboard holds `copy_text(0..len)`); `should_not_select_the_copy_label` (drag over a code block's button
  area selects only code text).
- [ ] **Step 2–4:** fail, implement, pass. **Step 5: Commit** `codeview: Rendered Markdown selects across blocks and
  copies`.

### Task 6: The panel switches Text | Rendered

**Files:**
- Modify: `codeview/src/host.rs` (`Command::ToggleMarkdownView`; rename `is_find` → `runs_in_panel`, true for the find
  commands and `ToggleMarkdownView`; update its four callers)
- Modify: `codeview/src/panel/protocol.rs` (`MarkdownView`, `Cast::SetMarkdownView`, `Event::MarkdownViewToggled`)
- Modify: `codeview/src/panel/state.rs` (`PanelState::markdown: Option<MarkdownView>`, `skip_serializing_if = "Option::is_none"`)
- Modify: `codeview/src/panel/mod.rs`, `codeview/src/panel/find.rs` (find dispatch to the shown body)
- Create: `codeview/src/panel/markdown.rs`, `codeview/src/panel/markdown_tests.rs`
- Modify: `codeview/src/text_editor/view.rs` (`pub(crate) fn shown_lines(&self) -> Range<usize>`)

**Interfaces:**
- Consumes: Task 3's `build`, Tasks 4–5's `RenderedView`.
- Produces:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)] #[serde(rename_all = "lowercase")]
  pub enum MarkdownView { #[default] Text, Rendered }
  Cast::SetMarkdownView(MarkdownView)       // the view to show; before content: the default for it
  Event::MarkdownViewToggled(MarkdownView)  // the user switched (key, pill); not on restore or SetMarkdownView
  impl Panel { pub fn markdown_view(&self) -> Option<MarkdownView>;   // None: not offered (not Markdown, unselectable, diff)
               pub fn markdown_offered(&self) -> Offer; }             // Offer::{No, TooLarge, Yes}
  ```
  The panel keeps its `Body` (editor or viewer) and, while Rendered, a `RenderedView` entity shown instead of it
  (keys, find, focus go to the shown one). Switching to Rendered: take the body's text (`body_find_text` → bytes),
  `build` on the background executor (cancel the previous), then show it scrolled to the body's top line. Switching
  to Text: scroll the body to `RenderedView::top_line()`; keep the caret if it is in the editor's `shown_lines()`,
  else `restore_view` with the caret at the first shown line, column 0, nothing selected (no undo entry). New content
  (`Cast::Open` reload) while Rendered: rebuild and `set_doc`. Text over `MAX_HIGHLIGHT_BYTES`: `Offer::TooLarge`, the
  command does nothing, a Rendered restore or `SetMarkdownView(Rendered)` shows Text. The find box: after a switch,
  `refind` against the shown body; Esc in Rendered closes without touching the editor.
- [ ] **Step 1: Write the failing tests** in `markdown_tests.rs`:
  - `should_offer_markdown_only_for_markdown_lines` (a `.md`-language Lines panel → `Some(Text)`; Rust → `None`;
    `unselectable()` → `None`; a diff → `None`).
  - `should_toggle_and_emit_once` (`Cast::Command(ToggleMarkdownView)` → `markdown_view() == Some(Rendered)`,
    exactly one `MarkdownViewToggled(Rendered)` and a `StateChanged`).
  - `should_render_the_unsaved_edits` (edit mode, `Cast::Type("# New\n")`, toggle → the rendered doc's first piece
    is `"New"`).
  - `should_keep_undo_and_caret_across_switches` (type, toggle twice, `Command::Undo` undoes the typing; caret where
    it was when it was in view).
  - `should_put_the_caret_on_screen_after_reading_elsewhere` (200-paragraph text, caret line 0, Rendered, reveal
    source line 300, back to Text → caret line == first shown line, column 0).
  - `should_keep_the_place_both_ways` (editor top line 100 → Rendered `top_line()` is the block holding 100's first
    line; back → editor top line equals it).
  - `should_rerender_on_new_text_keeping_the_top_block` (Rendered, scrolled; `Cast::Open` with an extra paragraph
    appended → same top block).
  - `should_save_and_restore_the_view` (`state().markdown == Some(Rendered)` after toggling; a new panel with
    `Cast::Restore(that state)` then `Cast::Open` shows Rendered; with `markdown: None` and `SetMarkdownView(Rendered)`
    first, shows Rendered).
  - `should_find_in_the_view_shown` (`**Java 26**` text: Rendered find "Java 26" → 1 match; Text → 1 match at the
    source bytes; the box stays open with its query across the toggle).
  - `should_refuse_rendered_over_the_limit` (text of `MAX_HIGHLIGHT_BYTES + 1` bytes → `Offer::TooLarge`, toggle
    keeps `Text`, no event).
- [ ] **Step 2–4:** fail, implement, pass; also all existing `codeview` tests (`cargo test -p codeview`).
- [ ] **Step 5: Commit** `codeview: the panel switches a Markdown text between Text and Rendered`.

### Task 7: The pill

**Files:**
- Create: `codeview/src/markdown/pill.rs`
- Modify: `codeview/src/panel/mod.rs` (render: `.children(self.pill(cx))` before `find_bar`; none while the find box
  is open), `codeview/src/panel/markdown_tests.rs`

**Interfaces:**
- Consumes: Task 6's `markdown_view`, `markdown_offered`; `find::bar`'s frame constants (make `MARGIN` and
  `BORDER` `pub(crate)`); `Host::click_target`, `Host::tooltip`.
- Produces: `pub(crate) fn pill(shown: MarkdownView, offer: Offer, switch: Rc<dyn Fn(MarkdownView, &mut Window, &mut App)>, cx: &App) -> AnyElement`.
  Two segments `Text` / `Rendered`, the picked one `accent` background with `background` text, the other `text`
  colour with `hover` on hover; `TooLarge` greys Rendered (`muted`, no click) with its tooltip. Key text in tooltips:
  `Cmd+Shift+M` under `cfg(target_os = "macos")`, else `Ctrl+Shift+M`.
- [ ] **Step 1: Write the failing tests**: `should_show_the_pill_until_find_opens` (after `Command::Find` no element
  with id `markdown-pill`; after Esc it is back); `should_switch_by_click_target` (clicking registered
  `markdown-view:rendered` → `Some(Rendered)`; clicking `markdown-view:text` → `Some(Text)`; clicking the picked one
  emits nothing); `should_grey_rendered_over_the_limit` (`markdown-view:rendered` not registered).
- [ ] **Step 2–4:** fail, implement, pass. **Step 5: Commit** `codeview: the Text | Rendered pill floats where the find box does`.

### Task 8: IVE: the key, the remembered view, file and scratch tabs

**Files:**
- Modify: `src/script/keymap.rs` (`Builtin::ToggleMarkdownView`, `supports` → `scope == Scope::Editor`),
  `src/script/declare.rs` (`"ToggleMarkdownView" => Builtin::ToggleMarkdownView`), `src/script/config.rs` (the name list)
- Modify: `src/codeview_host.rs` (`command_of`: `Builtin::ToggleMarkdownView => Command::ToggleMarkdownView`;
  `pub fn remember_markdown_view(panel: &Entity<Panel>, cx: &mut App)`: casts `SetMarkdownView(settings.markdown_view)`
  and saves the setting on `Event::MarkdownViewToggled`)
- Modify: `src/settings.rs` (`markdown_view: MarkdownView`, `skip_serializing_if` Text, `with_markdown_view`)
- Modify: `src/file_tab.rs:50`, `src/scratch/tab.rs:241` (call `remember_markdown_view` right after `serve`, before
  any `Cast::Restore`)
- Modify: `src/saved_tabs.rs` (test only), `assets/scripts/ive.rn` (the binding, after `act::FindPrevious`'s), `README.md`
  (actions table row: `| \`act::ToggleMarkdownView\` | editor | a Markdown file or scratch between Text and Rendered
  (Ctrl+Shift+M; macOS Cmd+Shift+M); new Markdown tabs open in the view last switched to |`)

**Interfaces:**
- Consumes: Task 6's `MarkdownView`, `Cast::SetMarkdownView`, `Event::MarkdownViewToggled`, `Command::ToggleMarkdownView`.
- [ ] **Step 1: Write the failing tests**:
  - `codeview_host`: `should_map_toggling_the_markdown_view_to_a_command` (as the diff-layout one);
    `should_open_markdown_tabs_in_the_view_last_switched_to` (a store with `markdown_view = Rendered` → a new md
    panel shows Rendered; toggling saves `Text`).
  - `settings`: `should_round_trip_the_markdown_view` and `should_leave_text_out_of_the_file`.
  - `script` (as `should_bind_find_in_files_and_its_hub_keys`): `should_bind_toggle_markdown_view` — `ive.rn`
    binds `cmd-shift-m` on macOS, `ctrl-shift-m` elsewhere, in the editor place; `ctrl-shift-v` stays unbound there.
  - `saved_tabs`: `should_restore_a_tab_saved_without_a_markdown_view` (a saved panel state TOML without `markdown`
    loads with `markdown: None`; one with `markdown = "rendered"` round-trips).
- [ ] **Step 2–4:** fail, implement, pass; `cargo test`, `cargo clippy --all-targets`.
- [ ] **Step 5: Commit** `Markdown tabs: Ctrl+Shift+M and the view last switched to; README`.

### Task 9: Seen working in the app

**Files:** none (fixes found go to the task that owns the code, as new commits)

- [ ] **Step 1:** `cargo build --release`; start `./run.sh -- --profile agent-markdown --background --stop-processes-on-quit .`
  and use the `ive` MCP tools against it (URL and key in `~/.config/ive/agent-markdown/mcp.toml`).
- [ ] **Step 2:** Open `README.md`: screenshot Text with the pill; `click markdown-view:rendered`, screenshot
  (headings, tables, lists, code with Copy); `send_text` Ctrl+F `cargo`, screenshot (pill gone, matches marked);
  Esc; screenshot (pill back); press Ctrl+Shift+M → Text, scrolled to the same place.
- [ ] **Step 3:** Select across two blocks by MCP drag if available (else note it as checked by tests only); click
  `markdown-copy:1`, read the clipboard.
- [ ] **Step 4:** Quit IVE (with `--stop-processes-on-quit`), restart with the same profile: the tab reopens Rendered.
- [ ] **Step 5:** Stop the instance. Report each check with its screenshot; anything that failed is fixed in its own
  commit with a test.
