# Completed current-parser migration inventory

Basis: stable product `da590bc` plus final Task17 tooling/adapter cleanup. All current components have one registered generated structural authority. The final integration, full gate record and independent review remain separate completion steps.

| Component | Structural authority | Retained handwritten responsibility |
| --- | --- | --- |
| Same-line definitions | definition.lalrpop | Immutable registry, bounded literal evaluation/expansion and capability policy. |
| Wrappers/directives/EOF | document.lalrpop | Shared lexical ownership, borrowed body bounds and origin mapping. |
| Family selection | header.lalrpop | Primitive tokenization and bounded ownership-normalized body scanning; no copied physical-line input. |
| Preambles/styles/chrome | preamble.lalrpop | Property/color primitives, typed stylesheet policy and existing chrome layout. |
| Sequence/state/UML/activity/Chen | Respective registered grammars | Primitive spelling/escaping, scoped typed builders, identity/constraint resolution and unchanged layouts. |
| Mindmap/WBS/files/board | tree/files/board grammars | Marker/path primitives, raw notes and existing typed builders/layouts. |
| JSON/HCL/YAML | Respective registered grammars | Primitive quote/comment/number/scalar/indent ownership and ordered lowering/layout. |
| Timing/Gantt | Respective registered grammars | Exact integers, finite ISO-date validation, time/identity semantics and existing layouts. |
| Salt/nwdiag/packetdiag | Respective registered grammars | Direct escapes, bounded literal ownership, typed constraints/arenas/identities and existing layouts. |
| Creole | creole.lalrpop | Finite tag/delimiter tokenization and same-line marker eligibility; generated nesting/EOF owns span structure, typed actions preserve run/style assembly. |
| ASCII Git history | git.lalrpop | Raw column primitives, canonical commit identities, sparse checked connector traversal and dedicated measured lane geometry. |

`plantuml-parser-responsibilities.json` explicitly classifies all 75 authored Rust modules and 1,131 function declarations, including methods and tests. Roles are generated facade, lexical, semantic, layout, infrastructure and test adapter. There are zero pending-syntax exemptions. The audit rejects that role, unclassified files/functions, and the obsolete physical-line input type. Native modules have no directory exemption. This declaration inventory is a maintenance contract, not a Rust semantic analyzer or substitute for review.

The generated inventory is one-to-one: all 22 grammar inputs, generated Rust outputs, SHA-256 pairs and module registrations must match. Stale grammar/output bytes fail before the external source ZIP checks. Independent generator regeneration remains necessary; inventory hashes alone do not prove generation.

Remaining parse/prefix/split/cursor helpers are explicit primitives or generated facades: `arrow::parse` decodes a finite arrow spelling; `gantt::CivilDay::parse` validates a finite ISO date without a synthetic line record; `parser/tree.rs::prefix` scans markers; `parser/structured.rs::Cursor` provides checked primitive delivery/storage/depth accounting. Prefix/split operations in registered lexical functions decode literal spellings, payloads, comments, escapes, colors and capability boundaries. They do not admit recursive family structure. Creole `activate` determines only same-line marker eligibility; generated productions decide nesting and matching. Typed model builders retain semantic duplicate/identity/range/style validation.

Removed: production `source::Line`, `ParsedDocument.lines`, `document::append_line`, header `PhysicalRows::locate`, the render body-line slice, the Gantt dummy diagnostic line, old source preparation/materialization helpers and native/parser Line-based test entry adapters. No old structural parser retry, alternate route or frozen comparator remains. Shared cfg(test) support constructs a full source document and invokes generated entries; activity's document-only helper preserves the original no-preprocessing setup. Public facade tests still exercise preprocessing, ownership and the complete render pipeline.

Accounting now reflects actual retained storage: document admission retains the body view and preamble token storage, without a line vector or cloned per-row strings. The render view remains charged before family entry. Header selection charges bounded lexical scans and generated actions over borrowed source. Worker-specific token/stack/depth/literal/model guards and global defaults (256KiB source, nesting64, work5m, model10k) are unchanged. Removing obsolete allocations and double test setup can reduce reported work; no fixed work-count golden was rewritten.

The immutable 43 source/family/SVG baseline is unchanged. The 60,000-row/500ms activity assertions, deadlines and input data remain unchanged; setup migration is separate from feature REDs. Carried Task4 concurrent failure and later serial passes show timing sensitivity without proving an exclusive cause. Audited vendor Atomic deprecations in merman-core operation.rs:449 and development test-support merman-render environment.rs:585 remain unchanged.
