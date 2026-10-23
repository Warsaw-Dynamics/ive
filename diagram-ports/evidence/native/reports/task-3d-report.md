# Task 3D report — shared measured styles, chrome and plain Creole

## Frozen checkpoint

- Released base: `6ec10226db28f084d627aad53276b51794009228`.
- Product HEAD: `0fe389716232d3403940f0b145f15de0a58a4bfe` — `Add native PlantUML measured styles and document chrome`. All25 committed files are inside the standalone product; tracked worktree is clean after commit.
- Worktree `/mnt/data/Projects/ive/.worktree/diagram-ports`, branch `diagram-ports`; all product changes are in its independent `diagram-ports/` project. This ignored report is a controller handoff artifact, not a root IVE product edit.
- Implemented and locally verified finite style/text stage; awaiting controller-arranged independent SPEC and QUALITY review. This is not full PlantUML parity. No next-task preprocessing was implemented.
- One implementer, no subagents/helpers/reviewers dispatched. Used apply_patch, TDD, systematic investigation of observed failures, and fresh verification. No merge/push, root IVE changes, added dependencies, lockfile changes, font/assets import, IVE instance, xdotool or focus activation.

## Files, APIs and responsibilities

Public RenderOptions/results/measurement/control APIs, worker protocol and watchdog remain unchanged. The engine stays in-memory and GPUI-free, without new runtime IO/process/environment/font-loading access. No changes to Mermaid/vendor implementations.

| Files | Contract/change |
| --- | --- |
| `engine/src/plantuml/style.rs` (new) | Typed Color, Font, Role, Alignment, Region and Preamble; finite prefix admission and full-consumption stylesheet parser; effective five-role fonts resolved in authored property order. |
| `engine/src/plantuml/creole.rs` (new) | Bounded tokens and styled runs; one-pass escapes, linear symbolic activation, strict active-span nesting and supported tag matching, resources/unsupported markup rejection. |
| `engine/src/plantuml/text.rs` | Context keeps one effective font array/cache/query counter for the whole render. Label now shares `Arc<[MeasuredLine]>`; each line has MeasuredRun text/style/advance, height and baseline. Retains label/styled_label/tree_label/data_label entry points; adds with_fonts/region_label and bounded append_label. |
| `engine/src/plantuml/chrome.rs` (new) | Measures real document regions around a completed family Scene, translates every shape, retains IDs/ownership, checks scene bounds through the existing push path. |
| `engine/src/plantuml/mod.rs` | Source preparation → prefix style/chrome preparation → family parse/layout → shared chrome composition → serializer. One Context and aggregate query cap, including chrome. |
| `scene.rs`, `svg.rs` | Typed background and measured-run serialization. Explicit identical font family/size/weight/slant and text segmentation; color, underline/strike and xml:space preserve. Raw labels never become attributes or markup. |
| `lexer.rs` | Quoted strings preserve supported escape spelling until one shared decoder; quoted label bound checked during accumulation. |
| `activity.rs`, `sequence.rs` | Raw multiline label accumulation bounded before append. Sequence additionally receives narrow tested paint-layer correction discovered in the styled capture. |
| `graph.rs`, `stategraph.rs` | Adapt internal label/run access and geometry test constructors without changing family models. |
| `yaml.rs` | Exposes existing quote-aware colon finder internally for first-payload-row precedence; no new YAML parsing semantics. |
| `engine/tests/plantuml_styles.rs` (new) | 16 public contract/admission/style/geometry/limits/cancellation tests. Existing private modules add measured-run/budget/layer tests. |
| `engine/tests/plantuml.rs` | Replaces only negatives for now-supported safe styling with substantive unsupported/resource negatives. Existing unpaired punctuation/empty-slot regression is retained unchanged. |
| `viewer/src/svg/tests.rs` | Actual installed-font/usvg shaping, decorations, significant spaces, own-node and canvas ink bounds, styled tree/data checks and activation/note raster-pixel regression. |
| `viewer/src/examples.rs`, `viewer/tests/native_lifetime.rs`, two new `.puml` fixtures | Styled sequence and nested graph built-ins and isolated native worker coverage. Fixtures are independently authored, not claimed original oracle outputs. |
| README, THIRD-PARTY-NOTICES, provenance manifest, audit tool/tests | Actual support ledger, limits/residuals, exact translated source identities/notices and narrowly extended audit scope. |

