//! MCP server that lets an agent drive IVE: tabs, terminal input, screen text and screenshots.
//! It runs on its own thread with a tokio runtime and asks the workspace through [`crate::api`].

pub mod endpoint;
mod server;
mod token;

pub use server::{start, url};
pub use token::{TOKEN_ENV, Token};
