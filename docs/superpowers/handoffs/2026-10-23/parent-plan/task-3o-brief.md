# Task 3O: Native legacy BPM branches

Read first: dedicated Task8 BPM requirements. HOLD until controller release from reviewed base. Read binding `native-global-constraints.md`, precedingreports and `/tmp/ive-plantuml-inventory-remaining-semantics.md` completely. One new family, NOT full BPMN or modern activity aliases.

Exact @startbpm/@endbpm, colon-events and implicitstart, balanced nested new branch/else/end branch with real ordered brancharms, split/merge and continuation. Immutable BPM instructions/FlowNode kinds Start/Event/Merge and ownededges. Dedicated sparse grid/branch compositor may share lowlevel activityports; never raw Mermaid rewriting or generic labels. No Check→Retry edge between alternatives; both live tails reach merge before Archive. Original END enum has no command; don't invent syntax.

Source firstelse creates exitmerge, laterelse closesarm/resumesentry, endbranch closeslastarm. Reject singlearm branch if its original exit semantics remain unverified, rather than crash or inventdefault. Namedmerge/goto/resume only if actualdistinct semantics: goto addsconnectionandmovesrow; resume relocatescursorwithoutthatconnection. Reject unresolved/redefineddestinations, unmatched/duplicate/invalidclose and modernactivitysyntax. Fullsourceconsumption, unknowncommands notignored.

Count allsyntheticsplits/merges/edges under model/depth/work limits. Sparsecell layout, bounded cleanerpasses or original deterministic compositor without Java's unchecked cleanup loops. Measure boxes/branches/ports and containlabels/routes; finite geometry/cancellation/strictterminalusvg.

RED topologytests: implicitstart/eventorder, splitandmerge/continuation, nestedidentity/multiplearmswhenadmitted, goto-vs-resumeifimplemented, malformed/unclosed/singlearm/unimplementedcommands, customsyntheticitems/gridwork/depth/SVG/cancel/determinism; actual indegrees/outdegrees anddistinctshapes/labelcontainment beyondSVGwords.

Exact `/tmp/ive-plantuml-native-oracles/bpm-events-branch.puml` and substantive nestedcases, freshoriginalprobesifneeded; preserveartifacts/sourcemappings. Honestviewerexample and actualbackgroundbranchedcapture/nativecounts, inspect/stopinstances. Commonfocused/fullverification, scopedcommit, `task-3o-report.md` here with APIs/sourceproof/grammarboundaries/REDGREEN/counts/commands/captures/residuals. Independent spec+quality gate.
