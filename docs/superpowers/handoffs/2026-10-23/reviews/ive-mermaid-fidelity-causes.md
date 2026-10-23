# Scoped Mermaid font fidelity causes — 2026-10-22

Read-only investigation of the **usecase 16px/12px** and **tree sans/serif** gaps. No product files changed; no GPL/EPL implementation inspected. Permitted sources: Mermaid 12.2.0 checkout `/mnt/data/Projects/mermaid` at `6d8921f94f9a12b76586eddcf45a566080d4767e`, and published MIT Merman 0.9.2 (declares Mermaid 12.1.0 compatibility). This does not establish an upgrade fixes either issue.

Source prefixes below:

- `M/` = `/mnt/data/Projects/mermaid/packages/mermaid/src/`.
- `R/` = `/home/kb/.asdf/installs/rust/1.99.0/registry/src/index.crates.io-1949cf8c6b5b557f/merman-render-0.9.2/src/`.
- `C/` = the sibling `merman-core-0.9.2/src/`.
- Oracle = `/tmp/ive-diagram-oracles.JwN5H5`; baseline `/tmp/ive-diagram-oracle-report.md` and `native-comparison.json` there.

## Conclusions

**There are coherent configuration/API workarounds for these fixtures; neither requires changing only SVG text after incompatible measurement.** The current wrapper's `RenderOptions` does not expose trusted Merman site configuration, so the tree workaround would require an adapter change. For general source-font fidelity, a small backend emission fix is cleaner than a fixed site stylesheet. Neither workaround makes the entire diagram geometrically identical with the deterministic fallback measurer.

1. **Usecase:** a legitimate `usecase.usecaseFontSize:16` and `usecase.usecaseFontFamily:"Liberation Sans"` configuration reaches both layout measurement and SVG emission. A source init directive already works through the current wrapper. A compatibility adapter could supply the equivalent resolved configuration before layout; a fixed 16px override is only a fixture/default-profile workaround, not a general reproduction of Mermaid's CSS cascade.
2. **Tree:** `fontFamily`/`themeVariables.fontFamily` already reaches layout but is omitted from SVG. Trusted site `themeCSS` targeting the two tree text classes, paired with the **same** family in layout configuration, fixes paint without changing the measurement contract. Source `themeCSS` is deliberately protected by hardened security and must remain so. A backend fix emitting the resolved family is preferable when arbitrary per-source font init must work automatically.
3. **Measured residual with a demonstrated operation cause:** tree viewBox height and width discrepancies can be eliminated independently through `RawBBoxHeight` and `RawBBoxWidth`, respectively. These must be real bounded host font measurements in production; the differential test below substitutes recorded browser measurements solely to establish causality.

## Usecase: actual CSS cascade differs from Merman's font plan

Mermaid's `M/styles.ts:42–45` emits the shared root font family/size. `M/rendering-util/rendering-elements/shapes/usecaseEllipse.ts:14–16` uses generic `labelHelper`; `.../shapes/util.ts:74–78` creates `g.label`, and `:93–105` calls `createText` with empty extra classes for plain labels. In the SVG-label fixture this does **not** match `.usecase-element .nodeLabel` or `.usecase-label` in `M/diagrams/usecase/styles.ts:276–282`. Those selectors contain the usecase-specific 12px/font-family custom properties, but the generic SVG label inherits the shared 16px font instead. This is observable behavior, regardless of whether the upstream selector mismatch is intended.

Merman does not reproduce that effective SVG cascade: `R/usecase/measure.rs:39–56` explicitly selects `usecaseFontSize` default **12**, `usecaseFontFamily` default `"Open Sans", sans-serif`; `:327` uses that style in the node label plan. `R/svg/parity/usecase.rs:88–89,125–131` then emits the same plan family/size as inline group CSS. Thus native measurement and paint agree with each other, but disagree with Mermaid 12.2's generic SVG label. Global font init alone cannot override the separately selected usecase font.

Executed the same source with this legitimate prefix in both implementations:

```text
%%{init: {"fontFamily":"Liberation Sans","themeVariables":{"fontFamily":"Liberation Sans"},"usecase":{"usecaseFontSize":16,"usecaseFontFamily":"Liberation Sans"}}}%%
```

Native host-callback trace changed `Wrap/Wrapped "Place order"` from font **12/Open Sans** to **16/Liberation Sans**. Its SVG `g.label` changed correspondingly; browser inspection confirmed the actual computed font matches the original's **16px Liberation Sans**. Native viewBox changed from `-12 -12 376 163.2` to `-12 -12 376 167.6`, proving layout was rerun with the larger style. Original remains `-12 -12 392.5234375 167`. Actor remains 14px/Open Sans in both; boundary title is 16px/Liberation Sans in both. No missing labels.

The remaining **16.5234375 width / 0.6 height** viewBox differences are not fixed by the style override. The callback used here intentionally declines `Wrapped`, so deterministic fallback remains. This experiment identifies the relevant callback and style, but does **not** isolate every usecase residual to that operation; do not promise that a generic text-width replacement alone fixes them. No wrapping regression is established by this short single-line fixture. General 12.2 parity needs effective SVG-label style selection (including node/class/inline overrides and HTML-vs-SVG paths), not a blanket 16px replacement of all usecase-family text.

## Tree: measurement family is honored; SVG family is missing

