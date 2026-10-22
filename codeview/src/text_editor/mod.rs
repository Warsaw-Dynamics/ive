//! A small multi-line plain-text editor.

mod colours;
mod history;
mod keys;
mod layout;
mod model;
mod view;

pub use model::{Motion, Selecting, SizeLimit, Unit};
pub use view::{Setup, TextEditor, TextEditorEvent};
