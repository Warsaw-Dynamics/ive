# puml v0.2.6: implementation-lineage follow-up

Date: 2026-10-23. Candidate: `alliecatowo/puml`, tag `v0.2.6`, commit **`c23103ec2d89778c63a625392be61ebe761041c3`**. Local checkout: `/tmp/ive-puml-audit.V9qwLm/puml`.

This is static source-risk evidence, not legal advice, a finding of infringement, or certification of every historical derivation. No upstream GPL PlantUML repository or EPL implementation was opened. No network was used. No product code was edited. The review ignored untracked agent executables and used `git show HEAD:<path>` / `git grep ... HEAD` for exact tracked sources.

## Adoption ruling

**Hold adoption of the current puml-derived implementation under the project's MIT-only PlantUML-source policy.** The important new evidence is an explicit source-based style-parser/catalogue lineage documented by puml itself, tied to `github.com/plantuml/plantuml` master, without a recorded MIT-edition or separate permissive grant for that source. IO/sprite hardening does not resolve this separate provenance issue.

This is more specific than the earlier audit's general “no verified MIT-only ledger” concern. It supersedes that earlier provenance assessment. The evidence does not demonstrate that all puml code is impermissible, and does not establish copied protected expression. It does identify an implementation surface that cannot currently be accepted as verified MIT-origin work under the owner's stated rule.

The graph-layout modules reviewed did **not** reveal a concrete Graphviz/Smetana/ELK source translation. Similar algorithm names and Graphviz-like output descriptions must not be converted into a copying allegation.

## Strongest evidence: style implementation, not just an oracle

All paths below refer to the pinned puml revision. Primary evidence links are pinned to that revision.

