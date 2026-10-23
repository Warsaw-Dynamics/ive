# Reviewed sequence parser interface

Task4 reviewed head1506eb1f14248eb6dfe109164308f6914e3421ec, basef5a10a0, integrated and fully verified at0c5177e. Original I1 and fix1 R1 addressed after two scoped fix rounds. See integration-1-interface.md for the approved combined BodySource/dispatch interface; this is a supported-subset checkpoint, not full-port completion.

- `parser::sequence::parse(BodySource, &mut Budget)` returns the preserved `sequence::Sequence`; generated productions own declarations, messages, notes, interaction frames, lifecycle, ordering and EOF. Builders receive typed values and preserve model/layout identities.
- BodySource retains a copied shared Ownership checkpoint from before the first physical family row and the adapter allowance. Other generated family parsers continue that checkpoint; never rerun document/preamble/comment ownership independently.
- Parse generated models before the remaining legacy scene/error bridge. Facades map structured failures through the immutable origin Document exactly once; layout and unmigrated-family bridge remain separate.
- Command recognition follows the first decoded primitive, including quoted reserved words and adjacent quote/operator boundaries. Frame display preserves raw spelling; canonical Alt/Par alone allows else. Quoted else retains the old raw caption slicing. Ordinary raw notes and quoted-command normalized notes retain their distinct source ownership.
- Before retaining raw frame/else text, validate all quoted primitives before the first unquoted single colon. Colon runs remain header text; post-colon quotes are literal caption data. Preserve bounded quote escapes and raw quoted-token length; no complete legacy command parser is used.
- Generated output SHAadd5ecf772bb37ce05df4a68a8050a7785d1ff06f82a2c1ec390c213f2fc356e:72states/25terminals,2d+8 live states, max4 reductions and12 expected terminals. Actual capacity/diagnostic/model growth is admitted before allocation; arbitrary universal table constants are insufficient.
- Reusing BodySource for several optional preparse routes uses common `reborrow()` and actual fixed live-view accounting, supplied by the approved0c5177e integration.

Evidence: task-4-report.md plus `/tmp/ive-plantuml-lalrpop-sequence-review.md`, `...-rereview-1.md`, `...-rereview-2.md`. Root independently verified all16 gates/561+7 on fix1; fix2 covering302 and unchanged generated hashes reviewed. Root inspected native note/primitive frames; complete combined baseline/native checks follow integration.
