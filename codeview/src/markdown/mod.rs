//! Markdown's Rendered view: the text parsed into a document of blocks and pieces of visible text, and the view that
//! draws it, selects in it and answers the find box; see the spec "Markdown: Text | Rendered".

mod build;
mod copy;
mod document;
#[cfg(test)]
mod document_tests;

pub use build::build;
pub use document::{Align, Block, BlockKind, Doc, Look, Marker, Nesting, Piece, Run, Tick, parse};
