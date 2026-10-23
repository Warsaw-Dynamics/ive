# PlantUML Task 6: bounded shared styles and literal preprocessing

Prepared 2026-10-23. Read-only preparation, **not implementation or product review**. Native interface baseline: `b9815e81df754985a7bf4c1159d196251a5acb24` in `/mnt/data/Projects/ive/.worktree/diagram-ports`. README, AGENTS, detailed native plan and Task 3A handoff were read. No product files, dependencies, commits or existing oracle artifacts were changed. No subagents or app instances were started.

Only translation/reference source: `/mnt/data/Projects/plantuml-mit/src/net/sourceforge/plantuml/` (abbreviated `p/` below), MIT 1.2026.9. No rejected candidate, other edition, GPL checkout, `gen/`, `h/`, Smetana/Graphviz/ELK implementation or untraced asset was opened/imported. Source artifact SHA-256: `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`, a local artifact check rather than authenticated Maven provenance.

## Recommended first increment

Implement one shared, preamble-only style/chrome stage for the **already admitted families**, paired styled text runs everywhere those families accept display labels, and an honest support ledger. Leave every `!` directive denied in this first increment. Its useful visible result is custom font/size/color, real measured title/header/footer/legend regions, bold/italic/underline/strike runs and explicit line breaks. It does not require implementing TIM, theme loading, general CSS, wrapping, assets, or a new diagram family.

Preamble-only is a deliberate initial restriction: commands below are admitted before the first family statement, outside any note/member/text body. An interleaved style/chrome command fails explicitly. Do not filter matching lines out of arbitrary family bodies. This avoids silently stealing a class member named `title`, a note payload or later a JSON scalar. A family parser still must consume every remaining statement, and a chrome-only document must not become a successful empty diagram.

Split rollout if necessary: (A) typed font/paint plus single-line chrome and skinparams, (B) selected `<style>` rules and paired Creole, (C) multiline chrome after raw block preservation, (D) exact eager immutable constants after source maps. A/B are a meaningful deliverable; do not label them complete Task 6 or full PlantUML style support.

## Exact proposed admission grammar

This grammar is **the proposed native subset**, not a claim of complete upstream grammar. `WS` is one or more ASCII spaces/tabs; `HWS` is zero or more. Keywords, skinparam/property names and finite keyword values are ASCII case-insensitive. Font-family spelling and literal text remain case-sensitive. Whitespace outside a value may be trimmed; whitespace inside a label is retained.

```text
separator := WS | HWS ':' HWS
align     := 'left' | 'center' | 'right'
chrome    := ('title' | 'legend') separator label
           | [align WS] ('header' | 'footer') separator label
label     := bare | '"' raw-content '"'
```

For `bare`, require at least one Unicode Letter/Number, `_` or `.` as the original single-line regex does. Quoted labels may contain punctuation only. Choose an explicit error for an empty/whitespace-only title rather than silently dropping it. Original `CommandTitle:138–163` and `CommandLegend:138–164` strip the enclosing quote pair through regex capture; they do **not** use the identifier lexer's C-like escape decoding. Keep this payload raw until the single label escape stage. Restrict quote delimiters to ASCII `"` initially; original `%g` additionally admits curly/invisible quotes (`Pattern2:124–132`). Header/footer allow a separated alignment only initially; original also admits `leftheader`, etc. (`CommandHeader:139–153`, `CommandFooter:139–151`). Unknown alignment and incomplete single-line commands fail.

Title is centered above the diagram; one-line legend is centered below. Header defaults right, footer center (original default-jar probes below), with explicit alignment respected. Repeated nonempty commands replace their preceding value, including alignment: original `TitledDiagram:257–260,299–315`; these are not several simultaneous left/center/right ribbons. Suggested native initial defaults: existing body palette/font unchanged; title 14px bold, legend 14px regular, header/footer 10px regular `#888888`, consistent with observed original default-jar output. These are chosen baseline values, not imported theme assets or a promise of theme parity.

Only these single-line skinparams initially:

| Native admitted name | Exact semantic target |
| --- | --- |
| `defaultFontName`, `defaultFontColor` | Root font family/color, including chrome |
| `defaultFontSize` | **Element** font size, excluding document chrome |
| `titleFont{Name,Size,Style,Color}` | Document title |
| `legendFont{Name,Size,Style,Color}` | Legend |
| `headerFont{Name,Size,Style,Color}` | Document header |
| `footerFont{Name,Size,Style,Color}` | Document footer |
| `BackgroundColor` | Whole canvas background |

