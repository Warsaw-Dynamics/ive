# Task 12 — generated timing/Gantt checkpoint

Base `e9c135421d434ce483f1d735298ddf9b1b545338`; frozen head
`f5f07f97f3fcbcad8637355ba2a61a21792589ca` on `diagram-parser-time`.
Checkout `/mnt/data/Projects/ive/.worktree/diagram-time-parallel`;16 scoped files.
Tracked tree clean; only intentional untracked diagram-ports/.cargo/ cache config remains.
Root requested a tested checkpoint and pause before central integration; no integration performed.

Actual entries: generated_timing::TimingParser and generated_gantt::GanttParser.
Facades take BodySource+Budget and return preserved timing::Timing/gantt::Gantt.
Generated productions own contextual keywords/names, aliases, numeric-time commands,
states, endpoint/caption structure, project header, tasks/verbs/units/offsets/references and EOF.
Finite identifier/quote/bracket/integer/civil-date primitives remain lexical.
Removed timing statement/declare/endpoint/time-source splitting and Gantt project/
statement/verb/bracket/duration/complement/reference source reparsers. Builders consume typed values.
No raw command forwarding, generated Line wrapper, retry, fallback, recovery or partial render.
Cfg(test) former parser setup helpers now call the same generated facades; model assertions unchanged.

Semantics preserved: separate player/time cursors, first numeric duplicate wins while
retaining losing labels for validation, initial state replacement, constraint-end base
versus current-diagram message base, bounded axis/signal validation, ordered task aliases,
seven-open-day civil calendar, half-open bars, last-occupied end milestones, offsets,
explicit predecessor durations, cycles and freeze-after-reference. Scenes are byte-unchanged.
Assigned robust/concise track-name repair has private and public model/connection controls.
Assigned trailing Gantt comment EOF repair was already shared-fixed; existing public
terminated/unterminated control passes without inventing a second RED.

RED `/tmp/ive-task12-red.log`: new timing/Gantt facade contracts failed2 with explicit
unavailable errors (three unrelated generated controls passed). After real grammar,
`/tmp/ive-task12-adjacency-red.log` failed2 on newly accepted `@ V` and `[A]as[B]`;
lexical spacing/generated adjacent name-at production restore the previous exclusions.
Interim Gantt diagnostic omitted exact “Project starts” reason; restored. A40k timing
code initially hit a new16KiB atom cap; code/source bounds were corrected so the unchanged
expanded-model-text boundary control passes. Initial generator binding/optional-at
ambiguity and unused line-field Clippy issues were fixed before final checks.

All Cargo/Cargo-spawning commands used OUTERMOST /tmp/ive-diagram-build-locked.sh,
--locked --offline and shared engine target. Private generator cwd/tmp, absolute
assigned manifest, target /mnt/data/Projects/ive/target/diagram-generator-time.
Final command: `cargo test --locked --offline -p diagram-engine --lib --test plantuml_timing --test plantuml_gantt --test plantuml_constants --test plantuml_control --test plantuml_styles --test plantuml -- --test-threads=1`.
`/tmp/ive-task12-tests-final.log`:414 pass236units+43common+44constants+25control+
17Gantt+26styles+23timing; zero failures. Direct generated controls cover EOF, six
delivery/action cancellation points per family, tiny source/depth/work/cancellation,
full offending statements and syntax/semantic expansion lineage mapped exactly once.
Shared From<RenderError>→terminal ParseFailure has focused category/location preservation
tests plus public exact model-limit Backend text and line4/column1 semantic error controls.

Strict engine Clippy passes `/tmp/ive-task12-clippy-final.log`; fmt check passes empty
`/tmp/ive-task12-fmt-final.log`. Existing vendor atomic deprecation warnings remain.
Private generator `cargo run ... -- check`:13 reproducible parsers, no tracked writes,
`/tmp/ive-task12-generator-final.log`. `node tools/audit-plantuml.mjs`:220MIT/43fixtures/
43modules, `/tmp/ive-task12-source-final.log`. Generator closure check67 passes
`/tmp/ive-task12-closure-final.log`; no dependency/lock/vendor/default-limit edits.

