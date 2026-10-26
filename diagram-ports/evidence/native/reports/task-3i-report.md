# Task 3I — native files, packetdiag, board and HCL

Status: implemented, verified and frozen for controller-arranged independent SPEC/QUALITY review. This is four bounded native family subsets, not complete PlantUML compatibility or original pixel parity.

- Explicitly released, independently approved base: `85887fa1989aa9f21834b8118cddd5735d1dac0a`.
- Frozen head: `1bf1f46c762d9ff4a5082aa688c6bff3dfaf2c7a` — `Add bounded native files packetdiag board and HCL models`.
- Worktree `/mnt/data/Projects/ive/.worktree/diagram-ports`, branch `diagram-ports`; standalone product `diagram-ports/`. Exactly29 scoped files committed; tracked worktree clean afterward. No worktree/branch creation, main IVE edits, dependency/lockfile/vendor change, merge, push or publication. No subagents or self-dispatched reviewers.
- Read canonical `/mnt/data/Projects/ive/progress.md` first, released [brief](task-3i-brief.md), [global constraints](native-global-constraints.md), native plan Task8 context, complete inventory semantics/source ledger, and shared 3H/3C/3D/3D2/3G reports including fixes. Actual translated authorized sources were inspected personally. TDD/verification instructions guided implementation. Controller owns both progress files; neither was edited by this implementer.
- Controller recovery interrupted the session after initial integration; existing WIP/context were reconciled without restarting or discarding work. Required current7.0.1 TDD/verification instructions were refreshed. The interrupted native test was rerun; no interrupted result is counted as verification.

## Shared contracts and isolation

Public render/options/result/control/text-measurement APIs, native worker protocol/service/watchdog, caches and SVG terminal admission remain unchanged. Engine remains GPUI-free, synchronous and caller-memory-only: no runtime file/network/environment/clock/locale/process/JVM/JS/browser/dot capability, no imported asset and no added crate.

Exact lowercase wrappers add `Wrapper::Files`, `Packet`, `Board`, `Hcl`; filenames/aliases and mismatched/multiple wrappers reject. The common preprocessor recognizes their exact wrappers as raw family input, retaining apostrophes, significant file-component spaces, HCL hashes/quotes and physical rows before family grammar. Existing eager literal expansion/origins/resource capability gates still apply. Genuine generated directives/wrappers gain no privilege. All prior family regressions remain green.

Files/packet/board reuse existing finite prefix fonts/styles/chrome and shared Creole below their family models. HCL owns its whole body; shared preamble preparation skips only initial blank/hash-comment rows for correct literal-definition ordering, and does not consume command-named HCL blocks. HCL style/chrome prefixes reject through its grammar. Prefix constants remain admitted in all four families; a dedicated regression caught/fixed the initial HCL blank-definition-row misclassification. No shared JSON/YAML, UML or Salt preamble behavior changed.

Existing `Context::data_label` preserves literal files names/HCL values and the explicit Arial/Liberation Sans/DejaVu Sans/Noto Sans/sans-serif family chain. File notes, packet labels and board cards use `tree_label` for the already-admitted finite Creole and matching measure/paint families. Real unsupported resource labels reject there; resource-looking file names/HCL strings remain literal data. No rich-label parser or font cache was duplicated.

## Files model, grammar and layout

`files::parse(&[Line], &mut Budget) -> Files` freezes an ordered `Vec<Entry>` with numeric stable IDs, `Kind::{Root,Folder,File,Note}`, parent/depth and ordered child IDs. Root0 is hidden. Interior folder lookup reuses only same-name sibling folders, retaining first insertion; final files always append, including duplicates and file/folder name collisions. Paths are stored as per-component strings, never expanded full-path copies. Leading `/` is syntax, with no filesystem access or normalization. Dot/dotdot and Unicode/XML/resource-looking names are literal; significant component spaces survive. A path must begin in column0; unusual empty components, `/`, `//` and `/a//b` reject. Trailing `/` declares a folder, including an empty folder.

