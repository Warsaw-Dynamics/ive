# Task 5 — UML/description/ArchiMate generated syntax

Base: `a23aaeeaf17dc33734874dc2e86c024121b9581e`.
Frozen head: `6f1244cf87cedad7a4656ab23e249e2557d32ac3` on `diagram-parser-uml`.
Checkout: `/mnt/data/Projects/ive/.worktree/diagram-uml-parallel`.
Scope: thirteen files; generated UML facade/grammar/output, typed UML builder, dispatch,
graph-test setup only, exact provenance/inventory/admission entries and minimal shared APIs.

Removed runtime route: `uml::parse(&[Line])`, `Parser::line`, token-slice declarations,
relations and left/right endpoint parsers. No Line or complete raw command reaches a builder.
Facade: `parser::uml::parse(BodySource, &mut Budget) -> Result<uml::Uml, RenderError>`.
Actual entry: `generated_uml::UmlParser`; quoted/unquoted names, alias ordering,
roles, abstract class, members, group nesting, notes, labels and EOF are productions.
The existing entities/IDs/parents/relations/fills/direction and graph/layout/shapes remain.
Finite arrow spelling/value parsing remains lexical under the explicit brief allowance.
No parser retry, old-route fallback, partial recovery, dependency or golden changes.

Shared interfaces: exact agreed BodySource::reborrow and fixed-view admission before dispatch;
context::push visibility becomes `pub(in crate::plantuml)` for checked semantic arenas;
Syntax/Unsupported detail uses Cow, static constructors remain and owned_syntax is added.
The existing origin mapper accepts Cow directly; no origin.rs change was necessary.
Exact compiler layouts: RenderError32, FailureKind32 and ParseFailure48 bytes before/after.
Duplicate/ambiguous/undeclared detail reserves bounded text before formatting and maps once.

Meaningful observed RED: `cargo test --locked --offline -p diagram-engine --lib parser::uml::tests`
failed2/2 on explicit generated-syntax-unavailable stub before implementation (exit101).
Direction exactness RED: `... --lib should_keep_direction_spacing_and_quotes_exact`
failed because `left\tto right direction` was accepted; WordEnding metadata repaired it.
Direct generated unmatched/suffix/EOF and delivery/action cancellation are additional controls.

Final GREEN command (every Cargo invocation used the outer shared lock):
`cargo test --locked --offline -p diagram-engine --lib --test plantuml --test plantuml_chen_archimate --test plantuml_constants --test plantuml_control --test plantuml_styles`.
319 pass:174 units+40UML/common+12Chen/ArchiMate+44constants+23control+26styles.
Log: `/tmp/ive-task5-engine-final.log`; unchanged namespace/model/geometry assertions pass.
`cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings` passes;
`cargo fmt --all --check` and git diff --check pass. Final Clippy log `/tmp/ive-task5-clippy-final.log`.
Private generator from /tmp with absolute manifest and target diagram-generator-uml:
`cargo run --locked --offline --manifest-path .../tools/plantuml-grammar-gen/Cargo.toml -- check`
passes6 reproducible parsers, writing no tracked output. Outputs were never hand-edited.
`node tools/audit-plantuml.mjs` passes220 MIT source/ZIP,43fixture,43native-module inventory.
From /tmp, `python3 -B .../tools/audit-plantuml-generator.py check` passes67 closure/notice checks.

Actual table witness:379states/36terminals; at depths0/1/2/8/64/65, reachable stacks
5235/10471/15707/47123/340339/345575; live8/10/12/24/136/138; max4reductions/30expected.
Diagnostic copy capacities12/16/20/44/268/272, growth peaks18/24/30/66/402/408.
Witness: `/tmp/ive-task5-stack-proof.py` and `/tmp/ive-task5-stack-proof.json`;
the script parses all actual goto arms, including generated state ranges.
Admission covers actual tokens/literals/adapters, geometric stack/diagnostic growth,
possible declaration/member/implicit-endpoint arena slots, text copies and terminal probes.
All token/entity/member/group/pending/edge vectors use fallible prechecked growth.
Depth/work/text limits and cancellation are unchanged; direct probes cancel checkpoints1–8.

Grammar SHA256 `4454c4ab05bfc5ae7ff9af1af3aebb4fd2b81c6d6c5c11cfd7509a589650010c`.
Generated SHA256 `20ba6dca03dc4b42ae98666dd90431c132254a4dfb1bbfa80775982a5398d8ef`.
Immutable43 baseline SHA256 `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe` unchanged.
Baseline driver, full16gates and native/empty-PATH/captures are CENTRAL ONLY and were not run here.

