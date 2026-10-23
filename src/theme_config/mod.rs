//! The theme `config.theme` holds: the bundled default, and the typed, checked form IVE draws with.

mod sizes;
pub mod user;
mod values;

use std::collections::BTreeMap;
use std::sync::LazyLock;

use gpui::Global;
use serde::Deserialize;

use crate::colors::{ColorKey, RawEntry};

pub use sizes::{Sizes, TerminalSizes};
pub use values::{Factor, FontPx, Fraction, Height, Px, RootPx, Rgb, Rgba, RowStyle, Weight};

/// IVE's default theme without its syntax colours, which come from the built-in schemes.
const DEFAULT_TOML: &str = include_str!("../../assets/theme/default.toml");
const DARK_SCHEME: &str = include_str!("../../assets/schemes/dark.toml");
const LIGHT_SCHEME: &str = include_str!("../../assets/schemes/light.toml");

/// IVE's whole default theme as TOML, syntax colours included; `ive.rn` and then `init.rn` change it as
/// `config.theme`.
pub static DEFAULT_TEXT: LazyLock<String> =
    LazyLock::new(|| format!("{DEFAULT_TOML}{}{}", syntax_tables("dark", DARK_SCHEME), syntax_tables("light", LIGHT_SCHEME)));

/// A scheme file's `["KEY"]` tables as `[<appearance>.syntax."KEY"]`, its comments dropped.
fn syntax_tables(appearance: &str, scheme: &str) -> String {
    let tables = scheme.lines().filter(|line| !line.starts_with('#')).map(|line| match line.strip_prefix('[') {
        Some(key) => format!("[{appearance}.syntax.{key}"),
        None => line.to_owned(),
    });
    format!("\n[{appearance}.syntax]\n{}\n", tables.collect::<Vec<_>>().join("\n"))
}

/// A whole theme, checked: what `config.theme` held after `init.rn`.
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeSpec {
    pub sizes: Sizes,
    pub dark: Palette,
    pub light: Palette,
    /// the palettes a panel draws with while it does not have the keyboard
    pub unfocused: Unfocused,
    pub panels: Panels,
}

impl Global for ThemeSpec {}

/// Each appearance's palette with its `unfocused` table merged over it.
#[derive(Debug, Clone, PartialEq)]
pub struct Unfocused {
    pub dark: Palette,
    pub light: Palette,
}

/// The theme besides its sizes, which are read first, with their root size.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSpec {
    dark: Palette,
    light: Palette,
    panels: Panels,
}

/// The colours of one appearance.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Palette {
    pub background: Rgb,
    pub sidebar: Rgb,
    pub tree: Rgb,
    pub border: Rgb,
    pub text: Rgb,
    pub muted: Rgb,
    /// the one-colour icons drawn as masks, such as the projects panel's header buttons
    pub icon: Rgb,
    pub accent: Rgb,
    pub accent_hover: Rgb,
    pub on_accent: Rgb,
    pub selected: Rgb,
    pub hover: Rgb,
    pub error: Rgb,
    pub diff_added: Rgba,
    pub diff_removed: Rgba,
    /// the find box's matches, and the current one
    pub find_match: Rgba,
    pub find_current: Rgba,
    pub mark_added: Rgb,
    pub mark_removed: Rgb,
    pub mark_modified: Rgb,
    /// the editor's change markers: bars on added and changed lines, triangles where lines were deleted
    pub change_added: Rgb,
    pub change_modified: Rgb,
    pub change_deleted: Rgb,
    pub close_hover: Rgb,
    pub scrollbar_thumb: Rgba,
    pub scrollbar_thumb_hover: Rgba,
    pub tab_active: Rgb,
    pub tab_active_border: Rgb,
    pub scrim: Rgba,
    pub shadow: Rgba,
    pub frame: Rgb,
    pub checkout_hues: [Rgb; 8],
    pub glow: [Rgb; 9],
    pub terminal: TerminalColors,
    pub git: GitColors,
    pub tree_states: TreeStates,
    pub virtual_folders: VirtualFolders,
    pub menu: MenuColors,
    /// colour key id → entry, over the built-in scheme
    pub syntax: BTreeMap<String, RawEntry>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalColors {
    /// black, red, green, yellow, blue, magenta, cyan, white, then the same eight bright
    pub ansi: [Rgb; 16],
    pub foreground: Rgb,
    pub background: Rgb,
    pub cursor: Rgb,
    pub selection: Rgb,
}

