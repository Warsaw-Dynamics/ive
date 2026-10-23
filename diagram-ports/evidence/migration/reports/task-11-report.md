# Task 11 — tested PlantUML-specific YAML grammar checkpoint

- Worktree `/mnt/data/Projects/ive/.worktree/diagram-yaml-parallel`; branch `diagram-parser-yaml`.
- Base `53dbad8aa05250b288db13e45402f6b17fb658e6` (tested Task10); tested head `349ccb5`.
- Tracked product tree clean; untracked `.cargo/config.toml` only contains the identical audited Merman cache-path overrides and is intentionally uncommitted.
- Resumed workflow: port → focused tests → fix. Broad review and combined central gates remain deferred by user instruction.

## Implemented scope and interface

- Added `parser/yaml.rs`, `grammar/yaml.lalrpop`, mechanical generated output and exact inventory/source-trace entries.
- `parse(BodySource, &mut Budget) -> Result<data::Value, RenderError>` runs before the remaining legacy layout/error bridge.
- Generated productions own nonempty block maps/lists, entry punctuation, empty/nested values, inline and bare-dash maps, literal chunks, flow fields/separators/brackets and EOF.
- Removed old recursive `Parser::{block,map,list,entry,flow}`, complete-entry `YamlLine` adapter and production Line bridge.
- Existing ordered `data::Value`, duplicate first-position replacement, all-string scalar semantics, datascene owning rows/IDs/geometry remain unchanged.
- `[]` keeps its mandatory empty-string element. Plain bracket-looking scalars enter flow parsing only under the original starts-and-ends-with-brackets boundary rule.
- Tabs expand to four spaces with exact source-byte breakpoints; raw literal blocks keep the original trimming, blank-row and nonstructured stopping quirk.
- Dash virtual indentation preserves inline-map continuation at physical-indent+2; bare-dash bodies admit maps and reject list-of-lists.
- Complete capability preflight precedes Value construction; tags/anchors/aliases/merge keys/folded blocks/nested flow and inline list-item literals fail explicitly.
- Immutable definitions/raw literal chunks/comments never enter another command/constant scan. No complete values/maps/lists are hidden inside lexical tokens.

## Observed RED/GREEN and focused checks

All Cargo commands used outer `/tmp/ive-diagram-build-locked.sh`; engine target `/mnt/data/Projects/ive/target`, locked/offline except fmt's supported CLI.

1. RED: `cargo test --locked --offline -p diagram-engine --lib should_generate_ -- --test-threads=1` →6 passed/2 FAILED at explicit `generated YAML structure unavailable` facade stub.
   Positive failures covered ordered maps/lists/duplicate replacement and literal/escaped flow structure; unsupported negatives were controls.
2. PRIVATE generator `/mnt/data/Projects/ive/target/diagram-generator-yaml`, cwd `/tmp`, absolute manifest `/mnt/data/Projects/ive/.worktree/diagram-yaml-parallel/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml`:
   `cargo run --locked --offline --manifest-path ... -- generate` → generated12 parsers; no grammar conflicts.
3. First structural GREEN:3 new facade tests passed, then225 engine units +141 focused public tests passed with old structural recursion removed.
4. Added focused direct-generated malformed/crossed/dangling streams, pre-model capability, exact source/depth64/65, before-token indent rejection, callback cancellation, tab/origin and lexical retained-allocation controls.
   `cargo test --locked --offline -p diagram-engine --lib yaml -- --test-threads=1` →22 passed before the final retained-buffer control.
5. Final command: `cargo test --locked --offline -p diagram-engine --lib --test plantuml_tree_data --test plantuml_inventory --test plantuml_constants --test plantuml_control --test plantuml_styles -- --test-threads=1`.
   `/tmp/ive-task11-focused-tests.log`:234 unit + constants44/control25/inventory27/styles26/tree_data19 =375 passed,0 failed.
6. Existing direct-flow scalar depth/item ordering, mandatory empty element, escaped-quote semantics and4*n+32/64 linear work assertions retained and passed; public long-flow20*n+1024 work assertions unchanged.
7. `cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings` →exit0 after final code edits (`/tmp/ive-task11-final-clippy.log`); audited vendor deprecation warnings only.
8. `cargo fmt --all --check` and private generator `CARGO_NET_OFFLINE=true cargo fmt --manifest-path ... --all --check` →exit0.
9. Private `cargo run --locked --offline --manifest-path ... -- check` → verified12 reproducible generated parsers, no tracked writes.
10. `node tools/audit-plantuml.mjs` → verified220 exact MIT source/ZIP hashes,43 immutable fixture hashes,43 native modules; parser inventories match.
11. `git diff --check` →exit0 before commit. No worker baseline/native/viewer/full16 gates or broad review.

## Actual generated admission and proof

