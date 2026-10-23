# Task 3B: Native state machines and structured activities

Read this first: it is the task's requirements. Execute detailed native-plan Task4 as the next reviewed PlantUML checkpoint. Implementation remains HOLD until the controller explicitly releases it from the independently reviewed Task3A commit. Tree/data and later families remain separate increments, not omitted scope.

## Binding constraints

- Owner requests a full-port attempt, other permissive dependencies allowed, standalone pgui viewer first. This task is a useful subset, not a full PlantUML port. Keep IVE's root product unchanged.
- Translate exclusively from `/mnt/data/Projects/plantuml-mit/src`, MIT edition1.2026.9. No GPL repository or rejected puml candidate source/examples; no gen/, h/, smetana/, Graphviz or ELK implementation translation. Record every source file whose behavior/constants are translated, its SHA256, destination and notices in the existing provenance manifest.
- Engine has no GPUI dependency and reads no files, network, environment or clock, and starts no processes. Host retains its own isolated same-binary worker. Preserve public render/options/result/measurement/control APIs, existing Mermaid/PlantUML behavior and truthful diagnostics.
- Default bounds remain source256KiB, SVG16MiB, model10,000, nesting64, work5,000,000, cooperative10s; viewer worker independently enforces10s/2GiB Unix ceiling. Charge new parsing/model/layout/emission and retained text before amplification/allocation; observe shared cancellation. Bound numbers/coordinates and output before appending.
- Use typed state/activity models and native scenes. Do not translate to Mermaid source or reuse a generic sequence/free-graph layout that discards state/control-flow meaning. Reject every unsupported substantive command, feature or decoration explicitly rather than emitting a different/partial diagram.
- One product implementer only, no subagents. Use apply_patch and TDD. Follow AGENTS small-function/parameter conventions and should_ tests with given/when/then. Commit only scoped task files and preserve unrelated changes.

## Scope and tests

1. Integrate state/activity selection into complete-block family admission. Preserve sequence precedence (e.g. actor/database messages), class members named loop, unsupported timing rejection and accurate source errors. Require whole-input consumption; don't silently strip unsupported styles/directives or reinterpret unknown commands as implicit entities.
2. State: named/aliased states, normal transitions with labels, distinct scoped initial/final pseudostates, nested composite ownership and state-body description lines. Initial/final `[ * ]` semantics are scoped and role-specific, not unique per occurrence; inspect the exact `[*]` grammar. Draw genuine initial dot/final bullseye and rounded state/composite shapes. Preserve label/entry/exit attachment and directed endpoints. Concurrent regions, history, choice/fork/join and cross-region constraints must fail until explicitly modeled/tested; don't fake them as ordinary states.
3. Modern activity: start, stop and end distinctions, `:action;`, nested if/elseif/else/endif guards, while/endwhile and repeat/repeat while, fork/fork again/end fork, groups/partitions. Use immutable structured instructions and bottom-up composition with explicit entry/exit ports. Real branch convergence, loop-back direction and separate fork lanes/join must survive lowering. Stop/end paths have no fallthrough into subsequent actions. Unsupported swimlanes, switch/break/kill/detach/backward/legacy syntax gets a descriptive rejection rather than accidental admission.
4. Add semantic/geometry tests for ordered transitions, scoped pseudostate reuse, aliases, nested state ownership and descriptions; if/elseif/else convergence, both loop forms, fork split/join, dead branches, nested partitions and retained guards/newlines/semicolons. Include malformed stack endings, invalid endpoints, unknown syntax, tiny model/depth/work/output bounds and cancellation inside layout. Assert topology and containment, not only SVG labels or family metadata.
5. Admit the exact fresh state/activity fixtures from `/tmp/ive-plantuml-native-oracles` (manifest independently verified), plus original meaningful multifeature cases. Compare actual admitted topology against original MIT oracle semantics and record geometry differences. Read-only grammar notes `/tmp/ive-plantuml-next-semantics.md` are supplementary evidence, not source authority; inspect every authorized source you translate.
6. Add honest viewer examples/file detection as necessary using the existing worker and source panel. Capture a real background nested state and a branched/looped activity; inspect frame contents and host/fallback counts; stop test instances. No font-metric or full geometry parity claim.

Common-source prerequisite: existing UML preparation trims/drops blank lines. Multiline actions must preserve meaningful blank rows and relative whitespace instead of inheriting that transformation blindly. Add raw multiline action and existing multiline-note regressions while changing shared preparation; blank/comment lines outside text bodies still follow their family grammar. Do not erase JSON/YAML/grid payloads in anticipation of later wrappers.

## Existing service contracts

Read `task-3a-report.md` here and current `engine/src/plantuml` modules for the frozen interfaces. Shared `Budget`, `text::Context`, typed `Scene`/`Shape`, terminal safe SVG serializer and checked Dagre adapter are the intended utilities. New state graph semantics remain separate from UML declarations; activity composition remains separate from graph layout. Extend narrow shared APIs only when justified by genuine semantics, with regression coverage.

## Completion/report

Begin with the listed focused RED tests, then implement and verify focused targets followed by full locked/offline workspace tests, strict clippy, fmt, release, selected license/source checks and existing source audits. Keep math/ELK/runtime capability exclusions intact. Report ordinary, private and actual worker-fault counts separately.

Commit scoped implementation/provenance/fixtures/docs after checks. Write `task-3b-report.md` here with exact base/head, supported/rejected syntax, typed-model/layout interfaces, source traces, RED/GREEN commands, tests, native captures/measurements and remaining semantic/geometry gaps. Return status, commit and one-line evidence. Fresh independent spec+quality review is mandatory before a subsequent implementation.
