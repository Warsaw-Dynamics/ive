# Integration batch 1 — frozen

Base: `1506eb1f14248eb6dfe109164308f6914e3421ec`, clean central `diagram-ports` checkout.
Frozen head: `0c5177e1f4cf50dcc23dc97ff3e0812b42d4070f`; tracked/untracked status clean.
State no-ff merge: `9da5226da8ffd3f4d21cddc2b1d9a286ded6c7ed`, preserving approved `ec608400b2fb375cd899c23301bb7472c27b558e`.
Tree no-ff merge: frozen head above, preserving approved `4a31cb7a3a3ceffefeaf17be2471e1dd20bee1e2`.

Conflict resolutions (apply_patch; no family implementation changes):
- State merge: `diagram-ports/provenance/plantuml-parser-admission.md`; retain sequence fix rounds 1/2 and append complete state proof/fix.
- Tree merge: same admission document; retain all sequence/state sections plus complete outline proof.
- `diagram-ports/engine/src/plantuml/parser/mod.rs`: union family/generated registrations; retain exactly one fixed-size `BodySource::reborrow`.
- `diagram-ports/engine/src/plantuml/mod.rs`: retain one actual-size shared live-view admission; union six preparsed routes before legacy error bridge; state now reborrows like the other routes.
- `diagram-ports/provenance/plantuml-parser.json`: union authored/generated inventory and combined scope/authorship.
- `diagram-ports/engine/src/plantuml/generated/manifest.json`: remove conflicted manifest and mechanically regenerate with private integration generator; nine outputs.
- Existing README parser paragraph now includes state alongside sequence/tree/files/board; other migrations remain pending.

Preservation checks:
- Compared 22 sequence/state/tree/files/board implementation, grammar and generated files byte-for-byte against their approved heads: identical.
- Exact source provenance retains original hashes/notices; state destination additions preserved; no vendor, dependency, fixture or baseline changes.
- Existing Ownership checkpoint, immutable expanded document and once-mapped facade errors retained; no fallback/new feature.

Fresh verification, all exit 0 (logs `/tmp/ive-integration-1-*.log`):
- `compile`: `cargo check --locked --offline -p diagram-engine --all-targets`; existing two vendor deprecation warnings only.
- `fmt`: `cargo fmt --all --check`; `git diff --cached --check` also passed before each merge commit.
- `generate` / `generated-check`: `cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- generate|check`; generated/verified nine parsers.
- `source`: `node tools/audit-plantuml.mjs`; 220 MIT-edition source/ZIP hashes, 43 fixture hashes and 43 native modules verified.
- Every Cargo command used outer `/tmp/ive-diagram-build-locked.sh`; engine target shared `/mnt/data/Projects/ive/target`, generator target private `diagram-generator-integration`, generator cwd `/tmp`.

No unresolved integration issue. Full16/native/43-baseline/empty-PATH checks and fresh independent integration review remain root-owned; not duplicated here.
No app/test processes started; no user-main/worker checkout mutation, no root-ledger edits.
