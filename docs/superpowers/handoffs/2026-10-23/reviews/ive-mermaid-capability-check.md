# Focused Mermaid math capability check

Result: **explicit capability rejection**, not typesetting, diagnostic-only fallback, or silent literal success, for the three tested authored math cases. All three ordinary-text/currency controls succeed and preserve their literal text. No defect established in this checked path; no product change proposed.

Scope: read-only inspection of `/mnt/data/Projects/ive/.worktree/diagram-ports` at `d07df148ac7d82eb2ff33ba5a1ce35fbe9c03d4b`; existing release engine/CLI, default deterministic `RenderOptions`, no suite/build reruns. Probe sources and generated results are solely in the new `/tmp/ive-mermaid-capability.jNPUsr` directory. No checkout/index/branch mutations. No upstream browser, external renderer, PlantUML candidate, GPL source, or ELK investigation/run was used. The concurrent viewer containment work was untouched.

Versions: selected backend is Merman 0.9.2 with the existing four-file compatibility patch ledger, Mermaid 12.1.0 baseline plus documented local 12.2 changes. Authorized Mermaid reference checkout `/mnt/data/Projects/mermaid` is `6d8921f94f9a12b76586eddcf45a566080d4767e`, package version 12.2.0. Its `packages/mermaid/src/docs/config/math.md` specifies `$$...$$` delimiters and documents flowchart/sequence examples. That document does not advertise class math support; the class case here establishes selected Merman admission behavior for an authored closed math expression, not original Mermaid class parity.

## Exact reproductions

`flowchart-math.mmd`:

```text
flowchart LR
    A["$$x^2$$"] -->|"$$\sqrt{x+3}$$"| B("$$\frac{1}{2}$$")
```

`class-math.mmd`:

```text
classDiagram
    class A["$$x^2$$"]
    class B["$$\frac{1}{2}$$"]
    A --> B : $$\sqrt{x+3}$$
```

`sequence-math.mmd`:

```text
sequenceDiagram
    participant A as $$\alpha$$
    participant B as $$\beta$$
    A->>B: Solve: $$\sqrt{2+2}$$
    Note right of B: $$\frac{1}{2}$$
```

These are authored math: matching delimiters around powers, fractions, radicals, or Greek-symbol commands in actual label/message/participant fields. They are not currency examples or incidental source comments.

Controls, respectively:

```text
flowchart LR
    A["Budget $5"] -->|"literal $$"| B["ordinary x^2; price $10"]
```

```text
classDiagram
    class A["Budget $5"]
    class B["literal $$"]
    A --> B : ordinary x^2; price $10
```

```text
sequenceDiagram
    participant A as Budget $5
    participant B as Reader
    A->>B: literal $$
    Note right of B: ordinary x^2, price $10
```

Single-dollar currency, unclosed `$$`, and undelimited `x^2` are ordinary text. They remain ordinary SVG text in all three successful controls. The initial sequence control used a semicolon instead of the displayed comma; that produced `InvalidSource("Diagram parse error (sequence): unexpected statement separator; expected central, signal type")`. This unrelated grammar failure was corrected before the final run and is not math admission evidence.

## Observed API and CLI outcomes

The probe calls `diagram_engine::render(&source, &RenderOptions::default())` against the already-built release rlib. Exact typed errors:

| Case | API result | Existing CLI exit | Output |
| --- | --- | --- | --- |
| flowchart-math | ``Backend("render session lacks capability `math` required by diagram `flowchart-v2`")`` | 1 | No SVG |
| class-math | ``Backend("render session lacks capability `math` required by diagram `classDiagram`")`` | 1 | No SVG |
| sequence-math | ``Backend("render session lacks capability `math` required by diagram `sequence`")`` | 1 | No SVG |
| flowchart-control | `Ok`, family `flowchart-v2` | 0 | 11,005-byte SVG |
| class-control | `Ok`, family `classDiagram` | 0 | 16,627-byte SVG |
| sequence-control | `Ok`, family `sequence` | 0 | 21,039-byte SVG |

