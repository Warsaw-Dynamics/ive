# Task 3H — native nwdiag global memberships and sparse buses

Status: implemented and frozen for controller-arranged independent SPEC/QUALITY review. This is a bounded nwdiag checkpoint, not full PlantUML compatibility.

- Released independently reviewed base: `f6fa5d9a11a558212afb6b910e3358e97374ef5d`.
- Frozen head: `85887fa1989aa9f21834b8118cddd5735d1dac0a` — `Add bounded native PlantUML nwdiag memberships and buses`.
- Existing `/mnt/data/Projects/ive/.worktree/diagram-ports`, branch `diagram-ports`; tracked worktree clean after commit. Exactly19 scoped standalone files committed. No root IVE product, dependency/lockfile/vendor, other-family, worker-service, branch/worktree, merge or push changes. No subagents or reviewers dispatched.
- Read controller-owned `/mnt/data/Projects/ive/progress.md` first, complete [brief](task-3h-brief.md), [constraints](native-global-constraints.md), [Task7 extract](task-3e-plan-extract.md), full `/tmp/ive-plantuml-specialized-semantics.md` and exact source ledger, preceding [3G including FIX ROUND1](task-3g-report.md), [3F](task-3f-report.md), [3D2 including fixes](task-3d2-report.md), and approved Salt re-review. TDD and verification skills informed implementation. Only the controller updates canonical progress.

## Identity, grammar and shared contracts

`nwdiag::parse(&[Line], &mut Budget) -> NwDiagram` freezes dedicated vectors of Network, Server, Membership and Group. Distinct immutable `NetworkId`, `ServerId`, `GroupId` index stable first-declaration identities. Network identity/display/address are separate; server global name/display/fill/group assignment are separate; Membership owns `(NetworkId,ServerId,address)`. Parser-only ordered registries resolve global names and unique membership pairs. A shared server never creates another box merely by joining another network. Physical grouping can reorder columns while model identities/order remain intact.

Repeated membership updates a nonempty address, preserving the current address for an empty/omitted update; no duplicate membership or cable is added. Server description/color updates affect the shared identity. Network address/CIDR never becomes a server property. Groups own explicit ServerIds, independently of membership pairs. A group inside a network adds both ownerships; a top-level group references an existing server without creating an unrelated network. One server may belong to only one group; a repeated reference even within that group rejects, matching source. Group name remains metadata; only nonempty supplied description paints header text. Duplicate group names retain distinct declaration IDs, as upstream does; duplicate explicit network names reject instead of silently merging declarations.

Admitted exact lowercase `@startnwdiag`/`@endnwdiag`, without filenames/aliases, encloses one fully consumed balanced canonical `nwdiag { ... }`. Both `nwdiag {` and `nwdiag{` tokenize coherently. Names accept Unicode letters/numbers plus `-._`; group names accept letters/numbers/underscore. Named declarations are required. Complete declaration dispatch distinguishes actual `network NAME {`/`group NAME {` from servers named network/group. Statements terminate with physical newline, semicolon or their owning close. Empty property lists are accepted; nonempty lists must consume every key/value/comma/bracket. Quotes preserve delimiters and literal apostrophes; there is no UML identifier normalization or regex-find skipping. Values with spaces require quotes; unquoted values are single atoms. Physical multiline values/lists and embedded escaped quote characters are not admitted.

Server properties: address/description/color. Network properties: address/description/color and exact `width=full`. Group properties: description/color. Typed color is exactly `#RRGGBB`, genuinely used as the bus/server/group fill; default strokes remain the native shared ink. Full width spans all columns, versus member-lane span by default. Comma-space server descriptions/interface addresses split into lines, following NServer.toTextBlock; bus/group text does not inherit that special transformation. Unknown/duplicate/malformed list properties, shapes, other widths/colors, nested groups/networks, network-in-group, empty groups, standalone/hidden-network constructs, undeclared top-level group servers and server-to-server links fail explicitly. A diagram must contain an explicit network and member server. These are documented narrower policies, not upstream completeness claims.

Existing prefix styles/chrome/literal constants affect genuine measurement and painting. Late preambles delegate to shared `style::is_preamble`, including colon/alignment/case forms, without scanning quoted property values as commands. Nwdiag uses existing comment normalization; inert comments and quoted resource-looking text remain literal, while actual capabilities remain denied. Unterminated nwdiag comments, including after the closing wrapper, reject. Timing keyword-ID/Gantt trailing-comment Minors remain untouched and deferred.

Public render/options/control/measurement/worker APIs are unchanged. Wrapper dispatch/preprocessing add only nwdiag. Internal `Paint::Color(Color)` adds safe typed fills; legacy Paint values serialize identically. The unchanged Context/Label/style/chrome/origin/native cache/terminal SVG contracts remain [3D](task-3d-report.md), [3D2](task-3d2-report.md), [3F](task-3f-report.md) and [3G](task-3g-report.md). No runtime engine filesystem/network/environment/clock/locale/process/JVM/JS/browser/dot capability was added.

