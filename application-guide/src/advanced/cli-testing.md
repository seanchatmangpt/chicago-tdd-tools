# CLI Testing

> 🔧 **HOW-TO** | 📚 **REFERENCE** | Test command-line interfaces with golden files

Test command-line interfaces using golden files (`.trycmd`) to verify commands work correctly.

## Why CLI Testing?

CLIs are complex because they involve:
- Argument parsing
- Environment variables
- Output formatting
- Exit codes
- Error messages

Golden file testing captures all of this:

```rust
// ✅ Golden file testing
test!(test_cli, {
    let output = run_command("myapp", vec!["list", "--verbose"]);
    assert_matches!(output, "cli_list_verbose");  // .trycmd file
});
```

The `.trycmd` file contains:
```
$ myapp list --verbose
stdout: Item 1
        Item 2
exit-code: 0
```

## Basic CLI Testing

### Creating a Test Command

```rust
use chicago_tdd_tools::cli::*;

test!(test_cli_basic, {
    let output = CliTest::new("myapp", vec!["help"])
        .run()?;

    assert!(output.contains("Usage:"));
    assert!(output.contains("Options:"));
});
```

### Testing with Arguments

```rust
test!(test_cli_with_args, {
    let output = CliTest::new("myapp", vec![
        "process",
        "--input", "data.txt",
        "--output", "result.txt",
        "--verbose"
    ]).run()?;

    assert!(output.contains("Processing"));
});
```

### Testing Environment Variables

```rust
test!(test_cli_with_env, {
    let output = CliTest::new("myapp", vec!["list"])
        .env("LOG_LEVEL", "DEBUG")
        .env("TIMEOUT", "30")
        .run()?;

    assert!(output.contains("DEBUG"));
});
```

## Golden File Format (.trycmd)

Golden files store expected output:

```
$ myapp list
Item 1
Item 2
Item 3

$ myapp list --filter active
Item 1
Item 3
```

### Command Line

```
$ myapp [args]
```

### Output

```
stdout:
Actual command output
Goes here

stderr:
Error output if applicable
```

### Exit Code

```
exit-code: 0  (Success)
exit-code: 1  (Failure)
```

## Real-World Example: File Tool

```rust
test!(test_file_commands, {
    // List files
    let list_output = CliTest::new("filetool", vec!["list"])
        .run()?;
    assert!(list_output.contains("data.txt"));

    // Copy file
    let copy_output = CliTest::new("filetool", vec![
        "copy",
        "source.txt",
        "dest.txt"
    ]).run()?;
    assert!(copy_output.contains("Copied"));

    // Delete file
    let del_output = CliTest::new("filetool", vec![
        "delete",
        "old.txt"
    ]).run()?;
    assert!(del_output.contains("Deleted"));
});
```

## Real-World Example: Configuration Tool

```rust
test!(test_config_commands, {
    // Get config
    let output = CliTest::new("config", vec!["get", "database.host"])
        .env("CONFIG_PATH", "./config.toml")
        .run()?;
    assert!(output.contains("localhost"));

    // Set config
    let output = CliTest::new("config", vec!["set", "database.port", "5433"])
        .env("CONFIG_PATH", "./config.toml")
        .run()?;
    assert!(output.contains("Updated"));

    // List all config
    let output = CliTest::new("config", vec!["list"])
        .run()?;
    assert!(output.contains("database.host"));
});
```

## Error Testing

### Command Failures

```rust
test!(test_cli_errors, {
    // Wrong arguments
    let output = CliTest::new("myapp", vec!["invalid-command"])
        .run();

    assert!(output.is_err());  // Command failed
});
```

### Exit Codes

```rust
test!(test_exit_codes, {
    // Success
    let result = CliTest::new("myapp", vec!["list"]).run()?;
    assert_eq!(result.exit_code, 0);

    // Failure
    let result = CliTest::new("myapp", vec!["error"]).run()?;
    assert_ne!(result.exit_code, 0);
});
```

## Assertion Helpers

### Contains

```rust
test!(test_cli_contains, {
    let output = CliTest::new("myapp", vec!["help"]).run()?;
    assert!(output.contains("Usage:"));
});
```

### Matches Pattern

```rust
test!(test_cli_pattern, {
    let output = CliTest::new("myapp", vec!["version"]).run()?;
    assert!(output.contains("v1."));  // Matches v1.0, v1.1, etc.
});
```

### Snapshot

```rust
test!(test_cli_snapshot, {
    let output = CliTest::new("myapp", vec!["help"]).run()?;
    assert_matches!(output, "myapp_help");  // Golden file
});
```

## Comprehensive CLI Test

```rust
test!(test_cli_comprehensive, {
    // Test 1: Help works
    let help = CliTest::new("mytool", vec!["help"]).run()?;
    assert!(help.contains("Usage:"));

    // Test 2: List works
    let list = CliTest::new("mytool", vec!["list"]).run()?;
    assert!(list.contains("Item"));

    // Test 3: Filter works
    let filtered = CliTest::new("mytool", vec![
        "list",
        "--filter", "active"
    ]).run()?;
    assert!(filtered.contains("Item 1"));

    // Test 4: Sort works
    let sorted = CliTest::new("mytool", vec![
        "list",
        "--sort", "name"
    ]).run()?;
    let lines: Vec<_> = sorted.lines().collect();
    assert!(lines.len() >= 2);

    // Test 5: Output format
    let json = CliTest::new("mytool", vec![
        "list",
        "--format", "json"
    ]).run()?;
    assert!(json.contains("{"));
});
```

## Best Practices

✅ **Do:**
- Test all major commands
- Test error cases
- Test environment variables
- Test output format
- Use golden files for complex output

❌ **Don't:**
- Hard-code full output (use snapshots)
- Test implementation details
- Ignore error exit codes
- Use shell pipes in tests
- Test external commands

## When to Use CLI Testing

✅ **Use for:**
- CLI applications
- Command subcommands
- Argument parsing
- Output formatting
- Error messages

❌ **Don't use for:**
- Library functions (use unit tests)
- Web services (use integration tests)
- Complex pipelines (too fragile)

## Combining with Other Techniques

### CLI + Snapshots

```rust
test!(test_cli_snapshot, {
    let output = CliTest::new("myapp", vec!["help"]).run()?;
    assert_matches!(output, "help_output");  // Snapshot
});
```

### CLI + Properties

```rust
test!(test_cli_properties, {
    let strategy = ProptestStrategy::new().with_cases(100);

    strategy.test(any::<String>(), |cmd| {
        let output = CliTest::new("myapp", vec![&cmd]).run();
        // Property: Command doesn't crash
        true  // If crashes, test fails
    });
});
```

## Troubleshooting

### Test Fails with Different Output

Check for:
- Timestamps (use `[TIMESTAMP]`)
- UUIDs (use `[UUID]`)
- Paths (use relative paths)

### Command Not Found

Ensure binary is built:

```bash
cargo build --bin myapp
# Then tests can run it
```

### Flaky Tests

Normalize output:

```rust
let output = CliTest::new("myapp", vec!["status"]).run()?;
let normalized = output.replace("2024-11-15", "[DATE]");
assert!(normalized.contains("Started on [DATE]"));
```

## Next Steps

Learn concurrency testing: [Concurrency Testing](concurrency-testing.md)

---

## Summary

CLI testing:
- ✅ Tests command-line interfaces
- ✅ Uses golden files
- ✅ Detects output changes
- ✅ Verifies exit codes

Perfect for CLI applications and scripts.

