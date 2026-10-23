# Task 3I: Small native inventory models—files, packetdiag, board, HCL

Read this first; it is your requirements. HOLD until explicit controller release from preceding reviewed commit. Read `native-global-constraints.md` here as binding requirements, previous frozen task reports for actual shared interfaces, and `/tmp/ive-plantuml-inventory-small-semantics.md` completely. This is the first detailed Task8 checkpoint: four small dedicated models sharing common scene/data utilities, not generic-family aliases. Chen/ArchiMate and railroad/other inventory remain later gates.

## Files

Exact @startfiles/@endfiles. Immutable ordered path trie, hidden root, distinct Folder/File/Note and numeric parent IDs. Leading slash is syntax, no IO/normalization; dot/dotdot literal. Reuse existing same-name sibling folders preserving first-insertion/preorder, append duplicate final files, distinguish file and folder same name, trailing slash declares folder. Reject unusual empty components if not implemented; never silently normalize.

Measured preorder rows with depth indent and nonoverlapping full row/note heights. No connector edges invented. Native folder/file/folded-note primitives, no Twemoji asset import; disclose symbol differences. Notes attach as sibling in last-file parent; after folder-only path anchor is root; consecutive notes retain anchor. If note grammar is not implemented reject it, not ignore through EOF. Unknown nonpath/note lines and unterminated notes are errors (stricter than original silent ignore).

RED tests: ordered folder reuse/DFS, duplicate file vs folder identity, empty folders, both note anchors and multiline clearance, Unicode/XML/dotdot noIO, invalid/empty paths and unknown lines, preallocation segment/depth/expanded-name/lookup/work limits. Avoid fullpath cloning at every ancestor.

## Packetdiag

Exact @startpacketdiag/@endpacketdiag; balanced canonical packetdiag{...} if braced form admitted. Dedicated ordered inclusive bit-range fields, config and generated row fragments retaining field IDs. Width=end-start+1, native correct row wrapping/splitting under common bit scale, proportional rectangles and actual bit ticks/indices. Explicit source ranges are laid out by declaration width, NOT visual gaps, or restrict noncontiguous input explicitly. Default colwidth16; implement admitted configuration effects (colwidth/scaleinterval/direction/nodeheight/sameheight) or reject, don't store unused keys. * auto-position and len/height attributes require exact positive/negative tests if admitted.

Conserve field widths across rows: colwidth8 with0-2 then3-22 fragments are5,8,7; original remainder code can emit5,8,8, so document deliberate correction. Same-height/variable-height clearance policy explicit. Rotate and continuation flags are not proof of original paint support; reject unsupported rotate and don't claim flags drawn when not. UTF8-safe bounded measured ellipsis if used, retain full semantic label and honest truncation diagnostics.

RED tests: inclusive widths/identity, three-row conservation, declaration-width positioning and * semantics if admitted, proportional4bit:8bit geometry/tickalignment, RTL block/tick reversal, interval/height effects, descending/zero/overflow/unknownattributes/rotate/malformedbraces, colwidth1/no modulozero, huge sparse numeric request hits fragment/tick/item/work/geometry limits before expansion. Actual packet contributor notices in report must be retained (kolulu23, Duy Nguyen, The-Lum on files translated).

## Board

Exact @startboard/@endboard. Zero-plus lines start ordered activities; plus-count gives stage row. New node parent is nearest prior ancestor with lower stage; jumps allowed and preserve empty rows. Dedicated activity/tree IDs, ordered children and per-activity cursor, not Kanban lanes. First child retains parentcolumn; later child column follows previous subtree; next activity starts after full prior branchwidth.

Measured cards on stage rows, dashed stage boundaries, each root painted ONCE (original duplicate-root paint is a documented correction). Larger cards/pitches may contain measured longlabels while preserving stages/columns. No invented arrows. Plain punctuation labels valid, but explicit denied capability/style/preprocessor commands must not fall through as root text.

RED tests: zero-plus roots, nearestlowerancestor, stagejumps/emptyrows, DFScolumns and nextactivityoffset, realstageboundaries/single rootpaint/longlabelcontainment, releasebeforeroot error, capability/unsupportedcommand rejection, plusdepth/generatedrows/columns/card budget and cancellation. Never unchecked recursive parent/computeX loops.

## HCL

Exact @starthcl/@endhcl; PlantUML-specific ordered data grammar, NOT Terraform evaluation. Typed blocks/fields lowered to existing ordered DataValue renderer with real compartment-row/child ownership. One distinct topblock unwrapped to its body; several blocks keep ordered textual names; duplicate topblock replacement-at-first-position implemented or explicitly rejected. Duplicate fields append preserving rows or explicitly reject, never silently deduplicate/sort.

Quoted/simple scalar numbers/bools/null are STRINGS, nested objects and quotedstring/object arrays as exact subset. Unsupported bare array tokens/nested arrays/calls/expressions/heredocs/interpolation must fail, not disappear. Function syntax is representational upstream, never executable, and may stay unsupported. Quote escape/backslash and line-concatenation behavior explicit; don't silently substitute standard-HCL/JSON semantics. Whole hash-comment lines allowed, hashes in strings remain data. No SmetanaForJson translation.

RED tests: singleblock unwrapping/fieldorder, multipleblocks/displaynames, duplicatepolicy, stringscalars, nested owningrow edges/emptycontainers, comments/escapes/Unicode/XML/resource-lookingliteralstrings, unsupportedarray/call/expression malformedquotes/braces, depth/items/work/retainedtext before allocation. EclipseSource MIT notice for translated ordered-object behavior preserved.

## Integration/evidence

Exact fresh reference fixtures in `/tmp/ive-plantuml-native-oracles`: files-tree, packetdiag-ranges, board-columns-cards, hcl-nested-data. Preserve hashes and original artifacts; add revealing multifeature source-derived cases beyond simple admissions, original probes when needed. Actual authorized sources/hashes/notice anchors are in the read-only report; inspect translated files yourself and extend only actual product traces.

Each output tested for model meaning/shape geometry/finite containment/unknownmixedsyntax/usvg, custom budgets/cancellation/determinism. Four honest viewer examples, actual background captures including split packet, branched board and nested data/files, inspect and stop instances. Apply common verification/report contract. Scoped commit and `task-3i-report.md` here with all four family support boundaries, source notices, RED/GREEN counts and new shared contracts. Independent review required; no other new family scope.
