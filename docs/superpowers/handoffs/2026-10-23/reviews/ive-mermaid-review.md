# Task 1 independent review

Reviewed product range `51ac3af..29353b5` in `/mnt/data/Projects/ive/.worktree/diagram-ports`, using the supplied task brief, implementer report, and review diff. No product files were changed, no commits were made, and no subagents were used. The supplied diff output was truncated; focused source/fixture reads completed the relevant portions rather than regenerating a diff. Bulk generated legal boilerplate was skipped after checking its source manifests, release provenance, and selected-feature risk.

**Spec compliance verdict: Changes required.** The standalone interface, broad family admission, native backend pin, safe SVG path, and documented compatibility limits substantially satisfy Task 1, but the resource guard adds an unapproved restriction on harmless Mermaid source.

**Code quality verdict: Changes required.** One important correctness defect blocks integration. The implementation is otherwise small and reasonably separated, with a credible bounded upstream execution path. Minor test improvements are listed separately.

## Important finding

### P2 — Detect actual resources without rejecting ordinary labels and comments

Location: `diagram-ports/engine/src/mermaid.rs:64` (especially lines 65–85), invoked before parsing at line 24.

The guard searches the entire lowercased source for substrings such as `data:`, `file:`, and `://`, and removes all whitespace before searching for `img:`. These matches do not establish an external resource. In particular, `data:` is a suffix of the ordinary word `metadata:`. Valid, inert Mermaid text consequently fails before the parser can distinguish text from resource syntax.

Confirmed against the newest already-built engine artifact with a small `/tmp` probe; the existing test suite was not rerun:

```text
flowchart LR
 A["metadata: loaded"] --> B[Done]
=> ExternalResource("data:")

flowchart LR
 %% See https://example.invalid/docs
 A[Start] --> B[Done]
=> ExternalResource("://")

flowchart LR
 A["https://example.invalid/docs"] --> B[Done]
=> ExternalResource("://")
```

This affects all families and common labels/comments, not only deliberately unsupported images or navigation links. The controller confirmed the accepted design permits refusing actual external resources and external effects, but does not accept blanket rejection of inert source text. Disclosure in `diagram-ports/README.md:33` and the implementer report does not amend that requirement.

Required correction: validate resource-bearing source constructs or parsed values while preserving literal labels/comments. Keep the terminal SVG resource checks and the refusal of actual external images/links/CSS resources. Add regressions admitting these harmless examples while retaining the actual-resource failures. Avoid replacing the guard with another global token heuristic.

## Scope and implementation assessment

- Public synchronous, headless interface and explicit PlantUML unsupported result match Task 1 (`engine/src/lib.rs:15`, `:45`, `:54`, `:66`, `:106`). No GPUI dependency or IVE product changes are present.
- `diagram-ports/Cargo.toml:11` pins `merman = 0.9.2`, disables defaults, and selects `all-diagrams`, `svg`, and `layout-cytoscape`. Locked offline metadata and the normal dependency tree agree with the selected native path; no ELK/math/export runtime is selected. Published `.cargo_vcs_info.json` confirms revision `8e303a43b548c9e40e73f0bc657925d66e7d4fb5`. The Mermaid 12.1.0 target is clearly identified rather than claiming 12.2 equivalence.
- The selected source manifests retain Mermaid MIT, DOMPurify's Apache option, sanitize-url MIT, Dagre/Graphlib MIT, and Manatee's MIT algorithm provenance (`THIRD-PARTY-NOTICES.md:3006` onwards). Fixture provenance names a concrete Mermaid revision and documentation paths (`fixtures/mermaid/README.md:3`). No GPL/EPL implementation was consulted for this review. No provenance defect found within the inspected scope.
- Explicit deterministic configuration and strict parse/runtime policy are installed (`engine/src/mermaid.rs:51`). Source-requested ELK is rejected rather than silently substituted (`:30`, `:90`). The admitted specialized layouts are distinguished from graph Dagre defaults in the documentation.
- The 37 fixture variants exercise every admitted family selector, with the four railroad dialects separately tested (`engine/tests/families.rs:48`). Expected text, finite positive viewBox, text elements, and usvg 0.46 parsing are stronger admission evidence than nonempty output. They establish representative family support, not complete grammar or visual equivalence. Version, ELK, math, native fonts/measurement, and geometry gaps are honestly reported.

