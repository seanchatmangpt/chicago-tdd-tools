//! Chicago TDD Tools Playground
//!
//! Comprehensive playground demonstrating all features of chicago-tdd-tools.
//! This serves as both a validation suite and a reference implementation.

#[macro_use]
extern crate chicago_tdd_tools;

mod core;
mod testing;
mod validation;
mod observability;
mod integration;

fn main() {
    println!("Chicago TDD Tools Playground");
    println!("============================");
    println!();
    println!("This playground demonstrates all features of chicago-tdd-tools.");
    println!("Run tests with: cargo test");
    println!("Run examples with: cargo run --bin playground");
    println!();
    println!("Feature Categories:");
    println!("  - Core: Fixtures, builders, assertions, macros, state, type_level, const_assert, alert");
    println!("  - Testing: Property, mutation, snapshot, concurrency, cli, generator, parameterized");
    println!("  - Validation: Coverage, guards, jtbd, performance");
    println!("  - Observability: OTEL, weaver");
    println!("  - Integration: Testcontainers");
}

