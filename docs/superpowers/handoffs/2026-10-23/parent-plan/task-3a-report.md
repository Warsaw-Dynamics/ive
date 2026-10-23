# Task 3A implementation report

## Frozen checkpoint

- Base: `f16795fa96f94826b8d2dc567524d14fcf0fe06f`.
- Product commit: `b9815e81df754985a7bf4c1159d196251a5acb24` — `Add bounded native PlantUML sequence and UML graph checkpoint`.
- Worktree: `/mnt/data/Projects/ive/.worktree/diagram-ports`, branch `diagram-ports`; tracked checkout clean after commit. This report is an ignored controller handoff artifact, not staged product code.
- Scope: detailed plan Tasks 1–3, one implementer, no delegated implementation/self-review, no later-family implementation. This is a functional native **subset checkpoint**, not a full PlantUML port or original geometry parity.
- Independent review is requested before further implementation. The controller owns review/release; branch/worktree are preserved and nothing was pushed or merged.

## Delivered behavior

| Area | Implemented |
| --- | --- |
| Common | Exactly one nonempty complete `@startuml` block; BOM/CRLF, quoted display/alias, Unicode/XML literal text, apostrophe comment lines and block comments. Caller limits validated before parsing. |
| Sequence | Ordered explicit/implicit participant, actor and database identities; aliases; real heads/lifelines; directional/reverse/dashed/open messages and self calls; single-participant left/right/over single/multiline notes; nested group/alt/else/loop/opt/par/break/critical frames; explicit balanced activation/deactivation, creation and destruction. |
| Class/object | Class, abstract class, interface, enum, annotation, object; members/slots kept in source order, compartment separators, literal visibility characters, prefix static underline/abstract italic, underlined object titles; package/namespace nesting and scoped identity resolution. |
| Graph relations | Unique graph IDs independent of aliases; directed compound multigraph; parallel/self/cyclic/disconnected edges; inheritance/realization, association/dependency, aggregation/composition extremities; center labels and quoted endpoint labels/multiplicities (including textual role labels). Scoped implicit class endpoints inherit the relation's parent. |
| Description | Explicit actor/usecase/component/database/node/rectangle/package declarations; native stick/ellipse/component/cylinder/deployment/package geometry and actual container ownership. These are one shared description family, not inflated separate backend counts. |
| Graph notes | Named `note "text" as Alias` with explicit authored relations; escaped newline labels. Positioned/attached and multiline named graph-note commands remain rejected. |
| Viewer | Five language-specific built-ins, `.puml`/`.plantuml` selection with explicit language override, correct default example/language switching. Existing isolated worker, per-job font caches, logical zoom/raster separation and hard ceilings unchanged. |

Unsupported constructs fail rather than returning blank diagrams or substituting a different family: other wrappers/families; preprocessing/includes/macros/functions/conditionals; title/legend/newpage/scale/style/skinparam/theme; rich text/links/images/sprites/math; sequence anchoring/parallel anchor commands, attached activation shorthand, autonumber, references, other participant symbols, bidirectional sequence arrows; graph ports, together/hidden/directional edge constraints, links to container boundaries, slash-role qualifiers, inline extends/implements, generic declarations/stereotypes, visibility selectors, mixed class/description declarations, shorthand description symbols, attached/positioned graph notes. Duplicate declarations fail instead of implementing original declaration reordering/reopening. Unknown quoted escapes fail instead of deleting backslashes.

`par` is an interaction frame/branch, not a claim of anchored simultaneous layout. Plain Unicode labels and URL/XML-looking literal text are retained; supported paint styles are measured with the corresponding style.

## Runtime boundaries and resource accounting

