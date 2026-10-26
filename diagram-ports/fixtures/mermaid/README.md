# Mermaid family fixtures

The `.mmd` files are reduced/adapted examples from Mermaid's MIT documentation at `https://github.com/mermaid-js/mermaid`, revision `6d8921f94f9a12b76586eddcf45a566080d4767e`, under `packages/mermaid/src/docs/syntax/`. Each file normally maps to the same family-named documentation page; exceptions are `class` → `classDiagram.md`, `er` → `entityRelationshipDiagram.md`, `git` → `gitgraph.md`, `journey` → `userJourney.md`, `quadrant` → `quadrantChart.md`, `tree` → `treeView.md`, `xychart` → `xyChart.md`, the four railroad dialects → `railroad.md`, `requirement` → `requirementDiagram.md`, `state` → `stateDiagram.md`, `swimlane` → `swimlanes.md` and `sequence` → `sequenceDiagram.md`.

Info is the minimal `info` command. The flowchart combines basic documented node/decision/subgraph/edge constructs; ER adds documented attribute syntax. Inputs have no external assets. No original snapshots or images are copied. Copyright (c) 2014–2022 Knut Sveidqvist; the complete MIT notice is retained in the workspace notices.

`engine/tests/families.rs` embeds these files and checks expected visible label text, a positive finite viewBox, SVG text elements, valid XML and successful native usvg parsing. Passing one fixture per variant proves admission of that example, not full family or visual parity.
