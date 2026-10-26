//! Markdown's Rendered view: the text parsed into a document of blocks and pieces of visible text, and the view that
//! draws it, selects in it and answers the find box; see the spec "Markdown: Text | Rendered".

mod build;
mod copy;
mod document;
#[cfg(test)]
mod document_tests;
mod paint;
mod split;
pub(crate) mod pill;
mod view;
#[cfg(test)]
mod view_tests;

pub use build::build;
pub use view::RenderedView;
pub use document::{Align, Block, BlockKind, Doc, Look, Marker, Nesting, Part, Piece, Run, Tick, parse};
