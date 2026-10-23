Controller context extract from the native plan. Actual implementation release is limited by each task-3i/3j/3k/etc brief; this extract does not release the entire backlog.

### Task 8: Broad backlog and parity iteration

Work through the inventory backlog in the order above, taking each family through parse/model, topology, geometry, SVG and viewer gates. Concurrently close shared style/Creole/TIM/arrow/port/page gaps based on real documents. Full compatibility cannot be declared while known substantive syntax is intentionally rejected. Future multi-page API design should be explicit rather than changing single-SVG output silently.

## Practical release sequence for this session

The first useful session checkpoint is Tasks 1–3: bounded common infrastructure, a nontrivial sequence renderer and one shared UML graph renderer covering class/object plus component/deployment/usecase shapes. This is meaningfully broader than three unrelated toy parsers because it shares typed aliases, relations, nesting, text and safety while preserving distinct semantics. It still must be labelled a subset.

If these gates pass, Tasks 4–5 broaden the backend to state/activity/tree/data. Timing/Gantt/Salt/Nwdiag follow as their own independently reviewable tasks, not as renamed generic scenes. A sustained session may complete several stages; source scale and the oracle results make a fixed promise of all-family completion irresponsible. Report actual admitted constructs and test evidence at each checkpoint. Do not stop at the first slice if more safe authorized work remains and resources allow continued progress.

## Review focus and verification gates

Five especially important cross-feature inputs, assigned to tasks above:

1. Quoted labels containing apostrophes, colons, braces, URLs, XML metacharacters or arrow text: retain text and location; never treat it as syntax or I/O. Tasks 1–3/5.
2. Reused aliases and nested scopes plus parallel/self edges: stable entity identity, correct relation ownership, no dropped links. Tasks 2–4.
3. Unsupported syntax in an otherwise valid diagram (style block, page, macro, sprite, concurrent region): report the unsupported construct rather than partial success. Tasks 1–7.
4. Custom tiny limits, cancellation during parser/layout, long labels and YAML expansion bombs: reject before excessive allocation/expansion and preserve the worker's hard deadline. Tasks 1/3/5/6.
5. Native measurements that decline or return invalid values: honest fallback counts, deterministic fallback, finite non-overlapping layout and no claim of native baseline/bbox parity. Tasks 1–3 and viewer gate.

For each family fixture assert selected family, all expected visible labels, model identities/topology, positive bounded viewBox, shape/decorations, containment and finite geometry. Parse the generated SVG with the existing `usvg 0.46.0` path. Source traces and golden images complement semantic assertions; merely finding `<svg>` is not acceptance evidence.

MIT oracle fixture generation records exact jar/version/hash, fonts, dot version/layout setting, limits and source provenance. Java/dot remain developer-only. Graph comparisons prioritize semantic topology, shape kind, label containment and direction because Dagre/dot geometry is deliberately different. Dedicated sequence/activity/tree/table layouts still need a documented mismatch ledger until upstream geometry comparisons pass.

An independent original-renderer baseline is now available at `/tmp/ive-plantuml-native-oracles-report.md`, with sources and manifest under `/tmp/ive-plantuml-native-oracles/`. Its 30 freshly authored cases have matching source/SVG/authorized grammar hashes; 28 contain meaningful original diagrams and expected painted labels. The two math/LaTeX cases are explicitly degraded raw-expression fallbacks because the MIT jar lacks `org.scilab.forge.jlatexmath.TeXFormula`; do not count them as equation rendering. This is original-side evidence only, not native acceptance. The report includes the exact MIT jar hash `e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1a515f7f70dd` and per-family arguments. JSON/YAML reference runs must omit the original CLI `-SdefaultFontName` injection, which otherwise changes their parsing into an error placeholder. The native API should not emulate that CLI bug. The source stems map directly to Tasks 2–8; future tests should read their actual `.puml` bytes and assert semantics beyond label counts.

Integration gate from `diagram-ports/`: `cargo test --workspace`; `cargo clippy --all-targets -- -D warnings`; `cargo build --release`; `cargo deny --config ../deny.toml check licenses sources`. Reuse the configured parent target directory if the controller requests it; avoid contention with an active Mermaid build. Capture a background pgui viewer frame for representative sequence, nested graph, activity and data cases with `--capture`; ensure every test process exits. This standalone viewer has its own capture path and no IVE state/profile; no need to manipulate the owner's IVE window.

## Design self-review and remaining decisions

All public API claims above were checked against exact cached 0.9.2 sources; product source was read only. The document does not promise that available APIs automatically provide PlantUML semantics. It explicitly accounts for measurement baseline limits, private Merman budget state, parallel edge IDs, wrapper dispatch, YAML pre-expansion bounds, source provenance, external asset denial and unsupported-feature errors.

Before implementation begins, the controller should confirm the Mermaid verification gate and the first staged ownership boundary. There is no new user authorization requirement introduced by this document. The owner already authorized local implementation and coordinated agent execution. No publishing/pushing or modification of the user's running IVE is proposed.
