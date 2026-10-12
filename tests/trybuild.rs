//! Compile-fail suite for the type-level AAA state machine (`core::state`).
//!
//! Poka-Yoke in the negative space: each case asserts that an unlawful API use
//! is rejected at compile time, not merely discouraged at review time.
//!
//! ## The committed .stderr expectations
//!
//! The suite's `.stderr` files under `tests/compile_fail/` ARE committed:
//! trybuild matches rustc diagnostics byte-for-byte, and those diagnostics
//! vary across rustc versions, so they are generated once and reviewed, never
//! written by hand. If a case's expectations are missing or a toolchain bump
//! legitimately changes the diagnostics, regenerate them with:
//!
//! ```bash
//! TRYBUILD=overwrite cargo test --test trybuild
//! # (or: TRYBUILD=overwrite just test-unit)
//! ```
//!
//! then review and re-commit the regenerated `tests/compile_fail/*.stderr`
//! files (`TRYBUILD=overwrite` is the regeneration path, not the shipping
//! mode). Cases are
//! kept minimal and built on crate-owned API shape (missing methods, private
//! fields, sealed traits) rather than incidental rustc wording, so the
//! expectations stay stable across toolchain bumps.

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
