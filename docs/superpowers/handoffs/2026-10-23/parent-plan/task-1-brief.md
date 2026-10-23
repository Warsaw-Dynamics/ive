### Task 1: Bounded headless Mermaid renderer with broad family coverage

**Files:** Create `diagram-ports/Cargo.toml`, `diagram-ports/engine/{Cargo.toml,src/lib.rs,src/mermaid.rs,src/limits.rs,tests/mermaid.rs}`, `diagram-ports/fixtures/mermaid/`, `diagram-ports/{README.md,THIRD-PARTY-NOTICES.md}`. Additional focused modules are allowed.

**Interfaces:** Produces `Language::{Mermaid,PlantUml}`, `RenderOptions`, `RenderedDiagram` with owned SVG and language/backend identity, `RenderError` with Display/Error, `render(source: &str, options: &RenderOptions) -> Result<RenderedDiagram, RenderError>`. PlantUml initially returns an explicit unsupported-language error. Preserve this public interface for subsequent tasks; publish exact fields/defaults in the task report. Do not add a GPUI dependency to the engine.

- [ ] Audit and pin Merman's selected release/revision, licenses, native API, metrics hooks, runtime I/O and cooperative budgets; use `default-features = false` and the widest verified permissive family/SVG features. Prefer a stable published version if it preserves the required coverage; identify its upstream Mermaid target.
- [ ] Write failing behavioral tests for rendering a flowchart with both node labels, sequence messages, class members, invalid source, XML metacharacters, Unicode, source limits and rejected external resources; observe red before implementing wrappers.
- [ ] Implement the native rendering wrapper, explicit deterministic configuration, descriptive errors, safe SVG checks and limits. Use a built-in Dagre path, reject explicitly requested unavailable layouts rather than silently promising ELK.
- [ ] Add and execute one meaningful fixture per admitted family, using existing permissive source examples with attribution. Record families that fail and the reason; exercise broad coverage rather than stopping at the initial three.
- [ ] Run engine tests and clippy; inspect selected Cargo feature tree for any ELK/EPL package. Record commands/results and backend API/metrics/cancellation facts in the report. Commit only task files after checks.

