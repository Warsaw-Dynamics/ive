//! Original bounded parser infrastructure; semantic source traces remain in provenance.
pub(super) mod activity;
pub(super) mod board;
pub(super) mod bpm;
pub(super) mod chart;
pub(super) mod chen;
pub(super) mod context;
pub(super) mod creole;
pub(super) mod definition;
pub(super) mod document;
pub(super) mod ebnf;
pub(super) mod error;
pub(super) mod files;
pub(super) mod gantt;
pub(super) mod hcl;
pub(super) mod header;
pub(super) mod json;
pub(super) mod nwdiag;
pub(super) mod outline;
pub(super) mod ownership;
pub(super) mod packet;
pub(super) mod preamble;
pub(super) mod railroad;
pub(super) mod regex;
pub(super) mod salt;
pub(super) mod sequence;
pub(super) mod state;
pub(super) mod structured;
#[cfg(test)]
pub(super) mod test_support;
pub(super) mod timing;
pub(super) mod token;
pub(super) mod tree;
pub(super) mod uml;
pub(super) mod yaml;

pub(super) use definition::parse_definition;

pub(super) struct BodySource<'s> {
    pub document: &'s super::origin::Document,
    pub range: std::ops::Range<usize>,
    pub ownership: ownership::Ownership,
    pub adapters: usize,
}

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/bpm.rs"]
mod generated_bpm;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/ebnf.rs"]
mod generated_ebnf;
#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/regex.rs"]
mod generated_regex;

impl BodySource<'_> {
    pub fn reborrow(&self) -> Self {
        Self {
            document: self.document,
            range: self.range.clone(),
            ownership: self.ownership,
            adapters: self.adapters,
        }
    }
}

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/tree.rs"]
mod generated_tree;
#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/files.rs"]
mod generated_files;
#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/board.rs"]
mod generated_board;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/chart.rs"]
mod generated_chart;

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

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/state.rs"]
mod generated_state;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/uml.rs"]
mod generated_uml;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/timing.rs"]
mod generated_timing;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/gantt.rs"]
mod generated_gantt;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/nwdiag.rs"]
mod generated_nwdiag;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/creole.rs"]
mod generated_creole;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/activity.rs"]
mod generated_activity;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/chen.rs"]
mod generated_chen;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/json.rs"]
mod generated_json;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/hcl.rs"]
mod generated_hcl;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/yaml.rs"]
mod generated_yaml;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/salt.rs"]
mod generated_salt;

#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
#[path = "../generated/packet.rs"]
mod generated_packet;