- Engine remains GPUI-free, synchronous and source-in-memory. No runtime filesystem/network/environment/date access, process spawn, Java/JS/browser/dot invocation or raw Mermaid-source lowering.
- Defaults remain source256KiB, SVG16MiB, model10,000, nesting64, work5,000,000 and cooperative10s. Existing host worker hard10s/2GiB Unix isolation and independent watchdog remain intact.
- Native Budget charges scanning, parser lookups, text, checked parent graph construction, Dagre WorkControl, scene and serialization. It checkpoints before/after native measurement and inside the Dagre kernel.
- Pending graph relations retain numeric parent IDs, not repeated namespace strings. Before constructing/cloning qualified keys, cumulative expanded key text is bounded by `min(8 * source_bytes, svg_bytes)` and charged as work. A scaled20-level namespace/100-leaf regression demonstrates rejection before retained text amplification.
- Labels are bounded at16KiB; aggregate measured lines4096; native coordinates are finite and at most1,000,000 magnitude; negative primitive dimensions/radii fail. Scene item count is bounded. SVG writer checks growth before append/allocation.
- Public Merman session supplies font measurement and real provenance/reporting; output uses Merman resvg-safe terminal pipeline, existing resource-admission postprocessor and existing final SVG validator.
- DAG parents use `try_set_unparented_parents_ix`, edges use `try_set_edge_named`. No assertion-prone unchecked parent mutation is used. A deliberately cyclic parent model fails without panic; an almost-exhausted budget interrupts inside `layout_controlled` after graph construction.
- Direct exact dependencies `merman-render=0.9.2` (defaults disabled) and `dugong=0.9.2` reuse the existing selected closure and workspace patches. Cargo.lock adds only two diagram-engine dependency edges, no package/version. Existing no-math/no-ELK policy remains.

## Frozen internal interfaces and plan deviations

The public `render(source, &RenderOptions) -> Result<RenderedDiagram, RenderError>` is unchanged. `UnsupportedFeature(String)` is added; InvalidSource text is language-neutral. Worker error transport remains strings.

| Module | Actual callable/data contract |
| --- | --- |
| `plantuml/mod.rs` | `render(&str, &RenderOptions)`; validates limits, prepares source, creates controlled render session, dispatches model/layout, serializes and terminal-validates. |
| `source.rs` | `Line { number: usize, text: String }`; `prepare(&str, &mut Budget) -> Result<Vec<Line>, RenderError>`. |
| `lexer.rs` | `Token { text, quoted }`; `tokens(&Line, &mut Budget)`, `name`, `split_label`, bounded identifier/arrow recognition. |
| `budget.rs` | `Budget::new`, `checkpoint`, `charge`, `item`, `depth`, `text`; shared limits/control and work count; statement diagnostic helpers. |
| `text.rs` | `Context::new(&RenderSession, &mut Budget)`; `label`, `styled_label`; `Label { lines, width, height, line_height }`. Cache keyed by text and Decoration within one render. |
| `scene.rs` | `Point`, `Rect`, `Paint`, `Dash`, `Decoration`, `Shape` (rect/ellipse/line/polygon/text), `Element` with stable ownership metadata, bounded `Scene::push`. |
| `svg.rs` | `serialize(&Scene, &mut Budget) -> Result<String, RenderError>`; private prebounded XML-escaping writer. |
| `sequence.rs` | `parse(&[Line], &mut Budget) -> Sequence`; `layout(&Sequence, &mut Context) -> Scene`; typed ordered participants/events and IDs. |
| `uml.rs` | `is_graph`, `parse` producing `Uml { family, entities, relations, direction }`; Kind/Parent/Member/Relation hold actual entity ownership and extremity semantics. |
| `graph.rs` | `layout(&Uml, &mut Context) -> Scene`; private measurement/checked graph construction/WorkControl/bounds/intersection/rendering. |
| `arrow.rs`, `shapes.rs` | Typed extremity parser/drawer and original native shape geometry; no Graphviz/Smetana translation. |

Deliberate deviations from the larger design: no separate diagnostic/ast/parse/style files yet; equivalent responsibilities are composed in the modules above. Locations are **statement line and column1**, not precise token spans or byte ranges; generated-model errors may identify line1. No generic GraphLayout public adapter exists yet: graph.rs currently accepts Uml. State should extract a narrowly shared checked-graph layer or add its own typed adapter, not masquerade as class data. No style language is admitted: fixed14px sans-serif, plus actual italic/static/object decoration semantics. Own model fields use enums rather than optional wrappers; library APIs use their native Option contracts.

Next stage may reuse Budget, Context/Label, Scene, SVG terminal integration and checked WorkControl, while introducing **distinct** state/activity/tree/data ASTs. Extend wrapper dispatch only with positive/negative semantic tests. Avoid YAML load-then-expand alias bombs: bound streaming/alias admission before expansion. Do not treat current internal models as immutable upstream API commitments.