Grammar: `skinparam WS admitted-name WS value`. No generic key acceptance, `skinparamlocked`, stereotype `<<…>>` suffixes or dotted names. `defaultFontStyle` is intentionally **not** listed: the inspected migration table has no general defaultFontStyle mapping. `defaultFontSize` mapping differs from FontName/FontColor (`FromSkinparamToStyle:164–168,226–257,507–511`); do not apply all three to one universal default object. Family-specific node, arrow, note and stroke skinparams can be a later finite table once all corresponding primitives use them.

An optional small block increment may admit `skinparam { name value … }` and `skinparam title|legend|header|footer { FontName|FontSize|FontStyle|FontColor value … }`, braces on separate lines, one declaration per line, no nested blocks. Scope concatenates directly with the property name, e.g. `title` + `FontSize`; original `SkinLoader:125,142–176`. An unknown key anywhere rejects the entire block. Do not claim these blocks are supported if only the single-line stage ships.

Selected inline style sheet (multiline tags; braces and declarations occupy their own lines):

```text
sheet := '<style>' NL rule* '</style>'
rule  := root-rule | element-rule | document-rule
root-rule := 'root' HWS '{' NL font-decl* '}' NL
element-rule := 'element' HWS '{' NL font-decl* '}' NL
document-rule := 'document' HWS '{' NL
                 (background-decl | chrome-rule)* '}' NL
chrome-rule := ('title'|'legend'|'header'|'footer') HWS '{' NL
               font-decl* '}' NL
font-decl := ('FontName'|'FontSize'|'FontStyle'|'FontColor') WS value [';'] NL
background-decl := 'BackGroundColor' WS color [';'] NL
```

Every rule in that subset must affect its selected target. Root matches all roles, element matches only diagram content, document matches canvas and nested document roles. Original title query is `root/document/title`; legend is `root/document/legend/<diagram-type>` (`core/DiagramChromeFactory:435–447`, `EntityImageLegend:122–128`), so document/legend is a valid bounded target. Accept repeated rules/properties with last-authored declaration precedence **per property among matching selectors**, and use one declaration sequence counter across admitted skinparam/style declarations. Do not invent CSS “deeper selector always wins”: without stereotypes/ancestor features, original priority ends in source order (`Specificity:129–151,170–172,203–221`). An original probe confirms a later `root FontColor` overrides an earlier nested title color. If mixed skinparam/style order differs through pending migration, either test it against original before admission or explicitly reject mixed syntax for overlapping targets in the first release; do not silently invent precedence.

Initially reject all other selectors/properties, including otherwise known upstream ones: family selectors, flat `header` ambiguous with class headers, stereotypes, comma lists, `*`, `depth(...)`, `@media`, variables/`var(...)`, `Image`, hyperlink properties, wrapping/`MaximumWidth`, padding/margin, stroke, shadows, imports and inline single-line `<style>…</style>`. Accept no empty rule as a means to swallow unsupported input. Original `RawStyleParser:275–284` silently ignores unknown properties and `188–197,287–305` forgives missing/extra braces; **native must reject both**, with the offending property/selector/value and its source location. Original grammar and the stricter subset must remain separate in documentation.

Values are parsed to closed typed values, never copied into SVG/CSS:

* Color: exactly `#RRGGBB`, case-insensitive hex, normalized on emission. No arbitrary named-color table, alpha, gradients, automatic/dark schemes, URL/functions or `none` initially. Original accepts more, including 1/3/6/8-digit hex and named colors (`HColorSet:152–231`); this is an explicit smaller subset avoiding untraced color data. Additional `#RGB` is straightforward only when deliberately admitted/tested.
* Font size: decimal integer 6–96 pixels; reject signs, fractions, units, overlong digits and overflow before allocation. The range is native policy, not an upstream bound.
* Font name: one family, 1–128 UTF-8 bytes; initially unquoted letters/numbers/spaces/hyphen/underscore, plus `sans-serif`, `serif`, `monospace`. In `<style>`, an enclosing ASCII quote pair is allowed and removed; raw stylesheet quoted strings are copied verbatim without C escapes (`RawStyleParser:312–349`). Reject controls, `<`, `>`, `&`, braces, semicolons, backslashes, commas and path/URL forms. No font file loading. Installed-family fallback is the existing host behavior and must remain honestly reported; custom-font geometry parity is not promised.
* Font style: `plain|normal|bold|italic` only, weight 400/700 and normal/italic axes. Do **not** reinterpret `bold italic` as a supported original keyword: `ValueImpl:273–291` recognizes the finite words and CSS weights, otherwise returns normal. Nested Creole legitimately combines bold and italic. Reject other authored values rather than reproducing original fallback-to-normal behavior.

