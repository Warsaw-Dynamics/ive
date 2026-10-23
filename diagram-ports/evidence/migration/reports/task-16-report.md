# Task16 — generated admitted Creole nesting checkpoint

Base `d79ef81c67d1db28af153d1bcfd3dabee56491c5`.
Frozen head `d398ee6b266a60ea806a7bc515a845851e587623`, branch `diagram-parser-creole`.
Sole product worktree `/mnt/data/Projects/ive/.worktree/diagram-creole-parallel`.
Central product, main checkout and progress.md untouched; awaiting central integration.
Only untracked worker-local `.cargo/config.toml` remains; it is intentionally uncommitted.

Generated `Creole` productions now own real admitted tag/symbol nesting, exact close
kind and complete EOF. Scope parent styles are retained by the generated stack.
Old handwritten runs() and styled-span syntax Vec stack are removed. The semantic
four-bit active-style mask only validates duplicate/depth policy; empty-content checks,
run coalescing and style restoration preserve the Vec<Vec<Run>> consumer contract.
Lexical residuals: primitive escapes/URLs/resources/tag spelling and same-line pair
eligibility; unmatched punctuation stays literal. No command forwarding or fallback.
text.rs, context consumers and JSON/YAML/HCL literal data paths are byte-unchanged.
Failures preserve UnsupportedFeature, honest label-local byte spans and existing origin
enrichment once. Public frozen-definition control pins exact span and one expansion.

TDD RED `/tmp/ive-task16-red.log`: new nesting contract failed on the explicit
unimplemented generated facade; 0pass/1fail. Not an old-parser compatibility failure.
Initial implementation compile corrected pending-array type inference to Option<usize>.
First GREEN `/tmp/ive-task16-first-green.log`:438 relevant tests passed.
Seven generated nesting/private contracts passed `/tmp/ive-task16-private.log`.
Public test initially incorrectly expected ambiguous-candidate wording for a unique
expansion; actual exact lineage/span was correct. Failure preserved in
`/tmp/ive-task16-focused-final.log`; corrected assertion pins one RICH expansion.
Final focused command: `cargo test --locked --offline -p diagram-engine --lib
--test plantuml_styles --test plantuml_constants --test plantuml --test plantuml_control
--test plantuml_tree_data -- --test-threads=1`, engine cwd, log
`/tmp/ive-task16-focused-complete.log`:445pass (287+43+45+25+26+19),0fail.
Coverage includes crossed/empty/duplicate/unclosed scopes, complete token EOF, exact
styles/base restoration, UTF8 spans, hard/blank lines, URL-adjacent escapes, paired
resource rejection, depth/run/text/work/retained limits and8 direct cancel checkpoints.
Unchanged cache/measurement/4096-token/aggregate/linear-work and literal-data tests pass.

All Cargo/Cargo-spawning commands outer-lock `/tmp/ive-diagram-build-locked.sh`.
Engine target `/mnt/data/Projects/ive/target`; strict command
`cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings`
passes `/tmp/ive-task16-clippy-final.log`; only pre-existing Merman dependency warnings.
`cargo fmt --all` passes `/tmp/ive-task16-format-final.log`; subsequent
`cargo fmt --all -- --check` passes with empty `/tmp/ive-task16-fmt-check-final.log`.
Private generator, cwd `/tmp`, target `/mnt/data/Projects/ive/target/diagram-generator-creole`:
`cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-creole-parallel/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- check`
passes18 reproducible outputs, no writes, `/tmp/ive-task16-generator-final.log`.
`node tools/audit-plantuml.mjs` passes220MIT/43fixtures/43modules,
`/tmp/ive-task16-source-final.log`. Generator closure first failed in engine cwd due
to unused Merman patches/locked metadata; `/tmp/ive-task16-closure-final.log` preserved.
Correct cwd `/tmp`: outer-locked `python3 /mnt/data/Projects/ive/.worktree/diagram-creole-parallel/diagram-ports/tools/audit-plantuml-generator.py check`
passes67 archives/licenses/notices `/tmp/ive-task16-closure-tmp-final.log`.
`git diff --check` passes. Source search confirms no old runs/Mark syntax stack remains.

