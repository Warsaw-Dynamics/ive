# Task 3G — native Salt widgets and tables

Status: implemented and frozen for controller-arranged independent SPEC/QUALITY review. This is one bounded Salt increment, not full PlantUML compatibility.

- Released independently reviewed base: `d84c31464d6c85bbdb4a78e69d64d3578e63d3db`.
- Frozen head: `82fad891549ce2ec70b77b48dcdb8c0929ed41ef` — `Add bounded native PlantUML Salt widgets and tables`.
- Existing worktree `/mnt/data/Projects/ive/.worktree/diagram-ports`, branch `diagram-ports`; tracked worktree clean after commit. Exactly18 scoped standalone files committed. No root IVE product, dependency/lockfile/vendor, other family, runtime service, worktree/branch creation, merge or push changes. No subagents/reviewers dispatched.
- Read complete [brief](task-3g-brief.md), [constraints](native-global-constraints.md), [Task7 Salt deliverable](task-3e-plan-extract.md), specialized semantics/exact source ledger, and preceding [3F](task-3f-report.md), [3E](task-3e-report.md), [3D2 including fixes](task-3d2-report.md). Shared Context/Label/Scene/style/chrome/origin/measurement/worker APIs and limitations remain their linked [unchanged3D contract](task-3d-report.md). TDD and verification skills informed implementation and final gates.

## Model, lexical ownership and grammar

`salt::parse(&[Line], &mut Budget) -> Salt` creates a dedicated frozen postorder widget arena. `Widget { id, kind }` is `Kind::Leaf(Leaf { kind, raw })` or `Kind::Table(Table { border, cells, rows, columns })`. Each table owns source-ordered `Cell { id, row, column, row_span, column_span, widget }`; stable widget/cell identities are distinct, child indices precede their parent, and spans are positive. Row spans are currently one; exact `*` extends only the contiguous preceding cell in the same row. Orphan/cross-row merges reject. No child AST cloning or generic graph/Mermaid conversion.

Only exact `@startsalt`/`@endsalt`, without filenames/aliases, is admitted. Shared preprocessing recognizes Salt as raw lexical input, preserving apostrophes, comment-looking text, quotes and physical rows. Prefix fonts/styles/chrome/literal constants use existing services; definitions inside tables and interleaved style/chrome requests reject. Existing raw note/action/data and all other family regressions remain passing.

Typed tokenizer syntax distinguishes leaf text from actual opening/closing braces even when an escaped leaf equals `{` or `}`. Direct `\|`, `\{`, `\}` escapes preserve literal delimiters; genuine U+E000/U+E001/U+E002 remain unchanged. No sentinel substitution or global UML quote/comment rewriting. Quotes do not hide unescaped structural delimiters. Plain opening `{` can immediately precede ordinary text, as in source `{foo}`; it is tokenized as a table plus text. Reserved structured markers are separate grammar. Empty separator tokens and blank physical rows are skipped, matching source. A trailing unescaped pipe leaves a NEWCOL terminator across the next physical line. The nested closing token determines its one parent cell's following column/row, including same-line closes and adjacent closes. Whole input must be consumed.

Admitted controls: ordinary text; quoted field; uppercase `[X]` checked checkbox, `[]`/`[ ]` unchecked; `[caption]` button; uppercase `(X)` selected radio, `()`/`( )` unselected. Field→checkbox→button precedence keeps `[X]` an empty-label checkbox; `[x]` is a button. Fields retain raw interior padding separately from trimmed paint. Native minimum width is max(measured caption width,8×UTF-16 units of the longest decoded visible line), plus field/button padding. Plain padding follows the source8-unit policy. Formatted/hard-break counting deliberately differs from upstream raw/tag-purged length; this is recorded in README/provenance/notices. Shared Creole/backslash restrictions still apply.

Tables admit `{` no border, `{+` outer border and `{#` full occupied-cell grid. Horizontal/vertical/titled borders, cardinal-region layout, tree/tab/menu/dropdown/scroll/line widgets, image/dictionary/resource/link requests, malformed widgets and substantive unsupported structure reject. Inert URLs/paths and ordinary unpaired caret/punctuation remain literal data. Rejection-only source traces are not feature support. Missing-close diagnostics currently use the existing coarse line1 fallback; other token errors retain physical line/offending text.

## Measured layout and safety

`saltscene::layout(&Salt, &mut Context) -> Scene` measures each leaf and table once bottom-up. Single-column minima are solved first; each colspan then grows its final covered column until its total covered width contains the measured child plus padding. Monotonic growth preserves earlier constraints. Row heights include measured nested tables and multiline labels. Sparse cells plus cumulative row/column boundary arrays replace dense rows×columns storage. A200-cell ragged case retains201 widgets and203 boundary entries rather than10,100 grid slots.

