//! Colour keys and schemes, modelled on IntelliJ's `TextAttributesKey`s and colour schemes.

mod attributes;
mod keys;
mod scheme;

pub use attributes::{Entry, RawEntry, TextAttributes};
pub use keys::ColorKey;
pub use scheme::{Overrides, ResolvedScheme, Scheme};
