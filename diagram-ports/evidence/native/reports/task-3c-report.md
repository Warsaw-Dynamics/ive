# Task 3C implementation report — native trees and ordered data

## Frozen checkpoint

- Status: implemented and locally verified; awaiting controller-arranged fresh independent SPEC and QUALITY review. No later family or concurrency work started.
- Explicitly released base: `49838e86eea67155710bf29d07179935e45009bc`.
- Exact product HEAD: `2b2355b3b6c9c4bf0558256708d762db9e3356ca` — `Add native PlantUML tree and ordered data checkpoint`.
- Worktree `/mnt/data/Projects/ive/.worktree/diagram-ports`, branch `diagram-ports`; product lives inside its `diagram-ports/` directory. All 26 committed files are in that standalone project. Tracked worktree clean after commit. This report is an ignored controller handoff artifact, consistent with preceding reports.
- One implementer, no implementation/review subagents. TDD and apply_patch used; only generated evidence/capture/export files were produced by their tools. No root IVE product edits, user profile, IVE instance, xdotool, focus activation, merge or push.

## Contracts and implementation

Public render/options/results/measurement/control/worker APIs are unchanged. Existing Budget, Context constructor/label/styled_label, Scene and serializer signatures remain callable as before. No dependency or Cargo.lock changes, no GPUI dependency or file/network/environment/clock/process access added to the engine. Worker protocol, hard watchdog, stale-job handling and native measurement capability are unchanged.

Narrow reusable additions for the next shared-style task:

| Interface | Final contract |
| --- | --- |
| `source::prepare_document` | Returns `(Wrapper, Vec<Line>)` for exact lower-case mindmap/WBS/JSON/YAML wrappers; delegates UML to unchanged `prepare`. Original line numbers and raw new-family body rows survive. One complete nonempty block, matching end and full outside-input consumption required. |
| `tree.rs` | Separate `Mindmap { nodes, roots }` and `Wbs { nodes }`; ordered `Node` has typed Parent/Side/Outline, normalized depth, label and alias. Mindmap has separate side chains/multiplier; WBS a global chain and one root. |
| `treescene.rs` | Separate native mindmap and WBS layout entry points. Shared measurement/storage, distinct geometry. Semantic nodes use `m{id}`/`w{id}`, owner IDs and branch/trunk/bus kinds. Pseudo WBS nodes remain semantic scene elements with no shapes. |
| `data::Value` | Owned typed `Object(Vec<(String, Value)>)`, `Array(Vec<Value>)`, String, Number(lexeme), Boolean, Null. JSON duplicates append; YAML replacements retain first key position. No serde_json::Value map/deduplication and no generic YAML loader. |
| `datascene.rs` | Containers `d{id}`, ordered typed rows `d{id}r{index}`, links owned by the actual corresponding row. Scalar placeholders remain between nested children. Container ownership points back to that row. |
| `Context::data_label` | Literal text measurement: no UML rich-text interpretation or backslash-n normalization. JSON decoding occurs once in parsing; YAML matching outer quote removal does not decode escapes. |
| `Context::tree_label`; private `Label.font_family` | Tree Display normalization plus the exact explicit family chain `Arial, Liberation Sans, DejaVu Sans, Noto Sans, sans-serif`; data uses the same chain. The Label carries the family through SVG serialization, so measurement and paint receive identical families/14px/normal style. Existing families retain `sans-serif` and their original decoration policy. Cache keys include label mode. |

The two prior `Label` literals in graph/stategraph **tests only** initialize `font_family: "sans-serif"`; their production layout logic was not changed. The SVG serializer reads the new field for all labels. Existing state ancestor-ownership rejection, command-like sequence identifiers, structural family selection and raw note/action preprocessing regressions all remain covered by the unchanged prior tests.