Exact `<note>` starts a physical multiline note; closing `</note>` permits surrounding whitespace. Notes retain blank rows/significant text, attach as siblings in the last file's parent, or at root after a folder-only path. Consecutive notes preserve that anchor. Unknown nonpath rows, note suffixes/openers and unterminated notes fail explicitly rather than upstream's ignore/to-EOF behavior.

`filesscene::layout` uses iterative DFS preorder,21-unit depth indentation and2-unit row gaps. Every row advances by the complete measured label/note height. Original native folder/file polygons and folded-corner notes replace upstream emoji assets; no Twemoji is imported. No connector edges are invented. Scene IDs `f{id}` retain parent owner `f{parent}` including hidden root0. Notes have real containing folded geometry; file/folder labels stay beside their symbols. Direct tests assert ordered IDs, kind separation, sibling anchors, leading blank rows, depth/lookup quota before retention, preorder indentation and full multiline clearance.

## Packet model, grammar and layout

`packet::parse` freezes dedicated `Config`, ordered inclusive `Field {start,end,label,height}` and generated `Fragment {field,row,offset,width}` vectors. IDs are stable field indices; fragments never copy field labels/models. Width is `end-start+1`. Explicit noncontiguous source ranges are laid out by declaration widths without visual gaps; `*` starts after the preceding declared end, or zero initially.

Admitted body is direct field/config rows or exactly one canonical `packetdiag {`/`packetdiag{` opener with matching `}`. Standalone braces, repeated/unclosed braces and substantive trailing input reject. Numeric field forms `N`/`N:`/`A-B`/`A-B:` require whitespace before the label; auto form is `*` without a colon. Optional suffix attributes consume the complete comma-separated grammar. Single-number/auto fields admit positive `len` and `height`; explicit ranges admit height and reject len. Unknown/duplicate/malformed attributes, rotation, descending ranges, zero values and endpoint overflow reject. Endpoints/attributes have at most seven decimal digits; final end cannot exceed9,999,999. Bracket-looking label text is reserved for the attribute suffix, a documented narrow syntax boundary.

Config directives use `key=value` with one optional semicolon: positive `colwidth`1–999/default16; `node_height`1–999/default32; positive `scale_interval`; exact lowercase `scale_direction=ltr|rtl`; `same_height=true|false`/defaultfalse. Every admitted property affects actual painting. Final interval is capped at final configured colwidth, and default is max(1,colwidth/2), safely handling colwidth1. Configuration follows source assignment order and may precede/follow fields; final settings drive layout. Native keeps the configured scale width even for a short first row, unlike upstream's final-width reduction.

Fragmentation conserves widths: colwidth8 with0–2 then3–22 produces Payload5/8/7 rather than upstream's erroneous5/8/8. `packetscene::layout` derives one common bit width from full measured labels/fragment widths and tick labels, so all rectangles are truly proportional without UTF8 truncation or ellipsis. Full semantic labels remain intact. This deliberately differs from upstream abbreviation and font-size×3 scale. RTL reverses both cells and indices. Ticks have genuine aligned short/full strokes and numbered interval labels along the top scale; repeated per-row scale paint is not claimed.

Requested cell heights use the positive vertical unit plus a minimum for measured text. `same_height=true` promotes actual cells to the row maximum. Variable-height cells retain individual heights, but every next row advances by the maximum, correcting upstream last-cell-height overlap. Every fragment remains a full rectangle; no continuation-marker or rotation paint is claimed. Metadata: semantic `p{id}` fields retain start/end/full label; `pf{id}` fragments own the field ID and record `row:offset:width`; `pt{index}` ticks retain their actual numeric index. Direct tests assert exact5/8/7 identities,4:8 proportional geometry, shared tick boundaries, RTL, interval labels, equal/variable heights and row clearance.

## Board model, grammar and layout

