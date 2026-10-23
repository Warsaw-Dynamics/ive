# State concurrency and pseudostates: separately reviewed extension research

Prepared 2026-10-23 for `/mnt/data/Projects/ive/.worktree/diagram-ports`. This is developer-only source/oracle research, **not native coverage or authorization to implement the extension**. The initial state/activity implementer can continue rejecting these features. No product files, git state, old oracle corpus, GPL checkout, rejected candidate, `gen/`, `h/`, Smetana, ELK or assets were read or changed. The preceding state section of `/tmp/ive-plantuml-next-semantics.md` and workspace README were read first.

Only the authorized MIT 1.2026.9 source tree and original MIT binary were consulted for behavior. New original probes and their artifacts are isolated in `/tmp/ive-plantuml-state-extension.KTsBTu/`. Java/dot were used solely to run the original developer oracle; this supplies no runtime translation permission. All Java probe processes exited, and the offline browser was closed in `finally`.

## Findings that affect the extension boundary

1. Concurrent regions are active implementation, not a proposal. A separator opens a generated region **after** the first region; first-region contents remain directly inside the composite. This makes original endpoint restrictions asymmetric.
2. Cross-region transitions are not generally permitted. Later-region endpoints cannot be freely used from outside their direct owning region, and final validation requires matching nearest concurrent ancestors. The first region has no concurrent ancestor, so its child can link to an outer ordinary state. Do not replace this with a claim that every first-region escape is rejected upstream.
3. `[H]` and `[H*]` are different singleton kinds per owning scope/region. `S[H]` targets the singleton inside S, and can create S as a composite. Named `state h <<history>>` is an independent named entity, not the `[H]` singleton.
4. Choice, junction, fork and join are active declaration kinds. Upstream treats fork and join as one leaf type and does not enforce UML incoming/outgoing cardinalities. Self-loops and a join with only an outgoing edge render. They describe graph shapes, not an executable state machine.
5. Original rendering can lose the dashed concurrent presentation when first-region/history boundary edges make a composite non-autonomous. A topology-compatible native implementation must decide explicitly whether to reproduce that geometry quirk or preserve visible regions. It must not infer region membership from painted separators alone.

## Active grammar, identity and ownership

`StateDiagramFactory.java:158–169` registers create/link/reverse/create-composite/close/description/concurrent commands. `CommandConcurrentState.java:135–144` admits a whole normalized line of `--+` or `\|\|+`: two or more identical hyphens, or two or more bars. The first character supplies separator orientation. A lone marker, mixed `-|`, or trailing arbitrary label is outside that command grammar. Exact leading/trailing whitespace policy is supplied by the shared command line machinery and was not independently probed here.

`StateDiagram.java:268–292` sets the current group's separator; when already inside a generated concurrent group, it closes that region; it then creates a new group using unique `CONC` sequence identity and enters it. On the first separator, the existing composite keeps all preceding contents. On later separators the composite's separator is not reassigned: `--` followed by `||` produced S.separator=`-`, CONC1.separator=`|`, CONC2.separator=`|`, and **two horizontal painted separators**. Thus a uniform-orientation policy would be an intentional native subset restriction, not original syntax validation. One `}` after a final region closes the region and its enclosing composite; region delimiters do not require their own brace.

`GroupMakerState.java:184–209` builds the first region from ordinary composite contents, then the generated regions in order, and composes them using the composite's separator. `ConcurrentStates.java:135–173,189–218` places `||` regions left-to-right (sum widths/max height; vertical separators) and `--` regions top-to-bottom (max width/sum heights; horizontal separators). Regions are independently measured subgraphs. They are not state nodes with painted headers. The literal original dashed stroke is visible in source and SVG; translating constants would require recording that source and preserving its notice.

Fresh `horizontal` topology is S owning A plus scoped Initial/Final, S.CONC1 owning B plus its own Initial/Final, S.CONC2 owning C plus its own Initial/Final; six transition edges connect only each local trio. Initial/final identity remains `(current owning scope, role)`, now including generated region scope. `reverse_region_stars` confirmed `A <-- [*]` is Initial→A and `[*] <-- A` is A→Final in each region. The raw topology dump prints stored entity order and link decoration; a reverse-arrow dump's printed `->` is **not** semantic direction when its type is `NONE-…-ARROW`.

