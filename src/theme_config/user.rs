//! User themes: `themes/<name>.toml` in the profile folder, each holding only what it changes of the default theme.

use std::fs;
use std::io;
use std::path::Path;

use crate::settings::ThemeChoice;

/// The folder of user themes, next to `init.rn`.
pub const THEMES_DIR: &str = "themes";
/// A theme file this big is not a theme; refuse it rather than parse it.
const MAX_THEME_BYTES: u64 = 256 * 1024;

/// What `config.theme` starts from, before `ive.rn` and `init.rn` run.
#[derive(Debug)]
pub enum Base {
    Default,
    /// the default with the picked theme merged over it, checked
    Themed(rune::Value),
    /// the picked theme has no file; the default applies
    Missing,
}

impl Base {
    pub fn value(self) -> rune::Value {
        match self {
            Self::Themed(value) => value,
            Self::Default | Self::Missing => super::default_value(),
        }
    }
}

/// The names of the themes in `dir/themes`, sorted: the stems of its `*.toml` files; none when it cannot be read.
pub fn names(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir.join(THEMES_DIR)) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|extension| extension == "toml"))
        .filter_map(|path| Some(path.file_stem()?.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    names
}

/// The theme `choice` names, merged over the default and checked; the error starts with the file, as
/// `themes/nord.toml: theme.dark.tree: …`.
pub fn base(dir: &Path, choice: &ThemeChoice) -> Result<Base, String> {
    let ThemeChoice::File(name) = choice else {
        return Ok(Base::Default);
    };
    let file = format!("{THEMES_DIR}/{name}.toml");
    let Some(text) = read(&dir.join(&file)).map_err(|error| format!("{file}: {error}"))? else {
        return Ok(Base::Missing);
    };
    let over: toml::Table = toml::from_str(&text).map_err(|error| format!("{file}:{}: {}", crate::toml_error::line_of(error.span(), &text), error.message()))?;
    let default: toml::Table = toml::from_str(&super::DEFAULT_TEXT).expect("the bundled default theme is valid TOML");
    let value: rune::Value = toml::Value::Table(merged(default, over)).try_into().map_err(|error| format!("{file}: {error}"))?;
    super::parse(&value).map_err(|error| format!("{file}: {error}"))?;
    Ok(Base::Themed(value))
}

/// The file's text; none when it does not exist.
fn read(path: &Path) -> Result<Option<String>, String> {
    let bytes = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if bytes > MAX_THEME_BYTES {
        return Err(format!("{bytes} bytes is more than a theme's {MAX_THEME_BYTES}"));
    }
    fs::read_to_string(path).map(Some).map_err(|error| error.to_string())
}


/// `over` on top of `base`: tables key by key, anything else replaced.
fn merged(mut base: toml::Table, over: toml::Table) -> toml::Table {
    for (key, value) in over {
        let value = match (base.remove(&key), value) {
            (Some(toml::Value::Table(below)), toml::Value::Table(above)) => toml::Value::Table(merged(below, above)),
            (_, value) => value,
        };
        base.insert(key, value);
    }
    base
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::theme_config::{Rgb, parse};

    fn themes_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(THEMES_DIR)).unwrap();
        dir
    }

    fn write_theme(dir: &Path, name: &str, text: &str) {
        fs::write(dir.join(THEMES_DIR).join(format!("{name}.toml")), text).unwrap();
    }

    fn file(name: &str) -> ThemeChoice {
        ThemeChoice::File(name.to_owned())
    }

    #[test]
    fn should_list_only_toml_files_sorted() {
        // given
        let dir = themes_dir();
        write_theme(dir.path(), "b", "");
        write_theme(dir.path(), "a", "");
        fs::write(dir.path().join(THEMES_DIR).join("notes.md"), "").unwrap();
        fs::create_dir(dir.path().join(THEMES_DIR).join("c.toml")).unwrap();

        // when
        let names = names(dir.path());

        // then
        assert_eq!(names, ["a", "b"]);
    }

    #[test]
    fn should_merge_theme_over_default() {
        // given
        let dir = themes_dir();
        write_theme(dir.path(), "nord", "[dark]\ntree = \"#202124\"\n");

        // when
        let theme = parse(&base(dir.path(), &file("nord")).unwrap().value()).unwrap();

        // then
        assert_eq!(theme.dark.tree, Rgb(0x202124));
        assert_eq!(theme.dark.background, Rgb(0x1e1f22));
    }

    #[test]
    fn should_replace_arrays_whole() {
        // given
        let dir = themes_dir();
        let ansi = vec!["\"#010203\""; 16].join(", ");
        write_theme(dir.path(), "nord", &format!("[dark.terminal]\nansi = [{ansi}]\n"));

        // when
        let theme = parse(&base(dir.path(), &file("nord")).unwrap().value()).unwrap();

        // then
        assert_eq!(theme.dark.terminal.ansi, [Rgb(0x010203); 16]);
    }

    #[test]
    fn should_name_file_and_key_of_unknown_theme_key() {
        // given
        let dir = themes_dir();
        write_theme(dir.path(), "bad", "[dark]\ntre = \"#000000\"\n");

        // when
        let error = base(dir.path(), &file("bad")).err().unwrap();

        // then
        assert!(error.starts_with("themes/bad.toml: theme.dark"), "{error}");
        assert!(error.contains("tre"), "{error}");
    }

    #[test]
    fn should_report_invalid_toml_with_file_name() {
        // given
        let dir = themes_dir();
        write_theme(dir.path(), "bad", "\n[dark");

        // when
        let error = base(dir.path(), &file("bad")).err().unwrap();

        // then
        assert!(error.starts_with("themes/bad.toml:2: "), "{error}");
    }

    #[test]
    fn should_refuse_oversized_theme() {
        // given
        let dir = themes_dir();
        write_theme(dir.path(), "big", &"#".repeat(262_145));

        // when
        let error = base(dir.path(), &file("big")).err().unwrap();

        // then
        assert!(error.contains("262145"), "{error}");
    }

    #[test]
    fn should_report_missing_theme_file() {
        // given
        let dir = themes_dir();

        // when
        let base = base(dir.path(), &file("gone")).unwrap();

        // then
        assert!(matches!(base, Base::Missing), "{base:?}");
    }
}
