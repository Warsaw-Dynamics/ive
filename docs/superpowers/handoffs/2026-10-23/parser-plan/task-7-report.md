# Task 7 — generated state syntax

Base: `c23afd66158758c992a0278511caf1b21cec3e88`.
Head: `5512ee69ffa9ebfa9123e2bd6eb267fb50adc72d` (`diagram-parser-state`).
Checkout: `/mnt/data/Projects/ive/.worktree/diagram-state-parallel`; scoped commit, frozen.
Tracked tree clean; only root-owned `diagram-ports/.cargo/config.toml` remains untracked.

Scope/interface:
- New `parser/state.rs::parse(BodySource, &mut Budget) -> Result<state::State, RenderError>`.
- Real `grammar/state.lalrpop` owns declarations/aliases, descriptions, nested composites, all supported closers, directed hyphen arrows and EOF.
- Generated actions lower typed names/captions/direction/spans into the preserved state model through a two-phase Builder.
- Removed old Line-based parse, declaration, statement, transition and closer syntax routes; no fallback, retry, raw command or text reparse remains.
- Preserved declaration IDs/order, scoped initial/final IDs, local-name isolation, ancestor rejection and forward composite references.
- Stategraph production/geometry unchanged; existing private identity/geometry assertions retained with new parser setup.
- Shared minimal changes: module/generated registration, preparsed state dispatch and identical `BodySource::reborrow()` coordinated with tree.
- Common dispatch checks adapters+actual sizeof BodySource against source*256, charges that copy, then propagates the enlarged allowance before reborrow.
- No dependency, lock, vendor, viewer or other-family parser edits; no master/ledger edits.

RED evidence (all real failing assertions, then repaired):
- `/tmp/ive-task7-red.log`: new facade positive model contract failed with explicit unavailable stub before generated implementation.
- `/tmp/ive-task7-boundary-red.log`: caught accidental `state : text` acceptance; generated required separator/left-name policy repaired.
- `/tmp/ive-task7-colon-red.log`: caught `A --> B: label` acceptance; generated attached-description versus separated-transition colon distinction repaired.
- `/tmp/ive-task7-qualified-red.log`: caught unquoted `state A::B`; restored quoted-versus-identifier identity policy.
- `/tmp/ive-task7-opener-red.log`: caught trailing bare `begin` declaration; explicit unsupported result restored, with quoted/composite positive controls.
- Focused RED commands: `cargo test --locked --offline -p diagram-engine --lib parser::state::tests::<named_contract>` (exact names/output in logs).

Final GREEN:
- All Cargo/outer Cargo drivers used `/tmp/ive-diagram-build-locked.sh`; engine target `/mnt/data/Projects/ive/target`.
- `/tmp/ive-task7-final-rust.log`: `cargo test --locked --offline -p diagram-engine --lib --test families --test plantuml --test plantuml_control --test plantuml_styles --test plantuml_constants --test plantuml_inventory`: 175 unit +193 public tests passed, zero failures.
- Same final log: engine `cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings` and `cargo fmt --all --check` passed.
- Existing two vendor Atomic::fetch_update deprecation warnings remain; no vendor changes.
- Tests cover structure/EOF/unknown suffixes, forward references, identical local names, scoped/reverse pseudo roles, descriptions/closers/arrows, exact depth64/65, source/work/items, lexer/delivery/reduction cancellation, UTF8/CRLF/BOM/comment gaps and once-mapped frozen origins.
- `/tmp/ive-task7-generation-check.log`: private generator target `/mnt/data/Projects/ive/target/diagram-generator-state`, cwd `/tmp`, absolute manifest; locked/offline `cargo run ... -- check`: 6 reproducible parsers, no tracked writes.
- `/tmp/ive-task7-audits.log`: exact PlantUML source/parser audit220/43/43, Merman656/four, and18 source/vendor/baseline guard tests passed.
- `git diff --check` passed before scoped commit.

