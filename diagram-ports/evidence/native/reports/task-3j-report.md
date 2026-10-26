# Task 3J — native Chen ER and icon-free ArchiMate description

Status: implemented, checked and frozen for controller-arranged independent SPEC/QUALITY review. This is a bounded native subset, not complete Chen/ArchiMate syntax or original pixel parity.

- Released independently approved base: `3231bb195bbf183e5e961135f9889a9e872bd611`.
- Frozen head: `35c108daab82b1fa9064ce313ff82a5970d34478` — `Add native Chen ER and icon-free ArchiMate description`.
- Existing `/mnt/data/Projects/ive/.worktree/diagram-ports`, branch `diagram-ports`; standalone Rust workspace `diagram-ports/`. Exactly24 scoped product files committed, tracked tree clean afterward. No new worktree/branch, main IVE changes, dependency/lockfile/vendor changes, merge/push/publication, subagents or self-dispatched reviewers. Neither controller progress file was edited.
- Read root README/progress, applicable AGENTS, binding 3J brief/globals, complete prepared small-family semantic inventory and relevant frozen 3A/3D/3H/3I contracts including HCL FIX1. Inspected actual authorized translation sources personally. Used TDD, systematic diagnosis of the alias test failure and verification-before-completion. Task3J's explicit controller release overrides its historical HOLD; later inventory and unrelated deferred Minors remain untouched.
- Evidence directory: `/tmp/ive-plantuml-3j/`. This report is an ignored controller handoff artifact, outside the product commit.

## Frozen model, layout and API contracts

Public render/options/results/control/text-measurement APIs, worker protocol, watchdog, generation handling, per-job caches and terminal SVG admission remain unchanged. The engine stays GPUI-free and caller-memory-only, without new runtime file/network/environment/clock/locale/process/JVM/JS/browser/dot capability or imported assets. No raw PlantUML-to-Mermaid conversion or inflated ArchiMate backend exists.

| Internal contract | Responsibility |
| --- | --- |
| `chen::parse(&[Line], &mut Budget) -> Chen` | Dedicated frozen node/link arenas; parser-only lexical owner stack. `ChenNode::{Entity,Relationship,Attribute(AttributeOwner)}` and `AttributeOwner::{Entity,Relationship,Attribute}` carry actual kind/owner IDs. Each node retains distinct key/alias/display. |
| `Link::{Ownership,Association}` | Ownership links retain real owner and attribute IDs. Associations retain declared endpoint IDs, cardinality and `Participation::{Normal,Bold}`. No directed Chen arrow type is used. |
| `chenscene::layout(&Chen, &mut Context) -> Scene` | Dedicated measured rectangles/2:1 diamonds/ellipses, checked multigraph routes, true boundary intersections and measured center-cardinality space. No UML/class/table lowering or package parenting. |
| `graph::layout_checked(&mut Graph<NodeLabel,EdgeLabel,GraphLabel>, &mut Budget)` | Narrow extracted checked Dagre service below the language model boundary. Existing UML layout uses the same unchanged WorkControl/error propagation. Chen constructs its own graph directly. |
| `uml::Kind::Archimate`, `Fill::{Default,Authored(Color)}` | Real description model extension with explicit typed paint. Generic class/description mixing checks include ArchiMate. Existing entity, alias, package and relation semantics remain intact. |
| `lexer::model_tokens` | Reuses the bounded lexer while charging each Chen token before scanning/allocating it; bare token size is checked before string allocation. Existing `tokens` retains prior-family accounting/behavior. |
| `Wrapper::Chen`, source/preprocess | Exact lowercase Chen wrapper and family-owned raw rows; Chen performs its own normal comment stripping. Shared finite prefix styles/chrome/literal constants apply. Closing-wrapper comment balance is checked. |
| `Dash::Bold`, bounded SVG serializer | Actual2px participation stroke, without parallel/double lines. Prior Solid/Dashed output paths remain unchanged. `style::color` visibility broadens internally only, sharing the existing safe hex parser. |

Chen IDs are stable declaration-order indices (`c{id}`). Attribute key `C/address/city` is qualified by the lexical owner's complete key, so two `name` attributes under different owners remain distinct. A composite attribute is an oval plus separate child ovals and real links; it is never a package/container. Relationship-owned attributes use `AttributeOwner::Relationship`. Synthetic ownership links point owner→attribute for stable layout/metadata only; painting is undirected. Every Chen link lacks triangle/decorated heads.

Measured entities have real containing rectangles. Relationships use a2:1 diamond sized from measured text width plus twice text height and native margin, so all text-box corners lie within its interior. Attributes use original native ellipse fitting, with sufficient horizontal/vertical margin for the full measured label rectangle. Links project to the actual rectangle, diamond L1 boundary or ellipse equation, including self-loop endpoints. Dagre reserves measured space for center cardinalities; direct tests confirm each cardinality center lies on its actual association route and clears every node in the tested parallel/cyclic/disconnected/self cases. Native label backplates retain the shared paper/ink appearance. Theme, spacing, routes and backplates differ from original pixels.

ArchiMate paints a safe authored fill into an actual ordinary rectangle. `#LightBlue` resolves to`#add8e6`, `#LightGreen` to`#90ee90`, case-insensitively; exact hex uses the existing audited parser. Color does not become raw CSS or alter font cache identity. Native family remains`description`; declaration kind metadata is`archimate`. Authored directed description links retain owner/target IDs and actual heads. Existing package ownership around an ArchiMate leaf is tested; no ArchiMate package declaration is admitted.

