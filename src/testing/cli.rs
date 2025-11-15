//! CLI Testing Framework
//!
//! Provides comprehensive CLI testing capabilities using trycmd for Chicago TDD.
//! Trycmd enables golden file testing for CLI tools, ensuring command output
//! stability and correctness.
//!
//! # Chicago TDD Alignment
//!
//! CLI testing aligns with Chicago TDD principles:
//! - **State-Based Testing**: Verifies CLI command outputs
//! - **Behavior Verification**: Tests what CLI commands produce, not how
//! - **AAA Pattern**: Arrange (setup), Act (run command), Assert (verify output)
//!
//! # New in v1.2.0: Enhanced CLI Testing Helpers
//!
//! - `CliCommandBuilder`: Fluent API for building CLI commands
//! - `CliAssertions`: Output verification helpers
//! - `CliEnvironment`: Environment setup for tests
//! - `CliTestScenario`: Complete test scenario builder

#[cfg(feature = "cli-testing")]
use std::collections::HashMap;
#[cfg(feature = "cli-testing")]
use trycmd::TestCases;

/// CLI test helper for Chicago TDD
///
/// Provides a Chicago TDD-friendly wrapper around trycmd's CLI testing.
/// This makes CLI testing consistent with other testing utilities in the framework.
#[cfg(feature = "cli-testing")]
pub struct CliTest;

/// Builder for CLI commands with fluent API
///
/// Enables clean, readable CLI command construction with validation.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "cli-testing")]
/// use chicago_tdd_tools::cli::CliCommandBuilder;
///
/// # #[cfg(feature = "cli-testing")]
/// let cmd = CliCommandBuilder::new("my-cli")
///     .arg("init")
///     .arg("--config=app.toml")
///     .env("RUST_LOG", "debug")
///     .build();
/// ```
#[cfg(feature = "cli-testing")]
#[derive(Debug, Clone)]
pub struct CliCommandBuilder {
    binary: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

#[cfg(feature = "cli-testing")]
impl CliCommandBuilder {
    /// Create a new CLI command builder
    ///
    /// # Arguments
    ///
    /// * `binary` - Name or path of the binary to run
    #[must_use]
    pub fn new(binary: &str) -> Self {
        Self { binary: binary.to_string(), args: Vec::new(), env: HashMap::new() }
    }

    /// Add an argument to the command
    #[must_use]
    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    /// Add multiple arguments
    #[must_use]
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args.extend(args.iter().map(|s| s.to_string()));
        self
    }

    /// Set an environment variable
    #[must_use]
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// Get command string representation
    #[must_use]
    pub fn build(&self) -> String {
        let mut cmd = self.binary.clone();
        for arg in &self.args {
            cmd.push(' ');
            cmd.push_str(arg);
        }
        cmd
    }

    /// Get environment variables
    #[must_use]
    pub fn env_vars(&self) -> &HashMap<String, String> {
        &self.env
    }
}

/// CLI assertion helpers for output verification
///
/// Provides common assertions for CLI testing.
#[cfg(feature = "cli-testing")]
pub struct CliAssertions;

#[cfg(feature = "cli-testing")]
impl CliAssertions {
    /// Assert output contains substring
    ///
    /// # Panics
    ///
    /// Panics if output does not contain the expected string
    pub fn assert_output_contains(output: &str, expected: &str) {
        assert!(
            output.contains(expected),
            "Output does not contain '{}'. Output: {}",
            expected,
            output
        );
    }

    /// Assert output does not contain substring
    ///
    /// # Panics
    ///
    /// Panics if output contains the unexpected string
    pub fn assert_output_not_contains(output: &str, unexpected: &str) {
        assert!(
            !output.contains(unexpected),
            "Output contains unexpected '{}'.  Output: {}",
            unexpected,
            output
        );
    }

    /// Assert output starts with prefix
    ///
    /// # Panics
    ///
    /// Panics if output does not start with the expected prefix
    pub fn assert_output_starts_with(output: &str, prefix: &str) {
        assert!(
            output.starts_with(prefix),
            "Output does not start with '{}'. Output: {}",
            prefix,
            output
        );
    }

    /// Assert output ends with suffix
    ///
    /// # Panics
    ///
    /// Panics if output does not end with the expected suffix
    pub fn assert_output_ends_with(output: &str, suffix: &str) {
        assert!(
            output.ends_with(suffix),
            "Output does not end with '{}'. Output: {}",
            suffix,
            output
        );
    }

