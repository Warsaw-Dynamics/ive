# Controller gate and capture evidence at 3478bbd

Product snapshot: `3478bbdf5d11865414817e5f6a01eb81cd2336da`. Fresh release viewer SHA-256 **`0c43488e42fca26229b00b9fbce13511726c8786c061cef8ce4827e04943bd2d`**. This report records the controller's supplied gate/capture facts. The archive worker checked structured identities, counts and external file hashes; it did not rerun Cargo gates, recapture windows or perform an independent visual/whole-branch review.

## Controller-reported completed gates

| Gate | Recorded result |
| --- | --- |
| Core workspace tests | 858 ordinary Rust tests plus seven actual worker-fault cases passed. |
| All-targets tests | 859 ordinary Rust tests plus seven actual worker-fault cases passed. |
| Strict checks | Clippy, formatting, release build and cargo-deny checks passed. |
| PlantUML source audit | 293 selected MIT source/ZIP hashes, 58 fixture hashes and 58 native modules. |
| Merman source audit | 656 imported files and four explicitly recorded patches. |
| Private generator | 29 reproducible grammars; 67-package source/license/feature/notice closure. |
| Immutable baseline | Frozen 43 source/family/SVG hashes passed without rerecording. |
| Full Node tool suite | 75 tests passed. |

These ordinary, all-target and worker-fault counts are separate results, not additive totals. Runtime tests, source audits and native captures are different evidence kinds.

## Actual gallery and negative captures

The [final gallery manifest](gallery/captures.json) records **72 PNGs** from the exact release viewer above; source `/tmp/ive-gallery-final-20261023a/captures.json`, SHA **`5e371805d21b29fe088cec130171c4fe2773ff7ee537e1dc894b5af3309a306a`**. All35 PlantUML examples have positive native measurements and zero deterministic fallback. Mermaid has positive native measurements in20/37 examples and zero fallback in9/37; fallback-only or mixed cases retain their actual counts.

Controller comparisons: all34 prior PlantUML **canvas crops are pixel-identical**; all37 prior Mermaid **full frames are byte-identical**. Those are different comparison scopes. The controller inspected the new advanced-state full frame. The archive worker independently derived the gallery counts, matched all72 external PNG byte lengths/hashes and72 external log hashes, and compared all37 Mermaid frame hashes against the prior gallery. It did not independently reproduce the PlantUML crop/pixel comparison or the controller's visual inspection.

Both language error panes were captured with exit1 and visually inspected by the controller. Metadata only is retained here:

| External historical image | SHA-256 |
| --- | --- |
| `/tmp/ive-final-invalid-mermaid.png` | `e36d79b575a54d33523f6ee9c5c40abcc9d35a88b83899f759436a19fc5e3061` |
| `/tmp/ive-final-invalid-plantuml.png` | `5f5b9e2c5d3a080529ff7139daac95cb8528c0c2f391f495bc9f3546a10dff8f` |

The archive worker checked both PNG hashes without copying images. Controller headless invalid exports under empty PATH and no display preserved the same existing-output sentinel SHA **`13a38ef5f4f720877b0015c90d46c61a3814f09cd34214a7cd9ed4f67af08c6a`**. Controller cleanup found no surviving viewer. These behavior/visual facts are controller-reported, not inferred from the gallery manifest alone.

## Corpus and state report scope

The [final corpus report](final-corpus-report.md) and exact [source snapshot](corpus/snapshot.json) record95 native positive fixtures:37 Mermaid and58 PlantUML. Original renders94, with the expected ZenUML core-plugin detection error. Complete structured source/status/hash closure and selected fixed-label/state-topology checks are preserved; this is not all-authored-label, full layout, font, glyph-ink or pixel-parity proof. The corpus driver has no family/host-metric telemetry; its deterministic exports do not substitute for the gallery's actual host counts.

The [advanced-state private checkpoint](state/task-3b2-report.md) reports `d42f15aedcefa21ef7710fea984aab54196b7b61`; the [integration/history-stereotype supplement](state/task-3b2-integration-followup-report.md) reports the isolated `cdd1c58e804e8b2553991efa727bf136114c4c4c` repair. Their pending-at-the-time wording is copied verbatim. The controller reports integration and final verification at3478bbd; the supplement's old awaiting-cherry-pick sentence is historical, not the current integration status. No independent whole-branch approval is inferred from either implementation report.

Raw SVGs, PNGs, logs, JARs and vendor source remain external. Absolute `/tmp` and worktree paths inside preserved documents/manifests are historical machine references, not runtime inputs or promises of future availability. The archive's checksum inventory verifies retained structured files; raw corpus/gallery closure was checked while the external artifacts were available. Original renderer nondeterminism, foreignObject differences, missing original ZenUML plugin, date/container differences and native metric/layout limits remain as recorded.

Final chronological Rulings consolidation and whole-branch independent review remain pending. No full upstream compatibility, formula rendering, default-ELK parity or universal native/pixel/font equivalence is claimed.