## Source/legal evidence

`diagram-ports/provenance/plantuml-mit-1.2026.9.json` records24 exact authorized Java files, each SHA-256, Rust destinations, translated/referenced behavior and original copyright/contribution lines. Every selected file was byte-compared with its source-JAR ZIP entry. The controller independently repeated all24 comparisons.

Source root: `/mnt/data/Projects/plantuml-mit/src`.
Observed source artifact: `/mnt/data/Projects/plantuml-mit/plantuml-mit-1.2026.9-sources.jar`.
Observed SHA-256: `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`.
This is **local artifact verification**, not an authenticated Maven checksum.

References cover BlockUmlBuilder/DiagramType/PSystemBuilder; NameAndCodeParser; selected sequence participant/arrow/group/activation/note commands; class/object/description factories/declarations/relations; named notes; Member/MethodsOrFieldsArea; StringBounder default descent. Exact MIT distribution grant is retained in LICENSE-PlantUML-MIT.txt; Arnaud Roques copyright and Hisashi Miyashita contributions are retained. Older header comments beneath the generated MIT grant are not hidden or relabelled.

No rejected candidate source/examples, other PlantUML edition, Graphviz/Smetana/ELK implementation, stdlib/icon/sprite/theme/font/math assets were imported. Native geometry, SVG/budget glue and Dagre adaptation are original code using separately audited permissive infrastructure.

Five fixtures match exact freshly authored independent oracle source bytes. Oracle jar SHA-256 is `e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1f1a515f7f70dd`, original reported version1.2026.9/18d0aae. Original admission is not native geometry equivalence. The broader original28-admitted/2-degraded inventory is **not** claimed as native implementation.

## RED/GREEN and final checks

Observed RED cases before implementation/fixes included seven initial public backend admissions (UnsupportedLanguage), missing viewer language/example integration, exact style/cache inconsistency, keyword-prefix identifiers, ambiguous actor/database family selection, class body sequence keywords, unsupported quoted escapes, attached syntax, mixed-family declarations, label markup, nested caption/child clearance, expanded namespace text, named graph notes, nested frame horizontal containment, negative primitive dimensions, missing ellipse intersection and finally the right-hand note exceeding its frame in the real capture. The corresponding tests are green. Existing safety tests added around already-correct code passed immediately; no claim they independently demonstrated RED.

Final full command after the last geometry fix:
`CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --offline --workspace` — exit0.

| Test target | Passing count |
| --- | ---: |
| Engine private native model/layout/writer tests | 7 |
| Engine compatibility integration | 11 |
| Engine Mermaid family integration | 37 |
| Engine Mermaid/general integration | 21 |
| Engine PlantUML public integration | 24 |
| Viewer unit tests | 17 |
| Viewer export integration | 5 |
| Viewer native cache/lifetime integration | 3 |
| Viewer worker integration | 6 |
| **Rust test total** | **131** |
| Separate worker fault scenarios | **7** |
| Node source-audit guard tests | **6** |

Engine integration count93; viewer integration14; private/unit24. Separate faults cover blocked stdin/stdout, stderr flooding, oversize framing, early exit, memory and cancellation. The previous Mermaid96-test baseline remains covered; its former unsupported-PlantUML test now verifies explicit successful dispatch.

Also exit0:
- `CARGO_TARGET_DIR=... cargo clippy --offline --all-targets -- -D warnings`
- `CARGO_TARGET_DIR=... cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=... cargo build --offline --release`
- `CARGO_TARGET_DIR=... cargo deny --config ../deny.toml check licenses sources` — licenses/sources OK.
- `node tools/audit-merman.mjs` —656 original imported files,4 recorded patches unchanged.
- `node tools/audit-plantuml.mjs` —24 exact MIT source/ZIP hashes,5 fixture hashes,12 native modules.
- `node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs` —6 guard tests.
- `git diff --check`, staged diff whitespace gate, and clean postcommit status.

Only the preexisting vendored Merman deprecated Atomic::fetch_update warning appears; audited vendor source was not modified. No root IVE product files were changed, so no root IVE build/tests were run.

## Native captures and process cleanup

Real release viewer readback, all opened with `--background` (no focus activation), captured through the existing GPUI app-frame path and automatically quit:

