//! OCEL 2.0 Generation Module (CA-10)
//!
//! Transforms test execution into Object-Centric Event Logs (OCEL 2.0).

#![allow(missing_docs)]
#![allow(dead_code)]

#[cfg(feature = "ocel-generation")]
pub mod collector;
#[cfg(feature = "ocel-generation-discovery")]
pub mod discovery;
#[cfg(feature = "ocel-generation")]
pub mod projections;
#[cfg(feature = "ocel-generation")]
pub mod types;
#[cfg(feature = "ocel-generation")]
pub mod wasm4pm;

#[cfg(feature = "ocel-generation")]
pub use collector::OcelCollector;
#[cfg(feature = "ocel-generation")]
pub use types::*;

// --- v26.10.11 assurance: OCEL 2.0 ingestion bridge (word packing + import) ---
#[cfg(feature = "ocel-ingest")]
pub mod idmap_bridge;
#[cfg(feature = "ocel-ingest-json")]
pub mod import_json;
#[cfg(feature = "ocel-ingest-xml")]
pub mod import_xml;
#[cfg(feature = "ocel-ingest")]
pub mod ocel64_bridge;
#[cfg(all(feature = "ocel-ingest-json", feature = "conformance"))]
pub mod powl_fixture_bridge;