## Supported syntax and rejection boundary

Only a prefix before the first family statement is extracted. Command-named sequence endpoints retain family precedence; class member rows and raw note/action bodies are not globally scanned. Interleaved UML/tree preambles fail in their family grammar. Chrome-only documents fail.

- Single-line `title`, `legend`, `header`, `footer`, with whitespace or colon separators. Header/footer accept preceding `left`, `center`, `right`. Title/legend center; header defaults right, footer defaults center. Repeated region replaces both text and alignment. Explicit supported line breaks work within a single-line command. UML/tree enclosing quotes are stripped as verified; empty/multiline block forms fail.
- Skinparam keys: defaultFontName/defaultFontColor affect all roles; defaultFontSize affects body elements only; title/legend/header/footer FontName/FontSize/FontStyle/FontColor; BackgroundColor. Case-insensitive admitted key spelling. Unknown, dotted, locked, stereotype, family-specific and block forms fail.
- `<style>` sheet: top-level `root`, `element`, `document`; document may contain title/legend/header/footer. Root/element/region FontName/FontSize/FontStyle/FontColor; document BackGroundColor. One selector opening or declaration per physical line, balanced standalone closing braces, optional declaration-ending semicolon. No general CSS, compact same-line blocks, arbitrary selectors or nesting. Unknown declarations anywhere reject the whole input. Empty/unclosed rules fail. Later authored matching properties win; all mixed skinparam/style usage is explicitly rejected rather than inventing precedence.
- Values: colors exactly #RRGGBB, sizes integer6–96, family1–128 UTF-8 bytes containing only alphanumeric characters, space, hyphen or underscore (optional enclosing quotes; whitespace-only fails). No comma/path/URL/CSS characters. FontStyle normal/plain, bold or italic maps to weight/slant; arbitrary combinations fail.
- Display/tree labels: paired **bold**, //italic//, __underline__, --strike--; nested distinct spans; paired case-insensitive b/i/u/s tags. Static/object underline and abstract italic compose with runs. Adjacent equal complete styles coalesce.
- Activation policy: pair each unescaped symbolic kind with its next same-kind occurrence within a physical or explicit hard line in linear work. Odd final markers are literal. Activated spans must be nonempty and correctly nested; crossed/duplicate active styles, mismatched symbolic/tag closures and unclosed recognized tags fail. Tags may span lines, but symbols pair only within each line. Paired wave `~~` and monospace double quotes remain unsupported. Lone symbolic punctuation and empty-slot quotes remain literal.
- Tilde hides only `_ - " # ] [ * . / <`; other tildes are literal. Backslash n, backslash, tilde and quote decode once; unknown/trailing escapes and r/l/t fail. Escaped delimiters never reactivate; genuine PUA survives unchanged. Literal http/https/ftp/file URL slashes remain literal through whitespace (explicit native deviation from accidental original pairing across URLs).
- Links/images/sprites/math, inline font/size/color, list/table/heading/horizontal-rule syntax and recognized unsupported tags fail. Ordinary unrecognized angle text is XML-escaped literal text, never output markup. All `!` preprocessing directives remain denied, including raw note/action/YAML directive regressions. No imports/themes/definitions/TIM/resources.

### JSON/YAML boundary

Both authorized data factories invoke StyleExtractor. The native deliberately narrow data prefix admits `title ` and the finite `<style>` block before the first payload row, applies effective fonts to data keys/values and title, and preserves every subsequent row as payload. Data titles keep enclosing quote characters, matching StyleExtractor; their display uses the ordinary measured document-title path. Ordinary data skinparams and header/footer/legend commands are rejected explicitly.