`board::parse` freezes a dedicated ordered card/tree arena and activity-root IDs. Each card stores `Parent::{Root,Node(id)}`, stage, name and ordered child IDs. Zero-plus lines start new activities; positive plus counts choose stage rows. The current activity's ancestor chain pops until the nearest previous lower stage, allowing jumps and retaining empty rows. Orphan releases fail. Traversal is iterative and charged; no unchecked recursive parent/computeX walks.

Plain punctuation captions remain valid. Explicit shared late preambles, denied preprocessors/wrappers and recognized substantive unsupported commands such as page/sprite/hide/show/scale/url reject rather than becoming root text. Safe prefix styles/chrome/constants use common services. Empty captions, oversized names and excessive stage depth reject before card retention.

`boardscene::layout` assigns DFS columns: first child keeps the parent's column; each later sibling begins after the preceding subtree. The next activity begins after the entire earlier branch width. Shared measured card width/height have150×70 minima and20-unit pitch gaps. Actual dashed boundaries span all columns between stage rows. Every root/card paints exactly once, correcting upstream duplicate root rendering. No arrows or Kanban lanes are invented. `b{id}` owns its actual parent; `bs{row}` identifies each stage boundary. Exact seven-card tests assert stages0/1/2/1/3/0/1, parent IDs and x positions24/24/24/194/194/364/364, plus measured long-label containment.

## HCL model, grammar and ordered-data ownership

`hcl::parse -> Document { blocks: Vec<Block> }` freezes dedicated `Block {name,fields}`, `Field {name,value}` and `HclValue::{Object,Array,String}`. It is PlantUML's notation, not a generic HCL/Terraform evaluator. `hcl::lower` transfers ownership into existing ordered `data::Value` without cloning child ASTs. Single block unwraps to its object body; multiple distinct blocks retain source order and textual names with quoted labels. Duplicate topblock names explicitly reject instead of implementing upstream first-position replacement. Duplicate fields append as separate rows, preserving source order.

Fields admit `=` or `:`, quoted/simple scalar strings, nested objects and arrays of quoted strings/objects. Bare numbers/booleans/null are strings, never JSON numeric/bool/null values. Empty arrays/objects retain their identities and compartments. Bare scalar tokens admit Unicode alphanumeric characters plus `_ . -`; other bare expression/call punctuation rejects. Object fields use whitespace adjacency, not commas. Arrays require commas (a final comma is allowed). Bare array tokens, nested arrays, calls, expressions, interpolation `${...}`/`%{...}`, heredocs, unclosed/unbalanced containers and malformed/physical-multiline quotes reject without dropping data.

Whole hash-comment rows are skipped; hashes inside strings remain data. Backslash consumes the next quoted character, matching the inspected PlantUML rule (`\\n` becomes `n`), rather than JSON decoding. Escaped physical newline/trailing escape rejects. Physical lines remain separators instead of upstream concatenation that can merge bare tokens. String/resource-looking text is inert and XML escaped. Command-named blocks such as `title`/`footer` remain ordinary block data; shared style/chrome prefix support is explicitly deferred.

Existing `datascene::layout` creates genuine ordered compartments `d{id}`, scalar/container rows `d{id}r{index}` and links `de{id}r{index}` from the corresponding row center. Container ownership points back to that row; scalar placeholders do not compress child-row positions. Tests assert zones/tags links from actual rows5/6, ordered duplicate/string semantics, empty containers and nested array/object ownership. Synthetic multiblock root depth is charged during lowering; exact depth2 accepts one scalar block but rejects two blocks until depth3.

## Bounds, cancellation and native verification

Defaults remain source256KiB/SVG16MiB/model10,000/depth64/work5,000,000/cooperative10s; independent worker hard10s/2GiB Unix,16KiB labels,4096 measurement queries,1,000,000 coordinate magnitude and existing raster caps unchanged. One Budget/Context/session is retained across parsing/lowering/layout/chrome/SVG. Scans, folder/block comparisons, retained strings, model nodes, packet attributes/fragments/ticks, stage boundaries, layout visits and metadata use shared charges. Label bounds precede board/packet model strings and files/HCL string appends. File total path depth precedes construction; wide sibling search charges before each compare. HCL value depth/items precede content scanning; lowering checks added synthetic root depth. No dense stage×column grid or per-ancestor fullpath cloning.

