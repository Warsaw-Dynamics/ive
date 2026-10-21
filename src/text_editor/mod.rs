//! A small multi-line plain-text editor for scratch tabs.

mod history;
mod keys;
mod layout;
mod model;
mod view;

pub use model::SizeLimit;
pub use view::{TextEditor, TextEditorEvent};