## Admitted and rejected grammar

Chen admits exactly one complete nonempty`@startchen`/`@endchen` block. `entity ALIAS {` and `relationship ALIAS {`, optionally `"Display" as ALIAS`, require balanced separate owner braces. Attributes are identifier aliases or `"Display" as ALIAS`, optionally opening a composite owner brace. Names use Unicode letters/numbers plus underscore/dot; broader spaces/colon/type/hyphen aliases reject rather than being normalized or lost. Shared display escaping/finite Creole, comments, fonts/styles/chrome and prefix literal constants use existing services.

Associations outside owner braces resolve only already declared entity/relationship endpoints, with no implicit nodes. Canonical`A -1- R`, `A =N= R`, `A -(0,N)- R` and empty-cardinality`A -- R` are supported, with optional endpoint spacing and whitespace after the tuple comma. No whitespace is admitted immediately inside the participation markers. Matching markers are required;`=1-`/`-1=` reject instead of silently applying the original first-marker-only rule. Parallel links, cycles, self links and disconnected components are retained.

Chen deliberately rejects **all five semantic decorations**: weak, identifying, key, multi and derived. Nothing paints stereotype text in a plain shape and claims their semantics. d/o/U subtype junctions/links also explicitly reject. Per-node/link colors, nested entity declarations while an owner is active, relations inside an owner body, implicit endpoints, orphan/duplicate attributes, reopened declarations, unknown suffixes and unbalanced owners reject descriptively. Thus no decorated Chen shape is falsely reported as implemented.

ArchiMate admits a single-line`archimate #COLOR NAME` or existing display/alias forms inside`@startuml`; aliases must satisfy the existing nonempty identifier grammar. Exact`#RRGGBB` plus only the two named fixture colors are admitted. Missing/quoted/unknown/function/resource colors reject. Original broader names/colors/declaration reopening are not claimed. Existing description relation/group semantics and precedence are preserved, including a sequence endpoint literally named`archimate` and rejection of class/description mixing in either declaration order.

Every icon-bearing`<<...>>` grammar position is rejected, including display-before-as, alias-before-as and trailing positions. Sprite/openiconic/image/stdlib macro/include forms reject through the existing structural grammar/resource gates. ArchiMate package and multiline square declarations reject; multiline body/end semantics are not claimed. An invented`@startarchimate` wrapper rejects. No iconography, layer/category or architectural relation-validation rules are invented from colored rectangles.

All admitted statements are fully consumed. Unsupported suffixes/commands are errors, with no catch-all stripping. Existing coarse statement columns and late label-local diagnostics remain.

## Bounds and genuine native measurement

Shared defaults remain source256KiB/SVG16MiB/model10,000/depth64/work5,000,000/cooperative10s; independent worker hard10s/2GiB Unix,16KiB labels,4096 aggregate measurement queries,1,000,000 coordinate magnitude and existing raster caps are unchanged. Custom limits are validated before scanning.

Chen tokens, nodes, synthetic ownership links and associations consume model quotas. Composite depth is checked before retaining its node/link/owner-stack entry. Label/token bounds precede retained model strings; expanded owner keys are charged before formatting and again for retained metadata. Lookups charge conservative key/alias comparison work inside growing loops. Graph nodes/edges, route vertices, output shapes and text use the shared budget; cancellation checkpoints reach the actual Dagre kernel and surround measurement callbacks. There is no dense node-pair/grid allocation. Private controls inspect parser state after tiny token/depth/label failures and demonstrate expanded-key rejection before deep ownership amplification.

Public tests independently reach the actual bounded SVG writer with8192-byte limits for both extensions, rather than mislabeling an earlier text failure as output enforcement. Tiny model/work controls and pre-cancelled renders reject. Both families cancel after exactly one actual host measurement when the callback cancels the render. Final SVG passes the mandatory common resvg-safe resource/XML pipeline; representative outputs also parse in usvg.

The actual native-font test extends the established real GPUI TextSystem→HostTextMeasurer→SVG resolver path at6/14/96px, using installed Liberation Sans. Ink is checked inside each owning rectangle and inside the **true ellipse/diamond equation**, plus canvas containment. Chen has12 painted runs/11 actual native calls; ArchiMate6/6, at each size; all have zero deterministic fallbacks. Existing four inventory families remain covered by the same assertions. Worker lifetime coverage adds both new examples and verifies cache release, without changing worker services.

## Exact source, author and fixture evidence

Sole translation root: `/mnt/data/Projects/plantuml-mit/src`, MIT1.2026.9. Source artifact SHA256 remains locally observed`88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`, not an authenticated Maven publication. No other edition, rejected candidate, gen/h/Smetana/EPL/ELK implementation or untraced asset was read/imported for translation.

The fourteen new selections below are recorded with exact hashes, native destinations, translated/rejected behavior and actual notice lines in`diagram-ports/provenance/plantuml-mit-1.2026.9.json`. Every file retains`(C) Copyright 2009-2024, Arnaud Roques` and`Original Author:  Arnaud Roques`. LinkType additionally retains the exact`Contribution :  Hisashi Miyashita`. Existing full MIT grant/notices remain; current unmodified LICENSE-PlantUML-MIT.txt SHA256 is`d89942169c8d3dfbaa34ba911d801095142bf4a592e7667e651e458708b8364c`. Historical headers beneath the generated MIT grant are not erased/relabelled.

