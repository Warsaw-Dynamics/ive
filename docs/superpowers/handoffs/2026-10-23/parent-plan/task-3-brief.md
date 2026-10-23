### Task 3: PlantUML native backend, broad corpus, strict include boundary

**Files:** Create `diagram-ports/engine/src/plantuml.rs`, `diagram-ports/engine/tests/plantuml.rs`, `diagram-ports/fixtures/plantuml/`; modify engine dispatch, manifests, notices, viewer example selection and README.

**Interfaces:** Consumes shared `render` API and uses `Language::PlantUml`. Preserve caller API and viewer isolation. Input includes/resources can only come from explicit caller-owned data; initially reject includes if dependency cannot enforce an in-memory resolver.

- [ ] Audit `puml` provenance and runtime API. Disable default CLI/network/include features. If the library reads files or lacks safe resolver control, adapt a minimal local permissive fork or implement safe native layers from the authorized MIT source; do not enable unsafe behavior to gain a family count.
- [ ] Write failing tests for sequence, class, component and activity examples, Unicode/escaping, malformed wrappers and local/URL/preprocessor include attacks. Test output dimension/resource limits and invalid source diagnostics.
- [ ] Implement native rendering and preprocessing restrictions with descriptive errors, preserve source provenance and notices. Expose backend coverage accurately.
- [ ] Add and execute one meaningful fixture per candidate-supported PlantUML family and adversarial fixtures; document unsupported or semantically wrong families. Make viewer select and render both languages.
- [ ] Run tests/clippy, verify dependency closure/licensing, render samples in viewer and inspect capture. Commit task files after checks.

