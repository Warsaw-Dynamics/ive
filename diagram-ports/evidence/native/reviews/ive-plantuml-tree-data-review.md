SPEC verdict: ❌ Issues found — YAML flow lists bypass the requested nesting boundary and perform quadratic work outside the cooperative work accounting. Flow failures also lose the original source line.

Scope: Task 3C only, base `49838e86eea67155710bf29d07179935e45009bc` to head `2b2355b3b6c9c4bf0558256708d762db9e3356ca`, worktree `/mnt/data/Projects/ive/.worktree/diagram-ports`. File references below are relative to that worktree.

### Strengths

- `diagram-ports/engine/src/plantuml/tree.rs:53`, `:119`; `diagram-ports/engine/src/plantuml/treescene.rs:45`, `:137`: mindmap and WBS have genuinely distinct parent policies and geometry. Independent mindmap side stacks/multiple roots are separate from WBS's global chain, one root, ordered first-level fork and descendant trunks. Pseudo nodes preserve topology without painting a box.
- `diagram-ports/engine/src/plantuml/data.rs:72`, `:100`; `diagram-ports/engine/src/plantuml/yaml.rs:351`: the typed ordered model preserves every JSON duplicate, while YAML replaces in the first key position. The JSON parser checks depth before descent, bounds numeric/string lexemes and requires complete input consumption.
- `diagram-ports/engine/src/plantuml/yaml.rs:28`, `:42`: a separate capability pass precedes value-model construction; literal-block context and quoted text are considered. The public alias-bomb test exercises early UnsupportedFeature rejection with model_items=1.
- `diagram-ports/engine/src/plantuml/datascene.rs:128`: child connectors start at the corresponding row center, retaining scalar rows between containers. Private geometry assertions inspect actual coordinates and owner identities, rather than merely looking for labels.
- `diagram-ports/engine/src/plantuml/text.rs:72`, `:124`; `diagram-ports/engine/src/plantuml/svg.rs:109`; `diagram-ports/viewer/src/svg/tests.rs:5`: label mode participates in cache identity, the chosen font family survives into serialization, and the native regression inspects actual text paint bounds with a positive inspected-text count. The unchanged callback section checkpoints both before and after measurement.
- `diagram-ports/provenance/LICENSE-PlantUML-MIT.txt:86`; `diagram-ports/tools/audit-plantuml.mjs:45`; `diagram-ports/tools/audit-plantuml.test.mjs:46`: EclipseSource's separate minimal-json notice and full MIT grant are retained, and the audit has an explicit negative copyright regression. No new runtime dependency, worker protocol or external rendering path appears in this diff.

### Critical

- None found.

### Important

1. **YAML flow elements exceed the caller's nesting limit.** `diagram-ports/engine/src/plantuml/yaml.rs:381` checks only the array's `depth`; the element construction paths at `:413` and `:433` never check `depth + 1`. With `nesting_depth = 2`, `@startyaml\na: [one]\n@endyaml` succeeds even though the root object, array and scalar occupy three model levels. A focused probe against the fresh built engine confirmed success; equivalent block YAML (`a:\n  - one`) and JSON (`{"a":["one"]}`) correctly fail with nesting depth 3 exceeding limit 2. This contradicts the required preconstruction depth boundary. Check the element depth before accumulating/constructing each flow scalar, including the mandatory empty-string element of `[]`, and cover the exact limit and one-below cases.

2. **Whitespace-only flow fields trigger uncharged quadratic scans.** `diagram-ports/engine/src/plantuml/yaml.rs:406` evaluates `field.trim().is_empty()` for every unquoted input character, before checking whether that character is a quote. A field containing n spaces rescans prefixes of lengths 0 through n−1 while the surrounding loop charges only constant work/text per character. This admits O(n²) parser work under an O(n) reported work budget. Focused 4,000/8,000/16,000-space inputs succeeded and reported 40,126/80,126/160,126 work units, with approximately 35/94/320ms elapsed in the existing debug engine. Timings are illustrative; the quadratic loop follows directly from the code. Source caps and the wall-clock/worker deadlines still limit the damage, but do not satisfy the independent work-budget contract. Track whether the field contains a non-whitespace character, or check for a quote before performing at most one necessary prefix trim per field. Add a focused long-whitespace regression that verifies bounded processing without relying on a fragile elapsed-time assertion.

