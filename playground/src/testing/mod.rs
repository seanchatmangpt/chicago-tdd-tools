//! Testing Features Examples
//!
//! Demonstrates all testing features: property, mutation, snapshot,
//! concurrency, cli, generator, parameterized

#[cfg(feature = "property-testing")]
pub mod property;

#[cfg(feature = "mutation-testing")]
pub mod mutation;

#[cfg(feature = "snapshot-testing")]
pub mod snapshot;

#[cfg(feature = "concurrency-testing")]
pub mod concurrency;

#[cfg(feature = "cli-testing")]
pub mod cli;

pub mod generator;

#[cfg(feature = "parameterized-testing")]
pub mod parameterized;

