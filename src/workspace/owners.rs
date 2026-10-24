//! Whose a tab or a row is: its project, and in a monorepo its module, with their favicons.

use std::path::{Path, PathBuf};

use super::Workspace;
use crate::favicon::Favicons;
use crate::project::{CheckoutRef, ProjectId};

/// A tab's or a row's owner: its project, the module folder (checkout-relative) its path is in, and the favicon to
/// show for it, the module's or else the project's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owner {
    pub project: ProjectId,
    pub module: Option<String>,
    pub icon: Option<PathBuf>,
}

impl Workspace {
    /// The favicons of `checkout`, once the hub's index read it.
    fn favicons(&self, checkout: &Path) -> Option<&Favicons> {
        self.hub_index.files(checkout).map(|read| read.favicons.as_ref())
    }

    /// The project's icon: its main checkout's root favicon, the same for every worktree.
    pub fn project_icon(&self, id: &ProjectId) -> Option<PathBuf> {
        let main = &self.project(id)?.main_checkout().path;
        self.favicons(main)?.root().map(|file| main.join(file))
    }

    /// The owner of `path` (absolute, or relative to the checkout) in `checkout`; no path, or one outside the
    /// checkout, is the project itself.
    pub fn owner_of(&self, checkout: &CheckoutRef, path: Option<&Path>) -> Owner {
        let relative = path.and_then(|path| relative_text(&checkout.path, path));
        let (module, file) = owner_in(self.favicons(&checkout.path), relative.as_deref());
        let icon = match file {
            Some(file) => Some(checkout.path.join(file)),
            None => self.project_icon(&checkout.project),
        };
        Owner { project: checkout.project.clone(), module, icon }
    }

    /// The favicon of module `folder` (absolute, or relative to `checkout`); `None` for a folder that is no module.
    pub fn module_icon(&self, checkout: &Path, folder: &Path) -> Option<PathBuf> {
        let favicons = self.favicons(checkout)?;
        let folder = relative_text(checkout, folder)?;
        let module = favicons.module_of(&folder).filter(|module| *module == folder)?;
        favicons.icon_of(module).map(|file| checkout.join(file))
    }
}

/// `path` relative to `checkout`, `/`-separated as the index keeps paths.
fn relative_text(checkout: &Path, path: &Path) -> Option<String> {
    let relative = checkout.join(path);
    let relative = relative.strip_prefix(checkout).ok()?;
    let parts: Option<Vec<&str>> = relative.components().map(|part| part.as_os_str().to_str()).collect();
    Some(parts?.join("/"))
}

/// The module folder `relative` is in and its favicon file, both checkout-relative; nothing while unread.
fn owner_in(favicons: Option<&Favicons>, relative: Option<&str>) -> (Option<String>, Option<String>) {
    let Some((favicons, relative)) = favicons.zip(relative) else {
        return (None, None);
    };
    let Some(module) = favicons.module_of(relative) else {
        return (None, None);
    };
    (Some(module.to_owned()), favicons.icon_of(module).map(str::to_owned))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_name_the_module_and_its_icon() {
        // given
        let favicons = Favicons::from_paths(["idp/.favicon.svg"]);

        // when
        let owner = owner_in(Some(&favicons), Some("idp/src/A.java"));

        // then
        assert_eq!(owner, (Some("idp".to_owned()), Some("idp/.favicon.svg".to_owned())));
    }

    #[test]
    fn should_have_no_owner_icon_before_the_index_is_read() {
        // given
        let unread = None;

        // when
        let owner = owner_in(unread, Some("idp/src/A.java"));

        // then
        assert_eq!(owner, (None, None));
    }
}