The original also accepts empty regions and a separator at root. `empty_regions` creates two generated empty groups and paints two boundaries. `root_separator` creates ROOT.A and ROOT.CONC1.B, but no dashed separator is painted. A future bounded extension can reject root separators explicitly; root rendering here is not evidence of a properly framed concurrent composite.

Generated IDs are globally sequenced, not necessarily local ordinals: `nested_concurrent` produced S.Inner.CONC1 and S.CONC2. They are implementation identity, not an advertised user syntax. A repeated `state A` after a separator does **not** automatically create another independent A: the `duplicate_region_name` probe fails with “The state A has been created in a concurrent state : it cannot be used here.” Native region IDs alone must not accidentally legalize such ambiguous textual reuse.

## Endpoint restrictions and routing

These are observed source rules, not a proposed full UML interpretation:

- `StateDiagram.java:149–164`: when an existing named entity is resolved inside a direct concurrent group, its direct parent must be that current group; an entity whose direct parent is concurrent can only be resolved while that parent is current. `CommandCreateState.java:267–270` and ordinary endpoints in `CommandLinkStateCommon.java:341–355` enforce this check. This can reject an ordinary nested descendant even when both endpoints share the same nearest region: `cross_nested_later` rejects `N.B --> C` from the region body.
- `StateDiagram.java:306–341`: final validation walks each endpoint's ancestors to its nearest `CONCURRENT_STATE`. Two absent ancestors are compatible; one absent and one present are incompatible; two present must be the same entity. This catches special endpoint paths that bypass the ordinary check. It compares nearest concurrent ancestry, not all enclosing state ancestry.
- `[H]`, `[H*]`, their named-scope suffix forms, and `==name==` are decoded **before** ordinary endpoint checking (`CommandLinkStateCommon.java:319–339`). Their early resolution does not imply permission for cross-region links: final validation still runs. `same_region_nested_history` successfully routes N's history→C within the same later region, while ordinary N.B→C fails the direct-parent check above.
- `first_region_to_outer` genuinely renders S.A→Outside; both endpoints have no concurrent ancestor. `later_region_to_outer` fails B→Outside. `cross_region` fails A→B from the second region, and `cross_region_outside` fails even with `S.CONC1.B` after the closing brace. No claim is made that direct references to generated names are supported public grammar.

The `final_history_cross_region` candidate produced an error SVG whose reported assumed family is sequence and whose text points at `state Outside`. It did **not** preserve a parsed StateDiagram for introspection. This is not a clean oracle confirmation of the final-validation diagnostic or its line location; use the inspected final-validation source for that rule. Do not present this candidate as a successful state fixture.

For future native routing, retain explicit semantic source/target node IDs, owning composite and owning region, and classify boundary crossings before placement. A transition to S itself is different from a transition to one of S's region children. `fork_composite_endpoints` succeeds F→S→J→Final while S contains A/B regions: the fork targets the composite boundary, not both region entries. Keep node-border attachment and container-border attachment distinct; never replace this with fork edges to every child. History is a visible circle inside its target composite, not an invisible boundary port.

`Entity.java:765–790` makes a concurrent group autonomous and tests other groups' crossing links. `GroupMakerState` uses separate regional images for autonomous composition; `GraphvizImageBuilder.java:509–511` skips direct printing of concurrent groups. The observed `first_region_to_outer` and `history_scopes` SVGs contain the expected labels/topology but **no dashed region separator**. This appears consistent with different grouping/render paths triggered by boundary edges; the full call chain was not traced. Keep that causal explanation labeled as an inference, not a verified universal rule.

## History and declaration stereotypes

