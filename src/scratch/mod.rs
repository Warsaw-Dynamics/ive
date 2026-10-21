//! Scratches: personal files of a project kept outside its repository, under `<config>/ive/scratches/<id>/`.

pub mod groups;
pub mod icon;
pub mod run;
pub mod search;
pub mod store;
mod tab;

use crate::project::ProjectId;
use store::{ScratchError, ScratchStore};

pub use tab::{ScratchTab, ScratchTarget};

/// Store errors go to stderr as well as to where the user acted.
pub fn log_error(error: &ScratchError) {
    eprintln!("ive: {error}");
}

/// A scratch: its project and its file name.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScratchRef {
    pub project: ProjectId,
    pub name: String,
}

/// The scratches folder of `project`, under the profile's folder.
pub fn store_of(project: &ProjectId) -> Result<ScratchStore, String> {
    let config = crate::profile::current().config_dir().ok_or("scratches need a config folder, and the OS reports none")?;
    ScratchStore::new(&config, project.as_str()).map_err(|error| error.to_string())
}