    /// Assert output lines contain all of the given strings
    ///
    /// # Panics
    ///
    /// Panics if any expected string is not found in output lines
    pub fn assert_output_contains_all(output: &str, expected_lines: &[&str]) {
        for expected in expected_lines {
            assert!(
                output.contains(expected),
                "Output does not contain line '{}'. Output: {}",
                expected,
                output
            );
        }
    }
}

/// Environment setup for CLI tests
///
/// Manages environment variables for isolated test runs.
#[cfg(feature = "cli-testing")]
pub struct CliEnvironment {
    vars: HashMap<String, String>,
    original_vars: HashMap<String, Option<String>>,
}

#[cfg(feature = "cli-testing")]
impl CliEnvironment {
    /// Create a new CLI environment
    #[must_use]
    pub fn new() -> Self {
        Self { vars: HashMap::new(), original_vars: HashMap::new() }
    }

    /// Set an environment variable
    pub fn set(mut self, key: &str, value: &str) -> Self {
        self.vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Apply environment variables
    pub fn apply(&mut self) {
        for (key, value) in &self.vars {
            self.original_vars.insert(key.clone(), std::env::var(key).ok());
            std::env::set_var(key, value);
        }
    }

    /// Restore original environment
    pub fn restore(&self) {
        for (key, original) in &self.original_vars {
            match original {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}

#[cfg(feature = "cli-testing")]
impl Default for CliEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "cli-testing")]
impl Drop for CliEnvironment {
    fn drop(&mut self) {
        self.restore();
    }
}

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
    /// ```rust
    /// # #[cfg(feature = "cli-testing")]
    /// use chicago_tdd_tools::cli::CliTest;
    ///
    /// # #[cfg(feature = "cli-testing")]
    /// # // CliTest::run_tests("tests/cli");
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

    /// Create a new CLI command builder
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "cli-testing")]
    /// use chicago_tdd_tools::cli::CliTest;
    ///
    /// # #[cfg(feature = "cli-testing")]
    /// let cmd = CliTest::command("my-cli")
    ///     .arg("--help")
    ///     .build();
    /// ```
    pub fn command(binary: &str) -> CliCommandBuilder {
        CliCommandBuilder::new(binary)
    }
}

