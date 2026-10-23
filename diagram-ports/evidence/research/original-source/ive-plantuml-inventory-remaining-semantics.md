# Remaining native PlantUML semantic preparation

Prepared 2026-10-23. Research only: no product edits, runtime backend, dependency change, app launch, or new renderer claim. Read repository README/AGENTS, full accepted native plan, and Task 3A report. The frozen common API report remains authoritative while its review fixes are in progress.

## Evidence and authorized boundary

Only translation/reference implementation consulted: /mnt/data/Projects/plantuml-mit/src/net/sourceforge/plantuml/, MIT edition 1.2026.9. No rejected candidate implementation/examples, other edition, GPL repository, gen/, h/, smetana/, SmetanaForGit implementation, external dependency implementation, or untraced asset was consulted/imported. Referencing a caller's dependency name below identifies a boundary; it is not permission to translate that dependency.

The 92 exact source files in the appendix were SHA256-hashed and independently byte-compared to their exact entries in /mnt/data/Projects/plantuml-mit/plantuml-mit-1.2026.9-sources.jar: all 92 equal. The archive hash was independently recomputed as 88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3. This verifies local identity, not authenticated Maven provenance.

Original-side reference evidence already exists at /tmp/ive-plantuml-native-oracles/, manifest.json and /tmp/ive-plantuml-native-oracles-report.md. Its seven actual source files are reproduced below for exactness; do not overwrite them. Original jar hash recorded there: e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1f1a515f7f70dd. No new Java/JS/dot/browser reference runs were needed here; recommendations beyond those seven originals are source-derived test proposals, not newly verified oracle admissions.

Paths below use p/ = /mnt/data/Projects/plantuml-mit/src/net/sourceforge/plantuml/. Line anchors refer to these hash-frozen files.

## Real family registration, with two important exclusions

PSystemBuilder.java:201–258 registers BPM (210), chart (227), flow (241), wire (245), Git (247), EBNF (252), regex (253), standalone Creole (235), Math (233), Latex (234). DiagramType.java:168–279 maps their explicit @startbpm/chart/flow/wire/git/ebnf/regex/creole/math/latex wrappers. Native wrapper dispatch should select exactly one dedicated model; no fallback to UML sequence, class or Mermaid source.

Ditaa is different: PSystemDitaaFactory is imported at PSystemBuilder.java:136 but never registered in 201–258, and DiagramType.java:191–196 has no ditaa branch. Its enum constant and on-disk classes do not establish functioning MIT-edition dispatch. It is not a successfully admitted family from this source inventory. Chronology is similarly not a delivered family: factory registration is commented out at PSystemBuilder.java:240.

Version/license/welcome/help/font/icon/sprite/color/crash/donation output are application utilities, not evidence for regex/EBNF/chart/etc. Factory count is not a native capability count.

## Shared prerequisite: family-preserving source admission

The current engine source.rs::prepare requires @startuml, trims every line, removes empty lines and runs UML comment processing. Those transformations must not be reused unchanged for these families:

- Git columns and wire indentation are semantic; preserve raw leading spaces/tabs and all grid rows until the family lexer decides their role.
- Regex literal apostrophes, backslashes, slashes, whitespace and character classes are not UML quotes/comments. A regex beginning with an apostrophe is not an empty comment.
- EBNF terminals use both 'single' and "double" quotes, ?special sequences?, and (* comments *). Multiline terminals/comments need lexical state. A quote containing ; must not end the rule.
- Creole blank lines and code content are document layout; preserve them.
- Wrapper recognition, BOM/CRLF normalization, located spans, single-output restrictions and denied resource/preprocessor capabilities can remain shared, but the lexical body rules belong to each family.

Proposed immutable family models should use typed IDs and enums, never visible text as identity. Preserve original source spans. Reuse Budget, Context/Label measurement, Scene and safe SVG pipeline, with carefully reviewed additions for styled runs/colors/paths. Current Shape supplies polylines/polygons, not native cubic path primitives; railroad rounded bends can first use honest finite polyline approximations or gain a bounded typed path variant. Do not stuff an unrelated family's model into Uml, rewrite raw PlantUML to Mermaid, or pass source through a JVM/JS/dot runtime.

## 1. Regex: typed railroad expression, never regex execution

Verified original input, regex-branch-repeat.puml:

~~~plantuml
@startregex
INV-(A|B)[0-9]+
@endregex
~~~

Its original 236×106 viewBox is a baseline, not a requirement for native pixel equivalence. Required semantics: ordered concatenation of INV-, a choice between A/B, then one-or-more occurrences of the digit range. A class [0-9] is one matcher/set, not three literal characters, and + is a returning loop with no zero-length bypass.

Exact reference anchors:

- p/regexdiagram/PSystemRegexFactory.java:125–140 and CommandRegexfSingleLine.java:141–161: family and one-line source collection.
- RegexExpression.java:119–185: tokenizer; 189–233 lookaround/named-group detection; 235–308 comments and special parentheses; 311–339 quantifier spelling; 342–448 escaping, groups and classes; 451–502 unicode/anchor handling.
- ReTokenType.java:112–145: typed tokens and implicit concatenation conditions.
- ShuntingYard.java:124–171: precedence/parenthesis lowering.
- PSystemRegex.java:205–251 consumes postfix tokens into typed ETile combinators; 270–305 preserves groups/assertions/repeat distinctions; 309–378 handles bounded-repeat labels; 381–427 choice/concatenation. Do not reproduce unchecked stack operations or numerical fallbacks.
- GroupSplitter.java:120–136: ranges and escaped group members. ETileRegexGroup.java:165–199 draws a multirow class box, while ETileRegexGroupAllBut.java:136–163 separates exclusion marker and excluded members. Negation is not optional text.
- Shared ETile anchors in the EBNF section describe layout, not a dependency on the Java runtime.

First meaningful increment: exactly one regex expression with literals/escaped punctuation, concatenation, nested groups, alternation, nonempty positive/negated classes with ranges, . and an explicitly enumerated class escape subset, and ?, *, +, {n}, {n,}, {n,m}. Store expression as RegexExpr with Literal/CharacterClass/Sequence/Choice/Repeat variants and typed RepeatBounds. A Group wrapper can retain capturing-vs-noncapturing source semantics even if its visible geometry is identical. Use one shared measured railroad compositor below this model; it is not a matcher, compiler, or DFA expansion.

Retain symbolic repetition limits and their visible label. Never clone the child n times. Validate integer syntax with checked parsing, n≤m, upper bound and source span before AST allocation. Initial maximum repeat count recommendation is 1,000,000, configurable only under existing hard policy; the number is metadata, not memory allocation. {0} and {0,0} deserve explicit semantics/tests: upstream PSystemRegex.java:312–315 treats min=0 and max=0 like zero-or-more. Do not accidentally advertise exact-language regex evaluation by copying that rendering quirk. Either draw the literal annotation with an explicit fidelity note or reject those corner spellings in the first subset.

