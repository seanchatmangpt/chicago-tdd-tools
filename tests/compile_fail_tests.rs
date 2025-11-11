//! Compile-fail tests for guard constraints
//!
//! These tests verify that invalid guard constraint values fail to compile,
//! ensuring compile-time validation works correctly.

/// Compile-fail tests for guard constraints
///
/// These tests verify that invalid guard constraint values fail to compile,
/// ensuring compile-time validation works correctly.
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/validated_run_compile_error.rs");
    t.compile_fail("tests/compile-fail/validated_batch_compile_error.rs");
}