| File under `/tmp/ive-plantuml-3a.jYEiG5` | Result | Actual counts |
| --- | --- | --- |
| `sequence.png` | Inspected heads/lifelines, messages/dash/reverse arrow, note, activation, group; note now contained after capture-driven correction. | native8, fallback0 |
| `deployment.png` | Inspected cluster/package nesting, component glyphs, database cylinder, labels and two directed links. | native7, fallback0 |
| `unsupported.png` | Inspected real error UI for exact original `nested-state.puml`; explicit line2/column1 unsupported state, exit1 as expected. | no render success claimed |

Hashes:
- sequence: `d210ccad2f0c0cf4c97ef085022525b1dbcaf3df873d28db6d03310967a6f47c`
- deployment: `162583364a9e75d2cab105694d08cd2d1a4b425eb462d88339b304f0571fa45f`
- unsupported: `27692b14e70db687501ca14e252677c2333e3d80f48e3054e1e8b606c4236781`

A first error-capture attempt used a nonexistent temporary basename, correctly produced a read error, and was replaced with the exact resolved `nested-state.puml` capture. It is not counted as the unsupported-family evidence.

`cargo test -p diagram-viewer --test native_lifetime should_measure_plantuml -- --nocapture` independently printed the same8/0 and7/0 counts and verified per-job native cache release. Mixed valid/invalid host answers are separately tested for real host/fallback provenance. Final `pgrep -a -x diagram-viewer` returned no processes (exit1). No viewer or worker was left running.

## Explicit fidelity/residual ledger

- Full PlantUML compatibility is not delivered. Later state/activity/tree/data/timing/Gantt/Salt/nwdiag/Archimate and other inventory families remain future checkpoints.
- Fixed14px sans-serif with explicit-newline layout only. Measured advance/line height can be genuinely native, but SVG baseline uses StringBounder font_size/4.5 approximation; no fabricated native ascent/descent/ink provenance. Automatic wrapping and broader style grammar are rejected.
- Original colors, circled class letters, visibility icons, exact member regrouping, lifeline footboxes and upstream symbol geometry are not reproduced. Native member visibility remains text and source member order is preserved.
- Dagre layout deliberately differs from Graphviz/Teoz. The inspected nested deployment edge crosses the package caption region; labels remain present but edge-label/caption collision avoidance needs refinement. Complex labels, horizontal compound captions and all original routing constraints are not geometry-parity claims.
- Usecase endpoints intersect true ellipses. Other nonrectangular native symbols currently use measured bounding-box endpoints; cylinder/cut-corner/actor exact boundary routing is a known fidelity gap.
- Sequence spacing is conservative/global widest-label spacing, not minimal constraint geometry. Nested frame and note containment and lifecycle endpoints are tested, not arbitrary original timing/layout parity.
- Precise token columns, a generic language-neutral graph model, broad preprocessing/style/macro admission, reopening semantics and slash-role syntax remain explicit next work. Ordinary unsupported commands are not silently dropped.
- Most functions use compact typed composition, but sequence/UML parsers and drawing matches remain comparatively large; future extraction should preserve the tested contracts and not introduce catch-all success paths.

## Fix round 1 — 2026-10-23

Independent review: `/tmp/ive-plantuml-core-review.md`. Fix base: `b9815e81df754985a7bf4c1159d196251a5acb24`. Exact fix commit: **`4156f5bf238e9df2b53ed9ebff4c01d9a89ea7d3`**. All six accepted Important findings and the nearby actual-note-line defect are addressed in this scoped wave. This is implementation evidence for the controller's mandatory independent re-review, not a self-review approval. No new family, dependency, worker contract or audited vendor changes.

### Finding-to-fix and RED/GREEN evidence

