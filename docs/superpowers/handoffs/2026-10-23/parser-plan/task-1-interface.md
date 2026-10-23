# Approved parser infrastructure interface — Task1

Reviewed head: af042d7f18713d65d46c160303aa3232d5aa2082. Controller independently verified509 Rust tests+seven actual worker faults,17Node guards,generator3tests,notice5tests,all strict workspace/generator gates and67 archive/source/license/feature/notice coverage. Full review and scoped fix1 review approve definition syntax only. Every other current parser remains pending.

- `parser::SourceView` borrows text and absolute byte base; `parser::parse_definition` returns borrowed typed name/literal and exact authored spans. The registry/eager freeze/original ownership contract stays in preprocessing.
- `parser/error.rs` holds structured `ParseFailure`/`FailureKind`; its current formatting bridge is definition-authored. Task2 generalizes expanded-byte mapping through the origin document, preserving terminal public errors.
- Lexing recognizes independent header primitives with explicit name-tail/raw-literal ownership. No production-index checks remain. The generated grammar owns required token order and EOF. Leading Rust whitespace within a physical row includes CR; RHS trim/separators are ASCII space/tab only.
- Token retention/growth checks count/actual capacity/source-derived bytes and cancellation/work before allocation. Temporary parser retention does not consume semantic model-item counts.
- `provenance/plantuml-parser-admission.md` proves the ACTUAL current generated/runtime bounds:7states,5shifts,2reachable EOF reductions,no recovery;35stack/diagnostic slots and82fixed work plus actual token-buffer capacity. This is NOT a general bound for new grammars: renew per generated table/runtime/allocation path, including expected-token diagnostics.
- Exact generator/utility0.23.2; separate developer crate/lock. Regenerate mechanically, never hand edit/rustfmt generated bodies. `check` is read-only. Register all new grammar/output/admission/provenance inputs; no Merman generation.
- Immutable43-fixture baseline SHA256 a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe is the pre-cutover witness. Do not rerecord it. All43 source/family/SVG records remain equal.
- Generator notice inventory maps67 exact versions to68 selected grants:57 primary,2 supplement,9 reused selections for8 packages. Existing audit verifies exact retained grants and selected compound-license coverage as well as original package bytes/licenses/features. Retain the Python behavior guards and explicit reused-notice mapping.

Task1 full evidence: task-1-report.md (FIX ROUND1 supersedes initial admission/lexer/notice details), /tmp/ive-plantuml-lalrpop-core-review.md and /tmp/ive-plantuml-lalrpop-core-rereview-1.md. Native captures are actual pgui frames and processes were cleaned up. Do not rerun original Java/browser oracles to reconfirm this parser migration.

Deferred Minors: baseline actual-default configuration reporting belongs to amended Task17 brief; inherited Atomic deprecation noise belongs to final triage. Four older family-boundary Minors remain assigned to Tasks2/6/12/14. Root alone owns ledgers/master; one product implementer at a time.
