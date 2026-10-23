# Task 3K: Native EBNF/regex typed railroad rendering

Read first: requirements for shared railroad Task8 checkpoint. HOLD until explicit controller release from preceding reviewed commit. `native-global-constraints.md` here binding; read it, preceding contract reports and `/tmp/ive-plantuml-inventory-remaining-semantics.md` fully. Only EBNF and regex with a shared low-level railroad geometry compositor. Source strings never rewritten to Mermaid; no regex matcher/execution or recursive grammar-reference expansion.

## Common railroad composition

Immutable family-specific ASTs with source spans and stable IDs. Shared measured tile geometry carries width/above-baseline/below-baseline/entry/exit. Concatenation aligns baselines, choice stacks ordered branches on shared rails, optional has bypass, one-or-more loop without zero bypass, zero-or-more both. Typed bounded polyline/path curves may be added with exact provenance/original geometry disclosure. Label boxes and all routes/markers nonoverlapping/contained, actual branch/loop topology—not a horizontal list of boxes.

No child AST cloning for counted repeats, no expanding references or character ranges into uncontrolled nodes. Bound tokens/nesting/symbolic numeric lexemes/count metadata/generated primitives/work before allocation, periodic cancellation and truthful shared font/style measurement/paint. Exact safe terminal/usvg pipeline unchanged.

## EBNF

Exact @startebnf/@endebnf. Ordered named rules; both quoted terminal kinds and escapes as inspected grammar; references distinct from terminals (even same visible text), explicit comma concatenation, | choices, grouping, [...] optional, {...} zero-or-more and {...}- one-or-more. Multiple and bounded multiline rules, semicolons inside quoted terminals do NOT terminate rules. Rule references remain symbolic labels, including undefined references; no UML resolution or expansion. Duplicate rule append semantics implemented or narrower rejection documented.

Use dedicated EbnfDocument/EbnfRule/EbnfExpr. Special ?sequence?, (*comments*) which become attached annotations, exception-minus and numeric*count only admitted when their real typed/visible semantics implemented; otherwise reject wholeconstruct. Never silently strip visible comments or reinterpret exception as concatenation. Empty expression needs explicit tested Empty rail; empty document error. Unterminated quote/comment/rule and crossed brackets fail located/bounded, don't inherit original unbounded scans.

RED tests: rule/reference/source order, distinct terminal/ref shapes, quote-contained semicolons/commas, correct precedence, alloptional/repeat/bypass distinctions, multiple/multiline rules, undefined symbolicref not expansion, malformed/unsupportedexception/count/comment handling, depth/tokens/manyrules/numerictext/output/cancellation bounds and geometric ports/containment.

## Regex

Exact @startregex/@endregex, initially one complete expression. Dedicated RegexExpr literal/class/sequence/choice/repeat/group variants. Admit escaped punctuation, concatenation, nested groups/alternation, positive/negated nonempty character classes/ranges, dot and explicitly modeled class escapes, ?,*,+,{n},{n,},{n,m}. Preserve symbolicmin/max, sourceorder and classnegation/range semantics. Validate n<=m and bounded numeric tokens; count is metadata, not allocation. {0}/{0,0} upstream rendering quirk must be explicit rejection or documented visibleannotation, never claim exact regex-language evaluation.

Capture vs noncapture distinction retained if admitted. Anchors/lookaround/namedgroups/unicodeclasses/backrefs/flags/atomic/possessive/lazy syntax requires actual model+decoration, otherwise named error—not swallowed '+'/'?' or literal fallback. Regex quotes/apostrophes/slashes/backslashes/whitespace are its own lexical grammar, not UML comments. Multiple unrelated lines explicitly reject until original combination semantics modeled.

RED tests: a|bc vs(a|b)c precedence, exact repeats/no clone, [^a-c\\]] exclusion/range/escaped ], distinct +loop/no bypass vs*loop+bypass, orderedchoice/concatports, malformedgroups/ranges/quantifiers/classes, unsupportedassertions/backrefs/flags, 65deepgroups/customdepth/items/work/output/cancellation, no regexengine/backtracking invoked.

## Evidence/delivery

Use exact `/tmp/ive-plantuml-native-oracles/{ebnf-optional-repeat,regex-branch-repeat}.puml`, preserve source/SVG manifests and reporttrue currentnative results. Add representative multifeature cases and freshboundedoriginalprobes if required. Shared ETile geometry anchors and notices are in report; inspect only authorized actualsourcestranslated, trace hashes/destinations/behavior.

Two honest viewer examples through existingworker; actual background captures show branches, loops, bypass and text/ports with native counts, inspect and stopinstances. Apply common fullverificationcontract; scopedcommit and `task-3k-report.md` here with AST/compositor APIs, source traces, exactadmitted/rejectedgrammar, REDGREEN/counts/commands/captures/residualgeometry. Review your own diff before reporting; do not dispatch a review agent. The controller owns the independent spec+quality gate.