1. **Sequence occupied bounds.** `sequence.rs` now allocates each anchor after reserving its event's extent above the anchor, then reserves its lower extent for the next event. The shared head-height calculation covers multiline created heads; message, self-call, note, frame-heading and lifecycle extents are explicit. Repro-derived private tests `should_reserve_multiline_message_occupied_height_before_its_anchor`, `should_reserve_multiline_note_occupied_height_after_a_message` and `should_reserve_created_multiline_head_before_the_next_message` were RED against the preceding allocation and are GREEN. They assert actual painted bounds, not only event identity.
2. **Compound caption/canvas containment.** `graph.rs` adds a measured, invisible caption reservation inside each checked compound parent and allows caption height in both rank/node separation. The reservation does not become an authored entity or painted shape. `Scene` retains negative minima; SVG canvas/background now include those coordinates. `should_contain_long_multiline_compound_captions_in_both_directions` and `should_include_negative_scene_coordinates_in_the_canvas` were RED, now GREEN. Both TB and LR captions are checked against their returned container bounds.
3. **Endpoint label space.** All middle/left/right relation labels are measured before graph construction. Their dimensions reserve layout space. Endpoint painting uses route tangent/normal, marker clearance and the owning bounds; a final outward clamp protects wide nodes on diagonal routes. Tests `should_place_endpoint_labels_outside_nodes_for_tb_lr_and_reverse_routes` and `should_keep_diagonal_endpoint_label_outside_a_wide_owning_node` cover forward/reverse TB/LR and the diagonal case. The missing-space/diagonal cases were observed RED before the corresponding fixes and are GREEN. Arbitrary multi-edge collision avoidance is still not claimed.
4. **Complete syntax consumption/name validation.** Declaration/lifecycle commands reject substantive colon suffixes instead of dropping them. Every endpoint branch, lifecycle name and note target uses the same quoted-aware name validation, including the right quoted-multiplicity branch. `should_reject_substantive_suffixes_and_validate_every_endpoint_branch` was RED for the retained suffix and slash-role counterexamples and is GREEN. Quoted colon note targets are handled through lexical splitting, not raw colon presence; `should_keep_colons_in_quoted_multiline_note_targets` was also RED then GREEN.
5. **Bounded rich-text recognition.** `text.rs` uses a linear lexical recognizer for known style/resource tags, case/alias variants, paired Creole delimiters and exact admitted tilde escapes. It rejects substantive unsupported markup in every label path, including implicit names, while preserving literal angles, URLs, unmatched punctuation and empty object-slot quotes. Normalized literal text is the same text measured and painted. `should_reject_rich_text_variants_in_every_label_bearing_path`, `should_preserve_literal_angles_urls_and_escaped_markup` and `should_keep_unpaired_punctuation_and_empty_object_slot_strings_literal` cover these contracts; the newly covered strike/variant and literal-preservation failures were observed RED then GREEN. This implements rejection/escaping, not rich-text rendering.
6. **Cooperative comment scanning.** The scanner keeps a constant-time non-whitespace-content flag rather than repeatedly trimming its growing output. Block-comment scanning is also incremental. Consumed work is charged/checkpointed every512 bytes (with at most one UTF-8 character of overshoot), with a final remainder charge. `should_charge_long_line_scanning_before_finishing_the_line` proves a small work budget fails within scanning. `should_bound_adversarial_comment_scanning_with_a_tiny_deadline` uses the retained128KiB hostile pattern,10ms deadline and200,000 work units; RED took5.635s in the regression run, GREEN completes below its500ms safety bound. This is regression evidence, not a benchmark guarantee.

The Minor note-source-location issue is fixed: `should_report_actual_multiline_note_statement_line` was RED and is GREEN. Note parsing now carries its actual statement line. **Precise token columns/byte spans remain unimplemented**: diagnostic columns are still1. Existing original five-fixture, style/cache, topology, nesting, checked-parent, inside-Dagre cancellation and limit tests remain covered.

### Final verification

All commands ran from `diagram-ports/`, with `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target` for Cargo. Full checks ran after the last production-code change; subsequent changes were README/report only. A final focused public/private run also passed immediately before commit.

| Command or suite | Observed result |
| --- | --- |
| `cargo test --offline --workspace` | exit0; **146 Rust tests**, plus **7 separate worker fault scenarios** |
| `cargo test --offline -p diagram-engine --lib --test plantuml` | exit0; **15 engine-private +31 public PlantUML tests** |
| `cargo clippy --offline --all-targets -- -D warnings` | exit0 |
| `cargo fmt --all -- --check` | exit0 |
| `cargo build --offline --release` | exit0 |
| `cargo deny --config ../deny.toml check licenses sources` | exit0; licenses and sources OK |
| `node tools/audit-plantuml.mjs` | exit0; **32 exact MIT source/ZIP entries,5 fixtures,12 modules** |
| `node tools/audit-merman.mjs` | exit0; **656 imported files,4 recorded patches**, unchanged |
| `node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs` | exit0; **6 guard tests** |
| `git diff --check`; `git diff --cached --check` | exit0 |

