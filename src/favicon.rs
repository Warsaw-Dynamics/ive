//! Project and module icons: a folder with a `.favicon.<ext>` of its own. The root folder's is the project's icon,
//! any other folder's makes it a module, and a path belongs to the deepest module above it.

use std::collections::BTreeMap;

const STEM: &str = ".favicon.";
/// A favicon file larger than this is left out: an icon drawn at 16 px needs far less, and decoding it is not free.
pub const MAX_FAVICON_BYTES: u64 = 512 * 1024;
/// When a folder has several, the first of these wins.
const EXTENSIONS: [&str; 5] = ["svg", "png", "jpg", "jpeg", "ico"];

/// The favicon files of a checkout, by folder; paths are checkout-relative with `/`, the root folder is `""`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Favicons {
    by_folder: BTreeMap<String, String>,
}

impl Favicons {
    /// The favicons among a checkout's file paths.
    pub fn from_paths<'a>(paths: impl IntoIterator<Item = &'a str>) -> Favicons {
        let mut ranked: BTreeMap<String, (usize, String)> = BTreeMap::new();
        for path in paths {
            let Some((folder, rank)) = favicon_rank(path) else {
                continue;
            };
            let better = ranked.get(folder).is_none_or(|(held, _)| rank < *held);
            if better {
                ranked.insert(folder.to_owned(), (rank, path.to_owned()));
            }
        }
        Favicons { by_folder: ranked.into_iter().map(|(folder, (_, path))| (folder, path)).collect() }
    }

    /// These favicons but the files `rejected` names (by their checkout-relative path); their folders then have
    /// none, so a module whose favicon is rejected is no module.
    pub fn without(self, rejected: impl Fn(&str) -> bool) -> Favicons {
        Favicons { by_folder: self.by_folder.into_iter().filter(|(_, file)| !rejected(file)).collect() }
    }

    /// The project's icon: the root folder's favicon file.
    pub fn root(&self) -> Option<&str> {
        self.icon_of("")
    }

    /// The deepest module folder that is `path` or holds it; the root folder is never a module.
    pub fn module_of(&self, path: &str) -> Option<&str> {
        let mut folder = path;
        loop {
            if let Some((held, _)) = self.by_folder.get_key_value(folder).filter(|_| !folder.is_empty()) {
                return Some(held.as_str());
            }
            folder = folder.rsplit_once('/')?.0;
        }
    }

    /// The favicon file of `folder`.
    pub fn icon_of(&self, folder: &str) -> Option<&str> {
        self.by_folder.get(folder).map(String::as_str)
    }
}

/// Whether `path` names a favicon file: a change to it changes a project's or module's icon.
pub fn is_favicon_file(path: &std::path::Path) -> bool {
    path.file_name().and_then(|name| name.to_str()).and_then(favicon_rank).is_some()
}

/// The folder of a favicon file and its extension's rank; `None` for any other path.
fn favicon_rank(path: &str) -> Option<(&str, usize)> {
    let (folder, name) = path.rsplit_once('/').unwrap_or(("", path));
    let extension = name.strip_prefix(STEM)?;
    EXTENSIONS.iter().position(|known| *known == extension).map(|rank| (folder, rank))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_pick_the_favicon_by_extension_order() {
        // given
        let paths = ["app/.favicon.png", "app/.favicon.svg", "app/.favicon.ico", ".favicon.jpg"];

        // when
        let favicons = Favicons::from_paths(paths);

        // then
        assert_eq!(favicons.icon_of("app"), Some("app/.favicon.svg"));
        assert_eq!(favicons.root(), Some(".favicon.jpg"));
    }

    #[test]
    fn should_put_a_path_in_its_deepest_module() {
        // given
        let favicons = Favicons::from_paths(["a/.favicon.svg", "a/b/.favicon.svg", ".favicon.svg"]);

        // when
        let modules = ["a/b/c/x.rs", "a/x.rs", "a/b", "ab/x.rs", "x.rs"].map(|path| favicons.module_of(path));

        // then
        assert_eq!(modules, [Some("a/b"), Some("a"), Some("a/b"), None, None]);
    }

    #[test]
    fn should_tell_a_favicon_file_among_changed_paths() {
        // given
        let paths = ["/r/svc/.favicon.svg", "/r/.favicon.ico", "/r/svc/favicon.svg", "/r/svc/.favicon.gif", "/r/svc"];

        // when
        let favicons = paths.map(|path| is_favicon_file(std::path::Path::new(path)));

        // then
        assert_eq!(favicons, [true, true, false, false, false]);
    }

    #[test]
    fn should_leave_out_a_favicon_too_big_to_draw() {
        // given
        let favicons = Favicons::from_paths(["a/.favicon.png", "b/.favicon.svg"]);

        // when
        let kept = favicons.without(|file| file == "a/.favicon.png");

        // then
        assert_eq!((kept.icon_of("a"), kept.icon_of("b")), (None, Some("b/.favicon.svg")));
    }

    #[test]
    fn should_ignore_other_favicon_names() {
        // given
        let paths = ["priv/static/favicon.svg", "src/favicon.ico", "x/.favicon.gif", "y/.favicon"];

        // when
        let favicons = Favicons::from_paths(paths);

        // then
        assert_eq!(favicons, Favicons::default());
    }
}
