# Markdown: Text | Rendered

## Why

READMEs, CLAUDE.md files, design notes and Markdown scratches are read far more often than they are edited. In IVE
they only show as coloured source: tables are pipes, lists are dashes, a link is brackets. Reading one formatted means
leaving IVE for a browser or another editor. A Markdown tab should switch between its source and the document
formatted, in place, without losing find or the tab's place.

The look and behaviour were settled in the mock (`mock/checkouts.html`, `mock/DESIGN.md` "Markdown: Text |
Rendered", commit `0c40698`): open a README.md tab there and press Ctrl+Shift+M.

## Goals

### 1. Which tabs

- **File tabs and scratch tabs** whose text is Markdown: the language codeview picks for the path is Markdown
  (`.md`, `.markdown`; `Language::for_path`). This includes a read-only file tab (a file gone from disk).
- **Not:** diff and commit tabs of `.md` files, the hub's preview, any other language.

### 2. The switcher: a floating pill

- **Text | Rendered**, two segments, floating at the top right of the text, **where Ctrl+F's find box floats and in
  the same box** (find box border, radius, background and shadow; segments styled like the other segmented controls,
  the picked one in the accent colour).
- **Always shown** on a Markdown tab, in both views. **While the find box is open it takes the pill's place**; closing
  the find box (Esc, ×) brings the pill back.
- No header line is added to file tabs. A scratch keeps its own header line; the pill floats under it, at the top of
  the text, as the find box does.
- Clicking the segment not picked switches. Each segment is a click target for IVE's MCP `click`:
  `markdown-view:text` and `markdown-view:rendered` (as `diff-layout:inline` is).
- Tooltips: "Edit the Markdown source (Ctrl+Shift+M)" and "Read it formatted (Ctrl+Shift+M)", with the key as bound
  (Cmd+Shift+M on macOS).

### 3. The key: Ctrl+Shift+M

- A new built-in action **`act::ToggleMarkdownView`**, bound in `ive.rn`:
  `#{ key: "m", mods: "SECONDARY|SHIFT", action: act::ToggleMarkdownView, scope: "editor" }` (Ctrl+Shift+M; Cmd+Shift+M
  on macOS). Rebindable in `init.rn` like any other.
- It switches the shown Markdown tab between Text and Rendered, from either view. On any other tab it does nothing.
- **Not Ctrl+Shift+V:** that is the terminal's paste on Linux (`terminal_clipboard`), and a habitual paste in Text
  would switch the view instead of pasting.
- The generated `scripting.md` (the actions list) names the new action.

### 4. Text

- Unchanged: the editor as today: coloured source, editable (or read-only for a gone file), saved as you type, undo
  history, caret, find, save bars.
- Switching to Rendered and back keeps the editor as it was: its undo history and unsaved edits (not lost, not forced
  to save). The caret and selection stay unless "Where the caret lands" below moves them.
- **Where the caret lands** on Rendered → Text *(our call, from the builder check)*: where it was when it is among the
  lines now shown; otherwise at the start of the first line shown, nothing selected, so the first key typed edits what
  is on screen. Moving the caret is not an edit: it adds nothing to the undo history.

### 5. Rendered

- The document formatted, **read-only**, from the text as it is now in the editor (unsaved edits included).
- **What renders** (CommonMark plus GitHub's tables, task lists and strikethrough):
  - headings 1 to 6 (1 and 2 with a rule under them), paragraphs, soft breaks as spaces, hard breaks;
  - bold, italic, strikethrough, inline code (monospace, a tinted box), links (accent colour);
  - bullet and numbered lists, nested; task items `[ ]` / `[x]` as greyed, unclickable checkboxes;
  - block quotes (a left rule, muted text), nested;
  - horizontal rules;
  - tables with a header row, bordered cells; column alignment from the separator row;
  - fenced and indented code blocks: monospace in a tinted box, **coloured as the editor colours that language** when
    the fence names one codeview knows (`Language::for_alias`), plain otherwise; long lines scroll sideways inside
    the block, they do not wrap.
- **Shown plainly, not rendered:** raw HTML blocks and inline HTML as their source text (monospace for blocks);
  images as their alt text in brackets with the target, e.g. `[image: logo](assets/logo.png)`; footnote references
  and definitions as their text.
- **Layout:** the UI font at 14 px, line height 1.6; prose at most 760 px wide, from the left with a 32 px margin;
  the page scrolls vertically. Light and dark follow the theme; colours come from the theme (text, muted, border,
  accent, hover tint), not new constants.
- **Live:** the file changing on disk (reload) re-renders, keeping the scroll near where it was.
- **All of it selectable, like a web page** *(the user's call)*: press and drag selects, and a selection runs across
  blocks (from a paragraph into a list, a table, a code block). Double click selects a word, triple click the block
  (a paragraph, a list item, a table cell, a whole code block), Shift+click extends, and Select All (Ctrl+A) selects
  the whole document. The selection is drawn in the theme's selection colour; the I-beam shows over text. Dragging
  past the top or bottom edge scrolls.
- **Copy** (`ive.rn`'s Copy, Ctrl+C / Cmd+C, and the primary selection on Linux as the editor does) copies the
  selected **visible text** as plain text: blocks separated by a blank line, list items on their own lines with their
  marker (`- `, `1. `, `[ ]`, `[x]`), table rows on their own lines with cells separated by a tab, code blocks as
  their lines. Markup characters are not copied.
- Double-click no longer switches views (it selects a word); the pill and the key do.
- **Copy on code blocks** *(the user's call)*: every code block (fenced or indented) has a **Copy** button in its top
  right corner, inside the block's tinted box: 12 px, muted text on the page background with a border and 6 px
  radius, brighter on hover; **shown only while the mouse is over the block** *(the user's call, after the first
  build: always shown, it covered the end of the first line)*, and while it says Copied. A click copies the block's code **as written** (its
  lines, without the fence or the language name, ending without an extra line break) to the clipboard, the
  primary selection too on Linux as the editor's Copy does, and the label says **Copied** for 1.5 s. Click targets
  for MCP: `markdown-copy:<n>`, the block's number from 1 in document order. Find does not match the label, and a
  selection does not take it. The mock shows it.
- **Reading keys** *(added after the first build)*: Page Up and Down, Home, End, ↑ and ↓ scroll Rendered. They are
  `ive.rn` bindings of `act::ScrollPageUp`, `act::ScrollPageDown`, `act::ScrollToTop`, `act::ScrollToBottom`,
  `act::ScrollLineUp` and `act::ScrollLineDown` in a new `"reader"` scope (keys while Rendered has the keyboard; the
  `"editor"` scope's keys apply there too), so `init.rn` can rebind or remove them; a key bound to nothing does not
  scroll.
- **Links do not open** in v1; the target shows as the link's tooltip.
- No "read-only" note: the app has no status line for one (the mock's is mock-only), and the picked Rendered segment
  says it. *(Our call, from the builder check.)*

### 6. Ctrl+F in Rendered

- Ctrl+F, F3 / Shift+F3, Enter / Shift+Enter, the options (match case, whole words, regex), the count ("3 of 12"),
  the marks on the scrollbar strip and Esc all work in Rendered as they do in Text.
- **It searches the visible text**, not the source: `**Java 26**` is found as "Java 26"; markup characters (`#`, `*`,
  `|`, backticks, link targets) are not matched. Text in code blocks, table cells and list items is searched.
- Separate blocks, table cells and list items never join into one word: a whole-word search for `App` finds the
  table header cell "App" next to "Stack".
- Matches are marked with the find colours used in the editor, the current one stronger, and stepping scrolls the
  match into view.
- **The find box stays open across a switch**, keeps its query and options, and searches the view now shown.
- **Esc in Rendered** closes the box and selects nothing (in Text it leaves the caret on the match, as now); the
  editor's caret is untouched until a switch places it.
- Ctrl+F's prefill in Rendered is the selected text (one line of it), else the last query, as in Text.

### 7. Remembered

- **Per tab:** each Markdown tab has its own view, saved with the tab (the panel's saved state, as a diff tab's
  layout is) and restored at restart.
- **Last picked:** a Markdown tab opened for the first time starts in the view last picked on any tab, kept in
  `settings.toml` (`markdown_view = "rendered"`; absent means Text), as `diff_layout` is.
- **Your place is kept:** switching scrolls the other view so the part of the document that was at the top of the
  view is at the top again: Text → Rendered to the block that holds the top line; Rendered → Text to the first
  source line of the block at the top. *(Our call: without it, every switch lands at the top of a long README.)*

### 8. Large files

- Rendered is offered up to the size the editor still colours (`MAX_HIGHLIGHT_BYTES`, 8 MiB). Over it, the Rendered
  segment is greyed with the tooltip "Over 8 MiB: shown as text only", the key does nothing, and a tab saved as
  Rendered opens as Text.
- Rendering never holds the UI: parsing and laying out a big document must not freeze typing in other tabs. *(Bound:
  a 1 MiB README renders without a visible stall; anything slower parses off the UI thread.)*

## Non-goals

- **Diagrams** in fences (Mermaid, Graphviz/DOT, PlantUML): shown as code blocks. A later change, building on the
  diagram work.
- Images, HTML rendering, footnotes as footnotes, math, emoji shortcodes, heading anchors and a table of contents.
- Opening links (relative files in a tab, URLs in the browser).
- Editing in Rendered, clicking task checkboxes to tick them, dragging selected text out.
- A side-by-side Text + Rendered view, scroll sync while both are shown.
- Rendered for diff and commit tabs or the hub preview.
- A Markdown setting per project or per file type beyond "last picked".

## Anti-goals

- Losing edits, undo history or the caret by switching views.
- A pill that hides behind or doubles the find box: exactly one of them shows at the top right.
- Ctrl+Shift+V doing anything new in an editor.
- A Rendered view that searches the source, so counts and marks disagree with what is on screen.
- A Markdown renderer that pulls in a browser engine, a JavaScript runtime or a heavy dependency tree.

## Constraints

- Code lives where it fits the existing split: the view and its find integration in the **codeview** crate (the
  panel owns the find box and the bodies it searches through `find_text`, `show_match`, `set_find_marks`…); the key,
  settings, saved tabs and the switch's click targets in **IVE** (`codeview_host.rs`, `settings.rs`, `saved_tabs.rs`,
  `script/`), as the diff layout's switch is split.
- **Parser:** `pulldown-cmark` (MIT, allowed by `deny.toml`) with default features off, plus tables, task lists and
  strikethrough; it gives source offsets per block, which the place-keeping needs. Add its notice to
  `THIRD-PARTY-NOTICES.md` as the project does for other crates. *(Our call: the tree-sitter Markdown grammar
  codeview already has is made for colouring and is awkward to render from.)*
- AGENTS.md's coding guideline and test layout (`should_…`, given / when / then).
- Each part shows in the running app through IVE's MCP server (`--profile agent-markdown`), not only in tests:
  screenshots of both views, the pill replaced by the find box, Ctrl+Shift+M.

## Left to the builder

- How the panel holds the Rendered body next to the editor (a second body kept alive, a mode on the panel…), as long
  as the editor is not rebuilt by a switch.
- The rendered document's model (blocks with styled runs, their source ranges, the flat visible text that find
  searches and selection and copy index into) and how find marks map onto it; how the scrollbar strip places marks in
  Rendered. Because selection runs across blocks, Rendered lays out its own text and hit-tests it (one offset space
  for the whole document), rather than per-block selectable elements; reusing codeview's `text_selection` where it
  fits.
- Spacing details not stated above, matching the mock where it says.
- Whether colouring code blocks reuses the background parse jobs or a small synchronous path for short blocks.
- Test split between codeview (parsing, flat text, find in Rendered, the panel's switch and saved state) and IVE (the
  key, settings, saved tabs, click targets).

