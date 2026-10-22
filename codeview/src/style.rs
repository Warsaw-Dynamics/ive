//! How the host looks, as far as codeview cares.

/// Dark or light: picks the colour scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
    Dark,
    Light,
}
