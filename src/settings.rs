use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use codeview::panel::DiffLayout;
use gpui::Global;
use serde::{Deserialize, Serialize};

use crate::overview::file_list::FileOrder;

/// A settings file this big is not something IVE wrote; refuse it rather than parse it.
const MAX_SETTINGS_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Dark,
    Light,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::System, Self::Dark, Self::Light];

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "Same as OS",
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }
}

/// Which theme the looks come from: IVE's own, or a file `themes/<name>.toml` in the profile folder.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(from = "Option<String>", into = "Option<String>")]
pub enum ThemeChoice {
    #[default]
    Builtin,
    File(String),
}

impl ThemeChoice {
    pub fn is_builtin(&self) -> bool {
        *self == Self::Builtin
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Builtin => "Default",
            Self::File(name) => name,
        }
    }
}

impl From<Option<String>> for ThemeChoice {
    fn from(name: Option<String>) -> Self {
        name.map_or(Self::Builtin, Self::File)
    }
}

impl From<ThemeChoice> for Option<String> {
    fn from(choice: ThemeChoice) -> Self {
        match choice {
            ThemeChoice::Builtin => None,
            ThemeChoice::File(name) => Some(name),
        }
    }
}

/// Whether a release build updates itself from GitHub Releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoUpdate {
    #[default]
    On,
    Off,
}

impl AutoUpdate {
    pub const ALL: [Self; 2] = [Self::On, Self::Off];

    pub fn is_on(&self) -> bool {
        *self == Self::On
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::On => "On",
            Self::Off => "Off",
        }
    }
}

/// User settings, stored as TOML; keys missing from the file take their defaults.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: ThemePreference,
    #[serde(skip_serializing_if = "ThemeChoice::is_builtin")]
    pub user_theme: ThemeChoice,
    /// project tree width in pixels, once the user resized it; IVE fits it to the window
    pub sidebar_width: Option<u32>,
    /// the diff layout the user last switched to: new diff and commit tabs open in it
    #[serde(skip_serializing_if = "is_inline")]
    pub diff_layout: DiffLayout,
    /// how the git views list changed files
    #[serde(skip_serializing_if = "FileOrder::is_default")]
    pub git_files: FileOrder,
    #[serde(skip_serializing_if = "AutoUpdate::is_on")]
    pub auto_update: AutoUpdate,
}

impl Settings {
    pub fn with_theme(&self, theme: ThemePreference) -> Self {
        Self { theme, ..self.clone() }
    }

    pub fn with_user_theme(&self, user_theme: ThemeChoice) -> Self {
        Self { user_theme, ..self.clone() }
    }

    pub fn with_sidebar_width(&self, width: Option<u32>) -> Self {
        Self { sidebar_width: width, ..self.clone() }
    }

    pub fn with_git_files(&self, git_files: FileOrder) -> Self {
        Self { git_files, ..self.clone() }
    }

    pub fn with_auto_update(&self, auto_update: AutoUpdate) -> Self {
        Self { auto_update, ..self.clone() }
    }

    pub fn with_diff_layout(&self, diff_layout: DiffLayout) -> Self {
        Self { diff_layout, ..self.clone() }
    }

    /// Reads settings from `path`; a missing file means defaults.
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        let bytes = match fs::metadata(path) {
            Ok(metadata) => metadata.len(),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => return Err(SettingsError::io(path, source)),
        };
        if bytes > MAX_SETTINGS_BYTES {
            return Err(SettingsError::TooLarge { path: path.to_path_buf(), bytes });
        }
        let text = fs::read_to_string(path).map_err(|source| SettingsError::io(path, source))?;
        let invalid = |reason: String| SettingsError::Invalid { path: path.to_path_buf(), reason };
        let settings: Self = toml::from_str(&text).map_err(|error| invalid(format!("line {}: {}", crate::toml_error::line_of(error.span(), &text), error.message().trim_end())))?;
        Ok(settings)
    }

    /// Writes settings to `path`, creating its folder; replaces the file atomically.
    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|source| SettingsError::io(dir, source))?;
        }
        let text = toml::to_string(self)
            .map_err(|error| SettingsError::Invalid { path: path.to_path_buf(), reason: error.to_string() })?;
        let staging = path.with_extension("toml.tmp");
        fs::write(&staging, text).map_err(|source| SettingsError::io(&staging, source))?;
        crate::retry::replace_file(&staging, path).map_err(|source| SettingsError::io(path, source))
    }
}

fn is_inline(layout: &DiffLayout) -> bool {
    *layout == DiffLayout::Inline
}

/// The settings in effect, and where they are saved (none when the OS has no config folder).
pub struct SettingsStore {
    current: Settings,
    path: Option<PathBuf>,
    /// why the file did not load; while set, saving would overwrite the user's file with defaults, so it is refused
    load_error: Option<String>,
}

