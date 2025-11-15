# Building a CLI Application

Complete example of testing a CLI application with Chicago TDD Tools.

## Project Structure

```
myapp/
├── src/
│   ├── main.rs         # CLI entry point
│   └── commands/
│       ├── list.rs     # List command
│       ├── add.rs      # Add command
│       └── delete.rs   # Delete command
└── tests/
    ├── cli_tests.rs    # CLI integration tests
    └── commands_tests.rs
```

## Testing CLI Commands

### Example: List Command Test

```rust
test!(test_list_command, {
    let output = CliTest::new("myapp", vec!["list"])
        .run()?;

    assert!(output.contains("Item"));
    assert!(output.exit_code == 0);
});
```

### Example: Add Command Test

```rust
test!(test_add_command, {
    let output = CliTest::new("myapp", vec![
        "add",
        "--name", "New Item",
        "--priority", "high"
    ]).run()?;

    assert!(output.contains("Added"));
    assert!(output.exit_code == 0);
});
```

### Example: Error Handling

```rust
test!(test_invalid_command, {
    let result = CliTest::new("myapp", vec!["invalid"])
        .run();

    assert!(result.is_err());
});
```

## Best Practices for CLI Testing

✅ **Do:**
- Test all commands
- Test argument combinations
- Test error cases
- Use snapshots for complex output
- Test environment variables

❌ **Don't:**
- Hard-code full output
- Test shell integration
- Test external tools

See: [CLI Testing](../advanced/cli-testing.md)