## Plain styled Creole, escaping and measured runs

Initial display subset: plain Unicode/XML-looking literal text; paired `**bold**`, `//italic//`, `__underline__`, `--strike--`; correctly nested distinct styles; optional paired `<b>…</b>`, `<i>…</i>`, `<u>…</u>`, `<s>…</s>` ASCII case-insensitive aliases. Do not admit legacy unclosed-to-end-of-line tags. No `~~wave~~` until a real wavy decoration primitive is implemented. Original STRIKE is `--`; **`~~` is wave, not strike** (`FontStyle:119–141,281–295`). Underline Creole is enabled only in original FULL mode (`CommandCreoleBuilder:149–165`); initial native labels can consistently advertise FULL plain runs, but this is a documented scope choice.

Algorithm contract: scan once with bounded UTF-8 cursor/stack, resolve escaped characters before recognizing delimiters, enforce properly paired nesting and nonempty spans, accumulate effective font/decorations for literal runs, restore parent style on closing. Charge scan/run/depth work. Reject unmatched/crossed delimiters and unsupported recognized markup rather than stripping it. Original `CommandCreoleStyle:142–155,176–190` recursively parses captured contents and restores preceding configuration; it supports permissive cases the native subset intentionally rejects. Native well-nested-only validation is a restriction, not an exact emulation of all original malformed markup.

Tilde escape is precise: `~` hides the following `_ - " # ] [ * . / <`; removes only that tilde; a tilde before any other character remains literal. `\~` produces a literal tilde in the original CharHidder stage. `~~` is **not** a generic escape for tilde. Preserve an escaped-token bit instead of copying the original private-use-character sentinel implementation; genuine U+E000–U+E0FF user text must not be accidentally “unhidden”. Anchors: `CharHidder:118–164,180–199`, `StripeSimple:213–231,346–380`. Example `~**bold~**` paints literal `**bold**`, no bold.

Decode `\n` into an explicit hard break and `\\` into a literal backslash **once**, before Creole analysis; physical multiline family-note lines are already hard breaks. `\\n` is the literal two-character `\n`, not another newline. Reject `\r`, `\l`, `\t`, trailing/unknown escapes in the new subset until their actual alignment/tab contracts are implemented. Original Display has newline/alignment/tab/backslash paths and preserves unknown backslashes (`Display:337–420`); do not claim generic backslash escape parity. Preserve original label-source spelling or escape tokens across identifier decoding: `lexer.rs:53–72` already decodes some escapes whereas `text.rs:50–54` separately replaces backslash-n. A new common decoder must receive enough information to avoid a second pass over an already decoded literal.

Two literal URLs can accidentally form a real original italic span: original `https://first.test/a and https://second.test/b` paints `first.test/a and https:` italic and consumes both `//` pairs. Current native URL-looking literal contract may be kept as an explicit deviation, but specify it rather than assuming upstream has a `://` exception. To align strict original Creole, ask users to escape `~//` in such labels and add compatibility tests. No network capability follows from ordinary URL text. Explicit `[[…]]`, `<img…>`, `<image…>`, `<$…>`, `<&…>`, emoji/sprites, math/LaTeX, QR, arbitrary `<text …>`, HTML tables, Creole tables/lists/headings/horizontal-rule blocks, and unsupported recognized tags fail. Unknown XML-looking literal text must still be escaped and displayed; a denylist of known syntax is not permission to emit unknown tags raw.

Add inline `<color:#RRGGBB>…</color>` and `<size:N>…</size>` only as a separately tested extension of these same typed runs. Their original grammars are `Splitter:125–127`; changes save/restore font configuration (`CommandCreoleColorChange:159–178`, `CommandCreoleSizeChange:155–167`). Original silently ignores invalid colors; native should fail. Mixed size text needs common line baselines, not one independent y-position per font. Defer `<font:…>` and `<font size=… color=…>` until their overlapping grammar is explicitly handled.

## Native interfaces and layout/paint obligations

Paths below are relative to `diagram-ports/` in the stated worktree, and refer to observed interfaces rather than assumed plan files.