impl Global for SettingsStore {}

impl SettingsStore {
    pub fn new(current: Settings, path: Option<PathBuf>) -> Self {
        Self { current, path, load_error: None }
    }

    /// The settings saved at `path`; defaults when there is no file, or when it does not load (see `load_error`).
    pub fn open(path: Option<PathBuf>) -> Self {
        match path.as_deref().map_or(Ok(Settings::default()), Settings::load) {
            Ok(current) => Self::new(current, path),
            Err(error) => Self { current: Settings::default(), path, load_error: Some(error.to_string()) },
        }
    }

    pub fn load_error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }

    pub fn current(&self) -> &Settings {
        &self.current
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Applies `settings` for this session and saves them; on a save error they still apply until restart.
    pub fn replace(&mut self, settings: Settings) -> Result<(), SettingsError> {
        self.current = settings;
        let path = self.path.as_deref().ok_or(SettingsError::NoConfigDir)?;
        if let Some(reason) = &self.load_error {
            return Err(SettingsError::NotLoaded { path: path.to_path_buf(), reason: reason.clone() });
        }
        self.current.save(path)
    }
}

/// `settings.toml` in the profile's folder, e.g. `~/.config/ive/default/settings.toml` on Linux.
pub fn default_path() -> Option<PathBuf> {
    crate::profile::current().settings_path()
}

#[derive(Debug)]
pub enum SettingsError {
    TooLarge { path: PathBuf, bytes: u64 },
    Invalid { path: PathBuf, reason: String },
    Io { path: PathBuf, source: io::Error },
    NoConfigDir,
    /// the file did not load at start, so saving would replace it with defaults
    NotLoaded { path: PathBuf, reason: String },
}

impl SettingsError {
    fn io(path: &Path, source: io::Error) -> Self {
        Self::Io { path: path.to_path_buf(), source }
    }
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { path, bytes } => write!(
                f,
                "settings file {} is too large: {bytes} bytes (limit {MAX_SETTINGS_BYTES})",
                path.display()
            ),
            Self::Invalid { path, reason } => write!(f, "invalid settings file {}: {reason}", path.display()),
            Self::Io { path, source } => write!(f, "cannot access settings file {}: {source}", path.display()),
            Self::NoConfigDir => write!(f, "settings not saved: the OS reports no config directory"),
            Self::NotLoaded { path, reason } => write!(
                f,
                "settings not saved, to keep {} as it is: it did not load ({reason}); fix it and restart IVE",
                path.display()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overview::file_list::{FileLayout, FileSort};

    #[test]
    fn should_keep_line_of_settings_error() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        fs::write(&path, "theme = \"dark\"\n\nsidebar_width = \"wide\"\n").unwrap();

        // when
        let error = Settings::load(&path).unwrap_err().to_string();

        // then
        assert!(error.contains("line 3: "), "{error}");
    }

    #[test]
    fn should_save_and_load_user_theme() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        let settings = Settings::default().with_user_theme(ThemeChoice::File("nord".to_owned()));

        // when
        settings.save(&path).unwrap();