1. [Upstream Source Audit, lines 5–7](https://github.com/alliecatowo/puml/blob/c23103ec2d89778c63a625392be61ebe761041c3/docs/internal/forensics/2026-05-31-plantuml-upstream-source-audit.md#L5) identifies `github.com/plantuml/plantuml` master `6c308a2`, version `1.2026.6beta1`, cloned to `/tmp/plantuml-src`. Lines 74–87 record the clone command and explain that the upstream tree was not committed for size/licensing/drift reasons. Lines 89–102 describe walking Java command implementations and cross-referencing Rust parser/render modules.

2. That source audit was explicitly research-only: lines 104–110 say it made no source modifications and ran neither builds nor fixtures. Therefore the family map in that document alone is **evidence of source consultation and gap analysis**, not evidence that every mapped renderer was copied or translated.

3. A subsequent [style-cascade design](https://github.com/alliecatowo/puml/blob/c23103ec2d89778c63a625392be61ebe761041c3/docs/internal/design/1404-style-cascade-design.md#L7) names the same source revision at line 7 and lines 99–103. It is an implementation plan, not merely an observation report. At lines 324–325 it states that `SName` is mirrored as a Rust enum “1:1” from `/tmp/plantuml-src/.../style/SName.java`. Lines 334–338 describe a new hand-written parser modeled directly on `/tmp/plantuml-src/.../style/parser/StyleParser.java`.

4. Current candidate headers corroborate the planned lineage:

| Existing puml module | Exact signal |
| --- | --- |
| [src/ast/style.rs:11](https://github.com/alliecatowo/puml/blob/c23103ec2d89778c63a625392be61ebe761041c3/src/ast/style.rs#L11), :14 | Selector catalogue mirrors upstream `SName.java`. |
| src/ast/style.rs:354, :357 | Property catalogue mirrors upstream `PName.java`. |
| [src/parser/style_block.rs:3](https://github.com/alliecatowo/puml/blob/c23103ec2d89778c63a625392be61ebe761041c3/src/parser/style_block.rs#L3), :218 | Parser modeled on `StyleParser.java`; string-reading behavior mirrors its `readString`. |
| [src/theme/style_builder.rs:6](https://github.com/alliecatowo/puml/blob/c23103ec2d89778c63a625392be61ebe761041c3/src/theme/style_builder.rs#L6) | Specificity scoring matches `StyleParser.java`; header also references upstream source-order behavior. |

This is a precise implementation-origin concern. Public selector/property names or matching behavior may have different legal treatment from protected source expression; this review does not decide that issue. The project rule requires permissible translation sources, and these records do not identify such a source.

### Related local integration surface

The design's own decomposition table at lines 528–532 identifies the proposed integration points: `src/parser/directives.rs`, AST style types, `src/theme/shared_cascade.rs`, `src/theme/cascade.rs`, `src/theme/effective.rs`, `src/normalize/family/directives.rs`, and family render style consumers. Current names include `src/ast/style.rs`, `src/parser/style_block.rs`, and `src/theme/style_builder.rs` rather than every proposed filename.

Those are review/dependency-tracing targets, **not a finding that each file contains copied expression**. Deleting only the design document, hiding the `<style>` UI, or disabling `<style>` syntax while retaining the same compiled implementation does not establish a permissible source origin. If replacing the flagged surface, trace actual imports and retained data rather than relying only on the old planning table.

## Graph-layout review: no concrete forbidden-engine import found

Reviewed puml implementation code and module contracts show a compact, customized hierarchical engine:

- `src/render/graph_layout/mod.rs:149` coordinates rank assignment, 12 barycenter sweeps, coordinates, group bounds, orthogonal channel routing and a local render scene.
- `rank.rs` implements DFS back-edge detection, Kahn topological ordering and longest-path ranks using BTree collections; it includes a puml-specific group-cohesion rank adjustment. No network-simplex or Smetana generated-code structure was identified.
- `crossing.rs` uses barycenter ordering, adjacent swaps and merge-sort inversion counting. These are ordinary algorithmic techniques; naming them is not evidence of a particular source translation.
- `coordinates.rs` centers ranks, wraps wide grouped rows, shifts overlapping groups and recomputes canvas bounds. `groups.rs` derives enclosing boxes from local node positions.
- `router.rs` and `router/contract.rs` define local channel tracks and route ownership. Contract comments explicitly distinguish their rounded corners from Graphviz B-splines.
- `spline_router.rs` describes anchor/tangent cubic construction and a maximum two-segment obstacle detour, explicitly deferring the full polygonal-channel spline algorithm. Its “Graphviz-like” language and handle-fraction comments do not identify copied source.
- `src/render/edge_smoothing.rs` documents and implements local rounded-corner SVG generation. Representative `family/class_layout.rs`, `family/box_grid.rs`, state layering, activity frame layout, sequence creation, scene and validation code use puml-owned structures and issue-specific geometry adjustments.

These observations are consistent with the candidate's claimed custom engine; they are not proof of independent authorship. No Graphviz/Smetana/ELK source files, generated Java/C binding vocabulary, separate engine license header, or explicit “translated from Graphviz/ELK” claim was identified in the reviewed implementation paths.

One weaker risk signal exists in [layout-engine-vision.md:122–127](https://github.com/alliecatowo/puml/blob/c23103ec2d89778c63a625392be61ebe761041c3/docs/internal/architecture/layout-engine-vision.md#L122): its reading list includes papers, Dagre source, Graphviz `lib/dotgen/` and layout-rs. Earlier text proposes either adopting a Rust crate or writing a minimal engine. A recommended source link alone does not demonstrate that it was read or copied. Do not conflate this with the much stronger style-specific implementation statements above.

## Broader source-audit mappings

The candidate's own source audit maps Java implementations to local families at lines 121–140: class/object to family parsers and class renderers; descdiagram to component/deployment/usecase; sequence/activity/state to their parser/render modules; mindmap/WBS/Gantt/timing to corresponding normalizers/layout/renderers; and specialized nwdiag/Archimate/Chen/Salt/JSON/YAML/EBNF/regex/SDL/board/files/wire modules. It mentions `SmetanaForJson.java` only in that **upstream family map**; no local Smetana port was identified.

The map establishes broad upstream implementation-source consultation. Its research-only methodology does not justify calling every mapped module a translation. The style design and matching current headers supply the additional implementation-origin evidence for that narrower surface.

## Licenses, history and limits of this review

Root `LICENSE` is MIT with an unnamed 2026 copyright holder. `THIRD_PARTY_NOTICES.md` records MIT Open Iconic and Bootstrap Icons, and Apache-2.0 Google Material icons. No recorded PlantUML MIT-edition/relicensed origin, extra source permission, or style-parser derivative notice was found in those files or the specific provenance/design/source-comment records reviewed. Root licensing alone does not answer those origin questions.

The clone is shallow: `git rev-parse --is-shallow-repository` is true and `git rev-list --count --all` is **1**. Only release commit `c23103e` is locally available. Earlier commits/PRs named inside documents cannot be independently inspected from this local history. No attempt was made to fetch more history, open cited upstream sources, perform code-similarity comparisons, or certify the historical development process. Dependency closure was intentionally not re-audited.

Files reviewed include the exact root license/notices/manifests; relevant graph module bodies and representative family modules listed above; `docs/internal/architecture/{layout-engine-vision,edge-routing,renderer-refactor-roadmap}.md`; source-audit methodology and mapping tables; style-design provenance and implementation-plan prose; candidate style module origin comments; and relevant candidate AGENTS/CLAUDE/contributing references. Embedded upstream implementation snippets were not opened or compared. The reading was targeted, not a line-by-line audit of the whole library.

## What would clear the adoption hold

For the flagged style surface, obtain documented rights/provenance identifying an allowed MIT edition or other permissive grant for the actual source used, or replace it from permissible sources/specifications with a recorded module/data origin. Do not represent arbitrary refactoring as removing derivation. Broader source-audit mappings remain review leads rather than proven taint; assess them proportionately instead of discarding all graph code based on algorithm resemblance.

Authorized fresh-implementation references located by filename are `/mnt/data/Projects/plantuml-mit/src/net/sourceforge/plantuml/style/SName.java`, `PName.java`, and `StyleBuilder.java`. The same filename check did not locate `CommandStyleMultilinesCSS.java`, `StyleSignatureBasic.java`, or `style/parser/StyleParser.java` in that artifact; do not assume every master-source class exists in this MIT edition. Availability of the three references does not retroactively establish a permissible origin for puml's existing implementation. Use the authorized artifact as the explicit source of new work, with its notices and module traces.

For now, preserve the experimental fork work, avoid integrating or redistributing the unresolved candidate as accepted permissive-origin PlantUML code, and continue independent viewer/Mermaid work. This ruling follows the project's stated source-license constraint and the concrete evidence above, not a claim of legal certainty.
