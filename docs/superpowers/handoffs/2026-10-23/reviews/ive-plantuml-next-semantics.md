# Next native PlantUML semantics: Tasks 4–5

Prepared 2026-10-23, read-only, for /mnt/data/Projects/ive/.worktree/diagram-ports. README.md, AGENTS.md and the full accepted 2026-10-23 plan were read. No product files, existing oracle artifacts, GPL checkout, rejected candidate, gen/, h/, smetana/ or ELK implementation were used or changed. This is implementation preparation, not a product review or a compatibility claim.

The only translation source consulted is /mnt/data/Projects/plantuml-mit/src, MIT edition 1.2026.9. All shortened Java paths below resolve under /mnt/data/Projects/plantuml-mit/src/net/sourceforge/plantuml/. Exact file hashes and line anchors appear in the appendix. Preserve actual notices: the json/ minimal-json files additionally contain the MIT grant and Copyright (c) 2016 EclipseSource (lines 75–100), not only Arnaud Roques's distribution notice. Existing older comment headers underneath the MIT grant are not a license relaxation.

## Implementation decisions that matter most

- State pseudostate identity is (owning composite/root, Initial or Final). Repeated [*] references in the same scope and role reuse one node. Initial and final never collapse together.
- Activity requires structured tiles with explicit live/dead output state. A stopped branch contributes no merge connector; an if with no else still has an empty false path.
- Mindmap and WBS share ordered storage but differ in depth normalization, side parentage, root count and layout. WBS is a vertical root fork with side-facing descendant trunks.
- JSON preserves duplicate members in input order. YAML replaces duplicate values in the original key position. YAML's scalar values are strings, including number/bool/null-looking text.
- YAML is PlantUML's narrow line grammar. Recognized syntax is not necessarily implemented upstream: folded > blocks and list-item | blocks fail. Reject unimplemented syntax explicitly.

## State: grammar, identity and geometry

Sources: statediagram/StateDiagram.java:166–197; command/CommandLinkStateCommon.java:305–355; CommandCreateState.java:158–194,252–310; CommandCreatePackageState.java:157–186; CommandAddField.java:149–188.

Admit named state, quoted display with alias in either order (`state "Ready now" as R`, `state R as "Ready now"`), implicit transition endpoints, labels after the transition colon, nested `state S { … }`, and attached descriptions. A quoted state without alias is also an upstream declaration. Composite opener accepts `state S begin`; closer accepts `}`, `end state`, or `endstate` (CommandEndState.java:140–163). A declaration uses letter/number/underscore/dot identifiers; transition identifiers additionally admit colon. Do not silently admit arbitrary punctuation by applying a universal string parser.

`[*] --> A` creates/reuses the current scope's Initial; `A --> [*]` creates/reuses its Final. Determine role after arrow direction: `A <-- [*]` is Initial → A, while `[*] <-- A` is A → Final. Reverse command binds ENT1 to the textual right side (CommandLinkStateReverse.java:134–148). Source scope is where the transition appears; mentioning a nested state does not transfer ownership of [*] to that endpoint. Use internal scoped IDs, never the visible string [*] as a map key.

`S : entry / open`, `S : exit / close`, `S : do / work`, and `state S : first line` are description lines attached to S, in source order, not graph edges or independently executed actions. Description-only syntax creates the state if absent. Inside a composite, using that composite's own name attaches to the composite rather than creating a child with the same name. A colon inside the description remains text; `\n` in supported Display text becomes an explicit line break.

Store display/alias separately; declaration and transition resolution are distinct phases upstream. Avoid order-dependent failure when a transition references a later composite declaration. Keep state-body height in node/group measurement. Nested header/body/child bounds must all fit inside the composite; pseudostates need genuine initial filled circle versus terminal bullseye shapes.

Negative boundary for this slice: concurrent separators/regions, [H]/[H*] (including State[H]), synchronization bars `==name==`, stereotypes that change state kind (choice/fork/join), ports and transition-node styles are named unsupported features until their own semantics exist. Forward state transitions use hyphen arrow bodies; don't accept sequence dotted arrows accidentally. Local arrow directions/styles/decorations must affect output if accepted. An empty label after `:` is not the nonempty transition-label grammar. Unbalanced groups and aliases that cannot resolve must give located errors.

## Structured activity: topology first

Active commands are registered in activitydiagram3/ActivityDiagramFactory3.java:181–238. Use dedicated typed constructs, with a bounded stack checked against the kind of the current construct.

Actions: `:text;` consumes an action whose delimiter is its ending semicolon; interior semicolons remain text when the rest of the line must match the command end. `:first; second;` is one action with text `first; second`, not two actions. The multiline variant starts at `:` and ends on a line satisfying the final semicolon plus optional admitted suffix grammar; preserve physical newlines and other punctuation (CommandActivity3.java:146–156, CommandActivityLong3.java:140–162,199–220). Reject unterminated actions, unsupported stereotypes/URL suffixes and unsupported alternate action styles rather than dropping their suffix. Do not split all source on semicolons.

