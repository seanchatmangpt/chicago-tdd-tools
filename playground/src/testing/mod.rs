//! Testing Features Examples
//!
//! Demonstrates all testing features: property, mutation, snapshot,
//! concurrency, cli, generator, parameterized
//!
//! v1.3.0 additions:
//! - snapshot_redaction: Snapshot redaction hooks and inline snapshots
//! - mutation_advanced: Advanced mutation operators (SwapValues, ToggleBoolean, NumericDelta)
//! - cli_environment: CLI environment presets and helpers

#[cfg(feature = "property-testing")]
pub mod property;

#[cfg(feature = "mutation-testing")]
pub mod mutation;

#[cfg(feature = "mutation-testing")]
pub mod mutation_advanced; // v1.3.0: Advanced mutation operators

#[cfg(feature = "snapshot-testing")]
pub mod snapshot;

#[cfg(feature = "snapshot-testing")]
pub mod snapshot_redaction; // v1.3.0: Snapshot redaction and inline snapshots

#[cfg(feature = "concurrency-testing")]
pub mod concurrency;

#[cfg(feature = "cli-testing")]
pub mod cli;

#[cfg(feature = "cli-testing")]
pub mod cli_environment; // v1.3.0: CLI environment presets and helpers

pub mod generator;

#[cfg(feature = "parameterized-testing")]
pub mod parameterized;