Separate increments: Anchor, Assertion(positive/negative lookahead/lookbehind), NamedGroup, UnicodeClass and POSIX class. Until their real model/decorations exist, reject them with their exact span. Reject unsupported backreferences/flags/atomic groups/possessive/lazy quantifiers instead of consuming '?', '+', or special parentheses as harmless text. Although upstream recognizes some of them imperfectly, matching an accidental fallback is not a support claim. Multiple regex body lines must be rejected until their upstream stack-combination behavior is specifically resolved; never return only stack.peekFirst() from several unrelated expressions.

Meaningful tests:

- should_preserve_regex_precedence: a|bc yields Choice(a, Sequence(b,c)); (a|b)c yields Sequence(Choice(a,b),c).
- should_preserve_regex_repeat_bounds: ?, *, + and {2,4} have distinct min/max, all labels retained, one child regardless of count.
- should_preserve_regex_class_exclusion_and_escape: [^a-c\]] retains negation/range/escaped ], not a topology branch.
- should_draw_regex_branch_and_repeat: ordered A/B alternatives share entry/exit; + has loop but no bypass; * has both; labels fit boxes and no route crosses a label.
- should_reject_unbalanced_regex_groups, should_reject_inverted_repeat_range, should_reject_unsupported_regex_assertion, should_bound_regex_depth_and_tokens, should_cancel_during_railroad_layout.
- Custom tiny work/model/SVG limits and 65-deep groups must fail before excessive parse/layout/measurement. Pathological regex strings must not trigger a regex engine or match-time backtracking.

## 2. EBNF: ordered rules and structurally distinct terminals/references

Verified original input, ebnf-optional-repeat.puml:

~~~plantuml
@startebnf
invoice = "INV", digit, { digit }, [ "-X" ];
@endebnf
~~~

Original viewBox 402×130. Required semantic model: named invoice rule with Sequence(Terminal(INV), Reference(digit), Repeat(Reference(digit),0..∞), Optional(Terminal(-X))). An undefined digit is a grammar reference, not an unresolved UML entity and not an instruction to expand another rule.

Exact reference anchors:

- p/ebnf/PSystemEbnfFactory.java:125–142: single/multiline rules and comment handlers.
- CommandEBnfSingleLine.java:141–166: ID = expression ; with optional above/below comments; 185–197 constructs rule.
- UBrexCommandEbnfMultilines.java:132–151,164–166: multiline collection through semicolon ending.
- EbnfExpression.java:143–203 lexical symbols (both quote kinds, special sequence, grouping/optional/repetition, }-, concatenation/choice, '-', comments); 219–243 rule name/title and empty-body rail; 266–295 distinct engine operations; 298–337 string/comment scans. The native parser must check end-of-input in readString rather than inherit its unbounded loop.
- Symbol.java:112–160 records operator kinds and precedence (* > comma > |).
- ShuntingYard.java:124–208: comments above/below, exception via NOT plus concatenation, optional/repeat lowering, }- as one-or-more.
- EbnfEngine.java:148–179 distinct boxes/options/repeats/count labels; 218–221 requires exactly one result; 229–242 annotations.
- PSystemEbnf.java:148–155 appends ordered rules; 170–185 stacks them. Name reuse/reference expansion is not performed here.
- ETile.java:135–146 has width, above-baseline extent H1, below-baseline extent H2; ETileConcatenation.java:139–187 aligns connector baselines and reserves intertile gaps; ETileAlternation.java:132–156 stacks branches with shared rails; ETileOptional2.java:141–180 adds a bypass; ETileOneOrMore.java:141–189 returns from exit to entry; ETileZeroOrMore.java:114–117 composes optional(one-or-more). ETileBox.java:253–269 distinguishes terminal rectangle, dashed special sequence and rounded reference/literal box.

First increment: ordered named rules; both quoted terminal forms; references; explicit comma concatenation; | choice; grouping; [...] optional; {...} zero-or-more; {...}- one-or-more; rule names; multiple rules; bounded multiline rules. Model EbnfDocument { rules }, EbnfRule { id,name,expression }, with EbnfExpr variants. Keep terminal quote category/source and Reference distinct even if labels coincide. RailroadGeometry { width, above, below, entry, exit } is the useful shared service with regex. Layout bottom-up into ports; rules stack vertically with measured title clearance. Preserve branch source order.

For comments, exception (-), n * term and ?special sequence?, either implement actual annotation/exception/repeat-count/special-box geometry or reject the entire construct. Do not strip comments and call full grammar support, because (*...*) can become visible attached notes. Numeric repetition and exception are not a universal Rust/regex grammar shortcut. Empty rule bodies may get an intentional Empty expression and start/end markers once tested; do not emit a blank successful document.

Meaningful tests:

- should_preserve_ebnf_rule_and_reference_order: invoice references digit without expansion, followed by an independent digit rule; duplicate-name policy explicitly matches append semantics or rejects as documented subset.
- should_keep_ebnf_terminal_quotes_and_semicolons: "A;B" and 'C,D' remain one terminal each; a ; within quotes/comments does not terminate prematurely.
- should_draw_ebnf_optional_and_repetition: optional has bypass/no loop, zero-or-more both, }- loop/no bypass; branches and concatenations have shared entry/exit and no reversed visible order.
- should_distinguish_ebnf_terminals_references_and_specials, should_attach_ebnf_comments_to_their_expression (when admitted).
- should_reject_unterminated_ebnf_quote_comment_and_rule, should_reject_mixed_delimiters, should_reject_unimplemented_ebnf_exception_and_count, should_bound_ebnf_depth_items_and_layout.
- Deep nesting, thousands of independent rules, a megadigit repeat token, long multiline terminal and low SVG quota must reject at the relevant admission boundary; never recursively expand references.

## 3. Chart: genuine data-to-coordinate layout

Verified original input, chart-bars.puml:

~~~plantuml
@startchart
x-axis "Week" ["Mon", "Tue", "Wed"]
bar "Orders" [3, 5, 4] labels
legend right
@endchart
~~~

Original viewBox 593×451. Source ChartAxis.java:129–136 defaults to min=0,max=100,autoscale=true; includeValue at195–203 expands these bounds rather than resetting to data extrema. The oracle actually paints ticks 0,20,...100 for values3/5/4. Do not normalize this baseline silently to0..5.

Exact anchors:

