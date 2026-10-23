### Task 9: Mindmap/WBS, files and board outline grammars

**Files:** Create `parser/{tree,files,board}.rs`, `grammar/{tree,files,board}.lalrpop`; modify syntax in `tree.rs`, `files.rs`, `board.rs` and relevant tests; preserve their scene modules.

**Interfaces:** separate generated entries return `tree::Mindmap`, `tree::Wbs`, `files::Files`, `board::Board`. Share lexical marker/indent machinery only, not a universal tree model.

- [ ] Add generated tests pinning independent mindmap sides/parents and WBS aliases/buses; ordered file folder reuse/duplicate files/sibling notes/empty folders/trailing spaces/dotdot; board nearest-lower parents/stage gaps/DFS columns and single root paint. Preserve literal files versus formatted notes/cards.
- [ ] Run real marker/path/note-block structure and malformed opener/indent RED; preserve unknown substantive command rejection and raw Unicode/XML data.
- [ ] Replace marker/path/row/note syntax. Keep semantic depth normalization, trie lookup/anchors and activity ancestry; no filesystem operations, dense grids or ancestor fullpath cloning.
- [ ] Run tree/data/inventory/constants/control/common gates, baseline and native captures for all four outputs, including note clearance and stage geometry.
- [ ] Self-review/commit/report each entrypoint/model explicitly; no family aliasing or skipped file in this batch.