### Minor

1. **Flow diagnostics point at line 1 instead of the failing source row.** `diagram-ports/engine/src/plantuml/yaml.rs:425`, `:431` hard-code line 1. The focused source `@startyaml\na: x\nb: [one,[two]]\n@endyaml` reports the unsupported nested flow container at line 1 rather than line 3. Pass the owning token's line number into `flow` and retain it in both errors; add a non-first-row error assertion. This is a narrow gap in the original-line diagnostic requirement.

2. **Reported inherited validation noise remains.** `.superpowers/sdd/2026-10-22-diagram-ports-plan/task-3c-report.md:111` discloses the existing audited vendored Merman `Atomic::fetch_update` deprecation warning. It is not introduced by this task and is not a reason to expand this patch, but the test evidence is not entirely warning-free.

### Focused checks and review boundary

- Read the prepared 4,226-line package in sequential manageable passes. No raw git derivation, checkout/index/HEAD mutation, product edits or subagents. Extracted line-number anchors from the package after reviewing it.
- Named risk: shared allocation/work controls might already account for the new flow loop. Read only `diagram-ports/engine/src/plantuml/budget.rs`; its `charge`/`text` methods charge the supplied units and do not account for prefix rescans. This confirms Important 2.
- Named risk: the shared label contract might omit cancellation around callbacks. The diff cuts the measurement function between hunks, so read only `diagram-ports/engine/src/plantuml/text.rs:95` through `:135` to inspect that missing section. Both checkpoints remain present.
- Named risk: scalar-list block markers might be misclassified. Consulted the supplied `/tmp/ive-plantuml-next-semantics.md` and only the authorized MIT `yaml/parser/YamlLine.java:126` and `YamlParser.java:118` paths. Bare scalar-list `- |` is literal upstream; keyed list-item blocks are the unsupported construct. No finding was raised for the bare scalar form.
- Executed only `/tmp/ive-task3c-review-probe.rs`, linked with `/mnt/data/Projects/ive/target/debug/deps/libdiagram_engine-61d0c8a2392931a0.rlib`, to investigate the specific depth, line-number and whitespace-work doubts. The probe and executable are under `/tmp`; no product test file was added. The second execution added three small whitespace cases. No workspace suites, audits or captures were rerun.
- Ordinary shell execution encountered the stated bwrap loopback setup failure; scoped elevated read/probe commands succeeded. No approval rejection occurred.

### Cannot verify from this diff

- ⚠️ Worker hard 10s/2GiB limits, stale-job handling and all unchanged defaults reside outside this task's hunks. Their preservation is consistent with the changed-file set, and the controller reports fresh independent worker validation; this review did not reopen that implementation or repeat those suites.
- ⚠️ Source/artifact hash identity and all four capture hashes cannot be independently established by the diff text. The manifest/notice/audit changes were reviewed; the controller's fresh exact-hash and capture checks supply that evidence. No broader source crawl or duplicate audit was performed.
- ⚠️ The reviewer cannot reconstruct historical source-reading provenance or RED-before-GREEN ordering from the final diff. The implementer's report records those claims; the review does not independently certify that history. Full syntax or pixel parity is explicitly outside this checkpoint.

QUALITY verdict: Needs fixes. The family separation, typed ordered data and scene/font contracts are sound, with substantive semantic and geometric tests. The two YAML flow-parser boundary failures prevent approval of the required bounded engine contract; fix them and verify the focused boundaries before releasing later work.