The146 Rust tests comprise engine-private15, compatibility11, Mermaid-family37, Mermaid/general21, PlantUML-public31, viewer-unit17, viewer-export5, native-lifetime3 and viewer-worker6. The7 fault scenarios and6 Node tests are not counted as Rust tests. Only the existing vendored Atomic::fetch_update deprecation warning remains; no vendor source was changed. No root IVE product changes or root IVE test claims.

Provenance adds eight selected authorized MIT references for actual style/token/escape behavior: FontStyle, legacy CommandCreoleBuilder, command Splitter/CommandCreoleStyle/CommandCreoleMonospaced/CommandCreoleSpace/CommandCreoleExposantChange, and utils/CharHidder. Each selected file was compared with its exact source-JAR ZIP entry and has behavior/destination/hash/notice mapping. The audit allowlist was extended narrowly for these paths. The source artifact/hash and local-not-external-authentication qualification above remain unchanged. No candidate or other-edition code/examples were read or imported.

### Refreshed native captures

Actual release viewer commands used `--language plantuml <file> --background --capture <png>`. All three app-frame PNGs were visually inspected; all autoquit. Sources and captures are under `/tmp/ive-plantuml-fix1.4tILBF`.

| Capture | Inspection/result | Native/fallback measurements | SHA-256 |
| --- | --- | --- | --- |
| `multiline-sequence.png` | Six-line message and note remain below the previous event; four-line created head clears the next message; frame contains the content. |23/0|`044f42bb32ba0207d788211969ce49b0d28c7027a6c21dbc82726a3b8290c43f`|
| `caption-roles-tb.png` | Long three-line caption inside package/canvas; source/target role labels outside class boxes. |8/0|`f000f12f18adf96050ff3fb93c91a6e66246329dc0757a10121aacfcbae5343e`|
| `caption-roles-lr.png` | Same caption/role containment with horizontal layout. |8/0|`fe51fbc01cc9dc067ce6e2b220687f746f203c928eadf7af54b5ef51ef94a112`|

Final `pgrep -a -x diagram-viewer` returned no process (exit1). No capture instance remains. The original unsupported-family error capture remains historical evidence; this wave did not broaden admitted families.

### Frozen contracts and residuals after fixes

Public render/options/error/worker interfaces are unchanged. Internal additions are `lexer::validate_name`, lexical literal normalization in Context, Scene minimum extents, sequence occupied-row allocation and measured RelationLabels before graph construction. Existing shared Budget/Context/Scene/SVG contracts remain reusable by the planned distinct next-family models. No generic language-neutral graph adapter was added.

Ordinary long/multiline captions and simple endpoint roles now have tested containment; this supersedes treating those basic failures as mere geometry-fidelity residuals. Remaining residuals are precise token spans/columns, exact upstream baseline/ink metrics, arbitrary complex edge-label/caption/label collision avoidance, Graphviz/Teoz pixel geometry, simplified symbols and all previously deferred family/preprocessing/style syntax. Caption reservations and event spacing are deliberately conservative and can leave extra whitespace. The fixed font/baseline policy, unsupported syntax ledger and full-port disclaimer remain in force. Independent scoped re-review is required before another implementation checkpoint.

## Fix round 2 — 2026-10-23

Scoped report: `/tmp/ive-plantuml-core-rereview-1.md`. Base **`4156f5bf238e9df2b53ed9ebff4c01d9a89ea7d3`**; exact fix commit **`d07df148ac7d82eb2ff33ba5a1ce35fbe9c03d4b`**. The reviewer marked findings1/2/3/4/6 and actual note line addressed, but correctly retained finding5: the URL fast path bypassed markup/escape recognition until whitespace. This also newly admitted URL-adjacent bold markup. This appendix supersedes the preceding broad claim that finding5 was completely addressed.

### Narrow correction and tests