Paths below are relative to`net/sourceforge/plantuml/`; Rust module stems are under`engine/src/plantuml/`.

| Selected Java path | SHA256 | Native destination/behavior |
| --- | --- | --- |
| cheneer/ChenEerDiagram.java | `0b059002a1b59627343447dc77e362cd1c4130fe860cfa9c7c87c3e95a3ff932` | chen: lexical ownership |
| cheneer/command/CommandCreateEntity.java | `5dc5ecec3bb8dc4eb2c6060d50a231c93fdf18d4c0db15fbf142f2ae35e44485` | chen: mandatory braces, kinds/alias |
| cheneer/command/CommandCreateAttribute.java | `5132aa2ebed227a4797091ae7f92d30377629eaf041d5920f08fd4581eddec36` | chen: qualified attributes/composites/links |
| cheneer/command/CommandAssociate.java | `dd7535cb4900aca7738d380440472e50b39436d10c7f9ff430303680bd6162db` | chen/chenscene: undirected cardinality/participation |
| cheneer/command/CommandEndGroup.java | `a8112e6b39f38b37030e867dcbf5afed521fde86fa4b0b056ec8d339152ef59d` | chen: lexical close |
| svek/image/EntityImageChenEntity.java | `2b8dab51933e203d773e6df0e5e7c89b2dd099b2552c0de304a78013c042f7e0` | chenscene: rectangle; weak rejected |
| svek/image/EntityImageChenRelationship.java | `a0ebf31a21bde41913e0e6f38673c35d8a8845d7a78b6fc076f09d5515061b21` | chenscene: fitted2:1 diamond; identifying rejected |
| svek/image/EntityImageChenAttribute.java | `7a4c20e3e0d361bd71c135bbc1c3ea28f1e15d031a7100ec374160542e26da5e` | chenscene: oval; key/multi/derived rejected |
| descdiagram/command/CommandArchimate.java | `9c4e956d56b471342d01337eb2fe3440961deaed824defbe38c709e4fd5d4c8b` | uml: description alias/fill, no icons |
| decoration/LinkType.java | `1346a0a1efdd3545377eae89c02d20274bfc08391b3d6176c2e1325cf9cbbb12` | chen/chenscene: no heads/goBold |
| decoration/LinkStyle.java | `f34b53429547a46203b03b07c0dcab5eb26eb797110fe043bc1956d3626c407a` | svg: genuine2px bold |
| decoration/symbol/USymbols.java | `0f2ae92a208f717c539a403e485676762cf070fa6e7caa6b68f94bba9adb24c5` | uml/graph: ARCHIMATE rectangle identity |
| decoration/symbol/USymbolRectangle.java | `2e794e523a701cb54264906ba50a5461e3e8b93308d6c44ebb458bfaedd9d0a5` | graph: measured plain rectangle; native margins differ |
| klimt/color/ColorTrieNode.java | `c4869733e8fe82394b8e128f239d98670f27db57918dfc09f86f5bbc15366757` | uml: **only** LightBlue/LightGreen constants |

Existing core/DiagramType185–186 maps Chen; descdiagram/DescriptionDiagramFactory207–209 registers ArchiMate DESCRIPTION commands. Their existing entries receive precise behavior mappings; no extra sources selected. The source-audit allowlist adds only the exact selected Chen/image/decoration/color paths. Unselected subtype/circle/symbol/color-loader/asset/Smetana siblings remain denied. Controller independently matched all14 additions against authorized extracted bytes, exact JAR members, hashes, dated authors/contribution headers and MIT grants. It also independently byte-compared both base fixtures. Final source inventory220, fixtures43, native modules43; Mermaid656/four recorded patches unchanged.

Exact untouched original stems remain`/tmp/ive-plantuml-native-oracles/chen-entities-relationship` and`archimate-authentic-uml`. No new JVM/original-renderer probe was run. Existing metadata records original-rendered with no missing expected/exact labels; those are original-side facts, not new native pixel comparisons.

| Artifact | Chen SHA256 | ArchiMate SHA256 |
| --- | --- | --- |
| Original `.puml` / exact native base copy | `d689aebba3196e628ed7c84bc805b89cfb806985f3f9df1e07d7fb4f3f27f163` | `5b3771ce83c85f3542cfa9be8ca5e3e77db3d77da91201a750db2a8a54f0f271` |
| Original `.svg` | `72d708a47ef6570889d99f6353dc518e4ad9e0782295d2d4f8caf97e2c0d359f` | `40945c8373e130b39c2ab4c715d85d39d6612b7817c612818bbee7ca8a15d49e` |
| Original `.result.json` | `78fc76954e72debaeba8b7cf8d9910a465febe1f7ad99f11ffdebe378e627127` | `846d840d40ad6de03158b297fe7b2a607ee808c23a7e7cb97b24f94aff9d2462` |
| Authored viewer example | `5666cbec5b209f9fa8ce034d3ef5f89a802fc4da0a5e09b295c6bfc56f098986` | `73041e5113179ecb130544fdbc6e23719e227cb14d6e831b2b31b670f8a15d75` |
| New deterministic headless SVG | `0a07c6b7b78a7174f4c43904f74e9d08acb567d541385af3ecba39c3017a7351` | `606bee63b44e4c7660015304bee2deaa6130c244017648a353313be43c383127` |