Upstream tree-specific CSS relies on shared root CSS in `M/styles.ts:42–45`. Native `R/tree_view/config.rs:47–48` resolves global/theme family; `R/tree_view.rs:34–44` builds normal/bold/italic measurement styles from it. A recording host callback confirms explicit source init produces **Liberation Sans, 16px**, with bold directory requests. However `R/svg/parity/tree_view/render.rs:180–195` emits size/color/weight/style but **no font-family**, nor does its root SVG opening supply one. A standalone SVG therefore falls back to browser Times New Roman (the oracle's actual platform font is Liberation Serif), even though it was laid out using the configured sans family. This is an emission omission, not a GPUI font-discovery failure.

Supported Merman API workaround, tested directly with `Engine::with_site_config`:

```json
{
  "fontFamily": "Liberation Sans",
  "themeVariables": {"fontFamily": "Liberation Sans"},
  "themeCSS": ".treeView-node-label, .treeView-node-description { font-family: Liberation Sans; }"
}
```

`R/svg/parity.rs:1444–1466` applies trusted effective `themeCSS` through `ScopedCssPostprocessor`; the terminal `SvgPipeline::resvg_safe()` still runs. Callback requests remain Liberation Sans, and browser inspection now confirms every tree label paints Liberation Sans with the appropriate weight. This is coherent only while the stylesheet family equals the **effective** layout family. Do not install a fixed Liberation Sans stylesheet and still allow a source init to change measurement to another family. `C/config/mod.rs:16–24` keeps `themeCSS` secure against source overrides; do not weaken that policy.

Minimum general fix: have the tree emitter include the same safely resolved family used by `config_font_family_css` (`R/config.rs:153–158`), at the root or on both text classes. An adapter can alternatively derive a narrowly scoped validated font declaration from the prepared semantic effective config and pass it via the SVG pipeline, coupled to measurement of that exact config. Current wrapper `engine/src/mermaid.rs:51–65` fixes site config internally; its public options expose measurement but not site CSS/config. Source font init alone cannot repair the missing family declaration. A backend upgrade is not inherently required; a small audited patch or coherent adapter is sufficient.

## Tree box residual: differential proof, not guessed font tuning

`M/diagrams/treeView/renderer.ts:82–94` measures direct SVG `<text>.getBBox()` and derives row height from bbox height + 10px padding. The original five labels each have bbox height **17.015625**; rows are **27.015625**, yielding total **135.078125**. Native `R/tree_view.rs:172–178,266–272` requests `SvgBBox/RawBBoxWidth` and `SvgBBox/RawBBoxHeight`. Its fallback raw-height implementation (`R/text/measure.rs:164–172`) delegates to generic measure; deterministic SVG-like measurement uses `font_size * 1.1` (`R/text/deterministic.rs:664–685`), hence **17.6**, rows **27.6**, total **138**.

Executable isolation:

- Returning **17.015625 only for `RawBBoxHeight`**, leaving every other operation on deterministic fallback, changed native viewBox from `-0.5 0 122.08 138` to `-0.5 0 122.08 135.078125`: the entire height residual disappeared without altering font size or family.
- Additionally returning the original **94.296875 only for `RawBBoxWidth("package.json")`** changed width to **134.296875**, exactly the original. This last label determines the fixture's rightmost extent. Other widths still use fallback; this is not proof every node bbox matches.
- With the coherent trusted tree CSS plus these two diagnostic substitutions, original and native viewBoxes both equal `-0.5 0 134.296875 135.078125`; computed label fonts/weights agree, 8 geometry elements each, no missing labels. This is structural/font/bounds evidence, **not pixel parity** and not a production metric implementation.

Consequently the real GPUI adapter should implement these distinct raw-bbox operations, including correct normal/bold family selection. Mapping `RawBBoxHeight` to a generic line advance reproduces the demonstrated residual. No other wrapping/box cause is asserted here.

## Reproduction and artifacts

Temporary, standalone diagnostic source: `/tmp/ive-mermaid-font-probe/{Cargo.toml,src/main.rs}`. It uses exact Merman 0.9.2 plus the product engine, records host requests, and normally returns `Ok(None)` to retain deterministic fallback. Its two oracle-value substitutions are explicitly diagnostic. Outputs and full callback traces: `/tmp/ive-mermaid-font-probe/output/` (`tree`, `tree-raw-height`, `tree-site`, `usecase`, `usecase-override`). No runtime dependency on this oracle is proposed.

Executed successfully:

```sh
cargo run --offline --manifest-path /tmp/ive-mermaid-font-probe/Cargo.toml
# 5 renders; requests and SVGs: /tmp/ive-mermaid-font-probe/output
cd /tmp/ive-diagram-oracles.JwN5H5
node oracles.mjs --mermaid /tmp/ive-mermaid-font-probe/output/usecase-override.mmd /tmp/ive-mermaid-font-probe/output/usecase-override-original.svg
node oracles.mjs --compare /tmp/ive-mermaid-font-probe/output/usecase-override-original.svg /tmp/ive-mermaid-font-probe/output/usecase-override.svg
node oracles.mjs --compare corpus-mermaid/tree.svg /tmp/ive-mermaid-font-probe/output/tree-site.svg
node oracles.mjs --compare corpus-mermaid/tree.svg /tmp/ive-mermaid-font-probe/output/tree-raw-height.svg
```

These use the existing offline Chromium oracle only for investigation, never as the native runtime. No product commit or changes made; worktree status shows only controller-owned viewer/manifests and PlantUML analysis work.
