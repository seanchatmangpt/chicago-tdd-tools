# CLI Testing Example

**Category:** How-To Guide
**Level:** Intermediate
**Prerequisites:** Understanding of CLI applications
**Features Required:** `cli-testing`

---

## Overview

This example demonstrates CLI testing using `trycmd` for Chicago TDD. CLI testing uses golden files to verify command output stability and ensures CLI commands work correctly.

**What you'll learn:**
- Building CLI commands with `CliCommandBuilder`
- Verifying command output with `CliAssertions`
- Managing environment variables with `CliEnvironment`
- Using golden files for output verification

---

## Quick Start

```bash
cargo test --features cli-testing --example cli_testing
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools with `cli-testing` feature

**Add to Cargo.toml:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["cli-testing"] }
```

---

## Key Concepts

### CLI Testing

Tests command-line interfaces by executing commands and verifying output. Uses golden files (`.trycmd`) to store expected output.

### Golden Files

Files containing expected command output:
- First run: Creates golden file
- Subsequent runs: Compares output to golden file
- If different: Test fails

### Command Building

`CliCommandBuilder` provides fluent API for building commands with arguments, environment variables, and options.

---

## Code Examples

### Example 1: Command Building

```rust
use chicago_tdd_tools::cli::CliCommandBuilder;

#[test]
fn test_cli_command_builder() {
    // Arrange & Act: Build command
    let cmd = CliCommandBuilder::new("echo")
        .arg("hello")
        .arg("world")
        .env("TEST_VAR", "test_value")
        .build();

    // Assert: Verify command string
    assert!(cmd.contains("echo"));
    assert!(cmd.contains("hello"));
    assert!(cmd.contains("world"));
}
```

### Example 2: Output Assertions

```rust
use chicago_tdd_tools::cli::CliAssertions;

#[test]
fn test_cli_assertions() {
    // Arrange: Create test output
    let output = "Usage: myapp [OPTIONS] <COMMAND>\n\nCommands:\n  help  Print help";

    // Act & Assert: Verify output contains expected text
    CliAssertions::assert_output_contains(output, "Usage");
    CliAssertions::assert_output_contains(output, "Commands");
    CliAssertions::assert_output_contains(output, "help");
}
```

### Example 3: Environment Management

```rust
use chicago_tdd_tools::cli::CliEnvironment;

#[test]
fn test_cli_environment() {
    // Arrange & Act: Set environment variables
    let mut env = CliEnvironment::new()
        .set("TEST_VAR1", "value1")
        .set("TEST_VAR2", "value2");
    env.apply();

    // Assert: Verify environment variables are set
    assert_eq!(std::env::var("TEST_VAR1").unwrap_or_default(), "value1");
    assert_eq!(std::env::var("TEST_VAR2").unwrap_or_default(), "value2");

    // Cleanup: Environment automatically restored on drop
}
```

---

## Common Patterns

### Pattern 1: Testing CLI Help Output

```rust
let output = execute_command(&["myapp", "--help"]);
CliAssertions::assert_output_contains(&output, "Usage:");
CliAssertions::assert_output_contains(&output, "Options:");
```

### Pattern 2: Testing Command Execution

```rust
let cmd = CliCommandBuilder::new("myapp")
    .arg("process")
    .arg("--input")
    .arg("data.txt")
    .build();

// Execute and verify
```

### Pattern 3: Environment-Dependent Tests

```rust
let mut env = CliEnvironment::new()
    .set("CONFIG_PATH", "/custom/path")
    .set("DEBUG", "true");
env.apply();

// Test with custom environment
```

---

## Golden File Workflow

### 1. Create `.trycmd` File

Create `tests/cli/mycommand.trycmd`:
```
$ myapp hello world
Hello, world!
```

### 2. Run Test

```bash
cargo test --features cli-testing
```

### 3. Verify Output

Test passes if output matches `.trycmd` file.

### 4. Update on Changes

If output changes intentionally, update `.trycmd` file.

---

## Best Practices

### 1. Use Golden Files for Stability

```
tests/cli/
├── help.trycmd
├── version.trycmd
└── process.trycmd
```

### 2. Test All Commands

```rust
// Test each subcommand
test_command_help();
test_command_version();
test_command_process();
```

### 3. Isolate Environment

```rust
// Use CliEnvironment for isolation
let mut env = CliEnvironment::new();
// ... test code ...
// Environment automatically restored
```

---

## Troubleshooting

### Error: "cli-testing feature required"

**Cause:** Feature not enabled

**Fix:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["cli-testing"] }
```

### Output Mismatch

**Cause:** Command output changed

**Fix:**
1. Verify change is intentional
2. Update `.trycmd` file if correct
3. Fix command if incorrect

### Environment Variables Not Set

**Cause:** Forgot to call `apply()`

**Fix:**
```rust
let mut env = CliEnvironment::new().set("VAR", "value");
env.apply();  // Don't forget this!
```

---

## Next Steps

After mastering CLI testing, explore:

1. **[Testcontainers](testcontainers_example.md)** - Integration testing
2. **[Snapshot Testing](snapshot_testing.md)** - Output verification
3. **[OTEL/Weaver](otel_weaver_testing.md)** - Observability testing

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [trycmd documentation](https://docs.rs/trycmd/) - Complete trycmd guide
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete API documentation

---

## Reference

### Key Types

- `CliCommandBuilder` - Builder for CLI commands
- `CliAssertions` - Assertions for CLI output
- `CliEnvironment` - Environment variable manager
- `CliTest` - CLI test wrapper

### Key Functions

- `CliCommandBuilder::new(command) -> CliCommandBuilder`
- `CliCommandBuilder::arg(arg)`
- `CliCommandBuilder::env(key, value)`
- `CliCommandBuilder::build() -> String`
- `CliAssertions::assert_output_contains(output, text)`
- `CliEnvironment::new() -> CliEnvironment`
- `CliEnvironment::set(key, value)`
- `CliEnvironment::apply()`

---

**Quality is the default. Prevention beats detection.**

*Example: cli_testing.rs | Version: 1.2.0 | Updated: 2025-11-15*
