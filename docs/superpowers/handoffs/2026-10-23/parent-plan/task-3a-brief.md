# Task 3A: Fresh MIT-only native foundation, sequence and UML graphs

Execute detailed Tasks 1–3 of `docs/superpowers/plans/2026-10-23-plantuml-native-plan.md` as one reviewed functional checkpoint. Read that complete plan and the parent design/plan. This brief does not authorize the later family stages yet. Implementation starts only after controller release and the Mermaid repair review gate.

## Global constraints

- Owner asks for a full-port attempt, additional permissive dependencies allowed, standalone pgui viewer first. Do not describe this checkpoint as a full PlantUML port.
- PlantUML translation sources are exclusively `/mnt/data/Projects/plantuml-mit/src`, MIT edition1.2026.9. No rejected candidate source/examples, GPL repository, Smetana/Graphviz/ELK implementation or conflicting third-party assets.
- Keep the engine GPUI-independent. No runtime files, network, environment/clock lookup, external renderer or process spawn in the engine. The host reads selected files and runs only its own bounded worker.
- Preserve the public render/options/result interface and current worker safety. Default source256KiB, SVG16MiB, model10,000, depth64, work5,000,000, cooperative10s; worker hard10s/2GiB Unix ceiling remains independent of callbacks.
- Exact existing-closure `merman-render =0.9.2` and `dugong =0.9.2` direct dependencies may be added, default features disabled as applicable. Resolve through existing workspace patches; retain no-math/no-ELK policy and notices.
- Use typed family models and native scenes, not PlantUML-to-Mermaid raw source substitutions. Preserve labels, identities, relations, nesting and source order. Reject every unsupported substantive construct; no blank or wrong-family success.
- Only one product implementer; no subagents or self-dispatched reviews. Use apply_patch, TDD, AGENTS coding/test conventions, existing target-cache path. Exclude all controller-owned dirty docs from scoped staging.

## Deliverable

1. Build the bounded located source/lexer/error/budget/text/style/scene/SVG service spine described in the detailed plan. Validate all limits; checkpoint scanning, measurement, layout and serialization. Use the public Merman render session and terminal resvg-safe pipeline plus existing resource admission. Bounded writer rejects excess before append; SVG values and geometry remain finite. No fabricated native baseline/ink provenance.
2. Implement substantive sequence semantics: ordered explicit/implicit participants and aliases, message direction/dash/extremities/self calls, real lifelines, notes, nested group/alt/else/loop frames, balanced activations and create/destroy. Unsupported anchors/parallel syntax and malformed stacks fail descriptively. Preserve event and lane topology in model/geometry tests.
3. Implement checked compound directed multigraph Dagre conversion with charged WorkControl. Use `try_set_unparented_parents_ix`, never unchecked cycle-asserting parent mutation. Test parallel/self/cyclic/disconnected edges, container bounds and interruption inside layout.
4. Implement class/object/description models and genuine shapes: compartments/member order/visibility, aliases, packages/namespaces, distinct inheritance/realization/association/dependency/aggregation/composition ends, multiplicities/roles; actor/usecase/component/database/deployment shapes and container ownership. Admit only actually drawn shapes/features; unsupported ports/constraints/icons get explicit errors. Description is the shared original family, not multiple marketing counts.
5. Add language-specific viewer examples and useful `.puml`/`.plantuml` selection without changing the isolated worker architecture or existing Mermaid behavior. Do not show unavailable examples as successful. Record actual native/fallback counts and baseline limitations.

## Test and acceptance evidence

Begin with the detailed plan's focused RED tests. Add public engine cases, private semantic/geometry assertions and negative boundary cases for comments/quoted text, wrappers/multiple blocks/pages, unknown commands/styles/directives, includes even inside unsupported macro bodies, Unicode/XML escaping, duplicate/scoped aliases and tiny source/model/depth/work/output limits. Check cancellation before/after native measurement and within Dagre.

Use exact freshly authored reference bytes under `/tmp/ive-plantuml-native-oracles/` for sequence/class/object/component-deployment/usecase. Original report and manifest are independently verified; they establish original admission, not native parity. Add meaningful multi-feature cases beyond those five and assert relationship ownership, not labels alone. Source manifest records exact authorized Java paths/hashes, Rust destination, translated behavior and notices. Retain full original MIT copyright/grant.

Run focused tests, then complete workspace tests/clippy/format/release, source-integrity audit and selected Cargo license/source checks. Existing Mermaid96-test baseline plus worker/audit scenarios must stay green. Capture real background native sequence and nested UML diagrams, inspect them, verify error UI and clean up processes. No IVE root checks needed unless its product is modified (not authorized here).

Commit only task3A product/source-provenance/fixture/doc files after checks. Write `.superpowers/sdd/2026-10-22-diagram-ports-plan/task-3a-report.md` with exact commits, supported/rejected constructs, shared API facts, source traces, RED/GREEN, commands/counts, measurements/captures, geometry gaps and next-stage contracts. Return commit/report and concise evidence. Fresh independent review is required before later implementation.