The base Chen topology is2entities/1relationship/2attributes,2ownership links and2associations, with labels CUSTOMER/ORDER/PLACES/name/number/1/N. The base ArchiMate topology is2colored description rectangles and1directed authored reads relation. Original/native viewBoxes differ: Chen`0 0 237 540` versus`0 0 320.12503453452655 605.4631601080075`; ArchiMate`0 0 145 169` versus`0 0 216.17999999999998 316.4`. New SVG sizes4394/1871bytes. These preserve topology/labels/shape kinds and demonstrate differing native geometry, not pixel parity.

Both final release exports ran with PATH empty and DISPLAY/WAYLAND_DISPLAY unset, using explicit`--language plantuml --render fixtures/plantuml/{chenbase,archimatebase}.puml --output /tmp/ive-plantuml-3j/{chen,archimate}-native.svg`. All exit0; original artifact directories were not written.

## Meaningful RED/GREEN and final gates

All new tests use should_ names and given/when/then sections. Initial test-only public run before implementation: **3passed/3failed**, exit101 (`red.log`). Meaningful missing-feature failures were Chen wrapper/typed-shape admission in two tests and ArchiMate description/color admission in one. Initial implementation made6/6 GREEN (`green-initial.log`). These public tests already asserted actual shape tags, qualified IDs, ownership counts, cardinalities, undirected/no-head output, bold2px stroke, safe distinct colors and directed description owner/target facts.

Additional real RED/GREEN:

- Strict ArchiMate alias/color test:7passed/1failed (`red-alias.log`), revealing an empty quoted identity inherited from generic name grammar. The cause was generic quoted names bypassing identifier validation; ArchiMate now validates its actual alias and unquoted color at declaration admission. The remaining quoted-invalid/color controls execute green after that repair; they are not separately claimed independent REDs.
- Viewer example selection:0passed/1failed for unknown plantuml-chen (`red-examples.log`), then GREEN after backend-approved examples were registered.
- Exact source admission: new guard rejected ChenEerDiagram before its narrow allowlist extension (`red-audit.log`), then all final15 Node guards GREEN.
- Actual native paint initially reached ArchiMate successfully but failed a shared minimum7-run assertion: this authored example has exactly6runs. The threshold is now explicitly6 only for ArchiMate, retaining7 for previous families; this was a test-count correction, not a claimed product RED. Final actual-paint log records all sizes and true-shape assertions.

Later direct model/geometry/preallocation/cancellation/output controls passed against working implementation; not every added assertion independently demonstrated RED. Final new coverage is11public tests,7private Chen tests and1viewer example test, plus extended genuine font-paint and worker-cache-lifetime scenarios. No prior-family assertion was weakened; unrelated known Minors were not repaired.

Final commands used the standalone workspace and`CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`; locked/offline flags applied wherever supported:

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

All exit0. **474 ordinary Rust tests passed**, zero failed/ignored, plus **7 separately reported actual worker faults**: blocked stdin/stdout, stderr flood, oversized frame, early exit, memory and cancellation. Breakdown: engine-private109; compatibility11; Mermaid families37; Mermaid21; PlantUML33; new Chen/ArchiMate11; constants31; control23; Gantt16; inventory27; nwdiag16; Salt18; styles17; timing21; tree/data18; viewer-private51; export5; native-lifetime3; worker6. Fifteen Node guards pass. PlantUML220/43/43 and Mermaid656/four audits pass; deny reports licenses/sources OK. No new dependency.

Logs:`tests-final.log`, `clippy-final.log`, `fmt-final.log`, `release-final.log`, `deny-final.log`, `audit-plantuml-final.log`, `audit-merman-final.log`, `audit-tests-final.log`, `native-paint.log`. Final provenance-only wording correction to the existing DescriptionDiagramFactory mapping was followed by a fresh source audit and all Node guards. No code/test/example changed after full final gates or captures. The inherited audited vendored Atomic::fetch_update deprecation remains; strict product linting is clean. Scoped escalation addressed the known bwrap RTM_NEWADDR initialization failure only; no network operation or automatic approval rejection occurred.

Self-reviewed the complete24-file staged diff, including new parser/layout/tests: exact wrappers/whole-input consumption; owner identities and synthetic links; actual shapes/intersections/cardinality placement; description precedence, safe paint and icon rejection; budgets/cancellation/native cache behavior; precise sources/credits/fixtures; documentation and isolated runtime boundaries. Whitespace checks pass and the committed tracked tree is clean.

## Genuine background captures and process cleanup

Each final command used`/mnt/data/Projects/ive/target/release/diagram-viewer --background --language plantuml --example plantuml-FAMILY --capture /tmp/ive-plantuml-3j/FAMILY.png`. Both exit0, real1360×880 GPUI app frames, opened and visually inspected directly. Existing worker/native text service is used; no screenshot recreation, fixture padding or canvas expansion hides geometry.