Terminal distinctions: start is a filled start circle; stop maps to FtileCircleStop (bullseye), end maps to FtileCircleEndCross (crossed circle), both terminals have no output port (VCompactFactory.java:186–208 and terminal tile anchors). They are separate AST variants.

If grammar includes `if (test) then (guard)`; the `then` clause and its guard are optional in CommandIf2. Another form is `if (test) is (guard) then` (also equal/equals). `elseif` and `else if` are both admitted by zero-space grammar; each adds another tested branch. `else (guard)` is optional; `endif`/`end if` closes. Optional terminal semicolons are accepted by these commands. Incoming/outgoing label/style variants are additional capability, not harmless tokens.

An if with no else has a real empty false branch (InstructionIf.java:210–221,264–271). At convergence, join only live branch exits; if all branches stop/end, the if tile has no output (FtileIfLongHorizontal.java:628–645,778–787). A dead branch must not point to the following action. Retain each test's positive guard and the final else guard on the appropriate edges. Test elseif as a chain of conditions, not unrelated parallel conditions. Duplicate else or elseif after else is invalid.

While: `while (test) is (yes)` / equal(s) optional guard, then body, `endwhile (no)` or `end while (no)` / `while end (no)`. Entry reaches the test first; positive edge enters the body, live body exits return to the test, false guard belongs to the test's exit. If body is dead it has no loop-back edge, but the zero-iteration false path remains. Source: CommandWhile3.java:144–159, CommandWhileEnd3.java:139–155, FtileWhile.java:192–242,301–345. Upstream may attach a stop/end immediately after endwhile as a special loop false terminal; native topology must still be test-false → terminal (ActivityDiagram3.java:251–265).

Repeat: `repeat` or `repeat :entry action;` creates entry before body; then `repeat while (test) is (again) not (done)`. The `is` clause, the `not` clause, both, or even the test are optional according to the command; equal/equals variants also exist. Test occurs after the body; positive guard labels the loop-back route, negative/out guard labels the onward exit. Do not invert these. A `repeat while … ->label` suffix labels the loop-back connection, distinct from the positive guard. Exact original command capture is greedy: adding a final semicolon in `… ->label;` retains it as part of LABEL (`label;`), confirmed by the fresh capture probe. Preserve this or record an intentional cleanup. Source: CommandRepeat3.java:144–155,196–208; CommandRepeatWhile3.java:142–183,228–243; InstructionRepeat.java:267–278; FtileRepeat.java:201–237,248–277. Keep backward-action grammar unsupported until its action/route actually exists.

Fork: `fork`, one lane; `fork again`, another lane; `end fork`/`fork end` joins with synchronization bar. `end merge` is a distinct merge diamond without synchronization. Optional `{join specification}` belongs to the join, not to one lane. Fork/fork again do not have a parenthesized guard in their core grammar. Lane-entry and lane-exit labels instead come from the next-arrow command, e.g. `-> lane guard;` before its first action and before `fork again`/the closer (CommandArrow3.java:139–148,175–188; ActivityDiagram3.java:299–315; ParallelBuilderFork.java:218–236,268–285). If this arrow command is outside the admitted subset, reject the whole source that requests it.

Each fork lane owns an ordered instruction list. Draw a split bar, separate measured lane extents, and the selected join kind. Only live lanes connect to the join (ParallelBuilderFork.java:195–200). Preserve the join specification label. Empty lanes and a single-lane fork are permitted by the original list-building semantics; requiring two nonempty lanes would be a documented subset restriction. Do not reinterpret `end merge` as a fork join.

Groups/partitions: `partition "Name" {` and `group Name {` wrap subsequent instructions, with `}` closing only the current group. The same command also accepts package/rectangle/card, each mapped to a distinct container symbol; admit only symbols actually drawn. Colors may appear before or after the name. Missing `{` is an accepted deprecated upstream form with a warning, not a reason to ignore the group (CommandPartition3.java:151–185,220–246; CommandCloseGroup3.java:134–151; ActivityDiagram3.java:495–508). Test nested group containment, header measurement, and closing a group while a nested if is still open. Optional colors/stereotypes need either real behavior or a located unsupported response.

Explicit enum examples: `Exit::Live(Port)/Dead`, `Terminal::Start/Stop/End`, `Join::Synchronize/Merge`. Connect sequential tiles only when the predecessor has a live exit (FtileFactoryDelegatorAssembly.java:140–153). Upstream still draws later disconnected actions after a stop, and may give those later actions local exits; do not fabricate stop → next connectivity. A fully semantic reachability pass may annotate disconnected components separately.

Negative boundary: mismatched close, else without if, elseif after else, fork again under an unclosed nested if, repeat-while outside repeat, unclosed loops/forks, and depth overflow fail. Swimlanes/break/kill/detach/switch/backward/goto/legacy activity remain named unsupported until implemented; no generic action fallback.

## Mindmap versus WBS

