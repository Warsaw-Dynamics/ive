# IVE - notes for agents

Read [README.md](README.md) first: how to run IVE, use it, and drive it over MCP.

- Build and test from this folder: `cargo test`, `cargo clippy --all-targets`, `cargo build --release`.
  Dependencies are optimised even in debug builds (`Cargo.toml`), so `cargo run` is representative.
- Check a change in the running app through IVE's MCP server, not xdotool:
  `./run.sh -- --profile agent-<something> --background --stop-processes-on-quit .` (Windows: `.\run.ps1 --profile
  agent-<something> --background --stop-processes-on-quit .`), then the tools in the README;
  the URL and token are in the startup output and in `~/.config/ive/agent-<something>/mcp.toml` (the `ive` MCP tools
  of your own session reach the IVE you run in, through `IVE_MCP_URL` and `IVE_MCP_KEY`). `--background` keeps the
  window from taking focus, so the user can keep working meanwhile.
- If xdotool is unavoidable: find IVE's window by process id, never by title (other apps have "IVE" in their
  titles), send input with `--window <id>`, and never activate windows or move the real mouse.
- Run test instances with their own profile, `--profile agent-<something>`: it keeps their projects, tabs, settings,
  scratches and terminals apart from the user's. Without it, one started while the user's IVE is closed adopts the
  user's running terminals and overwrites their saved tabs.
- `--stop-processes-on-quit` makes quitting a test instance stop its terminals and processes without the quit
  prompt, so none outlive it.
- Stop the IVE instances you start when done; a leftover one holds its MCP port. Without `--stop-processes-on-quit`
  its terminals keep running: end them with `IVE_PTY_DIR=$XDG_RUNTIME_DIR/ive-pty/<profile> ive --pty-host kill <id>`
  (macOS: `IVE_PTY_DIR="$HOME/.ive-pty/<profile>"`), or answer the quit prompt with `prompt:Stop all`. A killed
  (SIGKILL) or crashed IVE leaves them running either way.

## Coding guideline

These apply to any language. Where a rule names a construct, use the language's equivalent.

### Language and types

- Use the language's current idioms and features (records/data classes, type inference, pattern matching, etc.).
- Avoid null or nil where possible. Use the language's option or result types at boundaries instead.
- Do not store optional wrappers (Optional, Maybe, Option) in fields. Use them for return values only.

### Errors

- Do not use exceptions or errors for control flow. Fail fast, but prefer readability over avoiding exceptions.
- Make error messages self-contained: state what failed and the offending value, so they make sense in a log
  without a stack trace.
- Validate and bound inputs. Reject unreasonable values rather than letting them run.

### State

- Do not mutate state. Fields are immutable, except where a library requires setters or mutable structs.
- Keep collections as immutable as you can: return unmodifiable views, copies, or the language's immutable types.
- When data is fixed, prefer immutable literals and constructors (e.g. `List.of`, tuples, `frozenset`, `const`,
  `readonly`).
- Prefer composition over inheritance.

### Functions

- Keep each function at a single level of abstraction. Do not mix low-level operations (direct stdout/stderr
  printing, byte twiddling, raw I/O) with higher-level helper calls in the same function.
- Prefer fewer than 20 statements per function.
- Prefer at most 3 parameters per function. When a constructor needs more, use a builder, an options/config
  struct, or named arguments (enum-like constant definitions are exempt).
- Do not use boolean parameters, except in setters. Use an enum or two separate functions.
- Avoid nesting deeper than 3 levels.
- Avoid negation in conditions: `if (!condition)` becomes `if (condition)`.
- Return early:

  ```
  if condition:
      return 1
  return 2
  ```

  instead of

  ```
  if not condition:
      return 2
  else:
      return 1
  ```

### Naming

- Name functions with verbs, except property-like accessors.
- Avoid negation in function names: `notEmpty` becomes `isEmpty`.
- Follow the language's naming convention for constants, and give that style only to true compile-time or static
  immutable constants. Instance-level immutable fields use the ordinary field style (e.g. `setupFile`, not
  `SETUP_FILE`).

### Comments

- Doc comments state the contract: what it does, its inputs, and its errors. Keep implementation details in inline
  comments.
- Do not write comments that restate the code or a name. The best comment is a precise name. Comment only what the
  code cannot express: why, non-obvious invariants, references.
- Keep doc comments terse: a one-line description, plus an example only where it helps. Skip parameter, return and
  error tags when the names and types already say it. A doc comment should never be longer or harder to read than
  the body it documents.

### Tests

Name tests `should…` (or the language's equivalent, e.g. `should_…` in snake_case) and split each into three
sections, like the existing ones:

```
test should_<expected_behaviour>:
    // given
    ... setup ...
    // when
    ... code under test ...
    // then
    ... assertions ...
```
