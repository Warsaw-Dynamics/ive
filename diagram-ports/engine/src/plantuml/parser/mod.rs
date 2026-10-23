//! Original bounded parser infrastructure; semantic source traces remain in provenance.
pub(super) mod context;
pub(super) mod definition;
pub(super) mod error;
pub(super) mod token;

pub(super) use definition::parse_definition;

#[derive(Clone, Copy)]
pub(super) struct SourceView<'s> {
    pub text: &'s str,
    pub byte_base: usize,
}

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/definition.rs"]
mod generated_definition;
