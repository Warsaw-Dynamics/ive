# Task 3J FIX ROUND2 scoped independent re-review

**N1: ADDRESSED.**

**SPEC: PASS for this scoped repair. QUALITY: PASS for this scoped repair.** No new Critical, Important or Minor finding. I1/I2 remain closed; M1 remains separately deferred. Task approval and the LALRPOP migration release belong to the controller.

## Scope and method

Reviewed fix base `40b949a278fcc2747316c0cfd182b23add4685cb` → frozen head `4c197871caf992a1a852b7048311b4ee450d74ff` in `/mnt/data/Projects/ive/.worktree/diagram-ports`.

Read the binding Task 3J brief/global constraints and the full FIX ROUND2 appendix in `task-3j-report.md`. Read the prepared 8,956-byte `review-40b949a..4c19787.diff` once, in two bounded portions. Separately completed `preprocess::Processor::expand` because the production hunk cuts that function and the new cursor advance must be checked against its later Direct/Expansion handling. No regenerated git view, untouched-family survey or closed-finding re-review.

No suites, audits, captures, original renderers or headless controls were repeated. The code and supplied fresh evidence resolve the concrete N1 question; no unanswered code doubt justified a novel runtime diagnostic. No product/progress/index/HEAD edits, Java/browser/dot/network/GUI launch, subagents or worktrees. Only this report was written under `/tmp`.

## N1 — replacement gating can reuse a consumed comment-delimiter byte and suppress an active constant

**Verdict: ADDRESSED.**

The old boolean activity result allowed `expand` to visit the two bytes of a real comment delimiter separately. A consumed slash or apostrophe could then participate in an overlapping delimiter, creating a phantom comment state and suppressing the following active constant.

The repair makes that consumption explicit with `ScanStep::{Active,Comment,Delimiter(bytes)}`. Both real opener and closer transition the comment state and return `Delimiter(2)`. `expand` charges the consumed bytes, advances the original-source cursor by both bytes and continues before word lookup or another activity transition. This directly removes delimiter-byte reuse in both directions; it does not special-case a label, alias or reproduction string.

The two-byte advance is safe at the producer boundary: each Delimiter result follows a successful `starts_with` check for the corresponding two-byte ASCII delimiter. It therefore stays within the current UTF-8 source slice and progresses to a valid character boundary. The ordinary Comment branch keeps comment-owned words inactive.

The new regressions assert exact semantic text for the original Chen title and existing message reproductions, preserve the exact spaced control, and exercise the paired opening-delimiter overlap with a delimiter-valued constant. The supplied meaningful unchanged-production baseline was 39 passed / 3 failed, followed by 42 passed / 0 failed after the repair. The two preservation controls are correctly described as already passing before the fix.

## Named fix risks and new-breakage assessment

- **Original cursor and Direct bytes:** the delimiter branch advances `cursor` without advancing `direct`, deleting source or emitting an Expansion. Consequently the next existing append, or final append, retains the full untouched comment/delimiter slice. The private test byte-compares every Direct segment with its original source range.
- **Expansion spans:** replacement still uses `span.slice(cursor..end)` for the actual original active name and advances to its original end. Skipping a delimiter cannot create an expansion for its bytes. The private test requires exactly one Expansion, verifies the original trailing X byte offset/physical line/definition ID, and checks its replacement Ready. Commented X remains Direct text.
- **Both delimiter directions:** the explicit producer consumes each complete delimiter before any further activity scan. The opener regression prevents its apostrophe being reused as a closer; the original closer regressions prevent its slash being reused as an opener.
- **Quoted/raw/frozen ownership:** definition targets and explicit literal input still choose Active before invoking the scanner. Quoted source remains Active through the existing quote/escape handling and cannot enter the unquoted comment-delimiter branch. Dedicated preservation tests cover quoted participant display, raw note text and an eagerly frozen definition containing comment-looking bytes. Raw note/action/data ownership and the insertion/no-rescan implementation are unchanged by this diff.
- **Bounds/cancellation:** the existing per-iteration Budget charge occurs before the scanner, and the delimiter branch charges its bounded consumption before advancing. It allocates no unbounded intermediate representation and retains the same append/insert/origin storage checks. Every branch makes bounded cursor progress. The private 8/8 evidence includes prior source/model/retained-text/origin/lineage limits, cancellation-before-retention and original-cursor/no-rescan invariants.
- **Scope and maintained behavior:** the diff changes only the shared preprocessor and constants tests. It does not alter family grammars, public APIs, layouts, paint, dependencies, provenance selection or worker services. The closed dispatch/comment-definition fixes are not changed or reopened here.

No fix-introduced breakage was found in these paths. No additional considered behavior was promoted into a finding.

## Supplied verification and limits

The controller independently reports 488 ordinary Rust tests plus seven separately counted actual worker faults; 15 Node tests; all strict gates; PlantUML audits 220/43/43 and unchanged Merman 656/four patches. The appendix records the exact focused RED/GREEN commands, private 8/8 origin/bounds checks and fresh complete gate results.

The three exact prior reviewer inputs exported successfully with their required exact text: Chen title `start ' Ready`, message `start ' Ready`, and separated control `start  ' Ready`. The new genuine native Chen capture was controller-viewed/hash-verified with 2 actual native measurements / 0 fallback and the correct title. These are supplied independent controller facts, not duplicate reviewer executions or a new pixel-parity claim.

This review establishes the scoped code repair and no identified new breakage. It does not claim a fresh independent rerun of all runtime evidence or universal PlantUML compatibility.

## Out-of-scope observations and disposition

- M1 remains the existing separately deferred Chen keyword-dispatch Minor. Its parser is untouched by this fix.
- The user-mandated whole-current-PlantUML LALRPOP rewrite is separate upcoming work. This repair supplies corrected shared lexical ownership; it does not claim to implement or approve that migration.
- No new out-of-scope observation requires a ledger entry.

N1 is addressed with no remaining blocker found in FIX ROUND2. The scoped SPEC/QUALITY re-review passes; the controller owns Task 3J approval and the new implementation queue.
