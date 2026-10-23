# Task 8 — structured activity generated cutover

Base: `1506eb1f14248eb6dfe109164308f6914e3421ec`.
Frozen head: `1b0c234b73223abe74c8fcc7e4f6e3451a671f1c` (`diagram-parser-activity`).
Checkout: `/mnt/data/Projects/ive/.worktree/diagram-activity-parallel`.
Tracked tree clean; only root-owned untracked `diagram-ports/.cargo/` remains, untouched/uncommitted.

`parser::activity::parse(BodySource, &mut Budget) -> Result<Block, RenderError>` now drives generated ActivityParser.
Generated recursive Block/Instruction/ElseIfs/Lanes/Parenthesized/Guard productions own pairing, continuations and EOF.
Existing Block/Instruction/Branch/Condition/Join/GroupKind models and activitytile production layout are unchanged.
Removed runtime activity::Parser, recursive block/instruction/action/group/loop/fork methods and condition/guard reparsers.
No command-string semantic forwarding, legacy fallback, retries, recovery or partial render.
The cfg(test) activity::parse helper constructs the same document-only setup formerly supplied by source::prepare.
Minimal shared reborrow/view-byte accounting copies the coordinated state/tree pattern; root unions dispatch centrally.

Observed RED (all Cargo commands outer-locked, shared target, --locked --offline):
`cargo test -p diagram-engine plantuml::parser::activity::tests -- --test-threads=1`
Initial four facade contracts failed with `generated activity parser unavailable` before production implementation.
They exercise typed nested models, crossed/illegal continuations, depth admission and work/model/cancel behavior.
`cargo test -p diagram-engine should_locate_activity_guards -- --test-threads=1`
Observed guard error at line1/column1; generated guard spans plus Row breakpoints now report authored line3/column23.
An initial expected column22 was corrected after enumerating the literal source; the failure's original line1 was real.
Direct generated scheduled cancellation initially returned Ok at checkpoint7; final Activity reduction now checks Budget.
`cargo test -p diagram-engine --lib should_trim_comment_normalized_action -- --test-threads=1`
Observed Action("opening   \n  raw  \nclosing"); nonliteral opening normalization now yields "opening\n  raw  \nclosing".
`cargo test -p diagram-engine --lib should_include_unfinished_action -- --test-threads=1`
Observed Ok(15648) despite a live16KiB unfinished buffer under25,600-byte retention allowance; admission now rejects.
Interim action-control failures were a 500ms cancellation and below-cap work assertion after adding preprocessing to setup.
Restoring original document-only setup resolved both; test data, deadline, exact error and work assertions were unchanged.
Initial visibility/required Token::Clone compiler errors were corrected; no failing final gate remains.

Final GREEN command from diagram-ports, with prefix
`bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`:
`cargo test --locked --offline -p diagram-engine --lib --test plantuml_control --test plantuml_constants --test plantuml_styles -- --test-threads=1`
Result:180 engine units +44 constants +23 control +26 styles passed, zero failures.
This includes13 new facade/resource/origin contracts and unchanged exact topology/private geometry/raw action controls.
Public constants exercise full preprocessing, immutable expansion and expanded opening/continuation action terminators.
`cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings`: exit0.
`cargo fmt --all --check`: exit0. `git diff --cached --check` before commit: exit0.
Existing capped vendor atomic deprecation warnings remain; no vendor/dependency/lock changes were made.

Private generator, cwd/tmp, same outer lock, target `/mnt/data/Projects/ive/target/diagram-generator-activity`:
`cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-activity-parallel/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- check`
Result:Verified6 reproducible PlantUML generated parsers; check wrote no tracked outputs.
Grammar SHA256: `ca3f7156779b2fe62ac5254cffbf33901cad71b481ae3cc341f539965865570d`.
Generated SHA256: `965aeffbee0584b443512415e6651a25639777d2ffc6effcaee42702b2851d49`.
Actual proof: `/tmp/ive-task8-stack-proof.py`, output `/tmp/ive-task8-stack-proof.json`.
92states/27terminals; all92 states reached by depth3;187 viable-prefix edges equal independent finite reduction closure.
Four reductions per delivery/EOF; at most12 expected terminals; depth0..4 live maxima6/13/22/31/40.
Max-plus induction over all92 end-state distances proves9d+6; depth64/65/128 bounds582/591/1158.
Admission reserves9*recorded combined depth+24, actual vector growth/diagnostic/token/literal/action/view/builder sizes.
Semantic vector/string growth admits aggregate old+new storage before fallible reserve, charges copying and checks actual capacity.
Depth64 accepts;65 rejects before opener token/payload retention. Tiny token/model growth and seven cancel checkpoints pass.
Shared Ownership continues the preamble checkpoint; authored comment-gap and frozen guard-use origins map once at the facade.

`node tools/audit-plantuml.mjs`: Verified220 MIT source/ZIP hashes,43 fresh fixture hashes and43 native modules.
Only authorized existing activity command mappings gained parser/grammar destinations; grants/credits/hashes remain intact.
All global source/SVG/model/nesting/work/time/label/query/coordinate/raster defaults and public APIs remain unchanged.
Baseline SVG/hash/native/viewer/full16 gates are CENTRAL ONLY under parallel-context; not rerun in this worker checkout.
No fixtures/goldens were regenerated and no IVE/native process was started or left running by this worker.
Residuals: independent task review and combined integration gates/native captures remain root-owned.
