//! What the scripts folder holds of scripts, so a change to anything else there (IVE's own `state.toml`, saved on
//! every change) does not reload them.

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

use super::engine::IVE_RN_NAME;

/// File name → content hash of each `*.rn` file directly in `dir` but IVE's own `ive.rn`; empty when the folder
/// cannot be read.
pub fn fingerprint(dir: &Path) -> BTreeMap<String, u64> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return BTreeMap::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rn"))
        .filter(|path| path.file_name().is_some_and(|name| name != IVE_RN_NAME))
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().into_owned();
            let mut hasher = DefaultHasher::new();
            std::fs::read(&path).ok()?.hash(&mut hasher);
            Some((name, hasher.finish()))
        })
        .collect()
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