Mindmap plus/minus (CommandMindMapPlus.java:137–182): `[+-]+`, optional `[#color]`, optional immediate `_`, optional whitespace, then label. Raw level = marker count − 1; any minus anywhere chooses the reverse/left side. Mixed plus/minus markers are accepted. There is no indentation prefix in this grammar.

Mindmap org mode (CommandMindMapOrgmode.java:137–189) accepts leading spaces/tabs followed by one or more star/hash characters, then optional color and underscore. Smart level uses the first captured marker prefix; tabs become ONE space, not four; marker-only input is count − 1; space-bearing variants compare with the first prefix. A type ending in ** has its leading whitespace trimmed before the other rules (MindMapDiagram.java:210–230). These rules are not visual indentation guesses.

The first mindmap item is forced to level zero even if its raw level is larger. The first nonroot raw depth establishes an indentation multiplier; all later nonroot levels must divide by it evenly (MindMap.java:191–215). E.g. `* Root / *** Child / ***** Grandchild` normalizes to depths 0/1/2. A later raw depth 3 is then Bad indentation. A jump of more than one normalized level is invalid. Side branches maintain INDEPENDENT last-parent stacks (Branch.java:127–161): switching sides cannot attach to the most recent item on the other side. A second level-zero item creates a new mindmap, stacked below the prior one (MindMapDiagram.java:188–205,158–174), unlike WBS.

Mindmap `left side`/`right side` changes default side for org-mode nodes; plus/minus overrides it. The broad original direction-command regex also admits top/bottom and surrounding words; its execution only sets the side boolean: RIGHT/DOWN true, LEFT/UP false (CommandMindMapDirection.java:137–161, MindMapDiagram.java:138–142). Do not infer a rank-direction rotation from that command alone. The alternate `0 root label` form explicitly adds a root. Multiline org-mode `**:first\nlast;` uses marker count − 1 directly, not smart indentation, then the same model normalization (CommandMindMapOrgmodeMultiline.java:139–153,185–207). Empty labels are admitted upstream; boxless `_` removes outline, not the node/text/parent edge.

WBS markers (CommandWBSItemNew.java:140–206): leading spaces/tabs + one or more of * + −; optional shape/direction (`_` and `<`/`>` in either order), optional color/id in either order; id is `(alias)`; label requires separating whitespace. Alternate label form is `** "Display" as Alias`. Legacy order places color/id before shape/direction and is still accepted, with warning (CommandWBSItemOld.java:144–227). Any minus defaults left, but explicit < or > overrides it (utils/Direction.java:149–159).

WBS multiline form uses the same marker/modifier/alias prefix, then immediate `:`, text continued through a terminating `;` (optional ending stereotype). Unlike mindmap's multiline variant, it calls WBS getSmartLevel, so indentation rules still apply (CommandWBSItemMultilineNew.java:143–199; legacy order in CommandWBSItemMultilineOld.java:143–200). WBS factory tries quoted alias grammar before plain label grammar and retains both modifier orders; mindmap factory tries multiline before single-line org mode (respective factory anchors).

WBS smart level forces FIRST item to root; thereafter marker-only = marker count − 1, tab→ONE space, prefix comparison for indentation. There is NO mindmap multiplier and NO special trim for **. It has one global last-parent chain irrespective of side and rejects a second level-zero root and depth jumps (WBSDiagram.java:230–303). Same-depth items are siblings of last's parent.

WBS geometry is materially distinct: root at top, first-level children ordered horizontally beneath a bus (Fork.java:132–139,173–232); descendants have left/right lists attached to vertical trunks beneath their parent (ITFComposed.java:186–243,266–293). A first-level left node is prepended to the root's right collection so the root fork includes it (WElement.java:215–225). Thus a negative root-child marker is not a general guarantee that its center lies left of root center. Test source-defined sibling ordering and descendant side relative to its own parent/trunk.

WBS `_` with a label is boxless; without label it becomes a zero-size PSEUDO junction; without underscore and without label it errors. Keep pseudo topology despite no painted text (WBSTextBlock.java:174–185). Nonroot aliases are stored for extra links; same alias replaces lookup target with latest node, retaining both drawn nodes. Root aliases are not inserted in the upstream codes map. Crosslinks use declared nonroot aliases with [.-]+> and fail if unresolved (WBSDiagram.java:310–328, CommandWBSLink.java:142–184). Upstream captures but ignores crosslink labels. Prefer explicit unsupported until actual native crosslink geometry exists; don't silently accept a labeled crosslink while claiming its label is preserved.

## Ordered JSON and actual YAML subset

JSON entry is JsonDiagramFactory.java:147–167 → Json.parse. Parser's object-value handler APPENDS with object.add (json/Json.java:450–452); JsonObject stores parallel ordered name/value lists (411–421). `{"z":1,"a":2,"z":3}` retains THREE members in that order. get("z") returns 3; set("z",4) changes only the LAST matching member in place (572–587,912–918). TextBlockJson iterates Member entries directly, so both duplicate values appear; do not deduplicate during rendering. Use Vec<(key,value)> or equivalent ordered duplicate-preserving model, not serde_json::Value or a unique-key map. Lookup/set semantics matter only if later pure preprocessing needs them.