Actual resource/source evidence:
- Grammar SHA256 `8fb47d41dcede572dfe2ac565f2b804a8be0030e244266bea9182f130bb6f36e`.
- Generated SHA256 `8aaa54075c1e0ad039cf36db378650ea5284e455cec2f86c4aee064a94e83b7c`.
- Witness `/tmp/ive-task7-stack-proof.py`; exact output `/tmp/ive-task7-stack-proof.json`; actual59 states/18 terminals, max2d+8 live states/four reductions/eight expected terminals.
- Depth0/1/2/8/64/65 reachable stacks52/107/162/492/3572/3627; diagnostic copied capacities14/18/22/46/270/274, growth peaks21/27/33/69/405/411.
- Actual repeated composite prefix2,5; left-folded nonrecursive declarations/items/shaft. Full admission argument committed in `provenance/plantuml-parser-admission.md`.
- Checks before generated stacks/diagnostics and arena growth include concrete symbol payloads, actual buffered capacities/literals, adapters/views and retained model/string growth; direct exhausted-arena assertion observes zero allocation.
- Continuation reuses the shared Ownership checkpoint and immutable expanded document, truthful Row breakpoints and one origin mapping; no new wrapper/comment/evaluator pass.
- Exact authorized state source destinations added without source hashes/notices/credits changes; advanced concurrent/history/subtype/decorated features explicitly reject.

Owner-directed integration deferrals/residuals:
- Immutable43 baseline file SHA remains `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe`; no rerecord. Actual43 render comparison/native capture/full16 gates deferred to root's combined integration, as directed.
- Early shared-target generator run reported8 parsers despite6 local grammars; immediately reported root, no cache cleaning/reverts. Approved private generator target then produced/check-verified this checkout's six outputs.
- Sequence I1 ongoing in integration is outside this branch; no sequence syntax edits. Shared dispatch/admission unions require integration review.
- No viewer/app instance started; no worker processes/native captures to clean up. Fresh independent review is root-owned.

## Fix round 1 — reviewed arrow adjacency

- Review `/tmp/ive-plantuml-lalrpop-state-review.md` read completely; addressed its sole Important, no other findings.
- Fix base `5512ee69ffa9ebfa9123e2bd6eb267fb50adc72d`; current frozen head `ec608400b2fb375cd899c23301bb7472c27b558e`.
- Generated Hyphens/Arrow reductions now require adjacent normalized-row cursor payloads on each `-`, `<`, `>` primitive; expanded triples remain independent truthful origin spans. No statement/arrow source parsing or fallback was added.
- Private/public negatives cover split head, shaft, reverse and tab forms; contiguous/reverse/long arrows and zero-space inline-comment joins remain positive, with exact private endpoint IDs and literal-caption controls.
- `/tmp/ive-task7-round1-red.log`: `cargo test --locked --offline -p diagram-engine --lib --test plantuml_control split_between_operator_primitives` failed private assertion on `- >` before fix.
- `/tmp/ive-task7-round1-public-red.log`: `cargo test --locked --offline -p diagram-engine --test plantuml_control should_reject_state_arrows_split` failed public assertion on `- >` before fix.
- `/tmp/ive-task7-round1-rust.log`: locked/offline `cargo test -p diagram-engine --lib --test plantuml_control` passed177 unit+25 public; engine-targeted all-target strict Clippy and fmt check passed.
- `/tmp/ive-task7-round1-generation-check.log`: approved private-target locked/offline generator `check` verified6 reproducible outputs without writes.
- `/tmp/ive-task7-round1-audits.log`: exact source/parser220/43/43 and15 source/baseline guard tests passed. `git diff --check` passed; only root-owned untracked Cargo config remains.
- Current grammar/generated SHA256: `276497c8cc77dafb56d958d1de4765e91c4fd3bdaf5eeca3863edbc3112cdbca` / `ed52f0313911849eefa4c2290ed370fe0191bac2322a4bd6402069c2641d180f`.
- Renewed exact witness `/tmp/ive-task7-round1-stack-proof.py` and `.json`:59states/18terminals; all prior depth counts/max2d+8/four reductions/eight expected/diagnostic growth bounds reproduced. Generated Range payload size explicitly added to admission before allocation; full proof appended to committed admission document.
- Existing model/geometry/origin/resource/cancellation controls remain green; no model/layout/common API/source/license changes. Immutable baseline SHA unchanged; combined baseline/native/full gates remain central-only. Branch frozen for independent rereview.
