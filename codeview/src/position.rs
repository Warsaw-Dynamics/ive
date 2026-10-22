//! Where a view is scrolled and where its caret is, as `state.toml` keeps them.

use serde::{Deserialize, Serialize};

/// Where a text view was scrolled: the first visible line (fractional) and the horizontal offset in pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ScrollPosition {
    pub top_line: f64,
    pub left: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaretPosition {
    pub line: usize,
    pub column: usize,
}
