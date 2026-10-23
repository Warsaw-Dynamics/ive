Important late-preamble grammar finding — ADDRESSED. `diagram-ports/engine/src/plantuml/salt.rs:366` delegates recognition to `style::is_preamble`; the shared `region_command` now supplies both actual chrome parsing and late-command detection (`diagram-ports/engine/src/plantuml/style.rs:182`, `:225`, `:233`). Colon separators, optional left/center/right header/footer alignment, case and whitespace follow the same grammar. The new 25-form test checks both valid prefix rendering and descriptive late rejection (`diagram-ports/engine/tests/plantuml_salt.rs:415`); neighboring controls and noncommand text retain their semantic kinds (`:463`).

Minor inline `{Text}` finding — ADDRESSED. Removing only T from opener-suffix consumption leaves bare `{` plus ordinary text (`diagram-ports/engine/src/plantuml/salt.rs:167`). Regression coverage accepts Text/Textile/T#literal while continuing to reject exact T/T#/T+/T-/T!/T^ tree markers (`diagram-ports/engine/tests/plantuml_salt.rs:499`).

New breakage in fix diff — None found. The region extraction preserves the old role, remainder and default/explicit alignment behavior; data-specific admission and payload validation stay on their existing paths (`diagram-ports/engine/src/plantuml/style.rs:172-223`, `:233`). The tokenizer change removes only the erroneous T suffix; no model/layout/resource/runtime boundary changed (`diagram-ports/engine/src/plantuml/salt.rs:154-177`).

Out-of-scope observations — Previously ledgered timing keyword-ID and Gantt trailing-comment Minors and inherited vendored Atomic deprecation remain unchanged/nonblocking. No new out-of-scope defect found. The canonical `/mnt/data/Projects/ive/progress.md` was read and not edited; its status is controller-owned.

Checks — Read the complete Task3G report including FIX ROUND1 and the supplied `82fad89..f6fa5d9` diff once as the changed-file view. No changed-file rereads, git operations, product/index/HEAD edits, subagents, suites, captures or hashes. Inspected `/tmp/ive-plantuml-3g-fix1.lyhn8I/focused.log`: all three new named Salt tests pass; Salt18/style17/constants31 pass with zero failed/ignored. The only warning is the previously deferred vendored Atomic deprecation. No concrete unanswered doubt required another probe.

SPEC — Approved for this scoped fix round. Both original findings are closed; unchanged cross-task guarantees remain subject to the controller's independent exact-head checks.

QUALITY — Approved. The shared grammar removes duplicated recognition logic, the fix remains limited to three files, and regressions exercise admission, rejection and adjacent literals rather than merely checking successful output.

Fix round — All findings addressed, no new Critical/Important breakage.