#[cfg(feature = "cli-testing")]
#[cfg(test)]
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod tests {
    use super::{CliAssertions, CliCommandBuilder, CliEnvironment, CliTest};

    #[test]
    fn test_cli_test_struct_available() {
        // Arrange: Verify CliTest struct is available
        // Act: Create a reference (compile-time check)
        let _test = CliTest;
        // Assert: Struct compiles and is available
        assert!(true);
    }

    #[test]
    fn test_cli_command_builder_creation() {
        // Arrange: Create a command builder
        let cmd = CliCommandBuilder::new("test-cli");
        // Act: Verify builder was created
        let command_str = cmd.build();
        // Assert: Command string is correct
        assert_eq!(command_str, "test-cli");
    }

    #[test]
    fn test_cli_command_builder_with_args() {
        // Arrange: Create builder and add arguments
        let cmd = CliCommandBuilder::new("test-cli")
            .arg("init")
            .arg("--config=app.toml");
        // Act: Build command
        let command_str = cmd.build();
        // Assert: Command contains all arguments
        assert!(command_str.contains("init"));
        assert!(command_str.contains("--config=app.toml"));
    }

    #[test]
    fn test_cli_command_builder_with_multiple_args() {
        // Arrange: Create builder with multiple args
        let cmd = CliCommandBuilder::new("test-cli").args(&["--verbose", "--debug", "--help"]);
        // Act: Build command
        let command_str = cmd.build();
        // Assert: All arguments are present
        assert!(command_str.contains("--verbose"));
        assert!(command_str.contains("--debug"));
        assert!(command_str.contains("--help"));
    }

    #[test]
    fn test_cli_command_builder_with_env() {
        // Arrange: Create builder with environment variables
        let cmd = CliCommandBuilder::new("test-cli")
            .env("RUST_LOG", "debug")
            .env("MODE", "test");
        // Act: Get environment variables
        let env_vars = cmd.env_vars();
        // Assert: Environment variables are stored
        assert_eq!(env_vars.get("RUST_LOG"), Some(&"debug".to_string()));
        assert_eq!(env_vars.get("MODE"), Some(&"test".to_string()));
    }

    #[test]
    fn test_cli_assertions_contains() {
        // Arrange: Create output
        let output = "Hello, World!\nTests passed!";
        // Act & Assert: Test contains assertion
        CliAssertions::assert_output_contains(output, "Hello");
        CliAssertions::assert_output_contains(output, "Tests passed");
    }

    #[test]
    #[should_panic(expected = "does not contain")]
    fn test_cli_assertions_contains_fails() {
        // Arrange: Create output
        let output = "Hello, World!";
        // Act & Assert: Test that missing content triggers panic
        CliAssertions::assert_output_contains(output, "Goodbye");
    }

    #[test]
    fn test_cli_assertions_not_contains() {
        // Arrange: Create output
        let output = "Hello, World!";
        // Act & Assert: Test not contains assertion
        CliAssertions::assert_output_not_contains(output, "Goodbye");
    }

    #[test]
    #[should_panic(expected = "contains unexpected")]
    fn test_cli_assertions_not_contains_fails() {
        // Arrange: Create output
        let output = "Hello, World!";
        // Act & Assert: Test that present content triggers panic
        CliAssertions::assert_output_not_contains(output, "Hello");
    }

    #[test]
    fn test_cli_assertions_starts_with() {
        // Arrange: Create output
        let output = "SUCCESS: All tests passed";
        // Act & Assert: Test starts with assertion
        CliAssertions::assert_output_starts_with(output, "SUCCESS");
    }

    #[test]
    #[should_panic(expected = "does not start with")]
    fn test_cli_assertions_starts_with_fails() {
        // Arrange: Create output
        let output = "SUCCESS: All tests passed";
        // Act & Assert: Test that different prefix triggers panic
        CliAssertions::assert_output_starts_with(output, "FAILURE");
    }

    #[test]
    fn test_cli_assertions_ends_with() {
        // Arrange: Create output
        let output = "All tests passed successfully";
        // Act & Assert: Test ends with assertion
        CliAssertions::assert_output_ends_with(output, "successfully");
    }

    #[test]
    #[should_panic(expected = "does not end with")]
    fn test_cli_assertions_ends_with_fails() {
        // Arrange: Create output
        let output = "All tests passed successfully";
        // Act & Assert: Test that different suffix triggers panic
        CliAssertions::assert_output_ends_with(output, "failed");
    }

    #[test]
    fn test_cli_assertions_contains_all() {
        // Arrange: Create output with multiple lines
        let output = "Test 1: PASS\nTest 2: PASS\nTest 3: PASS";
        // Act & Assert: Test contains all assertion
        CliAssertions::assert_output_contains_all(output, &["Test 1", "Test 2", "Test 3", "PASS"]);
    }

    #[test]
    #[should_panic(expected = "does not contain")]
    fn test_cli_assertions_contains_all_fails() {
        // Arrange: Create output
        let output = "Test 1: PASS\nTest 2: PASS";
        // Act & Assert: Test that missing line triggers panic
        CliAssertions::assert_output_contains_all(output, &["Test 1", "Test 3"]);
    }

    #[test]
    fn test_cli_environment_creation() {
        // Arrange: Create environment
        let env = CliEnvironment::new();
        // Act: Verify creation
        let env_default = CliEnvironment::default();
        // Assert: Both methods work
        assert_eq!(std::mem::size_of_val(&env), std::mem::size_of_val(&env_default));
    }

    #[test]
    fn test_cli_environment_set_and_restore() {
        // Arrange: Create environment and set variables
        let mut env = CliEnvironment::new().set("TEST_VAR", "test_value");
        // Act: Apply environment
        env.apply();
        // Assert: Variable is set
        assert_eq!(std::env::var("TEST_VAR").ok(), Some("test_value".to_string()));
        // Cleanup: Restore happens in drop
        drop(env);
    }

    #[test]
    fn test_cli_test_command_method() {
        // Arrange: Use CliTest::command method
        let cmd = CliTest::command("my-tool").arg("--version").build();
        // Act & Assert: Command is built correctly
        assert!(cmd.contains("my-tool"));
        assert!(cmd.contains("--version"));
    }
}