| Capture | Actual native/fallback | Direct inspection | SHA256 |
| --- | --- | --- | --- |
| chen.png |11/0|Customer/Order rectangles, Places diamond, six separate attribute ovals, address→city/postcode composite links, relationship-owned since, scoped duplicate name, centered1/N, normal versus bold undirected participation; contained title/nodes/text|`f89e4cfda6f02bc6a543d880960be8a419bab485a9de894887c26bc8277028e1`|
| archimate.png |6/0|Blue Order service, green Customer record and hex amber Audit trail rectangles; reads/records directed links and heads; distinct real fill, contained labels/title; status family description|`5015e3a4db3b3139b8c84302e93365667da96d733d72f7d25b486060f17bf9cf`|

`capture-chen.log`/`capture-archimate.log` match actual status-bar native counts. No containment-expansion warning. Both apps self-quit. Final anchored pgrep for actual debug/release viewer executable paths found no processes (expected exit1). No IVE instance, xdotool, focus activation, mouse movement or user state/process interaction.

## Residuals and review handoff

Explicit subset boundaries above remain: all five Chen decorations and subtype semantics, broader attribute IDs/type syntax, colors/nested declarations/owner-body associations; ArchiMate icons/packages/multiline, all named colors except the two exact fixture constants, broader names and reopening. Existing baseline/font-fallback/source-column limitations remain. Native fixed margins, label backplates, Dagre routing and general complex label/edge collision avoidance differ from original styling/pixels; tested finite/nonoverlap cases do not establish universal original geometry. Some parser/test functions exceed preferred20statements while keeping grammar/model/layout responsibilities separate.

No known failing gate or unresolved new blocker. This frozen report is implementation evidence, not independent approval. Controller owns fresh independent SPEC/QUALITY review and any next release; all later families and unrelated deferred Minors remain held.

## FIX ROUND1 — relation dispatch and Chen comment ownership

Fix base: `35c108daab82b1fa9064ce313ff82a5970d34478`. Frozen fix head: `40b949a278fcc2747316c0cfd182b23add4685cb` (`Preserve relation and comment ownership in PlantUML graph parsing`). Exactly five product files changed,173 insertions/3 deletions: `engine/src/plantuml/{uml,preprocess}.rs` and `engine/tests/{plantuml,plantuml_chen_archimate,plantuml_constants}.rs`. Tracked tree is clean after commit. No controller progress files, source selection/manifest/notices, models/layout/scene/public APIs/dependencies/assets/worker services or later families changed. No subagents or self-dispatched reviewers.

Read the complete independent `/tmp/ive-plantuml-chen-archimate-3j-review.md` first. Its SPEC/QUALITY verdict was FAIL with two Important blockers, reproduced before production edits:

- **I1 — new declaration keyword breaks existing DESCRIPTION relations from an `archimate` node.**
- **I2 — Chen block comments execute literal definitions before Chen strips the comment.**

The original checkpoint's no-known-blocker statement above is historical and superseded by that review. **M1 — Chen keyword-shaped aliases/composite attributes have inconsistent dispatch** is Minor and explicitly deferred to4B/final triage by the controller; no Chen parser edit or M1 repair was made. Other deferred Minors and later families remain held. Systematic debugging, meaningful TDD and fresh verification governed this scoped repair.

### Causes and repairs

**I1:** `uml::Parser::line` dispatched a declaration whenever the first token's text mapped to a kind, before considering a relation. Adding Kind::Archimate therefore reinterpreted the valid existing `rectangle archimate` / `archimate --> B` relation as a declaration whose color was`-->`. The same failure affected a genuine colored declaration using alias`archimate`; role labels and quoted sources could not reach the relation parser. Family/source classifiers already avoided identifier-arrow misclassification, but this final boundary did not.

The UML parser now recognizes an **unquoted arrow token** and delegates the full statement to its existing relation grammar before declaration dispatch. Quoted display/role strings containing`-->` stay quoted tokens and cannot trigger that decision. The existing relation parser still validates complete endpoint/multiplicity/label/arrow grammar and resolves declared entities. There is no reserved-name rule, keyword blacklist or special-case alias string. A valid ArchiMate declaration still reaches color/display/alias admission. Class-member ownership remains ahead of this dispatch and unchanged.

**I2:** Chen had been added to preprocessor`raw_data`. This made definition/directive classification bypass normal comment stripping, although Chen's family parser later treated those same bytes as ordinary comments. Commented`!define A B` therefore entered the name registry before source normalization removed it, changing the actual entity identity/display toB. Commented includes incorrectly failed as active directives. No runtime include/file/network execution was demonstrated by the review or these checks.

Chen now uses normal comment-aware classification and is marked as a **graph** at its wrapper. The graph state prevents sequence-note/action raw ownership from taking over Chen rows without disabling ordinary comment/preamble/stylesheet ownership. Existing JSON/YAML/Salt/files/packet/board/HCL raw-data policy is unchanged.

A related revealing inline case proved why classification alone was insufficient: with active literal`END` equal to`'/`, the row`entity A /' END '/ {` previously expanded the name **inside the comment**, changing a delimiter and breaking the declaration. The existing CapabilityScan already tracks ordinary comment/quote activity for capability rejection. Its activity decision now gates literal-name replacement too. Inactive comment bytes remain direct original text; active quoted labels still expand, and definition literals/raw note/action/data payloads retain their existing explicit literal ownership. The original-source cursor, origin segments, retained-text/work bounds and no-rescan rule are unchanged. This is a lexical ownership repair, not deletion of the two reported strings.