Defaults remain source 256 KiB, SVG 16 MiB, model 10,000, depth 64, work 5,000,000 and cooperative 10 s; host Unix worker independently retains hard 10 s / 2 GiB. Parsing checks depth/items before descent/value construction, charges strings/lookups/scans and overwritten YAML values, and uses bounded numeric lexemes. Layout/traversal/scene emission use Budget checkpoints; measurement checkpoints run before/after the host callback. Existing finite geometry, label/line limits, terminal SVG admission, resource checks and prebounded writer apply unchanged.

## Admitted syntax and truthful exclusions

### Mindmap

Ordered root/child org-mode star/hash prefixes; plus/minus markers (any minus chooses left); optional immediate `_` boxless nodes; explicit `0` root form; explicit `\\n` Display line breaks; left/right/top/bottom **side** commands. The first item is forced to root. Org smart levels use the actual captured first prefix, one-space tab conversion and the special leading-whitespace trim for prefixes ending `**`. The first nonroot raw depth defines the multiplier; later nonmultiples/jumps fail. Each side retains its own parent chain. Later raw level zero starts another vertically stacked map, resetting that map's multiplier/side chains.

Measured subtrees reserve nonoverlapping vertical extents with actual left/right branch placement and source order. No guessed visual indentation. Colors, multiline command variants, stereotypes and broader original direction-command phrasing are explicit unsupported features; explicit `\\n` labels are separate admitted syntax.

### WBS

Distinct smart levels (no mindmap multiplier or `**` trim), first-item root coercion, one root and one global last-parent chain across sides. Admits new-order `_` and `<`/`>` modifiers in either order, `(alias)` and `"Display" as Alias`, explicit direction overrides, boxless text and unlabeled `_` pseudo junctions. Depth jumps, repeated modifiers, missing labels and second roots fail.

Root is above a horizontal first-level fork/bus. Negative root children are prepended to the fork order, as source-defined; this does not promise every negative root child lies left of root center. Descendant collections attach left/right to their own parent's vertical trunk. Pseudo nodes retain parentage/routes despite zero paint. Legacy modifier order, colors, multiline variants, stereotypes and all crosslinks remain explicit unsupported features; crosslink labels are never silently dropped. Aliases remain authored node metadata, not a claim of crosslink resolution.

### JSON and data scenes

Strict complete JSON input, ordered **all** duplicate members, distinct types, arrays and semantic indices, Unicode/surrogate decoding, XML/apostrophe/colon/comment-looking literal string content, nested/empty containers and root scalars. Strings are bounded before decoding through serde_json's string decoder; recursive values are built by the dedicated bounded parser. Standard syntax violations fail. Numeric spelling is retained, with 128-byte lexemes, bounded digit scans, exponent magnitude at most 308 and finite f64 admission. Original narrow `//` and column-zero `#` extensions are deliberately rejected; no upstream unbounded skipComment loop was copied.

Object/array compartments preserve every source row and type, connect nested children from the corresponding row center, and reserve scalar placeholders. Arrays display indices. A root scalar draws one indexed row; empty containers draw one blank row while retaining Object/Array identity. Null/bool display uses `␀`, `☑ true`, `☐ false`; XML-inexpressible decoded control text is painted as control pictures. Data model strings remain typed literal values. Native geometry differs from Smetana; no Smetana implementation was read or translated.

### YAML

The authorized narrow line parser subset, not YAML 1.2. Tabs expand to four spaces. Maps, scalar lists, lists of maps, plain-dash map items and keyed flow lists are admitted. Colon/comment boundaries are quote-aware. Scalars `2`, `true`, `null`, `~`, `{}` remain **strings**; matching outer quotes are removed without JSON escape decoding or doubled-apostrophe interpretation. Keyed `[]` yields one empty string. Duplicate keys replace the value without moving the first position. All overwritten work is still charged.

Literal `key: |` trims each consumed line and appends LF, including final LF. It deliberately retains the original stopping quirk: plain/blank rows are consumed regardless of indentation; a structured row stops it only when nondeeper. Quoted/block anchor/tag-looking characters stay literal data. Folded blocks, list-item blocks, chomping variants, plain continuation, document markers, nested flow containers and inconsistent indentation remain a narrower explicit rejection boundary.