## Limits, cancellation, SVG and stylesheet checks

- Source bounds are checked before preprocessing; caller limits are validated against finite maxima and a finite upstream profile (`engine/src/limits.rs:4`, `:20`). Render policy reaches both semantic preparation and SVG execution (`engine/src/mermaid.rs:25`, `:35`). Upstream source inspection confirmed SVG byte and structural preflights before custom passes, after passes, and during terminal finalization, with cooperative checkpoints. Thus the SVG cap is not merely an after-allocation length check in `engine/src/svg.rs:73`.
- The per-call child control preserves parent cancellation while giving each request its own relative deadline (`engine/src/mermaid.rs:17`). Selected native `OperationControl` supports deadlines even without the optional system-timing adapter. Existing tests cover pre-cancellation, an expired deadline, source size, and a small SVG cap (`engine/tests/mermaid.rs:77`, `:133`, `:144`, `:164`). Host callbacks remain trusted synchronous services and cannot be forcibly interrupted; this is documented.
- `SvgPipeline::resvg_safe()` remains the terminal policy (`engine/src/mermaid.rs:39`). Focused upstream inspection confirmed the custom normalizer runs before the unchanged built-in safety stages and final validation. XML stylesheet character data is decoded, then `&` and `<` are re-escaped (`engine/src/svg.rs:40`), so the usecase quote fix does not bypass those later checks. The info viewBox adaptation is narrow (`:28`). No additional blocking normalization defect was demonstrated.
- The extra XML guard rejects active/foreign/image elements, event/src attributes, and non-fragment href values (`engine/src/svg.rs:92`). This guard alone is not a complete CSS/resource validator; the upstream terminal finalizer supplies that missing layer, including structural and CSS references. The selected wrapper installs no math renderer or file/network/process resource resolver (`engine/src/mermaid.rs:35`).

## Minor improvements (not integration blockers independently)

1. Restrict expected-label extraction to rendered text descendants. Both `engine/tests/mermaid.rs:13` and `engine/tests/families.rs:30` concatenate all XML text nodes, including stylesheet/title/description text. The family tests also only require that some `<text>` exists. A future regression could leave the expected label solely in nonpainted metadata while still passing. The tests currently provide reasonable admission evidence, but “visible label” is stronger than the assertion actually proves.
2. Add focused normalization/output-boundary tests beyond the one usecase fixture: quotes, ampersands and XML-sensitive stylesheet content, and external references that reach the terminal validator rather than being rejected by the source-token shortcut. Add model/work limit and in-flight cancellation cases when strengthening this boundary; existing wrapper tests only exercise some of the exposed limits and cancellation before useful work begins.

## Evidence and cannot-verify items

Accepted controller evidence: committed `cargo test --locked --offline` passed 52 tests (37 family fixtures plus 15 behavior tests). The implementer report records clean clippy/format/release-build/license checks. I did not rerun the suite. Review checks used locked offline Cargo metadata/tree, selected cached dependency source reads, exact source/provenance inspection, and the focused temporary compatibility probe at `/tmp/ive-mermaid-review-probe.rs`.

Task 2 must still establish off-UI execution and hard worker timeout/memory/restart behavior. This synchronous engine explicitly offers cooperative limits, not hard isolation; that cross-task requirement cannot be verified from Task 1 and is not counted as a Task 1 defect. Actual host font measurement/painting, original-renderer geometry/raster parity, and future PlantUML behavior are also outside this review's verified scope. Historical RED execution is report evidence, not independently replayed evidence.
