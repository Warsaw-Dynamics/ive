# Task 6 — Chen structural grammar and contextual names

Base: `0c2ee0e3b52d4d5a02a360fd5f4ae61931fdb089` (fixed UML head).
Frozen head: `c465f21954d998404e4aa6e1941fed98208f4327` on `diagram-parser-chen`.
Isolated checkout: `/mnt/data/Projects/ive/.worktree/diagram-chen-parallel`.
Tracked files clean; only identical untracked `diagram-ports/.cargo/config.toml` cache setup remains.
Scope:13 files; Chen parser/grammar/output/model builder, scene-test setup, public regression,
minimal dispatch/registration, dead Chen-only model lexer removal and exact provenance entries.

Facade: `parser::chen::parse(BodySource, &mut Budget) -> Result<chen::Chen, RenderError>`.
Actual entry: `generated_chen::ChenParser`; productions own owner kinds/names/opening braces,
recursive composite attributes, relationship attributes, aliases, participation/cardinality and EOF.
Typed names/kinds/endpoints feed the existing qualified identity and owner construction semantics.
Removed old runtime Parser::line/declare/attribute/associate and complete name/cardinality reparsers.
Existing Chen/Node/Link/AttributeOwner/Participation types and chenscene production layout are unchanged.
No Line forwarding, command reparsing, fallback, recovery, partial render, dependency or fixture changes.
Chen no longer calls legacy lexer::model_tokens; its dead Model wrapper/branches are removed.
The base still has Work lexer callers in unmigrated state/activity; central integration owns their union/removal.

Observed deferred M1 RED before production cutover, with outer lock/shared target/locked/offline:
`cargo test -p diagram-engine --lib keyword -- --test-threads=1`.
Two new Chen regressions failed: `entity -1- relationship` was misread as a declaration;
`entity {` inside C was rejected as nested declaration while compact `entity{` reached attribute syntax.
Both now pass with literal keys C/entity and C/entity/leaf, typed nested ownership and reciprocal endpoints.
A public renderer regression pins the same identities and c3→c4/c4→c3 edge owners/targets.
Actual nested entity declarations, five weak/identifying/key/multi/derived decorations and subtypes still reject.
Normal comments, active constants, quoted raw-byte limits, cardinality spelling and existing text quotas remain.

Final focused GREEN command, from diagram-ports with prefix
`bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`:
`cargo test --locked --offline -p diagram-engine --lib --test plantuml --test plantuml_chen_archimate --test plantuml_constants --test plantuml_control --test plantuml_styles -- --test-threads=1`
334 passed:187 engine units +41 UML/common +13 Chen/ArchiMate +44 constants +23 control +26 styles.
Log: `/tmp/ive-task6-engine-final.log`; zero failures, all original geometry/model assertions retained.
The existing scene test still exercises genuine shape/intersection semantics at font sizes6/14/96.
Direct controls cover full generated malformed streams/EOF, depth64/65 before opener retention,
tiny pre-entry work/storage admission, primitive model items before next unterminated quote,
original16KiB raw escaped-quote boundary, lexical cancellation and eight generated delivery/action checkpoints.
Authored comment-gap and frozen definition-use failures map once at the facade before the layout bridge.

`cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings`:exit0;
log `/tmp/ive-task6-clippy-final.log`. Existing capped vendor atomic deprecation warnings only.
`cargo fmt --all --check` and staged `git diff --check`:exit0.
Private generator from/tmp, target `/mnt/data/Projects/ive/target/diagram-generator-chen`, same outer lock:
`cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-chen-parallel/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- check`
Verified7 reproducible parsers; check wrote no tracked output. Generated files were never hand-edited.
Grammar SHA256 `ae028b0dbaf00ed0cedf2bcb528f4fc94e81d8b322e74b922fa729a898d9d5ee`.
Generated SHA256 `506e35386ae10da006b918f33715aee6f5e5b4b54fb4b1f7c03736c2c3da393c`.

Actual witness: `/tmp/ive-task6-stack-proof.py` and `/tmp/ive-task6-stack-proof.json`.
50states/14terminals; observed96 prefix transitions equal independent finite reduction closure.
All50 states reachable by depth1; at depths0/1/2/8/64/65 live maxima10/10/11/23/135/137.
Maximum4 consecutive reductions,7 expected terminals; diagnostic growth peaks21 state bytes.
Max-plus induction D3<=D2+2 over every state proves2d+7 for d>=2; initial depths have bound10.
Admission uses2d+12, actual capacities/symbol sizes/diagnostics/tokens/literals/adapters/builder storage.
All semantic nodes/links/owner arenas admit aggregate old+new growth before fallible checked reserves,
then account for actual returned capacity. Original primitive/model/text/depth/work/default limits remain.
Full proof explanation is appended to `provenance/plantuml-parser-admission.md` in this commit.

`node tools/audit-plantuml.mjs`:Verified220 authorized MIT source/ZIP hashes,43 fixtures,43 native modules.
Only existing Chen source destinations gain parser/grammar entries; credits/grants/hashes remain unchanged.
Interim fixes: one LALRPOP named/anonymous binding error, expected public n3 IDs corrected to actual Chen c3,
and strict-Clippy dead Model accounting cleanup after its last caller was removed. No failing final check remains.
Per resumed user workflow, no broad independent review was run; broad review waits until the overall port ends.
Baseline/full16/native viewer captures remain root-owned central batch checks; no worker native instance was started.
Ready for the designated central integration writer; no other checkout product edits or central merge performed.
