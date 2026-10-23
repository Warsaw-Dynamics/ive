# User Themes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Users keep themes as partial TOML files in `<profile>/themes/`, pick one on the Appearance page, and the
Scripts page explains where every customisation goes, with a Copy button.

**Architecture:** The script thread owns the picked theme's name. Before it runs `ive.rn`, it merges
`themes/<name>.toml` over the default theme table, checks the result, and hands it to the scripts as the starting
`config.theme`. Theme files trigger reloads through the existing fingerprint and folder watch. The gpui side stores
the choice in `settings.toml`, sends it to the script thread, and lists the available names that every `Loaded`
event carries.

**Tech Stack:** Rust, gpui, Rune, `toml`, `serde`.

**Spec:** `docs/superpowers/specs/2026-10-23-user-themes-design.md`

## Global Constraints

- Theme files: `<profile dir>/themes/<name>.toml`. Only `*.toml` files directly in `themes/` count, and the name is
  the file stem.
- Larger than 256 KB (`262_144` bytes): refused with the size in the message.
- Layering, bottom to top: default, theme file, `ive.rn`, `init.rn`. Tables merge key by key; anything else,
  arrays included, replaces the default's value.
- Theme errors start with `themes/<name>.toml: `, followed by the existing `theme.<path>: …` message.
- Missing picked file: log `theme "<name>": no file themes/<name>.toml; using the default`. The setting is left
  unchanged.
- Click targets: `user-theme:Default`, `user-theme:<name>`, `copy-script-instructions`.
- README link: `https://github.com/Warsaw-Dynamics/ive#scripting`.
- Coding guideline from AGENTS.md: no boolean parameters, at most 3 parameters, no `Option` stored in fields where
  avoidable, tests named `should_…` with `// given`, `// when`, `// then`.

## Review Focus

1. A broken theme at start must keep `init.rn`'s bindings. Tested in Task 3:
   `should_keep_init_bindings_when_theme_broken_at_start`.
2. A broken theme on reload keeps the previous theme, not the default. Tested in Task 3:
   `should_keep_previous_theme_when_theme_breaks_on_reload`.
3. A theme with an unknown key such as `[dark] tre = "#000000"` is rejected with its path. Tested in Task 2:
   `should_name_file_and_key_of_unknown_theme_key`.
4. Non-TOML files and subfolders in `themes/` are not listed. Tested in Task 2:
   `should_list_only_toml_files_sorted`.
5. A file whose stem is `Default` stays distinct from the built-in. Tested in Task 1:
   `should_keep_file_named_default_distinct_from_builtin`.

---

### Task 1: The theme choice in settings

**Files:**
- Modify: `src/settings.rs`

**Interfaces:**
- Produces: `pub enum ThemeChoice { Builtin, File(String) }` (`Default` is `Builtin`; `Clone, Debug, PartialEq, Eq,
  Default`). It serialises as an optional string through `#[serde(from = "Option<String>", into = "Option<String>")]`.
  Also `ThemeChoice::label(&self) -> &str` (`"Default"` or the name), the field `Settings::user_theme: ThemeChoice`
  with `#[serde(skip_serializing_if = "ThemeChoice::is_builtin")]`, and `Settings::with_user_theme(&self,
  ThemeChoice) -> Self`.

- [ ] **Step 1: Write the failing tests** in `settings.rs`'s tests:
  - `should_save_and_load_user_theme`: settings with `File("nord")` saved and loaded back are equal, and the file
    contains `user_theme = "nord"`.
  - `should_leave_builtin_theme_out_of_file`: the default settings serialise without `user_theme`.
  - `should_keep_file_named_default_distinct_from_builtin`: `File("Default")` round-trips as `File("Default")`, not
    as `Builtin`.
- [ ] **Step 2:** `cargo test settings::` should fail to compile (`ThemeChoice` is missing).
- [ ] **Step 3:** Implement as in Interfaces.
- [ ] **Step 4:** `cargo test settings::` passes.
- [ ] **Step 5:** Commit: `settings: remember the picked user theme`.

### Task 2: Reading and merging theme files

**Files:**
- Create: `src/theme_config/user.rs` (declared in `src/theme_config/mod.rs` as `pub mod user;`)
- Modify: `src/theme_config/mod.rs`: expose `merged` to the submodule (`pub(crate)` or move it into `user.rs`)
- Test: in `user.rs`, with test data written to a `tempfile::tempdir()`

**Interfaces:**
- Produces, in `crate::theme_config::user`:
  - `pub const THEMES_DIR: &str = "themes";`
  - `pub fn names(dir: &Path) -> Vec<String>`: sorted stems of `*.toml` files in `dir/themes`; empty when the folder
    cannot be read.
  - `pub enum Base { Default, Themed(rune::Value), Missing }`
  - `pub fn base(dir: &Path, choice: &ThemeChoice) -> Result<Base, String>`: for `Builtin`, `Base::Default`. For a
    missing file, `Base::Missing`. Otherwise the file is read (with the 256 KB bound), parsed as TOML, merged over
    the default as `toml::Value` (tables key by key), turned into a `rune::Value` the same way `default_value()`
    does, and checked with `theme_config::parse`. Every error is prefixed `themes/<name>.toml: `.
  - `impl Base { pub fn value(self) -> rune::Value }`: `Default` and `Missing` give `theme_config::default_value()`.