- p/chart/ChartDiagramFactory.java:138–152 prioritizes chart-specific axes, bar/line/area/scatter/legend/stack/orientation/annotations over common commands.
- command/CommandChartHAxis.java:137–157 accepts h-axis/x-axis title, numeric range or category data, spacing, label-right, grid; 202–217 chooses numeric vs category axis; 220–233 simple comma-splitting caveat.
- CommandChartVAxis.java:139–164 accepts v/y/v2/y2 axes, title/range/category labels/custom ticks/spacing/label-top/grid.
- CommandChartBar.java:140–157 bar name/data/color/labels; 169–198 creates ordered series; 201–214 number parsing.
- CommandChartLine.java:140–161 includes secondary-axis and labels flags; 179–221 distinguishes index data vs explicit(x,y) pairs; 240–260 begins coordinate parsing.
- ChartSeries.java:119–153 distinguishes BAR/LINE/AREA/SCATTER and explicitX/indexed data.
- ChartDiagram.java:128–158 defaults; 173–194 orientation mapping; 267–328 consistency and autoscale validation. Explicit x-coordinate series require numeric explicit x range, cannot mix category and coordinate formats, and preserve axis assignment.
- ChartAxis.java:186–203 affine value mapping and include-value rule.
- ChartRenderer.java:155–163 margins and measured legend/layout from210–227; BarRenderer.java:179–247 uses category slots, clamped zero baseline, sign-dependent bars and end labels; LineRenderer.java:139–188 connects actual x/y points or category centers.
- CommandChartOrientation.java:133–167, CommandChartStackMode.java:133–167, CommandChartLegend.java:133–173 define actual orientation/group/legend semantics.

First increment: categorical vertical bars plus categorical line series, series names/order, finite signed values, default/explicit nondegenerate y range, value labels for bars, measured legend at the admitted position(s), category labels and axis title. A ChartModel owns AxisKind(Category/Numeric), ScaleRange(Auto/Fixed), SeriesKind(Bar/Line), indexed samples, LabelsMode and LegendPosition enums. No Mermaid XY source generation. Draw native axes/ticks, measured labels, exact per-series samples, zero baseline, actual heights and line endpoints. A single series plotted as equal-height label boxes is not chart support.

Grouped multisets are a meaningful follow-up; stacked values need cumulative positive/negative baselines and their own model/tests. Horizontal orientation swaps category/value roles according to ChartDiagram.java:173–184, not a string rename. Coordinate-pair lines/scatter, secondary axis, area, custom ticks/grids/spacing, annotations, stereotyping/styles and arbitrary orientation are separate increments; reject their commands/flags until effective. Upstream BarRenderer/LineRenderer truncate with Math.min(categoryCount,values.size()) (212 and175); initial native subset should reject mismatched lengths explicitly rather than silently discard samples. This is a documented stricter admission policy.

Bounds: parse all numbers as finite checked f64; cap magnitude to a documented sensible domain (recommended |value|≤1e12 before scaling); enforce min<max and denominator finite/nonzero; bound generated tick count (≤1,024 under item quota) before generation, including custom tiny spacing. Series×categories is charged before grouped allocation; no cartesian expansion beyond quota. Empty data/axis/series and clipped out-of-range fixed-axis samples need a documented deliberate rule; initial reject is preferable to an accidental blank scene.

Tests: should_preserve_chart_category_and_series_order; should_keep_chart_default_zero_to_hundred_axis; should_scale_chart_positive_zero_and_negative_values; should_draw_chart_bar_labels_and_legend; should_keep_chart_line_point_coordinates; should_reject_chart_nonfinite_or_inverted_axis; should_reject_chart_mismatched_sample_count; should_reject_unimplemented_chart_secondary_axis_stack_and_annotation; should_bound_chart_ticks_and_sample_product. Assert actual pixel-height ratios against the chosen range, zero-baseline alignment, monotone sample x positions, label/legend containment and all series samples, plus terminal/usvg validation.

## 4. ASCII Git: parse a history graph, do not execute git

Verified original git-linear-history.puml:

~~~plantuml
@startgit
* c3
|
* b2
|
* a1
@endgit
~~~

Original viewBox47×155. This means commits c3→b2→a1 (arrows toward older ancestors), not a generic three-line text box.

Exact anchors:

- p/gitlog/GitDiagramFactory.java:129–143 collects raw body lines without trimming.
- GitTextArea.java:127–140 scans commit rows using first *, creating commits with cursor coordinates; 154–159 lookup by name.
- CursorPosition.java:148–177 finds optional(parenthesized comment) and commit-name token; 179–185 commit at cursor; 191–285 traces ASCII graph connector patterns with |,/,\ ,_,.,- and movement.
- Commit.java:122–127 validates * / *- marker forms; 151–178 identifies downward candidate lanes and horizontal '.' merge tips; 182–190 traces ancestor commits.
- GitDiagram.java:129–132 turns commits into GNode objects; 139–155 explicitly delegates drawing/layout to SmetanaForGit. Do not translate or inspect that excluded layout implementation.

First useful native increment: an ASCII graph scanner with exact row/column ownership, typed commits/parent edges and measured commit labels, at least a linear chain plus a simple two-parent merge/branch. Use GitHistory { commits, parents, source_lanes }, separate from UML. May use extracted checked Dagre adapter for native DAG layout, or a dedicated lane layout; preserve ancestor topology and explicitly document geometry difference from original Smetana. A label c3 is an identifier, not a source-control lookup, and no on-disk repository/git subprocess is involved.

Recommended narrow first ASCII dialect supports star commit rows, | vertical continuation, /\ diagonal branch/merge connectors and a documented *- tip form only once tested. Connector-only lines must be consumed and validated. Reject unknown ASCII decoration, ambiguous crossings, dangling lanes, duplicate/empty commit names, malformed parentheses, multiple stars per row until interpreted, and octopus/underscore/horizontal patterns until actual graph tracing exists. Do not import familiar Mermaid gitGraph keywords such as commit/branch/checkout as aliases.

Source-derived fresh proposed merge fixture (not newly run here):

~~~plantuml
@startgit
* merge4
|\
| * side3
* | main2
|/
* base1
@endgit
~~~

Expected parent graph must be verified in a bounded original probe before it becomes an oracle contract: merge4 has main2+side3 parents, each reaches base1. Source-derived local test can already require faithful parsed cells and rejection of ambiguous/unimplemented crossing patterns.

Bound grid work by actual row bytes/cells, not width×height padded rectangle: one long line plus many short rows can inflate padding quadratically. A connector traversal tracks visited(row,col,direction) states, charges every transition and must end at a commit or explicit error. Build reverse indices once; no unbounded retry/recursive path tracing. Bound total trace states by work policy, commits/parent edges by model quota, path vertices by scene quota, labels by text quota.

Tests: should_preserve_git_ascii_columns; should_resolve_linear_git_ancestors; should_preserve_two_parent_git_merge (after oracle); should_keep_git_parenthesized_comment; should_reject_dangling_git_connector_and_duplicate_commit; should_reject_git_command_language; should_bound_git_grid_and_connector_walks. Geometry asserts newest→oldest placement, unique visible commit symbols, one label per identity and all parent edges, with no invisible edge collapse.

## 5. Wire: hierarchical cursor blocks and real wire/bus semantics

Verified original wire-components-link.puml:

~~~plantuml
@startwire
* Input [100x50]
---
* Output [100x50]
Input -> Output : signal
@endwire
~~~

Original viewBox261×101. --- is a new-column command; -> is a horizontal wire. It is not a generic dashed relation. Source WOrientation.java:116–120 selects VERTICAL when style contains -- or == and HORIZONTAL otherwise. WLinkType.java:116–131 selects NORMAL for '-' or BUS for '=', with15/25 spacing. WArrowDirection.java:115–125 preserves no/forward/reverse/both arrows.

