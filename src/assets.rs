use std::borrow::Cow;

use gpui::{App, AssetSource, SharedString};

/// Files under `assets/`, compiled into the binary so IVE runs without an install directory.
pub struct Assets;

/// `(path under assets/, contents)`, sorted by path; `build.rs` writes it.
static FILES: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/assets.rs"));

impl Assets {
    /// The file at `path` under `assets/`, e.g. `fonts/Inter.ttf`.
    pub fn get(path: &str) -> Option<&'static [u8]> {
        FILES.binary_search_by_key(&path, |(name, _)| name).ok().map(|at| FILES[at].1)
    }

    fn paths() -> impl Iterator<Item = &'static str> {
        FILES.iter().map(|(name, _)| *name)
    }
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        Ok(Self::get(path).map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(Self::paths().filter(|asset| asset.starts_with(path)).map(SharedString::from).collect())
    }
}

/// Registers the bundled `fonts/*.ttf` with the text system, so IVE's default fonts resolve without being installed.
pub fn load_fonts(cx: &App) {
    let fonts = Assets::paths().filter(|path| path.starts_with("fonts/") && path.ends_with(".ttf"));
    let data = fonts.filter_map(Assets::get).map(Cow::Borrowed).collect();
    cx.text_system().add_fonts(data).expect("bundled fonts are valid TrueType");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_a_bundled_file_by_its_path_under_assets() {
        // given
        let path = "scripts/ive.rn";

        // when
        let found = Assets::get(path);

        // then
        assert_eq!(found, Some(include_bytes!("../assets/scripts/ive.rn").as_slice()));
        assert_eq!(Assets::get("scripts/missing.rn"), None);
    }

    #[test]
    fn should_list_bundled_files_under_a_folder() {
        // given
        let folder = "fonts/";

        // when
        let listed = Assets.list(folder).unwrap();

        // then
        assert!(!listed.is_empty());
        assert!(listed.iter().all(|path| path.starts_with(folder) && Assets::get(path).is_some()));
    }
}