- [ ] **Step 1: Write the failing tests:**
  - `should_list_only_toml_files_sorted`: with `b.toml`, `a.toml`, `notes.md` and a subfolder `c.toml/`, `names` returns `["a", "b"]`.
  - `should_merge_theme_over_default`: `[dark]\ntree = "#202124"` gives `parse(&base.value())` with
    `dark.tree == Rgb(0x202124)`, and `dark.background` keeps its default `Rgb(0x1e1f22)`.
  - `should_replace_arrays_whole`: a theme setting `[dark.terminal] ansi = [16 colours]` gives exactly those 16.
  - `should_name_file_and_key_of_unknown_theme_key`: `[dark]\ntre = "#000000"` gives an `Err` that starts with
    `themes/bad.toml: theme.dark` and contains `tre`.
  - `should_report_invalid_toml_with_file_name`: `[dark` gives an `Err` that starts with `themes/bad.toml: `.
  - `should_refuse_oversized_theme`: a file of 262_145 bytes gives an `Err` that contains `262145`.
  - `should_report_missing_theme_file`: `File("gone")` gives `Ok(Base::Missing)`.
- [ ] **Step 2:** `cargo test theme_config::user` fails.
- [ ] **Step 3:** Implement as in Interfaces.
- [ ] **Step 4:** `cargo test theme_config::` passes.
- [ ] **Step 5:** Commit: `theme: user theme files merged over the default`.

### Task 3: The script thread loads the picked theme

**Files:**
- Modify: `src/script/engine.rs` (`Engine::load`), `src/script/thread.rs` (`start`, `Runner`, `ScriptCommand`,
  `ScriptEvent::Loaded`), `src/script/fingerprint.rs`, `src/main.rs` (passes the saved choice)
- Test: `src/script/thread.rs`, `src/script/fingerprint.rs`, `src/script/engine.rs` tests

**Interfaces:**
- Consumes: Task 1 `ThemeChoice`, Task 2 `user::{base, names, Base}`.
- Produces:
  - `Engine::load(&self, dir: Option<&Path>, theme: rune::Value, generation: u64) -> Result<Loaded, String>`:
    `theme` replaces `theme_config::default_value()` as the starting `config.theme`. Existing callers and tests
    pass `default_value()`.
  - `pub fn start(dir: Option<PathBuf>, choice: ThemeChoice, log: ScriptLog, calls: UnboundedSender<Call>) -> Scripts`.
    Four parameters: put `choice` into the existing `Thread` struct, or bundle `dir` and `choice` as
    `ScriptsSetup { dir, theme }`.
  - `ScriptCommand::UseTheme(ThemeChoice)`: stores the choice and loads unconditionally.
  - `ScriptEvent::Loaded { …, themes: Vec<String> }`: `user::names(dir)` at load time; empty without a dir.
  - `fingerprint` also covers `themes/*.toml`, keyed as `themes/<file>`.
- Load flow in `Runner::load` (the decided behaviour):
  1. `user::base(dir, &choice)`. On `Missing`, log the missing line from Global Constraints and use the default.
  2. On `Err(theme_error)`, if `current` is some, call `fail(theme_error)`: the previous load stays. At start, log
     it, load with the default theme, `apply`, then send `LoadFailed` so the log opens and `script_status` reports
     the failure.
  3. Otherwise run `engine.load(dir, base.value(), …)`. In `fail`, the start-up fallback `engine.load(None, …)`
     passes the same theme value, so a broken `init.rn` still shows the picked theme.

- [ ] **Step 1: Write the failing tests** (follow `should_apply_init_theme_override` in `thread.rs` for the setup):
  - `should_start_with_picked_theme`: with `themes/nord.toml` = `[dark]\ntree = "#112233"` and start choice
    `File("nord")`, the first `Loaded` has `theme.dark.tree == Rgb(0x112233)` and `themes == ["nord"]`.
  - `should_let_init_override_picked_theme`: the same, plus an `init.rn` setting `dark.tree` to `#202124`, wins
    with `0x202124`.
  - `should_keep_init_bindings_when_theme_broken_at_start`: `nord.toml` = `[dark]\ntre = "#000000"` and an
    `init.rn` binding F5. Expect `Loaded` with the F5 binding and the default `dark.tree` (`0x191a1c`), then
    `LoadFailed` whose message starts with `themes/nord.toml: `.
  - `should_keep_previous_theme_when_theme_breaks_on_reload`: start with a good `nord.toml` (`#112233`), write a
    broken one, send `Reload`. Expect `LoadFailed` and no `Loaded`.
  - `should_load_picked_theme_on_use_theme`: start with `Builtin`, send `UseTheme(File("nord"))`. Expect `Loaded`
    with `0x112233`, without changing any file.
  - `should_use_default_when_picked_theme_missing`: start with `File("gone")`. Expect `Loaded` with the default
    tree, and the log containing `theme "gone": no file themes/gone.toml; using the default`.
  - `should_reload_when_theme_file_changes` (`fingerprint.rs`): writing `themes/a.toml` changes `fingerprint(dir)`.
