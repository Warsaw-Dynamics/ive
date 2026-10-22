# Native diagram ports and standalone viewer

Owner direction, 2026-10-22: attempt full Mermaid and PlantUML ports, in that order; start a new Rust project, use IVE's pgui framework, permit other suitable dependencies, and deliver a standalone viewer before integrating into IVE. The earlier analysis-only handoff remains the technical and licensing reference; this direction authorizes implementation and subagent execution after the owner's three scope answers.

## Outcome and architecture

Create `diagram-ports/`, an independent Rust workspace with a headless `diagram-engine` crate and a `diagram-viewer` application. Neither engine requires GPUI. Source enters a bounded request, passes through an explicitly selected language backend, and produces self-contained SVG plus provenance/coverage diagnostics. The viewer reads caller-selected local sources and renders results using the same pinned `gpui` and `gpui_platform` revision as IVE. These two packages are parts of the owner's pgui framework.

The implementation should build on audited native Rust work where it preserves more upstream behavior than starting over. Evaluate Merman for Mermaid first, with all permitted diagram families enabled and EPL ELK disabled. Evaluate `puml` for PlantUML after Mermaid works. Adoption is contingent on source provenance, permissive dependency closure, offline execution, and bounded behavior. A candidate that cannot meet the runtime boundary must be adapted locally or replaced. Never quietly substitute external commands, JavaScript runtimes, network rendering, or browser rendering.

This is an attempt at broad compatibility, not permission to relabel a small parser as a full port. Maintain an executable family corpus and coverage table; distinguish family admission, semantic correctness, SVG validity and actual upstream geometry parity. Record unsupported syntax and known geometry differences. Mermaid's local checkout defaults to ELK; comparisons against a Dagre backend must explicitly configure Mermaid to Dagre.

## Runtime boundary

The engine accepts source text and caller-provided options/resources. It never reads files, fetches URLs, or starts processes. The standalone host may read exactly the file the user selects. PlantUML includes are rejected initially unless an audited in-memory resolver can enforce the boundary; URL includes are always disabled. Rendering must occur outside the UI thread with stale-result suppression, cooperative deadlines where supported, and a hard process boundary where the selected engine cannot provide bounded execution. Limit source bytes, output bytes and render time; fail with self-contained diagnostics. Untrusted source must not become an SVG external resource request.

SVG must work with resvg/usvg: avoid foreignObject, external CSS/resources and active content. Real host font metrics should be wired where the backend supports them; deterministic metrics are used for headless tests. Any backend with fixed/approximate metrics must state that limitation instead of claiming GPUI measurement parity.

## Viewer

Provide a runnable GPUI application that accepts a source-file path and can show built-in examples without one. Include language/example selection, source visibility, rendering/error state, fit/zoom, and SVG export through an explicit CLI output path. Focus on inspecting diagram compatibility; a Markdown view and IVE integration remain later work. Opening test windows must not steal focus. Provide a deterministic screenshot or equivalent inspectable capture for verification, and stop every test process started.

## Verification and licensing

Follow IVE's coding and test conventions. Use real engine output in tests, parse it as SVG, verify expected labels and nonempty geometry, cover each available diagram family, and test malformed input, Unicode, XML metacharacters, large inputs, denied includes and timeouts. Record exact commands and test results. Use original tools only as development references and never as runtime dependencies. Import only provenance-checked permissive fixtures and preserve their notices.

Pin the engine versions/revisions and pgui revision. Audit selected Cargo features and the complete selected dependency graph against IVE's `deny.toml`. Exclude ELK translations and Smetana/Graphviz-derived EPL code, including code presented inside otherwise MIT-labelled packages. Preserve original MIT/Apache notices and track copied code/resources in standalone notices. Root metadata alone is insufficient evidence of provenance.

## Execution

The controller coordinates subagents and independently checks their work. Finish and review Mermaid before adding PlantUML. Use a fresh review for each implementation deliverable and a final integration review. Keep a persistent ledger with coverage, evidence, remaining work and any decisions made on the owner's behalf. Local implementation and verification are authorized; publishing, pushing, and modifying the owner's running IVE session are outside this task.

## PlantUML adoption gate, 2026-10-23

The audited `puml` candidate was not adopted. Its own pinned design records and source headers explicitly identify style-parser/catalogue implementation lineage from the GPL master repository without a recorded MIT-edition basis. This is an unresolved permissible-source question under the owner's hard rule, not an infringement finding. No candidate source was imported into the project. Dependency metadata and resource hardening cannot resolve that separate question. The [source-provenance audit](../plans/2026-10-23-puml-provenance-audit.md) records the precise evidence and its limits.

Use the fresh [native PlantUML plan](../plans/2026-10-23-plantuml-native-plan.md): typed family models and native scenes translated only from the authorized MIT artifact, with separately audited Merman measurement/SVG services and bounded Dugong/Dagre layout. Preserve source-file hashes and notices. Review meaningful increments, then continue toward the remaining inventory; do not describe the first admitted subset as a full port.
