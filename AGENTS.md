# IVE - notes for agents

Read [README.md](README.md) first: how to run IVE, use it, and drive it over MCP.

- Build and test from this folder: `cargo test`, `cargo clippy --all-targets`, `cargo build --release`.
  Dependencies are optimised even in debug builds (`Cargo.toml`), so `cargo run` is representative.
- Check a change in the running app through IVE's MCP server, not xdotool:
  `./run.sh -- --profile agent-<something> --background .`, then the tools in the README; the URL and token are in
  the startup output and in `~/.config/ive/agent-<something>/mcp.toml` (the `ive` MCP tools of your own session
  reach the IVE you run in, through `IVE_MCP_URL` and `IVE_MCP_KEY`). `--background` keeps the window from taking
  focus, so the user can keep working meanwhile.
- If xdotool is unavoidable: find IVE's window by process id, never by title (other apps have "IVE" in their
  titles), send input with `--window <id>`, and never activate windows or move the real mouse.
- Run test instances with their own profile, `--profile agent-<something>`: it keeps their projects, tabs, settings,
  scratches and terminals apart from the user's. Without it, one started while the user's IVE is closed adopts the
  user's running terminals and overwrites their saved tabs.
- Stop the IVE instances you start when done; a leftover one holds its MCP port. Its terminals keep running: end
  them with `IVE_PTY_DIR=$XDG_RUNTIME_DIR/ive-pty/<profile> ive --pty-host kill <id>`, or answer the quit prompt with `prompt:Stop all`.
- Tests follow `should_…` names with `// given`, `// when`, `// then` sections, like the existing ones.
