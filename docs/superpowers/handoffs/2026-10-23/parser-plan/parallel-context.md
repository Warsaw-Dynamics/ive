# Owner-directed acceleration — parallel family work

User asked why the work takes so long and how to speed it up. Independent remaining family implementations now use isolated git worktrees, one product writer per checkout. This supersedes the earlier self-imposed single-writer-across-the-entire-plan rule; one integration writer still owns the main diagram-ports checkout. Public semantics, resource limits, source/license rules, no fallback and independent review remain binding.

Shared integration baseline is c23afd66158758c992a0278511caf1b21cec3e88, verified553 Rust+7faults/all16gates/43 immutable fixtures. Sequence I1 first-command compatibility repair is ongoing only in the main diagram-ports worktree. Do not edit sequence or try to fix that known issue in another branch. Root will integrate its corrected shared base before the independent families. The shared definition/document/header/preamble interfaces are otherwise reviewed through f5a10a0 and extended by the c23afd6 Ownership checkpoint.

## Isolation and ownership

- Use ONLY your assigned worktree for product edits and git commits; use absolute apply_patch paths. Never change the user's main checkout or the integration checkout.
- State task7: diagram-state-parallel / branch diagram-parser-state. Tree/files/board task9: diagram-tree-parallel / branch diagram-parser-tree. UML task5 prepared: diagram-uml-parallel / branch diagram-parser-uml, basea23aaee (Task4 R1 known, fixed centrally). Root supplies the exact brief/report paths separately.
- Activity task8 prepared: diagram-activity-parallel / branch diagram-parser-activity, base1506eb1 (sequence review approved). Its generator target is `/mnt/data/Projects/ive/target/diagram-generator-activity`; all other isolation/build/central-verification rules apply. Root supplies its start signal separately.
- JSON/HCL task10: diagram-data-parallel / branch diagram-parser-data, base0c5177e (first combined batch fully approved). Its generator target is `/mnt/data/Projects/ive/target/diagram-generator-data`; all isolation/build/central-verification rules apply.
- Keep scope to your family model/parser/grammar/tests. Minimal shared module registration/dispatch and your exact generated/provenance entries may be edited in YOUR checkout; retain all other entries. Root will integrate shared-file unions once, using a dedicated integrator for conflicts. Do not merge/rebase/cherry-pick other branches yourself.
- Do not change dependency/lock/vendor/viewer/chrome/measurement code or other family parsers. Raise necessary common-API refinements with root while continuing independent work.
- Your worktree's untracked diagram-ports/.cargo/config.toml is root-owned cache configuration; do not commit/delete it. It points at the identical audited vendor copies in the integration checkout to avoid recompiling Merman for every worktree. No vendor changes are permitted. All remaining untracked product files must be intentionally committed.

## Build coordination

All Cargo commands and commands that spawn Cargo must use the OUTERMOST shared lock:

`bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo ...`

Lock only the outermost invocation; nested Cargo runs inherit the hold. Keep locked/offline flags. Code editing and read-only inspection do not need this lock. The existing500ms/60000-row action assertion is unchanged; do not add competing release builds or repeated retries. Do not run viewer release builds/native captures or Cargo-run example/baseline binaries in worker branches: shared unhashed executables are integration-owned.

**Generator exception (confirmed cache hazard):** each generator uses a PRIVATE target directory: `/mnt/data/Projects/ive/target/diagram-generator-state`, `diagram-generator-tree`, `diagram-generator-uml`, or `diagram-generator-integration` respectively. Run generator Cargo commands from `/tmp` with an absolute --manifest-path so worker engine-only patch config does not contaminate its separate lock. Still use the outer lock. Do not cargo-clean shared caches or use rustc metadata tricks. Shared-target generator binaries can embed another checkout's CARGO_MANIFEST_DIR even when Cargo reports Fresh; private targets prevent wrong-root generation. Engine hashed test artifacts/dependency cache remain shared.

## Lean worker verification and reports

Observe meaningful focused RED, implement real generated syntax and typed model construction, then run engine unit tests plus relevant public family/control/styles/constants tests, engine-targeted strict Clippy, fmt, private-target generator read-only check and exact source/parser audit under the lock. Immutable43 baseline driver is CENTRAL ONLY because Cargo-run example executables share unhashed aliases. Preserve model IDs/geometry/cancellation/source/work assertions. New grammar admission must still account for actual generated stack/reductions/diagnostics and retained arenas before allocation; use concise proof artifacts, not copied universal constants. No generator hand edits or baseline rerecord.

Root runs the complete16 gates, dependency closure audit, viewer release/native/empty-PATH checks once on the combined integration batch. A fresh independent task review still gates each family's diff; integration review covers shared-file conflict resolution. Do not duplicate reviewer seats or perform main-branch integration yourself.

Central full Rust test execution uses `-- --test-threads=1` under the outer build lock after observed test-thread contention around the existing500ms activity control. Assertions/deadlines and tests' own explicit concurrency stay unchanged. Preserve earlier failure/serial-control evidence for final triage.

Keep the full report concise (about60lines): base/head/scope, removed legacy runtime route and new interface, observed RED/GREEN commands/logs, actual admission/witness/hash/source evidence, baseline result, known residuals. No repeated plan/history narration. Commit/freeze your scoped branch and return short status/head/report path. Root alone edits master progress.md and both SDD ledgers.
