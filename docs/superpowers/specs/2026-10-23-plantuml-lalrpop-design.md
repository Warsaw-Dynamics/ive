# PlantUML LALRPOP parser rewrite

Date: 2026-10-23. Owner direction: “use LALRPOP for parsing plantuml. rewrite current adhoc parser.” This supersedes the earlier suggestion of a small future-family pilot. Execute under the existing local, subagent-driven workflow; new-family breadth waits for this migration.

## Intent and completion boundary

Replace the structural syntax authority for **every currently implemented PlantUML family and its shared syntax** with real LALRPOP productions. Preserve the typed semantic models, identities, layouts, text measurement, scenes, safe SVG, viewer and worker. Migration targets the existing supported subset, not all upstream PlantUML syntax. Existing unsupported capabilities remain explicitly rejected.

This includes document wrappers, admitted literal-definition declarations, preambles/styles/chrome, sequence, UML/description/ArchiMate, Chen, state, activity, mindmap/WBS, JSON, YAML, HCL, timing, Gantt, Salt, nwdiag, files, packetdiag, board and admitted Creole nesting. Do not exclude data/markup merely because their old parsers live in different modules.

Not completion: a generated `Line*` grammar forwarding raw commands to old parsers; a generated selector with handwritten family parsing underneath; production retry/fallback to a legacy parser; or a generator demo without actual family cutovers. During staged work, report migrated and pending components individually.

## Approaches

1. One universal grammar/model: rejected. These languages have incompatible raw-text, indentation, whitespace and semantic ownership rules; replacing all models would discard verified rendering work.
2. **Shared generated front matter and family-specific generated grammars, staged cutovers: selected by controller.** Preserve existing model/layout interfaces; use bounded lexical infrastructure and typed semantic builders. Remove the old syntax implementation when each component cuts over.
3. Generated line forwarding or permanent dual parsers: rejected. It leaves the requested ad-hoc syntax authority in place and makes success/failure ambiguous.

The owner selected LALRPOP and the rewrite; family decomposition, checked-in generation and token/budget organization are controller implementation choices. Costs are generation/dependency maintenance and possible syntax-parity rework; reviewable staged commits bound those risks.

## Baseline and invariants

Start from reviewed `4c197871caf992a1a852b7048311b4ee450d74ff`. Its Chen/ArchiMate fixes include lexer-aware relation precedence and atomic two-byte comment-delimiter consumption. Controller evidence: 488 ordinary Rust tests, seven separate worker-fault scenarios, strict Clippy/fmt/release/license/source checks, fifteen Node guards, and inspected native captures.

- Keep public `render`, options/result/error/control/measurement APIs unchanged. Keep the engine GPUI-free and caller-memory-only; no runtime files/network/environment/clock/locale/process/JVM/JS/browser/dot.
- Preserve worker isolation, native/fallback accounting, cache lifetime, zoom/raster separation and terminal resvg-safe SVG admission.
- Preserve defaults: source256KiB, SVG16MiB, model10,000, nesting64, work5,000,000, cooperative10s; host hard10s/2GiB Unix; labels16KiB, queries4096, finite coordinate magnitude1,000,000 and existing raster limits.
- Translate PlantUML semantics only from the authorized MIT1.2026.9 tree. Preserve exact source hashes, actual credits/grants, fixtures and Merman's audited 656-file/four-patch baseline. New parser infrastructure dependencies receive a separate permissive-source/license audit.
- Retain all positive and negative semantic/geometry/resource tests. No broad golden regeneration or relaxed rejection policy to hide migration drift.

## Architecture

```text
caller source
  -> bounded original-source lexical ownership + generated document/directive syntax
  -> existing eager immutable literal evaluator and expansion provenance
  -> bounded expanded-source lexical ownership + generated preamble/family syntax
  -> typed syntax values / semantic builders
  -> existing typed models -> existing layouts/text/scenes -> safe SVG/viewer
```

The two lexical views are intentional: current-row authored raw ownership precedes expansion, while an expanded note/action terminator determines the next row. Preserve those regression-tested rules. Comment delimiters consume both bytes atomically. Generated directives/wrappers remain denied; inserted literal values are never recursively rescanned as preprocessor programs.

### Generated syntax and retained non-syntax work

Generated productions own command shapes, token order, aliases, arrows, separators, complete nesting and EOF. Semantic builders receive typed declarations/relations/properties/conditions, never an entire raw command to resplit. They may intern identities, resolve scopes/references, validate values and construct existing models.

Custom lexical code may decode primitive escapes/numeric lexemes, emit significant newline/indent/dedent/delimiter tokens and protect explicitly owned raw text. A raw label/action/member/body token is legitimate only after lexical/generated-header ownership is established. It must not conceal a handwritten complete statement parser. Contextual keywords remain usable as names where the language admits them.