`CommandLinkStateCommon.java:150,319–339` admits ordinary identifiers, `[H]`, `[H*]`, identifier-suffixed `S[H]`/`S[H*]`, and synchronization tokens with at least two equals signs on each side. Both source and target accept history. Uppercase H forms were probed; case quirks were not. A quoted display name is not a transition identifier merely because declarations allow quoted display aliases.

`StateDiagram.java:199–266` creates/reuses shallow `PSEUDO_STATE` and `DEEP_HISTORY` nodes under the current scope. The suffix overload temporarily enters/creates the named group and returns that group's corresponding history singleton. Fresh `history_scopes` proves repeated local `[H]` reuses one S singleton, and Outside→S[H] / S[H*] reuse S's local singletons. S.CONC1 has distinct shallow/deep nodes. `history_implicit_group` makes Missing a composite from `A --> Missing[H]` and reuses that composite for `Missing[H*] --> B`.

`named_history_distinct` contains four nodes inside S: named h/d and the scope-singleton shallow/deep markers. Both shallow nodes paint H and both deep nodes paint H*, so displayed glyphs cannot serve as identity. `history_source_and_reverse` confirms incoming and outgoing history transitions are admitted, including reverse syntax. There is no runtime restore-history algorithm in this diagram model; shallow/deep are different static pseudostate kinds and glyphs.

`Stereogroup.java:188–225` inspects **only the first** trimmed stereotype label, case-folded, for kind selection:

| First stereotype | Active leaf kind | Rendering dispatch |
| --- | --- | --- |
| choice | STATE_CHOICE | diamond/branch |
| junction | JUNCTION | filled circle, distinct from Initial identity |
| fork or join | STATE_FORK_JOIN | synchronization bar |
| start / end | CIRCLE_START / CIRCLE_END | Initial / Final shapes, named identities |
| history / history* | PSEUDO_STATE / DEEP_HISTORY | circle containing H / H* |

`stereotype_order` confirms `<<custom>> <<choice>>` remains an ordinary state, whereas `<<choice>> <<custom>>` is a choice. CommandCreateState applies a kind only when actually creating an entity; existing entities are reused, not unconditionally retyped by every later declaration. Repeated declaration order/retyping was source-inspected but not separately probed.

`GeneralImageBuilder.java:219–231,268–269,289–296` contains actual dispatch for these kinds. Junction uses the start-circle painter while keeping `JUNCTION` semantic type, so it must never be mistaken for the diagram's initial node. `EntityImageSynchroBar.java:141–146` uses a horizontal 80×8 bar for top-to-bottom rank and vertical 8×80 for left-to-right. `EntityImagePseudoState.java:138,180–209` paints a 22×22 circle with centered H; `EntityImageDeepHistory.java:116–128` supplies H*. These dimensions are original constants, not a requirement that native text metrics reproduce every original pixel.

`pseudos` has eight entities and eight transitions: Initial→Split; Split→A/B with distinct left/right labels; A/B→Join; Join→Choose; Choose→Junction labeled yes; Junction→Final. Names Choose/Junction/Split/Join are identities and are not painted inside their shapes. `permissive_pseudos` renders fork self-loop, join→A without an incoming edge, choice self-loop and junction self-loop. Therefore native degree validation would be a declared stricter subset; it is not original parser behavior. Fork/join do not exempt endpoints from concurrency restrictions or create implicit cross-region synchronization.

Legacy `==sync==` and `===sync===` normalize to the same named `SYNCHRO_BAR` entity by stripping all surrounding equals signs. `state_synchronization` confirms three edges and one bar. The otherwise identical unforced `synchronization` text is detected as **legacy activity**, not state; require state-family evidence when choosing oracle cases. The explanatory comment saying `=name=` is less exact than the active endpoint regex's minimum two equals signs.

## Recommended bounded future model — proposal, not implemented or fully verified

Use immutable ordered scopes with `ScopeId`, `RegionId`, `NodeId`, and `SourceSpan`. Every composite has an explicit primary region in the native model, plus ordered later regions. Preserve whether a region was primary so original resolution/escape compatibility can be implemented deliberately. Keep generated display-free region IDs private. Represent Initial/Final and shallow/deep singleton keys as `(scope-or-region, kind)`; named stereotype pseudostates remain ordinary named NodeIds with an independent kind. A `HistoryRef` resolves either current scope or a named composite, never a string node labeled H.