Concrete drawing: underlined field with two endstrokes; rounded button with centered caption;10-unit square/check polygon;10-unit radio circle and4-unit selected dot. Controls use a20-unit label inset. Full-grid internal segments stop at span boundaries, so no vertical line crosses a spanning cell. Root/nested borders, stable cell→widget ownership and source order remain inspectable in scene/SVG metadata. Native gap6, field/button padding, shared strokes/colors and baseline differ from upstream pixels.

One existing Budget, Context, native session, bounded writer and terminal resvg-safe/resource-admission pipeline remain in force. GPUI-free caller-memory engine; no added filesystem/network/environment/clock/locale/process/external-renderer capability. Defaults remain source256KiB/SVG16MiB/model10,000/depth64/work5,000,000/cooperative10s; Unix worker hard10s/2GiB; labels16KiB/queries4096/finite coordinates1,000,000 and existing raster caps unchanged. Tokens, cell/span/model retention, text/metadata, boundary arrays, scans/span sums and generated geometry charge shared work/text/items before construction. Cancellation checks stay inside growing loops and around host callbacks. The real callback-cancellation test stops after exactly one query. Tiny public source/model/depth/work/SVG limits assert the actual intended guard; the SVG case reaches the writer, not the expanded-text limit.

## Provenance and reference evidence

Personally inspected actual selected sources under the sole authorized `/mnt/data/Projects/plantuml-mit/src` tree. The [manifest](../../../diagram-ports/provenance/plantuml-mit-1.2026.9.json) adds26 exact Salt source paths/SHA256/destinations/behavior/actual credit entries, four fixtures and two native modules. Controller independently matched all26 to extracted bytes, exact source-JAR members and dated/author/contributor headers; source selection did not change afterward. All credit `(C) Copyright 2009-2024, Arnaud Roques` and `Original Author:  Arnaud Roques`; complete MIT grant remains in existing licence/notices, including generated-edition historical comments. No GPL/rejected/gen/h/Smetana/EPL/ELK/untraced asset was imported. Artifact SHA remains locally observed `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`, not authenticated Maven identity.

- DataSourceImpl/Positionner2/Cell/PSystemSalt/Pyramid and six simple factories → `salt.rs`: raw terminators/escaping, IDs/cells/spans, nested grammar and ordered control dispatch.
- TableStrategy, ElementPyramid, AbstractElementText, ElementTextField, ElementButton, ElementRadioCheckbox → `salt.rs`/`saltscene.rs`: typed borders, measured constraints, padding and concrete shapes.
- Border/Tree/Tab/Menu/Droplist/Image/RetrieveFromDictonnary/Line/Scroll factories → `salt.rs`: explicit unsupported marker discovery only.
- Previously selected core/DiagramType → existing `source.rs`/preprocessor exact Salt-wrapper admission.

Exact independently authored reference-input copies (original artifacts unchanged):

| Fixture | Origin | SHA256 |
| --- | --- | --- |
| saltbase.puml | /tmp/ive-plantuml-native-oracles/salt-form.puml | `0fba629e3bf1d9ee00505dc3ed39a9d6cdc6b585b62c5301aabb44dae1a91e41` |
| saltwidgets.puml | /tmp/ive-plantuml-specialized-probes.Kf1afs/salt-widgets-span.puml | `515f278740900fe267eaae4ea2531c8b625f752972e872c60afc8d5c4f01343b` |
| saltliteral.puml | sibling salt-literal-delimiters.puml | `fa5e09a31c612c743b0986c1e87188195d370ff3781b2341314330fb2b7d4c47` |
| salt.puml | authored native viewer example | `6d4ad1eb7f1814f46a447246a3491b5ee4e7d9211758b2af4b78df01b48f52b5` |

All three exact references export through the final release CLI and parse in usvg. Native widgets viewBox329.38×223 versus original233×106; literal206.18×126.8 versus original140×58. These are semantic/native-layout evidence, not original geometry/pixel parity. No new JVM probes or changes to prior reference artifacts.

## RED/GREEN and final gates