Data label text itself never enters Creole/escape decoding: JSON decoded strings and YAML scalar contents retain prior literal behavior, even for `**...**`, backslash-n, title/skinparam words or angle-like text. Quoted rows, lists and literal blocks are not extracted. YAML first keyed rows win over preamble recognition using the existing quote-aware colon rule, including `title heading: value`, `skinparam defaultFontName: literal`, and `left header heading: literal`. Thus an unquoted colon inside a proposed YAML prefix title denotes payload, a documented deliberate ambiguity resolution preserving admitted data. There is no payload-wide command stripper.

## Measurement, paint and geometry

Base body font remains14px; older families retain generic sans-serif, tree/data retain the identical explicit family list `Arial, Liberation Sans, DejaVu Sans, Noto Sans, sans-serif`. Authored family overrides reach both host measurement and SVG paint for participant/message/note/frame/header/member/slot/container/edge/endpoint/state/activity/branch/group/tree/data and document-region roles. Title defaults14px bold, legend14px normal, header/footer10px #888888; root name/color affects chrome too.

Each coalesced nonempty run is measured with the exact emitted text/family/size/weight/slant. Cache identity includes original text, display/tree/data mode, font family/size/weight/slant/color, underline and strike. Immutable line storage is shared by cache/layout/scene clones. Width is summed run advances; line height is max(font size+4, measured height+4). All runs on a line share baseline `height - font_size/4.5`. Blank lines retain an empty SVG row and height without a host query. `xml:space="preserve"` retains significant spaces; space-only runs have no usvg ink node but their measured x advance still positions later runs.

This is not native ascent/descent/ink measurement. Native bridge metrics are genuine, but baseline remains the documented approximation. Generic and absent-font resolution can still differ between measurement and usvg; tests/captures use installed Liberation Sans and preserve existing tree/data fallback coverage. No claim that arbitrary missing fonts fit identically.

Chrome order is header, title, complete family scene, legend, footer. Width is max(body canvas width, each region width+48). Body translation centers it beneath the widest region and compensates min_x/min_y. Region heights and12-unit gaps are real allocations;24-unit outer margin uses native layout constants, not fixture/oracle coordinates. All existing scene IDs/owner/target metadata survive translation. Scene push rechecks finite coordinate limits. Own-node containment is tested directly from flattened usvg text; no canvas expansion or fixture padding is used to cure node overflow. Final captures report no containment-expansion diagnostic.

### Narrow capture-discovered layering correction

The first styled sequence capture showed an existing activation rectangle overpainting a note. Cause: activation geometry is completed at deactivate and previously appended after earlier note/message foreground. A scene-order regression failed; an actual resvg pixel was white `[255,255,255,255]` instead of the note background `[238,242,255,255]`. A stable final ordering now places lifelines first, activation rectangles next (earlier-start outer before nested), and unchanged event foreground last. The sort is work-charged before allocation. Coordinates, lifecycle extents, IDs and semantic event order are unchanged; messages/arrows remain foreground. Both regressions pass and regenerated capture is visibly correct. This is original native renderer repair, not additional translated upstream code; source inventory remains110.

## Bounds and failure-stage correction

Defaults remain source256KiB, SVG16MiB, model10,000, nesting64, work5,000,000, cooperative10s; hard isolated Unix worker10s/2GiB remains unchanged. Labels are at most16KiB before Creole lexing/cache key construction; quoted and raw multiline accumulation checks before append. At most4096 lexical tokens per label; styles/tokens/runs/lines/regions charge shared model/text/depth/work budgets before retention. Expanded text cap remains min(8×source_bytes,svg_bytes). One aggregate4096 measured-run request cap includes body plus chrome, checked before the next callback. Tests observe4096 actual host queries then rejection of4097, not independent per-region counters. Cancellation is checked inside scans and before/after each host call; an in-flight host cancellation prevents the later styled run request. Bounded SVG writer and terminal resource validator remain shared.

The previous 60,000-row whitespace action scanner fixture creates a120001-byte label, now intentionally outside the16KiB frontier. Initial final-suite run failed its old success assertion. With controller approval it retains the entire60,000-row source scan but asserts exact `PlantUML label bytes value 16385 exceeds limit 16384` during bounded accumulation, with linear work ceiling. Before the fix the same refined test failed with value120001, proving late rejection. A separate4000-row/8001-byte success control and cancelled parse check remain. No generic is_err substitution or stress-size reduction was used.