Family selection under `@startuml` is deterministic typed header/lexical evidence, not “try parsers until one accepts.” Preserve existing sequence/UML/state/activity/timing precedence and implicit sequence behavior with explicit ambiguity regressions. Wrapper-specific families select one parser directly. Each selected family parser consumes its complete body.

Shared syntax is not hidden legacy scope:

- Generated wrapper productions validate exact case rules, one document, paired ends, body and trailing input; lexical EOF checks also cover comments after closing wrappers.
- Generated definition syntax recognizes the supported same-line name/literal declaration. The existing bounded immutable evaluator, registry, capability policy and origin storage remain semantic machinery; full TIM is not added.
- Generated preamble/style productions recognize admitted skinparams, selectors, blocks, font/color/property and title/header/footer/legend structure. Keep typed property validation, effective style and `chrome::compose` layout.
- Generated Creole productions own admitted span nesting. Existing escape/URL awareness and paired-delimiter eligibility remain lexical disambiguation so unmatched symbols stay literal. Keep typed runs/coalescing/measurement; literal data families remain literal.
- JSON/YAML/HCL structural grammars are generated. Primitive JSON string decoding is allowed; old recursive value parsers are not. Preserve each family's unusual order, duplicate, scalar and physical-line policies.

## Files and interfaces

Add private infrastructure under `engine/src/plantuml/parser/`: orchestration, borrowed tokens, lexical modes, structured errors, checked token storage/arenas and grammar inputs. Checked-in generated Rust lives under `engine/src/plantuml/generated/`. Keep semantic builders beside their existing models at first; extract only where required for a clear boundary.

Use custom spanned token events shaped as `Result<(usize, Token<'s>, usize), ParseFailure>`. Locations are UTF-8 byte offsets into the expanded document, not token indices. A body view carries its source range/mode. Generated grammar context borrows the single mutable `Budget`; no new public parser API is required.

Prefer bounded borrowed-token materialization before generated parsing, then an iterator checking a cloned render control. This avoids simultaneous mutable budget borrows or global/interior-mutable budget sharing. Retaining actions are fallible and charge before allocation; arena handles or bounded list append avoid recursive AST duplication. Raw/header mode boundaries need explicit reviewed lexical/generated-header rules, not a second command parser.

Existing `parse(&[Line], &mut Budget) -> Model` facades may become adapters temporarily; the final internal body view should carry spans instead of rebuilding `Line` strings. No converted family may call its old source parser. Model/layout results stay the comparison contract.

### Family cutover inventory

| Syntax to replace | Model/layout to preserve | Required semantic boundary |
| --- | --- | --- |
| sequence declarations/messages/notes/frames/lifecycle | `Sequence`, existing sequence layout | source order, automatic participants, activation and frame ownership |
| UML/class/object/description/ArchiMate declarations/members/relations | `Uml`, graph/shapes | scoped identities, aliases, members, parent IDs, safe fills, relation direction |
| Chen owners/attributes/associations | `Chen`, chenscene | qualified lexical attribute owners, real undirected topology/cardinalities |
| state declarations/descriptions/transitions/closers | `State`, stategraph | forward references and scoped initial/final/composite ownership |
| modern activity recursive commands | `Block`/`Instruction`, activitytile | actual live/dead continuation, branches/loops/forks |
| mindmap/WBS markers/direction/aliases | tree models, treescene | independent sides/parent chains and buses |
| files paths/note blocks | `Files`, filesscene | ordered trie, duplicate files, sibling note anchors |
| board stage rows | `Board`, boardscene | nearest-lower ancestor, DFS columns, one root paint |
| JSON values/containers | `data::Value`, datascene | ordered duplicate members and exact bounded numeric spelling |
| PlantUML YAML block/flow forms | `data::Value`, datascene | string scalars, first-position replacement, indentation/literal blocks |
| HCL topblocks/fields/arrays/objects | HCL model/lowering, datascene | ordered duplicate fields, string scalars, header-line boundary |
| timing statements | `Timing`, timingscene | independent cursors, exact time arithmetic and duplicate policy |
| Gantt task/calendar statements | `Gantt`, ganttscene | civil dates, half-open bars, milestones and frozen references |
| Salt tables/rows/widgets/spans | `Salt`, saltscene | stable cell ownership/direct escapes/same-row colspan |
| nwdiag blocks/server/property statements | `NwDiagram`, nwscene | global servers versus memberships/groups, sparse buses |
| packet ranges/config/attributes | `Packet`, packetscene | inclusive widths, stable field IDs and conserved fragments |
| common arrows/names/style/definition/markup syntax | existing typed values/evaluator/runs | no hidden raw-string syntax reparsing |

## Accounting, cancellation and diagnostics