/// File name colours in the tree by git status.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitColors {
    pub modified: Rgb,
    pub added: Rgb,
    pub deleted: Rgb,
    pub unversioned: Rgb,
    pub renamed: Rgb,
    pub conflicted: Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeStates {
    pub hover: RowStyle,
    pub excluded: RowStyle,
    pub ignored: RowStyle,
    pub active: RowStyle,
    pub selected: RowStyle,
    pub secondary: RowStyle,
}

/// The menus of tabs and tree rows.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuColors {
    pub background: Rgb,
    pub border: Rgb,
    pub text: Rgb,
    /// a line's key, on its right
    pub shortcut: Rgb,
    /// a line that does not apply to what was clicked
    pub disabled: Rgb,
    /// the line under the pointer or the arrow keys
    pub highlight: Rgb,
    pub highlight_text: Rgb,
    pub separator: Rgb,
}

/// The chips of the tree's rows that are not on disk.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VirtualFolders {
    /// unused since `@run` went; kept so themes that set it still load
    pub chip: RowStyle,
    /// `@hidden`, weaker: it holds real files
    pub hidden: RowStyle,
}

/// Rounded panels on the frame colour, and the project glow.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panels {
    pub gap: Px,
    pub radius: Px,
    pub glow_peak: Px,
    pub glow_width: Px,
    pub glow_height: Px,
    pub glow_blend: Fraction,
}

impl Panels {
    /// Whether the tree and the editor are drawn as separate panels on the frame; a gap and radius of 0 is the flat
    /// layout.
    pub fn shown(&self) -> bool {
        self.gap.0 > 0. || self.radius.0 > 0.
    }
}

/// `config.theme` as scripts get it before they change anything.
pub fn default_value() -> rune::Value {
    toml::from_str(&DEFAULT_TEXT).expect("the bundled default theme is valid TOML")
}

/// The theme a script left in `config.theme`; the error names the key path and the offending value.
pub fn parse(value: &rune::Value) -> Result<ThemeSpec, String> {
    let mut json = serde_json::to_value(value).map_err(|error| format!("theme: {error}"))?;
    let sizes = json.as_object_mut().and_then(|theme| theme.remove("sizes")).ok_or("theme.sizes: missing")?;
    let sizes = parse_sizes(sizes)?;
    let over = (take_unfocused(&mut json, "dark")?, take_unfocused(&mut json, "light")?);
    let palettes = (json["dark"].clone(), json["light"].clone());
    let rest: RawSpec = values::with_rem(None, || serde_path_to_error::deserialize(json)).map_err(|error| format!("theme.{}: {}", error.path(), error.inner()))?;
    check_syntax("dark", &rest.dark.syntax)?;
    check_syntax("light", &rest.light.syntax)?;
    let unfocused = Unfocused { dark: unfocused_palette("dark", palettes.0, over.0)?, light: unfocused_palette("light", palettes.1, over.1)? };
    Ok(ThemeSpec { sizes, dark: rest.dark, light: rest.light, unfocused, panels: rest.panels })
}

/// The `unfocused` table of `appearance`'s palette, taken out of `json` (empty when there is none).
fn take_unfocused(json: &mut serde_json::Value, appearance: &str) -> Result<serde_json::Value, String> {
    let over = json.get_mut(appearance).and_then(serde_json::Value::as_object_mut).and_then(|palette| palette.remove("unfocused"));
    let over = over.unwrap_or_else(|| serde_json::json!({}));
    if over.get("syntax").is_some() {
        return Err(format!("theme.{appearance}.unfocused.syntax: syntax colours do not change with focus"));
    }
    Ok(over)
}

/// `palette` with `over` merged over it, checked as a palette of its own.
fn unfocused_palette(appearance: &str, palette: serde_json::Value, over: serde_json::Value) -> Result<Palette, String> {
    let merged = merged(palette, over);
    values::with_rem(None, || serde_path_to_error::deserialize(merged)).map_err(|error| format!("theme.{appearance}.unfocused.{}: {}", error.path(), error.inner()))
}

