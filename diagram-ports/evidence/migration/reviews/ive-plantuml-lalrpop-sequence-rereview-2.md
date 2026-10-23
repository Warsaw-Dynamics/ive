**R1 — Reject malformed attached quoted frame suffixes before retaining raw frame display. — ADDRESSED.** `diagram-ports/engine/src/plantuml/parser/sequence.rs:208` validates frame/else suffixes before `frame_kind` or caption copying. The new scanner at `:376` visits attached and separated quotes; `:399` rejects unterminated identifiers and unsupported escapes. Thus the reported `group"unterminated` bypass no longer exists. Private/public regressions cover all seven frame commands plus else at `:779` and `diagram-ports/engine/tests/plantuml.rs:104`.

**Check — exact lexical and compatibility boundaries:** `diagram-ports/engine/src/plantuml/parser/sequence.rs:389` stops only at an unquoted colon outside a colon run, matching native `lexer::split_label` at `diagram-ports/engine/src/plantuml/lexer.rs:126`; quotes consume their internal colons and escaped quotes atomically. `sequence.rs:410` applies the native raw 16KiB quoted-content bound, including UTF-8 and escaped bytes, and `:413` admits the same four escapes as native `lexer.rs:92`. Existing frame spelling, raw quoted-else caption and canonical Alt/Par else selection remain untouched; post-colon quote text remains literal. The new private/public colon controls at `sequence.rs:810` and `engine/tests/plantuml.rs:135` pass in the supplied covering log.

**Check — actual work, storage and origins:** `diagram-ports/engine/src/plantuml/parser/sequence.rs:385` and `:407` charge/check each visited UTF-8 character, including delimiters, before processing. The extra scan is linear and retains only fixed-size cursors/escape state; no input-sized allocation, generated symbol/table or retained payload changes occur. New failures use existing `Row::span` at `:418` and `:429` and the unchanged once-mapped facade. No legacy statement parsing, fallback or changed default limit is introduced.

**Check — supplied evidence:** inspected the named covering output `/tmp/ive-task4-fix2-covering-green.log`: **302/302** (167+42+44+23+26), including both new malformed-header and post-colon contracts. The report names the exact covering command. Existing strict Clippy log completes successfully; fmt log is empty; source audit reports **220/43/43**; private-target generation log verifies **five** reproducible outputs without tracked writes. Broad integration gates/baseline/native remain controller-owned. No test, generator, native probe or suite was rerun; no concrete unresolved doubt required one.

**Check — review scope:** read the brief, binding global/parallel contexts, previous full feedback, FIX ROUND 2 appendix and complete prepared fix diff once. No git command, checkout/source/index/HEAD mutation or subagent was used.

**New breakage in the fix diff:** None.

**Out-of-scope observations:** None.

**Fix round:** **All findings addressed, no new Critical/Important breakage.** R1 is closed; I1 remains addressed.