Grammar SHA256 `4203f6ac35369754e9b2db6b1f614bdc6fd11ad41992fefbe3d158ace1e91649`.
Generated SHA256 `142b44ea65358f201c91d03e29500dbf0aad3111d4457ead4a84eb41a81ef5ce`.
Manifest SHA256 `536961c04bb0f4484291ccf2da3cc3d00172d3aac171c71a1af0439cfcc140d7`.
Actual-table witness `/tmp/ive-task16-stack-proof.py` and `.json`:39states19terminals,
7596 reachable depth4 stacks,max11live/4reductions/12expected; diagnostic growth33.
Complete witness hashes, semantic gate/diagnostic overapproximation and concrete
allocation/work bounds are documented in provenance/plantuml-parser-admission.md.
Existing authorized MIT source destinations only; no dependencies/grants/hashes changed.
No broad review, full16, immutable baseline rerun or native launch; root owns these.
No concrete blocker. Remaining: central union with newer Salt/nwdiag registrations,
focused combined integration checks, then root's final broad review/gates/native.

## Central integration checkpoint

Integrated worker `d398ee6b266a60ea806a7bc515a845851e587623` into central base
`91abbdc54a44db35c9bf0fb2aa8806769ae5e139` (nwdiag product e4ace61).
Stable central commit `a796711d5f7aef88e178cb1c9feab1a70747101b`.
Three textual conflicts resolved by union: parser/mod.rs keeps both generated_nwdiag
and generated_creole registrations; admission.md keeps both complete appendices;
parser.json keeps newer Salt/nwdiag scope plus Creole scope/authorship. No whole-side
selection. Manifest/MIT destination lists merge mechanically with no conflicts.
Assertion inspection preserves all45 prior authored-file entries,20 prior generated
inventory entries and all19 prior manifest grammar entries without any changed values.
All five worker Creole implementation/grammar/generated/public-test files are byte-identical
to tested d398ee6. Model/layout/text/context/literal-data code and source limits untouched.
progress.md is unchanged. Central worktree is clean after commit; main checkout untouched.

Final combined command, engine cwd, common outer lock and shared engine target:
`cargo test --locked --offline -p diagram-engine --lib --test plantuml
--test plantuml_chen_archimate --test plantuml_constants --test plantuml_control
--test plantuml_gantt --test plantuml_inventory --test plantuml_nwdiag --test plantuml_salt
--test plantuml_styles --test plantuml_timing --test plantuml_tree_data
-- --test-threads=1`:577pass,0fail. Counts304units and43/13/45/25/17/27/17/18/26/23/19
public cases; `/tmp/ive-task16-integration-tests.log`. Includes generated contracts,
style/text/cache/control, literal data exclusions and current Salt/nwdiag geometry.
`cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings`
passes `/tmp/ive-task16-integration-clippy.log` (existing Merman dependency warnings only).
`cargo fmt --all -- --check` passes, empty `/tmp/ive-task16-integration-fmt.log`.
Private integration generator, cwd `/tmp`, outer lock, target
`/mnt/data/Projects/ive/target/diagram-generator-creole-integration`:
`cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- check`
passes20 reproducible outputs, no writes, `/tmp/ive-task16-integration-generator.log`.
`node tools/audit-plantuml.mjs` passes220MIT/43fixtures/43modules,
`/tmp/ive-task16-integration-source.log`. Outer-locked cwd `/tmp`
`python3 /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/tools/audit-plantuml-generator.py check`
passes67 archives/licenses/notices `/tmp/ive-task16-integration-closure.log`.
`git diff --cached --check` passes empty `/tmp/ive-task16-integration-diff.log`.
Hashes `/tmp/ive-task16-integration-hashes.log`: Creole grammar/generated hashes unchanged;
combined manifest SHA256 `9cda3adf214cdfc7ffb8d90fd6fe9a10fea8d07e7a2e7656a0a54dfd3adc1eb4`.
All final commands exit0; no integration test failure or product fix required.
No full16/baseline/native or broad independent review run; root owns remaining gates.
Integration complete with no blocker; paused for parent coordination.