/// `over` on top of `base`: tables key by key, anything else replaced.
fn merged(base: serde_json::Value, over: serde_json::Value) -> serde_json::Value {
    match (base, over) {
        (serde_json::Value::Object(mut base), serde_json::Value::Object(over)) => {
            for (key, value) in over {
                let below = base.remove(&key).unwrap_or(serde_json::Value::Null);
                base.insert(key, merged(below, value));
            }
            serde_json::Value::Object(base)
        }
        (_, over) => over,
    }
}

/// `sizes` with every value written in rem resolved against its `rem`, read first.
fn parse_sizes(sizes: serde_json::Value) -> Result<Sizes, String> {
    let rem = match sizes.get("rem") {
        Some(rem) => RootPx::deserialize(rem).map_err(|error| format!("theme.sizes.rem: {error}"))?,
        None => RootPx::default(),
    };
    values::with_rem(Some(rem.0), || serde_path_to_error::deserialize(sizes)).map_err(|error| format!("theme.sizes.{}: {}", error.path(), error.inner()))
}

/// Every syntax entry names a known colour key and is a valid entry.
fn check_syntax(appearance: &str, syntax: &BTreeMap<String, RawEntry>) -> Result<(), String> {
    for (id, entry) in syntax {
        let path = format!("theme.{appearance}.syntax.\"{id}\"");
        if ColorKey::by_id(id).is_none() {
            return Err(format!("{path}: unknown colour key"));
        }
        entry.to_entry(&path)?;
    }
    Ok(())
}