Cancellation checkpoints remain inside growing loops and around genuine host callbacks. The four-family callback regression cancels during the first request and observes exactly one callback, with an explicit cancellation error. Tiny caller model limits and pre-cancelled sources reject all four families. Huge packet requests stop at shared fragment-item quotas before expansion; colwidth is bounded before tick generation. Final finite geometry/containing Scene checks, bounded SVG writing and mandatory terminal resvg-safe/resource/XML admission remain unchanged. Public success cases are deterministic and parsed by usvg.

Actual installed-font paint test uses the viewer's real native text system and actual SVG resolver, asserts positive inspected text counts, zero native fallbacks and ink inside canvas/owning card/fragment/note/data compartment paths. Files/packet/board run6/14/96px; HCL14px (shared HCL style prefixes are deliberately unsupported). Recorded counts: files13 painted runs/12 actual host calls, packet11/9, board8/8, HCL25/21; all zero fallback. Native-worker lifetime coverage includes all four new examples and releases each per-render cache. No fixture padding or canvas expansion was used to hide overflow.

## Authorized provenance and exact reference inputs

Exactly22 new selected files, personally inspected under the sole authorized `/mnt/data/Projects/plantuml-mit/src`, are recorded with actual source SHA256, native destinations, translated behavior and verbatim credits in `diagram-ports/provenance/plantuml-mit-1.2026.9.json`. Controller independently matched every addition against extracted bytes AND exact source-JAR members/hashes/dated copyright/author/contributor headers/MIT grants. Selection did not change afterward.

| Authorized source under net/sourceforge/plantuml | Rust destinations |
| --- | --- |
| filesdiagram/FEntry, FilesDiagram, FilesListing | files.rs / filesscene.rs |
| board/Activity, BNode, BoardDiagram, CommandBoardPlus | board.rs / boardscene.rs |
| board/CardBox, PostIt | boardscene.rs |
| hcl/HclParser, HclSource | hcl.rs |
| packetdiag/PacketDiagram | packet.rs / packetscene.rs |
| packetdiag/PacketBlock, PacketIndicator | packetscene.rs |
| packetdiag/command/CommandNumRange, CommandColWidth, CommandNodeHeight, CommandScaleInterval | packet.rs |
| packetdiag/command/CommandScaleDirection, CommandSameHeight | packet.rs / packetscene.rs |
| packetdiag/command/CommandPacketDiagStart, CommandPacketDiagEnd | packet.rs |
| Previously selected json/JsonObject and core/DiagramType | HCL duplicate rows / exact shared wrapper dispatch traces extended |

All new files retain generated `(C) Copyright 2009-2024, Arnaud Roques`. Packet files additionally retain historical `(C) Copyright 2009-2026, Arnaud Roques`; their Original Author is kolulu23, except CommandSameHeight credits Duy Nguyen. PacketDiagram/CommandNumRange/CommandScaleDirection credit `Contribution: The-Lum`. Both sets of packet years and exact spellings are retained in the manifest/notices. Ordered HCL reuses Copyright(c)2016 EclipseSource minimal-json behavior and its complete existing MIT grant. Other selected files credit Original Author Arnaud Roques. Historical comment headers below generated MIT grants remain exact, without cleaning/relabeling. No GPL/rejected-candidate/gen/h/Smetana/EPL/ELK/untraced asset was imported.

Local source artifact SHA remains `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`, not externally authenticated Maven identity. Full grants remain in LICENSE-PlantUML-MIT.txt and THIRD-PARTY-NOTICES. Final audit:206 selected sources/JAR matches,39 fixtures,41 modules. Allowlist admits only22 exact paths; unselected FilesDiagramFactory/BArray/OtherParser/PacketDiagramFactory remain denied. Existing Mermaid656 imported files/four patches unchanged.