An entire bounded quote/context-aware capability pass completes before constructing any Value model. Unquoted anchors/aliases/tags/merge keys (including spaced `<< :`) fail at their first token; quoted flow escapes do not turn literal `*`/`&` into capabilities. The exponential alias regression uses model_items=1 and returns UnsupportedFeature at line 2, proving rejection before value construction/expansion.

Active leading preprocessing directives are different from inert payload text: the common raw-wrapper boundary rejects them even in literal YAML blocks. Existing UML raw note/action behavior is preserved. No blanket substring ban is applied to data strings or literal block anchor/tag characters.

## Authorized sources, hashes and notices

Only actual authorized MIT-edition source under `/mnt/data/Projects/plantuml-mit/src` was inspected for translated behavior. The complete required semantic research and preceding reports (including fix appendices) were read. No GPL checkout, rejected candidate/examples, gen/, h/, Smetana, ELK or untraced asset was read/imported for translation.

The provenance manifest now has **95 source entries**, **11 exact fixtures**, **21 native modules**. Its 26 new entries contain complete exact source-file SHA-256, destination files, behavior and actual author/contributor/copyright lines. Each file is byte-compared with its exact source-JAR ZIP entry by the audit. New sources are:

- mindmap `CommandMindMapPlus`, `CommandMindMapOrgmode`, `CommandMindMapDirection`, `CommandMindMapRoot`, `MindMapDiagram`, `MindMap`, `Branch` → tree parser/typed parent policies.
- wbs `CommandWBSItemNew`, `WBSDiagram`, utils `Direction` → marker/modifier/alias/smart-level grammar and rejection boundaries.
- wbs `WElement`, `Fork`, `ITFComposed`, `WBSTextBlock` → ordered root fork, side-relative descendant trunk and zero-paint topology; measured native geometry is original Rust.
- json `JsonParser`, `Json`, `JsonObject` → strict recursive scalar/container semantics and ordered duplicate members; native bounds/Unicode decoding are intentionally stricter.
- jsondiagram `JsonDiagramFactory`, `TextBlockJson`, `JsonDiagram` → strict payload boundary, row/type/child placeholders and scalar/empty-root drawing policy, not Smetana layout.
- yaml/parser `YamlLine`, `YamlParser`, `YamlBuilder`, `Monomorph`, `MonomorphToJson` → actual narrow line/string/flow/block/list-map/duplicate policies.
- text `TLineType` → active leading preprocessor classification/rejection in raw wrappers. `TContext` was inspected to confirm dispatch; no execution logic was translated, so it is not added to the translation inventory.

Source artifact SHA-256 remains `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`; jar identity remains `e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1f1a515f7f70dd`. These are verified local artifacts, not independent external authentication.

Existing Arnaud Roques, Hisashi Miyashita and Serge Wenger credits remain. New embedded minimal-json semantics retain **Copyright (c) 2016 EclipseSource** and its complete separate MIT grant in LICENSE-PlantUML-MIT.txt and THIRD-PARTY-NOTICES. The audit checks all actual dated copyright lines and existing flexible-spaced Original Author/Contribution lines; a negative regression removes EclipseSource and must fail. The complete minimal-json grant is compared after trailing-whitespace normalization only (one original blank-comment trailing space was removed for git whitespace hygiene); source/ZIP bytes/hashes remain exact. No additional actual new author credit was found or invented.

## Observed RED/GREEN and tests

Cargo commands run from standalone `diagram-ports/` with `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`, locked/offline. Ordinary exec in this worktree encountered the environment's bwrap loopback RTM_NEWADDR permission failure; scoped elevated execution was used. No automatic escalation rejection occurred.

Observed behavior RED before its corresponding implementation/fix:

1. `cargo test --locked --offline -p diagram-engine --test plantuml_tree_data`: initial **2 pass / 6 fail**, each successful-family case rejected its missing wrapper at line 1. Six tests cover side chains/root stacking, depth normalization, WBS parent/pseudo/bus topology, duplicate JSON/type/row links, YAML semantics and literal data content. Negative/limit tests already passed because every new wrapper was rejected; those initial passes are not independent safety evidence.
2. Extended public run **12 pass / 2 fail**: explicit `0` root missing, and active `!include` inside literal YAML painted as text. After exact root grammar and raw-wrapper directive rejection, **14/14 passed**. Quoted/inert block capability-looking controls stayed admitted.
3. Viewer example lookup failed for missing `plantuml-mindmap` before adding all four examples; focused example tests then passed. An obsolete prior test expecting valid `@startjson {}` to fail was changed to a genuinely mismatched `@endyaml` case.
4. Spaced YAML merge request test failed (`unquoted merge request admitted`) before context-aware key detection. After correction the same test also exercised escaped-quote flow literal `*`/`&` preservation and passed.
5. Audit acceptance initially rejected newly selected source directories. After scoped allowlist extension, **4 pass / 1 fail** showed the missing EclipseSource copyright guard (`Missing expected exception`). Enforcing source copyrights and complete embedded MIT grant made **5/5 PlantUML audit tests** pass.
6. Actual capture exposed long data values protruding past compartment borders. Genuine viewer-resolver glyph test was RED: long URL painted right=424.2991 while border right=419.4541. The new same-family Label field/Context modes/serializer made it GREEN without fixture padding. Full log `red-native-paint.log` → `green-native-paint.log` in evidence directory.

An initial attempted native integration test used default usvg options and could pass without inspecting text when generic resolution failed. It was removed and replaced with the actual viewer resolver, recursive text inspection, an explicit positive inspected-text count, and border **path** bounds rather than box+text union bounds. It is not claimed as meaningful evidence.

Additional private semantic/geometry controls passed immediately against working implementation; they are supplemental, not invented RED observations. They inspect immutable typed values, duplicate identities, side/global parents, raw normalization differences, WBS direction/alias/pseudo retention, JSON pre-descent depth rejection, numeric scan bounds, exact YAML flow/block/list-map conversion, pairwise node nonoverlap, label containment, root bus versus parent trunks, and actual data row connector coordinates. Public tests cover malformed data, alias bombs, each caller limit, cancellation during native layout measurement, literal Unicode/resource-looking text, root/empty policy and exact fixtures. Every public success uses terminal validation and usvg with positive finite geometry/deterministic SVG.

Final commands after the final Rust behavior edit:

| Gate | Final result |
| --- | --- |
| `cargo test --locked --offline --workspace` | **235 Rust tests, 0 failures: 42 engine-private + 193 nonprivate** |
| Breakdown | 42 engine-private; 11 compatibility; 37 Mermaid families; 21 general/Mermaid; 33 prior PlantUML; 23 control PlantUML; 16 new tree/data; 38 viewer unit; 5 export; 3 native lifetime; 6 worker integration |
| Separate actual hostile-worker executable | **7 passed**: blocked stdin/stdout, stderr flood, oversize frame, early exit, memory, cancel; not included in 235 |
| Strict Clippy all targets | `cargo clippy --locked --offline --all-targets -- -D warnings`, exit 0 |
| Release | `cargo build --locked --offline --release`, exit 0 |
| Formatting/whitespace | `cargo fmt --all -- --check`, `git diff --check`, staged diff check, exit 0 |
| License/source closure | `cargo deny --config ../deny.toml check licenses sources`, licenses ok/sources ok |
| PlantUML source audit | 95 exact MIT source/ZIP hashes, 11 fixtures, 21 modules, credits/grants preserved |
| Existing vendor audit | 656 imported files, 4 recorded patches unchanged |
| Node audit guards | 8 total passed: 5 PlantUML + 3 existing vendor |

Final logs are `/tmp/ive-plantuml-3c.DYZIBF/tests-final.log`, `clippy-final.log`, `release-final.log`, `native-final.log`; source/license checks were also read directly. The only warning is the unchanged audited vendored Merman Atomic::fetch_update deprecation. No omitted current failures. Root IVE tests/build were not run or claimed.