The API's displayed error prefix is `native diagram rendering failed: `. The CLI stderr additionally prefixes that with `diagram-viewer: diagram worker: `. Each control's API and CLI SVG hashes match exactly. Successful API metadata includes the existing general diagnostic `ELK and math are unavailable; graph layout defaults to Dagre.` within the full compatibility diagnostic. This general diagnostic is not masking literal success for math: the math cases fail before any successful render result.

Commands to repeat, with explicit absolute paths:

```sh
rustc --edition=2024 /tmp/ive-mermaid-capability.jNPUsr/probe.rs --extern diagram_engine=/mnt/data/Projects/ive/target/release/deps/libdiagram_engine-876d0a0afa49963e.rlib -L dependency=/mnt/data/Projects/ive/target/release/deps -o /tmp/ive-mermaid-capability.jNPUsr/probe
/tmp/ive-mermaid-capability.jNPUsr/probe /tmp/ive-mermaid-capability.jNPUsr/flowchart-math.mmd /tmp/ive-mermaid-capability.jNPUsr/flowchart-control.mmd /tmp/ive-mermaid-capability.jNPUsr/class-math.mmd /tmp/ive-mermaid-capability.jNPUsr/class-control.mmd /tmp/ive-mermaid-capability.jNPUsr/sequence-math.mmd /tmp/ive-mermaid-capability.jNPUsr/sequence-control.mmd
node /tmp/ive-mermaid-capability.jNPUsr/check-cli.mjs
```

The checked runner executes only `/mnt/data/Projects/ive/target/release/diagram-viewer --render INPUT --output OUTPUT` for those six inputs, with a 15-second external per-command bound. Raw results are `api-results.txt` and `cli-results.jsonl` in the same temporary directory.

## Responsible current API/config/source path

All following source paths are relative to the checked worktree:

1. `diagram-ports/Cargo.toml:16`: Merman `=0.9.2`, defaults disabled, only `all-diagrams`, `svg`, `layout-cytoscape` enabled; render defaults disabled too. No math feature selected.
2. `diagram-ports/engine/src/mermaid.rs:35`: render environment explicitly invokes `SvgEnvironment::deterministic().without_math_renderer()`. `create_renderer()` config disables HTML labels; tested closed math still requests math capability despite that setting.
3. `diagram-ports/vendor/merman-render-0.9.2/src/environment.rs:1624`: `without_math_renderer` clears the renderer. `RenderSession::supports_capability`, line 1775, reports `Math` available only when the renderer exists and capability policy allows it.
4. `diagram-ports/vendor/merman-render-0.9.2/src/math.rs:97` and `:124`: `parse_delimited_math_line` recognizes a matching pair of `$$` delimiters; `contains_delimited_math` checks individual HTML-break-separated lines. An unmatched delimiter does not request math.
5. `diagram-ports/vendor/merman-render-0.9.2/src/family.rs:1252`, `:1268`, `:1287`: semantic flowchart/sequence field scans and the class dispatch recognize these labels. Class field coverage is implemented in `diagram-ports/vendor/merman-render-0.9.2/src/class.rs:39` and `:56`.
6. `family.rs:1338`/`:1406`: `required_capabilities` adds `RenderCapability::Math`; `plan_render_with_policy` at `:1501` computes missing capabilities. `prepare_with_render_policy_impl` at `:1627` calls `capability_plan.ensure_available()` at `:1636`, before family layout or SVG emission. `ensure_available` at `:178` produces `Error::MissingCapability`.
7. `diagram-ports/engine/src/mermaid.rs:110`: `map_error` translates this remaining backend error into the public `RenderError::Backend`, retaining the explicit missing-math wording.

The downstream `Option`-returning label helpers can decline math, but that does not imply literal fallback through this selected API: the central capability gate rejects these inputs first. Those gate/math/environment/class files are original imported Merman sources, not among the four local patched files.