## Source provenance and notices

Read the full prepared semantics evidence and inspected the authorized MIT files actually translated under `/mnt/data/Projects/plantuml-mit/src`. Only that source tree was used; no rejected candidate, GPL/EPL/Smetana source or untraced assets. Manifest records exact SHA-256, native destinations, behavior and verbatim source copyright/contributor lines; audit compares source bytes to source-JAR entries.

New15 selected files (under net/sourceforge/plantuml):

| Sources | Translated responsibility |
| --- | --- |
| command/CommandTitle, CommandLegend, CommandHeader, CommandFooter, CommandSkinParam | Finite single-line admission, quoting/alignment and key/value grammar. |
| TitledDiagram; core/DiagramChromeFactory | Replacement/default region roles and composition order, not original theme geometry. |
| style/FromSkinparamToStyle; style/parser/RawStyleParser; style/value/Specificity, ValueImpl | Finite role mapping, selectors and authored matching assignment order. |
| klimt/creole/Display; klimt/creole/legacy/StripeSimple | Shared display escaping/line behavior and literal URL interpretation boundary. |
| jsondiagram/StyleExtractor; yaml/YamlDiagramFactory | Verified narrow data prefix extraction and application. |

Five existing entries received extended behavior/destination mappings: jsondiagram/JsonDiagramFactory; klimt/font/FontStyle; klimt/creole/legacy/CommandCreoleBuilder; klimt/creole/command/CommandCreoleStyle; utils/CharHidder. All selected additions carry Arnaud Roques2009–2024/Original Author lines and complete MIT grant in notices/manifest. Existing Hisashi Miyashita, Serge Wenger and EclipseSource/minimal-json credits remain unchanged. No additional source was added after controller's independent110-entry notice/hash verification.

Final inventory:110 selected source/JAR hashes,24 native modules,13 fixture hashes. Audit allowlist extended only for exact selected TitledDiagram, Display and four style source paths; StyleLoader and unrelated parser paths remain denied. Merman audit unchanged:656 imported files,4 recorded patches. No new dependencies. The source-JAR identity is a locally checked artifact hash, not a claim of externally authenticated Maven checksum.

## RED/GREEN and verification commands

All cargo invocations from the standalone product directory use `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`; tests/build/Clippy use `--locked --offline`. Scoped elevated execution was needed for the environment's bwrap RTM_NEWADDR failure; no network operation was needed.

Observed iteration ledger (earlier focused outputs were in tool transcript; final logs below are retained):