Controller independently reran the235 Rust tests, all seven actual worker faults, both source audits, all eight Node guards, strict Clippy/fmt/release and deny, and independently matched all95 selected source/JAR hashes and exact credits; controller reported PASS. This is verification evidence, not the outstanding independent specification/quality review gate.

## Actual native font/capture evidence

Capture investigation recorded query/emitted exact string `https://example.test/a:b # text`, family `sans-serif`, size 14px, normal style/default weight400. Native selected **Liberation Sans**. Viewer generic font query was unresolved; its resolver selected first face **FontAwesome**, and actual positioned glyphs used **Noto Sans CJK JP**. This explains the measured/painted width mismatch; it was not fixed with invented glyph widths, fixture-specific padding or canvas expansion.

After the controller-authorized narrow fix, all new tree/data labels request AND emit `Arial, Liberation Sans, DejaVu Sans, Noto Sans, sans-serif`, 14px normal400. Actual Latin glyphs now use Liberation Sans in both; the URL paints to411.35156 inside the same419.4541 border. The Unicode control uses actual Liberation Sans/Noto Sans CJK JP/Noto Color Emoji glyph fallbacks. The regression includes short strings, long strings, Unicode, nested rows and both tree families. This is actual glyph-bound test evidence on this installed-font environment, not a universal font fallback/ink-bounds capability claim. Existing-family generic resolution is unchanged and remains a broader future shared-style concern.

Final background release captures were regenerated after the final product edit, opened and visually inspected at1360×880. All use `/mnt/data/Projects/ive/target/release/diagram-viewer <multifeature.puml> --background --capture <family>-final.png`, exit0 and automatic quit. Evidence directory `/tmp/ive-plantuml-3c.DYZIBF` retains source and matching logs; initial captures remain `*-before-font.png`.

| Capture | Native/fallback | Actual inspection | SHA-256 |
| --- | --- | --- | --- |
| mindmap-final.png | 12/0 | Separate left Tests/Unit/Integration and right Delivery/Package/Operations/Monitor; boxless Documentation and second vertically stacked map; text clear of branches | `7c75b725962f613235e9c49e4fe064c0efeda4f832b6ea92a89e62196cde5a88` |
| wbs-final.png | 10/0 | Migration above bus; prepended Preparation column; Design left Validation/right Schema and descendant trunks; boxless Planning/Monitor; Review retained | `be796331ec1a60a4b12a775ba10c35626d7b0cc85238175d3c9667e23e0946e2` |
| json-final.png | 18/0 | Both service rows in input order with separate Billing/Ledger containers; scalar enabled/nothing placeholders; ports indices and empty container; long literal URL now inside compartment | `41690627b73510a0a5c2d1dc572a13967d8eb0ef66a6d3452055074110ba2e9f` |
| yaml-final.png | 23/0 | replicas latest3 in first position; north/south order, people list/map ownership, string true/false, literal block capability-looking text contained, quoted colon/hash literal | `a195e8bc0a9b4e750601d22e36eff4065d758ae5c8f1da159596f0a7cf9646bb` |

Focused actual-worker lifetime test passes and releases each native cache: sequence8/0, description8/0, state8/0, activity8/0, mindmap5/0, WBS5/0, JSON11/0, YAML11/0 (`native-final.log`). Final `pgrep -a -x diagram-viewer` found no processes. No IVE instance was started.

## Original oracle comparison

Four exact fixture files byte-compare with the independently authored `/tmp/ive-plantuml-native-oracles` originals; source hashes are recorded in the product manifest and below. Existing original artifacts were not modified. Every native exact fixture export is deterministic and parsed by usvg in integration tests.