/// The bundled theme, checked.
pub fn defaults() -> ThemeSpec {
    parse(&default_value()).expect("the bundled default theme is valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Appearance;

    /// The default theme with the value at `path` replaced, as `init.rn` would leave it.
    fn with(path: &[&str], value: impl Into<serde_json::Value>) -> rune::Value {
        let mut json = serde_json::to_value(default_value()).unwrap();
        let (last, parents) = path.split_last().unwrap();
        let table = parents.iter().fold(&mut json, |table, key| &mut table[*key]);
        table[*last] = value.into();
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn should_match_todays_dark_colours() {
        // when
        let spec = defaults();

        // then
        assert_eq!(spec.dark.background, Rgb(0x1e1f22));
        assert_eq!(spec.dark.tree, Rgb(0x191a1c));
        assert_eq!(spec.dark.diff_added, Rgba(0x39ac6024));
        assert_eq!((spec.dark.find_match, spec.light.find_current), (Rgba(0xffc43c47), Rgba(0xf2a33acc)));
        assert_eq!(spec.light.text, Rgb(0x000000));
    }

    #[test]
    fn should_match_every_old_theme_colour() {
        // given
        let old = [(OLD_DARK, Appearance::Dark), (OLD_LIGHT, Appearance::Light)];

        // when
        let spec = defaults();

        // then
        for (old, appearance) in old {
            let palette = match appearance {
                Appearance::Dark => &spec.dark,
                Appearance::Light => &spec.light,
            };
            assert_eq!(palette_colours(palette), old.colours(), "{appearance:?}");
            assert_eq!(palette.terminal.ansi.map(|colour| colour.0), old.terminal_ansi, "{appearance:?}");
            assert_eq!(palette.checkout_hues.map(|colour| colour.0), old.checkout_hues, "{appearance:?}");
        }
    }

    #[test]
    fn should_default_menu_colours_to_intellij_popups() {
        // when
        let spec = defaults();

        // then
        let (dark, light) = (spec.dark.menu, spec.light.menu);
        assert_eq!((dark.background, dark.border, dark.separator), (Rgb(0x2b2d30), Rgb(0x43454a), Rgb(0x43454a)));
        assert_eq!((dark.text, dark.shortcut, dark.disabled), (Rgb(0xdfe1e5), Rgb(0x868a91), Rgb(0x6f737a)));
        assert_eq!((dark.highlight, dark.highlight_text), (Rgb(0x2e436e), Rgb(0xdfe1e5)));
        assert_eq!((light.background, light.highlight, light.shortcut), (Rgb(0xffffff), Rgb(0xd5e1ff), Rgb(0x818594)));
    }

    #[test]
    fn should_default_to_rounded_panels() {
        // when
        let spec = defaults();

        // then
        assert_eq!((spec.panels.gap, spec.panels.radius), (Px(5.), Px(8.)));
    }

    #[test]
    fn should_default_icon_colour_to_the_icons_own() {
        // when
        let spec = defaults();

        // then
        assert_eq!((spec.dark.icon, spec.light.icon), (Rgb(0xced0d6), Rgb(0x6c707e)));
    }

    #[test]
    fn should_reject_unknown_key_with_its_path() {
        // given
        let value = with(&["dark", "tre"], "#202124");

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert!(error.starts_with("theme.dark.tre:"), "{error}");
    }

    #[test]
    fn should_reject_short_colour() {
        // given
        let value = with(&["dark", "tree"], "#12");

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert_eq!(error, r##"theme.dark.tree: expected a colour "#RRGGBB", not "#12""##);
    }

    #[test]
    fn should_reject_opaque_colour_where_alpha_is_expected() {
        // given
        let value = with(&["dark", "diff_added"], "#39AC60");

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert_eq!(error, r##"theme.dark.diff_added: expected a colour "#RRGGBBAA", not "#39AC60""##);
    }

    #[test]
    fn should_reject_out_of_range_size() {
        // given
        let value = with(&["panels", "gap"], 10_000.0);

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert!(error.starts_with("theme.panels.gap:") && error.contains("10000"), "{error}");
    }

    #[test]
    fn should_reject_zero_row_height() {
        // given
        let values = [
            with(&["sizes", "tree", "row_height"], 0.0),
            with(&["sizes", "diff", "row_height"], 0.0),
            with(&["sizes", "git", "row_height"], 0.0),
        ];

        // when
        let errors: Vec<String> = values.iter().map(|value| parse(value).unwrap_err()).collect();

        // then
        assert!(errors[0].starts_with("theme.sizes.tree.row_height:"), "{errors:?}");
        assert!(errors[1].starts_with("theme.sizes.diff.row_height:"), "{errors:?}");
        assert!(errors[2].starts_with("theme.sizes.git.row_height:"), "{errors:?}");
    }

    #[test]
    fn should_reject_huge_text_size_everywhere() {
        // given
        let values = [with(&["sizes", "git", "text_size"], 500.0), with(&["sizes", "settings", "text_size"], 500.0)];

        // when
        let errors: Vec<String> = values.iter().map(|value| parse(value).unwrap_err()).collect();

        // then
        assert!(errors[0].starts_with("theme.sizes.git.text_size:") && errors[0].contains("500"), "{errors:?}");
        assert!(errors[1].starts_with("theme.sizes.settings.text_size:"), "{errors:?}");
    }

    #[test]
    fn should_reject_unknown_weight() {
        // given
        let value = with(&["dark", "tree_states", "selected"], serde_json::json!({ "weight": "heavy" }));

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert!(error.starts_with("theme.dark.tree_states.selected.weight:") && error.contains("heavy"), "{error}");
    }

    #[test]
    fn should_reject_non_bool_italic() {
        // given
        let value = with(&["light", "virtual_folders", "chip"], serde_json::json!({ "italic": "yes" }));

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert!(error.starts_with("theme.light.virtual_folders.chip.italic:"), "{error}");
    }

    #[test]
    fn should_reject_unknown_virtual_folder() {
        // given
        let value = with(&["dark", "virtual_folders", "run"], serde_json::json!({}));

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert!(error.starts_with("theme.dark.virtual_folders.run: unknown field"), "{error}");
    }

    #[test]
    fn should_seed_syntax_from_builtin_scheme() {
        // when
        let spec = defaults();

        // then
        assert_eq!(spec.dark.syntax["CSS.COLOR"].foreground.as_deref(), Some("56A8F5"));
        assert_eq!(spec.light.syntax["BASH.EXTERNAL_COMMAND"].foreground.as_deref(), Some("0073BF"));
    }

    #[test]
    fn should_reject_unknown_syntax_key() {
        // given
        let value = with(&["dark", "syntax", "FOO.BAR"], serde_json::json!({ "bold": true }));

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert_eq!(error, r#"theme.dark.syntax."FOO.BAR": unknown colour key"#);
    }

    #[test]
    fn should_reject_bad_syntax_entry() {
        // given
        let value = with(&["light", "syntax", "CSS.COLOR"], serde_json::json!({ "foreground": "12" }));

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert!(error.starts_with(r#"theme.light.syntax."CSS.COLOR""#) && error.contains("\"12\""), "{error}");
    }

    #[test]
    fn should_document_only_existing_keys() {
        // given
        let readme = include_str!("../../README.md");
        let defaults = serde_json::to_value(default_value()).unwrap();

        // when
        let paths: Vec<&str> = readme
            .match_indices("config.theme.")
            .map(|(at, prefix)| {
                let rest = &readme[at + prefix.len()..];
                let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.')).unwrap_or(rest.len());
                let path = rest[..end].trim_end_matches('.');
                // `….push(…)` calls a method of a list: the path ends before it
                match rest[end..].starts_with('(') {
                    true => path.rsplit_once('.').map_or(path, |(list, _)| list),
                    false => path,
                }
            })
            .collect();
        let missing: Vec<&&str> = paths.iter().filter(|path| path.split('.').try_fold(&defaults, |value, key| value.get(key)).is_none()).collect();

        // then
        assert!(paths.len() >= 5, "the README documents config.theme: {paths:?}");
        assert!(missing.is_empty(), "documented but not in the default theme: {missing:?}");
    }

    #[test]
    fn should_accept_changed_value() {
        // given
        let value = with(&["dark", "tree"], "#202124");

        // when
        let spec = parse(&value).unwrap();

        // then
        assert_eq!(spec.dark.tree, Rgb(0x202124));
    }

    /// The default theme with each `(path, value)` replaced.
    fn with_each(changes: &[(&[&str], serde_json::Value)]) -> rune::Value {
        let mut json = serde_json::to_value(default_value()).unwrap();
        for (path, value) in changes {
            let (last, parents) = path.split_last().unwrap();
            let table = parents.iter().fold(&mut json, |table, key| &mut table[*key]);
            table[*last] = value.clone();
        }
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn should_read_rem_and_px_sizes() {
        // given
        let value = with_each(&[(&["sizes", "tree", "row_height"], "1.75rem".into()), (&["sizes", "tree", "text_size"], "15px".into())]);

        // when
        let spec = parse(&value).unwrap();

        // then
        assert_eq!((spec.sizes.tree.row_height, spec.sizes.tree.text_size), (Height(28.), FontPx(15.)));
    }

    #[test]
    fn should_scale_rem_sizes_by_root_and_round() {
        // given
        let value = with_each(&[
            (&["sizes", "rem"], 18.0.into()),
            (&["sizes", "tree", "row_height"], "1.5rem".into()),
            (&["sizes", "tree", "text_size"], "0.8125rem".into()),
        ]);

        // when
        let spec = parse(&value).unwrap();

        // then
        assert_eq!((spec.sizes.tree.row_height, spec.sizes.tree.text_size), (Height(27.), FontPx(15.)));
    }

    #[test]
    fn should_keep_a_tiny_rem_size_at_one_pixel() {
        // given
        let value = with_each(&[
            (&["sizes", "rem"], 8.0.into()),
            (&["sizes", "tree", "icon_gap"], "0.0625rem".into()),
            (&["sizes", "tree", "tag_gap"], "0rem".into()),
        ]);

        // when
        let spec = parse(&value).unwrap();

        // then
        assert_eq!((spec.sizes.tree.icon_gap, spec.sizes.tree.tag_gap), (Px(1.), Px(0.)));
    }

    #[test]
    fn should_default_root_size_when_init_removes_it() {
        // given
        let mut json = serde_json::to_value(default_value()).unwrap();
        json["sizes"].as_object_mut().unwrap().remove("rem");
        let value: rune::Value = serde_json::from_value(json).unwrap();

        // when
        let spec = parse(&value).unwrap();

        // then
        assert_eq!(spec.sizes.rem, RootPx(16.));
    }

    #[test]
    fn should_reject_unknown_unit() {
        // given
        let value = with(&["sizes", "tree", "text_size"], "1.2em");

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert!(error.starts_with("theme.sizes.tree.text_size:"), "{error}");
        assert!(error.contains(r#""13px" or "0.8125rem", not "1.2em""#), "{error}");
    }

    #[test]
    fn should_name_rem_form_in_range_error() {
        // given
        let value = with(&["sizes", "tree", "text_size"], "7.5rem");

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert!(error.contains(r#"not 120 ("7.5rem" at rem 16)"#), "{error}");
    }

    #[test]
    fn should_reject_root_size_out_of_range() {
        // given
        let value = with(&["sizes", "rem"], 40.0);

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert_eq!(error, "theme.sizes.rem: expected a root size from 8 to 32 px, not 40");
    }

    #[test]
    fn should_reject_rem_outside_sizes() {
        // given
        let (gap, opacity) = (with(&["panels", "gap"], "1rem"), with(&["sizes", "tabs", "drag_chip_opacity"], "0.5rem"));

        // when
        let errors = (parse(&gap).unwrap_err(), parse(&opacity).unwrap_err());

        // then
        assert!(errors.0.starts_with("theme.panels.gap:"), "{}", errors.0);
        assert!(errors.1.starts_with("theme.sizes.tabs.drag_chip_opacity:"), "{}", errors.1);
    }

    /// Writes the default sizes as resolved now, for `should_resolve_default_sizes_to_todays_pixels`; run once by
    /// hand, before the defaults change.
    #[test]
    #[ignore]
    fn write_default_sizes_golden() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/theme_config/testdata/default_sizes_px.txt");
        std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
        std::fs::write(path, format!("{:#?}\n", defaults().sizes)).unwrap();
    }

    #[test]
    fn should_resolve_default_sizes_to_todays_pixels() {
        // when
        let sizes = format!("{:#?}\n", defaults().sizes);

        // then
        assert_eq!(sizes, include_str!("testdata/default_sizes_px.txt"));
    }

    #[test]
    fn should_write_text_related_defaults_in_rem() {
        // given
        let toml: toml::Value = toml::from_str(DEFAULT_TOML).unwrap();

        // when
        let (tree, editor) = (&toml["sizes"]["tree"], &toml["sizes"]["editor"]);

        // then
        assert_eq!(tree["text_size"].as_str(), Some("0.8125rem"));
        assert_eq!(tree["row_height"].as_str(), Some("1.5rem"));
        assert_eq!(editor["text_size"].as_float(), Some(14.0));
        assert_eq!(tree["border"].as_float(), Some(1.0));
    }

    #[test]
    fn should_merge_unfocused_over_palette() {
        // when
        let spec = defaults();

        // then
        assert_eq!(spec.unfocused.dark.selected, Rgb(0x33353b));
        assert_eq!(spec.unfocused.dark.tab_active, Rgb(0x26282c));
        assert_eq!(spec.unfocused.dark.tab_active_border, Rgb(0x40434a));
        assert_eq!(spec.unfocused.dark.tree_states.selected.background, Some(Rgba(0x33353bff)));
        assert_eq!(spec.unfocused.dark.tree_states.hover, spec.dark.tree_states.hover);
        assert_eq!(spec.unfocused.dark.background, spec.dark.background);
        assert_eq!(spec.unfocused.light.selected, Rgb(0xe9eaee));
        assert_eq!(spec.unfocused.light.tab_active_border, Rgb(0xd1d3d9));
    }

    #[test]
    fn should_use_palette_when_unfocused_table_is_missing() {
        // given
        let mut json = serde_json::to_value(default_value()).unwrap();
        json["dark"].as_object_mut().unwrap().remove("unfocused");
        let value: rune::Value = serde_json::from_value(json).unwrap();

        // when
        let spec = parse(&value).unwrap();

        // then
        assert_eq!(spec.unfocused.dark, spec.dark);
    }

    #[test]
    fn should_reject_unknown_unfocused_key() {
        // given
        let value = with(&["dark", "unfocused", "colour"], "#000000");

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert!(error.starts_with("theme.dark.unfocused.colour:"), "{error}");
    }

    #[test]
    fn should_reject_bad_unfocused_value_with_its_path() {
        // given
        let value = with(&["dark", "unfocused", "tree_states", "selected", "weight"], "heavy");

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert!(error.starts_with("theme.dark.unfocused.tree_states.selected.weight:"), "{error}");
    }

    #[test]
    fn should_reject_syntax_in_unfocused() {
        // given
        let value = with(&["dark", "unfocused", "syntax"], serde_json::json!({}));

        // when
        let error = parse(&value).unwrap_err();

        // then
        assert_eq!(error, "theme.dark.unfocused.syntax: syntax colours do not change with focus");
    }

    fn palette_colours(palette: &Palette) -> Vec<u32> {
        vec![
            palette.background.0, palette.sidebar.0, palette.tree.0, palette.border.0, palette.text.0, palette.muted.0,
            palette.accent.0, palette.accent_hover.0, palette.on_accent.0, palette.selected.0, palette.hover.0,
            palette.error.0, palette.diff_added.0, palette.diff_removed.0, palette.mark_added.0, palette.mark_removed.0,
            palette.mark_modified.0, palette.close_hover.0, palette.scrollbar_thumb.0, palette.scrollbar_thumb_hover.0,
            palette.tab_active.0, palette.tab_active_border.0, palette.terminal.foreground.0,
            palette.terminal.background.0, palette.terminal.cursor.0, palette.terminal.selection.0,
        ]
    }

    /// The colours `src/theme.rs` hard-coded before the theme became configurable, as they were at 51b3804b8a.
    struct OldTheme {
        colours: [u32; 26],
        terminal_ansi: [u32; 16],
        checkout_hues: [u32; 8],
    }

    impl OldTheme {
        fn colours(&self) -> Vec<u32> {
            self.colours.to_vec()
        }
    }

    const OLD_DARK: OldTheme = OldTheme {
        colours: [
            0x1e1f22, 0x2b2d30, 0x191a1c, 0x393b40, 0xdfe1e5, 0x868a91, 0x3574f0, 0x4682fa, 0xffffff, 0x2e436e,
            0x393b40, 0xf75464, 0x39ac6024, 0xdd3c4924, 0x5fb865, 0xf75464, 0x548af7, 0xe81123, 0xffffff33,
            0xffffff59, 0x25324d, 0x35538f, 0xdfe1e5, 0x1e1f22, 0xced0d6, 0x214283,
        ],
        terminal_ansi: [
            0x2b2d30, 0xf75464, 0x5fb865, 0xe0bb65, 0x548af7, 0xc77dbb, 0x2aacb8, 0xdfe1e5, 0x6f737a, 0xff7b85, 0x89d18f,
            0xf2c55c, 0x7aa2ff, 0xd99ddb, 0x56c7d4, 0xffffff,
        ],
        checkout_hues: [0x498fdf, 0x49df8f, 0xdf9949, 0xb249df, 0xdf495d, 0x49d0df, 0xdfdf49, 0xdf49ad],
    };

    const OLD_LIGHT: OldTheme = OldTheme {
        colours: [
            0xffffff, 0xf7f8fa, 0xf7f8fa, 0xebecf0, 0x000000, 0x818594, 0x3574f0, 0x315fbd, 0xffffff, 0xd5e1ff,
            0xdfe1e5, 0xdb3b4b, 0x39ac602e, 0xdd3c492e, 0x067d17, 0xc7222d, 0x0033b3, 0xe81123, 0x00000026,
            0x00000040, 0xe3ecfd, 0xa8c3f5, 0x000000, 0xffffff, 0x000000, 0xa6d2ff,
        ],
        terminal_ansi: [
            0x000000, 0xc7222d, 0x067d17, 0x9e880d, 0x0033b3, 0x871094, 0x00627a, 0xc9ccd6, 0x818594, 0xe55765, 0x3e8f4a,
            0xc29e00, 0x4a86e8, 0xb05fb5, 0x1f9cad, 0xffffff,
        ],
        checkout_hues: [0x226dc3, 0x22c36d, 0xc37822, 0x9322c3, 0xc32238, 0x22b3c3, 0xc3c322, 0xc3228e],
    };
}
