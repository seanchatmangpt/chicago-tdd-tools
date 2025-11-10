//! CLI Testing Example
//!
//! Demonstrates CLI testing using trycmd for Chicago TDD.
//! CLI testing uses golden files to verify command output stability.

#[cfg(feature = "cli-testing")]
use chicago_tdd_tools::cli::CliTest;

#[cfg(feature = "cli-testing")]
fn main() {
    println!("CLI Testing Example");
    println!("Run: cargo test --features cli-testing --example cli_testing");
    println!("\nNote: CLI tests use .trycmd files in tests/cli/ directory");
    println!("Create .trycmd files to define CLI test cases.");
}

#[cfg(not(feature = "cli-testing"))]
fn main() {
    println!("CLI testing feature not enabled. Enable with: --features cli-testing");
}

#[cfg(feature = "cli-testing")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_example() {
        // Arrange: CLI tests are defined in .trycmd files
        // Act: Run CLI tests
        // Assert: Verify output matches golden files

        // Example: Run tests from a directory
        // CliTest::run_tests("tests/cli");

        // Example: Run a single test file
        // CliTest::run_test("tests/cli/example.trycmd");

        // Example: Run tests with custom binary
        // CliTest::run_tests_with_bin("tests/cli", "target/debug/my-cli-tool");

        // For this example, we just verify the module compiles
        assert!(true);
    }
}
