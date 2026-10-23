# Task 8 small-family semantic source inventory

Prepared 2026-10-23. Read-only research, not implementation/native acceptance.
Repo: /mnt/data/Projects/ive/.worktree/diagram-ports.
Only translation source: /mnt/data/Projects/plantuml-mit/src, MIT1.2026.9.
Java paths below abbreviate /mnt/data/Projects/plantuml-mit/src/net/sourceforge/plantuml/.
Read README, AGENTS, full docs/superpowers/plans/2026-10-23-plantuml-native-plan.md, task-3a-report.md and existing independent original report. No product files, existing fixtures/manifests or source files changed. No new Java probes/processes. No GPL repository/rejected candidate source/examples, gen/h/smetana algorithms, or icon/stdlib/theme data were read.

## Common contract

Exact wrappers: @startfiles/@endfiles, @startpacketdiag/@endpacketdiag, @startboard/@endboard, @starthcl/@endhcl, @startchen/@endchen. ArchiMate is an @startuml description command, not separate wrapper/family. Dispatch: core/DiagramType.java:168-248; PSystemBuilder.java:228,249-257.

Reconcile adapters with the controller's in-progress core repair. Reuse Budget, Context/Label, Scene, bounded SVG and terminal validation. Distinct typed models: direct layout for files/board/packet; ordered data renderer for HCL; checked graph geometry for Chen; description extension for ArchiMate. Do not manufacture class nodes/edges for other families.

Preserve defaults256KiB source/16MiB SVG/10,000 items/depth64/5,000,000 work/10s and caller limits. Charge before scanning, allocation, descendant traversal, expanded names, generated segments/ticks and measurements. Count attributes, links, fragments/markers, not just lines. Final models immutable; numeric IDs; own enum states rather than optional-wrapper fields. Consume all substantive input; common unsupported styles/resources/macros/pages fail, not strip. Literal paths/strings never trigger I/O.

Tests use should_… and given/when/then. Each family requires model/topology assertions, geometry assertions, bounded finite positive viewBox, native/fallback measurement evidence and terminal/usvg validation. Label presence alone is insufficient. Viewer acceptance uses standalone background/capture/exit workflow, with no manipulation of the user's running IVE.

## Files: ordered path trie and sibling notes

Anchors:
- filesdiagram/FilesDiagramFactory.java:123-132 selects FILES.
- FilesDiagram.java:151-173 slash paths and <note>…</note> body.
- FEntry.java:145-182 root/folder/file/note insertion; :198-202 emoji selection; :205-226 preorder dimensions/paint; :229-255 file text versus note.
- FilesListing.java:127-159 root traversal and note placement.

Existing fresh original-admitted input:

~~~plantuml
@startfiles
/app/src/main.rs
/app/README.md
@endfiles
~~~

Minimum useful native model: Folder/File/Note nodes, hidden root, explicit parent IDs and ordered children. Leading / is syntax, not a filesystem path. Interior segments create/reuse sibling folders by exact name; trailing / declares a folder. Final files append even when duplicate names exist. A file and folder with same visible name are distinct entries. Folder reuse preserves first-insertion order; output is DFS preorder, not sorted paths. Dot/.. components remain literal; never normalize/read disk.

Grammar-derived extension (not newly oracle-probed):

~~~plantuml
@startfiles
/app/src/main.rs
<note>
Entry point
</note>
/empty/
<note>
Root note after folder-only declaration
</note>
@endfiles
~~~

lastCreated is the final file, or null for trailing /. Notes after files are siblings in the file's parent, not children/tooltips. Notes after folder-only declaration are at root. Consecutive notes retain anchor. Model Anchor::Root/ParentOf(FileId). Plain folded-corner notes need actual measured geometry; full note Creole may be deferred.

Layout: measured preorder rows; increasing x per depth; y accumulates prior full row/note height plus gap. Upstream21px indent/2px gap. Folder/file emoticons reference Twemoji; assets remain denied. Draw original native folder/file primitives, disclose visual deviation, do not copy emoji bytes. This family implies no connector edges.