Four exact source copies byte-compare to untouched original reference artifacts; four further native examples are independently authored. No new JVM/original-renderer probe was run or claimed.

| Fixture / original source | Exact source SHA256 | Original viewBox | Native deterministic viewBox |
| --- | --- | --- | --- |
| filesbase / files-tree | `49292373d102be17bc4b8cdca87e8145810714c12520b2f49a3dcea21a260a0c` |0 0 144 120|0 0 164.26 133.7|
| packetbase / packetdiag-ranges | `ed4d574921091eaf20c41f79fb22f97ff647e24388a8472d2c50f13b51c45669` |0 0 735 150|0 0 447.68 178.20000000000002|
| boardbase / board-columns-cards | `3479cbd3c10b0e2df33f6fa27c3e9fd1629d48acb40370b0a579f5ba3df8080c` |0 0 372 193|0 0 368 208|
| hclbase / hcl-nested-data | `40fb9ae3934394d2712f93624b8957bfd240d7af2098ae6c5223fa8299ecce39` |0 0 234 109|0 0 302.22 178.20000000000002|

All four final release exports succeed with PATH empty and DISPLAY/WAYLAND_DISPLAY unset. Outputs in evidence directory: files-native.svg2160bytes/SHA`71cabca938004ecfc3068b62575bc9ba8d1a3d43031d6d5f5151a03f34d4da30`; packet-native.svg11359bytes/`286d93d70540bafceb59adad41eec0e99f28dcea8da0d4f5f99eac2c633f77ba`; board-native.svg2252bytes/`da801272997f324fb888324ed90fbe97bab2def8f1f60c3214ab3c2606ee453b`; hcl-native.svg6010bytes/`477c400fc7b506a1ccc151d55d2c43b381db98794b4ea226c432316b9063859b`. Dimensions/symbol decomposition differ; these are semantic/native-layout comparisons, not pixels.

## Meaningful RED/GREEN and complete gates

All test names use should_ and given/when/then. Initial public12 cases: **8 meaningful missing-wrapper/behavior failures,4 rejection controls passing**, before implementation. After one explicit-lifetime compile correction,11/12 passed; the remaining test used the wrong existing metadata name `data-string`, corrected to actual `data-row-string`, then12/12. That test correction is not claimed as a feature RED.

Additional actual RED observations before corresponding fixes:

- Leading blank note rows were lost; public SVG metadata normalization expectation was corrected to XML's spaces and direct model test verifies the genuine newline string. Correct row accounting made it green.
- HCL blocks named title/footer were consumed as chrome. Dedicated HCL body ownership fixed this; escaped physical newlines also reject.
- Missing viewer registration returned unknown plantuml-files; registering four honest examples made the viewer selection test pass.
- Exact source admission rejected FEntry before the narrow22-path allowlist extension; final Node guards pass and continue denying unselected siblings.
- File-note/board/packet shared display markup initially painted literally; shared tree-label path made supported bold genuine and resource labels reject while file/HCL data stayed literal.
- Board/packet labels16385 bytes were retained by their parsers; pre-retention guards fixed both private RED tests.
- Synthetic HCL multiblock root was omitted from depth; exact-limit public regression failed before depth-aware ownership lowering.
- Repeated numeric colon `0:::` was admitted; single optional delimiter consumption fixed it and repeated config semicolons/attribute suffixes reject.
- HCL prefix definitions were misclassified at their retained blank row; skip-only blank/hash prefix preparation repaired exact shared origin ordering.
- File component trailing spaces were silently trimmed; raw column0 path parsing now preserves them. Unsupported `*:` auto-colon spelling rejects.

Additional direct identity/geometry/preallocation, real native paint, fixture, cancellation, variable-height and wide-lookup controls passed against working implementation; not every later assertion had an independent RED. Final coverage:22 public inventory tests,13 new private engine tests, two new viewer-private tests plus four added native-worker lifetime sources. No old assertion was weakened or unrelated timing/Gantt/nwdiag Minor repaired.

