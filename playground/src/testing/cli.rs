//! CLI Testing Examples
//!
//! Demonstrates CLI testing with trycmd for golden file testing.

#[cfg(feature = "cli-testing")]
use chicago_tdd_tools::testing::cli::CliTest;

#[cfg(feature = "cli-testing")]
/// Example: CLI test pattern
pub fn example_cli_test() {
    // Arrange: CLI tests are defined in .trycmd files
    // Act: Run CLI tests
    // Assert: Verify output matches golden files

    // Example: Run tests from a directory
    // CliTest::run_tests("tests/cli");

    // Example: Run a single test file
    // CliTest::run_test("tests/cli/example.trycmd");

    // Example: Run tests with custom binary
    // CliTest::run_tests_with_bin("tests/cli", "target/debug/my-cli-tool");

    // For playground, we demonstrate the API pattern
    assert!(true);
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "cli-testing")]
    use super::*;

    #[cfg(feature = "cli-testing")]
    test!(test_cli_example, {
        // Arrange-Act-Assert: Run example
        example_cli_test();
    });
}

