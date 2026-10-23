# Parser migration checkpoint: Task17 preparation

Basis: product `e4ace61405c0cb5670df02b2879a44160f085fab`. This is an inventory/tooling checkpoint while Tasks15/16 are in flight. Final removal, integration gates, captures and independent review remain pending.

| Component | Authority at this checkpoint | Remaining work |
| --- | --- | --- |
| Definitions/documents/headers/preambles | Registered generated grammars | Remove obsolete physical-line materialization after packet/header/test consumers migrate. |
| Sequence/state/UML/activity/Chen | Registered generated grammars | Retain lexical primitives, model builders and layout; remove redundant test setup adapters in final cleanup. |
| Mindmap/WBS/files/board | Registered generated grammars | Retain marker/path primitives and typed builders; remove final Line-based test setup. |
| JSON/HCL/YAML | Registered generated grammars | Retain primitive quote/comment/scalar decoding and unchanged ordered lowering/layout. |
| Timing/Gantt | Registered generated grammars | Retain exact-integer/date primitives, identity/time semantics and layout; remove Line-only diagnostic/setup adapters where practical. |
| Salt/nwdiag | Registered generated grammars | Retain direct escapes, primitive capability checks, semantic arenas/identities and layout. |
| Packetdiag | `packet.rs::parse`, `configure`, `field`, `attributes` | Task15 must replace this production line parser. `number`/`positive` are bounded numeric primitives; `fragment` constructs the existing model. |
| Creole | `creole.rs::parse`, `lex`, `tag`, `reject_block`, `activate`, `runs`, `rich_tag` | Task16 owns delimiter/tag admission, pairing, style-stack validation and capability rejection. `symbol`/`literal`/`push` are primitive/token helpers; `append` coalesces model runs. |

`plantuml-parser-responsibilities.json` registers all 69 authored Rust modules and 1,059 function declarations individually, including methods and test helpers. The roles are generated facade, lexical, semantic, layout, infrastructure, pending syntax and test adapter. The 11 pending-syntax functions above are explicitly admitted for this checkpoint. New/deleted/renamed functions and new parser/native files fail the audit until classified. There is no exemption for native modules. This declaration inventory is a maintenance contract, not a Rust semantic analyzer or a substitute for review.

The parse/prefix/split/cursor residues have these responsibilities:

- `parser/*::parse*` entries invoke registered generated parsers; `parse_test`, `test_parse` and native family `parse` adapters are cfg(test) setup. `arrow::parse` recognizes a finite arrow spelling; the first `gantt::Date::parse` validates a finite ISO date spelling. Neither admits a family body.
- `parser/tree.rs::prefix` scans marker primitives. Primitive prefix/split calls in the registered lexical functions decode spellings, payloads, quotes, escapes, numbers, dates, colors and capability boundaries. Structural nesting/ordering belongs to the grammar, except the two explicitly pending components above.
- `parser/structured.rs::Cursor::{new,peek,take,depth_delta,measure_symbol}` is the bounded character/token cursor used by generated JSON/HCL/YAML/Salt and related frontends. Cursor admission/push functions enforce stack/storage/work bounds. It does not replace generated container or table productions.
- `preprocess.rs` remains the immutable same-line definition evaluator, registry and bounded literal expansion; declaration syntax is generated. `origin.rs`, `ownership.rs`, `context.rs`, `error.rs` and `budget.rs` retain ownership, limits and error mapping.

The exact physical-line cleanup boundary is `source::Line`, `parser/document.rs::{Prepared.lines,append_line}` and its retention accounting, `mod.rs`'s body-line slice, `parser/header.rs::select`, the production packet route, Gantt's primitive date diagnostic, and cfg(test) source/family/outline adapters. `source::{prepare_document,prepare,prepare_styled}` are test-only; the first preserves document-only setup semantics. Final cleanup must preserve all existing public facade regressions, action-control data/assertions/deadlines, source ownership and admission accounting. No frozen legacy comparator was introduced here.

The generated inventory is one-to-one: all 19 `.lalrpop` inputs, generated Rust outputs, SHA-256 pairs and module registrations must match. The audit rejects stale grammar/output bytes before the external source ZIP checks. The pinned generator `check` remains the independent regeneration proof; inventory hashes alone cannot prove generation.

Final integration must update this ledger after packetdiag/Creole merge, remove pending syntax and redundant adapters, run the immutable 43-fixture baseline and all central gates serially under the outer build lock, and then arrange independent review. Keep the inherited 60,000-row/500ms action-control assertion unchanged. Task4's concurrent failure and subsequent serial passes show timing sensitivity; they do not prove an exclusive cause. The inherited Atomic deprecations in audited `merman-core` operation.rs:449 and development test-support `merman-render` environment.rs:585 remain unchanged.
