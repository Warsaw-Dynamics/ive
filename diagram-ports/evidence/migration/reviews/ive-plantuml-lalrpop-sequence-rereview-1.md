**I1 — Restore command recognition at the first primitive-token boundary.** — **ADDRESSED.** `diagram-ports/engine/src/plantuml/parser/sequence.rs:205` obtains the first primitive, `:206` selects the command from its decoded spelling, `:382` reuses the bounded quoted decoder, and `:391` uses the primitive punctuation/quote/arrow boundary scanner. Adjacent participant/lifecycle arguments become separate tokens; adjacent-arrow and quoted reserved endpoints become commands and reach generated suffix rejection. The added private/public tests cover all four original polarity directions, including quoted commands and aliases (`diagram-ports/engine/src/plantuml/parser/sequence.rs:623`, `diagram-ports/engine/tests/plantuml.rs:8`).

**New breakage — Important R1: reject malformed attached quoted frame suffixes before retaining raw frame display text.** `diagram-ports/engine/src/plantuml/parser/sequence.rs:208`, `:240`, `:355`. The restored command classification newly sends `group"unterminated` through `frame_kind`: it copies the entire raw prefix into `FrameKind::Literal`, emits an empty caption, and never visits the quoted lexer for the attached suffix. Consequently this complete input now renders successfully:

```plantuml
@startuml
participant A
group"unterminated
A -> A : body
end
@enduml
```

**R1 compatibility evidence:** the removed native sequence entrypoint first called `lexer::tokens` on the complete pre-colon head before command dispatch (supplied original review package, removed `sequence::parse`); native `diagram-ports/engine/src/plantuml/lexer.rs:56` enters `quoted` at the adjacent quote and `:110` rejects the unterminated identifier. The fix base also rejected this spelling: its whitespace command prefix did not match, and its primitive scan reached that quote. The new frame branch bypasses that validation. Preserve valid attached/quoted frame display and exact else eligibility while retaining lexical rejection of malformed quoted header primitives; add a focused rejection control. This is new acceptance introduced by the fix, within the named attached-frame/primitive-boundary risk, not an unrelated pre-existing canonical-caption issue.

**Check run — one novel attached-frame malformed-quote probe:** under `bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`, invoked the existing release `diagram-viewer --language plantuml --render /tmp/ive-task4-rereview1-attached-frame-unterminated.puml --output /tmp/ive-task4-rereview1-attached-frame-unterminated.svg`. Exit **0**, **2,171 SVG bytes**. No Cargo/build, original four public probes, package suite, gate, generation, baseline or capture was repeated. Probe artifacts are the named new `/tmp` files.

**Check — supplied package and evidence:** read the required brief/global/parallel context, initial finding, FIX ROUND 1 appendix and complete 39,012-byte `review-c23afd6..a23aaee.diff` once. The initial combined context response was truncated; only the omitted necessary brief/global context was retrieved separately. No git command or checkout/index/HEAD mutation was performed. Independently inspected the existing covering logs: private `parser::sequence::tests` **13/13**; public/common **40+44+23+26 = 133/133**, with the new boundary, caption/note and noncanonical-frame tests explicitly passing. The remaining all-16-gate/561+7/43-baseline claims were left to the controller's independent verification.

**Check — named frame/else/note compatibility:** focused comparison with removed native `command`, `open`, `collect_note`, unchanged native primitive lexer, shared Ownership/Row spans, adapter normalization and unchanged generated sequence grammar. Canonical versus literal frame display, exact Alt/Par else eligibility, raw byte-4 quoted-else captions and quoted-note normalized rows agree with those removed semantics. Ordinary literal notes retain their existing ownership; discarded normalized rows charge/check before omission. The malformed attached quoted frame suffix above is the one uncovered exception. No additional runtime probe was needed for these controls.

**Check — named ownership/allocation/origin risks:** `FrameKind::Literal` capacity enters retained syntax admission at `diagram-ports/engine/src/plantuml/parser/sequence.rs:544`; actual Token/Name/Declaration/Anchor sizes determine generated symbol reservation at `:565`; `Builder::open` charges the separate event display copy before allocation at `diagram-ports/engine/src/plantuml/sequence.rs:286`; generic `retain` checks actual enlarged frame-element growth at `:371`. RowMode/Suffix are fixed local enums; no input-sized metadata or origin pass was added. New slices use Row byte mapping, physical CRLF/base advancement stays intact, and the facade maps failures once. Grammar/generated tables and token alphabet are outside the fix diff; no legacy raw-command forwarding was introduced.

**Out-of-scope observations:** None.

**Fix round:** **Findings remain open — new Important R1.** Original I1 is addressed; the fix introduced the malformed attached-frame quotation acceptance above.