Represent `NodeKind::{State, Initial, Final, ShallowHistory, DeepHistory, Choice, Junction, ForkJoin, Synchronization}` and retain original fork/join stereotype spelling as metadata if useful. Keep static graph semantics: do not infer execution, automatic broadcast to regions, guard evaluation or history storage. Store explicit direction and arrow decoration so reverse syntax cannot reverse pseudostate roles twice.

An initial extension can deliberately admit only separators inside balanced composites, homogeneous separator orientation per composite, scoped initial/final and history, named choice/junction/fork/join, and explicit transitions whose endpoints pass supported ownership rules. Reject any unsupported escape/crossing with a located capability error before geometry. If broad original first-region escapes/history crossings are to be accepted, implement container-aware routing first and document the original presentation difference; accepting a token then omitting boundaries is not adequate native behavior.

Bound region count, composite depth, total scopes/nodes/edges, declaration work, identifier bytes and label bytes before allocation. Reuse of singleton nodes must not hide unbounded input-edge work. Suggested initial limits should follow existing diagram limits rather than inventing a separate unreviewed numeric policy. Layout each region's subgraph independently; compose measured extents along the separator axis; put the composite header/body above that content; keep each region's labels/ports within its own bounds. No edge may cross a separator merely because the outer rectangle has a generic attachment point.

Required future native tests include `should_scope_initial_and_final_to_each_region`, `should_reuse_local_and_named_scope_history`, `should_keep_named_history_distinct_from_scope_history`, `should_stack_horizontal_regions_and_place_vertical_regions_side_by_side`, `should_reject_cross_region_endpoint_reuse`, `should_handle_first_region_escape_by_declared_policy`, `should_distinguish_choice_junction_and_initial_identity`, `should_preserve_fork_join_explicit_edges`, `should_keep_composite_endpoint_distinct_from_region_children`, and `should_fail_region_or_depth_limits_before_layout`. Follow repository given/when/then sections. Add original asymmetry, mixed separator and empty/root cases as intentional-boundary fixtures, not automatic compatibility successes. Verify semantic topology, scoped identity, visible shape kind, finite bounds, containment, label ownership, SVG admission/usvg parsing, determinism and cancellation. This research performed no native tests or running-app validation.

## Oracle evidence and reproducibility

Original binary: `/tmp/ive-diagram-oracles.JwN5H5/plantuml-mit-1.2026.9.jar`, SHA256 `e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1f1a515f7f70dd`. Original error SVG version: PlantUML 1.2026.9 / 18d0aae [2026-10-17 08:34:41 UTC]. Environment: Java 24.0.2+12-54; Graphviz 2.43.0. The public `SourceStringReader` / `StateDiagram` API probe records entities, groups, kinds, ownership, link endpoints/decorations, labels and final error before rendering SVG. No production code is derived from this probe.

Commands: `node /tmp/ive-plantuml-state-extension.KTsBTu/probes.mjs` (19 initial candidates), then the six named supplementary candidates recorded in `manifest-supplemental.json`. Each original Java source-file invocation is bounded to 20 seconds and 2 MiB captured output, with headless AWT and PlantUML SANDBOX profile; all 25 invocations exited 0. Original errors also exit 0 through the API; diagram class and painted error text determine rejection. Eighteen candidates produce StateDiagram, six produce error diagrams, one intentionally demonstrates activity-family detection. `manifest.json` and `manifest-supplemental.json` record exact source/SVG hashes; each candidate retains `.puml`, `.svg`, `.topology.txt` and `.stderr.txt`.

