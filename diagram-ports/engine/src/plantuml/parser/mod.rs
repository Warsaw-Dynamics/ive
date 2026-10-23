//! Original bounded parser infrastructure; semantic source traces remain in provenance.
pub(super) mod context;
pub(super) mod definition;
pub(super) mod document;
pub(super) mod error;
pub(super) mod header;
pub(super) mod ownership;
pub(super) mod preamble;
pub(super) mod sequence;
pub(super) mod token;

pub(super) use definition::parse_definition;

pub(super) struct BodySource<'s> {
    pub document: &'s super::origin::Document,
    pub range: std::ops::Range<usize>,
    pub ownership: ownership::Ownership,
    pub adapters: usize,
}

#[derive(Clone, Copy)]
pub(super) struct SourceView<'s> {
    pub text: &'s str,
    pub byte_base: usize,
}

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/definition.rs"]
mod generated_definition;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/document.rs"]
mod generated_document;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/header.rs"]
mod generated_header;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/preamble.rs"]
mod generated_preamble;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/sequence.rs"]
mod generated_sequence;
