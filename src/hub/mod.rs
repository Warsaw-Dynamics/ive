//! The hub (Ctrl+Shift+E): recent tabs, changed files, running processes and an fzf-style search over every file of every
//! open project.

pub mod matcher;
pub mod content;
pub mod corpus;
pub mod creations;
pub mod files;
pub mod search;
pub mod history;
pub mod key_label;
pub mod model;
pub mod rows;
pub mod sources;
pub mod index;
pub mod view;
pub mod preview;
pub mod output;