- Grammar SHA256 `79c5ea1406e2a3095557050c81a38fbc26485e429df63f94344d799f04c7f7d7`; generated SHA256 `3b6c19c762dfd2969315108b9627699374884d702795c987e5083b666c237d94`.
- Actual-table proof `/tmp/ive-task11-stack-proof.py`, SHA256 `80f06683ee874bc9a78d490abd2fb19aabbc67d140b844a2b3286122e99daa9e`.
- Saved witness `/tmp/ive-task11-stack-proof.json`, SHA256 `3378fdb8c94a5b04454985a12a23b14d8d0f7e6ea546a40341888715c398d2e4`.
- Complete shift/pop/goto closure: Yaml48states/76edges/13terminals (11 rounds); Flow8states/7edges/13terminals (5 rounds).
- All-lookahead reduction-top graphs acyclic: max5 Yaml/2 Flow reductions. Independent exhaustive depth0..4 witnesses agree.
- Weighted viable-prefix longest paths at depths0/1/2/8/64/65/128: Yaml2/9/14/44/324/329/644, Flow2/5/5/5/5/5/5.
- Runtime reserves conservative Yaml5*d+4 states from actual lexical nesting; Flow fixed5. Concrete generated symbol max payload+tag64 bytes; spanned token48 bytes on tested64-bit target.
- Existing Task10 arena admission accounts for concrete stack/symbol growth, diagnostic copies/epsilon growth, expected strings, token capacity and existing adapters before allocation.
- YAML lexical admission additionally counts simultaneous physical-row vector/text/tab-map capacities, token/temp scalar reserve, actual indent capacity and old+new token capacities before buffer growth.
- Normalized rows are dropped before generated Value construction; model vectors grow fallibly through the admitted arena; literal/duplicate work charged before mutation.
- Intentional work accounting changes:6 units/Yaml token and5 EOF reductions;3/2 for Flow, plus concrete stack/diagnostic work. Existing source/model/depth/label limits unchanged.
- Detailed formulas, complete-closure reasoning, source ownership and boundary controls appended to own `provenance/plantuml-parser-admission.md`.

## Remaining helper classification and limitations

- Lexical primitives in `yaml.rs`: `find_unquoted`, `remove_comment`, `unquote`; `FlowField::{push,starts_quote,open_quote,finish}` tracks one scalar's quoting/content/trimming with constant-time quote admission.
- Lexical infrastructure in `parser/yaml.rs`: `Physical::{indent,content,span,fragment_span}`, `normalize`, `preflight`, `reject_capabilities`, `lex`, `Lexer::{push,bound_tokens,copy,indent,row,value}`, `lex_flow`, `begin_flow_scalar`. These emit primitive punctuation/indent/scalar/literal tokens and enforce lexical/resource ownership.
- Semantic-only Builder: scope/item budget admission, moving list/map vectors, duplicate replacement and literal concatenation; no source strings reparsed. Facade/generation error mapping is separate; `test_flow` is cfg(test)-only generated-flow setup.
- Root owns shared-file integration, deferred broad review, complete16 gates, immutable43 baseline SVG parity and actual native ordered/nested YAML capture.
- No known focused failures remain; no claim that central native/baseline/integration gates ran. No dependency/vendor/source-grant bytes, fixture records, public API or layout assertions changed.

## Central integration checkpoint

- Sole integration writer applied tested `349ccb5927706c59b7fd367ca7e2005fa70d237f` onto central `d6c013bc5a01e83707b875275f9bae3d1323c8ee` with `git cherry-pick --no-commit`, then committed verified union `3d3c35a` (`Integrate generated YAML parser with preserved family routes`).
- Five actual content conflicts: `engine/src/plantuml/mod.rs`, `parser/mod.rs`, `generated/manifest.json`, `provenance/plantuml-parser.json`, `provenance/plantuml-parser-admission.md`.
- Dispatch rebuilt from exact central HEAD plus YAML facade/model/scene branch; all existing UML/activity/Chen/JSON/HCL/timing/Gantt/sequence/state/files/board routes preserved. Parser registration adds YAML alongside existing modules. Shared `structured.rs` adds only Yaml/YamlFlow cases and leaves JSON/HCL admission unchanged.
- Parser inventory mechanically unions central records with the three YAML paths; scope preserves every existing family and adds YAML. Admission documentation preserves complete central timing/Gantt and prior proofs plus the exact Task11 appendix. Source-trace merge preserves every source hash/grant/credit and fixture/native-module record.
- Central private generator mechanically regenerated17 parsers. Explicit record comparison preserved all16 previous grammar/generated SHA256 pairs and added only YAML; grammar/output hashes remain `79c5ea1406e2a3095557050c81a38fbc26485e429df63f94344d799f04c7f7d7` / `3b6c19c762dfd2969315108b9627699374884d702795c987e5083b666c237d94`.
- Central generated-manifest SHA256 `8dc612163594253880892ddb8fc65740cb0462b16b0625ce7efccb670c823f9b`.
- Combined command (outer lock, shared target, central diagram-ports cwd): `cargo test --locked --offline -p diagram-engine --lib --test plantuml --test plantuml_tree_data --test plantuml_inventory --test plantuml_constants --test plantuml_control --test plantuml_styles --test plantuml_chen_archimate --test plantuml_timing --test plantuml_gantt -- --test-threads=1`.
- `/tmp/ive-task11-central-focused-tests.log`:280 engine units + PlantUML43/Chen-ArchiMate13/constants44/control25/Gantt17/inventory27/styles26/timing23/tree-data19 =517 passed,0 failed.
- Final strict `cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings` exit0 (`/tmp/ive-task11-central-clippy.log`); workspace `cargo fmt --all --check` exit0.
- Private integration target `/mnt/data/Projects/ive/target/diagram-generator-integration`, `/tmp` cwd, absolute central generator manifest: `cargo run --locked --offline ... -- check` verified17 reproducible parsers read-only; private generator offline-env fmt check exit0.
- `node tools/audit-plantuml.mjs` verified220 exact MIT source/ZIP hashes,43 immutable fixture hashes,43 native modules. Cached diff whitespace check passed; explicit progress.md diff was empty. Central tracked tree clean after product commit.
- No broad review, full workspace/native/baseline/full16 gates were run during this focused integration; root retains those gates. No app instances started. Integration writer stops here before Task15 instructions.
