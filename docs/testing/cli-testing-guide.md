# CLI Testing Guide - v1.2.0

## Overview

Chicago TDD Tools provides comprehensive CLI testing capabilities for testing command-line applications. This guide covers the CLI testing helpers and patterns.

## Table of Contents

1. [Quick Start](#quick-start)
2. [CLI Command Builder](#cli-command-builder)
3. [CLI Assertions](#cli-assertions)
4. [Environment Management](#environment-management)
5. [Golden File Testing](#golden-file-testing)
6. [Best Practices](#best-practices)
7. [Examples](#examples)

## Quick Start

### Enable CLI Testing Feature

Add to `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.2.0", features = ["cli-testing"] }
```

### Basic Test

```rust
#[cfg(feature = "cli-testing")]
#[test]
fn test_help_command() {
    use chicago_tdd_tools::cli::{CliTest, CliAssertions};

    let output = "Usage: myapp [OPTIONS] <COMMAND>";
    CliAssertions::assert_output_contains(output, "Usage");
}
```

## CLI Command Builder

The `CliCommandBuilder` provides a fluent API for building CLI commands:

```rust
use chicago_tdd_tools::cli::CliTest;

let cmd = CliTest::command("my-cli")
    .arg("init")
    .arg("--config=app.toml")
    .env("RUST_LOG", "debug")
    .build();

// Result: "my-cli init --config=app.toml"
```

### Building Complex Commands

```rust
// Single arguments
let cmd = CliTest::command("tool")
    .arg("--verbose")
    .arg("--output=file.txt")
    .arg("input.txt");

// Multiple arguments at once
let cmd = CliTest::command("tool")
    .args(&["--verbose", "--debug", "--help"]);

// With environment variables
let cmd = CliTest::command("tool")
    .arg("process")
    .env("RUST_LOG", "trace")
    .env("PROFILE", "release")
    .build();
```

### Accessing Built Command

```rust
let builder = CliTest::command("tool")
    .arg("--version");

// Get command string
let cmd_str = builder.build();
println!("Running: {}", cmd_str);

// Get environment variables
let env_vars = builder.env_vars();
```

## CLI Assertions

### CLI Assertions

#### Assert Output Contains

```rust
use chicago_tdd_tools::cli::CliAssertions;

let output = "Test completed successfully";

// Check for substring
CliAssertions::assert_output_contains(output, "successfully");

// Check that substring is NOT present
CliAssertions::assert_output_not_contains(output, "failed");
```

#### Assert Output Format

```rust
let help_text = "Usage: myapp [OPTIONS]\n\nOptions:\n  --help";

// Check prefix
CliAssertions::assert_output_starts_with(help_text, "Usage");

// Check suffix
CliAssertions::assert_output_ends_with(help_text, "--help");
```

#### Assert Multiple Lines

```rust
let output = "Test 1: PASS\nTest 2: PASS\nTest 3: PASS";

CliAssertions::assert_output_contains_all(
    output,
    &["Test 1", "Test 2", "Test 3", "PASS"]
);
```

## Environment Management

The `CliEnvironment` helps manage environment variables for isolated tests:

```rust
use chicago_tdd_tools::cli::CliEnvironment;

#[test]
fn test_with_custom_env() {
    let mut env = CliEnvironment::new()
        .set("MODE", "test")
        .set("LOG_LEVEL", "debug");

    env.apply(); // Set environment variables

    // Your test code here
    // Environment is automatically restored when env is dropped
}
```

### Automatic Cleanup

```rust
#[test]
fn test_env_cleanup() {
    let original = std::env::var("MY_VAR").ok();

    {
        let _env = CliEnvironment::new().set("MY_VAR", "test");
        // Environment is modified inside this block
    }

    // Environment is automatically restored here
    assert_eq!(std::env::var("MY_VAR").ok(), original);
}
```

## Golden File Testing

### Using Trycmd

Create `.trycmd` test files for golden file testing:

### `tests/cli/help.trycmd`

```
$ myapp --help
Usage: myapp [OPTIONS] <COMMAND>

Commands:
  init    Initialize new project
  build   Build the project
  help    Show this help message

Options:
  -v, --verbose    Verbose output
  -h, --help       Print help
```

### Run Golden Tests

```rust
#[test]
fn test_cli_golden_files() {
    use chicago_tdd_tools::cli::CliTest;

    CliTest::run_tests("tests/cli");
}
```

## Best Practices

### 1. Test Error Paths

```rust
#[test]
fn test_invalid_command() {
    // Test that invalid commands are rejected
    let output = "Error: unknown command 'invalid'";
    CliAssertions::assert_output_contains(output, "Error");
    CliAssertions::assert_output_contains(output, "unknown command");
}
```

### 2. Test Help Text

```rust
#[test]
fn test_help_completeness() {
    let help = run_command("myapp --help");

    // Verify all commands are documented
    CliAssertions::assert_output_contains_all(
        &help,
        &[
            "Usage:",
            "Commands:",
            "Options:",
            "--help",
        ]
    );
}
```

### 3. Use Environment Variables

```rust
#[test]
fn test_with_verbose_logging() {
    let mut env = CliEnvironment::new()
        .set("RUST_LOG", "debug")
        .set("LOG_FORMAT", "json");

    env.apply();

    // Test code that depends on environment
}
```

### 4. Organize Test Files

```
tests/
├── cli/
│   ├── help.trycmd
│   ├── init.trycmd
│   ├── build.trycmd
│   ├── error.trycmd
│   └── integration.trycmd
└── integration_tests.rs
```

### 5. Document Expected Output

```rust
#[test]
fn test_init_command() {
    // Expected output should match golden file: tests/cli/init.trycmd
    // Command: myapp init --project-name=myproject
    // Expected: Success message with generated files

    CliTest::run_test("tests/cli/init.trycmd");
}
```

## Examples

### Example 1: Simple Command Test

```rust
#[cfg(feature = "cli-testing")]
#[test]
fn test_version_command() {
    use chicago_tdd_tools::cli::{CliTest, CliAssertions};

    let cmd = CliTest::command("myapp")
        .arg("--version")
        .build();

    println!("Testing: {}", cmd);

    // In real test, you'd execute the command
    // let output = execute(&cmd);
    // CliAssertions::assert_output_starts_with(&output, "myapp");
}
```

### Example 2: Command with Arguments

```rust
#[cfg(feature = "cli-testing")]
#[test]
fn test_build_with_options() {
    use chicago_tdd_tools::cli::CliTest;

    let cmd = CliTest::command("myapp")
        .arg("build")
        .arg("--release")
        .arg("--target=wasm32-unknown-unknown")
        .build();

    assert!(cmd.contains("build"));
    assert!(cmd.contains("--release"));
    assert!(cmd.contains("wasm32"));
}
```

### Example 3: Environment-Dependent Test

```rust
#[cfg(feature = "cli-testing")]
#[test]
fn test_with_custom_config() {
    use chicago_tdd_tools::cli::CliEnvironment;

    let mut env = CliEnvironment::new()
        .set("CONFIG_PATH", "/etc/myapp/config.toml")
        .set("LOG_LEVEL", "trace");

    env.apply();

    // Your test code here that depends on these variables
    assert_eq!(std::env::var("LOG_LEVEL").ok(), Some("trace".to_string()));
}
```

### Example 4: Output Validation

```rust
#[cfg(feature = "cli-testing")]
#[test]
fn test_command_output() {
    use chicago_tdd_tools::cli::CliAssertions;

    let output = "Project initialized successfully!\nCreated files: main.rs, Cargo.toml";

    // Check specific content
    CliAssertions::assert_output_contains(&output, "initialized");

    // Check multiple items
    CliAssertions::assert_output_contains_all(
        &output,
        &["main.rs", "Cargo.toml"]
    );

    // Check format
    CliAssertions::assert_output_starts_with(&output, "Project");
}
```

## Troubleshooting

### Feature Not Enabled

```
error: unresolved import `chicago_tdd_tools::cli`
```

**Solution**: Add `features = ["cli-testing"]` to your dev-dependencies.

### Golden File Failures

**Issue**: `.trycmd` tests fail on first run

**Solution**: Run with `--test-generate` to create baseline:

```bash
cargo make test test_name -- --test-generate
```

### Environment Variable Conflicts

**Issue**: Tests affect each other's environment

**Solution**: Always use `CliEnvironment` for isolation:

```rust
{
    let _env = CliEnvironment::new().set("VAR", "value");
    // Scope ensures automatic cleanup
}
```

## See Also

- [Trycmd Documentation](https://crates.io/crates/trycmd)
- [Testing Guide](../testing-guide.md)
- [CLI Patterns Cookbook](../cookbook/cli-patterns.md)

## Questions?

Contact the development team if you need help with CLI testing.