| Current contract | Narrow extension needed |
| --- | --- |
| `source.rs:6–9,11–51`: `Line {number,text}`; `prepare -> Vec<Line>` trims all lines and removes blank lines | Add original byte ranges/columns before admitting source-mapped expansion. Retain raw text block lines separately for multiline chrome; do not recover indentation/blank lines from trimmed strings. |
| `mod.rs:24…`: source, family parse, context/layout, serialize, terminal validation | Prepare shared preamble into `{family_lines, document_chrome, resolved_styles}`; select family from family_lines only. Reuse **one** render session and aggregate budget for body and chrome. |
| `text.rs:11–23,35–97`: Label strings + one line height, cache `(String,Decoration)` | `FontSpec {family,size,weight,slant}`, `TextDecorations {underline,strike}`, `TextRun {text,font,color,decorations,origin}`, `TextLine`, `MeasuredRun {run,advance}`, `MeasuredLine {runs,width,height,baseline}`, `MeasuredLabel {lines,width,height}`. Own optional states use enums, not Option fields. |
| `scene.rs:26–63`: fixed Paint palette, exclusive Plain/Underlined/Italic decoration, Text(Point,Label,Decoration) | Typed RGB color plus fill `None|Color`, typed stroke, and Text(Point,MeasuredLabel). Keep static-member underline, abstract italic and object underline as composable base decorations; Creole adds to them. Never a raw CSS string. |
| `graph.rs:43–107`: all node/member/relation sizes measured before Dagre; `sequence.rs:366–398`: heads/events measured before lanes/rows | Route all labels through role-specific resolved styles and common measured runs. Font changes must reach node dimensions, header/member compartments, endpoint/relation labels, nested captions, sequence lane spacing, notes, frames and row heights. |
| `svg.rs:12–15,105–109`: white canvas/fixed stroke and 14px sans-serif text | Serialize measured per-run font family/size/weight/slant, color and decorations; use accumulated measured advances to place runs. XML-escape attributes/text through the bounded writer. Canvas color comes from typed document style. Preserve semantic IDs and terminal safety validation. |

Library adapter is available now: `vendor/merman-render-0.9.2/src/text/types.rs:18–24` has family/size/weight/style; metrics have only width/height/line_count. `viewer/src/native_text.rs:127–173` resolves font family and applies weight/slant to real GPUI layout. One query per nonempty coalesced effective font run, bounded to the existing aggregate 4096 **measurement requests**, not merely 4096 source lines. Keep 16KiB label cap, finite metrics, cancellation checkpoints, per-render cache lifetime and source/model/SVG/work limits. Cache key includes text and **all font properties** (and full run styling if caching complete painted labels). Color/underline/strike do not change shaping, but must not be lost by a cached painted-label result. Coalesce adjacent equal effective styles; avoid per-character host calls. Each drawn text run is shaped separately, matching its measured segmentation; do not measure one concatenated string then split paint at different shaping boundaries.

Line width is the sum of advances; label width is maximum line width. A shared line baseline and height must contain every run using a documented font-size/4.5 descent approximation plus measured line-box metrics, without claiming native ascent/descent/ink provenance. All font sizes identical in A/B, so a conservative shared line height suffices. For later mixed sizes, reserve top/bottom extents under the same documented baseline rule. No automatic wrapping.

Chrome is a shared composition pass around the completed family scene, not fake family entities/Dagre nodes. Measure all regions first, then recompute width, center/alignment and translations: header → title → top legend → body → bottom legend → footer. First one-line subset only has bottom legend. Account for widest chrome, line heights, margins and every translated primitive; maintain nonnegative finite viewBox and budget charges. Original order is legend around body, title above that, then header/footer (`DiagramChromeFactory:195–206,417–447,474–504`). Native spacing can be chosen and documented rather than imported untraced skin constants. Retain real family ownership metadata after translation.

Multiline follow-up requires grammar-specific handling: `title` … `end[optional one whitespace]title`; `legend [top|bottom] [left|right|center]` … `end[optional one whitespace]legend`; `[align] header|footer` blocks with analogous ends. Bare title/header/footer are block openers, not incomplete inline labels. Original title expands newlines, removes common empty columns and replaces backslash-t (`CommandMultilinesTitle:131–159`); legend smart-trims, removes columns and replaces tabs (`CommandMultilinesLegend:136–156,190–210`); header/footer trim but do not use that same pipeline (`CommandMultilinesHeader:132–134,165–185`; footer analogous). Do not implement one blanket `.trim()`/drop-empty-lines routine. Nested block-looking body lines are text until the exact terminator, but preprocessor/resource requests remain denied before rendering.

## Pure preprocessing: exact findings and guarded follow-up

**Do not implement a recursive `replace(NAME,value)` macro system.** Original has two distinct `!define` paths (`TLineType:132–142,240–247`):

1. `!define NAME text` is `AFFECTATION_DEFINE`: parse variable, skip spaces, eagerly expand the remainder using existing state, store the resulting literal string globally (`EaterAffectationDefine:120–133`). Quotes around `text` remain literal quote characters.
2. `!define NAME(args) …` is a legacy function, evaluated differently and potentially recursively (`EaterLegacyDefine:120–129`, `TFunctionImpl:235–245`). Deny it, `!definelong`, full TIM, all builtins, variables/assignments, conditionals, loops, includes/imports/themes and functions/procedures.