Charge scanned bytes including discarded trivia; check token/decoded-string/depth retention before append. Bound buffered tokens, generated syntax storage and semantic models under the caller's limits. Count syntax work explicitly rather than charging only after parsing. Preserve exact direct/expansion origin segments and no-rescan behavior.

LALRPOP owns internal state/symbol vectors; fallible semantic actions alone do not bound shifts or epsilon reductions. Before parser entry, establish a checked conservative stack/reduction bound from the bounded token stream and actual generated grammar. Include fixed stack allocation, prohibit nullable recursive cycles/recovery accumulation, and inspect generated code. Iterator/reduction/build checkpoints propagate typed cancellation/limits, never fake EOF. The first infrastructure review must see a demonstrated admission bound and cancellation interval, not a claim based only on the process watchdog.

Structured internal parse failures retain an offending expanded span and category; map through `origin::Document` before formatting the existing public error. Bound offending/expected-token output. No recovery-and-render of a partial model. Layout-only label errors may keep the existing honest bounded-candidate policy until exact metadata is available; never invent unique source precision from repeated text.

## Generator/dependency workflow

Existing Merman actually uses checked-in parsers generated by LALRPOP0.23.2; the workspace lock selects `lalrpop-util0.23.2` despite the vendor manifest's compatible0.23.1 requirement. Pin the new engine's direct utility to `=0.23.2`, default features off. Use `lalrpop=0.23.2` in a small developer-only generation crate with its own committed lockfile, outside the shipped engine/viewer dependency closure.

Generate checked-in Rust; ordinary locked/offline builds require no generator, Java, JS or external renderer. Provide explicit write-generation and read-only drift-check commands. Drift checking regenerates into a fresh bounded temporary mirror with stable logical paths and byte-compares outputs; it does not overwrite checked-in files. Record generator version/options, grammar inputs and generated hashes in a deterministic inventory. Fail on conflicts, missing outputs, version/options mismatch or drift. Never hand-edit/rustfmt generated bodies; lint allowances remain local to generated modules.

The runtime utility is cached; the generator is not. The first implementation task must acquire/audit its pinned development closure through the normal dependency workflow, inspect actual Configuration APIs, and prove fresh offline regeneration. No download or installation has happened during design preparation. Do not regenerate or modify Merman's audited sources.

## Delivery gates and sequence

1. Generation/accounting infrastructure plus immutable current-fixture baselines and a real production cutover of supported definition syntax; retain its existing evaluator/registry semantics.
2. Shared document/definition/ownership front end, then preamble/styles/chrome, with explicitly tracked adapters for not-yet-migrated families.
3. Sequence, UML-description/ArchiMate and Chen cutovers, each independently reviewable.
4. State and structured activity cutovers.
5. Outline-oriented mindmap/WBS/files/board grammars, with distinct semantics.
6. JSON/YAML/HCL generated structural grammars.
7. Timing/Gantt grammars.
8. Salt/nwdiag/packet grammars.
9. Creole nesting, final legacy-syntax removal and all-family audit.

The implementation plan may split these groups into smaller review gates; it may not drop components. A test-only frozen comparator is temporary and never a runtime fallback. Preserve exact source fixtures and deterministic model/SVG evidence; explain any difference rather than bulk-updating goldens. Add direct generated-parser RED tests, meaningful defect RED/GREEN, whole-input rejection and budget/cancellation tests before each cutover. Preserve real native tests/captures and cleanup.

The four previously deferred boundary defects move into their owning migration tests: timing keyword track names, Gantt unterminated trailing block comments, nwdiag spaced keyword-shaped server names, and Chen keyword aliases/spaced composites. Do not preserve those mistakes as compatibility. All other unsupported-subset boundaries remain.

Every increment gets focused tests, full strict workspace checks when shared code changes, generation drift/source/license audits, controller verification and fresh independent SPEC/QUALITY review. Keep source/text/geometry/negative controls, UTF-8/CRLF/BOM/expansion spans and pathological bounded inputs. Headless empty-PATH exports and genuine background native captures prove no runtime generator/external renderer and preserve actual native/fallback counts. Final structural audit classifies every remaining source parser; only genuine lexing, primitive decoding and semantic validation may stay handwritten.

After this migration gate, resume the remaining family inventory with LALRPOP-based parsing. The full upstream syntax/style/layout/asset and IVE integration backlog remains separately disclosed.

## Design evidence

Architecture preparation: `/tmp/ive-plantuml-lalrpop-migration-design-prep.md`, read completely by controller (one truncated display recovered with bounded ranges). It maps exact current entrypoints, preservation boundaries and generator/cache evidence. It was written before its agent's final capacity error; no duplicate reconnaissance is needed. The durable final evidence archive must retain it with temporary-path caveats.

No product parser migration or dependency installation is claimed by this document. Independent builder check and concrete task plan precede implementation under the owner's existing execution directive.
