# Task 9 — generated mindmap/WBS/files/board syntax

Base: `c23afd66158758c992a0278511caf1b21cec3e88`.
Head: `a60eda0a5e14c0443e0da8efb3a5a971ac4866b9`, branch `diagram-parser-tree`.
Checkout: `/mnt/data/Projects/ive/.worktree/diagram-tree-parallel`.
Tracked tree frozen clean; only root-owned untracked `diagram-ports/.cargo/` remains.

Four routes remain distinct:
- `parser::tree::parse_mindmap(BodySource, Budget)` → generated `MindmapParser` → `tree::Mindmap`.
- `parser::tree::parse_wbs(BodySource, Budget)` → generated `WbsParser` → `tree::Wbs`.
- `parser::files::parse(BodySource, Budget)` → generated `FilesParser` → `files::Files`.
- `parser::board::parse(BodySource, Budget)` → generated `BoardParser` → `board::Board`.

Generated productions own marker/modifier/alias/separator, slash-component, note-block and stage-row structure and EOF.
Removed production marker parsers, files path/note collection and board command-row parser; no retry/fallback or whole-command reparse.
Remaining `Line` adapters are cfg(test)-only setup for unchanged model/scene assertions.
Separate semantic builders retain side stacks/depth normalization, WBS aliases/pseudo nodes, ordered reused folders/duplicate files/sibling notes, and nearest-lower board ancestry.
All scene modules, literal file names versus formatted note/card policy, geometry IDs/assertions and public APIs are unchanged.

Meaningful RED evidence:
- `/tmp/ive-task9-red.log`: three focused tests failed at unavailable mindmap/files/board stubs before implementation; the combined tree contract stopped at mindmap before reaching WBS.
- `/tmp/ive-task9-parser.log`: the new frozen WBS alias-origin contract failed with two expansion annotations.
  Fixed by parsing all four typed models before the remaining legacy render/error bridge; the same contract now passes once-only.
- Direct generated malformed modifier/alias/opener/path/note/stage streams and cancellation are additional controls, not claimed new REDs.

Final observed command (outer shared lock, engine shared target, locked/offline):
`cargo test -p diagram-engine --lib --test plantuml_tree_data --test plantuml_inventory --test plantuml_constants --test plantuml_control --test plantuml_styles -- --test-threads=1`
`/tmp/ive-task9-final-tests.log`: 175 unit +44 constants +23 control +27 inventory +26 styles +18 tree_data, zero failures.
The unchanged 60,000-row/500ms activity test hit its deadline once in an earlier parallel test run; final serial run passed unchanged.
`/tmp/ive-task9-final-checks.sh` ran under the outer lock and exited0:
- Strict engine `cargo clippy --locked --offline -p diagram-engine --all-targets --no-deps -- -D warnings`: pass (`final-clippy.log`).
- `cargo fmt --all --check`: pass (`final-fmt.log`); `git diff --check`: pass.
- MIT source/parser inventory audit:220 sources/43 fixtures/43 native modules (`source-audit.log`).
- Merman audit:656 files/four patches (`merman-audit.log`);18 Node audit/baseline-guard tests pass (`audit-tests.log`).
- Read-only generator check:8 reproducible grammars, no writes (`generated-check.log`).
All named logs above use `/tmp/ive-task9-` prefix. Immutable vendor deprecation warnings remain untouched.

Actual admission evidence: `/tmp/ive-task9-stack-proof.py` and `.json`, plus committed `provenance/plantuml-parser-admission.md`.
Mindmap:14 states/15 terminals/10 reachable stacks/max6 live; Wbs:165/15/161/max13.
Files:21/6/14/max6; Board:8/3/4/max5. All entries:at most4 reductions; expected-token maxima3/6/2/2.
Finite exhaustive table closure includes malformed orders/EOF; all repetitions fold left, no growing generated nesting or recovery.
Admission covers actual token capacity, state/symbol/diagnostic growth, builder sizeof, simultaneous adapters/text and every retained model-vector growth before allocation.
Semantic model/depth/text quotas remain unchanged; generated delivery/EOF/actions and row scans retain cancellation controls.
BodySource reborrow/checkpoint accounting matches state worker's common sizeof admission exactly; no Document/text clone or independent wrapper/comment pass.