Shared stylesheet ownership remains correct: an active definition inside a real Chen stylesheet rejects with its physical line; a`<style>` token authored only inside a block comment cannot put later genuine prefix definitions in stylesheet state. Active includes/TIM function requests remain denied immediately after a comment ends. No resource evaluator or new runtime capability was added.

### Meaningful RED/GREEN and covering names

Evidence directory: `/tmp/ive-plantuml-3j-fix1/`.

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline --no-fail-fast -p diagram-engine --test plantuml_chen_archimate --test plantuml --test plantuml_constants
```

The first targeted command lacked`--no-fail-fast` and stopped after the existing-description target's genuine regression:33passed/1failed, exit101 (`red.log`). The all-target unchanged-production run then reported76passed/7failed (`red-all.log`). One of those seven was an overly specific test diagnostic expectation: the existing expanded-stylesheet guard already rejected the attempted definition with`definition inside an expanded stylesheet`, while the new test expected`definition inside a stylesheet`. The control was corrected to assert a definition/stylesheet error at line4, preserving its actual contract; this is not claimed as a product defect.

The corrected unchanged-production baseline was **77passed/6failed**, exit101 (`red-corrected.log`):

| Covering test | Observed RED behavior |
| --- | --- |
| should_keep_existing_description_relations_from_an_archimate_alias | Previously valid DESCRIPTION rectangle alias rejected at line4 with`"-->"`; controls also cover quoted alias and endpoint role labels. |
| should_link_an_archimate_declaration_with_arrow_looking_quoted_display | Genuine colored alias relation rejected as`declaration colon suffix`; test verifies actual fill, owner/target IDs and exact arrow-looking display/role/relation text. |
| should_keep_chen_commented_definitions_inert_without_changing_identity | Successful render had identityB instead of exact expectedA. |
| should_accept_chen_comment_owned_directives_and_wrappers | Inert block-comment include rejected at line3. Test also owns function-looking, wrapper and stylesheet tokens inside that comment. |
| should_keep_chen_comments_inert_around_active_constants_and_shared_preamble | Commented ENTITY definition entered the registry, so the genuine active ENTITY definition failed as duplicate. Test verifies real Account identity and expanded Ready title/display afterward. |
| should_avoid_constant_expansion_inside_chen_inline_block_comments | Commented END expansion changed a delimiter and caused a Chen name/decorations error with expansion context. |

Two added controls passed before production edits after the diagnostic wording correction: `should_retain_chen_stylesheet_ownership_of_active_definitions` and `should_deny_active_chen_resources_after_comment_ownership_ends`. They are not independent RED claims. Previous raw note/action/data expansion, ordinary comments, quoted URLs/apostrophes, stylesheets, origin/cancellation/budget controls remain in the selected suites.

After the cause-level repair, the same focused command was **83passed/0failed**, exit0 (`green.log`): PlantUML34, Chen/ArchiMate12 and constants37. No prior assertion was removed or weakened. All eight new tests use should_ names with given/when/then sections.

### Fresh final checks and exact reviewer controls

After final formatting, ran the complete workspace suite once, then fresh strict Clippy/fmt/release/deny and all source/Node audits using the exact common commands from the checkpoint above. All Cargo commands used the shared target and locked/offline flags where supported. All gates exit0:

- **482 ordinary Rust tests passed**, zero failed/ignored; **seven actual worker-fault cases** pass separately. Only public PlantUML33→34, Chen/ArchiMate11→12 and constants31→37 changed; all other checkpoint counts remain unchanged.
- Fifteen Node audit guards pass. PlantUML220source/JAR matches/43fixtures/43modules, Merman656/four patches and deny licenses/sources remain green.
- Strict product Clippy/fmt/release are clean. The inherited audited vendored Atomic::fetch_update deprecation remains. No vendor/dependency/source selection changed.
- Complete five-file diff self-reviewed for lexical cause, full relation grammar, quoted-arrow safety, comment/definition/stylesheet ownership, active-resource denial, origin/budget invariants and scope. Working/staged whitespace checks pass; committed tracked tree clean. No product edit followed final checks/captures.

Logs: `tests-final.log`, `clippy-final.log`, `fmt-final.log`, `release-final.log`, `deny-final.log`, `audit-plantuml-final.log`, `audit-merman-final.log`, `audit-tests-final.log`. Scoped escalation addressed only the known bwrap RTM_NEWADDR setup failure, with no network operation or automatic approval rejection.

All four **exact unchanged reviewer inputs** now export with the final release binary, PATH empty and DISPLAY/WAYLAND_DISPLAY unset. Only new outputs under the fix evidence directory were written:

| Input after `/tmp/ive-3j-review-` | Actual final result | New SVG SHA256 |
| --- | --- | --- |
| description-regression.puml | exit0; two rectangle identities archimate/B, relation owner n0→target n1 | `f01a398af30d156bcd78255822c8258cfbc6c51fee592ba6983788ccd6a2853d` |
| archimate-endpoint.puml | exit0; actual ArchiMate identity archimate, fill`#123456`, rectangleB, relation n0→n1 | `74540f6295774fc0c638be8abe1a8c3c7a08e402aaa2c81141ea38d47bb78381` |
| chen-comment-definition.puml | exit0; sole Chen entity identity/textA | `1b298dc6a1b770048db12d7b1a71d78aaabc9f466641c3fdbc934fe63bfc4677` |
| chen-comment-include.puml | exit0; same sole identity/textA and byte-identical SVG to commented-definition control | `1b298dc6a1b770048db12d7b1a71d78aaabc9f466641c3fdbc934fe63bfc4677` |