JSON scalar kinds remain distinct. Source text can include escaped Unicode, XML metacharacters, apostrophes and colons with no comment splitting. Standard syntax failures include unquoted keys, trailing commas, malformed escapes/numbers and trailing substantive tokens. The inspected original also has a narrow // comment extension only around object-member boundaries (JsonParser.java:302–337), and the factory discards column-zero # lines; reject these explicitly as outside an initial strict JSON subset rather than accidentally changing string data. Never reproduce skipComment's unbounded EOF loop. Bound depth/items/work before descent and allocation, including overwritten YAML values and all JSON duplicate members.

Data layout: each object entry/array element has an ordered row, nested containers connect from THAT row, preserving scalar-row placeholders between children (TextBlockJson.java:185–209,269–287). Array indices belong to semantic row identity. Scalar text policy upstream: strings unquoted, number spelling via toString, null U+2400, boolean U+2611 true / U+2610 false (249–266). Root scalars are wrapped as one visual row; root empty object/array becomes one blank row (JsonDiagram.java:154–162). Preserve actual typed empty containers in the native model even if their visual presentation is normalized.

YAML ACTIVE path is YamlDiagramFactory.java:148–164 → yaml/parser/YamlParser → MonomorphToJson, not SimpleYamlParser. It is not YAML 1.2.

- Tabs expand to FOUR spaces in YamlLine.build; map/list indentation is bounded parser stack state, not tree marker indentation.
- Colon separator is the first colon outside a single/double-quoted region, with no requirement for a following space. Quoted keys such as "a:b" and values such as "v:c # text" work. x:a:b maps x to a:b.
- YAML comments: column-start # is blank; space+# outside quotes starts a comment. # inside matching quotes survives. Quote scanning is simple and not a general escape-aware YAML scanner.
- A map scalar merely removes matching outer quotes; it DOES NOT decode JSON escapes or doubled single quotes. `a: "x\\ny"` keeps a backslash+n; `b: 'it''s'` keeps doubled apostrophes.
- ALL scalar values become JSON strings, including `2`, `true`, `null`, `~`, `{}`. Empty `key:` with no deeper content is empty string.
- Nested maps, scalar block lists, list-of-map entries, plain '-' map items, root scalar lists and keyed flow lists work. List items count two extra indent columns for "- ". Keyed flow lists split commas outside quotes and handle a backslash by retaining the next character; this differs from plain map scalar handling.
- Keyed `[]` becomes a list containing ONE empty string, because toList always appends a final field. Flow maps are not parsed as maps. Nested flow containers are not recursively parsed; list-of-lists conversion is not implemented by MonomorphToJson.convertToArray.
- Map `key: |` gathers subsequent lines as a literal string, trims EACH collected line, appends LF after each including the last. Its stopping rule permits any NO_KEY_ONLY_TEXT or EMPTY_LINE regardless of indentation, and otherwise stops only at a nondeeper structured line. Test this quirk or restrict it explicitly; don't promise YAML block indentation/chomping semantics.
- A `key:` followed by deeper plain text joins subsequent NO_KEY_ONLY_TEXT lines with spaces (blank lines skipped); this is distinct from | blocks.
- A raw > value is classified as KEY_AND_FOLDED_STYLE but has no parser branch and errors. |-/|+/>-/indent indicators are not block syntax. Block style on a list item errors. Top-level plain text and document markers ---/… are not supported documents.
- YAML maps use LinkedHashMap.put: latest duplicate value replaces prior value without moving its FIRST key position (Monomorph.java:177–184). `z: first / a: middle / z: last` becomes ordered z:last, a:middle. This is deliberately different from JSON.
- The original treats alias/anchor/tag/merge-looking scalar text literally; native must enforce the accepted plan's stronger boundary: quote/context-aware rejection of anchor/alias/tag requests and merge keys BEFORE expansion or model construction. No generic YAML loader. Quoted/block literal text containing these characters stays text. Include an exponential alias source negative test and ensure it fails at its first capability token, not after expansion.

Sources: YamlLine.java:126–169,175–250,262–302; YamlParser.java:118–227; YamlBuilder.java:157–253; Monomorph.java:167–184; MonomorphToJson.java:118–155.

## Recommended semantic and geometry tests

All tests use should_ names and given/when/then sections. Existing matching fixtures are in /tmp/ive-plantuml-native-oracles/; don't rewrite their .puml/.svg/.json/.result.json.