The browser inspection in `geometry-v2.json` found positive finite painted bounds for all 25 SVGs and no non-fragment href references. Error images also have positive bounds; this is not semantic success. Horizontal's separators are at y=257.068 and 473.068; Vertical's separator is x=78, height=186; Mixed's separators are both horizontal at y=192.068 and 343.068. XML validation via an XML parser, font identity, individual text containment and native/usvg acceptance were not checked by this small inspection script. The first `geometry.json` selected only explicit stroke attributes and missed style-encoded separator strokes; retained `geometry-v2.json` corrects that selector without altering SVGs.

Probe source SHA256: `Probe.java` = `241565da83c5f9841670610ed17ee221c21080249a3156e6d4a6f6f9485981c6`; `probes.mjs` = `8aac0f0ba18b8ff45da0dc58cde7a02310948f383e1ac96691ad10660ee265c2`; `inspect.mjs` = `fe3d363dfe042de05bc6e43aa4200b712fdfc645b939a1cbd3133403c7b8b8bf`; `provenance.mjs` = `f5ccdd043fad8dccb9567199cea942c82a2b7b7d0c9091d295ba38f0f9e5f31e`.

## Exact source provenance and notices

Source archive `/mnt/data/Projects/plantuml-mit/plantuml-mit-1.2026.9-sources.jar` SHA256 is `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`. This is **local byte identity, not external authentication**. For every file below, the exact archive entry is `net/sourceforge/plantuml/` plus its relative path, and the absolute consulted path is `/mnt/data/Projects/plantuml-mit/src/net/sourceforge/plantuml/` plus that same relative path. `provenance.json` records both exact paths and separately hashed archive-entry bytes; every one equals the working source byte-for-byte. Hashes cover entire files, not excerpts.

All consulted files carry the generated MIT distribution grant at the beginning: `(C) Copyright 2009-2024, Arnaud Roques`, permission grant, requirement to include copyright/permission in copies or substantial portions, and warranty disclaimer. The legacy original GPL boilerplate under the generated MIT distribution header remains part of the exact source; no separate GPL checkout was consulted. Preserve the actual notice if translating logic/constants, and record only actually translated files with native destinations in product provenance. Additional original credits: StateDiagram, CommandLinkStateCommon and EntityImageDeepHistory credit Serge Wenger; GeneralImageBuilder and GraphvizImageBuilder credit Hisashi Miyashita and Serge Wenger. Other inspected files name Arnaud Roques as original author. These are source credits, not a claim that unrelated bundled asset notices apply to this future native work.

| Relative authorized Java path / exact archive-entry suffix | SHA256 (source and entry identical) |
| --- | --- |
| statediagram/StateDiagram.java | 1a9460c41b7564a730f2b45f7a9b8ccbbe265c5b84d83a2f030e73d8974fb505 |
| statediagram/StateDiagramFactory.java | e093dad16866e8ca745374d9f32289f7e0f158f4cb49bfe4525350c35d0e2060 |
| statediagram/command/CommandConcurrentState.java | d4f69081c7c332617d0eccb3fdad8ebbb82b5d70e12ecdd36d4da2c2356d119a |
| statediagram/command/CommandLinkStateCommon.java | 145981d68d8c5407a97cd3c2988f08559b533e5c5c577def628a5fe2157cc7a7 |
| statediagram/command/CommandCreateState.java | 91421507e39550003f8066c311404ff8366950f67600bde35a81bb04bc86891d |
| statediagram/command/CommandCreatePackageState.java | c69fafa7be25599d78ddd91444e5f1a012a13c822bfa033f3dd0ac15c1387b0c |
| statediagram/command/CommandEndState.java | 5b12e265c82dbb4a860fc2ce640a105f16d3b69f78c6d21ddb42361180ec22fd |
| stereo/Stereogroup.java | 6b9adab91e026f0357bb952b4a565cf731014669274d6bd5a5c78012d296a4bb |
| abel/Entity.java | 0580776d6731e88ce39951957db5fa68c05f27487c77fe5777264420f23c6dc0 |
| abel/Link.java | 0d9e97591938eda45948170bc695e4e82f886688310f8fb8b7ad69549a99101b |
| abel/GroupType.java | a9cc57ef5a59d4596debe151340b248e1f165efa967d007d4c0af018d46903a5 |
| svek/GroupMakerState.java | b38a8dc011bc52f85ef6428ac28eb506a3386c40d4072d1c3b4e195c39f546e2 |
| svek/ConcurrentStates.java | da6619b2ca2477a1d5793a9663bb2332ae789dbd9ddaeb32f67e957f7593ed4c |
| svek/GeneralImageBuilder.java | 36c16b9edfbab1be607e8dcc9cca4529821232b726590f9a64837d7d60b5eb26 |
| svek/GraphvizImageBuilder.java | 2b91ed684af66b7f7f099ee47380ee77a474de18ee4c78005e2e110af2a57e40 |
| svek/image/EntityImageSynchroBar.java | 661c581c4867dceb9a550b2ab8e9ebd0e01b36df19a577aae11439f62be84bc6 |
| svek/image/EntityImagePseudoState.java | 83f3f27625402659820c79716d1853030d301e0a598decdc39641a881f5bbfa4 |
| svek/image/EntityImageDeepHistory.java | 965cb7efec977eada2f39a51de8e93ad6b124d6ed3d3b8e78b5237442c789aec |
| SourceStringReader.java | cc27acfa6cce676c1f1f26d766acaf77451e53c4c7eade0757f7b6a0487b9d1d |
| BlockUml.java | f0f6acca3626d329a8a51c6ac42b987e8eb534a41b86c215308f3d24a15f5227 |