## Layout, bounds and direct evidence

`nwscene::layout(&NwDiagram, &mut Context) -> Scene` is original native bus/column geometry. Source-ordered horizontal buses sit above one server row. Each actual membership owns a distinct cable lane, one junction dot on its owning bus and its own interface label. An ordered sparse sweep opens an actual gap in every later bus crossed by a foreign earlier cable; there is no dot/painted bus touching that foreign crossing. Nonadjacent router memberships therefore never imply membership in the middle bus. Routes reach the single measured server box, with a short local horizontal fan-in below the reserved header band.

Network labels/CIDRs reserve the left column and bus heights. Interface labels reserve their server column width and row band; separate cable gutters avoid text. Group members occupy contiguous columns; the first member's content region reserves the measured header width so other cable gutters cannot cross its text. Group enclosures contain member boxes/header and are painted before buses/cables/nodes. Server labels are centered inside genuinely measured rectangles. All scene label bounds are contained. This deliberately differs from upstream's first-network server rows, theme coordinates and route bundling; general original geometry/pixel parity is not claimed.

Sparse per-network membership lists, cable arrays and ordered active crossings replace a networks×servers grid. A120-network/120-server private test retains120 memberships,120 cable entries and120 total bus-member references. Crossing sweeps can still require quadratic work in a dense arrangement; each visit/segment is charged and shared work limits bound that case rather than silently allocating a dense grid.

Shared limits unchanged: source256KiB/SVG16MiB/model10,000/depth64/work5,000,000/cooperative10s; worker hard10s/2GiB Unix; labels16KiB/queries4096/coordinates1,000,000 and existing raster caps. Token/property/model/group ownership/text/metadata, lookup/sort/sweep work and generated geometry are charged before retention. Block depth is checked before retaining the new block. Private tests inspect model state after tiny item/depth failures; cancellation is checked in growing loops and around host measurement. Real inflight callback cancellation stops after exactly one measurement. Final scene checks bound every coordinate and the existing bounded SVG writer/terminal resvg-safe/resource admission remain in force.

Seven private tests assert exact stable IDs/display/address/group facts; unique cable endpoints/bus ownership; actual open foreign-crossing intervals; group/header/interface/node containment at6/14/96px; sparse storage; item/token/text/depth failure before retention. Public tests assert exact SVG owner/target sets and one node per global server, not only valid XML. Actual native font paint separately checks10 ink runs inside server/group/canvas bounds at6/14/96px; all10 requests are Host provenance, zero fallback.

## Provenance and reference fixtures

Personally inspected actual authorized grammar/models under `/mnt/data/Projects/plantuml-mit/src` only. Ten selected sources are recorded with exact whole-file hashes, native destinations, translated behavior/policies and actual copyright/author notices in [manifest](../../../diagram-ports/provenance/plantuml-mit-1.2026.9.json). All ten credit `(C) Copyright 2009-2024, Arnaud Roques` and `Original Author:  Arnaud Roques`; the full MIT grant remains in existing licence/notices, including the generated-edition historical header facts. Controller independently matched the exact ten extracted files, JAR members, hashes and headers; no selection changed afterward. No GPL/rejected candidate/gen/h/Smetana/EPL/ELK/untraced asset was imported. Source artifact SHA256 remains locally observed `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`, not authenticated Maven publication.

| Selected authorized source | Native behavior/destination |
| --- | --- |
| nwdiag/CommandNwDiagInit, CommandNetwork, CommandElement, CommandProperty, CommandGroup, CommandEndSomething | `nwdiag.rs`: canonical block, names, complete membership/property grammar, group/network scopes, strict braces |
| nwdiag/NwDiagram | `nwdiag.rs`/`nwscene.rs`: global registry, membership/group semantics, per-declaration networks, painted properties; sparse geometry is original |
| nwdiag/core/NServer | `nwdiag.rs`/`nwscene.rs`: per-network address map/update policy, shared description/color, comma-space labels, default rectangle; symbols rejected |
| nwdiag/core/Network, NwGroup | `nwdiag.rs`/`nwscene.rs`: distinct bus address/display, full width/color, group ownership/description-only header and enclosure |
| Previously selected core/DiagramType239–241 | `source.rs`: exact nwdiag wrapper, no aliases/filenames; provenance entry extended |

Three exact reference-input copies retain original bytes/artifacts; no new JVM oracle run or modification to old reference directories:

| Fixture | Origin | SHA256 |
| --- | --- | --- |
| nwdiagbase.puml | /tmp/ive-plantuml-native-oracles/nwdiag-network.puml | `7bff741790360b248374d315934527d8169e7181427efd4af341892a810951f3` |
| nwdiagshared.puml | /tmp/ive-plantuml-specialized-probes.Kf1afs/nwdiag-shared-group.puml | `4938d58ff6cb66f0750a492cf46c21c907bcb5d5ac19c8b95c2cf27136749a2e` |
| nwdiagrepeated.puml | sibling nwdiag-repeat-membership.puml | `fa3d9eef9458fda3701b8f6196922baefbb878e42a7ee35643b0b9aac6359bbe` |
| nwdiag.puml | authored native viewer source | `b3245713091fdb2296c7280e0de275694484c65a32e96fb8a3052eb676d8b6cc` |

