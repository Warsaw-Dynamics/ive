//! Which file answers a request path: rewrites, the file (`index.html` for a folder), the fallback; never a file
//! outside the served folder.

use std::path::{Component, Path, PathBuf};

use regex::Regex;

use super::ServeSetup;

/// The served folder (canonical) and the rules a path goes through.
pub struct Site {
    pub root: PathBuf,
    rewrites: Vec<(Regex, String)>,
    fallback: Option<String>,
}

/// The file answering a path, or none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    File(PathBuf),
    NotFound,
}

impl Site {
    /// Fails when the root is not a readable folder or a rewrite is not a regex.
    pub fn new(setup: &ServeSetup) -> Result<Self, String> {
        let root = dunce::canonicalize(&setup.root).map_err(|error| format!("cannot serve {}: {error}", setup.root.display()))?;
        if !root.is_dir() {
            return Err(format!("cannot serve {}: not a folder", root.display()));
        }
        let rewrites = setup
            .rewrites
            .iter()
            .map(|rewrite| Regex::new(&rewrite.from).map(|from| (from, rewrite.to.clone())).map_err(|error| format!("rewrite {:?}: {error}", rewrite.from)))
            .collect::<Result<_, _>>()?;
        Ok(Self { root, rewrites, fallback: setup.fallback.clone() })
    }

    /// The file for `path` (decoded, rewritten); a GET of a page that is not there gets the fallback.
    pub fn resolve(&self, method: &str, path: &str) -> Resolution {
        let Some(decoded) = percent_decoded(path) else { return Resolution::NotFound };
        let rewritten = self.rewritten(&decoded);
        if let Some(file) = self.file(&rewritten) {
            return Resolution::File(file);
        }
        let page = Path::new(&rewritten).extension().is_none_or(|extension| extension.eq_ignore_ascii_case("html"));
        match &self.fallback {
            Some(fallback) if method == "GET" && page => self.file(fallback).map_or(Resolution::NotFound, Resolution::File),
            _ => Resolution::NotFound,
        }
    }

    /// `path` after the first rewrite whose `from` matches it.
    pub fn rewritten(&self, path: &str) -> String {
        match self.rewrites.iter().find(|(from, _)| from.is_match(path)) {
            Some((from, to)) => from.replace(path, to.as_str()).into_owned(),
            None => path.to_owned(),
        }
    }

    /// The file `path` names under the root, or its `index.html` for a folder; never one outside the root.
    pub fn file(&self, path: &str) -> Option<PathBuf> {
        let relative = Path::new(path.trim_start_matches('/'));
        // on Windows `\\host\share` or `C:\x` would replace the root, and opening it, before the check below, would
        // already sign in to that host
        if relative.components().any(|part| matches!(part, Component::Prefix(_) | Component::RootDir)) {
            return None;
        }
        let found = dunce::canonicalize(self.root.join(relative)).ok()?;
        let found = if found.is_dir() { dunce::canonicalize(found.join("index.html")).ok()? } else { found };
        (found.starts_with(&self.root) && found.is_file()).then_some(found)
    }
}

/// `path` with `%XX` escapes decoded; `None` when they are broken or decode to a NUL or to text that is not UTF-8.
fn percent_decoded(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).ok().filter(|decoded| !decoded.contains('\0'))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::run::file::Rewrite;
    use crate::run::serve::ServeSetup;

    fn site(fallback: Option<&str>, rewrites: &[(&str, &str)]) -> (tempfile::TempDir, Site) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("site");
        fs::create_dir_all(root.join("blog")).unwrap();
        fs::write(root.join("index.html"), "home").unwrap();
        fs::write(root.join("blog/index.html"), "blog").unwrap();
        fs::write(root.join("blog/first.html"), "first").unwrap();
        fs::write(root.join("app.css"), "css").unwrap();
        fs::write(dir.path().join("secret"), "secret").unwrap();
        let setup = ServeSetup {
            root: root.clone(),
            port: 4200,
            live_reload: true,
            fallback: fallback.map(str::to_owned),
            rewrites: rewrites.iter().map(|(from, to)| Rewrite { from: (*from).to_owned(), to: (*to).to_owned() }).collect(),
            script: None,
        };
        let site = Site::new(&setup).unwrap();
        (dir, site)
    }

    fn served(site: &Site, path: &str) -> Option<String> {
        match site.resolve("GET", path) {
            Resolution::File(file) => Some(fs::read_to_string(file).unwrap()),
            Resolution::NotFound => None,
        }
    }

    #[test]
    fn should_serve_index_of_a_folder() {
        // given
        let (_dir, site) = site(None, &[]);

        // when
        let pages = [served(&site, "/"), served(&site, "/blog/"), served(&site, "/blog")];

        // then
        assert_eq!(pages, [Some("home".to_owned()), Some("blog".to_owned()), Some("blog".to_owned())]);
    }

    #[test]
    fn should_apply_first_matching_rewrite() {
        // given
        let (_dir, site) = site(None, &[("^/posts/([^/]+)$", "/blog/$1.html"), ("^/posts/.*$", "/index.html")]);

        // when
        let page = served(&site, "/posts/first");

        // then
        assert_eq!(page.as_deref(), Some("first"));
    }

    #[test]
    fn should_fall_back_for_extensionless_path() {
        // given
        let (_dir, site) = site(Some("index.html"), &[]);

        // when
        let pages = [served(&site, "/app/settings"), served(&site, "/missing.html")];

        // then
        assert_eq!(pages, [Some("home".to_owned()), Some("home".to_owned())]);
    }

    #[test]
    fn should_not_fall_back_for_asset() {
        // given
        let (_dir, site) = site(Some("index.html"), &[]);

        // when
        let page = served(&site, "/missing.css");

        // then
        assert_eq!(page, None);
    }

    #[test]
    fn should_refuse_parent_traversal() {
        // given
        let (_dir, site) = site(None, &[]);

        // when
        let pages = [served(&site, "/../secret"), served(&site, "/%2e%2e/secret"), served(&site, "/blog/%2E%2E/%2e%2e/secret")];

        // then
        assert_eq!(pages, [None, None, None]);
    }

    #[cfg(windows)]
    #[test]
    fn should_refuse_a_path_that_names_a_drive_or_a_host() {
        // given
        let (_dir, site) = site(None, &[]);
        let inside_root_by_its_full_path = site.root.join("index.html").display().to_string();

        // when
        let page = served(&site, &format!("/{inside_root_by_its_full_path}"));

        // then
        assert_eq!(page, None);
    }

    // creating a symlink needs privileges on Windows
    #[cfg(unix)]
    #[test]
    fn should_refuse_symlink_out_of_root() {
        // given
        let (dir, site) = site(None, &[]);
        std::os::unix::fs::symlink(dir.path().join("secret"), dir.path().join("site/leak")).unwrap();

        // when
        let page = served(&site, "/leak");

        // then
        assert_eq!(page, None);
    }

    #[test]
    fn should_decode_percent_escapes() {
        // given
        let (_dir, site) = site(None, &[]);
        fs::write(site.root.join("a b.html"), "spaced").unwrap();

        // when
        let page = served(&site, "/a%20b.html");

        // then
        assert_eq!(page.as_deref(), Some("spaced"));
    }
}