| Fixture / original source | Exact source SHA-256 | Original viewBox | Native deterministic viewBox |
| --- | --- | --- | --- |
| mindmap / mindmap-hierarchy | `079e4ebf4341974d9c1176f5560a17ef677a31362ac23ee2df7aa2b1bac1a79c` | `0 0 347 133` | `-57.21 0 384.64 138.8` |
| wbs / wbs-hierarchy | `0dc40c5ffe0feeb9ffad194298c9efe06b65558f4f460a58b1a6486fac8504ae` | `0 0 262 197` | `0 0 332.18 242.2` |
| json / json-nested | `fefb478ac2559c3d452798502155a3a0ade75f569048d3fa237cfd809703f626` | `0 0 358 80` | `0 0 494.72 118.8` |
| yaml / yaml-nested | `5685849a4dbd310cb5bce028c2d8986d2a830927851af3972e97ec7f444566cb` | `0 0 338 102` | `0 0 486.22 154.2` |

Matching native exports are `{mindmap,wbs,json,yaml}-native.svg` in the evidence directory. Original/native shape counts are respectively9/10,13/14,19/12,19/12; row compartments and connector decomposition differ. No pixel/style/geometry parity is claimed.

Additional fresh original probes use only the verified MIT jar, headless Java, SANDBOX security profile, UTF-8, Liberation Sans and a20s timeout (`original-probe.mjs` and generated `*-original.{svg,json}` in evidence directory). All four multifeature sources admitted original, exit0. YAML literal-block file include exits200 with `cannot include missing-task3c-file.puml`; literal-block URL include exits200 with `Cannot open URL`; inert quoted/block capability-looking control exits0. Native rejects active preprocessing at the actual line3 and preserves inert controls. Java remains development-only oracle evidence, absent from production rendering.

## Residuals and review boundary

This is a reviewed-family subset, not the completed full PlantUML port. Explicit exclusions above remain errors. Exact baseline/ascent/descent/ink metrics and automatic wrapping are not added. Installed glyph fallback can still differ across platforms; actual same-family/paint containment regression covers this environment and representative strings, not every possible fallback glyph or font installation. Existing older-family generic fallback behavior was deliberately not broadened in this task.

Tree/data geometry is conservative original layout, with different spacing/connector decomposition from upstream. Pairwise node/label containment and tested row/trunk topology do not prove universal edge/label collision freedom. Source columns remain1. Some parser/layout/test functions remain longer than the preferred20 statements; responsibilities are separated by typed model/parser/layout rather than meeting that preference uniformly. YAML deliberately rejects some original quirks beyond the admitted forms, including plain continuation/legacy tree grammar; no generic YAML compatibility is claimed.

Fresh independent SPEC and QUALITY review must evaluate this exact base-to-head checkpoint, including raw-wrapper preprocessing, context-aware YAML preflight, duplicate rows/replacement, exact side/global parent policies, preallocation bounds, actual connector coordinates and font-family propagation. No concurrency/history or later inventory task is released by this implementation report.

## FIX ROUND 1 — YAML flow depth and charged linear field work

### Scope and exact revision

- Complete independent findings source read: `/tmp/ive-plantuml-tree-data-review.md`. Controller accepted Important R1/R2 and authorized one scoped repair wave; no additional family, subagent or self-dispatched review.
- Exact repair base: `2b2355b3b6c9c4bf0558256708d762db9e3356ca`.
- Exact repair HEAD: `6ec10226db28f084d627aad53276b51794009228` — `Bound YAML flow scalar depth and field scanning work`.
- Only committed files: `diagram-ports/engine/src/plantuml/yaml.rs` and `diagram-ports/engine/tests/plantuml_tree_data.rs`. Tracked worktree clean after commit. This appendix updates the same ignored handoff report.
- Evidence directory: `/tmp/ive-plantuml-3c-fix1.vLTFI8`.

### R1 — check each scalar before accumulation, including `[]`

`begin_flow_scalar` checks `depth + 1` and charges the scalar model item before creating/accumulating the first field, and again before the field following each comma. The final field no longer waits until construction to charge its item. The total admitted item count is unchanged, including the mandatory empty-string element of `[]`; rejection occurs earlier. Thus root object → flow array → scalar requires nesting_depth3, like the equivalent block YAML and JSON. A nested map adds a fourth level.