Tests:
- should_reuse_folder_identity_and_keep_preorder: /b/x,/a/z,/b/y -> b(x,y),a(z).
- should_keep_duplicate_files_and_distinguish_folder_names: /a and /a/x coexist.
- should_preserve_empty_folder; should_place_note_beside_last_file; should_place_note_at_root_after_folder_only_path.
- should_indent_children_and_clear_multiline_notes: proper x ownership, nonoverlapping measured y.
- should_keep_unicode_xml_and_dotdot_names_without_io.
- should_reject_unterminated_note_and_unknown_content. Original silently ignores nonpath/nonnote lines and accepts note to EOF; strict native should fail with location.
- should_bound_segments_depth_and_wide_folder_lookup_before_construction. Avoid expanded full-path cloning at every ancestor.

Subset boundary: reject note blocks if deferred. Reject unusual empty components (/a//b,//,/) explicitly if not supported, rather than normalize. Empty rendered tree errors. No asset/style/rich-note support inferred from plain rows.

## Packetdiag: bit widths, row fragments and scale

Anchors:
- packetdiag/PacketDiagramFactory.java:143-158 command registration.
- command/CommandPacketDiagStart.java:141-150 and End.java:141-151 optional packetdiag/braces; commands themselves do not track balanced braces.
- command/CommandNumRange.java:155-166 grammar; :169-222 attributes and auto start.
- command/CommandColWidth.java:144-164; NodeHeight.java:141-159; ScaleInterval.java:141-158; ScaleDirection.java:142-159; SameHeight.java:141-155.
- PacketDiagram.java:157-199 defaults; :415-418 field; :447-477 build/scale; :484-559 row split; :573-600 range/fragment data.
- PacketBlock.java:288-325 labels/abbreviation; :328-364 rectangle paint.
- PacketIndicator.java:150-236 tick/number geometry.
- nwdiag/NwDiagram.java:363-377 permissive attribute extraction.

Existing fresh original input:

~~~plantuml
@startpacketdiag
packetdiag {
  0-3: Version
  4-7: Flags
  8-15: Length
  16-31: Payload
}
@endpacketdiag
~~~

Minimum: PacketModel config and ordered fields {id,start,end,label,height}; generated row fragments retain field ID and widths. Inclusive width=end-start+1; accumulate widths in declaration order, wrapping at colwidth(default16), splitting fields. Render proportional rectangles under common bit scale. No graph.

Actual grammar accepts N[:],A-B[:] or *, whitespace+label, optional [attributes]. * starts after last declared end or0. Single number [len=4] spans4. Explicit bitStart does not insert visual gaps: 0-3 then8-11 are adjacent4-bit cells. Preserve that semantics or explicitly restrict native to contiguous ranges; never reinterpret explicit ranges as lengths.

colwidth controls wrapping; scale_interval numbering; scale_direction reverses both row block order and tick indices; same_height promotes each row's requested heights to maximum; node_height sets vertical unit. Original reduces final colwidth to first row width if shorter; default interval is set before reduction. Admit directives only when they actually affect drawing.

Original limitations to record:
- rotate is parsed into PacketItem.textRotation but never forwarded to PacketBlock; no text rotation paint. Reject unsupported rotate natively.
- leftOpen/rightOpen flags are stored but never read by PacketBlock paint, which draws a full rectangle; openerStroke is unused. Flags do not establish rendered continuation fidelity.
- multirow remainder branch PacketDiagram.java:502-505 can use full row width for a smaller remainder. Correct native splitting should conserve declared width and disclose upstream deviation.
- when same_height=false, row dimensions/y advance use last cell height, not maximum (:239-244,:270-282). A native max-height clearance policy is a deliberate fix.
- abbreviation loops over UTF-16 substrings with repeated measurement (:315-324); native must preserve UTF-8, bound work, retain full semantic label and document measured ellipsis.
- permissive attribute regex/find and default-on-invalid integers must not become silently ignored malformed attributes in strict native parsing.

Tests:
- should_convert_inclusive_ranges_and_keep_field_identity: widths4,4,8 in row0, Payload16 row1.
- should_split_field_across_three_rows_and_conserve_bits: colwidth8; 0-2 then3-22 -> second field fragments5,8,7 with same ID.
- should_place_explicit_ranges_by_declaration_width; should_auto_position_after_declared_end.
- should_reverse_fields_and_ticks_for_rtl; should_honor_scale_interval; should_equalize_requested_row_heights.
- should_draw_proportional_widths_and_shared_boundaries:4-bit/8-bit width1:2, ticks align with boundaries, labels clear scale.
- should_reject_descending_ranges_zero_lengths_overflow_unknown_attributes_and_rotate.
- should_reject_unbalanced_packet_braces_if_braced_form_admitted (stricter than upstream no-op brace commands).
- should_bound_fragments_ticks_total_bits_and_requested_height_before_expansion; tiny source can request millions of rows. Numeric endpoints max7digits, len/height upstream signed ints. All arithmetic checked; validate positive geometry.
- should_handle_colwidth_one_without_modulo_zero: original default half-width interval can be0. Reject colwidth0/node_height0 instead of original ignore/clamp, with explicit strict-subset docs. Tiny work/item/cancellation covers scale and fragmentation.

Subset: start with explicit positive ranges, actual row wrapping and scale; * and attributes need separate semantics tests. Styles/colors/rotation/rich labels fail until implemented. Equal-width label cells are insufficient.

## Board: activity trees projected to stage rows

Anchors:
- board/BoardDiagramFactory.java:123-143; CommandBoardPlus.java:136-154 zero+ then label.
- BoardDiagram.java:160-172 horizontal activities/dashed stages; :195-202 zero+ opens activity.
- Activity.java:128-150 nearest-lower-stage owner; :156-187 array/draw.
- BNode.java:130-138 increasing-stage invariant; :158-173 DFS columns.
- BArray.java:123-130 unique grid cells/maxima; CardBox.java:142-153 fixed150x70; PostIt.java:126-131170x90 pitch.

Existing original Backlog/+Add search/Done/+Fix login is valid, but it conceals the tree semantics. Grammar-derived revealing input (not newly oracle-probed):

~~~plantuml
@startboard
Checkout
+ Draft
++ Validate
+ Submit
+++ Ship
Audit
+ Record
@endboard
~~~

Minimum model: ordered Activity roots and Release nodes with stage=plus_count, parent and ordered children. Every activity has own cursor. Walk ancestors until stage lower than new stage, then attach. Stage jumps are accepted; +Submit/+++Ship leaves row2 empty. New zero+ line starts another activity. No implicit arrows.

Layout uses stage for y. First child inherits parent's column; each later child advances global DFS column after previous subtree. Above: Checkout,Draft,Validate column0/stages0,1,2; Submit column1/stage1; Ship column1/stage3. Audit starts after full Checkout branch width. Render actual stage rows and full-board dashed boundaries, not one generic card stack.

Upstream paints root once before iterating an array that includes root: duplicate root label/card at same position. Existing original metadata confirms duplicate Backlog/Done. Native should draw one root and report duplicate-paint fix. Measured cards/pitches may exceed upstream fixed150x70 to contain long labels, while preserving stages/columns.

Tests:
- should_start_ordered_activities_on_zero_plus_lines.
- should_choose_nearest_lower_stage_ancestor; should_keep_stage_jump_empty_rows.
- should_assign_columns_after_prior_subtree; should_offset_next_activity_by_full_branch_width.
- should_draw_stage_boundaries_and_one_root_card; should_contain_long_measured_card_labels.
- should_reject_release_without_activity: original getLastActivity indexes empty list.
- should_reject_common_resource_or_style_directives_in_board: do not let denied commands fall into catch-all root labels.
- should_bound_plus_depth_cards_stage_rows_and_width; stage count controls separator generation. Avoid unchecked recursive computeX/parent walking.

Boundary: plain labels/+ grammar only is useful. Most punctuation is ordinary root label upstream; recognized capability commands must still be rejected. No arbitrary Kanban properties/arrows/multiline Creole/colors/styles implied.

## HCL: PlantUML-specific ordered data

Anchors:
- hcl/HclDiagramFactory.java:142-159 converts HCL to JsonDiagram(HCL).
- HclSource.java:122-126 skips whole #comment lines, concatenates other lines without newline/separator.
- HclParser.java:135-163 top blocks/single-block unwrapping; :197-239 fields/values; :242-254 arrays; :262-305 token/quote rules; :308-330 punctuation; :166-194 representational functions.
- json/JsonObject.java:389-421 ordered append preserves duplicate fields; distinct EclipseSource MIT at :75-99.

Existing fresh original:

~~~plantuml
@starthcl
service "billing" {
  name = "ledger"
  zones = ["north", "south"]
  tags = { tier = "core" }
}
@endhcl
~~~

Minimum: ordered HclDocument blocks/fields lowered to shared ordered DataValue. Text block names join unquoted words and quoted labels. If one distinct top block, original renders its body directly; service "billing" does not appear as root title. Multiple distinct blocks become ordered entries with textual block names. Repeated block name overwrites prior value retaining first position through LinkedHashMap; repeated fields append through JsonObject.add. Implement or explicitly reject duplicates; no sorted maps or accidental collapse.

Quoted AND simple scalar values are strings:3,true,false,null are not typed JSON values. Objects {key=value} and quoted-string/object arrays work. Array parser silently drops bare tokens, not a full recursive generic list parser. Strict native must reject unsupported array members rather than lose them. No general HCL/Terraform evaluation, interpolation, heredocs, comprehensions, dynamic blocks or expression semantics inferred.

Function calls are data: f() -> string "f()"; f("x","y") -> object {"f()":["x","y"]}. No function executes. Defer and reject calls by name if not implementing this representational lowering.

Quoted escape consumes slash+next char, not JSON decoding; backslash-n becomes n. Unterminated quote may pass upstream. Newline concatenation may merge bare tokens. Native needs source locations and explicit escape/line-boundary policy, not silent standard-HCL interpretation.

Tests:
- should_unwrap_one_hcl_block_and_keep_field_order.
- should_keep_multiple_block_names_and_order; should_implement_or_reject_duplicate_block_policy.
- should_keep_bare_numbers_booleans_and_null_as_strings.
- should_link_nested_data_from_owning_field_rows: zones -> north/south, tags -> tier/core, actual child edges/compartments.
- should_preserve_duplicate_field_rows_or_reject_explicitly; should_keep_empty_object_and_array.
- should_skip_whole_hash_comment_lines_and_keep_hash_in_strings.
- should_reject_unsupported_bare_array_members_nested_arrays_and_calls_without_data_loss.
- should_reject_unterminated_quotes_unbalanced_containers_heredocs_expressions.
- should_bound_hcl_tokens_depth_strings_data_items_before_allocation; cancellation during scan/layout.
- should_keep_literal_resource_looking_strings_without_io.

Boundary: blocks, =/: fields, string scalars, nested objects, quoted-string/object arrays. Use native ordered data geometry; do not translate SmetanaForJson, stringify whole body into one label, or claim Terraform support.

## Chen ER: typed graph, not table fields

Anchors:
- cheneer/ChenEerDiagramFactory.java:130-146.
- ChenEerDiagram.java:126-167 lexical owner stack expressly not package containment.
- command/CommandCreateEntity.java:143-161 entity|relationship, optional "display" as alias, stereotype/color, mandatory {; :178-215 kind/owner.
- CommandCreateAttribute.java:151-167 grammar; :183-224 owner-qualified IDs, duplicate errors, actual attribute-owner link and composite scope.
- CommandAssociate.java:148-160 NAME[-=][cardinality][-=]NAME; :177-202 declared endpoints, normal/bold participation and central cardinality.
- CommandEndGroup.java:134-150 closing owner/error.
- svek/image/EntityImageChenEntity.java:148 weak, :188-213 rectangle/double border.
- EntityImageChenAttribute.java:154-165 key/multi/derived; :194-228 oval/dash/double/underline.
- EntityImageChenRelationship.java:151 identifying; :185-215 2:1 diamond fitting/double border.
- command/CommandSimpleSubclass.java:147-158,:175-205; CommandMultiSubclass.java:151-166,:184-237 d/o/U junction and true links.
- decoration/LinkType.java:201-202 bold; LinkStyle.java:179-1802px stroke.

Existing original:

~~~plantuml
@startchen
entity CUSTOMER {
  name
}
entity ORDER {
  number
}
relationship PLACES {
}
CUSTOMER -1- PLACES
PLACES -N- ORDER
@endchen
~~~

Minimum: ChenNode::Entity/Relationship/Attribute, stable aliases, AttributeOwner::Entity/Relationship/Attribute, independent typed undirected associations and ownership links. Attributes are separate ovals connected to owners, not table fields. Rectangles/diamonds/ovals need distinct measured geometry. Cardinalities1,N,(0,N) appear at association center. Reused attribute short name under different owners yields separate identity. Composite attribute {…} creates a parent oval with child ovals, not containing boxes.

Examples E -1- R, E =N= R, E -(0,N)- R. Only first -/= chooses normal/bold upstream; second is parsed without consistency check. Strict native may reject mismatched =1-/-1=. Never infer directed arrows. Exact original uses bold2px for =, not literal parallel lines; conventional double-line native participation would be an explicit fidelity change.

Semantic decorations: entity<<weak>> double rectangle; relationship<<identifying>> double diamond; attribute<<key>> underline,<<multi>> double ellipse,<<derived>> dashed ellipse. Draw or reject each, not plain stereotype words.

Tests:
- should_keep_three_node_kinds_and_ownership_edges.
- should_resolve_aliases_and_owner_qualified_attributes.
- should_link_composite_attribute_children_without_container_parenting.
- should_keep_relationship_attributes_and_central_cardinalities.
- should_draw_rectangle_diamond_oval_and_intersect_actual_boundaries: diamond text fits diamond interior, not just bbox; true ellipse/diamond intersections for links.
- should_render_key_multi_derived_weak_identifying_if_admitted.
- should_keep_undirected_links_and_participation_stroke; no arrowheads.
- should_reject_undefined_endpoints_orphan_attributes_duplicate_attributes_unbalanced_owners.
- should_reject_unsupported_subtypes; should_bound_ownership_depth_expanded_names_and_graph_work.
- should_preserve_parallel_associations_and_finite_cyclic_disconnected_geometry.

Boundary: subtype syntax requires typed d/o/U junction circle and subset/superset decorations; reject until implemented. Flat Chen plus attributes is substantive. Source allows nested entity declarations while owner stack nonempty without containment; reject if deferred, do not silently make packages. Dagre/Graphviz ordering/routes differ.

## Icon-free native ArchiMate: description extension

Anchors:
- descdiagram/DescriptionDiagramFactory.java:207-209 three commands.
- command/CommandArchimate.java:148-176 grammar; :212-236 declaration/color; :225-233 icons -> $archimate lookup.
- CommandArchimatePackage.java:148-177,:215-240 true package ownership and symbol/color.
- CommandArchimateMultilines.java:149-169 square delimiters; :225-247 duplicates/body; actual leaf uses RECTANGLE(:231-232), not ARCHIMATE.
- decoration/symbol/USymbols.java:140 ARCHIMATE=new USymbolRectangle(SName.archimate).
- USymbolRectangle.java:139-172 rectangle/10px margins/measured label; :178-213 big caption.

Existing original:

~~~plantuml
@startuml
archimate #LightBlue "Order service" as Orders
archimate #LightGreen "Customer record" as Record
Orders --> Record : reads
@enduml
~~~

Minimum: description Kind::Archimate with safe explicit fill color, label/alias and measured rectangle, authored relation endpoints. Remains description family. Native command does not encode full ArchiMate layer/category/relationship constraints. Color must actually paint. Literal #RRGGBB-only initial support is honest, but existing named-color fixture remains unsupported until its names resolve; do not drop color.

Package form archimate #DDEEFF "Domain" as Domain {…} means actual container. Multiline archimate #DDEEFF A [ then body lines then ] means multiline rectangle. Original ignores text on opening/closing delimiter lines; strict subset should require delimiter-only ends. Reject forms when deferred.

Every <<…>> position selects an icon, not harmless stereotype text. Reject all icon-bearing syntax, sprite/openiconic/image syntax and stdlib macro/include calls until separately licensed traced assets exist. No sprites/archimate or stdlib/archimate data inspected/imported. Invented @startarchimate must fail.

Tests:
- should_keep_archimate_in_description_with_color_alias.
- should_paint_distinct_safe_colors_and_measure_labels.
- should_resolve_relation_endpoints_and_direction.
- should_contain_package_children_if_admitted; should_keep_multiline_body_if_admitted.
- should_reject_icons_in_every_grammar_position_and_stdlib_macros.
- should_reject_unknown_color_invalid_alias_unsupported_package_wrapper.
- should_bound_labels_colors_and_group_depth.

## Existing original-side evidence

Six source fixtures and .result.json metadata read without modification. Each status original-rendered, expected labels present, real shapes. This is readback of existing independent evidence, not a rerun/native acceptance. Full report /tmp/ive-plantuml-native-oracles-report.md. Jar SHA256 e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1f1a515f7f70dd.

| Stem under /tmp/ive-plantuml-native-oracles | Source SHA256 |
|---|---|
| files-tree | 49292373d102be17bc4b8cdca87e8145810714c12520b2f49a3dcea21a260a0c |
| packetdiag-ranges | ed4d574921091eaf20c41f79fb22f97ff647e24388a8472d2c50f13b51c45669 |
| board-columns-cards | 3479cbd3c10b0e2df33f6fa27c3e9fd1629d48acb40370b0a579f5ba3df8080c |
| hcl-nested-data | 40fb9ae3934394d2712f93624b8957bfd240d7af2098ae6c5223fa8299ecce39 |
| chen-entities-relationship | d689aebba3196e628ed7c84bc805b89cfb806985f3f9df1e07d7fb4f3f27f163 |
| archimate-authentic-uml | 5b3771ce83c85f3542cfa9be8ca5e3e77db3d77da91201a750db2a8a54f0f271 |

Board root labels duplicate upstream; files original invokes emoji drawing. Native original primitives and single root paint are disclosed differences. Only ArchiMate uses original CLI Liberation Sans flag; others renderer defaults. Proposed extension examples/tests above are source-derived, not falsely claimed oracle runs.

## Source/notice integrity

54 exact source hashes below. First74 lines of52 initially indexed files share generated MIT distribution-header SHA256 e81958f131bc72c47da814eaf865f905c22c04fd0eb2b322d206f222d674fed0 (original UTF-8 bytes including LF). Names Arnaud Roques2009–2024 and MIT grant. Retained older header under generated grant is not erased/relabelled.

Packet files additionally name Arnaud Roques2009–2026 and Original Author kolulu23, except CommandSameHeight names Duy Nguyen. PacketDiagram, CommandNumRange, CommandScaleDirection name Contribution The-Lum. Preserve these names. JsonObject.java has separate Copyright(c)2016 EclipseSource MIT grant at75–99. Preserve that notice for translated ordered-object behavior. Other examined implementation headers name Arnaud Roques. Boilerplate asset-provider list is not permission to import assets.

The manifest must map actually translated behavior to Rust destination and notice, and byte-compare selected sources against authorized source-JAR entries through controller audit. Hashes here verify local artifact, not Maven authenticity. Package-info files carry no translated logic and are omitted.

Existing retained license: /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/provenance/LICENSE-PlantUML-MIT.txt
SHA256 718f2ca0c807d57d7443183b13124ea966232bac23281b5d36416a9d097dce98

Additional precise retained-comment hashes (comment delimiter through closing delimiter, no trailing newline; original UTF-8). These include the old upstream comment for edition-history fidelity, not a change to the generated MIT distribution grant:

| Relative file | Comment lines | SHA256 |
|---|---|---|
| json/JsonObject.java | 75–98 | 2180329e139c8e41510b6362cc2b905a2902d920df8d21fda165d3bcd0b5e7d0 |
| filesdiagram/FEntry.java | 75–108 | e8f658bef62036242942d8362ac0e265cb4fda8dfe792ed6062714de3acc7482 |
| packetdiag/PacketDiagram.java | 75–109 | 681d16f357bd3a45a5d377f37061b9800b0eaaeae19ebf31d2766aece0c25e67 |
| packetdiag/PacketDiagramFactory.java | 75–109 | 963eaf3d5ba5af1d60a2f8bc3096ca38cbd304801e7824c1a22ffacc03680f60 |
| packetdiag/command/CommandSameHeight.java | 75–109 | 2764641eac330618e91ad339614d078c73a9900b5f51e9ff521b0e487f2f4d6a |

| Authorized relative Java path | SHA256 |
|---|---|
| board/Activity.java | 5b2add7755cad9040bb80f49f788f69d90c87f2755e10ef92ef98337ac08a521 |
| board/BArray.java | 145d07b35049099a00216017fbee37db716a24eb5da12481bdaf4b6a3e1b046a |
| board/BNode.java | f07671a3bf3032f22ffa9afb0833dd8606660582ddd85e65144128b58726fa55 |
| board/BoardDiagram.java | f6a098aae12e37da640dad1153c63df27545453a1eb08ca7e9eb4b2fd87db5a4 |
| board/BoardDiagramFactory.java | ab153d81d84c5369876c3215d4295d2eba2a5cfb28a625cfb8356803633194ee |
| board/CardBox.java | f3b8a925b6484f8b7d095599a52375e2103e71568739f15d7cf4af3c2ee5f03a |
| board/CommandBoardPlus.java | 393d60f87b57a7a2dbadaa7b5e534b78baa6295a41c4cdfad23f2366b8401960 |
| board/PostIt.java | e0c40b492cfc1465c627540b562ac26f98c52fe156328559e708ba2cb3206446 |
| cheneer/ChenEerDiagram.java | 0b059002a1b59627343447dc77e362cd1c4130fe860cfa9c7c87c3e95a3ff932 |
| cheneer/ChenEerDiagramFactory.java | dd1d73aad9f1be4a8a9a7fe533e894c4c0ba1fe2515bff76f40ce8c74bc083c6 |
| cheneer/command/CommandAssociate.java | dd7535cb4900aca7738d380440472e50b39436d10c7f9ff430303680bd6162db |
| cheneer/command/CommandCreateAttribute.java | 5132aa2ebed227a4797091ae7f92d30377629eaf041d5920f08fd4581eddec36 |
| cheneer/command/CommandCreateEntity.java | 5dc5ecec3bb8dc4eb2c6060d50a231c93fdf18d4c0db15fbf142f2ae35e44485 |
| cheneer/command/CommandEndGroup.java | a8112e6b39f38b37030e867dcbf5afed521fde86fa4b0b056ec8d339152ef59d |
| cheneer/command/CommandMultiSubclass.java | 9e0d315e0ea95af1266aad7314272c527f0954d5aa9518dcc6bb38d753bee199 |
| cheneer/command/CommandSimpleSubclass.java | 639ba769cdafe45648d325e06dc970fae72c218b7d66184acc8ffe55ae9c1896 |
| core/DiagramType.java | 590d9685e213d5ad51bf6113239081c7a55973715a306f6f16a2a78480659483 |
| decoration/LinkStyle.java | f34b53429547a46203b03b07c0dcab5eb26eb797110fe043bc1956d3626c407a |
| decoration/LinkType.java | 1346a0a1efdd3545377eae89c02d20274bfc08391b3d6176c2e1325cf9cbbb12 |
| decoration/symbol/USymbolRectangle.java | 2e794e523a701cb54264906ba50a5461e3e8b93308d6c44ebb458bfaedd9d0a5 |
| decoration/symbol/USymbols.java | 0f2ae92a208f717c539a403e485676762cf070fa6e7caa6b68f94bba9adb24c5 |
| descdiagram/command/CommandArchimate.java | 9c4e956d56b471342d01337eb2fe3440961deaed824defbe38c709e4fd5d4c8b |
| descdiagram/command/CommandArchimateMultilines.java | 120bd358012c850f5eecd8bf34331c250893159beea36739cc697624dc03c74e |
| descdiagram/command/CommandArchimatePackage.java | 2b34327a1153d7aa0ad6a42a854a6935e3541feb62b9b18d15fd30ca294d8b41 |
| descdiagram/DescriptionDiagramFactory.java | 18ff38a4f03d7b396cad21f9fc0464280a54f682c195cc2a3df79acb710a14df |
| filesdiagram/FEntry.java | a499ebfff655214e6e77251b221e542e20acf2892e23e9c64f55875b41676195 |
| filesdiagram/FilesDiagram.java | 010231ff785e054a5b89708cad02304ec54ba5350408f5bac0bd5a6632c08943 |
| filesdiagram/FilesDiagramFactory.java | 4b8a10bfc2cea4b0810332dce2c647ee830018ed0579691b07a5f2ecbfa6288a |
| filesdiagram/FilesListing.java | 10c2c9f7ac8cfa2890bd7767f36062df25a57643a081b3f1c1f98b9a9c250351 |
| filesdiagram/FilesType.java | 47fbff9461b14881c71af464c3b7cc6eb47982d8affed47be46e9933454144c9 |
| hcl/HclDiagramFactory.java | 5dff20eb73401b058a94ca88a40a0c87542df3130f784ac74977d38ab083aa94 |
| hcl/HclParser.java | f372a7c8e217e83986e8359988ca68418b48710a6950177b5ecc39d1ef05d09f |
| hcl/HclSource.java | 0bc7b0f32e18c5304905e759c6ec6b042063c2cc280b12f37f922e9365996839 |
| hcl/HclTerm.java | 44cdff97b6160f41dd99715feadeab0e1657fbe89061811f7145dd670233dc79 |
| hcl/SymbolType.java | af240140f80a8143babdca78dd4eea903022c44e625f14696fa6f7fcdd1eeff1 |
| json/JsonObject.java | a2894810da21e1287c93821803a4fc03a79c1530c5ef623aedc982b04a373c63 |
| nwdiag/NwDiagram.java | de4367425d36699de08115ede130808ee1fe637dd94b8480043616797629a33f |
| packetdiag/command/CommandColWidth.java | cabfe5b701d19ad5319aea43a2ba10dd397087b256227f3fb445be15536adba1 |
| packetdiag/command/CommandNodeHeight.java | 7fc01797660e4e085891cd809c0eaf8d7cbdc508f0848b0ed5610743376af669 |
| packetdiag/command/CommandNumRange.java | aa39aa72767f58098585f47115632aa9b318844e117a951cfa79fe71344f23e9 |
| packetdiag/command/CommandPacketDiagEnd.java | 613835aa795bb4ab75cf6ec1314ff3031267fdd9c6f3f2c9a9dcb4910bcbae4c |
| packetdiag/command/CommandPacketDiagStart.java | ecd9da600b3d21a1ed82aa442a55a7346d59166795cff7a16d99895e18c9a7e1 |
| packetdiag/command/CommandSameHeight.java | 5066322812e31b91bd788dc03a611a11199c66c93315fdb51790c8f02a78b09e |
| packetdiag/command/CommandScaleDirection.java | fe5762087b20c2b4ab7e2501ba6714710b4802496783fe3d49f7efafdd79e7e4 |
| packetdiag/command/CommandScaleInterval.java | dcfb42c720974ede07acb21a4485cc3ffc5f24b13a78fb5d17b4099e2f3a3350 |
| packetdiag/PacketBlock.java | 01b6a68d0016ee1ded28a844315eed855abbabffc28fc1fe985498a14b836b48 |
| packetdiag/PacketDiagram.java | 21bf35b925a3a8b0810190f77a077f58582a6a2acf88f96c1ad03a6221b3939d |
| packetdiag/PacketDiagramFactory.java | 0a86fd90b9a658f5026a058e001c52d5416b9004176e46183207e3daa2c109a8 |
| packetdiag/PacketIndicator.java | ae2cbfc8ff3e254b9284d0323169a4cefe6d9857c814a07a967194dd9791be87 |
| PSystemBuilder.java | 00383dd63082f93e471900cb70f44daee20899a862e4c3590fd95660b5d6e679 |
| svek/image/EntityImageChenAttribute.java | 7a4c20e3e0d361bd71c135bbc1c3ea28f1e15d031a7100ec374160542e26da5e |
| svek/image/EntityImageChenCircle.java | af35627bf01bac6767b302874d331d2036cf11ca3db753d9dca0f4be5ea45e06 |
| svek/image/EntityImageChenEntity.java | 2b8dab51933e203d773e6df0e5e7c89b2dd099b2552c0de304a78013c042f7e0 |
| svek/image/EntityImageChenRelationship.java | a0ebf31a21bde41913e0e6f38673c35d8a8845d7a78b6fc076f09d5515061b21 |

