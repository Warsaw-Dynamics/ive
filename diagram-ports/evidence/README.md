# Historical diagram-port evidence archive

This archive preserves completed implementation reports, the available historical independent review documents, and the preparatory research required by Task4B. Its branch starts at `1b0c1edadadefe76676206340dedf477b6067cf0`. It does **not** provide final advanced-state, corpus, coverage, native capture or whole-branch review acceptance.

Start with the [catalog](catalog.md), [recorded review verdict index](verdict-index.md), and [machine-readable manifest](manifest.json). The manifest maps every old source path to its durable copy, records source and archived SHA-256/byte counts, and preserves exact reported base/head lines and verdict excerpts. The report text itself remains the authority for those historical claims; extracted identifiers are not a newly approved product-head list.

## Evidence kinds

| Kind | Preserved payloads | What the evidence establishes |
| --- | ---: | --- |
| Native/foundation implementation and integration reports | 25 | What implementers/controllers reported at their recorded checkpoints, including fixes and later appendices. |
| LALRPOP implementation and integration reports | 23 | Reported generated production cutovers, tests, resource witnesses and removed legacy authorities. |
| Completed developer tooling reports | 3 | Scoped import/oracle and native-corpus/deadline work; not completion of all Task4B. |
| Historical independent review documents | 44 | Their actual scoped verdicts and findings, including earlier needs-fixes rounds and later rereviews. |
| Required preparatory research and accompanying source ledgers | 12 | Original/source semantics, architectural preparation and historical oracle work; not current native support. |
| Historical controller ledgers | 2 | Exact old events/rulings, including stale checkboxes and superseded orchestration notes. |
| Generator/provenance snapshots | 5 | Exact dependency/notice/admission/residual/generated-hash metadata at this archive's product base. |

The **114 payloads contain 1,844,096 bytes**. New indexes, manifest and checksum file are separate archive metadata. All copied payloads are byte-identical to the sources captured for this archive. That independent preservation check does not mean this job reran the historical tests, source audits, model proofs or review gates.

An implementation report is **reported evidence** even when it describes independent controller checks. A review document is **historical independent review evidence**, limited to its stated range and scope. A later family with a tested checkpoint and deferred broad review is not upgraded to independent approval. Read the final scoped rereview together with its earlier findings and implementation fixes; do not interpret an old needs-fixes review as a new finding against the current branch.

## Historical paths and provenance

Payloads retain their original bytes, including `/tmp`, machine-specific worktree paths, earlier source locations, commands, line references and pending-at-the-time wording. Those paths are **historical references**, may be unavailable on another machine, and are never runtime inputs. The durable links in this directory and `manifest.json` are the recovery map. Ignored SDD sources have not been removed or changed.

The archive base identifies this archive checkout and the copied product-provenance snapshot. Ignored reports and temporary research were captured as files, not falsely claimed to be tracked blobs in that base. Their exact captured identity is their recorded source SHA-256. Existing byte-identical durable or temporary copies are listed as origin aliases where available.

The required [builder-check artifact](research/preparation/ive-plantuml-lalrpop-builder-check.md) is exactly **23 bytes**, `nothing important left` followed by a newline. It is preserved as a **placeholder**, with no substantive builder verification inferred. The [migration design preparation](research/preparation/ive-plantuml-lalrpop-migration-design-prep.md) is separate research, not a replacement verdict.

[Original semantic research](catalog.md#original-source-research), [source ledger/verification](catalog.md#original-source-provenance), and [historical oracle reports](catalog.md#historical-oracle-research-report) retain their original evidentiary scope. The PlantUML original-oracle report's 30 cases include two degraded formula fallbacks, not successful typesetting. The broader oracle report also contains explicitly identified older native comparisons; none establish the final current corpus. Existing historical corpus hash caveats remain unchanged.

Only the authorized PlantUML MIT artifact/source scope is relevant to translated native behavior. A locally measured source/JAR identity is not external Maven authentication. The [existing source-lineage audit](../../docs/superpowers/plans/2026-10-23-puml-provenance-audit.md) remains the durable rejection record for the excluded candidate; this archive does not recommend or import it. No vendor implementation source, generated Rust parser, raw review diff, asset or binary is duplicated here.

## Existing live authorities

These remain outside the historical archive and may evolve after this snapshot. Their exact archive-base hashes and sizes are in `manifest.json` under `liveReferences`.

- [Current PlantUML source/fixture/module traces](../provenance/plantuml-mit-1.2026.9.json), [MIT notice](../provenance/LICENSE-PlantUML-MIT.txt), and [parser/function responsibility inventory](../provenance/plantuml-parser-responsibilities.json).
- [Current parser registration/provenance](../provenance/plantuml-parser.json), [admission evidence](../provenance/plantuml-parser-admission.md) and [residual inventory](../provenance/plantuml-parser-residuals.md).
- [Generator dependency grants](../provenance/LICENSE-PlantUML-Generator-Dependencies.md) and [supplement](../provenance/LICENSE-PlantUML-Generator-Supplement.md). The archive preserves dependency/notice mappings without copying these legal texts again.
- [Parent implementation plan](../../docs/superpowers/plans/2026-10-22-diagram-ports-plan.md) and [LALRPOP migration plan](../../docs/superpowers/plans/2026-10-23-plantuml-lalrpop-plan.md).

Historical [native](history/native-controller-ledger.md) and [migration](history/migration-controller-ledger.md) ledgers preserve the controller's old rulings/events, not the owner's current checklist. The controller owns the canonical `progress.md`, final chronological Rulings handoff and any link updates before SDD cleanup. This archive job does not edit them.

## Pending controller additions

| Entry | Status |
| --- | --- |
| Advanced-state implementation/integration/report and review | Pending controller update after the released work completes. |
| Final post-state coherent original/native corpus and actual native family captures | Pending; older checkpoints and non-final rehearsals do not substitute. |
| Final `COVERAGE.md`, documentation and independently verified full gates | Pending; no final coverage-preparation report is imported here. |
| Final chronological Rulings handoff | Pending controller-owned consolidation from the exact historical records. |
| Whole-branch final independent review and dispositions | Pending; the 44 historical documents are scoped earlier reviews. |

Future evidence should be added as a new, explicitly dated/current entry with its actual source identity and checks. Preserve existing payloads verbatim. Update the catalog, manifest and checksum inventory when adding evidence; do not silently rewrite old reports or promote a placeholder into approval.

## Preservation verification

From this directory:

```sh
sha256sum -c SHA256SUMS
```

`SHA256SUMS` covers all payloads plus `README.md`, `catalog.md`, `verdict-index.md` and `manifest.json`; it excludes itself. The controller handoff reports the checksum file's own SHA-256. The manifest's original locations allow byte comparisons when those sources remain available. A missing historical `/tmp` source does not invalidate the durable archived copy.

Four required source-semantic research documents retain their original blank EOF lines: inventory-remaining, inventory-small, next-semantics and specialized-semantics. A whole-archive `git diff --check` reports those preserved bytes. The newly authored indexes and manifest pass their scoped whitespace check; copied research is not normalized to change its source identity.

This archive makes no new runtime, syntax, layout, font, native-measurement, pixel-parity, source-authentication or full-compatibility claim.
