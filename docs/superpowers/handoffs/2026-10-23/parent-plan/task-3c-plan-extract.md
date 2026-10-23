### Task 5: Mindmap/WBS and bounded JSON/YAML

Files: `tree/…`, `data/…`, fixtures/tests.

- [ ] Tree tests: marker-defined parentage, left/right branches, depth changes, boxless nodes, multiple roots/invalid jump policy against source, WBS directions/aliases and crosslinks if supported. Assert subtree bounds and side of root.
- [ ] Implement distinct mindmap/WBS policies using shared ordered tree storage and bounded iterative or depth-limited layout.
- [ ] Data tests: ordered keys, escaped/Unicode values, nested arrays/objects, null/bool/number/string type labels, empty containers, very deep input rejected before construction, model-item budget, duplicate-key semantics. YAML aliases/tags/merge keys must fail before expansion; test exponential alias source specifically.
- [ ] Implement a bounded ordered JSON deserialization visitor and authorized PlantUML YAML subset; preserve source/line diagnostics and data semantics. Render object compartments and true child connections. Record different native geometry from upstream Smetana.

