# Task 4A — narrow Mermaid compatibility repairs

Base: `c3486fd0752681f18a06061468eb9794d7cfda58`. Implemented only standalone Mermaid stage A; PlantUML, IVE and controller-owned plan/analysis files untouched. No subagents or self-review. Fresh controller review remains required.

Commit: `4f80a73236186f602212bee8a669709f7c66aeb4` (`Repair Mermaid depth coloring and coherent SVG font styles`). Post-commit source audit passes; worktree contains only the two controller-owned dirty plan/analysis documents. All committed paths are under `diagram-ports/`.

## Changes

- Released Mermaid12.2 `mindmap.colorBy: depth` now changes the consumed typed semantic model **before layout**. Controlled iterative traversal carries structural parent-child depth, preserving branch defaults, root section, eleven-slot palette cycle, decorations and child-edge section projection. The existing renderer consumes those corrected sections normally.
- Generic SVG usecase labels measure and paint with their effective inherited global/theme font, matching the original generic SVG label path. Actor/HTML-specific font handling and authored node font overrides are retained.
- Tree emits the same existing validated `config_font_family_css` list used for layout, rather than omitting paint family. Quoted commas, individual quoted names and generic fallbacks remain separate CSS list entries. No full-list quoting or cosmetic SVG font replacement.
- API signatures/options unchanged. Runtime metadata now reports `merman 0.9.2 + local compatibility patches`, target `Mermaid 12.1.0 baseline + 12.2 depth coloring`; diagnostics identify font repairs and explicitly deny full12.2 parity.
- Only the published `merman-core` and `merman-render` crates are patched. All four changed upstream files carry prominent local-modification notices. Other backend packages remain registry-pinned.

## Provenance / licensing

Exact published version0.9.2, VCS `8e303a43b548c9e40e73f0bc657925d66e7d4fb5` retained in original metadata. Registry archive SHA-256:

- core: `f1d2afe2084c0223b772756489a5da51ad0ecbd195aef90268f3037648738796`.
- render: `321c4ca6f834e07b01790c1bc2591dc3a51f450c56c4c71a8111000a50efec0f`.

`diagram-ports/vendor/IMPORTS.json` records all **656 imported original file hashes** before editing. Controller independently checked these against the exact cached archives. `PATCHES.json` records the four changed paths, original/final hashes and purpose; `vendor/README.md` explains source/spec and repair boundaries. Only irrelevant archive Cargo.lock/examples are omitted; original manifests, MIT/Apache legal texts, embedded-source manifests and all corresponding required license texts retained. `.gitattributes` disables vendor line-ending conversion so source hashes survive checkout.

Cargo patches preserve package versions; original checksums are preserved in IMPORTS even though Cargo.lock necessarily removes registry source/checksum for path packages. Selected feature closure remains defaults-off `all-diagrams`, `svg`, `layout-cytoscape`, ELK/math/native ambient adapters OFF. No GPL/EPL implementation inspected/imported. Optional Merman-owned ELK bridge source belongs to its permissive archive; the EPL ELK backend is not selected. No candidate PlantUML code read/imported.

Dependency notices regenerated with cargo-about for the updated lock; legal texts and embedded-source attribution retained. Node source-audit tool verifies complete inventory and exact unchanged/explicitly patched hashes; three executable checks cover acceptance, unrecorded mutation and unexpected-file rejection in disposable copies. Neither Node nor source-audit filesystem access is a runtime dependency.

## RED / GREEN

Before production edits:

```sh
env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test -p diagram-engine --test compatibility -- --nocapture
```

Observed RED: **2 passed / 7 failed**, with failures for typed nodes/edges, depth palette cycling, uneven indentation/decorations, insertion stability, public SVG sections, effective usecase font and tree family declaration. The branch-default and authored per-node font override controls passed.

First post-patch run:7 passed/2 failed due newly reached test-helper assumptions (label split across tspans and CSS family quoting/spacing). Tests were corrected to inspect combined label text and CSS tokens, not change the intended behavior. Final covering GREEN: **11/11**. Additional controls cover non-default quoted-comma font lists/generic fallback and injected font CSS removal. Expectations are hand-derived sections/styles, not oracle dimensions or production helper outputs.

## Final verification

All commands from `diagram-ports/`; all cargo commands use `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`:

- `cargo test --workspace`: **96 Rust tests passed**, plus **7 worker-fault scenarios passed** (blocked stdin/stdout, stderr flood, oversize frame, early exit, memory, cancellation). Includes37 family SVG/XML/usvg fixtures,21 engine safety/behavior tests,11 compatibility tests,15 viewer unit tests,4 export tests,2 cache-lifetime tests,6 normal worker tests. No ignored/failed tests; final full rerun after source notices/diagnostics changes passed.
- `cargo clippy --all-targets -- -D warnings`: exit0; workspace code clean.
- `cargo fmt --all -- --check`: exit0. `rustfmt --check --edition 2024` for all four patched upstream files: exit0.
- `cargo build --release`: final exit0 (31.46s).
- `cargo deny --config ../deny.toml check licenses sources`: `licenses ok, sources ok`.
- `cargo about generate --config ../about.toml ../tools/third-party-notices.hbs --offline --locked -o /tmp/ive-diagram-notices-4a.md`: exit0; generated dependency section mechanically refreshed, normalized LF/trailing whitespace; standalone/embedded notices preserved.
- `node tools/audit-merman.mjs`: `Verified 656 imported files; 4 explicitly recorded source patches.` Final ledger includes modification notices.
- `node --test tools/audit-merman.test.mjs`: **3 passed**; disposable-copy tampering rejected.
- `cargo tree -e normal,build`: local core/render selected, no ELK/ratex backend dependency. Existing unrelated core_maths/palette_math numerical crates remain.
- `git diff --check -- .` and final `git diff --cached --check -- . ':(exclude)vendor/**'`: exit0 for task-owned edits. Unmodified imported files retain original bytes/formatting; only the four deliberate source patches were formatted.

Known untouched upstream warning now visible because local path dependencies are not capped like registry dependencies: `vendor/merman-render-0.9.2/src/environment.rs:585` uses `AtomicU64::fetch_update`, deprecated under installed Rust1.99 in favor of `try_update`. It is not a clippy failure. Renaming would unnecessarily raise the original crate's advertised Rust1.95 compatibility; no unrelated source patch made. cargo-about also reports the existing proprietary workspace crates have no license file; dependency audit still passes.

## Real runtime / original evidence

Real background viewer captures (each `timeout30s`, actual native SVG renderer/worker, exit0 and automatic quit):

- `--example tree --capture /tmp/ive-diagram-4a-tree.png`:10 explicit deterministic fallbacks; no false native bbox claim.
- `--example usecase --capture /tmp/ive-diagram-4a-usecase.png`:3 explicit deterministic fallbacks.
- `/tmp/ive-mermaid-12.2-probe/input/mindmap-depth.mmd --capture /tmp/ive-diagram-4a-depth.png`:25 native measurements/15 explicit fallbacks; same-depth nodes across branches share colors. Final capture confirms patched-backend metadata. Images inspected. No viewer/worker processes left.

Headless exports of identical explicit Liberation Sans font-init sources:

```sh
/mnt/data/Projects/ive/target/debug/diagram-viewer --render /tmp/ive-mermaid-font-probe/output/usecase.mmd --output /tmp/ive-diagram-4a-usecase.svg
/mnt/data/Projects/ive/target/debug/diagram-viewer --render /tmp/ive-mermaid-font-probe/output/tree.mmd --output /tmp/ive-diagram-4a-tree.svg
cd /tmp/ive-diagram-oracles.JwN5H5
node oracles.mjs --compare font-study/original/usecase.svg /tmp/ive-diagram-4a-usecase.svg
node oracles.mjs --compare font-study/original/tree.svg /tmp/ive-diagram-4a-tree.svg
```

Offline original Mermaid12.2 browser inspection: **all computed font families/sizes/weights now agree** for these two fixtures, no missing labels. Usecase:16px Liberation Sans boundary/usecase,14px Open Sans actor. Tree:16px Liberation Sans, bold directories. This pass did not invoke fixed-output corpus generation or modify reference corpus files.

## Honest limitations

This is not full12.2 syntax/layout/pixel parity. Specialized bbox/wrapping operations remain unsupported by native host and explicitly fall back. No generic advance/line-height result is mislabeled as an ink bbox; no fixture dimensions are introduced. Font config is coherent through measurement and SVG, but fallback metric accuracy remains limited: same-config usecase native viewBox `376 × 167.6` vs original `392.5234375 × 167`; tree `122.08 × 138` vs `134.296875 × 135.078125`. The current real tree capture can clip the final wide label because those fallback bounds underestimate its actual painted width. Existing usecase label-position residual remains; this stage fixes font selection, not all label geometry. Stage4B should retain these measured residuals, not hide them with postprocessing.

No PlantUML backend added; its existing unsupported-language behavior and worker safety remain unchanged. Fresh independent review required before further product implementation.