1. `cargo test --locked --offline -p diagram-engine --test plantuml_styles`: first8 tests →3 controls passed/5 failed for missing style/chrome/data behavior (RED); after measured-run/preamble/chrome integration8/8 GREEN.
2. Expanded styles suite13 tests: unsupported block-markup regression RED12 pass/1 fail; finite block rejection →13/13 GREEN.
3. First keyed-YAML/blank-family regression run15 tests →13 pass/2 fail; payload-boundary and grammar corrections →15/15 GREEN. Final in-flight styled cancellation makes16/16.
4. Existing `--test plantuml --test plantuml_control --test plantuml_tree_data` →33+23+18=74 GREEN after safe-style frontier negative replacement; occupied events, compound captions, outward endpoint labels, raw directives, family keyword precedence, activity gutters/joins, distinct tree ancestry and data row-edge regressions retained.
5. `cargo test --locked --offline -p diagram-engine plantuml::text::tests` →5 GREEN: aggregate4096 requests/body+chrome; oversized label before work/cache/host; complete-style/color cache and coalescing; token cap before host; linear escaped-marker work1000/2000/4000.
6. Viewer example selection RED for unknown styled built-ins, then GREEN with both registered. `cargo test --locked --offline -p diagram-viewer --lib` →39 GREEN (actual native font/paint inspection included).
7. Old full run →53 private engine pass/1 fail at superseded120001-byte scanner-success assertion. Refined exact earlier-bound regression RED0/1 with reported120001; bounded append implementation →`cargo test ... -p diagram-engine plantuml::activity::tests`2/2 GREEN, including below-cap/cancel control.
8. `cargo test ... -p diagram-engine should_paint_activations_between_lifelines_and_event_foreground` RED0/1; `cargo test ... -p diagram-viewer should_shape_styled_runs_and_preserve_spaces_in_actual_native_paint` RED0/1 actual white raster pixel. Corrected an initial test-only tiny-skia namespace compile error before observing that raster RED. Stable layers →both GREEN; native shaped runs14 sequence/15 class, actual font report Liberation Sans→Liberation Sans.
9. Fresh full gates after the final product edit all exit0:

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline --workspace
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all --check
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo build --locked --offline --release
node tools/audit-plantuml.mjs
node tools/audit-merman.mjs
node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo deny --config ../deny.toml check licenses sources
git diff --check
```

Final workspace tests:268 passed,0 failed/ignored. Breakdown: engine private56, compatibility11, Mermaid families37, Mermaid21, PlantUML33, control23, styles16, tree/data18; viewer private39, export5, native-lifetime3, worker6. In addition7 worker-fault cases passed (blocked stdin/stdout, stderr flood, oversize frame, early exit, memory, cancel);0 doc tests. Eight source-audit tests passed. Strict Clippy/fmt/release and deny licenses/sources pass. Inherited vendored merman-render environment.rs:585 Atomic::fetch_update deprecation warning remains; no new product warnings, no vendor mutation.

Final retained logs: `/tmp/ive-plantuml-3d.QOjUH3/tests-final.log`, `clippy-final.log`, `release-final.log`; earlier native focused log `viewer-focused.log`. No full gates rerun after each minor edit; final gates followed the final code correction.

## Native background captures and inspection

Actual release viewer, own stateless instances, no focus activation:

```sh
/mnt/data/Projects/ive/target/release/diagram-viewer --background --language plantuml --example plantuml-styled-sequence --capture /tmp/ive-plantuml-3d.QOjUH3/styled-sequence.png
/mnt/data/Projects/ive/target/release/diagram-viewer --background --language plantuml --example plantuml-styled-graph --capture /tmp/ive-plantuml-3d.QOjUH3/styled-graph.png
```

Both exit0,1360×880 GPUI scene-readback PNGs, inspected directly. Sequence shows18px participants/body,26px two-line title, aligned header/footer, real legend, nested bold+italic, underline and strike, raw-note blank row/significant spaces; activation bar is behind the note after the narrow repair. Nested graph shows Commerce/Accounts containment, abstract class italic, static+bold member, italic method, underlined endpoint multiplicity/member, struck edge label and four nonoverlapping document regions. Fit scaling differs because their intrinsic layouts differ; this is not oracle screenshot parity.

| Capture | SHA-256 | Native host queries / deterministic fallbacks |
| --- | --- | --- |
| styled-sequence.png | `547c168ed5ec8cac12af5ba39ff6f6de7798e6ddbfbb3ad670829ffc9089f242` |20 /0 |
| styled-graph.png | `fa6360cdc9d6d1386704720cdb6851bbc2849a1568ae49c1ba2f90dfe6147b5d` |22 /0 |

Logs `capture-sequence.log`, `capture-graph.log` contain those truthful host counts. Separate actual native tests inspect14/15 painted text runs (whitespace-only rows have advances but no ink nodes), actual bold/italic/underline/strike spans, significant-space strings and own-node/canvas ink bounds. These tests use their own controlled sources, so their run counts are not mislabeled as capture host counts. Capture instances self-quit and workers ended; scoped pgrep of debug/release diagram-viewer paths returned no processes. No user processes were killed.

## Self-review, concerns and residuals

Reviewed complete diff and new modules for prefix/body ownership, source-order semantics, escaping, shared request/emission fonts, data literal policy, cache fields, preallocation limits, cancellation, geometry/ownership, resource denial and exact source scope. No known failing gate or unresolved blocker. Captures exposed a genuine existing layer defect and it was repaired/tested, not hidden by fixture changes or extra padding. The wide-label/compound/endpoint regressions remain substantive.

Residuals are intentional: finite line-oriented selectors/skinparams and single-line chrome only; all mixed style/skinparam declarations rejected; no themes/resources/general CSS, inline size/color/font or waves/monospace/rich markup; no literal !define or other preprocessing; no extra family features. Font fallback mismatches for generic/unavailable families and approximate baseline remain, with no claim of arbitrary-font glyph-bound parity. No automatic wrap. Native Dagre/layout, simplified symbols, group frame styling, connector routing and complex edge-label collisions do not claim original Graphviz/Teoz exactness. Shared Creole failures currently report line1 within the label parser, whereas prefix/style errors retain authored source line numbers. Source spans for per-run diagnostics are a later diagnostic improvement, not silently claimed here. Independent SPEC/QUALITY review is the next gate; no next-task implementation begun.

## FIX ROUND 1 — I1 measured sequence frame headers

Review source read completely: `/tmp/ive-plantuml-styles-review.md`. Important I1 was accepted after inspecting the fixed opening extents and drawing offsets: the frame kind was measured only during drawing; 20-unit kind/caption separation and top border offsets did not scale with the admitted font. The old18px capture did have a border intersecting `group`; the earlier capture self-review missed that separate defect. This appendix supersedes its frame-header assessment, not the activation/note pixel correction. Minor M1 remains inherited vendor noise, outside this wave.

- Fix base: `0fe389716232d3403940f0b145f15de0a58a4bfe`.
- Fix HEAD: `71cd81767477c11fcc28c97c0498c69a49f98bcf` — `Reserve measured PlantUML sequence frame headers`. Tracked worktree clean after scoped commit.
- Exactly3 product files changed: `engine/src/plantuml/sequence.rs`, `engine/tests/plantuml_styles.rs`, `viewer/src/svg/tests.rs`. No source/notice/dependency/fixture changes, no new translation source (inventory remains110), no next-task work, subagents or reviewer dispatch.

### Geometry/API correction

Private `EventLabel { caption: Label, kind: Label }` carries both already-measured labels. Opening kinds are measured through the same Context before row allocation; drawing performs no late kind measurement. Non-opening kind placeholders reuse the empty-label cache and issue no host request. Public APIs/worker protocol/aggregate query counter are unchanged.

Defined `FRAME_GAP=8` is an inter-box/edge gap, not a replacement fixed font-height offset. An opening anchor is now the frame top. Its occupied header height is `kind.height + caption.height + 3*FRAME_GAP`: top inset, measured kind, gap, measured caption (including every physical/styled line), bottom inset. Kind starts at top+gap; caption starts at top+2*gap+kind.height. Subsequent events reserve their own extents after this complete header. Frame bottom still follows the closing event's existing16-unit occupied extent. Both labels are inside the frame's border; nested frame headers are below their parents' occupied header, and following messages start after the preceding header/frame extents.

Horizontal layout includes max(caption.width,kind.width) in both participant stepping and frame margin calculation. Header x is derived from its actual frame-left edge+gap, so long frame kinds as well as captions have reserved width. No canvas expansion, larger fixed vertical offset or fixture padding was introduced. Relative event order, participant ownership, lifelines/create/destroy and activation semantics are unchanged; event coordinates move only to reserve measured headers. The stable activation/background/foreground ordering from the prior checkpoint is retained exactly. This remains an original native rectangular frame layout, not a claim of upstream tab/theme geometry.

### Meaningful RED/GREEN and named coverage

All commands below run in the standalone product directory with `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target` and locked/offline Cargo flags.

1. `cargo test --locked --offline -p diagram-engine should_reserve_measured_nested_frame_headers_at_small_and_large_fonts`: RED0/1, assertion `18px frame text crosses horizontal border`. This private test now covers18/96px, both labels, styled multiline outer caption, nested critical frame, horizontal/vertical measured-box containment, strict kind/caption separation, parent header versus nested border, inside message versus nested header, and subsequent message after outer close.
2. `cargo test --locked --offline -p diagram-engine --test plantuml_styles should_use_measured_frame_kind_and_multiline_caption_extents`: RED0/1, recorder-backed top/height exceeds frame rectangle. This public test uses exact known host widths/heights, checks actual kind query at the authored size, both frame labels, explicit multiline/styled caption and horizontal/vertical serialized measured extents.
3. `cargo test --locked --offline -p diagram-viewer should_shape_styled_runs_and_preserve_spaces_in_actual_native_paint`: RED0/1 with18px native text ink `top254.20703 bottom267.42578` versus frame border `top263.14063`, proving actual text/stroke overlap. Existing24px sequence/class controls ran first and passed their assertions. The extended test includes18/96px nested frames, actual Liberation Sans shaping, all four decorations, header text/text separation and strict inset from every owning border. Cross-nested-frame horizontal borders are also checked against all header ink boxes, not merely overall canvas bounds.
4. After the measured-header implementation, private focused test1/1 and all17 public styles tests GREEN; native styled test1/1 GREEN. Nocapture output confirms Liberation Sans→Liberation Sans; prior controls14/15 painted runs and both18/96px nested-frame cases13 runs each.
5. After adding cross-nested and adjacent-event assertions, `cargo test --locked --offline -p diagram-engine plantuml::sequence::tests` passes7/7. Named retained checks include occupied multiline messages/notes, created head reservation, nested frame/lifeline destruction, reverse/lifecycle semantics and `should_paint_activations_between_lifelines_and_event_foreground`. Native paint test again passes1/1 including the prior activation/note raster pixel.

### Fresh final gates

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline --workspace
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all --check
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo build --locked --offline --release
node tools/audit-plantuml.mjs
node tools/audit-merman.mjs
node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo deny --config ../deny.toml check licenses sources
git diff --check
```