- [ ] **Step 2:** `cargo test script::` fails.
- [ ] **Step 3:** Implement as in Interfaces. In `main.rs`, read the saved choice before `script::start`:
  `settings::default_path().and_then(|path| Settings::load(&path).ok()).map(|settings| settings.user_theme).unwrap_or_default()`.
- [ ] **Step 4:** `cargo test script::` and `cargo test theme_config::` pass.
- [ ] **Step 5:** Commit: `scripts: load the picked user theme under ive.rn and init.rn`.

### Task 4: Watch `themes/`, and the Appearance picker

**Files:**
- Modify: `src/script/state.rs` (keeps `themes: Vec<String>` from `Loaded`, `pub fn themes(&self) -> &[String]`,
  `pub fn use_theme(&self, ThemeChoice)` sending `UseTheme`)
- Modify: `src/workspace/scripts.rs` (`ScriptWatch` also creates and watches `<dir>/themes`; `scripts_folder_changed`
  reloads for either folder)
- Modify: `src/workspace/settings_actions.rs` (`pub fn set_user_theme(&mut self, choice: ThemeChoice, cx)`: save
  with `with_user_theme`, `use_theme`, `after_settings_change`)
- Modify: `src/settings_page/appearance.rs` (a second row, "Theme": segmented buttons `Default` and then
  `ScriptState::themes()`. When the saved `File(name)` is not in the list, it is shown muted after them. The row
  is hidden when `script::default_dir()` is `None`. Click targets `user-theme:<label>`.)
- Test: `src/script/state.rs` (`should_keep_theme_names_from_load`)

**Interfaces:**
- Consumes: Task 1 `ThemeChoice`, `Settings::with_user_theme`; Task 3 `ScriptCommand::UseTheme`, `Loaded.themes`.

- [ ] **Step 1:** Write `should_keep_theme_names_from_load` in `state.rs`: after `apply(Loaded { themes: ["a","b"], … })`,
  `themes() == ["a","b"]`.
- [ ] **Step 2:** Run it and see it fail.
- [ ] **Step 3:** Implement the files above. Let the buttons wrap (`flex_wrap`) when the names don't fit.
- [ ] **Step 4:** `cargo test` and `cargo clippy --all-targets` are clean.
- [ ] **Step 5:** Commit: `appearance: pick a user theme; themes/ reloads on change`.

### Task 5: The Scripts page as instructions, plus docs

**Files:**
- Modify: `src/settings_page/scripts.rs`: `pub fn instructions(dir: &Path, log: &Path) -> String`, the approved
  text from the spec (Goal 6, with "file, line or key") as plain text with `**` dropped, and the absolute paths
  filled in. The page renders it as paragraphs (bold headings for the "To …:" lines) between the version line and
  the existing links. A `copy-script-instructions` button writes `instructions(..)` with
  `cx.write_to_clipboard(ClipboardItem::new_string(..))`. "IVE's README" links to the URL in Global Constraints
  via `cx.open_url`.
- Modify: `assets/theme/default.toml`, header comment: also mention `themes/<name>.toml` next to `init.rn`, picked
  under Settings › Appearance.
- Modify: `README.md`: the Theme section gets a "User themes" paragraph (where, shape, merge, layering, picking,
  errors); the click-target table row "tab row, Settings" gains `user-theme:<name>` and
  `copy-script-instructions`.
- Test: `src/settings_page/scripts.rs`

- [ ] **Step 1: Write the failing tests:**
  - `should_fill_paths_into_instructions`: `instructions(Path::new("/p/ive/x"), Path::new("/s/scripts.log"))`
    contains `/p/ive/x`, `themes/<name>.toml`, `/s/scripts.log` and `script_status`.
  - `should_explain_init_override_example`: contains `config.theme.dark.tree = "#202124";`.
- [ ] **Step 2:** Run them and see them fail.
- [ ] **Step 3:** Implement as above. Get the log path from `ScriptState::log().path()`.
- [ ] **Step 4:** `cargo test`, `cargo clippy --all-targets`.
- [ ] **Step 5:** Commit: `settings: Scripts page explains where customisations go, with Copy`.

### Task 6: Check it in the running app

- [ ] Start `./run.sh -- --profile agent-themes --background --stop-processes-on-quit .`, then:
  - write `~/.config/ive/agent-themes/themes/nord.toml` with `[dark]\ntree = "#112233"`
  - through MCP: open Settings, click `settings-page:Appearance`, click `user-theme:nord`, take a screenshot (the
    tree colour changes), and confirm `settings.toml` holds `user_theme = "nord"`
  - write a broken value: `script_status` reports `failed`, with `themes/nord.toml: …`; fix it: `loaded`
  - open `settings-page:Scripts`, take a screenshot, click `copy-script-instructions`
- [ ] Quit the instance and remove `~/.config/ive/agent-themes`.
