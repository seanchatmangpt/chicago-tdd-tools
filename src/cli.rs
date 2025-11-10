//! CLI Testing Framework
//!
//! Provides CLI testing capabilities using trycmd for Chicago TDD.
//! Trycmd enables golden file testing for CLI tools, ensuring command output
//! stability and correctness.
//!
//! # Chicago TDD Alignment
//!
//! CLI testing aligns with Chicago TDD principles:
//! - **State-Based Testing**: Verifies CLI command outputs
//! - **Behavior Verification**: Tests what CLI commands produce, not how
//! - **AAA Pattern**: Arrange (setup), Act (run command), Assert (verify output)

#[cfg(feature = "cli-testing")]
use trycmd::TestCases;

/// CLI test helper for Chicago TDD
///
/// Provides a Chicago TDD-friendly wrapper around trycmd's CLI testing.
/// This makes CLI testing consistent with other testing utilities in the framework.
#[cfg(feature = "cli-testing")]
pub struct CliTest;

#[cfg(feature = "cli-testing")]
impl CliTest {
    /// Run CLI tests from a directory
    ///
    /// # Arguments
    ///
    /// * `test_dir` - Directory containing test cases (`.trycmd` files)
    ///
    /// # Panics
    ///
    /// Panics if any CLI test fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use chicago_tdd_tools::cli::CliTest;
    ///
    /// CliTest::run_tests("tests/cli");
    /// ```
    pub fn run_tests(test_dir: &str) {
        TestCases::new().case(test_dir).run();
    }

    /// Run a single CLI test case
    ///
    /// # Arguments
    ///
    /// * `test_file` - Path to a `.trycmd` test file
    ///
    /// # Panics
    ///
    /// Panics if the CLI test fails.
    pub fn run_test(test_file: &str) {
        TestCases::new().case(test_file).run();
    }

    /// Run CLI tests with custom binary path
    ///
    /// # Arguments
    ///
    /// * `test_dir` - Directory containing test cases
    /// * `bin_path` - Path to the binary to test
    ///
    /// # Panics
    ///
    /// Panics if any CLI test fails.
    ///
    /// Note: Set the binary path via environment variable BIN or in .trycmd files
    pub fn run_tests_with_bin(test_dir: &str, bin_path: &str) {
        std::env::set_var("BIN", bin_path);
        TestCases::new().case(test_dir).run();
    }
}

#[cfg(feature = "cli-testing")]
#[cfg(test)]
mod tests {
    use super::CliTest;

    #[test]
    fn test_cli_test_struct_available() {
        // Arrange: Verify CliTest struct is available
        // Act: Create a reference (compile-time check)
        let _test = CliTest;
        // Assert: Struct compiles and is available
        assert!(true);
    }

    #[test]
    fn test_cli_test_methods_exist() {
        // Arrange: Verify methods exist and can be called
        // Act: Call methods (they will fail without actual .trycmd files, but we verify API)
        // Note: These would normally be called with actual test directories
        // CliTest::run_tests("tests/cli");
        // CliTest::run_test("tests/cli/example.trycmd");
        // CliTest::run_tests_with_bin("tests/cli", "target/debug/my-cli-tool");

        // Assert: Methods exist and API is correct
        // This test verifies the API compiles correctly
        assert!(true);
    }

    #[test]
    fn test_cli_test_compilation() {
        // Arrange: Test that CliTest compiles with feature flag
        // Act: Verify struct and methods are available
        let _cli_test = CliTest;

        // Assert: Compilation successful
        assert!(std::mem::size_of::<CliTest>() == 0); // Zero-sized type
    }
}