Interim addressed findings: B::port misread as label; exact direction spacing/quotes;
overbroad semantic slot estimate preempting the namespace-text assertion; initial grammar
binding/LR factoring and retention visibility compile errors. Fixture-only assumptions
(#red, late !define, ninth checkpoint) were corrected against existing contracts.
Closure audit initially inherited worker patch config and failed --locked; rerun from /tmp passed,
with no lockfile/config/dependency edit. Existing vendor Atomic::fetch_update warnings remain.
Residual integration: lexer::name is retained solely for legacy state.rs callers at this base;
remove it centrally once the state cutover eliminates those calls. UML has no route to it.
No instance was started; the stopped task-owned generator mirror was removed. The only
remaining untracked checkout path is root-owned diagram-ports/.cargo/, intentionally untouched.

## Fix round 1/5 — R1 raw quoted-token boundary

FIX_BASE `6f1244cf87cedad7a4656ab23e249e2557d32ac3`.
Frozen fix head `0c2ee0e3b52d4d5a02a360fd5f4ae61931fdb089`; two files, no WIP.
Full review read: `/tmp/ive-plantuml-lalrpop-uml-review.md`; only Important R1 addressed.
Five production lines restore the original raw interior index/index+UTF8-width checks
alongside decoded allocation checks. Grammar, typed construction, origin mapping,
budget/cancellation, escape handling and all other bounds are unchanged.

Observed RED before production change (each exit101):
`cargo test --locked --offline -p diagram-engine --lib should_bound_escaped_uml_quoted_tokens_by_raw_bytes`
in `/tmp/ive-task5-fix1-private-red.log`:8192 accepted;8193 wrongly accepted in all four
declaration/quoted endpoint/left role/right role model contexts.
`cargo test --locked --offline -p diagram-engine --test plantuml should_reject_uml_quoted_tokens_beyond_raw_byte_limit`
in `/tmp/ive-task5-fix1-public-red.log`: renderer already rejected later at label tokens4097/4096,
not at the original earlier raw-byte boundary. No public positive was added above that guard;
the public regression asserts the restored earlier label-byte error in all four positions.
`cargo test --locked --offline -p diagram-engine --lib should_preserve_uml_quote_utf8_escape_and_eof_boundaries`
in `/tmp/ive-task5-fix1-eof-red.log`:16385 raw interior bytes without a closing quote
wrongly returned unterminated Syntax instead of the existing raw-byte Backend limit.
Focused GREEN11 private UML tests: `/tmp/ive-task5-fix1-private-green.log`;
focused public GREEN1: `/tmp/ive-task5-fix1-public-green.log` (same respective commands,
private command uses `--lib parser::uml::tests`). Valid n/backslash/tilde/quote escapes,
UTF8/exact16384-byte closed input and exact-cap unterminated/trailing-backslash cases pass.

Every final Cargo/Cargo-spawning check used the outer shared lock; engine target shared,
generator target `/mnt/data/Projects/ive/target/diagram-generator-uml`, generator cwd `/tmp`.
Actual final commands, distinct logs and observed outputs (all exit0):
`cargo test --locked --offline -p diagram-engine --lib --test plantuml --test plantuml_chen_archimate --test plantuml_constants --test plantuml_control --test plantuml_styles`
→ `/tmp/ive-task5-fix1-engine-final.log`:322 pass (176+41+12+44+23+26), zero failures.
`cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings`
→ `/tmp/ive-task5-fix1-clippy-final.log`: Finished dev profile; existing vendor deprecation warnings only.
`cargo fmt --all --check` → `/tmp/ive-task5-fix1-fmt-final.log`: empty output, exit0.
`cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-uml-parallel/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- check`
→ `/tmp/ive-task5-fix1-generator-final.log`: Verified6 reproducible parsers; no tracked writes.
`node tools/audit-plantuml.mjs` → `/tmp/ive-task5-fix1-source-final.log`:220MIT/43fixtures/43modules verified.
`python3 -B /mnt/data/Projects/ive/.worktree/diagram-uml-parallel/diagram-ports/tools/audit-plantuml-generator.py check`
→ `/tmp/ive-task5-fix1-closure-final.log`:67 archives/sources/licenses/features/notices verified.
`git diff --check` → `/tmp/ive-task5-fix1-diff-final.log`: empty output, exit0.

`/tmp/ive-task5-fix1-hashes-final.log` byte-compares FIX_BASE via git show and current files:
grammar8477bytes/SHA4454c4ab05bfc5ae7ff9af1af3aebb4fd2b81c6d6c5c11cfd7509a589650010c;
generated462599bytes/SHA20ba6dca03dc4b42ae98666dd90431c132254a4dfb1bbfa80775982a5398d8ef;
manifest2278bytes/SHA02fd7917928b4c9822184c409a885f7e700199d44df6b5999bd390ec9ac9fe1e;
baseline11664bytes/SHAa14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe.
All four are byte-identical. The prior379state/36terminal witness/admission still applies.
The same log reruns `/tmp/ive-task5-layout`: RenderError32, Before/AfterKind32,
Before/AfterFailure48; no types, payloads or arenas changed in this repair.
No instances started. Only root-owned .cargo/ remains untracked in the private checkout.
Handoff: fix is GREEN and frozen, but fresh scoped R1 re-review remains pending;
controller then integrates the fix while preserving its current shared interfaces,
and owns the remaining combined baseline/full16/native/empty-PATH checks.
Stopped after the user's near-weekly-limit handoff request; no further work started.

## Resumed central integration — UML and activity

Owner resumed with implement → focused tests → fixes; broad review is deferred until assembled.
Sole central writer operated only `/mnt/data/Projects/ive/.worktree/diagram-ports`;
the controller's unstaged progress.md edit was preserved and excluded from both commits.
Central base `780e3ab`; UML head0c2ee0e integrated as merge
`8a58a71df097d482bafee7906d7b989ba96f2e9d`; activity head1b0c234 integrated as merge
`e9c135421d434ce483f1d735298ddf9b1b545338` (final stable central HEAD).
Shared unions retain every approved sequence/state/mindmap/WBS/files/board route,
one common BodySource view admission, UML/activity preparse and once-origin facade mapping.
Parser/generated/provenance inventories and admission proof sections are unions;
the manifest was mechanically regenerated, never hand-edited. State test matches Cow::Borrowed.
After final state/UML/activity callers migrated, unused lexer::name/validate_name/
validate_identifier/tokens/split_label and TokenAccounting::Work were removed.
Remaining Chen model_tokens retains identical item/byte accounting and quoted decoding.

All Cargo/Cargo-spawning commands used the outer build lock and --locked --offline.
Engine target shared; generator cwd/tmp, absolute central manifest, private target
`/mnt/data/Projects/ive/target/diagram-generator-integration`.
UML-focused command `cargo test --locked --offline -p diagram-engine --lib --test plantuml --test plantuml_chen_archimate --test plantuml_constants --test plantuml_control --test plantuml_styles -- --test-threads=1`
passed362 (212+43+12+44+25+26) in `/tmp/ive-integration2-uml-tests.log`.
The same final combined command passes375 (225+43+12+44+25+26), zero failures,
in `/tmp/ive-integration2-tests-final.log`; intermediate activity pass is separately
recorded in `/tmp/ive-integration2-activity-tests.log` with the same375 count.
`cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings`
passes `/tmp/ive-integration2-clippy-final.log`; `cargo fmt --all --check` passes
empty `/tmp/ive-integration2-fmt-check.log`; git diff --cached --check passes before commits.
Initial Clippy found newly unused helpers; cleanup initially missed next's obsolete
parameter and failed compilation, corrected before final tests/Clippy. No residual failure.
Existing vendor Atomic::fetch_update warnings remain outside permitted vendor scope.

Central private `cargo run --locked --offline --manifest-path .../tools/plantuml-grammar-gen/Cargo.toml -- generate`
produced10 then11 parsers in `/tmp/ive-integration2-{uml,activity}-generate.log`.
Same private command with `check` passes11 reproducible parsers, no tracked writes,
in `/tmp/ive-integration2-generated-check.log`.
`node tools/audit-plantuml.mjs` passes220MIT/43fixtures/43modules in
`/tmp/ive-integration2-source-audit.log`.
`/tmp/ive-integration2-source-equality.log` byte-compares four UML and five activity
family/model/layout/grammar/generated files to their tested branch heads: all identical.
Immutable43 baseline remains SHAa14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe.
No native instance, baseline driver, full16-gate batch or independent review was run.
No blocker remains. Root/Chen worker notified that e9c1354 is available as a rebase base;
remaining combined baseline/native/full checks and final broad review stay controller-owned.