Initial public10-test suite: meaningful RED8 failed at missing Salt wrapper/constant admission,2 negative controls passed. First implementation briefly had a token-assignment placement panic; correcting it made10/10 GREEN. Expanded syntax tests exposed separate tree-marker acceptance and over-rejection of plain caret labels (11/13), then13/13 GREEN. Interleaved style acceptance had a meaningful RED at `skinparam defaultFontSize30` inside a table, then GREEN; an initial accidentally valid wrapper negative was corrected before that RED and is not feature evidence. Callback cancellation and all15 public tests pass. Viewer example registration and exact-source allowlist each had actual unknown-example/unauthorized-source RED before GREEN. Six private model/geometry/bounds tests and real native paint strengthen coverage; not every later assertion had a separate RED cycle. All tests use should_ and given/when/then conventions.

Focused commands: `cargo test --locked --offline -p diagram-engine --test plantuml_salt`15/15; `cargo test --locked --offline -p diagram-engine --lib plantuml::salt`6/6; viewer Salt example selection1/1; actual native Salt paint1/1 at6/14/96px. Native font resolved Liberation Sans→Liberation Sans;9 actual painted runs at each size, button ink inside its border, field ink above/within underline extents and every label inside canvas. Worker-native lifetime coverage now includes the Salt viewer fixture and retains cache-release/zero-fallback assertions.

Final evidence directory `/tmp/ive-plantuml-3g.pDShPW/`. After the final product/test edit, from standalone `diagram-ports/`, all Cargo commands used `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`; scoped escalation bypassed only the known bwrap RTM_NEWADDR setup failure:

```sh
cargo test --locked --offline --workspace
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all --check
cargo build --locked --offline --release
cargo deny --config ../deny.toml check licenses sources
node tools/audit-plantuml.mjs
node tools/audit-merman.mjs
node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs
git diff --check
```

All exit0. **385 Rust tests passed**, zero failed/ignored, plus all seven separately reported hostile-worker scenarios (blocked stdin/stdout, stderr flood, oversized frame, early exit, memory, cancellation). Counts: engine-private82; compatibility11; Mermaid families37; Mermaid21; PlantUML33; constants31; control23; Gantt16; Salt15; styles17; timing21; tree/data18; viewer-private46; export5; native-lifetime3; worker6. Twelve Node audit tests pass. PlantUML audit174 source/JAR entries/27 fixtures/32 modules; Merman656/four patches unchanged. Deny licenses/sources OK. Logs: `tests-final.log`, `clippy-final.log`, `release-final.log`, `deny-final.log`, `audit-*-final.log`. Only inherited vendored Atomic::fetch_update deprecation remains. Self-review completed; no product edit after these gates.

## Genuine background capture and handoff

```sh
/mnt/data/Projects/ive/target/release/diagram-viewer --background --language plantuml --example plantuml-salt --capture /tmp/ive-plantuml-3g.pDShPW/salt.png
```

Exit0, real1360×880 GPUI readback inspected directly. SHA256 `8a05a20c7afdf1107643e4aa160bf17f54ef685ae168471b637febc72417907b`. `capture-salt.log` and status bar report **12 actual native measurements /0 deterministic fallbacks**. Account/alice field, checkbox/check/empty square, selected/unselected radio, pale rounded Cancel/Continue buttons, dark supported font color, bold Continue, title/footer, outer grid, nested outline and span-wide status label are all visible/contained. No containment-expansion warning or fixture-padding correction. Viewer self-quit; final anchored debug/release viewer process lookup empty. No IVE instance, xdotool, focus/window activation, real mouse or user-state/process interaction.

Residuals: bounded table/control-only Salt grammar, explicit rejected advanced widgets/borders/regions/resources, formatted-text minimum-width deviation, coarse missing-close diagnostic and existing baseline/unavailable-font/original-theme/pixel limitations. Timing keyword IDs and Gantt trailing-comment Minor cleanup ledger remain untouched, outside this increment. No known failing gate or new blocker. Independent frozen-head SPEC/QUALITY review is the controller's next gate; no later family started.

## FIX ROUND 1 — shared chrome grammar and inline T text

Fix base: `82fad891549ce2ec70b77b48dcdb8c0929ed41ef`. Frozen fix head: `f6fa5d9a11a558212afb6b910e3358e97374ef5d` (`Fix Salt chrome commands and inline text admission`). Tracked worktree clean. Read complete independent review `/tmp/ive-plantuml-salt-review.md` and controller-owned `/mnt/data/Projects/ive/progress.md`; only the controller edits that progress file. The review's Important and Minor findings were reproduced before production changes. The initial report's no-known-issue statement was historical: its self-review missed these two grammar boundaries. Both are repaired in this scoped wave; independent re-review is still required.

