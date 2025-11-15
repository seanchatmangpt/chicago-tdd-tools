//! CLI Testing Example
//!
//! Demonstrates CLI testing using trycmd for Chicago TDD.
//! CLI testing uses golden files to verify command output stability.

#[cfg(feature = "cli-testing")]
#[allow(unused_imports)] // Example code - imports shown for demonstration
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
    #[test]
    fn test_cli_command_builder() {
        // Arrange: Create CLI command builder
        use chicago_tdd_tools::cli::CliCommandBuilder;

        // Act: Build a CLI command with arguments and environment variables
        let cmd = CliCommandBuilder::new("echo")
            .arg("hello")
            .arg("world")
            .env("TEST_VAR", "test_value")
            .build();

        // Assert: Verify command string is correct
        assert!(cmd.contains("echo"), "Command should contain 'echo'");
        assert!(cmd.contains("hello"), "Command should contain 'hello'");
        assert!(cmd.contains("world"), "Command should contain 'world'");
    }

    #[test]
    fn test_cli_assertions() {
        // Arrange: Create test output
        use chicago_tdd_tools::cli::CliAssertions;

        let output = "Usage: myapp [OPTIONS] <COMMAND>\n\nCommands:\n  help  Print help";

        // Act & Assert: Verify output contains expected text
        CliAssertions::assert_output_contains(output, "Usage");
        CliAssertions::assert_output_contains(output, "Commands");
        CliAssertions::assert_output_contains(output, "help");
    }

    #[test]
    fn test_cli_environment() {
        // Arrange: Create CLI environment manager
        use chicago_tdd_tools::cli::CliEnvironment;

        // Act: Set environment variables and apply them
        let mut env = CliEnvironment::new().set("TEST_VAR1", "value1").set("TEST_VAR2", "value2");
        env.apply();

        // Assert: Verify environment variables are set
        assert_eq!(std::env::var("TEST_VAR1").unwrap_or_default(), "value1");
        assert_eq!(std::env::var("TEST_VAR2").unwrap_or_default(), "value2");

        // Cleanup: Environment automatically restored on drop
    }
}
