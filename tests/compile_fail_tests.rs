//! Compile-fail tests for guard constraints
//!
//! These tests verify that invalid guard constraint values fail to compile,
//! ensuring compile-time validation works correctly.

use chicago_tdd_tools::test;

/// Compile-fail tests for guard constraints
///
/// These tests verify that invalid guard constraint values fail to compile,
/// ensuring compile-time validation works correctly.
test!(compile_fail_tests, {
    // Arrange: Set up trybuild test cases
    let t = trybuild::TestCases::new();

    // Act & Assert: Verify invalid guard constraint values fail to compile
    t.compile_fail("tests/compile-fail/validated_run_compile_error.rs");
    t.compile_fail("tests/compile-fail/validated_batch_compile_error.rs");
});