## Identities (SHA-256)

```text
release diagram-viewer 6887f8a59f8b06f47243e32a6f72accd5951a0a687a897b8dd89298871c7c624
release libdiagram_engine-876d0a0afa49963e.rlib 79dcd663ad1b31cc6335a95f8291dff5ee76fcbb102f1c82d9a5b6e318f00958
probe.rs e95f4147d94fbe0ba39164c2151ef6981f8e6d3c8a9f496d7b768145c90530c0
probe executable 92478a5dac4416ccf37deea68b5bd9b7d5267b32d6c88054b924843b41d0b67f
check-cli.mjs 11635c9ff4dc7aa256e47756d94304591fd64db6dcaa44d020b54086a4400671
flowchart-math.mmd 2a2a94d9f9365683110c4b7dd21401efafe5d7ba29857491ac4dc987bec81c0d
flowchart-control.mmd e4e35038dbf5349e348cd5c5782336c970368711c99add31f0574dc8445ed102
class-math.mmd 23fbbead79bf9fc508f21caf4a776cc578efa21b27a7ba1217c3c75682180323
class-control.mmd 8a2462c6bf23adff97c9975935594b246d42971706888622a28ccd0f04b5bf83
sequence-math.mmd 12d85fa4b81fd7d479ab83ac5bf99eac038ebcbd901ac0ce2bc108ed05e9f4bb
sequence-control.mmd a4feb0b73170ab0158c54dfa642753aa2456b91aa0969ffc4c16ad6f2ee2dd62
api-results.txt 737562934159e822f90c449e9e7c9f71ca237d594d4e7d1ced54d7b25a679a40
cli-results.jsonl cd27582ab2c541c198874b2185166145c904032eadd073a210124a3560f3e749
flowchart control SVG (API and CLI) 6783a264df89a58add6c74798c53314e661584bf439fc7a76023803011556095
class control SVG (API and CLI) 7e6983d93f51c95b066020028d27c19e9b4bcd8a8d73b1e1f7e1dcb638e9a8b7
sequence control SVG (API and CLI) 0dd5e7372687b5a5b035a901b3e48560971e0d36ac1d90259cfc8f5b2d8c2c78
diagram-ports/Cargo.toml 5ea7f67dddb8f5cf3ea02accc7643854e121119a0eee0f79abccbfa56e15ade5
engine/src/mermaid.rs be6740ad49a652775bff90654daa88dccaff956fbc52a8bd6d3f53233cb1f75f
vendor/merman-render-0.9.2/src/family.rs 8ccf9239212815744ed9e239c6d5781189583f06bae17a84c9fbe3c774b23810
vendor/merman-render-0.9.2/src/math.rs 250704f90bb22729c123039eedbdea485bbf33da4acc7a235e9488e77e8fde5c
vendor/merman-render-0.9.2/src/class.rs 1d860a480c256633cc09b1e73614b7071950fd4f707afb4294f0f56dfe4237df
vendor/merman-render-0.9.2/src/environment.rs 4f56f1a5eefe1e07ad45d4f5f5b1fa9f4fa38f41becf66f673f129d1a8387e77
vendor/IMPORTS.json 1ae0c769c596f22ec729b79d382969b5e32891b5d5e4e36eeabb87b70e866a98
vendor/PATCHES.json 946ff1849d290f045af691c50c8e26efac5b581e6efd8419bf09cc2ac56ce828
```

Severity/limit: no new issue found in the narrow tested admission contract. Math remains an intentional unavailable compatibility capability; the public typed error is generic `Backend` rather than `UnsupportedFeature`, with an explicit capability message. This focused check is not an exhaustive field/escape/config/family matrix or proof of original Mermaid parity. It does not claim any mathematical typesetting, nor that direct lower-level helper calls outside the selected render API have identical admission.