        // then
        assert!(fs::read_to_string(&path).unwrap().contains("user_theme = \"nord\""));
        assert_eq!(Settings::load(&path).unwrap(), settings);
    }

    #[test]
    fn should_leave_builtin_theme_out_of_file() {
        // given
        let settings = Settings::default();

        // when
        let text = toml::to_string(&settings).unwrap();

        // then
        assert!(!text.contains("user_theme"), "{text}");
    }

    #[test]
    fn should_keep_file_named_default_distinct_from_builtin() {
        // given
        let settings = Settings::default().with_user_theme(ThemeChoice::File("Default".to_owned()));

        // when
        let loaded: Settings = toml::from_str(&toml::to_string(&settings).unwrap()).unwrap();

        // then
        assert_eq!(loaded.user_theme, ThemeChoice::File("Default".to_owned()));
    }

    #[test]
    fn should_not_save_over_a_file_that_failed_to_load() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        let broken = "theme = \"purple\"\nsidebar_width = 250\n";
        fs::write(&path, broken).unwrap();
        let mut store = SettingsStore::open(Some(path.clone()));

        // when
        let result = store.replace(Settings::default().with_sidebar_width(Some(300)));

        // then
        assert!(matches!(result, Err(SettingsError::NotLoaded { .. })), "{result:?}");
        assert_eq!(fs::read_to_string(&path).unwrap(), broken);
        assert_eq!(store.current().sidebar_width, Some(300));
    }

    #[test]
    fn should_tell_why_settings_did_not_load() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        fs::write(&path, "theme = \"purple\"\n").unwrap();

        // when
        let store = SettingsStore::open(Some(path));

        // then
        let reason = store.load_error().map(str::to_owned).unwrap_or_default();
        assert!(reason.contains("purple"), "{reason}");
        assert_eq!(store.current(), &Settings::default());
    }

    #[test]
    fn should_save_normally_after_a_clean_load() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        let mut store = SettingsStore::open(Some(path.clone()));

        // when
        let result = store.replace(Settings::default().with_theme(ThemePreference::Light));

        // then
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(store.load_error(), None);
        assert!(fs::read_to_string(&path).unwrap().contains("light"));
    }

    #[test]
    fn should_default_auto_update_to_on() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        fs::write(&path, "theme = \"dark\"\n").unwrap();

        // when
        let loaded = Settings::load(&path).unwrap();

        // then
        assert_eq!(loaded.auto_update, AutoUpdate::On);
    }

    #[test]
    fn should_save_and_load_auto_update_off() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");

        // when
        Settings::default().with_auto_update(AutoUpdate::Off).save(&path).unwrap();
        let loaded = Settings::load(&path).unwrap();

        // then
        assert!(fs::read_to_string(&path).unwrap().contains("auto_update = \"off\""));
        assert_eq!(loaded.auto_update, AutoUpdate::Off);
    }

    #[test]
    fn should_leave_auto_update_out_of_the_file_while_on() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");

        // when
        Settings::default().with_auto_update(AutoUpdate::On).save(&path).unwrap();

        // then
        assert!(!fs::read_to_string(&path).unwrap().contains("auto_update"));
    }

    #[test]
    fn should_load_old_settings_with_colors_table() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        fs::write(&path, "theme = \"dark\"\n\n[colors.dark.\"JSON.KEYWORD\"]\nforeground = \"CF8E6\"\n").unwrap();

        // when
        let loaded = Settings::load(&path).unwrap();

        // then
        assert_eq!(loaded.theme, ThemePreference::Dark);
    }

    #[test]
    fn should_save_and_load_the_git_file_order() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        let settings = Settings::default().with_git_files(FileOrder { sort: FileSort::Modified, layout: FileLayout::Tree });

        // when
        settings.save(&path).unwrap();
        let loaded = Settings::load(&path).unwrap();

        // then
        assert_eq!(loaded, settings);
        assert!(fs::read_to_string(&path).unwrap().contains("[git_files]\nsort = \"modified\"\nlayout = \"tree\""));
    }

    #[test]
    fn should_save_and_load_the_diff_layout() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        let settings = Settings::default().with_git_files(FileOrder { sort: FileSort::Modified, layout: FileLayout::Tree }).with_diff_layout(DiffLayout::SideBySide);

        // when
        settings.save(&path).unwrap();
        let loaded = Settings::load(&path).unwrap();

        // then
        assert_eq!(loaded, settings);
        assert!(fs::read_to_string(&path).unwrap().contains("diff_layout = \"side-by-side\""));
    }

    #[test]
    fn should_save_and_load_theme() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ive").join("settings.toml");
        let settings = Settings::default().with_theme(ThemePreference::Light);

        // when
        settings.save(&path).unwrap();
        let loaded = Settings::load(&path).unwrap();

        // then
        assert_eq!(loaded, settings);
        assert_eq!(fs::read_to_string(&path).unwrap().trim(), r#"theme = "light""#);
    }

    #[test]
    fn should_save_sidebar_width_and_read_old_files_without_it() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        let old = dir.path().join("old.toml");
        fs::write(&old, "theme = \"dark\"\n").unwrap();
        let settings = Settings::default().with_sidebar_width(Some(340));

        // when
        settings.save(&path).unwrap();
        let loaded = (Settings::load(&path).unwrap(), Settings::load(&old).unwrap());

        // then
        assert_eq!((loaded.0.sidebar_width, loaded.1.sidebar_width), (Some(340), None));
    }

    #[test]
    fn should_use_defaults_when_file_is_missing() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let settings = Settings::load(&dir.path().join("settings.toml")).unwrap();

        // then
        assert_eq!(settings.theme, ThemePreference::System);
    }

    #[test]
    fn should_reject_unknown_theme_naming_file_and_value() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        fs::write(&path, r#"theme = "blue""#).unwrap();

        // when
        let error = Settings::load(&path).err().unwrap().to_string();

        // then
        assert!(error.starts_with(&format!("invalid settings file {}:", path.display())), "{error}");
        assert!(error.contains("blue"), "{error}");
    }

    #[test]
    fn should_keep_settings_for_session_when_there_is_no_config_dir() {
        // given
        let mut store = SettingsStore::new(Settings::default(), None);

        // when
        let result = store.replace(Settings::default().with_theme(ThemePreference::Dark));

        // then
        assert!(matches!(result, Err(SettingsError::NoConfigDir)));
        assert_eq!(store.current().theme, ThemePreference::Dark);
    }
}