## Exact fresh oracle hashes

Every basename below resolves under `/tmp/ive-plantuml-state-extension.KTsBTu/`; use its `.puml` and `.svg` suffixes. Error cases are recorded evidence, not successful diagram fixtures.

| Basename | Original family/status | Source SHA256 | SVG SHA256 |
| --- | --- | --- | --- |
| horizontal | state | 9b98e7fa6a8e5e6fc9c029e685cc6f13ba6766ba034437cdbfa76f8438ecf927 | 651801737a03d3115523fba877363b8cd38e4cdd34c7415792a4b227545b9a34 |
| vertical | state | 0058c1a555f8a5ad9e363c44ba36d37834ad5fb483263c40bbb75189cec09d76 | 5a1dd152e4498b19b592d8fea94f53eb30823a67861b874848a91f78f470a37b |
| mixed | state | 1436684b48cf3459317ed3ad56f63c1a58c4bc62a884ba73b8579d4cb10a031d | d5de89b07bc707418c551c64991ccc8ebe46027b6336fe72632b55fe51455739 |
| root_separator | state | 39d3b16e11480e766747c17ec1571380e0008a5855d27d69ec91963aec7c8576 | 635839b10ba6fb1d5b2475dd80ae68366ec209dd5cdea675e8e5c90ddf335cfa |
| empty_regions | state | bdc7c3efdaedf1709051308418791ac75d0aa58f1e813e61205e44510e71dab1 | 31dbc78b36f0f8fabbd2b05e6b53e08896503e45fcc9aad4a7698e0d1135e140 |
| cross_region | rejected | 50fd5b7200fa5548c946a62a3e40638ad22cf28a46496c982e9b448f53140b1b | fa7d09e919c462bd62d5a7b1105397b750a4a2fb7fdeb48c30d898c25dcefeee |
| cross_region_outside | rejected | 9dd5daf6cbd1f67cfe7e99f4633faba6c39b9131776ea7c88b91ef0b3b76ceae | 0a0f8b3d837d8bbda5edbadf2cefd7b91a0c4012738cc045b5c4976dec7282a0 |
| first_region_to_outer | state | 05f0773e012b3fc081c4455dd40e939fb5f699244cf37115513231943d5d9aa0 | 8d02d610de12e4e1127579ce24c1d603ddfa887547566e6e3bec76aac2f6e882 |
| later_region_to_outer | rejected | 4e07e782fa1405c945588151228fdbfd84f087de7443a6c5ac193c0a55ceb120 | 6724352c888d345cd28d02c218565e99efe6c9322b5f899ed70bf63344312b9a |
| history_scopes | state | e33d9afa38c11180da7f15a0c9eb190294851ef108873c1c09bf896f4cfde045 | c6779fcc9ed0a09e3b140701c72ec7cf28b5c584f2c76f9826b7db596c977e34 |
| history_implicit_group | state | 5bf3e83a49315c5ed6893bc41aa5b046d877dccb5f89466321bc34fb29cce928 | 615caf552773d034aa8bf62db7285f0f0d5d16c6599e2a765c6cff5a60c6c9e5 |
| history_source_and_reverse | state | aa0a5f9904bde432c07da383636cad32d5fe97c377140470d20277d69b93fd5f | 77e6c79ad506d31d04df06a7bd97db09c3125f552e47e89c2d74c6938a19bf69 |
| pseudos | state | 9b68f372b1dbb1f8bc4e9268b0607c046bcdb8fa158b722fb9f7148af8e5c1ff | 7de3ab089dafc491a3b4e3511b7313cd3cdb8da5ddbc2513e572fa243a07d282 |
| permissive_pseudos | state | 08f23edbaaed700650a4c031bacf83e469c86745d4901f5f8be8f3b99758506f | 8c98502b8159f63e7872e369c9ee7732627675b661425a30257897c4a83257dc |
| stereotype_order | state | 3507e00eeb29db63099f2224ba09380c1018bae4147e1908a1e69632cb73534b | 33d9bfe2dea837f88cb3564de34761a518494ef057e47e69afbed76b8f8e8ef0 |
| synchronization | activity, not state | a8d9d11a46caf2306481180f97c10097c6ab4b690841bbef9d70fb3afb3c5184 | 5b8eaee01d81cdbac52fcc65eada2f0459ec5fdab2b7df1399641be312898b88 |
| nested_concurrent | state | 869a7e03d38eec7b6cd2fee2716edadb4d33b65ebab21483462502b4f876539e | 9a545e69af141bd4029615fc7114b0a27eb7d28a2ddc8f17998dafb7b4f7fa4f |
| duplicate_region_name | rejected | 03dfa3dc8fb8fb884332bf3eabc815bfed3f27ebe5c6d2ef8f61a9a99e8d8f06 | 1f32ed9135b1c92763506512246b90c98c110fb4c9dc0e2d13fd1890aa8dc685 |
| cross_nested_later | rejected | 6183ba402f39285b7f43bc702fbe243c10693c4e583377e1bf8e6f9bf238934b | 15922af9fc003f67dc7c1b5078eafe8d0cf7041893bdda8b4fa9eb4eea05f544 |
| state_synchronization | state | 97975945102b34dd18abf0c045bd3d6e4248c0ff2b57af243b627cc3e72c0bc3 | 23ef642fcd72e269575d6ce796ee811c338e6a1edb0e7ba83b3a408470d78b76 |
| final_history_cross_region | rejected | 51f98ac0b055fecf69a05809b91563007b0c23cc5fa1ef4cff3b41dd6d652318 | bf731eb897b4225a2d7364bcb1f4a60d56bdd1949e566356ceec4ff61dd464f8 |
| same_region_nested_history | state | bbe4d1040105a6df876c50162b89259f7cea4982f341fe83b220d830c509e998 | 24602f7d6892088a4447c5fdb99c5996afadcea980dbea2aa99a63ae378b57f2 |
| named_history_distinct | state | 273afa175561eee22ce7ff11434542c89fcde9c50ea7b255faaa4d578288c47c | 3a9e3c34a40e200b01b5dda29beb5e8d9efe3a33098df3a536f5d74b56a4437f |
| fork_composite_endpoints | state | 914832f0bcb2c089b6ca8983de1524b02faea4616faef764cca7c8630936900c | 9dd947d324d7b95ff847239875ee56b4744a19b7c93f213cf94a1bf673960867 |
| reverse_region_stars | state | 2959c43d08cabbb05e97fc392ea044b5059f113618921b678eefcf74737724fc | 28ef44264e7067a670319db997b0436c0b22364f49592b12029130670e325d4a |
