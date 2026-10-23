# User themes and the Scripts page as instructions

## Why

Users want to customise IVE, usually by asking a coding agent to do it. Today the Scripts page (Settings ›
Scripts) only lists the files in the profile folder, and the theme can only be changed by setting
`config.theme.…` in Rune from `init.rn`. Theme overrides should be plain TOML in the same shape as `theme.toml`,
there should be several of them to choose from, and the Scripts page should explain where everything goes. The
page must read well for a person, and also work when copied as is into an agent's prompt. It is not written only
for agents.

## Goals

1. **User themes.** Each `<profile dir>/themes/<name>.toml` is a theme, e.g.
   `~/.config/ive/default/themes/nord.toml`. It has the same shape as the read-only `theme.toml`
   (`[sizes…]`, `[panels]`, `[dark…]`, `[light…]`, the syntax tables included) but holds only what it changes.
   Every key it leaves out keeps IVE's default. One theme holds both looks, dark and light.
2. **Picking a theme.** The Appearance page keeps its System | Dark | Light row, which chooses the look. A new
   **Theme** row chooses where the look comes from: `Default` and one entry per file in `themes/`, sorted by
   name. The choice is saved in `settings.toml`.
3. **Layering.** From bottom to top: built-in default, then the picked theme file, then `ive.rn`, then
   `init.rn`. `config.theme` in scripts starts from the default with the picked theme already merged in, so
   `init.rn` can still adjust any theme and can read the theme's values.
4. **Live reload.** Saving, adding or removing a file in `themes/` reloads it the same way saving a `.rn` file
   does, and the picker's list follows. Picking a theme applies it at once.
5. **Errors.** A theme file is checked like `init.rn`'s theme: an unknown key, a wrong type, a malformed
   colour or a size out of range is reported with the file, the key's path and the value, e.g.
   `themes/nord.toml: theme.dark.tree: expected a colour "#RRGGBB", not "#12"`. The message goes to
   `scripts.log`, which IVE opens as it does for a script error, and agents read it through `script_status`.
6. **The Scripts page** becomes general instructions with this profile's absolute paths filled in, and a
   **Copy instructions** button that copies the instruction text (paths included) to the clipboard. Approved
   text, with `<dir>` and `<log>` filled in:

   > **Customising IVE**
   >
   > All your changes go in `<dir>`. IVE writes two reference files there at every start. They are
   > read-only, and editing them changes nothing:
   > - `ive.rn`: every default key binding, mouse binding and menu
   > - `theme.toml`: every default colour and size, for both the dark and light looks
   >
   > **To change keys, mouse clicks, menus or excluded folders:** write `init.rn` in that folder. IVE runs it
   > after `ive.rn`. An entry with the same scope and key replaces the default one. Look up the entry to
   > override in `ive.rn`. IVE reloads it on save.
   >
   > **To make a theme:** create `themes/<name>.toml` in that folder, with the same sections as `theme.toml`
   > (`[sizes]`, `[dark]`, `[light]`…) but only the values you change. Everything else stays at the default.
   > Pick it under Settings › Appearance › Theme. IVE reloads it on save.
   >
   > **To adjust any theme from a script:** set `config.theme.<path>` in `init.rn`. This applies on top of the
   > picked theme, for example `config.theme.dark.tree = "#202124";` for `[dark] tree`.
   >
   > **Checking it worked:** load errors, with file, line or key, and the bad value, go to `<log>`. IVE opens the log
   > when a load fails. Agents connected over MCP can read it with `script_status`. A failed load keeps the
   > previous settings.
   >
   > Full reference: "Scripting" and "Theme" in IVE's README.

   The version line stays at the top. The links to `ive.rn`, `theme.toml` and `settings.toml` stay at the bottom.
   "IVE's README" links to `https://github.com/Warsaw-Dynamics/ive#scripting`.

## Non-goals

- No themes are shipped with IVE, and nothing is imported from IntelliJ or VS Code.
- No per-project themes.
- No in-app theme editor.
- Keys, mouse bindings and menus stay in `init.rn`. Theme files hold only the theme.
- No prompt box on the Scripts page. The text is general, not a template with the user's request in it.

## Anti-goals

- A broken theme file must not cost the user their `init.rn` key bindings.
- Editing the read-only `theme.toml` must still change nothing. Theme files live only in `themes/`.
- Do not mutate the user's settings file when the picked theme's file disappears. The saved choice stays, so the
  theme comes back when the file does.

## Behaviour details (decided)

- **Merge:** tables merge key by key, recursively. Any other value, arrays included (`glow`, `ansi`,
  `checkout_hues`), replaces the default's value whole.
- **A broken theme file** (unreadable, not valid TOML, or failing the check) fails the load like a broken
  `init.rn`: the error is logged with the file name, the log is shown, and `LoadFailed` is sent, so
  `script_status` reports it as `failed` with the error. *(Our call, matching "a failed load keeps the previous
  settings".)*
  - On a reload, the previous load stays: the theme it had and its bindings.
  - At start there is no previous load. IVE loads again without the theme file, so `ive.rn` and `init.rn` still
    apply over the default theme. Only when that also fails does it fall back to `ive.rn` alone, as it does today.
    This keeps key bindings, as the anti-goal requires.
- **A broken `init.rn`** with a good picked theme: the start-up fallback (`ive.rn` alone) still uses the picked
  theme. On a reload, the previous load stays as today, so picking a theme while `init.rn` is broken applies once
  `init.rn` loads again. *(Our call.)*
- **Error positions:** theme errors name the file, the key's path and the value. They have no line numbers,
  because the check works on parsed values. The page text says "with file, line or key, and the bad value"
  instead of "with file, line and the bad value". *(Our call: a small change to the approved wording.)*
- **A picked theme whose file is missing:** logged once per load (`theme "nord": no file themes/nord.toml; using
  the default`), the default is used, and the setting is left as it is. The picker shows the saved name as
  missing (muted) so it can still be seen and changed.
- **Size bound:** a theme file larger than 256 KB is refused with its size in the message.
- **Names:** a theme's name is the file stem. Only `*.toml` files directly in `themes/` count. `Default` is the
  built-in and cannot be shadowed: a file `themes/Default.toml` is listed under its own name, and choosing it
  stores that name, not the built-in. *(Our call: settings store the built-in as the absence of a name.)*
- **Folder:** IVE creates `themes/` next to `init.rn` at start, as it creates the profile folder, so it is
  there to put files in. Without an OS config folder there are no user themes and the Theme row is not shown.
- **Click targets** for MCP: `user-theme:Default`, `user-theme:<name>`, `copy-script-instructions`. The
  existing `theme:<label>`, `open-ive-rn`, `open-theme-toml` and `open-settings-toml` stay.
- **`theme.toml`'s header comment** mentions `themes/<name>.toml` next to `init.rn`.
- **README:** the Theme section describes user themes. The Settings and click-target tables gain the new
  targets.

## Left to the builder

- How the picked name reaches the script thread (a new command, a field of `Reload`, a shared value) and how the
  list of theme names reaches the settings page (with the `Loaded` event or read on render).
- How `settings.toml` stores the choice, within the coding guideline (no optional wrapper in a field where
  avoidable).
- The picker's widget: segmented buttons like the existing row, wrapping when there are many, are fine.
- Watching `themes/`: the profile folder is watched non-recursively today, and the fingerprint covers only `*.rn`
  files. Both must cover `themes/*.toml`.
- Tests for: merge, a broken theme keeping `init.rn`'s bindings, a missing picked theme, `init.rn` overriding a
  theme value, the fingerprint seeing theme files, the settings round trip, and the instruction text having the
  real paths.