Important: extracted the existing chrome head/role/alignment/remainder parser into `style::region_command`. Actual shared `region` parsing and Salt's `style::is_preamble` recognition now consume that same grammar. Colon separators, optional left/center/right header/footer alignment, case and tabs therefore agree; there is no second Salt list of document-region keywords. Existing region payload validation, default/explicit alignment, styles, origin mapping and budgets are unchanged. Salt retains exact skinparam/style markers and the same descriptive late-preamble error. Actual field/button/control captions and ordinary noncommand keyword-like text remain literal.

Minor: removed only `T` from tokenizer opener-suffix consumption. Reinspected authorized `salt/DataSourceImpl.java:122` and `salt/factory/ElementFactoryTree.java:201-209`: ordinary `{Text}`, `{Textile}` and `{T#literal}` now tokenize as bare `{` followed by ordinary text. Existing first-token exact `T`, `T#`, `T+`, `T-`, `T!`, `T^` rejection remains; no tree feature is admitted. Direct delimiter escapes/PUA, nested-close terminators, IDs/spans/layout, resource denial and all limits remain covered.

Exactly three committed files changed: `engine/src/plantuml/salt.rs`, `engine/src/plantuml/style.rs`, `engine/tests/plantuml_salt.rs`. No new translated source, provenance/fixture/dependency/asset, model or runtime capability. Existing source inventory and notices remain174/27/32 with all26 Salt selections unchanged. No unrelated timing/Gantt cleanup, new family, subagent or root IVE edit.

Meaningful TDD evidence:

- `cargo test --locked --offline -p diagram-engine --test plantuml_salt`: first expanded18-test run **16 passed/2 failed** (exit101). The command matrix showed24 incorrectly accepted late forms; all three ordinary inline T labels incorrectly failed at unsupported `{T`. Literal field/button/checkbox/radio and noncommand controls already passed.
- Shared chrome parser correction alone: **17 passed/1 failed** (exit101). Every late chrome form rejected; only the independent T-token regression remained red.
- Removing only the T suffix: **18/18 GREEN**. Final matrix covers25 prefix-valid commands: title/legend/header/footer colon forms, spaced colon, mixed case, and all left/center/right header/footer combinations with space/colon/spaced-colon separators plus a mixed-case tabbed header. Every form is accepted before the table and rejected inside it. Exact tree markers continue to reject. Neighboring literals retain distinct semantic widget kinds.
- Final focused command `cargo test --locked --offline -p diagram-engine --test plantuml_salt --test plantuml_styles --test plantuml_constants`: **18+17+31 pass**, exit0, after the last test/code edit. Log: `/tmp/ive-plantuml-3g-fix1.lyhn8I/focused.log`.

Fresh final gates after the last code/test edit used the same locked/offline workspace test, strict Clippy, fmt, release, deny and source/Node-audit commands listed above, all with the common Cargo target directory and scoped bwrap-failure escalation. **388 Rust tests passed, zero failed/ignored**, plus all seven separately reported actual worker-fault scenarios. Only Salt public count changed15→18; other suite counts are unchanged. Twelve Node tests pass; PlantUML audit174 selected source/JAR matches/27 fixtures/32 modules, Merman656/four patches, licenses/sources OK. Inherited vendor Atomic deprecation only. `git diff --check` and self-review clean; no product edit after these gates. Evidence directory `/tmp/ive-plantuml-3g-fix1.lyhn8I/`: `tests-final.log`, `clippy-final.log`, `release-final.log`, `deny.log`, `audit-plantuml.log`, `audit-merman.log`, `audit-tests.log`.

Final release checks of the exact reviewer inputs:

- `/tmp/ive-salt-review-late-chrome.puml` now exits1: `PlantUML at line 4, column 1: "Salt styles/chrome must precede the table: \"title: Late title\""`.
- `/tmp/ive-salt-review-tword.puml` now exits0 and exports `tword.svg` in the fix evidence directory.

The unchanged registered example was recaptured with the repaired release binary using `--background --language plantuml --example plantuml-salt --capture /tmp/ive-plantuml-3g-fix1.lyhn8I/salt.png`. Exit0; real1360×880 image; **12 actual native measurements/0 fallback**, recorded in `capture-salt.log`. `cmp` against the original capture exits0; both SHA256 are **`8a05a20c7afdf1107643e4aa160bf17f54ef685ae168471b637febc72417907b`**, proving identical pixels. Full-suite actual native paint at6/14/96px remains green, including9 painted runs and button/field/canvas ink containment. No fixture padding or containment expansion. Viewer self-quit; final anchored viewer-process lookup empty. No IVE/user-state/focus/mouse interaction.

Both reviewed findings are addressed in code/tests; no known failing gate or new blocker. Prior documented Salt/fidelity/diagnostic limitations remain unchanged. Controller-arranged scoped re-review gates approval; nwdiag remains held.