Actual exhaustive witness `/tmp/ive-task12-stack-proof.py` → `/tmp/ive-task12-stack-proof.json`:
timing121states/14terminals/149stacks/max10live/4reductions/9expected/diagcap16/growth24;
Gantt131states/20terminals/143stacks/max12live/4reductions/7expected/diagcap12/growth0.
Admission accounts actual token/literal/adapters, concrete symbol/payload sizes,
generated/diagnostic vector growth, every terminal simulation and semantic vector/BTree
storage before generated allocation; syntax copies spend work separately from model text.
Typed sites keep late validation linear; failure-only bounded source slices add offending
row text without reparsing. Full proof is appended in provenance/plantuml-parser-admission.md.

Timing grammar SHAfacf164b0b98e353b582b0bfb61fefa034ad6ab95a77e283fb2e279d3af43947;
generated SHA176a5b538e7acd0c84e6fd9a1a2d7100e989b3ec85f1fcc7e6b32b18e819156f.
Gantt grammar SHA24a60c9cfe97a9d628e501f7f7a6c6688344e558bafff51722384295f49f35d8;
generated SHAecc79fcca2ac4e6f6912475271169171e410478b691241baa98e10602d409f06.
Immutable43 baseline SHAa14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe unchanged.
No native instance/full16/baseline driver was run; those remain CENTRAL ONLY.
Residuals: decimals/named time aliases/clock/analog/binary/special states/colors/notes,
advanced calendar/resources/mixed units/percent/printscale and pixel parity remain excluded.
Broad review deferred by owner; root coordinates integration with YAML/Salt and final gates.

## Central integration checkpoint

Integrated tested Task12 headf5f07f9 into central base474a52c as merge
`6664c863b28838067f831658803a75bdf4467f8d`. Sole central writer; progress.md untouched/unstaged.
Five conflicts resolved by union: plantuml/mod.rs, parser/mod.rs, generated/manifest.json,
provenance/plantuml-parser-admission.md and provenance/plantuml-parser.json.
Source provenance and shared error conversion merged automatically. All Chen/JSON/HCL,
UML/activity/state/tree/files/board/sequence routes, declarations and tests were retained.
Generated manifest/output mechanically regenerated with private integration target/cwd/tmp,
absolute central generator manifest; no generated Rust or inventory was hand-edited.

Final focused command, outer-locked/shared engine target/--locked --offline:
`cargo test --locked --offline -p diagram-engine --lib --test plantuml_timing --test plantuml_gantt --test plantuml --test plantuml_constants --test plantuml_control --test plantuml_styles --test plantuml_chen_archimate --test plantuml_tree_data --test plantuml_inventory -- --test-threads=1`.
`/tmp/ive-integration4-tests.log`:505 passed268units+43common+13Chen+44constants+
25control+17Gantt+27inventory+26styles+23timing+19tree/data, zero failures.
Strict engine Clippy passes `/tmp/ive-integration4-clippy.log`; fmt check passes empty
`/tmp/ive-integration4-fmt.log`; git diff --cached --check passes before merge commit.
Private integration generator generate/check:16 reproducible parsers, no tracked check writes,
`/tmp/ive-integration4-generate.log` and `/tmp/ive-integration4-generated-check.log`.
Exact authorized-source audit220MIT/43fixtures/43modules passes
`/tmp/ive-integration4-source-audit.log`; existing vendor deprecation warnings remain.
`/tmp/ive-integration4-source-equality.log`: eight Task12 family/model/grammar/generated
files byte-matchf5f07f9; ten Chen/JSON/HCL model/parser/generated files byte-match474a52c.
Timing/Gantt scenes and immutable43baseline SHAa14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe unchanged.
No blocker, native instance, broad review or full16/native/baseline-driver run. Paused after
integration; root owns those remaining combined gates and will coordinate Task14 separately.
