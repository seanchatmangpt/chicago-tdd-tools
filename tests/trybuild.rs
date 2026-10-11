//! Compile-fail suite for the type-level AAA state machine (`core::state`).
//!
//! Poka-Yoke in the negative space: each case asserts that an unlawful API use
//! is rejected at compile time, not merely discouraged at review time.
//!
//! ## Seeding the .stderr expectations
//!
//! This suite ships WITHOUT committed `.stderr` files by design: trybuild
//! matches rustc diagnostics byte-for-byte, and those diagnostics vary across
//! rustc versions, so they cannot be written by hand. On the first run,
//! trybuild fails and writes its observed output to
//! `tests/compile_fail/*.stderr`. The coordinator must seed them once with:
//!
//! ```bash
//! TRYBUILD=overwrite cargo test --test trybuild
//! # (or: TRYBUILD=overwrite just test-unit)
//! ```
//!
//! then commit the generated `tests/compile_fail/*.stderr` files. Cases are
//! kept minimal and built on crate-owned API shape (missing methods, private
//! fields, sealed traits) rather than incidental rustc wording, so the
//! expectations stay stable across toolchain bumps.

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
