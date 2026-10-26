# Task 13 — generated Salt structure and widgets

Base: `435d73cf87f53d0b284535576ab7eddb14f79aa6` (stable central product checkpoint).
Frozen head: `0b9f4a6689a2789cdebad6a173176d69e49ec5a5` on `diagram-parser-salt`.
Checkout: `/mnt/data/Projects/ive/.worktree/diagram-salt-parallel`.
Tracked tree clean; only identical untracked cache configuration `diagram-ports/.cargo/` remains.
Scope11 files: Salt facade/grammar/generated output, typed arena builder, scene-test setup,
minimal dispatch/registration and exact parser/source/admission inventories.

Facade: `parser::salt::parse(BodySource, &mut Budget) -> Result<salt::Salt, RenderError>`.
Generated Table/Row/Column/Cell/Widget/Leaf productions own actual recursive structure and EOF.
Widget alternatives select field/button/checkbox/radio kinds from delimiter/X/space/payload primitives.
No complete raw widget command reaches a semantic classifier; textual leaves remain bounded payloads.
Removed runtime Salt Parser::table/leaf, tokenize/retain_token, classify and old tree-marker parser path.
Existing Salt/Widget/Kind/Table/Cell/Leaf model types and saltscene production code remain unchanged.
Typed frames reserve original preorder IDs; completed children enter the existing postorder widget arena.
Parent row/column placement uses the child table closing terminator; only same-row adjacent cells absorb spans.
Direct escaped pipe/braces remain literal; PUA values remain ordinary data, with no sentinel substitution.
Blank rows/empty pipe pieces, checkbox-before-button precedence, tree marker context and late-chrome policy remain.

Meaningful RED command (outer shared lock, shared engine target, locked/offline, serial tests):
`cargo test -p diagram-engine --lib plantuml::parser::salt::tests -- --test-threads=1`
Three new contracts failed against the explicit unavailable stub before implementation.
Log `/tmp/ive-task13-red.log`: nested postorder/spans, direct escapes/precedence/blank rows and malformed/late syntax.
Original exact model-ID/span assertions remain; original lexical escaped-brace/work assertions moved to facade tests.
Initial generated GREEN covered model/grid controls; full public Salt suite later passed18/18 unchanged tests.

Additional admission RED: `... --lib should_admit_salt_default_depth_at_its_exact_source_quota`.
Log `/tmp/ive-task13-admission-red.log`: valid149-byte depth64 source failed front-end token150>149.
Salt can emit two primitives per input byte (delimiter plus End); the bounded token allowance now reflects that.
Further runs exposed arena62475/66123>61947 from conservative growth retention; checked1.5x lexical growth,
one-slot semantic starts, separately admitted fixed stacks and release of deallocated old buffers fixed it.
Log `/tmp/ive-task13-admission-green.log`: depth64 at exact source quota passes; depth65 still rejects pre-retention.
No source/model/nesting/work/text default or public API was enlarged to pass the test.

Final command from diagram-ports with prefix
`bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`:
`cargo test --locked --offline -p diagram-engine --lib --test plantuml_salt --test plantuml_styles --test plantuml_constants --test plantuml_control --test plantuml -- --test-threads=1`
425 passed:269 engine units +43 PlantUML/common +44 constants +25 control +18 Salt +26 styles.
Log `/tmp/ive-task13-focused-final.log`; zero failures. Earlier full run's sole failure was the admission case above.
`cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings`:exit0, `/tmp/ive-task13-clippy.log`.
`cargo fmt --all --check`, working/staged `git diff --check`:exit0. Existing vendor atomic deprecations only.
All unsupported widgets/borders/regions, orphan spans, tree markers and admitted late chrome forms still reject.
Private controls cover model/depth/text/arena work before retaining children, direct malformed generated streams,
lexical cancellation, eight scheduled generated delivery/action checkpoints and once-only frozen origin mapping.

Private generator cwd/tmp, target `/mnt/data/Projects/ive/target/diagram-generator-salt`, same outer lock:
`cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-salt-parallel/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- check`
Verified15 reproducible parsers; check wrote no tracked output. Generated output was never hand-edited.
Grammar SHA `bdf09607b889c1bc60e11aa67d2312d8aa28f72987dd28a83579915ab1444697`.
Generated SHA `96d2e88d2ad55d4dbc98dc425a9f4a812fdc92e10f10d7386c7e16ea18bea3d2`.
Proof script `/tmp/ive-task13-stack-proof.py`, SHA `0b328976dc4772950ba3562c16dbdc7c3b96293d06e38773d0e0f18a134481da`.
Witness `/tmp/ive-task13-stack-proof.json`, SHA `dd31149231a6379bd8751860168955f9426abbfcd7d7849ab192ad4d2918ebab`.
Actual49states/17terminals; observed65 edges equal finite reduction closure; all-lookahead reduction graphs prove7.
Max10 expected terminals. Finite max-plus induction across all49 states proves3d+5; admission uses3d+8.
Depth64/65/128 live bounds197/200/389. State/symbol/growth/diagnostic/token/literal/frame capacities are admitted.
Storage rationale/formulas and exact-source repairs are appended to `provenance/plantuml-parser-admission.md`.

`node tools/audit-plantuml.mjs`:Verified220 exact authorized MIT source/ZIP hashes,43 fixtures and43 native modules.
Existing Salt traces gain parser/grammar destinations; source hashes/grants/credits/fixtures/goldens remain untouched.
Existing private grid/span/measurement geometry tests retain their assertions and use the generated facade setup.
Native captures/full16/baseline/broad review remain controller-owned and deferred; no worker native instance was started.
No central product/progress.md edits or central merge were performed. Paused after frozen tested checkpoint for integration.