| Family fixture | Source SHA256 | Required assertions beyond SVG success |
| --- | --- | --- |
| nested-state.puml | 081bdcec241cbc1b3cb035cfe0d7e42dcee2efa45813e0de058c0df8533b24bf | Three distinct scoped initial nodes, Fulfilment-scoped final; Packed owns Sealed/Checked; labeled inspect edge has correct endpoints. |
| activity-branch-merge.puml | 5f2ccae11dc644b2227663247a92936b17fe6d0b8f280e4a7d9aa813579d2979 | yes→Persist and no→Reject, both live exits converge before Reply; stop has no fallthrough. |
| mindmap-hierarchy.puml | 079e4ebf4341974d9c1176f5560a17ef677a31362ac23ee2df7aa2b1bac1a79c | Tests and Delivery siblings; Unit/Package correct parents and source order; measured subtrees do not overlap. |
| wbs-hierarchy.puml | 0dc40c5ffe0feeb9ffad194298c9efe06b65558f4f460a58b1a6486fac8504ae | Root above first-level bus; Design/Rollout columns and Schema/Monitor descendant trunks, unlike mindmap layout. |
| json-nested.puml | fefb478ac2559c3d452798502155a3a0ade75f569048d3fa237cfd809703f626 | service then enabled rows; ports indices 0/1 preserve numbers; child connectors originate from service/ports rows. |
| yaml-nested.puml | 5685849a4dbd310cb5bce028c2d8986d2a830927851af3972e97ec7f444566cb | replicas is STRING "2"; zones north/south order and correct service ownership. |

Add before implementation:
1. State: should_reuse_initial_and_final_per_scope; should_bind_reverse_pseudostates_by_semantic_role; should_attach_ordered_state_descriptions; should_resolve_forward_composites; should_reject_history_and_concurrency. Use repeated [*] in two sibling composites and root, and assert shape counts, unique scoped identity and container bounds.
2. Activity: should_converge_only_live_branches; should_keep_implicit_false_path; should_have_no_exit_when_all_branches_stop; should_preserve_elseif_guard_chain; should_retain_interior_semicolons_and_newlines; should_route_while_false_exit_and_live_back_edge; should_route_repeat_true_back_and_false_out; should_label_both_fork_lane_ends; should_distinguish_synchronizing_join_from_merge. Assert explicit port ownership and edge topology, no edge out of stop/end, positive lane separation and guard containment.
3. Trees: should_use_independent_mindmap_side_stacks; should_normalize_mindmap_first_child_indent; should_reject_nonmultiple_or_jumped_depth; should_stack_multiple_mindmaps_but_reject_second_wbs_root; should_keep_wbs_parent_chain_across_sides; should_override_minus_with_wbs_direction; should_preserve_boxless_and_pseudo_parentage. Test "--- grandchild" after right side switch; distinguish mindmap side-specific parentage from WBS global last. Assert side only relative to correct parent/trunk, plus source ordering and subtree extents.
4. Data: should_preserve_json_duplicate_rows; should_replace_yaml_duplicate_in_original_position; should_keep_yaml_scalars_as_strings; should_keep_quoted_colons_and_hashes; should_preserve_actual_literal_block_behavior; should_reject_folded_list_blocks_and_document_markers; should_fail_alias_bomb_before_expansion; should_reject_depth_before_construction. Include nested empty JSON containers, huge exponent/long numeric lexeme, escaped Unicode, unknown styles/directives, resource-looking literal text and low item/work/depth limits.

Every successful case checks family, all expected labels/identities, finite geometry and positive bounded viewBox, label containment, correct shapes, stable deterministic SVG, terminal SVG admission and usvg parsing. Exercise cancellation during parsing/layout and native measurement decline/fallback. Geometry may differ from dot/Smetana; topology and source semantics must not. No SmetanaForJson implementation was read.

## Fresh original semantic probes

New probe source: /tmp/ive-plantuml-next-probes.QwTCEk/Probe.java, SHA256 767d66c10adde81b57a8c314c1d397938dbd3a25f06b2332addb0a54e2cc8161.
Ran `timeout 20s java --class-path /tmp/ive-diagram-oracles.JwN5H5/plantuml-mit-1.2026.9.jar /tmp/ive-plantuml-next-probes.QwTCEk/Probe.java`; exit 0 in 0.32 seconds. This calls the original public Json/YamlParser methods directly and changes no existing oracle files.

Jar SHA256 independently checked: e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1f1a515f7f70dd (same exact jar as existing corpus).
Observed:
```text
json-duplicates={"z":1,"a":2,"z":3}; get(z)=3
json-set-last={"z":1,"a":2,"z":4}
yaml-types={"n":"2","b":"true","nil":"null","empty":"","obj":"{}","seq":[""]}
yaml-duplicates={"z":"last","a":"middle"}
yaml-quoted-colon={"a:b":"v:c # text","x":"a:b"}
yaml-list-map={"people":[{"name":"one","age":"2"},{"name":"two","age":"3"}]}
yaml-root-list=["north","south"]
yaml-flow={"a":["one","two,three","true"]}
yaml-flow-empty={"a":[""]}
yaml-literal-block={"a":"first\nsecond: value\n","b":"next"}
yaml-folded-block=ERROR UnsupportedOperationException: wip4 YamlLine(KEY_AND_FOLDED_STYLE indent=0, key=a, value=>)
yaml-block-chomp=ERROR YamlSyntaxException: YamlLineType.NO_KEY_ONLY_TEXT
yaml-plain-continuation={"a":"first second","b":"next"}
yaml-quoted-escape={"a":"x\\ny","b":"it''s"}
yaml-list-block=ERROR UnsupportedOperationException: wip3 YamlLine(KEY_AND_BLOCK_STYLE indent=4, key=name, value=|)
yaml-anchor-literal={"a":"&x value","b":"*x","<<":"*x","tag":"!!str value"}
yaml-document-marker=ERROR YamlSyntaxException: YamlLineType.NO_KEY_ONLY_TEXT
```