Final release exports with empty PATH and DISPLAY/WAYLAND_DISPLAY unset all exit0. Base topology1bus/2servers/2memberships, shared2/3/4 plus one group, repeat2/1/2. Shared native viewBox672.32×331 versus original306×296; repeated native224.08×298.2 versus original127×195. These preserve semantics and demonstrate different geometry, not pixels. Export hashes: base `283ff7413fb707fc9132390a92c9d963924bceca6997eeb7d3de0930c58668c4`; shared `839781131fccc24de4d6620ce7d50ed6ec992febda86636a4de4303b788259fe`; repeated `b77fc3ae7cdda02a35f9752214bbcec1d00c1268f92e976911059a22beb06657`.

## RED/GREEN and final verification

Initial public10-test suite: meaningful RED7 failed at missing nwdiag wrapper/constant admission,3 negative controls passed. First implementation had a local type-inference compile error, then9/10 GREEN. The remaining SVG test reached shared model-text exhaustion at4096 before the SVG writer;8192 specifically reaches SVG bytes and made10/10 GREEN. Later exact fixture/cancellation/limits/geometry assertions strengthened coverage; not every passing assertion had an independent RED cycle.

Additional meaningful RED/GREEN: comma-space description/interface lines initially painted unsplit then passed after the source rule; viewer example initially unknown then registered; exact NServer source initially unauthorized then admitted by ten-path allowlist; keyword-shaped network/group servers initially misclassified as blocks then passed with complete dispatch; private tiny-depth test initially retained one network before failure, then passed after pre-retention checking. Final16 public +7 private nwdiag tests pass. Source admission tests continue rejecting unselected NPlayField/GridTextBlockDecorated/CommandLink files. All tests use should_ and given/when/then.

Evidence directory `/tmp/ive-plantuml-3h.SIaRmg/`. Final commands after the last product/provenance edit, with common Cargo target and scoped escalation only for known bwrap RTM_NEWADDR setup failure:

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline --workspace
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all --check
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo build --locked --offline --release
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo deny --config ../deny.toml check licenses sources
node tools/audit-plantuml.mjs
node tools/audit-merman.mjs
node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs
git diff --check
```

All exit0. **413 Rust tests passed, zero failed/ignored**, plus seven separately reported actual worker-fault scenarios (blocked stdin/stdout, stderr flood, oversized frame, early exit, memory, cancellation). Counts: engine-private89; compatibility11; Mermaid families37; Mermaid21; PlantUML33; constants31; control23; Gantt16; nwdiag16; Salt18; styles17; timing21; tree/data18; viewer-private48; export5; native-lifetime3; worker6. Thirteen Node audit tests pass. PlantUML184 selected source/JAR matches/31fixtures/34modules; Merman656/four patches. Deny licenses/sources OK. Logs: `tests-final.log`, `clippy-final.log`, `fmt-final.log`, `release-final.log`, `deny-final.log`, `audit-*-final.log`; actual paint in `native-paint.log`. Only inherited vendor Atomic::fetch_update deprecation remains. Self-reviewed the shared diff and dedicated grammar/layout/bounds; no product changes after final gates.

## Genuine capture and remaining scope

```sh
/mnt/data/Projects/ive/target/release/diagram-viewer --background --language plantuml --example plantuml-nwdiag --capture /tmp/ive-plantuml-3h.SIaRmg/nwdiag.png
```

Exit0, real1360×880 GPUI frame directly inspected. SHA256 `b68c1e821d1e123682e360f32410914ff1307cef6dd35ff8a426e92d09c75e04`. Status bar/stderr (`capture-nwdiag.log`) show **21 actual native measurements/0 deterministic fallbacks**. Three full buses and their CIDRs are visible; one bold Edge router has distinct edge/storage addresses and skips office with an open bus crossing. Web service has its own edge/office interfaces. Storage zone contains Data store/Backup behind their cables; labels, title/footer, fills, nodes and header are contained and distinct. No canvas expansion warning or fixture padding correction. Viewer self-quit; anchored lookup for actual debug/release viewer executables found none. No IVE instance, xdotool, focus activation, mouse movement or user-state/process interference.

Residuals: canonical named grammar, quotes required for spaced property values, exact hex fills, explicit advanced-shape/hidden-network/link rejection, one server row and grouped column reorder, simplified routing/themes, shared baseline/font fallback/diagnostic limitations. Foreign crossings are correct but dense arrangements may exhaust charged work; no complete original routing/layout/pixel parity. No known failing gate or new blocker. Frozen-head independent SPEC/QUALITY review belongs to the controller; no next family started.
