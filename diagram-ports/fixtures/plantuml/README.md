# Native PlantUML checkpoint fixtures

These five inputs were freshly authored by the independent reference-oracle agent for this task, not copied from upstream examples or a candidate port. They are exact bytes of the corresponding inputs in `/tmp/ive-plantuml-native-oracles/` and were admitted by the original authorized MIT 1.2026.9 renderer. Their original and local hashes, expected painted labels and oracle version/toolchain are recorded in [the provenance manifest](../../provenance/plantuml-mit-1.2026.9.json).

The original result establishes syntax admission, not native pixel or layout parity. The Rust tests assert actual labels, typed topology, shapes, ownership, safety and finite geometry. Native sequence uses its own lanes/rows; native UML graphs use checked Dugong/Dagre, not Graphviz.

The source artifact is verified by local SHA-256 and selected ZIP-entry comparisons, not an externally authenticated Maven checksum. Run `node tools/audit-plantuml.mjs` from the standalone workspace while the authorized source artifact is available.

Timing adds exact independent original-reference inputs `timingbase.puml`, `timingdual.puml` and `timinglane.puml`, with their original temp paths and SHA-256 values in the manifest. The separate `timing.puml` viewer example adds the admitted shared font/chrome to the numeric mixed-track case. Native timing uses a common measured linear axis, robust step waveforms, concise ribbons, an owned dimension and cross-track message; it does not claim decimal, named-time or original pixel parity.