A second fresh probe, /tmp/ive-plantuml-next-probes.QwTCEk/ProbeCaptures.java (SHA256 58636c31173ecaaa46be01679024ddb58a13768ec8b7104ba67868881daf73eb), invoked only the original command regex matchers through reflection; exit 0 in 0.23 seconds. It confirmed `CommandRepeatWhile3: LABEL=label;` and `CommandActivity3: LABEL=first; second`. No source outside the authorized tree was read.

## Exact authorized source anchors and hashes

Each path is relative to /mnt/data/Projects/plantuml-mit/src/net/sourceforge/plantuml/. SHA256 covers the complete source FILE, not just cited lines. These are inspected translation candidates; product provenance must include only files whose logic/constants the implementation actually translates, with destination and preserved notice. This report itself makes no product translation.

| Relative Java path | Lines | SHA256 |
| --- | --- | --- |
| statediagram/StateDiagram.java | 166–197 | 1a9460c41b7564a730f2b45f7a9b8ccbbe265c5b84d83a2f030e73d8974fb505 |
| statediagram/command/CommandLinkStateCommon.java | 148–150,234–289,305–355 | 145981d68d8c5407a97cd3c2988f08559b533e5c5c577def628a5fe2157cc7a7 |
| statediagram/command/CommandLinkState.java | 132–154 | 195f191f2ae3b87a8affe527a1edc359315eab1db3c349aedc8872ca0c87d9b5 |
| statediagram/command/CommandLinkStateReverse.java | 134–161 | 4a0cd26a4fe297b924599bacbef8637eccc0295fa3d07a68431f0c4e818f426f |
| statediagram/command/CommandCreateState.java | 158–194,252–310 | 91421507e39550003f8066c311404ff8366950f67600bde35a81bb04bc86891d |
| statediagram/command/CommandCreatePackageState.java | 157–186,240–254 | c69fafa7be25599d78ddd91444e5f1a012a13c822bfa033f3dd0ac15c1387b0c |
| statediagram/command/CommandAddField.java | 149–157,170–188 | 32222dabeb23f023c4960b9d713abdbea1b16fe8086683b6b322844f761cc2ef |
| statediagram/command/CommandEndState.java | 140–163 | 5b12e265c82dbb4a860fc2ce640a105f16d3b69f78c6d21ddb42361180ec22fd |
| activitydiagram3/ActivityDiagramFactory3.java | 181–238 | be56c0dc71969fd9b34dc042d42684621fb5601d504962b226e44dc7c568867f |
| activitydiagram3/ActivityDiagram3.java | 227–265,290–319,381–485,495–508 | 204fd76ab15942fbd49a106fdd3514f6d129e8abc07cf3e57d8e12b0a5cf390f |
| activitydiagram3/InstructionIf.java | 210–221,240–271 | 163328578e518263992a3a9bf040555f6e3e89aafcc7a53c253408b354c3fb95 |
| activitydiagram3/InstructionRepeat.java | 237–278 | 8b87458208c6815c7a176e3c86f800bf45a662cd255e3c16c2a94fbe857bc817 |
| activitydiagram3/InstructionFork.java | 158–174,195–215,257–270 | 7202ec312daebfc0e2e17b8355658fe1b22077616e50786771b7ff9c6248e586 |
| activitydiagram3/command/CommandActivity3.java | 146–156,221–222 | 291c4718ac1f4d3c27ed5608b45273da843d4fe92f50490cb98a8d490ee9e2db |
| activitydiagram3/command/CommandActivityLong3.java | 140–162,199–220 | fa0ff1dcb54b2a354739ca38c005503fdab76c5211f94ddecec9bb70841b0c8b |
| activitydiagram3/command/CommandIf2.java | 149–168 | 8139589658873c4d29a73dfdd5c5532e8e64aee123e5ac19717cf9bbb99b6fc1 |
| activitydiagram3/command/CommandIf4.java | 143–162 | 646622fecf1ed1281eaba9b062f0f072dac81cf6e4b07471e3db168e48569504 |
| activitydiagram3/command/CommandElseIf2.java | 148–183 | bd37cc74050d4d7dabea2bf90f3f7bb8b8418934087e4385fbb5b4214a9229ea |
| activitydiagram3/command/CommandElseIf3.java | 148–188 | babacbb49c18532b7cdfe22fba68ac801b854beed459f8409547b09e26f7cc9f |
| activitydiagram3/command/CommandElse3.java | 140–152 | d9aeeb018b03494aec5681567908056492a6923fe96b16da9f8394f1f9d76537 |
| activitydiagram3/command/CommandEndif3.java | 139–146 | 600fa7c42ee6a49f02facd559117be5fd7c06f6524a6ce0177958ddcd16c5849 |
| activitydiagram3/command/CommandWhile3.java | 144–159 | 376f54302e4993de0d1c1066b52fc70380dc9452fadddd9cadc10a37b6f5ca12 |
| activitydiagram3/command/CommandWhileEnd3.java | 139–155 | f7cbbaf3788cc2c1dd420db497596d50e9558d5bfc4bf6eb8dde87f6bc385124 |
| activitydiagram3/command/CommandRepeat3.java | 144–155,196–208 | 3b9823f49ea06af651fd1a4dff06f2d8cc9917a6235d56cb3b9c3809f28f8b86 |
| activitydiagram3/command/CommandRepeatWhile3.java | 142–183,228–243 | 78c3c294d6b224b359c3891c427291f55eca14b81b1fedde9f9e62a4757e20c3 |
| activitydiagram3/command/CommandFork3.java | 138–143 | c90d1da7ef7a224e8051310e8c53536c892520949530462aaa940e944711d3f4 |
| activitydiagram3/command/CommandForkAgain3.java | 135–140 | ee369912599c95a85bee467169786fcf50c3acd8abbc03a48d11e9ab9b806912 |
| activitydiagram3/command/CommandForkEnd3.java | 138–160,168–190 | a3f19d9f35ea7e17b55ae9fd3893174fc071883f7db76fa55f5613f3d15b6147 |
| activitydiagram3/command/CommandArrow3.java | 139–148,175–188 | da89bace2b3625d5b5f35d4ac674dad100dd2de2c4b6d6348c0cd678c6a69c4b |
| activitydiagram3/ftile/FtileGeometry.java | 215–229 | 59d8bcc39e734439344eda47e569b3af6d413f677cd317471d0dad2b4c18ab13 |
| activitydiagram3/ftile/vcompact/FtileFactoryDelegatorAssembly.java | 140–153 | ced19b00be7109e69e38383253b5663a6d2be0104c34dcb82ea9ea272d9d5202 |
| activitydiagram3/ftile/vcompact/FtileIfLongHorizontal.java | 279–301,628–645,778–787 | cd265facdc9a94a5fad9635f1a14c336ca14daf7760435b69d4678db1846afa0 |
| activitydiagram3/ftile/vcompact/FtileRepeat.java | 192–278,369–386 | c463fcc6e9e80b643b74c880f661e0af74468db1e3a4bbbdaf903642caf41227 |
| activitydiagram3/ftile/vcompact/FtileWhile.java | 192–242,301–345 | 63031d49b1bb5957c2b05e12d0f5470d0b9dc9eb9be87d6ce7943e6af65660bd |
| activitydiagram3/ftile/vcompact/ParallelBuilderFork.java | 155–205,218–236,268–285 | 13841498eee7b7b160c32ce9af9b89bc9221b035056dd2ada753304d9570cb04 |
| activitydiagram3/ftile/vcompact/VCompactFactory.java | 186–208 | fc22d8c40c3608795ffd3995e02efd7bbede0a9f4aa1cb52a4f03dd1e3b3dff7 |
| mindmap/CommandMindMapPlus.java | 137–143,171–182 | d4b6070d5a2d0e1b74f95956106b306d8ef1bd25858c5b77894e2c749cce5a9f |
| mindmap/CommandMindMapOrgmode.java | 137–143,178–189 | d6a9840f2bf1e3f32a5cab1c9db491a2752b37d89a91c4f3973b53c52cc08a60 |
| mindmap/CommandMindMapOrgmodeMultiline.java | 139–153,185–207 | 14a6b37e449791bd88b42b54a054b91d78daf72ddf42381829b40ed01664ed62 |
| mindmap/CommandMindMapDirection.java | 137–161 | 64940e094de1881e49b648bc2e3d1a703a180be8d9c384897f79944f33c7b27b |
| mindmap/CommandMindMapRoot.java | 134–156 | 06ecb74653d0d6ff780bb8960eda78fcb0f7ee7a2cf443f69a653b4aa3960f81 |
| mindmap/MindMapDiagram.java | 138–142,158–174,188–230 | ad1609f74d3796707f406c29f45736957a44122fc183f88349d98bf2e22c5616 |
| mindmap/MindMap.java | 191–224 | 74a1abd5a2dcb55445dc774ad2d58ef12827dc2a5e3aa9ce0622692877f05ce4 |
| mindmap/Branch.java | 127–161 | 0717f0179ef4f1583e8add65d99659bc6ff2f7fa85219b5051a518d959446ac8 |
| wbs/CommandWBSItemNew.java | 140–179,189–206 | f09c9b68fca677be74c1145fa549ab07be33e41823c628cda5e2e3d575c1f93e |
| wbs/CommandWBSItemOld.java | 144–183,193–227 | ee3e789c61c5e5b4cb6f0504837dab04701973c4ffd4ddff5d019e8d0b393544 |
| wbs/WBSDiagram.java | 230–303,310–328 | 358da9a4a466632afedd157b30a50b0fc55e1c0fcb8abf939c715f9d71100afd |
| utils/Direction.java | 149–159 | 3cc5b64e1e30a642f2314b23a5682898bd38f69ba597225330d0cd3605fbc5d7 |
| wbs/WElement.java | 215–225 | 0f93fde9d950b941cf04795a54de9d3036eb0e4a85364fd781314704824f9a3d |
| wbs/Fork.java | 132–139,173–232 | 88065f9aae786569612a7d877572d8fcd8b0c0e99d726868168cf565916aa086 |
| wbs/ITFComposed.java | 186–198,201–243,266–293 | 7c25f216bd6e1d36bb9d156b607b349b56e25fe6c5a278a81585ef2cb44800ed |
| wbs/WBSTextBlock.java | 174–185 | 71a00faf056f7ff1ffe3a39924c53609d2144a3a147fc2d98076306040101475 |
| wbs/CommandWBSLink.java | 142–153,170–184 | cea1d11b53f7837bb38bc5172a10ed9c051c66d617c0c2c4a213c190582e1d3b |
| jsondiagram/JsonDiagramFactory.java | 147–167 | 22ae9ae1f4101aa029c6198011181e57cd552371a35f8840313d9729e1530c53 |
| json/JsonParser.java | 201–223,264–343 | 78b2c149f9d8fb76cd912525f53e2c0abec1fa606747904e96a37d319bee41e9 |
| json/Json.java | 450–452 | 2fbdfa64a137ee58d8e96d8a86000d84eefd326e5bd7281cb7bdda5d34d00372 |
| json/JsonObject.java | 411–421,572–587,912–918 | a2894810da21e1287c93821803a4fc03a79c1530c5ef623aedc982b04a373c63 |
| jsondiagram/TextBlockJson.java | 185–209,249–287 | fe8caf987bd4001716649bc8badca5cb7c522e820f70f1dcac9ba2bc5953363a |
| jsondiagram/JsonDiagram.java | 150–163 | f45a4ef281001673b3a501dd6c430efb5c2e51d0aa8dd2e268595aa28f93e657 |
| yaml/YamlDiagramFactory.java | 148–164 | 49763c6ca543425f31c63255265f0f35ba7010719bb0c62b8f15dd623ad32824 |
| yaml/parser/YamlLine.java | 126–169,175–250,262–302 | 868aaeb45d1567c03c5c2a6830aa528cb6d9032a9434b41aabc1675331d4ec7f |
| yaml/parser/YamlParser.java | 118–166,170–227 | f676afed3976925d03382dff0f1ae86ccd2704999756276ef9c95825704c8877 |
| yaml/parser/YamlBuilder.java | 157–170,182–253 | b152242c11e8f195860ac0f71eeccaae36739fa0fcfebce8598f0ae38665e360 |
| yaml/parser/Monomorph.java | 167–184,228–243 | b64718fee43b56d000713dc8a07e7fca9e69ee1cb458e83ba3b6d6741df1d3a3 |
| yaml/parser/MonomorphToJson.java | 118–155 | 811b59d7900989a076a05dbdc4c3f2faec8938a2728150aa04fb4c483fb2413f |
| mindmap/IdeaShape.java | 112–120 | 3c4f2795f1b106c83eb42747ef64ecffb07c6305631b222fb25ad578c0ccd7e8 |
| activitydiagram3/ftile/vertical/FtileCircleStop.java | 165–168 | 01e3473a7ef45d5a79942259f970fac7a86ced1d70427ba192dabdc3e87250c3 |
| activitydiagram3/ftile/vertical/FtileCircleEndCross.java | 172–195 | ae6b661b40621c7cab43cef2ecf1043f2f2c31985ee92a8ec5137fd10bb665cb |
| activitydiagram3/command/CommandPartition3.java | 151–185,220–246 | d70b680df6f5e5bfe05a54445d0fb6d4db4eb726b1b40035fb08ce728c3f4e16 |
| activitydiagram3/command/CommandCloseGroup3.java | 134–151 | b2960122b10528c0d6f0da05a4c67e6ce1779ec99f589a3435b54124dfd9b937 |
| wbs/CommandWBSItemMultilineNew.java | 143–168,177–199 | 99de914b0725be27898e506b3adc800070fcdbb03b07d69811ed9557385560e4 |
| wbs/CommandWBSItemMultilineOld.java | 143–168,177–200 | a5d4890966ce5cb8448078c1fdc6233fc8c7ace9b261ce3c0f97421264ebd636 |
| wbs/WBSDiagramFactory.java | 130–138 | 02192935f1b936c53a5a24d7542a52d090bebbc4cd3663b4fcaefa8a0274306a |
| mindmap/MindMapDiagramFactory.java | 130–139 | 0ec4048347a3694734e1d25c7c88d516ae9331b4678ec341ad881e086764e026 |