Evidence directory `/tmp/ive-plantuml-3i.SWV6Rh/`. Final gates after the last behavior/test/documentation edit:

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline --workspace
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo clippy --locked --offline --all-targets -- -D warnings
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo fmt --all --check
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo build --locked --offline --release
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo deny --locked --offline --config ../deny.toml check licenses sources
node tools/audit-plantuml.mjs
node tools/audit-merman.mjs
node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs
git diff --check
git diff --cached --check
```

All exit0. **450 ordinary Rust tests passed, zero failed/ignored**, plus seven separately reported actual worker-fault scenarios (blocked stdin/stdout, stderr flood, oversized frame, early exit, memory, cancellation). Breakdown: engine-private102; compatibility11; Mermaid families37; Mermaid21; PlantUML33; constants31; control23; Gantt16; inventory22; nwdiag16; Salt18; styles17; timing21; tree/data18; viewer-private50; export5; native-lifetime3; worker6. Fourteen Node guards pass; both source audits and deny pass. Logs: tests-final.log, clippy-final.log, fmt-final.log, release-final.log, deny-final.log, audit-plantuml-final.log, audit-merman-final.log, audit-tests-final.log. Inherited audited vendor Atomic::fetch_update deprecation remains; strict product linting is clean. Early local Clippy boolean-comparison diagnostics were fixed before final gates. Scoped escalation bypassed only known bwrap RTM_NEWADDR initialization failure; no network/approval rejection was involved.

Self-reviewed the complete staged29-file diff, including new files, before commit: raw ownership/complete grammar, stable identities, row/fragment/branch topology, HCL ownership transfer, preallocation/cancellation, actual font/shape containment, precise sources/notices/fixtures and runtime scope. No product edit followed final gates/captures. Commit leaves a clean tracked tree.

## Genuine final viewer captures

Each command used the actual release executable, `--background --language plantuml --example plantuml-FAMILY --capture /tmp/ive-plantuml-3i.SWV6Rh/FAMILY.png`. All exit0, real1360×880 GPUI app frames, opened and visually inspected directly. Status bars and capture-FAMILY.log match actual native counts; all zero fallback and no containment-expansion warning.

| Capture | Native/fallback | Inspection | SHA256 |
| --- | --- | --- | --- |
| files.png |12/0|Native icons, nested app/src preorder, duplicate main.rs, sibling multiline note, empty folder and separate root note; all contained|`7c297f03c91c27bc91476f7a5daf49323f668550dd40a14481cf98ef03a562e1`|
| packetdiag.png |9/0|Type3, Payload5/8/7 across three rows, final one-bit Flags and aligned0/2/4/6/8 ticks; full labels|`f5955225468efbee1243c445f9d1811d9e76f70f579f7eb2790278c36da76d47`|
| board.png |8/0|Checkout/Draft/Validate column0, Submit/Ship column1 with stage2 gap, Audit/Record column2, three dashed boundaries, each root once|`c1445f0f5a1dfea0c5f74bc2107cecfbf634c4447d0e35a263ad6623c3a732a9`|
| hcl.png |21/0|String3/true rows, zones/tags/endpoints owning compartments and two nested endpoint objects; all data text contained|`136d9d61188ed9ce53d9aff0fc8d6d8961aa47e167ac2a732e74c6635d800fca`|

All four viewer instances self-quit. Final anchored pgrep for actual debug/release viewer executable paths found no processes (expected exit1). No IVE instance, xdotool, focus activation, mouse movement or user-state/process interaction.

## Residuals and next gate

Explicit narrower grammar/deviations above remain: files column0/strict components/native symbols; canonical packet body/attribute rules and changed full-label scale/row clearance; simplified measured board shapes/pitches; HCL rejected duplicate topblocks/styles/chrome/evaluation and newline-separator policy. Shared baseline approximation, unavailable-font fallback, coarse source columns and general data-route collision/original-theme/pixel-parity limits remain. HCL native font-size variants are not claimed beyond default14px. Some parser/layout/test functions exceed preferred20 statements, as existing family implementations do; responsibilities remain separated by typed grammar/model/layout boundaries.

No known failing gate or unresolved new blocker. This report freezes implementation evidence, not independent approval. Controller-arranged fresh SPEC/QUALITY review gates Task3I approval and any Chen/icon-free ArchiMate release. All later families and unrelated deferred Minors remain untouched.

## FIX ROUND 1 — I1 physical HCL header statement boundaries

Fix base: `1bf1f46c762d9ff4a5082aa688c6bff3dfaf2c7a`. Frozen fix head: `3231bb195bbf183e5e961135f9889a9e872bd611` (`Enforce physical statement boundaries in HCL block headers`). Exactly two product files changed: `engine/src/plantuml/hcl.rs` and `engine/tests/plantuml_inventory.rs`,82 inserted lines. Tracked worktree clean after commit. No controller progress edits, later-family changes, subagents or self-dispatched reviewers.

Read full independent `/tmp/ive-plantuml-inventory-3i-review.md` before any repair action. It returned SPEC/QUALITY CHANGES REQUIRED with one Important: **I1 — HCL block-name accumulation consumes unsupported prefix statements across physical lines**. The original self-review missed this case; the initial report's no-known-blocker statement above is historical and superseded by that finding. Systematic-debugging, receiving-code-review and TDD governed this scoped repair.

### Root cause and exact repair

`hcl::parse` previously accumulated name tokens until `{` through `Parser::peek`, whose shared whitespace handling correctly skips physical newlines for values/containers. Thus `title Unsupported chrome` followed by `service { x=1 }` became one name `title Unsupported chrome service`; the intentional single-block unwrapping then removed it and exported only x/1. The style-only and combined inputs followed the same path. After an existing block, the same bug combined a separate unsupported statement into a later block's displayed name. This was syntax admission/data loss, without runtime evaluation or resource execution.

The parser now saves each topblock header's starting physical line. Before consuming another name token, it checks that `peek()` has not crossed into another physical line; otherwise it returns a located error naming the complete offending header and the next physical line. This check occurs before scanning/allocating that next token or lowering the document. No keyword blacklist, preamble stripper, label workaround or alternate evaluator was introduced.

Opening `{` may still follow the header on a later line (including after blank/hash-comment rows), because the header loop stops when the brace is found. Genuine `title "named block"` and `footer` names remain valid. Shared `peek`, nested body/value parsing, arrays/objects, literal constants, ownership lowering, budgets, rendering and every other family are unchanged. Name tokens themselves must stay within one physical header line; this is the documented statement-separator policy now enforced at its cause. Physical newlines remain allowed between field names/separators/values and inside arrays/objects.

### Meaningful RED/GREEN

Evidence directory `/tmp/ive-plantuml-3i-fix1.HBrstU/`.

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --test plantuml_inventory
```

Before production edits, the first test-only run had22 passed/5 failed (`red-initial.log`): the four intended I1 cases incorrectly returned rendered diagrams, and the new valid nested control had a hand-count error expecting six data links instead of the actual five. That expectation was corrected to the independently derived five edges (two topblock links, nested and items links, one object-array-item link); it is not counted as a product failure. A quoted late-header test value was simplified to ordinary `service billing`, retaining the grammar risk without mixing in diagnostic quote escaping.

The corrected unchanged-production baseline was **23 passed/4 failed**, exit101 (`red.log`). Independent failing tests:

- `should_reject_hcl_title_statement_before_a_separate_block`: exact title-only reproduction incorrectly returned `Ok(RenderedDiagram)` with x/1.
- `should_reject_hcl_style_statement_before_a_separate_block`: exact style-only reproduction incorrectly returned x/1 at14px.
- `should_reject_hcl_combined_prefix_statements_without_lowering_them_away`: exact combined reproduction incorrectly returned x/1.
- `should_reject_hcl_header_token_continuation_after_any_complete_block`: a later title statement was combined into the next displayed block name; controls also cover ordinary incomplete headers and `service billing`, so the fix cannot rely on reserved-word rejection.

The valid `should_keep_hcl_command_names_split_opening_braces_and_nested_value_newlines` already passed before the fix: quoted title/footer names, braces on following lines, comment before brace, field values on later lines, nested object and array/object values, ordered scalar row names and five real compartment links. Existing exact command-name and escaped-physical-newline controls also remained green.

After the ten-line parser guard, the same focused suite was **27 passed/0 failed**, exit0 (`green.log`). All four actual I1 regressions now pass with originating line2 (or line3 after a completed block), complete offending header text and physical-line diagnostics. No test was weakened to generic failure or removed.

### Fresh final checks and exact reviewer inputs

After the last product/test edit, final full workspace tests, strict Clippy, fmt, release, deny and the same source/Node-audit commands listed above all exited0, using the shared Cargo target and locked/offline flags where supported. **455 ordinary Rust tests passed**, zero failed/ignored, plus all seven separately counted actual worker-fault scenarios. Only public inventory count changed22→27; all other suite counts remain unchanged. Fourteen Node guards pass; PlantUML206/39/41, Merman656/four and deny licenses/sources remain green. Logs: `tests-final.log`, `clippy-final.log`, `fmt-final.log`, `release-final.log`, `deny-final.log`, `audit-plantuml-final.log`, `audit-merman-final.log`, `audit-tests-final.log`. Inherited vendor Atomic deprecation only. Source selection, source/fixture hashes, notices, dependencies, APIs and scene/font contracts are unchanged; no new manual source/JAR matching was needed. Scoped escalation addressed only the existing bwrap initialization failure.

Final release CLI checks used the exact unchanged `/tmp/ive-3i-review-hcl-{title,style,prefix,command-names}.puml` inputs, writing outputs only into the new fix evidence directory:

- title: exit1, `PlantUML at line 2, column 1: HCL block header "title Unsupported chrome" crosses a physical line before '{'; next token is at line 3`.
- style: exit1, same located error with header `"skinparam defaultFontSize 96"`.
- combined prefix: exit1, same line2 title-header error before any lowering.
- command-names: exit0, `command-names.svg` retains genuine title/footer named blocks and x/y compartments.

Original reviewer source/SVG artifacts were not modified. Complete two-file diff was self-reviewed for cause-level grammar enforcement, located diagnostics, brace/newline controls, preservation of nested values and unchanged budgets/ownership/runtime boundaries. Whitespace/staged checks pass; no product change followed final gates.

### HCL-only native capture and residuals

Refreshed only the affected family's capture with the final release binary:

```sh
/mnt/data/Projects/ive/target/release/diagram-viewer --background --language plantuml --example plantuml-hcl --capture /tmp/ive-plantuml-3i-fix1.HBrstU/hcl.png
```

Exit0, actual1360×880 GPUI frame directly inspected: nested zones/tags/endpoints and scalar strings retain the prior contained geometry. `capture-hcl.log` and status bar show **21 actual native measurements/0 deterministic fallbacks**. SHA256 **`136d9d61188ed9ce53d9aff0fc8d6d8961aa47e167ac2a732e74c6635d800fca`** exactly matches the original HCL capture; `cmp` exits0, proving unchanged pixels. No containment-expansion warning. Viewer self-quit; anchored debug/release viewer process lookup found none. Unaffected files/packet/board captures were not rerun or falsely relabeled fresh.

I1 is addressed in code/tests; fresh controller-arranged scoped re-review is still required. Previously documented grammar/fidelity/diagnostic residuals remain, including unsupported HCL styles/chrome/evaluation and multiline quoted strings. Split header-name tokens now explicitly reject while split opening braces and valid nested/body/value newlines remain supported. No later family is released by this fix report.