Exact anchors:

- p/wire/WireDiagramFactory.java:129–138 lists component, spot, goto, move, wire, new column, print.
- CommandComponent.java:137–155 indent + * name + optional[width x/* height] + color; 166–185 parser.
- CommandNewColumn.java:133–150 indent + dash-run; CommandGoto.java:133–162 nonnegative absolute cursor(x,y); CommandMove.java:133–162 signed relative cursor offsets.
- CommandWLink.java:141–165 dotted endpoint paths, optional source(x,y) with absolute/%/%±offset coordinates, style/color/label; 179–205 typed route dispatch.
- WireDiagram.java:174–181 nesting/columns; 211–214 converts every four spaces to one tab, then counts remaining indentation; 216–247 distinct h/v links whose destination is destination block's x=0 or y=0 boundary.
- WBlock.java:155–195 absolute recursive coordinates/% conversion; 218–228 dotted scoped lookup; 234–258 column/goto/move; 272–307 deferred next-position calculation/last-child nesting; 310–316 underscores become spaces in painted names; 342–362 natural/forced dimensions (unsized leaf100×100); 365–388 sequential source attachment tracks.
- CommandSpot.java:137–181 and CommandPrint.java:133–161 describe optional future annotation support; these cannot be ignored on admission.

First increment: ordered sized/unsized blocks, real hierarchy based on preserved indentation, local names/dotted identities, columns, normal horizontal and vertical links, labels and arrow direction. Model WireDocument { blocks, placements, links }, Block { parent,id,size }, PlacementOp(Column/Block/Move/Goto/Print if admitted), WireLink { source_anchor,destination,orientation,type,direction,label }. Layout interprets bounded local cursor operations into immutable positions, then resolves global anchors. Keep family-specific placement; Dagre would violate explicit dimensions/cursor semantics.

A useful stage boundary can admit flat blocks/columns first, but must reject indentation rather than flatten children. Next add nesting and explicit anchor expressions; next bus ribbon geometry, spots, prints and cursor moves. '=' is a bus with a distinct ribbon/arrow geometry, not a double-thickness line or dash style. Do not claim BUS until drawn. Preserve original one-axis destination semantics (target x for horizontal, y for vertical), which can produce a line crossing the target's overall y/x extent; either document this fidelity behavior or reject out-of-extent attachment instead of silently inventing orthogonal target routing.

Bounds: validate sizes before allocation/measurement (recommended1..100,000 logical units each;0 is only the deliberate auto-size sentinel); final coordinates still ≤1,000,000. Checked nesting and cumulative placement offsets; precompute natural sizes bottom-up once to avoid repeated descendant scans; resolve dotted paths through bounded indexed scoped identities. Bound per-block outgoing attachment count and source-anchor % expressions; keep symbolic PercentPlusOffset until size is known. Negative move supported only after final min/max translation is correct; current Scene expands max bounds but does not translate negative coordinates automatically.

Tests: should_keep_wire_column_and_indent_ownership; should_size_wire_blocks_and_autosize_parent; should_preserve_wire_dotted_identity; should_distinguish_wire_orientation_from_direction; should_draw_wire_signal_label_and_both_heads; should_keep_repeated_wire_source_attachment_order; should_reject_unimplemented_wire_bus_spot_print_and_move; should_reject_wire_missing_target_and_duplicate_sibling; should_bound_wire_dimensions_depth_and_coordinates. Assert hierarchy, all labels, exact forced sizes, source anchor order, column separation and effective orientation. Forced parent size may be smaller than its contents upstream; do not assert unconditional containment without a stated native admission rule.

## 6. BPM: its own branch/event semantics, not full BPMN

Verified original bpm-events-branch.puml:

~~~plantuml
@startbpm
:Receive;
new branch
:Check;
else
:Retry;
end branch
:Archive;
@endbpm
~~~

Original viewBox527×153. Required topology: implicit start→Receive→split; Check and Retry are separate branches; both reach a merge before Archive. This legacy BPM format is not generic activity syntax or complete BPMN notation.

Exact anchors:

- p/bpm/BpmDiagramFactory.java:128–136 has exactly seven command kinds, without common-command registration.
- CommandDockedEvent.java:133–153 :label; creates DOCKED_EVENT.
- CommandNewBranch.java:133–150 new branch; CommandElseBranch.java:133–149 else; CommandEndBranch.java:133–150 end branch.
- CommandMerge.java:136–153 ID:?<+> creates named MERGE; CommandGoto.java:133–150 goto ID; CommandResume.java:133–150 resume ID.
- BpmDiagram.java:139–150 implicit start/model; 162–165 events; 170–218 event-to-grid interpreter; 221–228 sequential edges; 232–251 branch lowering; 128–137 repeated grid cleaners.
- BpmBranch.java:125–146 generates separate branch entry/exit identities and Resume/Goto events.
- BpmElementType.java:111–114 distinguishes START/END/MERGE/DOCKED_EVENT, but END has no factory command here; do not invent its syntax.
- BpmElement.java:225–236 draws actual start circle, diamond merge and plain event box, with directional connectors200–214.

First increment: colon events, implicit start, balanced nested new branch / else / end branch, multiple else arms only when the native branch model supports them, real split/merge topology and continuation. BpmModel stores Event/Branch with ordered arms plus explicit FlowNodeKind Start/Event/Merge, preserving separate IDs. A dedicated grid/row layout or branch compositor may share low-level box/port primitives with activity; it must not disguise syntax as a Mermaid activity graph.

The source lowers first else by inserting an exit merge and resuming branch entry; subsequent else sends the completed arm to the exit and resumes entry (BpmDiagram232–251). Last arm goes to exit at end branch. Native model should represent those edges directly, avoiding synthetic user-visible aliases. An end branch without else would refer to an exit not yet added upstream; initial native admission should reject it as malformed/subset unsupported, not crash or silently create an unverified default arm.

Named merge/goto/resume are a separate useful increment: goto both connects the current tail to named merge and moves cursor to a new row; resume relocates cursor without adding that connection (BpmDiagram181–207). They are not interchangeable and cannot be dropped. Resolve IDs before layout; reject missing/redefined destinations explicitly.

Bounds: depth for branch stack, count every branch boundary/synthetic merge/edge under item budget; avoid clone-of-whole-arm expansion; build sparse rows/grid cells. Grid cleaning loops must have decreasing proof or explicit per-pass/work bounds; do not copy while(true) cleanup termination assumptions. Initial native layout can avoid the Java cleaner algorithm altogether with deterministic bounded branch composition; document geometry differences.

Tests: should_preserve_bpm_implicit_start_and_event_order; should_split_and_merge_bpm_branches; should_keep_nested_bpm_branch_identity; should_continue_bpm_after_merge; should_distinguish_bpm_goto_and_resume (only when admitted); should_reject_bpm_else_or_end_without_branch; should_reject_bpm_unclosed_or_single_arm_branch; should_reject_modern_activity_commands_inside_bpm; should_bound_bpm_synthetic_nodes_and_grid_work. Assert actual indegrees/outdegrees, no edge Check→Retry, both branch tails→merge, merge→Archive, distinct start/merge shapes and label containment.

## 7. Flow: directional tiles and explicit link ports

Verified original flow-directed-tiles.puml:

~~~plantuml
@startflow
Read "Read"
s Check "Check"
e Store "Store"
@endflow
~~~

Original viewBox401×141. Read→Check is southward, Check→Store eastward. This is the actual flow family, not modern activity or a Mermaid flowchart alias.

Exact anchors:

- p/flowdiagram/FlowDiagramFactory.java:124–137 registers two commands only.
- CommandLineSimple.java:138–147 accepts optional[nsew] + ID + quoted label;159–169 defaults omitted direction to SOUTH.
- CommandLink.java:138–145 accepts optional[nsew] + existing ID without label;156–165 adds existing-target link.
- FlowDiagram.java:167–179 creates new tile adjacent to last tile and advances lastTile;182–193 linkSimple adds a link from last tile without advancing lastTile;196–225 draws boxes and directed endpoint circles;238–260 intersects each side by geometry.
- ActivityBox.java:132–146 font/margins/shape and162–181 rounded colored box with measured label.
- p/golem/TilesField.java:136–154 root/new tile/path construction;156–171 permitted paths;174–205 adjacency;223–245 collision shifting east;249–256 occupancy. This is authorized PlantUML-specific geometry, not excluded Smetana. Translate only selected rules, or independently implement a bounded equivalent with traced semantic differences.

First increment: first named labeled tile, subsequent default-south/n/s/e/w new tiles, unique IDs, implicit connecting paths with actual side ports; explicit links to existing IDs with preserved direction and no cursor advance. Model FlowTiles { tiles, placements, paths }, Cardinal enum, Endpoint { tile,side }. Preserve source order and actual directed edge topology. Initial path admission can restrict explicit links to adjacent opposing sides plus same-row west→east routes accepted by TilesField156–171; reject other routes instead of silently invoking a generic DAG layout.

The original draws a red line and a filled small destination circle (FlowDiagram215–225), not an ordinary triangular arrowhead. Native output should reproduce that endpoint marker or declare an intentional symbol deviation. Direction is still semantic ownership of source/destination ports. root direction is ignored by original first tile; retain as metadata/diagnostic or enforce documented root grammar.

Collision handling is substantive: getFreePosition shifts an affected southeast region east until slot free. Initial native subset may reject a requested occupied position explicitly; a later bounded implementation can preserve this rule. Do not flatten all directions into one row. Long measured labels can exceed upstream fixed slots; native measured cell dimensions should prevent overlap while preserving ordering/cardinal placement, with explicit geometry-fidelity note.

Tests: should_preserve_flow_cardinal_placement; should_add_flow_implicit_edges; should_keep_flow_last_tile_after_explicit_link; should_keep_flow_endpoint_sides_and_circle_marker; should_reject_flow_unknown_id_and_duplicate_tile; should_reject_unroutable_or_unimplemented_flow_collision; should_bound_flow_grid_coordinates_and_shift_work. For explicit link state, source-derived proposed case is A "A"; e B "B"; w A; s C "C": the last creation C is south of B, not A. Verify full source in a fresh bounded original probe before an oracle claim. Bounds: sparse occupied-cell map; checked coordinates; account each shifted tile/pass under work quota; never allocate a width×height rectangle based on distant cells.

## Standalone Creole, Ditaa and formula boundaries

### Creole is a real document family, but not present native support

Registered wrapper @startcreole comes from DiagramType181–182 and PSystemBuilder235. p/klimt/creole/legacy/PSystemCreoleFactory.java:123–132 accumulates lines; PSystemCreole.java:148–161 creates FULL Creole sheet with serif14 and no line wrapping. It is more than one label.

CreoleParser.java:153–187 chooses raw/code, table, tree, Latex and simple stripes;215–255 also admits embedded diagrams upstream. CommandCreoleBuilder.java:149–194 registers bold/italic/underline/strike/wave/colors/sizes/super/subscript, image/QR/icon/emoji/math/latex/sprite/font/URL/SVG-text commands. Only selected safe syntax should be implemented; file presence is not a license to invoke rich resource handlers.

First worthwhile increment is a styled in-memory document with explicit lines/blank paragraphs, nested bold/italic/underline and measured runs; code blocks after a real CodeBlock AST exists. FontStyle.java:119–141,281–295 establishes **bold**, //italic//, __underline__, --strike--, ~~wave~~. CommandCreoleStyle.java:175–190 recursively analyzes a styled subspan and restores prior configuration. Ordinary labels in other families can share TextRun and bounded style stack once that is implemented, but standalone document family needs paragraph/line composition.

Current Scene Decoration has only Plain/Underlined/Italic and Context measurement style keys match that narrow set; adding bold/combinations requires combined style enum/struct and cache keys, plus serialization, not only hiding delimiters. A legitimate first subset must reject supported-looking unimplemented table/list/tree/image/math/link/font/HTML commands and unknown markup, not render literal delimiters and call them parsed rich text.

Proposed source-derived positive (not original-run here):
~~~plantuml
@startcreole
**Invoice** //ready//
__Paid__

Plain detail
@endcreole
~~~
Tests should assert styled runs, line and blank-paragraph height, measured style changes/caches, nested-style restore and escaped delimiters, plus rejection of <img:...>, <$sprite>, <math>...</math>, links, <style>, embedded diagram markers and table/list syntax until each is implemented. Bound markup depth, runs and aggregate measured bytes/lines; no raw HTML/SVG copying or resource resolution.

### Ditaa has no confirmed functioning MIT factory dispatch; no native claim

p/ditaa/PSystemDitaaFactory.java:131–157 selects corner/shadow/separation/transparency/scale flags;161–194 has a legacy ditaa declaration path. PSystemDitaa.java:117–120 imports org.stathissideris.ascii2image.*,171–190 constructs TextGrid/Diagram/BitmapRenderer and exports PNG even outside the ATXT branch. This is an external raster implementation boundary, not a native vector algorithm.

Both normal wrapper recognition and factory registration are absent from the examined MIT dispatch as explained above. A future independent ASCII-grid/vector feature would need explicit scope, separately established original oracle and fresh native design/license review, not the assertion that these wrappers implement Ditaa. Keep @startditaa and legacy ditaa command explicit unsupported now. Do not inspect/copy ascii2image implementation or assets on the strength of the PlantUML header. No Ditaa raster/image is proposed as SVG fallback.

### Math and Latex factories exist, but oracle typesetting failed

p/math/PSystemMathFactory.java:119–132 and PSystemLatexFactory.java:119–132 select real @startmath/@startlatex. PSystemMath.java:145–148 and PSystemLatex.java:145–148 delegate equation generation;151–188 accept color/background/scale/dpi and otherwise replace stored formula with last ordinary line, not a multiline concatenate operation.

ScientificEquationSafe.java:145–162 wraps construction;167–183 catches rendering failure;211–213 returns raw monospaced formula;215–224 exports SVG/image. LatexBuilder.java:140–153 delegates to TeXIconBuilder. TeXIconBuilder.java:126–140 reflects org.scilab.forge.jlatexmath.TeXFormula, an external implementation dependency; no math-engine implementation was consulted or authorized by this boundary research.

Existing math-equation.puml and latex-fraction.puml are original-degraded-formula-fallback, because that exact MIT jar lacks TeXFormula. Valid SVG/raw expression text is not equation rendering or successful math support. Native first useful increment requires a separately reviewed native math grammar/layout/font+glyph resource solution with superscript/fraction/radical semantics and genuine math geometry. Measurement of a raw expression, trivial superscript replacement, Merman without_math_renderer, JVM/JS, copying ASCIIMath conversion tables, or an embedded image cannot satisfy that gate. Current recommendation is named unsupported formula capability with zero success/fallback claims. No optional dependency installed, no formula assets imported.

## Unified runtime/resource contract for any later implementation

These are future acceptance requirements, not newly passing implementation claims.

1. Preserve current effective defaults and caller lower limits: source256KiB, SVG16MiB, model10,000, nesting64, work5,000,000, cooperative10s; viewer hard10s/2GiB isolation/cancellation/reaping. Validate limits before scan.
2. Charge actual token scans/lookup/parse states before allocation. Token text is a bounded view/span where possible; no string-amplification copies. Count synthetic branch/railroad/path nodes and labels as items. Bound individual labels16KiB and aggregate measured lines4096 as current API.
3. New ASTs, nested groups/rules/doc styles/hierarchies call depth admission before descending; use arena/iterative traversal when practical. No AST cloning for repeat counts, rule references, graph ancestors, chart point grids or nested size recomputation.
4. Charge every layout traversal, branch/port, grid shift, chart tick, Git trace and route vertex. Precharge cross-products only with checked arithmetic. Check cancellation before/after measurement and periodically during every potentially long loop.
5. Treat geometry as finite measured data. Deny negative sizes/radii and coordinates beyond current1,000,000 bound; translate negative-origin layouts deliberately, then include labels/routes/markers in final viewBox. Prevent scene-shape count growth independently of element count; a single enormous line/polyline must not bypass an element-only quota.
6. Bounded safe writer checks output growth before append/allocation, XML-escapes every label and preserves stable semantic IDs/owners/targets. Final Merman resvg-safe resource admission and engine SVG validator remain mandatory; usvg parses representative output.
7. No engine runtime filesystem/network/environment/date/process/JVM/JS/dot access; no wrapper names as filenames; includes/themes/macros/functions/resource directives and unimplemented styles/pages are explicit located errors. Resource-looking literal text is text only where the admitted family grammar actually says literal.
8. Every admitted family gets exact fixture bytes, positive and unsupported mixed-syntax tests, model topology tests, finite shape/label containment checks, native/fallback measurement evidence and background standalone viewer capture after backend acceptance. Reuse common worker; no new UI/app state path.
9. Full source/legal manifest entries include exact SHA, actual translated behavior and Rust destinations. Do not pre-authorize all92 for wholesale translation: several are boundary inspection only. No Smetana/external renderer or asset transitive import.
10. Before a completion claim, run appropriate focused tests and full workspace tests/clippy/build/deny/audit under controller target-dir/build sequencing. This research task ran no product tests/build because it changes no product.

## Recommended implementation ordering and review units

First unlock family-preserving wrapper/body preparation with regressions for wire indent, Git columns and EBNF quotes. Introduce the shared native railroad compositor with a typed EBNF subset, then regex on separate AST/tokenizer; review topology and text bounds before adding assertions/comments/quantifier edge cases. Next chart categorical bar+line semantics, ASCII Git graph tracing, wire placement/attachments, BPM structured branches, then flow tiles/links; each should be an independent semantic increment over the existing common spine. Standalone Creole is shared text work plus a real document model. Ditaa/math remain explicit boundary backlog until the prerequisites above are resolved.

No family is implemented by this report. It supplies source evidence and concrete acceptance contracts so future implementation can claim only what it actually renders.

## Exact inspected source manifest

All paths below are relative to p/. Every row byte-matched the exact source-JAR entry. Notice codes:
A = generated MIT distribution copyright2009–2024 Arnaud Roques at line8, MIT grant lines19–39; older retained copyright/authorship remains present.
D = same generated grant and copyright, additionally retained Original Author: David Fyfe at line106 (all inspected chart files).
R = retained Original Author: Arnaud Roques at line106 where present.
The per-row retained notice text appears after the hash so older2009–2020 vs2009–2024 headers are not obscured. These edition-prefixed generated files retain older GPL commentary beneath the MIT grant; this report preserves that fact and never substitutes a different edition. Preserve the distribution MIT grant and every relevant author/contribution notice in later module/provenance/third-party-notice updates.

| Authorized relative source path | SHA256 | Retained notice anchors |
|---|---|---|
| `PSystemBuilder.java` | `00383dd63082f93e471900cb70f44daee20899a862e4c3590fd95660b5d6e679` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `core/DiagramType.java` | `590d9685e213d5ad51bf6113239081c7a55973715a306f6f16a2a78480659483` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `regexdiagram/PSystemRegexFactory.java` | `e85b6d64f2956155dbbc22a633fed56b2865599e53dc4118bbebac051ee22ef6` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `regexdiagram/CommandRegexfSingleLine.java` | `b81d7c9709c77b0f8893dd0a84d92f620b8c27050a645fc56dfe894e55af2214` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `regexdiagram/PSystemRegex.java` | `e7932fd6e9c4dff1b4d79dc68cb2d2ad0a6a7c6bc7ef1c87f94a012a0184aea2` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `regexdiagram/RegexExpression.java` | `1431e40cbb99422d854f6f90b13a75007317f11c5706fe2ee25f95b60a9de786` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `regexdiagram/ReTokenType.java` | `0426d5062d0a24135f2a8b81d8e3337a8daf1bcb9a75b61a6aeafbafaae8bb98` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `regexdiagram/ShuntingYard.java` | `4f9f8a3fb1b86c213f85eb4839b13705a0d359122018fd09311031b809ce8a0f` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `regexdiagram/GroupSplitter.java` | `676850d3f107cfe4773de9a861ac85134365197a4b4e8f6c08ffb1a7f3b1d49d` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/PSystemEbnfFactory.java` | `9ebfbab372707a2034cc4a7bf715cc76b8fe9321804aca3697f30b2d1b0f969a` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/CommandEBnfSingleLine.java` | `b538b2d766276b45a6487e3b03a1cd1e2e06313708fbf756b255f7f6e50f77bb` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/UBrexCommandEbnfMultilines.java` | `6fb503691539faab6b8ac95abd00b5c5541d597408ed811a808906481b5945ef` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/PSystemEbnf.java` | `d2b72d188d4295cf7ddaa7f167af58ff4dcff2f1db44e84575d1681eb05f051b` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/EbnfExpression.java` | `a6fda59a2f91452f35ef8b7141b477e1734f19713c1bfde7366b4893a3f2d3cb` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/EbnfEngine.java` | `c2a3ed7de4c511fb03157edc3a1cfdd3c0d4e4558ce20cd838714541c5b8511c` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/Symbol.java` | `eaaa1dfac44afb0967200093a5ff3433fa36b7ab2a528fe7984b3be7dede42b0` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/ShuntingYard.java` | `34879416adc68ed181a5b084d409d2005782af1c17466ff2681875c2e590988b` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/ETile.java` | `f7704c5a2ef397a006ca020bb0b1a59fe4da7f789e9c17d36bc19c42fce70c32` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/ETileBox.java` | `0f5a244ad3168e4ad66bc7583d525fe6e8d898e18463d0b09abc8b2749bfe396` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/ETileConcatenation.java` | `86221aedae1266bfeedc5bb86ef68bda2791a33301fb80587408245164f3e1f0` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/ETileAlternation.java` | `046948c86e6087de66952bbe2bd619785feb084fff7dadd04fa6ac55a09d5da4` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/ETileZeroOrMore.java` | `5fd9ec99f47f940b83d6bf44e78a807b569424701a83034203bb2d712b469716` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/ETileOneOrMore.java` | `2ce22e328b93e285657d7da91cf1e1d9ccf653eb872932c0ae2063ce05217643` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/ETileOptional2.java` | `307b35a377c0f8b58963b30af794c3229f1b3094294b554004c4a5bd1af3d3ca` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/ETileRegexGroup.java` | `cc6f3ecfcd928a72bd2215fb867582efbdc74d6e607311ebc7945acff5279ae9` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ebnf/ETileRegexGroupAllBut.java` | `6c6e63048894a2268b6f7ac5f3aecb345887730bc113cca8c415052d6d0d67bf` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2020, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `chart/ChartDiagramFactory.java` | `4a933c75ea54507d58bf188f09bb87324b7aeb015e9f78a0b715a5845e01d713` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/ChartDiagram.java` | `350a0c82096eec2b930aa7e87a2db047e02debc89c8302b1ce138a50242d8aad` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/ChartAxis.java` | `201a78a557be46d111c59b9afe3feb8974e56a5448e1ddb03fbd96cee9fc8bfe` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/ChartSeries.java` | `ad982eb5eb3a11579d1e4f5f4da41d6f93baa9c8865a2ce26bf1f8db3bdc3da6` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/ChartRenderer.java` | `6e21f772064be5d8691951a9536364fbfe637a7920f62634c2455da5be5786bd` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/BarRenderer.java` | `e29486f130f0550fbc5127e852dbf82627042568699c00ac6b6930e151b18a5a` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/LineRenderer.java` | `4437d2eae71b60f582eb4f50b68227b0031c4c35e21fa8c120c836cb0a391dfd` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/command/CommandChartHAxis.java` | `6b0ecc3899fdf9020949e7eb34cbe55bfccc3df4bfeac46cf123e922cb038543` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/command/CommandChartVAxis.java` | `f43158d9c76bcdf8019993a7fe1ff5c6855dc77e3bba1c9056943df6ff4a7873` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/command/CommandChartBar.java` | `fd21bec8a3893014a9319b787f1288c8fdda94d26b3965df757558206b82b265` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/command/CommandChartLine.java` | `c823a70e90771ec35ad497c7b66444e8b0b10bf8ae36dc5508c24a25c18c3cac` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/command/CommandChartOrientation.java` | `04a50d3d6740dd49589f2de7dfaec244f08af9c389f2ef120a4310652739a72b` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/command/CommandChartStackMode.java` | `13388fb27917cebd37e6c39134d1fe8d7f0438662d9913e14c0c8dadabdbc01c` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `chart/command/CommandChartLegend.java` | `db002c2ba1ff39a8cecac9005b560e88e34507a42b5f8d93e62271000446452c` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  David Fyfe |
| `gitlog/GitDiagramFactory.java` | `0ae469585329d8152981dc21a622cb9ebccab4a840240416232153f571ce3b43` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `gitlog/GitDiagram.java` | `a410be41c76bb04947a399e930f9ee65d111cb0d87469e6abad8bee3018626c3` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `gitlog/GitTextArea.java` | `60ee3dac8f69d8696be70440437afea22e6c2a9b6d66545c8745c5d4aa8f096c` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `gitlog/CursorPosition.java` | `0537f4a34ad1cdf29e9771ec5e59acdafcf752f0a77edc15fe7ac40a3bdf3a6e` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `gitlog/Commit.java` | `12edc7a814662c61c59fb367ec47b0aba7f4666556fa2e406a0ab6708974fed0` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/WireDiagramFactory.java` | `6f6c3863457e2212b3f457436dfa4969e0b8f286b3ab1044d64d9b5b22e5b221` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/CommandComponent.java` | `67c7056419eba96d859bdb1dc4de9e394988b98baf68c42b07530b849dc55664` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/CommandWLink.java` | `1eee3fcb43d34880b4af071614b639b274ae12267bda9adcd4b81f6197ceac2a` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/CommandNewColumn.java` | `65f29ca03c2cab968266045c034fdcf21fd8f3799cc6e22003786f120b6ecdad` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/CommandMove.java` | `2ed90a15f1bc63e60e07976cd99e191bc2438ea55337cdb7e4a1ecc7fb0fbf59` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/CommandGoto.java` | `c65bc8eefa1d5132c35d5c34ade5cce73055c45c7cf4135aa3dc1f626e9cb89c` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/CommandSpot.java` | `d36d99d65e0366513440e1c2dab25bae347bd6e5718012ed02a9eea8f8a72f53` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/CommandPrint.java` | `fdbccbb217cb8a88576683411e0f682fd1dd05c4651009e44f5526ae11fbbf8e` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/WireDiagram.java` | `b8c935c1ea3dc0c0d06f3ee27544cc7d29214780239bb0b1e0978a3369e70b08` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/WBlock.java` | `05acd575509ca1c5f24edce3cc91c56ffbce800233b78252ee9807dbba7736eb` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/WOrientation.java` | `1031a268abd369e0c1288bc5ecb3128a17a126ee6ce90e9ff459a33728f7eafe` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/WLinkType.java` | `28c14c0ea8bb3b0d8593af40cb049ab391165dce8d68a8d3261ea23d030b6687` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `wire/WArrowDirection.java` | `e5e31c8ffc6de241a620a756ea403425697d28f18e16bc423529d128bed3d2c8` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/BpmDiagramFactory.java` | `ad140b9a4b8218dda1bdda58eecc25c0924cba55a0f7ff2696b885f64db013c7` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/CommandDockedEvent.java` | `f293927559ea6d744b3ccb1d7a919b6f21f3fc57f63a84e2fd4dafa3a465d179` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/CommandNewBranch.java` | `e3028cffae75372b7c5c83506d7a590ee03cf2e472c2643ac03a4ec787a53ee8` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/CommandElseBranch.java` | `06f0bf6a9c23c1de3a488c6528ed17792c080462b063b88e52f785f872993edb` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/CommandEndBranch.java` | `736be0e130edea437af81ac71b4be45303558a570c37b2b34bf6b2b795552e6d` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/CommandMerge.java` | `2423f48dfb47064589c6a6e8d8b9c84ca1134202bacb3994ded42e23216db68c` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/CommandGoto.java` | `c8a2906008697866bc980cf39b4c255074533925bd0c5ec5e6fbcb87c0380677` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/CommandResume.java` | `7a9bb675c92146507f74bd42e2e1eb0a97cbd820c25fad52e202032524f2bd34` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/BpmDiagram.java` | `3140cd11a82a4706b6592691865eb0592a45181ca395bae1d706b638e0e3cbb8` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/BpmBranch.java` | `95639259a339b4075417142abe4eb7ab78762977c3fc99579bc75286f762d52f` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/BpmElement.java` | `8ddc900fe9d7b713534397642850a09aaa0227f4affc81a61207cbd3e70c0054` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `bpm/BpmElementType.java` | `1bfd5869d152126ed10b3018f44bde2dc204cbac246214cc46be751f67afce17` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `flowdiagram/FlowDiagramFactory.java` | `76fdfd19aabc35d48e40ce4c0428741ab3d23ca56b304fdbad9ae00304cf3dae` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `flowdiagram/CommandLineSimple.java` | `36d4aaceb2d1a48d4bc73a791b3bcc71a4d843658bbb7fb139c3756a22105f26` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `flowdiagram/CommandLink.java` | `b5d82ad1f79152dec546a6f55ad042439fc29e8a66d6acadea0e65fb2809b4ee` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `flowdiagram/FlowDiagram.java` | `d133a9306d8003e6ddf56c35277d39e4ca5ed210011be0562f17a6d204f564e6` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `flowdiagram/ActivityBox.java` | `00d858c58407a72060d73f52d7986f6a5cfc5c68eb7ff40368d2a3f803f591e0` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `golem/TilesField.java` | `fa5809564779df62b98091cef5eb6200cf00efb84dc85295384fe9517759a438` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `klimt/creole/legacy/PSystemCreoleFactory.java` | `3598121fb8c91ba6f3e923411b8d347157297ce417ccb94e801585a635b8291b` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `klimt/creole/legacy/PSystemCreole.java` | `097d09be780a10cb0164d99f209a4c85d01dcbf91d5d2fa0ca53ff3d93aa9d01` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `klimt/creole/legacy/CreoleParser.java` | `0e8c0ab639019e9ea0850e2e0803b0378ee2a972e7e79a84a6910ff57360a8d7` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `klimt/creole/legacy/CommandCreoleBuilder.java` | `236a6b30a4c3b65cbb79ffb2b3d0fa8f3509407ed127a08ec79abf20b88cf8e0` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `klimt/creole/command/CommandCreoleStyle.java` | `014c7f897d8b192767933be8fddc333c7ed96141b64f1b5633a70f13dd6c6da9` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `klimt/creole/Parser.java` | `8d94530dc1f0d2d37301e30e102559736ded1cca817c4ae927b87a133d7c6757` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ditaa/PSystemDitaaFactory.java` | `50a08c3830d0909e28d28f3790ff6c5430e482e52b9a0ed66856ac0f19d1315b` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `ditaa/PSystemDitaa.java` | `23b08cf77de0f967451d02de07787c5fbde8eff0740f3ab626cf5137b28f3c2c` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `math/PSystemMathFactory.java` | `52ad424715231f0db2bd0448dbe3261759b6535929dd59db3f967bc112ea473c` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `math/PSystemLatexFactory.java` | `493c8ade97a688cef608fd492462fc4a9d777df0d55c19e1cbb885877ff22a3a` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `math/PSystemMath.java` | `e9cd5f272d6f228465905a1f0c3a7e0cd495e9ec5628ffb2bd6fa6781939873b` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `math/PSystemLatex.java` | `69315194397f3cbe9fa0c8a5557ef7a3d1c8758c19cad9ef482c238d03451fc1` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `math/ScientificEquationSafe.java` | `d4e9d4f8113cab3480951a8cbc38ffa4dda02d71017b4e58489c88fcf1ac7985` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `math/LatexBuilder.java` | `f4d0bea3c150e489b051335d2f06e18ed07099c968b4375bbbc2a97d60ac5b4f` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `math/TeXIconBuilder.java` | `300f001bc2a5969fd6854db95ec4046363f27933a0a63f86e0edbce1931991a2` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |
| `klimt/font/FontStyle.java` | `7b6fb79f8a28e95cb42d64bd6d3c73e92f6a6d665e895992cf04b1d65093da28` | 8: (C) Copyright 2009-2024, Arnaud Roques; 79: (C) Copyright 2009-2024, Arnaud Roques; 106: Original Author:  Arnaud Roques |

## Existing oracle bytes independently rehashed during this research

These seven .puml and .svg files were read only. Values identify actual existing artifact bytes, not new original/native outputs.

| Fixture stem | Source SHA256 | Original SVG SHA256 |
|---|---|---|
| `regex-branch-repeat` | `68b258c9db493a896f5f248717cfe1bc21e233cfffffcbbb3d4c52a6e77a9370` | `1aa30fbed6f06a7cc8d67e4d200bf65b0acab95057aad293fbb1fd199e793b8e` |
| `ebnf-optional-repeat` | `63126d525ce71732630bd0840e94e970eed8c5be946db0c8fbc2c1571f0a197b` | `606bf8a8d613b50bef4072d53d5ec8f70ca30495862ffc957c048619d405777b` |
| `chart-bars` | `5304d4e71659f101b9452b3d25333d2c31c4bad87e7dd4dabd6ae4061bf65630` | `76bd7e49fd0881fe86db5d825cc9f46cb0e030886b0d62d3ab52014caa3ebefe` |
| `git-linear-history` | `03dbabe21fea378aed7f4e54e765a1ebdc9fb1079cc1b787abfacceaac722e3e` | `bfcd27973d4cc060f18e8a0298283dae28364c217f4b3643a8e95cb4bb61cdf4` |
| `wire-components-link` | `d2da7ba763012f397c23facf849424dfcac3b507c108547ad548f23f67bd0bb2` | `cd6c83a6600296774d484c71da66fea0e85e0055f7671960f5b21ea8b84b13d6` |
| `bpm-events-branch` | `d75839e3c0b6f86055f781fb5cf38f8c7dad3b27f40ec3e1ecb9b7ff5ae8c79e` | `8207cac838c4e03cca0f0e5a897f07423b44665704d55bfd9d2370056322809b` |
| `flow-directed-tiles` | `a50ecc4dd253fca2a44b862d94ffdb2c1e11c63ca374e92127ff4f236904a8e8` | `d06bf7b87b5b46dc5137f5ef9383cb34c3f08c8beedc89715b154e2e2ecc8eb9` |

