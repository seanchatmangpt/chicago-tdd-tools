//! Transport primitives for MCP testing.
//!
//! - [`pipe`]: In-process duplex transport (no subprocess, no network).
//! - [`child`]: Subprocess transport wrapper around `rmcp`'s `TokioChildProcess`.

pub mod child;
pub mod pipe;