All exit0 after the final product edit.270 Rust tests pass,0 failed/ignored: engine private57, compatibility11, families37, Mermaid21, PlantUML33, control23, styles17, tree/data18, viewer private39, export5, native-lifetime3, worker6. Seven separate hostile-worker scenarios pass. Eight Node audit tests pass. PlantUML audit still110 source/ZIP hashes,13 fixtures,24 modules; Merman still656 files/4 patches. Deny prints `licenses ok, sources ok`. Existing Atomic::fetch_update warning is unchanged; no new product warning.

Retained fresh logs: `/tmp/ive-plantuml-3d-fix1.Gjfz6O/tests.log`, `clippy.log`, `release.log`. Focused RED/GREEN outputs are in the tool transcript. Full suites were run after focused iteration, not after each edit.

### Regenerated affected capture and final inspection

```sh
/mnt/data/Projects/ive/target/release/diagram-viewer --background --language plantuml --example plantuml-styled-sequence --capture /tmp/ive-plantuml-3d-fix1.Gjfz6O/styled-sequence.png
```

Exit0; actual1360×880 GPUI frame inspected directly. `group` and bold `Checkout` now occupy distinct measured rows inside the top-left frame border with visible clearance; neither is crossed by the frame line. First message follows the complete header. Note remains above the activation bar, the return arrow remains foreground, and all four document regions are visible. No source-fixture changes or canvas-containment correction. Native host queries20, deterministic fallbacks0, unchanged from the earlier capture. The unchanged graph capture remains the preceding checkpoint's evidence, not falsely labeled regenerated here.

- New sequence PNG SHA-256: `d3cc76d27aa310c21eccdaeaf4f0c99cfff1d697b50948758eb73b32520f6159`.
- Log: `/tmp/ive-plantuml-3d-fix1.Gjfz6O/capture.log`.
- Viewer self-quit; scoped debug/release diagram-viewer pgrep returned no processes. No user processes killed.

Self-review checked the measured-kind flow precedes allocation, width and height consumers use it, header drawing has no late measurement, nesting/close/lifecycle bounds remain coherent, and tests observe actual ink rather than canvas-only containment. No unresolved I1 failure or new concern found locally. Baseline/font fallback and broader parity limitations remain as documented. Frozen for controller's scoped re-review; later work remains held.