Original review sources/SVGs and original renderer artifacts were not modified. Active-resource denial and genuine prefix constants are separately exercised by the exact semantic regression controls above. No new original/JVM renderer probe or pixel-equivalence claim was made.

### Genuine repaired-case captures and residuals

Refreshed only the cases needed to demonstrate the repaired boundaries. Commands used the actual release viewer with`--background --language plantuml --capture OUTPUT.png INPUT.puml`. Both exit0, real1360×880 GPUI frames, opened and directly inspected:

| New capture under `/tmp/ive-plantuml-3j-fix1/` | Actual native/fallback | Inspection | SHA256 |
| --- | --- | --- | --- |
| description-regression.png |2/0|Source shows the exact old rectangle grammar; both archimate/B labels are contained and the authored downward directed head reachesB. Status family description. | `fedc9ccc24242f4e02158e09130f1b6bab6a769c089bafe4aaab88b641109bec` |
| chen-comment-definition.png |1/0|Source visibly contains the commented definition; the actual single rectangle paintsA, preserving its semantic identity. Status family chen. | `ad20be8aa330aa4fc7f3015ba8e300446067e44a5eac2a726c4f0358d404be9b` |

Capture logs match actual status-bar counts, with no containment-expansion warning. Both apps self-quit; anchored lookup for actual debug/release viewer executables found no surviving processes (expected exit1). No IVE instance, xdotool, focus activation, mouse movement or user-state/process interaction. Original built-in captures above remain checkpoint evidence and are not relabelled fresh; the fresh full native-paint/worker-lifetime tests still cover their unchanged shapes/colors/cache behavior.

I1/I2 are repaired in code/tests; controller-arranged scoped independent re-review remains required. M1 is explicitly still deferred, and all prior grammar/fidelity/font/diagnostic residuals remain. No later family was released by this fix.

## FIX ROUND2 — atomic shared comment-delimiter consumption

Fix base: `40b949a278fcc2747316c0cfd182b23add4685cb`. Frozen fix head: `4c197871caf992a1a852b7048311b4ee450d74ff` (`Consume PlantUML comment delimiters atomically during expansion`). Exactly two product files changed,123 insertions/8 deletions: `engine/src/plantuml/preprocess.rs` and `engine/tests/plantuml_constants.rs`. Tracked tree clean after commit. No controller progress edits, family grammar work, new family, dependency/source selection expansion, asset/model/layout/public API/worker change, subagent or self-dispatched reviewer.

Read the complete `/tmp/ive-plantuml-chen-archimate-3j-rereview-1.md` first. It adjudicated I1 and the original I2 as **ADDRESSED**, but returned SPEC/QUALITY FAIL for one new Important, verbatim: **“N1 — replacement gating can reuse a consumed comment-delimiter byte and suppress an active constant.”** The FIX ROUND1 completion evidence does not override that gate. M1 and all other deferred family Minors remain untouched. The user's new LALRPOP migration requirement is acknowledged: this repair is solely shared lexical/preprocessor comment ownership needed independently of generated parsers; no migration implementation or handwritten family grammar change was made.

### Confirmed cause and exact repair

The shared CapabilityScan previously returned a boolean activity flag. At a closing`'/`, it cleared its comment state at the apostrophe, but the expander then advanced one character. The already consumed closing slash could combine with the following literal apostrophe to form a phantom`/'` opener on the next iteration. The authoritative comment grammar consumes the real delimiter's two bytes together. Once FIX ROUND1 used this activity state to gate literal replacement, the phantom comment suppressed a genuinely activeX after the closed comment, silently painting`start ' X` instead of`start ' Ready` in both Chen title and existing sequence message grammar.

The same consumption mismatch existed at an opening delimiter: its apostrophe could be reused with an immediately following slash as a phantom closer. A revealing paired test put a delimiter-valued constant inside that actual comment; incorrect activity expanded it and painted extra`'/` text. This was repaired at the same lexical boundary, not by special casing either reproduction.

CapabilityScan now returns a typed`ScanStep::{Active,Comment,Delimiter(bytes)}`. Both real opener and closer transition comment state and return their complete two-byte consumption. The expander charges those bytes, advances the original-source cursor by that count and continues **before** another activity/word scan. Thus no delimiter byte can participate in an overlapping delimiter. Normal inactive comment words remain unexpanded; the comment gate is retained.

Explicit quoted/raw/definition ownership remains ahead of this scanner and yields Active as before. Definitions still freeze eager original-cursor insertions, raw note/action/data payload policy remains unchanged, active quoted labels still expand, and inserted values are not rescanned. Direct comment bytes remain original text in the expansion document and are normalized only through the established family/source path. Existing append/insert/origin storage, source/label/model/work/output bounds, cancellation checks and runtime restrictions are unchanged. No formerly valid input is reserved/rejected to hide N1.

### Meaningful RED/GREEN and exact invariants

