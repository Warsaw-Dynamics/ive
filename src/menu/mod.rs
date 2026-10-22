//! Menus for what was clicked: a tab's file or the tree's selection. Which declared items show, in what state, and
//! how the keyboard and the pointer move through them. Plain data; drawing is `crate::menu_view`.

pub mod keys;
pub mod nav;
pub mod resolve;
pub mod when;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathKind {
    File,
    Folder,
}

/// Where a path is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Within {
    /// in a checkout, at this `/`-separated relative path
    Checkout(String),
    /// in no open checkout: a scratch, a file from elsewhere
    Outside,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectPath {
    /// absolute
    pub path: PathBuf,
    pub within: Within,
    pub kind: PathKind,
}

/// The one checkout every path is in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Home {
    Checkout(PathBuf),
    /// no paths, paths outside every checkout, or paths in several
    Mixed,
}

/// What a menu applies to: the paths of a tab's file or of the tree's selection, in the tree's order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject {
    pub paths: Vec<SubjectPath>,
    pub home: Home,
}

impl Subject {
    pub fn empty() -> Self {
        Self { paths: Vec::new(), home: Home::Mixed }
    }
}