Only `engine/src/plantuml/text.rs` and `engine/tests/plantuml.rs` changed. URL recognition now consumes the fixed scheme prefix, not the entire non-whitespace run. A local URL flag only preserves subsequent literal slash pairs until whitespace; tags, resources, non-slash paired delimiters and escapes continue through the same lexical scanner. Fixed-prefix checks and per-character advancement remain linear, with existing source/label bounds and budget contracts. The same normalized label still feeds measurement and painting; cache identity/lifetime and style policy are unchanged. No new grammar family, runtime I/O, crate, provenance source, vendor edit or shared layout change.

New regressions:

- `should_reject_url_adjacent_markup_in_every_label_bearing_path`:15 substantive tag/resource/delimiter variants across8 real label paths (message, declared participant, class member, named graph note, implicit sequence name, implicit graph name, quoted endpoint multiplicity, sequence note). Covers the retained exact strike and bold repros, upper/mixed-case aliases, font/color, image/sprite/icon/math, links and paired delimiters. Asserts `UnsupportedFeature`, not merely any error.
- `should_normalize_url_adjacent_escapes_without_changing_literal_paths`:8 literal/escaped controls across the same8 paths, including repeated URL slash pairs, query/fragment characters, unpaired path punctuation, HTTP/HTTPS/FTP/file schemes, literal angle text, escaped lower/uppercase tags, escaped sprite text and escaped paired styling markers. Assertions inspect actual SVG text against independently written expected literals.

Observed RED command: `cargo test --offline -p diagram-engine --test plantuml url_adjacent -- --nocapture`, exit101,2 failed. The first failed because `https://example.test/<s>strike</s>` was accepted; the second because `https://example.test/~<s>literal~</s>` retained escape characters. After the narrow fix, `cargo test --offline -p diagram-engine --lib --test plantuml` returned exit0 with15 private and33 public tests. Existing style/cache, source-scan budget, geometry and original fixture tests remain covered.

### Verification after the final code change

Cargo commands used `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target` from `diagram-ports/`:

| Command | Result |
| --- | --- |
| `cargo test --offline --workspace` | exit0; **148 Rust tests +7 separate worker fault scenarios** |
| `cargo clippy --offline --all-targets -- -D warnings` | exit0 |
| `cargo fmt --all -- --check` | exit0 |
| `cargo build --offline --release` | exit0 |
| `cargo deny --config ../deny.toml check licenses sources` | exit0; licenses/sources OK |
| `node tools/audit-plantuml.mjs` | exit0;32 source/ZIP entries,5 fixtures,12 modules |
| `node tools/audit-merman.mjs` | exit0;656 files,4 recorded patches |
| `node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs` | exit0;6 guard tests |
| `git diff --check`; `git diff --cached --check` | exit0 |

Rust counts: engine-private15, compatibility11, Mermaid-family37, Mermaid/general21, PlantUML-public33, viewer-unit17, export5, native-lifetime3, worker6. Separate fault scenarios and Node tests are not included in148. Only the unchanged vendored Atomic::fetch_update deprecation warning remains.

### Focused release/native evidence

Both real background app-frame captures were inspected under `/tmp/ive-plantuml-fix2.L4C8vR`; prior geometry captures were not rebuilt because geometry is unchanged:

- `safe-url.png`: literal repeated-slash/query URL plus escaped strike-tag text after a URL. Native measurements4, fallbacks0, exit0. SHA256 `b9595cc68d032b577a812e85226bb9cd1c9c22777fd3640a22886f1bbad6a178`.
- `url-style-error.png`: exact `/tmp/ive-rereview-url-style.puml` now displays unsupported rich-text error at line2/column1, expected exit1. SHA256 `16536cf9fb5b3217be3c7a75119a824a98662333e5dc0288b9a7a18a2aa2e8c6`.
- Exact retained `/tmp/ive-rereview-url-bold.puml` through release `--render` also returns exit1 with the same located unsupported-rich-text diagnostic, rather than normal escaped-bold text.

Each capture autoquit; final `pgrep -a -x diagram-viewer` produced no process output. No diagnostic instance remains. No full-port claim: precise token columns, broad rich-text rendering, deferred families, baseline approximation and complex-layout fidelity residuals remain exactly as previously recorded. Finding5's URL loophole is implemented and regression-tested; independent scoped re-review must determine acceptance before the next task.
