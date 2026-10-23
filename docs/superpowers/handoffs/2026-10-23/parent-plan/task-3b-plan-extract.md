### Task 4: State and structured activity

Files: `state/…`, `activity/…`, fixtures/tests. Share drawing/text/graph utilities, not ASTs.

- [ ] State tests: initial/terminal nodes are distinct, aliases and transition labels, nested composite ownership, state body lines; reject concurrent/history syntax until its dedicated implementation. Add the concurrent-region feature as a separate reviewable extension.
- [ ] Activity tests: if/elseif/else branches converge correctly, while and repeat edges point back, fork has separate lanes and join, guards remain on edges, action text retains semicolons/newlines as grammar permits, stop does not create a fallthrough path. Test malformed stack endings and depth budget.
- [ ] Implement immutable models, bottom-up activity tiles with explicit input/output ports, and a distinct state graph adapter. Expand swimlanes/break/kill/detach only with route/lifecycle tests.
- [ ] Compare branch/loop topology to oracle fixtures before considering geometry polish.