Generated SHA256:
- tree: `46efd9c392b300833e8fa021f7f5dbb691f30d0e04c8445f18819495175a04c3`.
- files: `54e8692400cd49a5b57718ec911a330e277957a6666037c752ac5d2d6b2ebdb3`.
- board: `bbd732d9384e12db4faab021b965620d17dc46ae1875cb1ca0dd8bb718eafab1`.
Exact grammar hashes remain in generated manifest; authorized source hashes/notices/dependencies/vendor/fixtures unchanged.
Baseline file SHA256 remains `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe` (`baseline-sha.log`).
Per root's updated parallel contract, actual43 render-baseline comparison, full16 gates/closure/native/release/empty-PATH checks are integration-owned and were not repeated here.

Tooling incident: an initial shared-target cached generator binary mechanically regenerated its embedded other-checkout path before this hazard was detected; root was notified, no manual cross-checkout edits/reverts followed.
One generator-package shared-cache clean occurred before root prohibited it. Final generation/check use the approved private `target/diagram-generator-tree`, cwd `/tmp`, absolute manifest, outer shared lock.
Residuals: root integration must union module/dispatch/inventory/admission-doc changes with the other branches, then independently review this frozen diff and run central baseline/native gates.

## FIX ROUND 1 — Important I1

FIX_BASE `a60eda0a5e14c0443e0da8efb3a5a971ac4866b9`; fixed head `4a31cb7a3a3ceffefeaf17be2471e1dd20bee1e2`.
Read full `/tmp/ive-plantuml-lalrpop-tree-review.md`; confirmed shared Lexer::copy incorrectly imposed the display-label16KiB limit on marker prefixes and both WBS alias forms.
RED: locked/offline `cargo test -p diagram-engine --lib --test plantuml_tree_data long_ -- --test-threads=1` failed3 private contracts with label-bytes16385 (`/tmp/ive-task9-fix1-red.log`).
Dedicated `cargo test -p diagram-engine --lib should_preserve_long_parenthesized_and_quoted_label_aliases -- --test-threads=1` executed both forms before asserting and reported two label-bytes errors (`fix1-alias-red.log`).
Correction: explicit `copy_syntax` keeps per-value source, aggregate two-source-byte syntax, chunked work and cancellation admission; `copy_label` adds the unchanged16KiB guard. Only prefixes/aliases use syntax copying; actual labels, file components, notes and cards retain label copying.
Private controls cover16,385-byte normalized mindmap/WBS roots, both alias spellings, oversized plain/quoted display-label rejection, source/aggregate/work limits and cancellation. New public regression renders all four long-syntax cases and retains label rejection.
Final `/tmp/ive-task9-fix1-checks.sh` ran under the outer shared lock and exited0; engine target shared, generator target private/cwd `/tmp`/absolute manifest.
`cargo test --locked --offline -p diagram-engine --lib parser:: -- --test-threads=1`:64 passed (`fix1-private.log`).
Unfiltered `cargo test --locked --offline -p diagram-engine --test plantuml_tree_data --test plantuml_constants --test plantuml_control -- --test-threads=1`:19+44+23 passed (`fix1-public.log`).
Strict engine-targeted Clippy and fmt check pass (`fix1-clippy.log`, `fix1-fmt.log`); inherited vendor warnings unchanged.
MIT/source/parser audit220/43/43 passes (`fix1-source.log`); private read-only generator check verifies8 parsers (`fix1-generated-check.log`).
Grammar/generated diff from FIX_BASE is empty (`fix1-generated-diff.log`); rerun actual stack witness is byte-identical (`fix1-stack-proof.json`). Logs above use `/tmp/ive-task9-` prefix.
Tracked worktree frozen clean except root-owned untracked `.cargo/`. No dependency/vendor/fixture/default-limit/public-API changes. Central baseline/native/full gates remain root-owned; scoped independent re-review pending.
