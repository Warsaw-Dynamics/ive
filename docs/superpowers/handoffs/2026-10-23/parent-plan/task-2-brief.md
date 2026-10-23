### Task 2: Standalone pgui viewer and isolated worker

**Files:** Create `diagram-ports/viewer/{Cargo.toml,src/main.rs,src/app.rs,src/worker.rs,src/cli.rs}`, viewer tests; modify standalone workspace membership and README. Additional focused modules allowed.

**Interfaces:** Consumes Task 1's `render` API. Produces `diagram-viewer [FILE]`, `diagram-viewer --render FILE --output FILE.svg`, and a bounded worker protocol. Viewer can start with built-in corpus samples. A worker process of the same binary is acceptable as an isolation mechanism; no third-party external executable is allowed. Public headless engine remains GPUI-independent.

- [ ] Test CLI argument validation and worker failure/timeout/output-limit behavior before implementation. Use deterministic small test limits; actual defaults source 1 MiB, SVG 16 MiB, render timeout 10 seconds unless audited backend limits warrant stricter values.
- [ ] Implement the pgui host using the exact revision, platform features as IVE, and real SVG rendering; show language/example selection, source, progress/error, fit/zoom. Keep resource work off UI thread and drop stale results.
- [ ] Wire host text measurement if backend can accept it safely. State exact metric behavior and SVG font handling in README.
- [ ] Add explicit CLI SVG export and a background/no-focus launch option, plus app-frame capture or deterministic GPUI test capture for verification. Do not use xdotool unless no native alternative exists, and never activate windows.
- [ ] Run standalone tests, clippy and release build; launch the viewer against corpus samples, inspect a captured frame, confirm errors visible and shut down all test processes. Commit task files after checks.