Normal expansion scans input left-to-right, appends a stored value and advances over the original token; it does **not** rescan inserted value text (`TContext:458–520`, `VariableManager:131–155`). Definitions see only prior definitions. Example `!define FIRST SECOND`, then `!define SECOND Ready`, paints `FIRST | SECOND` as `SECOND | Ready`. With prior `BASE Ready`, `NEXT BASE now` eagerly stores `Ready now`. A mutual-looking pair `A B`, `B A` paints `A | B` as `B | B`, not a recursive cycle. A cycle detector must never change this into lazy resolution. Original permits redefinition/shadowing (`TMemoryGlobal:149–156`); immutable native definitions must reject duplicates explicitly as a documented restriction.

Safe optional grammar after source maps: `!define WS NAME WS literal`, where NAME is ASCII `[A-Za-z_][A-Za-z0-9_]{0,63}`, nonempty same-line literal <=16KiB, all definitions occur in the preamble inside the wrapper, immutable per render. Reject parameter lists/empty definitions, continuation/multiline replacement, `%…` builtins, `$` capabilities, `##` concatenation, generated directives/wrappers, and invalid XML/control characters. Plain text is not an evaluated expression. Whether literal references to prior immutable constants are admitted must be explicit; if admitted, use eager expansion exactly as above. Store a frozen value, not a future symbol lookup.

Match names case-sensitively at original word boundaries, **including inside quotes/labels**; quote state does not disable original preprocessing. Original preceding/following word is Unicode Letter, `_`, ASCII digit or UTF-16 surrogate (emoji); `\n` immediately before a name is an exception to the preceding-boundary check (`VariableManager:216–236`, `TLineType:344–365`). ASCII Rust `\b` is insufficient; Unicode `is_alphanumeric` is also not exactly equivalent. Choose a bounded explicitly narrower surrounding-character policy if exact Java Unicode category/surrogate equivalence cannot be guaranteed. `##` removes neighboring concatenation markers (`VariableManager:131–155`), so reject that capability while unavailable. `$X` original expands bare X after a literal `$`; do not accidentally implement a different dollar-variable grammar.

Required origin representation: `SourceSpan {start_byte,end_byte,line,column}` plus bounded segments `Direct(span)|Expansion {use_span,definition_id}`; immutable definitions retain definition span and references to earlier provenance, with a depth counter. Map emitted diagnostics to the authored use span and identify the definition span when relevant. Bound and charge the segment count too: copying an expanding provenance tree can blow up even if final string bytes fit. Public errors may still be strings; engine API/worker protocol need not change.

Apply conservative explicit ceilings before allocation: existing source byte cap for **aggregate retained expanded source plus frozen definitions**, per-label16KiB, definition count charged against model_items, source-map segments against a small checked multiple of expanded bytes/model_items, expansion work against shared Budget, lineage depth `min(32, limits.nesting_depth)`. Checked sums before every append/copy; cancellation inside scans; no budget resets. These tighter limits are native policy. An active-definition stack/depth guard must protect any future recursive extension, but eager frozen constants should never recurse through inserted text. If the desired contract specifically requires rejecting forward/self/mutual name dependency cycles, add that as an **explicit restriction** with diagnostics rather than pretending upstream uses lazy cycles. First-release recommendation remains to defer preprocessing until these tests/source maps exist.

After expansion, run strict source/wrapper/command admission again. Literal generation must never grant capability: an expansion that produces `!include`, another wrapper, a `<style>` import, an image/resource command or newline statement fails. Likewise a prohibited directive hidden in a rejected function/block must still fail, never disappear into a swallowed block. No engine resolver, clock, environment access, Java, subprocess or network is introduced.

## Useful acceptance tests (future implementation)

Use `should_…` with given/when/then; style-sensitive fake measurer should record the exact requested text+family+size+weight+slant and return size/weight-dependent widths, so tests can prove both measurement and paint. Use public render plus SVG/usvg parsing and semantic geometry assertions, not label counts alone.

