# Task 4A independent review

Range: `c3486fd0752681f18a06061468eb9794d7cfda58..4f80a73236186f602212bee8a669709f7c66aeb4`.
Checkout: `/mnt/data/Projects/ive/.worktree/diagram-ports`.

## SPEC COMPLIANCE: PASS for stage 4A

The implementation meets the compatibility-repair stage of the task brief. It does not complete stage 4B or establish full Mermaid 12.2 parity.

- Mindmap depth coloring is selected from effective configuration before layout-model projection. The iterative traversal uses structural parent-child depth, retains the root's absent section, cycles through eleven sections, and leaves the branch default unchanged. Child-edge sections/classes continue to derive from the child's assigned section. Decorations and existing cancellation checkpoints remain intact. I compared the change with the MIT Mermaid `mindmapDb.ts` implementation at the specified checkout.
- Usecase SVG nodes now use the generic effective global/theme font while actors and HTML labels retain their existing specialized paths. Authored node styles still apply after selecting the base font, and the same label plan supplies measurement and paint.
- Tree paint obtains its family list from the same helper used by tree layout. Existing source-value admission and font-value validation remain upstream of CSS emission. Quoted names, quoted commas and fallback lists are covered by public-engine tests.
- API shape and selected features remain unchanged. Runtime identity and documentation identify the local patches and expressly disclaim full 12.2 fidelity.
- Provenance and licenses are retained: original manifests and notices, both license choices, embedded-source legal files, import hashes, four-entry patch ledger, and prominent modified-source comments. The lockfile change only replaces registry identities for the two locally patched crates. The selected defaults-off family/SVG/Cytoscape feature set does not newly select ELK or math.

## CODE QUALITY: PASS

No critical or important findings. The behavioral changes are narrow and use existing semantic-model, label-plan and font-configuration paths. Tests inspect typed sections, emitted SVG classes, requested font styles and parsed CSS tokens instead of snapshotting fixture dimensions or reproducing implementation logic. The source audit rejects unexpected files and unrecorded mutations.

### Nonblocking tooling recommendation

`diagram-ports/tools/audit-merman.mjs:33-37`: `--record-import` authenticates the archive checksum but computes `upstreamSha256` from the independently extracted vendor directory. Thus this initialization command alone cannot establish that those recorded bytes came from that archive. Reproduction: in a disposable import workspace without IMPORTS.json, alter one extracted source file and run `--record-import` with the genuine archives; the altered extracted bytes will be recorded as upstream. Prefer deriving the baseline hashes from authenticated archive members, or explicitly checking the extraction against them during initialization.

This is defense in depth, not a blocker for this import: the controller independently compared every one of the 656 original hashes with the exact archives. I independently compared all four patched files with their cached registry originals and checked their original SHA-256 values against the ledger. Do not regenerate the current manifest merely to address this recommendation.

## Evidence inspected and checks performed

- Read task-4 brief, task-4a report, current implementation design/plan, relevant repository instructions, review guidance, all new tests/audit tools, manifests/notices changes, and wrapper/docs changes.
- Compared each of the four original files against the actual cached 0.9.2 registry source. Only the described compatibility edits and modification notices appear in those diffs; all four original SHA-256 values match PATCHES.json.
- Inspected the original depth algorithm, parent/child and edge projection, cancellation checkpoints, usecase label plans/styles, tree layout font selection, CSS emission, and source presentation filtering.
- Fresh local `node tools/audit-merman.mjs`: verified 656 imports and four recorded patches.
- Fresh local `node --test tools/audit-merman.test.mjs`: all three acceptance/mutation/inventory tests passed.
- Additional public CLI probe with a font family containing an ampersand confirmed that existing source admission filters the unsafe XML value and emits valid fallback font CSS. No application window was started.
- The controller reports a fresh locked/offline full Rust pass: 96 ordinary tests plus seven worker-fault scenarios. I did not duplicate that full suite. The task report records successful clippy, release build, license/source checks and native captures; these were inspected as reported evidence, not rerun by this reviewer.

## Declined to judge as completion of this stage

- Full Mermaid 12.2 syntax/layout/pixel parity: expressly outside this narrow stage, and not claimed by the implementation.
- Specialized native glyph-bbox/wrapping accuracy and the recorded wide tree-label clipping: real known residuals that need final-viewer/fidelity work; the current change repairs font selection and does not claim to repair those metrics.
- Existing usecase label-position residual: documented geometry limitation outside these effective-font repairs.
- PlantUML backend implementation: no PlantUML implementation enters this diff; the candidate's provenance rejection and next implementation decision belong to the controller's separate work.
- Final cross-family coverage matrix, corpus comparison tooling and whole-project final handoff: stage 4B deliverables, not missing stage 4A implementation.

## Assessment

Ready to accept stage 4A and proceed to the next planned work. The current source provenance is independently established, and no blocking compatibility or safety regression was found. Preserve the explicit geometry residuals in final coverage documentation; passing this stage does not convert those residuals into parity claims.
