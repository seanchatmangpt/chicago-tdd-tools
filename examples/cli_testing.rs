//! # CLI Testing Example - Comprehensive Guide
//!
//! Demonstrates CLI testing using `trycmd` for Chicago TDD. CLI testing uses golden files
//! to verify command output stability and ensures CLI commands work correctly.
//!
//! ## Tutorial: Getting Started
//!
//! This example demonstrates CLI testing:
//!
//! 1. **Command Building**: Use `CliCommandBuilder` to build CLI commands
//! 2. **Output Assertions**: Use `CliAssertions` to verify command output
//! 3. **Environment Management**: Use `CliEnvironment` to manage environment variables
//!
//! **Run tests**: `cargo test --features cli-testing --example cli_testing`
//!
//! **Note**: CLI tests use `.trycmd` files in `tests/cli/` directory. Create `.trycmd` files
//! to define CLI test cases with expected output.
//!
//! ## Explanation: Concepts
//!
//! **CLI Testing**: Tests command-line interfaces by executing commands and verifying output.
//! Uses golden files (`.trycmd`) to store expected output and compare against actual output.
//!
//! **Golden Files**: Files containing expected command output. On first run, creates golden file.
//! On subsequent runs, compares actual output to golden file. If different, test fails.
//!
//! **Command Building**: `CliCommandBuilder` provides fluent API for building commands
//! with arguments, environment variables, and options.
//!
//! **Output Assertions**: `CliAssertions` provides helpers for verifying command output:
//! - Contains expected text
//! - Matches patterns
//! - Has correct exit codes
//!
//! **Environment Management**: `CliEnvironment` manages environment variables for tests.
//! Automatically restores environment on drop, ensuring test isolation.
//!
//! **Test Isolation**: Each test runs in isolated environment. Environment variables are
//! restored after test, preventing test interference.
//!
//! ## How-to: Common Tasks
//!
//! - Build CLI commands: See `test_cli_command_builder`
//! - Assert command output: See `test_cli_assertions`
//! - Manage environment variables: See `test_cli_environment`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `CliCommandBuilder`: Builder for CLI commands
//! - `CliAssertions`: Assertions for CLI output
//! - `CliEnvironment`: Environment variable manager
//! - `CliTest`: CLI test wrapper
//!
//! **Key Functions**:
//! - `CliCommandBuilder::new(command) -> CliCommandBuilder` - Create command builder
//! - `CliCommandBuilder::arg(arg)` - Add argument
//! - `CliCommandBuilder::env(key, value)` - Set environment variable
//! - `CliCommandBuilder::build() -> String` - Build command string
//! - `CliAssertions::assert_output_contains(output, text)` - Assert output contains text
//! - `CliEnvironment::new() -> CliEnvironment` - Create environment manager
//! - `CliEnvironment::set(key, value)` - Set environment variable
//! - `CliEnvironment::apply()` - Apply environment variables
//!
//! **Key Concepts**:
//! - **Golden Files**: Expected output stored in `.trycmd` files
//! - **Command Building**: Fluent API for constructing commands
//! - **Output Verification**: Assertions for command output
//! - **Environment Isolation**: Automatic environment restoration

use chicago_tdd_tools::cli::CliTest;
#[cfg(feature = "cli-testing")]
#[allow(unused_imports)] // Example code - imports shown for demonstration
use chicago_tdd_tools::prelude::*;

#[cfg(feature = "cli-testing")]
fn main() {
    chicago_tdd_tools::alert_info!("CLI Testing Example");
    chicago_tdd_tools::alert_info!("Run: cargo test --features cli-testing --example cli_testing");
    chicago_tdd_tools::alert_info!("\nNote: CLI tests use .trycmd files in tests/cli/ directory");
    chicago_tdd_tools::alert_info!("Create .trycmd files to define CLI test cases.");
}

#[cfg(not(feature = "cli-testing"))]
fn main() {
    chicago_tdd_tools::alert_info!(
        "CLI testing feature not enabled. Enable with: --features cli-testing"
    );
}

#[cfg(feature = "cli-testing")]
#[cfg(test)]
mod tests {
    // Example: CLI command building
    //
    // ## How-to: Build CLI Commands
    //
    // Use `CliCommandBuilder` to build CLI commands with arguments and environment variables.
    // Provides fluent API for constructing commands programmatically.
    //
    // ## Reference
    //
    // - **Builder**: `CliCommandBuilder::new(command) -> CliCommandBuilder`
    // - **Methods**:
    //   - `arg(arg)` - Add command argument
    //   - `env(key, value)` - Set environment variable
    //   - `build() -> String` - Build command string
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::cli::CliCommandBuilder;
    //
    // let cmd = CliCommandBuilder::new("echo")
    //     .arg("hello")
    //     .arg("world")
    //     .env("TEST_VAR", "test_value")
    //     .build();
    // ```
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

    // Example: CLI output assertions
    //
    // ## How-to: Assert CLI Output
    //
    // Use `CliAssertions` to verify command output contains expected text. Provides
    // helpers for common output verification patterns.
    //
    // ## Reference
    //
    // - **Function**: `CliAssertions::assert_output_contains(output, text)`
    // - **Parameters**:
    //   - `output`: Command output string
    //   - `text`: Expected text to find in output
    // - **Behavior**: Panics if text not found in output
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::cli::CliAssertions;
    //
    // let output = "Usage: myapp [OPTIONS]";
    // CliAssertions::assert_output_contains(output, "Usage");
    // ```
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

    // Example: CLI environment management
    //
    // ## How-to: Manage Environment Variables
    //
    // Use `CliEnvironment` to set environment variables for CLI tests. Environment
    // variables are automatically restored on drop, ensuring test isolation.
    //
    // ## Reference
    //
    // - **Manager**: `CliEnvironment::new() -> CliEnvironment`
    // - **Methods**:
    //   - `set(key, value)` - Set environment variable
    //   - `apply()` - Apply environment variables to process
    // - **Cleanup**: Environment automatically restored on drop
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::cli::CliEnvironment;
    //
    // let mut env = CliEnvironment::new()
    //     .set("TEST_VAR1", "value1")
    //     .set("TEST_VAR2", "value2");
    // env.apply();
    // // Environment automatically restored when env goes out of scope
    // ```
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
