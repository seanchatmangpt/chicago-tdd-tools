//! Test harnesses for MCP servers and clients.
//!
//! - [`server`]: `McpServerHarness` — spawns an MCP server binary and drives it.
//! - [`client`]: `McpStubServer` — stub server for testing MCP client code.
//! - [`session`]: `McpSession<S>` — phantom-type state machine enforcing lifecycle order.

pub mod client;
pub mod server;
pub mod session;