Covering tests:

- Private `should_check_flow_scalar_depth_before_any_field_accumulation`: nonempty, quoted, mandatory empty, whitespace-only, escaped and multi-field inputs reject at scalar depth3/limit2. It asserts `budget.work == 0`, proving the rejection precedes all field scanning, retained text and item work.
- Private `should_admit_flow_scalars_at_exact_depth_including_mandatory_empty_element`: depth3 admits the exact String values, including the one empty string.
- Public `should_enforce_flow_scalar_depth_at_exact_limit_and_one_below`: root/nonempty and root/empty flow arrays succeed at3 and fail at2; nested-map variants succeed at4 and fail at3. Successful SVG is parsed by usvg.
- Supplemental `should_enforce_flow_item_limit_before_next_field_accumulation`: model_items1 rejects a second field with identical consumed work whether its unseen suffix is three characters or16,000 spaces plus text.

### R2 — constant-time field state and fully charged finishing

The old `field.trim().is_empty()` prefix scan ran before testing for a quote on every unquoted character. Merely asserting a linear **reported** work count would not expose that defect, because the scan was uncharged. The evidence therefore distinguishes three stages explicitly:

1. Tests were written first. The uninstrumented baseline work tests passed while R1 failed. Direct flow whitespace4k/8k/16k reported8,001/16,001/32,001 work, despite the actual repeated prefix scans. This passing baseline is **not** claimed as proof of bounded processing.
2. With controller agreement, diagnostic accounting was added immediately before the existing trim to charge the prefix length. This conservatively meters the exact worst-case all-whitespace scans while retaining the old algorithm. Under the tests' linear work ceilings, both private long-field tests became RED: whitespace work16,109 exceeded16,032 and mixed-field work80,201 exceeded80,200. The public long-field test also became RED at81,533 versus81,524. This accounting-baseline instrumentation is not shipped.
3. The scan was replaced with a `FlowField` content flag maintained on every appended character, including escaped characters. Quote-opening checks now inspect only flags/current character. There is one final trim for an unquoted field, charged by its input byte length; the retained trimmed copy is charged with `Budget::text` **before** allocation. Quoted fields transfer their owned String without trimming/copying, preserving significant whitespace. Escaped characters receive their own scan charge and the ordinary bounded text charge. Flags reset between fields.

No elapsed-time threshold or benchmark claim is used. The final deterministic work evidence comes from executing the production scanner under explicit linear work budgets. All per-character paths are constant-time; plain finishing scans/copies each field at most once.

| Final executed case | Input bytes | Actual charged work | Allowed work |
| --- | ---: | ---: | ---: |
| Private whitespace | 4,000 / 8,000 / 16,000 | 12,001 / 24,001 / 48,001 | 16,032 / 32,032 / 64,032 |
| Private mixed leading whitespace, quoted text, escaped whitespace and quote | 20,034 / 40,034 / 80,034 | 52,087 / 104,087 / 208,087 | 80,200 / 160,200 / 320,200 |
| Public whitespace source | 4,025 / 8,025 / 16,025 | 44,126 / 88,126 / 176,126 | 81,524 / 161,524 / 321,524 |
| Public quoted/escaped source | 8,045 / 16,045 / 32,045 | 84,370 / 168,370 / 336,370 | 161,924 / 321,924 / 641,924 |

Private work tests are `should_scan_long_whitespace_flow_fields_with_linear_charged_work` and `should_scan_long_quoted_and_escaped_flow_fields_with_linear_charged_work`; public coverage is `should_render_long_whitespace_quote_and_escape_flow_fields_with_bounded_work`. They assert real typed values/SVG labels and bounded successful execution, not only counter constants. Supplemental semantic/bound tests `should_preserve_escaped_content_when_deciding_whether_quotes_open` and `should_charge_trimmed_flow_copy_before_allocating_a_second_string` passed against the working fix; they are not claimed as independent historical RED observations. Escaped nonwhitespace must prevent a later quote from discarding the literal prefix; escaped space/NBSP must continue allowing a quote to open. The copy-bound test verifies a rejected second allocation leaves the original field String intact.

