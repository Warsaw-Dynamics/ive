//! What the scripts folder holds of scripts, so a change to anything else there (IVE's own `state.toml`, saved on
//! every change) does not reload them.

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

use super::engine::IVE_RN_NAME;
use crate::theme_config::user::THEMES_DIR;

/// File name → content hash of each `*.rn` file directly in `dir` but IVE's own `ive.rn`, and of each theme as
/// `themes/<file>`; empty when the folder cannot be read.
pub fn fingerprint(dir: &Path) -> BTreeMap<String, u64> {
    let scripts = hashed(dir, "rn").filter(|(name, _)| name != IVE_RN_NAME);
    let themes = hashed(&dir.join(THEMES_DIR), "toml").map(|(name, hash)| (format!("{THEMES_DIR}/{name}"), hash));
    scripts.chain(themes).collect()
}

/// File name and content hash of each file with `extension` directly in `dir`.
fn hashed(dir: &Path, extension: &'static str) -> impl Iterator<Item = (String, u64)> + use<> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(move |path| path.extension().is_some_and(|found| found == extension))
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().into_owned();
            let mut hasher = DefaultHasher::new();
            std::fs::read(&path).ok()?.hash(&mut hasher);
            Some((name, hasher.finish()))
        })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn should_ignore_files_other_than_rune() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("init.rn"), "pub fn init(config) {}").unwrap();
        let before = fingerprint(dir.path());

        // when
        fs::write(dir.path().join("state.toml"), "version = 2").unwrap();

        // then
        assert_eq!(fingerprint(dir.path()), before);
    }

    #[test]
    fn should_reload_when_theme_file_changes() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("themes")).unwrap();
        let before = fingerprint(dir.path());

        // when
        fs::write(dir.path().join("themes").join("a.toml"), "[dark]").unwrap();

        // then
        assert_ne!(fingerprint(dir.path()), before);
    }

    #[test]
    fn should_change_when_a_rune_file_changes() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("init.rn"), "pub fn init(config) {}").unwrap();
        let before = fingerprint(dir.path());

        // when
        fs::write(dir.path().join("init.rn"), "pub fn init(config) { }").unwrap();

        // then
        assert_ne!(fingerprint(dir.path()), before);
    }

    #[test]
    fn should_ignore_ive_rn() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("init.rn"), "pub fn init(config) {}").unwrap();
        let before = fingerprint(dir.path());

        // when
        fs::write(dir.path().join("ive.rn"), "// IVE's defaults").unwrap();

        // then
        assert_eq!(fingerprint(dir.path()), before);
    }
}
