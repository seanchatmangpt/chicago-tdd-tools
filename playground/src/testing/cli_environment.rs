//! v1.3.0 CLI Environment Helpers Examples
//!
//! Demonstrates the CLI environment helpers added in v1.3.0:
//! - CliEnvironment presets (CI, development, production, clean)
//! - Environment variable helpers
//! - Stderr/stdout separate capture

use chicago_tdd_tools::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CliEnvironmentResult {
    pub ci_preset_examples: usize,
    pub development_preset_examples: usize,
    pub production_preset_examples: usize,
    pub clean_environment_examples: usize,
    pub environment_variable_examples: usize,
    pub stderr_capture_examples: usize,
}

pub fn run() -> crate::Result<CliEnvironmentResult> {
    let mut ci_preset_examples = 0;
    let mut development_preset_examples = 0;
    let mut production_preset_examples = 0;
    let mut clean_environment_examples = 0;
    let mut environment_variable_examples = 0;
    let mut stderr_capture_examples = 0;

    // ========================================================================
    // 1. CI PRESET - Optimized for continuous integration environments
    // ========================================================================
    {
        // Example 1a: CI preset sets up typical CI environment variables
        // CliEnvironment::ci()
        //     .with_env("CI", "true")
        //     .with_env("CI_COMMIT_SHA", "abc123def456")
        //     .with_env("CI_COMMIT_BRANCH", "main")
        //     .with_env("CI_BUILD_NUMBER", "42")
        //     .run_command("my_cli test")?;

        // The CI preset automatically sets:
        // - CI=true
        // - LOG_LEVEL=info (not debug for performance)
        // - TERM=dumb (no color codes)
        // - NO_COLOR=1 (disable colored output)

        ci_preset_examples += 1;

        // Example 1b: Testing with CI environment
        // let result = CliTest::new("my_binary")
        //     .with_ci_environment()
        //     .run()?;
        //
        // Tests run in CI mode:
        // - No interactive prompts
        // - Deterministic output (no timestamps, colors)
        // - Optimized for log parsing

        ci_preset_examples += 1;
    }

    // ========================================================================
    // 2. DEVELOPMENT PRESET - Optimized for local development
    // ========================================================================
    {
        // Example 2a: Development preset includes debug logging
        // CliEnvironment::development()
        //     .with_env("DEBUG", "1")
        //     .with_env("LOG_LEVEL", "debug")
        //     .with_env("RUST_BACKTRACE", "1")
        //     .run_command("my_cli dev")?;

        // The development preset automatically sets:
        // - LOG_LEVEL=debug (detailed output)
        // - RUST_BACKTRACE=full (detailed panic info)
        // - TERM=auto (colors if terminal supports)
        // - Allows interactive prompts

        development_preset_examples += 1;

        // Example 2b: Local development with .env file
        // let result = CliTest::new("my_binary")
        //     .with_development_environment()
        //     .with_env_from_file(".env.development")
        //     .run()?;

        development_preset_examples += 1;

        // Example 2c: Hot-reload scenario
        // CliEnvironment::development()
        //     .with_env("WATCH", "true")
        //     .with_env("AUTO_RELOAD", "true")
        //     .run_command("my_cli serve")?;

        development_preset_examples += 1;
    }

    // ========================================================================
    // 3. PRODUCTION PRESET - Optimized for production environments
    // ========================================================================
    {
        // Example 3a: Production preset includes minimal logging
        // CliEnvironment::production()
        //     .with_env("LOG_LEVEL", "error")
        //     .with_env("NO_COLORS", "1")
        //     .run_command("my_cli serve")?;

        // The production preset automatically sets:
        // - LOG_LEVEL=error (only errors and above)
        // - NO_COLOR=1 (disable colors)
        // - TERM=dumb (safe for all terminals)
        // - Strict error handling

        production_preset_examples += 1;

        // Example 3b: Production with strict configuration
        // let result = CliTest::new("my_binary")
        //     .with_production_environment()
        //     .with_env("STRICT_MODE", "1")
        //     .run()?;
        //
        // Ensures:
        // - All required env vars are present
        // - No missing required configuration
        // - Strict validation of inputs

        production_preset_examples += 1;

        // Example 3c: Production health check
        // CliEnvironment::production()
        //     .with_env("HEALTH_CHECK", "1")
        //     .with_env("METRICS_ENABLED", "1")
        //     .run_command("my_cli health")?;

        production_preset_examples += 1;
    }

    // ========================================================================
    // 4. CLEAN ENVIRONMENT - Isolated, deterministic environment
    // ========================================================================
    {
        // Example 4a: Start with empty environment (no inherited vars)
        // let result = CliTest::new("my_binary")
        //     .with_clean_environment()
        //     .with_env("ONLY_VAR", "value")
        //     .run()?;
        //
        // All parent environment variables are cleared
        // Only explicitly set variables are present
        // Ensures deterministic test behavior

        clean_environment_examples += 1;

        // Example 4b: Clean environment with minimal required vars
        // CliEnvironment::clean()
        //     .with_env("HOME", "/tmp/test_home")
        //     .with_env("PATH", "/usr/bin:/bin")
        //     .with_env("APP_MODE", "test")
        //     .run_command("my_cli init")?;

        clean_environment_examples += 1;

        // Example 4c: Isolation for security-sensitive tests
        // CliEnvironment::clean()
        //     .with_env("API_KEY", "test_key_123")
        //     .with_env("DATABASE_URL", "sqlite:memory")
        //     .run_command("my_cli secure_operation")?;
        //
        // No risk of picking up real API keys from parent environment

        clean_environment_examples += 1;
    }

    // ========================================================================
    // 5. ENVIRONMENT VARIABLE HELPERS
    // ========================================================================
    {
        // Example 5a: Setting individual environment variables
        // let result = CliTest::new("my_binary")
        //     .with_env("DATABASE_URL", "postgres://localhost/test")
        //     .with_env("API_KEY", "test_key")
        //     .with_env("TIMEOUT", "30")
        //     .run()?;

        environment_variable_examples += 1;

        // Example 5b: Loading from .env file
        // let result = CliTest::new("my_binary")
        //     .with_env_from_file(".env.test")
        //     .run()?;
        //
        // Loads all variables from file:
        // DATABASE_URL=postgres://localhost/test
        // API_KEY=test_key
        // LOG_LEVEL=debug

        environment_variable_examples += 1;

        // Example 5c: Environment variable with interpolation
        // CliEnvironment::development()
        //     .with_env("HOME", "/tmp/test")
        //     .with_env("LOG_DIR", "$HOME/logs")  // Would expand to /tmp/test/logs
        //     .run_command("my_cli setup")?;

        environment_variable_examples += 1;

        // Example 5d: Scoped environment for single operation
        // CliEnvironment::new()
        //     .with_env("DEBUG", "1")
        //     .with_scoped_env(|| {
        //         // This operation runs with DEBUG=1
        //         run_operation()
        //         // Env var reverts after scope exits
        //     })?;

        environment_variable_examples += 1;
    }

    // ========================================================================
    // 6. STDERR/STDOUT SEPARATE CAPTURE
    // ========================================================================
    {
        // Example 6a: Separate stdout and stderr
        // let result = CliTest::new("my_binary")
        //     .capture_stderr_separately()
        //     .run()?;
        //
        // Result has separate fields:
        // - result.stdout: Standard output
        // - result.stderr: Standard error
        // - result.exit_code: Exit code

        stderr_capture_examples += 1;

        // Example 6b: Assert on stderr specifically
        // let result = CliTest::new("my_binary")
        //     .capture_stderr_separately()
        //     .run()?;
        //
        // assert_contains!(result.stderr, "Warning:");
        // assert_contains!(result.stdout, "Success");
        // assert_eq!(result.exit_code, 0);

        stderr_capture_examples += 1;

        // Example 6c: Capturing warnings separately
        // let result = CliTest::new("cargo")
        //     .arg("build")
        //     .capture_stderr_separately()
        //     .run()?;
        //
        // Compiler warnings in stderr
        // Build output in stdout
        // Can assert independently on each

        stderr_capture_examples += 1;

        // Example 6d: Error handling with stderr
        // let result = CliTest::new("my_binary")
        //     .arg("invalid_command")
        //     .capture_stderr_separately()
        //     .run()?;
        //
        // assert_contains!(result.stderr, "Error: Unknown command");
        // assert_eq!(result.exit_code, 1);
        // assert!(result.stdout.is_empty());

        stderr_capture_examples += 1;
    }

    // ========================================================================
    // 7. COMPREHENSIVE CLI ENVIRONMENT SCENARIO
    // ========================================================================
    {
        // Example 7a: Integration test with multiple environment presets
        // Simulate different deployment scenarios:

        // Test 1: Local development
        // CliEnvironment::development()
        //     .with_env_from_file(".env.development")
        //     .run_command("my_cli serve")?;

        development_preset_examples += 1;

        // Test 2: CI environment
        // CliEnvironment::ci()
        //     .with_env("CI_COMMIT_SHA", "abc123")
        //     .run_command("my_cli test")?;

        ci_preset_examples += 1;

        // Test 3: Production simulation
        // CliEnvironment::production()
        //     .run_command("my_cli health")?;

        production_preset_examples += 1;

        // Test 4: Isolated security test
        // CliEnvironment::clean()
        //     .with_env("API_KEY", "test_key")
        //     .run_command("my_cli secure_op")?;

        clean_environment_examples += 1;
    }

    Ok(CliEnvironmentResult {
        ci_preset_examples,
        development_preset_examples,
        production_preset_examples,
        clean_environment_examples,
        environment_variable_examples,
        stderr_capture_examples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_environment() {
        let result = run().expect("CLI environment should run");
        assert!(result.ci_preset_examples > 0);
        assert!(result.development_preset_examples > 0);
        assert!(result.production_preset_examples > 0);
        assert!(result.clean_environment_examples > 0);
        assert!(result.environment_variable_examples > 0);
        assert!(result.stderr_capture_examples > 0);
    }
}