* `should_measure_and_paint_font_size_family_weight_and_slant`: same visible string at different supported styles has distinct recorded queries, corresponding SVG attributes, larger node/row reservations, no cache collision; color-only change does not corrupt cached paint. Include member/object base underline plus nested bold-italic.
* `should_keep_document_font_size_separate_from_default_element_font_size`: defaultFontSize22 changes body, not title/legend; titleFontSize30 changes title and shifts body; width follows the longest chrome. Header/footer align to actual final width and are contained.
* `should_compose_chrome_around_sequence_and_nested_graph`: real existing sequence and package graph bodies, all ownership/edge IDs retained; no overlaps; every measured label inside viewBox; wide header and two-line title via `\n`.
* `should_apply_only_known_style_rules_and_reject_unknown_nested_properties`: unknown `Bogus`, unsupported `Image`, `actor`, `.tag`, `@media`, comma/star/depth selector, malformed braces and unknown declaration within a supported title block each fail with offending text/location. No partial success.
* `should_preserve_original_plain_selector_source_order`: nested title red then later root blue yields blue. For mixed skinparam/style precedence, record fresh original probes before claiming compatibility or reject the overlapping mix.
* `should_parse_nested_creole_and_escape_delimiters`: `**heavy //slant//**`, `__line__`, `--cut--`, `~**literal~**`, XML metacharacters, apostrophes, genuine private-use text, `\~`, trailing tilde, `~~` wave rejection, mismatched/crossed/unclosed syntax, unsupported links/resources inside styled spans.
* `should_decode_explicit_breaks_once`: raw `\n` breaks; `\\n` stays literal; quoted declaration versus unquoted message versus title obey the same actual escape-origin contract; physical multiline notes preserve blank rows; unsupported alignment/tab escapes fail.
* `should_bound_runs_requests_depth_and_work_before_allocation`: many tiny style changes hit aggregate4096 requests, deep nesting hits configured limit, long family value/invalid size/color rejected, cancellation before/after a run query, overlarge rendered SVG stopped before append. Record honest native/fallback counts.
* Later constants: `should_expand_eagerly_without_rescanning`, `should_keep_quoted_define_payload_quotes`, `should_match_case_sensitive_token_boundaries_and_backslash_n_exception`, `should_reject_immutable_shadowing`, `should_handle_mutual_looking_definitions_without_lazy_recursion` (original B|B or explicit subset rejection), `should_reject_parameterized_self_recursive_macro`, `should_bound_definition_lineage_depth_output_work_and_origin_segments`, `should_map_expanded_error_to_use_and_definition`, `should_reject_generated_directives_and_all_resource_capabilities`.

When implemented, run focused tests then workspace test/clippy/build/deny/audits as controller directs; capture a background standalone viewer with styled sequence and graph examples and verify host queries/visible paint. This research ran **no native tests or product acceptance checks** and claims no native support.

## Fresh original probes (memory only; no corpus overwritten)

Used `/tmp/ive-diagram-oracles.JwN5H5/plantuml-mit-1.2026.9.jar`, SHA-256 rechecked `e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1f1a515f7f70dd`, Java headless, `PLANTUML_SECURITY_PROFILE=SANDBOX`, `-charset UTF-8 -tsvg -pipe`, no CLI font injection, cwd `/tmp`, synchronous child timeout20s/output8MiB. Fourteen independently authored inputs were piped in memory; all exits0, empty stderr. No output artifact was persisted. Checks extracted actual `<text>` attributes/text; they are original-side semantic probes, not geometry/native comparison.

| Probe body (within @startuml/@enduml) | Observed painted behavior |
| --- | --- |
| FIRST SECOND; SECOND Ready definitions, message FIRST\|SECOND | `SECOND | Ready` |
| BASE Ready; NEXT BASE now definitions | `Ready now` |
| TOKEN `"Ready"` definition | Literal `"Ready"` including quotes |
| X Ready; `X X1 aX X_a (X) \nX $X ##X##` | `Ready X1 aX X_a (Ready)` then `Ready $Ready Ready` |
| A B; B A definitions; message A\|B | `B | B` |
| `~**bold~** **heavy //slant//** __line__ --cut-- ~~wave~~` | literal stars; weight700; combined700+italic; underline; line-through; **wavy** underline |
| `**unfinished <b>heavy` | Original accepts permissive unmatched text; native strict rejection is deliberate |
| defaultFontColor#123456, defaultFontSize22, titleTop, legendBottom | Body22, title14bold, legend14; all #123456 |
| headerHead, footerFoot | Header/footer10, #888888; default header right/footer center |
| root FontSize25/font/color; document/title FontSize30 | Body25; title30bold; requested family/color preserved |
| Two unescaped literal https URLs | Original pairs the slashes into an italic middle run |
| title quoted `A \\n B` | Literal backslash-n, no second decoding |
| nested title color then later root color | Later root color wins including title |
| `<b>a <i>b</i> c</b> <size:20>big</size>` | Nested bold+italic, restored bold, size20 run on shared baseline |

## Authorized source anchors and SHA-256