### Observed commands and results

All Cargo commands ran locked/offline from standalone `diagram-ports/`, with `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`. Scoped elevated execution was used for the same bwrap loopback setup failure; no approval rejection occurred.

| Stage / command suffix after `cargo test --locked --offline -p diagram-engine` | Actual output |
| --- | --- |
| `--lib yaml::tests::should_ -- --nocapture` before fixes (`red-depth-unmetered-work.log`) | **7 passed / 1 failed**; R1 returned Array(String("one")) instead of rejecting depth3/limit2; unmetered long-work tests passed |
| `--test plantuml_tree_data flow_ -- --nocapture` before fixes (`red-public.log`) | **3 passed / 1 failed**; public one-below depth case rendered successfully |
| `--lib flow_scalar -- --nocapture` after R1 (`green-depth-private.log`) | **2 passed**, including zero pre-accumulation work and mandatory-empty exact depth |
| `--test plantuml_tree_data should_enforce_flow_scalar_depth -- --nocapture` after R1 (`green-depth-public.log`) | **1 passed** |
| `--lib should_scan_long_ -- --nocapture` with metered old scan (`red-metered-prefix-work.log`) | **0 passed / 2 failed**, deterministic work ceilings exceeded |
| `--test plantuml_tree_data should_render_long_ -- --nocapture` with metered old scan (`red-metered-public-work.log`) | **0 passed / 1 failed**, deterministic public work ceiling exceeded |
| Final `--lib yaml::tests::should_ -- --nocapture` (`green-private.log`) | **11 passed**, all old YAML private tests plus seven new controls |
| Final `--test plantuml_tree_data flow_ -- --nocapture` (`green-public.log`) | **4 passed**, all focused flow integration/semantic checks |

The final private item-admission test was added after the focused public run and passed in the final private/full-workspace runs. No later production edit followed these final gates:

| Final gate | Observed result |
| --- | --- |
| `cargo test --locked --offline --workspace` | **244 Rust tests passed:49 engine-private +195 nonprivate**, zero failures |
| Full suite breakdown | 49 engine-private;11 compatibility;37 Mermaid-family;21 general/Mermaid;33 prior PlantUML;23 control PlantUML;18 tree/data;38 viewer unit;5 export;3 native lifetime;6 worker integration |
| Actual worker fault executable | **7 passed**: blocked stdin/stdout, stderr flood, oversize frame, early exit, memory, cancel; separate from244 |
| `cargo clippy --locked --offline --all-targets -- -D warnings` | exit0 |
| `cargo fmt --all -- --check`; `git diff --check`; staged whitespace check | exit0 |
| `cargo build --locked --offline --release` | exit0 |
| `node tools/audit-plantuml.mjs` |95 source/ZIP hashes,11 fixture hashes,21 modules unchanged |
| `node tools/audit-merman.mjs` |656 imported files,4 recorded patches unchanged |
| `node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs` |8 passed |
| `cargo deny --config ../deny.toml check licenses sources` |licenses ok, sources ok |

Final logs: `tests-final.log`, `clippy-final.log`, `release-final.log` in the repair evidence directory. No source/notice/dependency/worker/scene/font contract changed, so no provenance entry or asset was added. Rendering/geometry captures were not regenerated for this parser-only repair; all existing family/native-worker/native-paint regression tests passed in the fresh full suite. Root IVE was not changed or run.

### Remaining review boundary

Important R1/R2 are implemented and covered; independent controller-arranged scoped re-review remains required. The review's Minor1 remains deferred by explicit controller scope: nested-flow and unterminated-flow-quote diagnostics still use line1; existing source-line propagation elsewhere was not changed. Minor2 also remains: the inherited audited vendor Atomic::fetch_update deprecation warning is disclosed, not refactored. All previously documented syntax/geometry/fallback limitations remain. No later work is released by this fix.