Evidence directory: `/tmp/ive-plantuml-3j-fix2/`.

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --test plantuml_constants
```

Before any production edit, **39passed/3failed**, exit101 (`red.log`). The three actual semantic RED tests were:

- `should_expand_active_constant_after_closed_comment_and_literal_apostrophe_in_chen_title`: exact reviewer title reproduction painted`start ' X`; expected exact`start ' Ready` and entityA.
- `should_expand_active_constant_after_closed_comment_and_literal_apostrophe_in_message`: exact existing-message reproduction painted`start ' X`; expected exact`start ' Ready`.
- `should_consume_comment_opener_before_recognizing_any_closing_delimiter`: paired opener case painted`start  '/ Ready`; expected exact`start  Ready` while keeping entityA.

Two new preservation controls already passed against unchanged production and are not claimed independent REDs:

- `should_preserve_separated_comment_delimiter_and_apostrophe_spacing` requires the exact spaced control`start  ' Ready`.
- `should_keep_comment_looking_bytes_active_in_explicit_literal_ownership` covers an active quoted participant display, raw note body and frozen definition literal; each must retain the literal comment-looking bytes while replacingX withReady.

After the typed scan-step/cursor repair, the same target was **42passed/0failed**, exit0 (`green.log`). No diagnostic-expectation correction, removed assertion or reduced stress case was needed in this round.

Added the direct private invariant `should_keep_consumed_comment_bytes_direct_and_expand_only_the_active_original_span`. It asserts the full expansion document with commentedX unchanged and only the trailing activeX replaced; every retained Direct segment byte-compares to its original source range, exactly one Expansion segment identifies the actual original activeX byte/line/definition ID, and that expansion containsReady. This control was added against the working repair and is not an independent RED claim.

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --lib plantuml::preprocess
```

Result **8passed/0failed**, exit0 (`origin-bounds.log`), including the new exact-byte/span control and all previous retained-text/origin/lineage/model/source limits, cancellation-before-retention and original-cursor/no-rescan tests. All six new tests use should_ names and given/when/then sections.

### Final strict checks, exact reviewer exports and genuine capture

After final formatting, ran the complete workspace suite once and fresh strict Clippy/fmt/release/deny/source/vendor/Node audit gates with the common commands above, shared Cargo target and locked/offline flags where supported. All exit0. **488 ordinary Rust tests pass**, zero failed/ignored, plus **seven separately counted actual worker faults**. Only engine-private109→110 and constants37→42 changed from FIX ROUND1; all other suite counts remain unchanged. Fifteen Node guards, PlantUML220source/JAR matches/43fixtures/43modules, unchanged Merman656/four patches and deny licenses/sources pass. No source matching refresh was required because no selection/hash/notice changed.

Logs: `tests-final.log`, `clippy-final.log`, `fmt-final.log`, `release-final.log`, `deny-final.log`, `audit-plantuml-final.log`, `audit-merman-final.log`, `audit-tests-final.log`. The inherited audited vendor Atomic::fetch_update deprecation remains; strict product linting is clean. Scoped escalation addressed only the known bwrap RTM_NEWADDR setup failure, with no network operation or automatic approval rejection.

The final release executable exported all three **exact unchanged reviewer sources** with empty PATH and DISPLAY/WAYLAND_DISPLAY unset, writing only into the new fix evidence directory. Every command exits0, and direct XML readback confirms the required exact text:

| Source after `/tmp/ive-3j-fix1-review-` | Final painted text | New SVG bytes / SHA256 |
| --- | --- | --- |
| chen-comment-overlap.puml | entityA, title`start ' Ready` |1008 / `cada7fd43e65ecbc8fa4fed62f0dae9a43b659eb58a29d80cd2b2213fd62f0dc` |
| comment-overlap.puml | participantsA/B, message`start ' Ready` |2243 / `312a93975839c6889259a0cb9d5ba4f7ea4e5c8239c9d09d62551ac3b48996c3` |
| comment-separated.puml | participantsA/B, exact message`start  ' Ready` |2233 / `da628490d22d4a0a6783791b04e3b5191bc2828bb568d1ba42831a9a038eac33` |

Original reviewer inputs/outputs and original-renderer artifacts were not written. No original/JVM renderer or broader parity claim is introduced.

Fresh native capture:

```sh
/mnt/data/Projects/ive/target/release/diagram-viewer --background --language plantuml --capture /tmp/ive-plantuml-3j-fix2/chen-comment-overlap.png /tmp/ive-3j-fix1-review-chen-comment-overlap.puml
```

Exit0, genuine1360×880 GPUI app frame, opened and visually inspected directly. Source visibly shows the original boundary case; the actual title paints`start ' Ready` above the containing entityA rectangle. Status bar and `capture-chen.log` report **2actual native measurements/0fallback**. SHA256 **`6099ebfa17146b89005404e5d9c1f04a9b73f5416744395d69c45ad2e6dc9c17`**. No containment-expansion warning. Viewer self-quit; anchored debug/release viewer lookup found no surviving process (expected exit1). No IVE instance, xdotool, focus activation, real mouse movement or user-state/process interaction.

Self-reviewed the complete two-file diff for atomic delimiter/cursor coupling in both directions, exact original spans, quoted/raw/frozen ownership, no-rescan/work/cancellation, preservation of FIX ROUND1 controls and strict scope. Working/staged whitespace checks pass. No product change followed final gates/capture; committed tracked tree is clean.

N1 is repaired in code/tests; controller-arranged scoped re-review still owns approval. Prior I1/I2 coverage remains green. M1 and other family Minors stay deferred, and no later family is released. The separately planned LALRPOP migration remains a distinct task with no product changes from this repair.