All paths relative to `p/`. Each of the 47 files below was read as needed and/or hashed, and byte-compared with its ZIP entry in the authorized sources jar (all matched). Hashes identify this edition's actual generated MIT bytes, including older header comments; they do not authenticate upstream publication. Line anchors are exact in this extracted tree. A product manifest must record **every** translated additional file, Rust destinations and actual notices; this research table does not itself change the product manifest.

| Path: relevant lines | SHA-256 |
| --- | --- |
| command/CommandTitle.java:138–163 | `98f79ba3e6032bf689c4aa0c481d902c6a6d60721d42a62fe15a4215530b5246` |
| command/CommandMultilinesTitle.java:131–162 | `ac8b4e08356b4906fd48fa869cfb1d1be413b0f20becb1363a764674f4efd11d` |
| command/CommandLegend.java:138–164 | `d7db02ee08f9c736a92b8220cdf47dee993fda09774d6f0b6e6b3c56064dd4bc` |
| command/CommandMultilinesLegend.java:136–156,190–210 | `f07a60b39b6736e4f7ac97538429e185d8000731be66c749ec7794b3235f7d9c` |
| command/CommandHeader.java:139–184 | `f19a73844bc50966b47d2d4f4808e512c497500cae0b42f734b1b3081862122e` |
| command/CommandFooter.java:139–182 | `582704ef20242ba7f79e04ac619dd70a403380e62545343c97e9b8eeb6c028e3` |
| command/CommandMultilinesHeader.java:132–185 | `78fcd7ca3ac98f689baf32f7f107bb9808c077fc7f998bf1caa6ae9ba216dcd6` |
| command/CommandMultilinesFooter.java:132–185 | `459ce56924745d41c3fa58f3d49ed31a33575e6f2a9a2ab2eb5f3eb40f1fd041` |
| command/CommandSkinParam.java:136–142,168–184 | `477ba349a5cc2259c843ddfa31c06344b60f305714b3a6f286d15912064d69ae` |
| command/CommandSkinParamMultilines.java:125–139,170–182 | `99594f8a35280a6c6dd4f6732ac5ca7f893e1597b073b16424c3a00f36b0026c` |
| command/SkinLoader.java:125,142–176 | `2a93407ea8d268cb939d7bf1143e25ebbdd4aeda1ccb8d9ad3ba1e51c8f0a506` |
| command/CommandStyleMultilinesCSS.java:134–145,160–172 | `9462c9bc09b6ac18502196e18c710a65484fb0c59b16a06c5f99fd3b6097b763` |
| TitledDiagram.java:257–260,299–315 | `ac6ef70a87c9e7af5fa69340659b85e6f9cd2459ed3a57f9189a59cad3b7371e` |
| core/DiagramChromeFactory.java:195–206,417–504 | `8c81c5c8e12baabc1902797a5bb6cdcb7a7a79a6da7531782ad04938a40c2b28` |
| activitydiagram3/ftile/EntityImageLegend.java:122–128 | `5a968d84e67ad5e50f6f3fcd9aa3523d73d40f7d8a24f558b6393416cefaa4da` |
| klimt/creole/Display.java:337–420 | `7322037fe78634321b6e2592a8c03a6ff0fc0f0ab2aba422a6db730d16605ef8` |
| klimt/creole/legacy/StripeSimple.java:213–231,346–380 | `6b8894ef3c7e9c10f275a886ac4a4e1716efc2b663c04bb9af35d9cf24ee5ecb` |
| klimt/creole/legacy/CommandCreoleBuilder.java:149–194 | `236a6b30a4c3b65cbb79ffb2b3d0fa8f3509407ed127a08ec79abf20b88cf8e0` |
| klimt/creole/command/CommandCreoleStyle.java:142–155,176–200 | `014c7f897d8b192767933be8fddc333c7ed96141b64f1b5633a70f13dd6c6da9` |
| klimt/creole/command/AddStyle.java:121–131 | `2a9fc94d59f01d33858d8dc86cdca81bbde7f5626d1aecc981fe64742698a867` |
| klimt/creole/command/Splitter.java:123–154 | `7538ae4ac4a6dc5449bc41c2ba691b5891570f1dcf13fc9bbaf0680d1fb41599` |
| klimt/creole/command/CommandCreoleSizeChange.java:130–138,155–167 | `cd9cd8fd6f013d26bce95b5d6d91647362d524930a8e4bf59467234be9f6407d` |
| klimt/creole/command/CommandCreoleColorChange.java:132–143,159–178 | `be9c29a8e1edc3cebc40d9a883992bce91e4234bb66391d94716152730dbae12` |
| klimt/creole/command/CommandCreoleFontFamilyChange.java:128–139,156–168 | `09d73bd78519572f24956607d9ad25d9302bcfb0bbe2fd8931d2ebfac6b9ca54` |
| klimt/font/FontStyle.java:119–159,187–235,281–295 | `7b6fb79f8a28e95cb42d64bd6d3c73e92f6a6d665e895992cf04b1d65093da28` |
| utils/CharHidder.java:118–164,180–199 | `accd201736dca2bef239b3b86bb9bb1ccd85f1632c310d972a58df632c38e276` |
| style/FromSkinparamToStyle.java:164–168,226–257,507–511 | `291a7ce954e6cdd357c8c64f04ca629a0866123c695f3e80301f4bf265f95235` |
| style/PName.java:112–151 | `722521de45434d6b18f4e8cf931f74404db670271192e740307d56db3b6d9f87` |
| style/Style.java:202–215,322–348 | `509654e15e6aeb15f49f7282052c1aaa6e726544ce0f4fc26201d2793f7e2408` |
| style/StyleLoader.java:168–194 | `33c8200d4bf811f6e39a8e9e5c6bdca6ff89cfa9ca14f340c52b13729fdb91ed` |
| style/StyleIndex.java:257–302 | `5e47f477996cc7633f1d9526e128be21b50489aaea1a2bdcd3667c98f4b3a382` |
| style/StyleQueries.java:114–128 | `4ed2d7d1ecf10f007534d62d835e6d8fbc50d86a137c9606bd028b3341068988` |
| style/parser/RawStyleParser.java:126–144,167–305,312–465 | `16f0691b8e0b1ed5cd46ad97f308f16c8820159b759c61faca9d09ce0280cf01` |
| style/value/Specificity.java:129–151,170–172,203–221 | `0a0a2bfc88886e51821377605705eb1f065cdae3bf03ff1b0fe3cf5934542bfe` |
| style/value/ValueImpl.java:150–160,273–291 | `a5765e750c82c653cbb8095347ee97f4b0a37b124851b7bbcec7bc5d7ec8fdc5` |
| klimt/color/HColorSet.java:152–256 | `ad23f277208273f7766c3d2d8f910306b69dbda4f36a6306fe0476413c88363d` |
| regex/Pattern2.java:124–132,192–196 | `44f65f0995340f76c38cb2bdd6e6c37ef80d286b488cf9a92f2705178cbe3ed0` |
| text/TLineType.java:132–147,240–247,344–365 | `e2d8f4d06627210fc6148df329bc9d18663b53c5647927f4e456271431eaac84` |
| tim/EaterAffectationDefine.java:120–133 | `001009a5bd744874e72233f839523eea054c07459621a137e68ab7062afb84db` |
| tim/TContext.java:435–520,865–875 | `b914f5053b91d59c2b9578f663d531823e59f34615657fadd3562a11f529fc5d` |
| tim/VariableManager.java:131–155,216–236 | `a9ec4da9f0ed946f97e81daa54b4a67a747b39561739c170a827237166476a36` |
| tim/TMemoryGlobal.java:124–156 | `5ef8f49e7356eaf65b0b3c8f178e7d9632ab7f3815b6bc86ca9fd84c2ce38b3f` |
| tim/TrieImpl.java:167–179 | `7f3f2be6de55897b0baf9c5fcf22779f0ffa82cee5be5380db89f932c8af2bec` |
| tim/EaterLegacyDefine.java:120–129 | `bb6fba29d6548706734bdc5d6e778230f0aa8cf6e0b99d88bf677e59bd283b39` |
| tim/FunctionsSet.java:192–198,235–245 | `9f8032ae1f58c2274efb6ada4f640fdac8dfcfe449220b0e94ca070d0a19e23c` |
| tim/Eater.java:265–280,377–415 | `d0ba4fb76878c93b3344a5f57d0715dbc256d0713f456affb8ba482172d09240` |
| tim/TFunctionImpl.java:235–245 | `7f392c6689394ffbc457ed15a58906e8fa8713eae0abf36a480c3fef12d64b64` |

All selected generated MIT headers carry `(C) Copyright 2009-2024, Arnaud Roques`; preserve the exact MIT grant and actual original-author/contribution lines for translated files. The generated edition retains older GPL comment headers underneath its MIT distribution grant; do not relabel, erase or conceal this history. Generic header credits to bundled themes/icons/math/compression data do not authorize importing those assets. This increment needs none. Continue existing audited Merman/Dagre notices; no dependency change is needed. Extend `provenance/plantuml-mit-1.2026.9.json`, relevant Rust module notices, support ledger and source audit only when controller releases implementation.
