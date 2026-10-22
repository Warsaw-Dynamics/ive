//! A small multi-line plain-text editor.

mod colours;
mod history;
pub(crate) mod keys;
mod layout;
pub(crate) mod model;
mod view;

pub use model::{Motion, Selecting, SizeLimit, Unit};
pub use view::{Setup, TextEditor, TextEditorEvent};
