# Test Timeout Enforcement - 1s SLA Compliance

This document describes the multi-layered timeout enforcement system ensuring all tests complete within 1 second.

## Architecture

### Layer 1: Test-Level Timeouts (ntest crate)

**Synchronous Tests**: `chicago_test!` macro automatically adds `#[ntest::timeout(1000)]` attribute to all tests.

```rust
chicago_test!(test_example, {
    // Test automatically has 1s timeout enforced
});
```

**How it works**: The `ntest` crate spawns tests in separate threads with timeout monitoring. Tests exceeding 1s are terminated.

### Layer 2: Async Test Timeouts (tokio::time::timeout)

**Async Tests**: `chicago_async_test!` and `chicago_fixture_test!` macros wrap test bodies with `tokio::time::timeout(Duration::from_secs(1))`.

```rust
chicago_async_test!(test_async_example, {
    // Test body automatically wrapped with 1s timeout
});
```

**How it works**: Tokio's timeout mechanism cancels the future if it doesn't complete within 1s, panicking with "Test exceeded 1s timeout (SLA violation)".

### Layer 3: Test Runner Timeouts (cargo-nextest)

**Configuration**: `.config/nextest.toml` enforces:
- `slow-timeout = { period = "1s", terminate-after = 1 }`
- `test-timeout = "1s"`
- `overhead-timeout = "500ms"`

**How it works**: cargo-nextest monitors test execution and kills slow tests immediately.

### Layer 4: Process-Level Timeouts (Unix timeout command)

**Makefile.toml**: All test tasks wrapped with `timeout 1s` command.

```toml
[tasks.test]
command = "timeout"
args = ["1s", "cargo", "nextest", "run", "--all-features"]
```

**How it works**: Unix `timeout` command kills the entire process if it exceeds 1s.

## Benefits

1. **Defense in Depth**: Multiple layers ensure timeout enforcement even if one layer fails
2. **Fast Test Execution**: cargo-nextest provides parallel execution and better performance
3. **Clear Error Messages**: Each layer provides specific timeout violation messages
4. **SLA Compliance**: Guaranteed 1s maximum test execution time

## Usage

### Running Tests

```bash
# All tests with 1s timeout enforcement
cargo make test

# Unit tests only
cargo make test-unit

# With verbose output
cargo make test-verbose
```

### Fallback

If cargo-nextest is not available, use:
```bash
cargo make test-cargo  # Falls back to standard cargo test
```

## Configuration Files

- **`.config/nextest.toml`**: cargo-nextest timeout configuration
- **`Makefile.toml`**: Process-level timeout wrappers
- **`src/macros.rs`**: Test macro timeout enforcement
- **`Cargo.toml`**: ntest dependency for synchronous test timeouts

## Troubleshooting

### Tests Timing Out

If tests consistently timeout:
1. Check test complexity - optimize slow operations
2. Review async operations - ensure they're not blocking
3. Verify timeout configuration in `.config/nextest.toml`
4. Check for infinite loops or deadlocks

### Timeout Not Enforced

If timeouts aren't working:
1. Verify cargo-nextest is installed: `cargo install cargo-nextest`
2. Check `.config/nextest.toml` exists and is valid
3. Verify ntest dependency in `Cargo.toml`
4. Check macro expansion: `cargo expand --test test_name`

## Performance Impact

- **Overhead**: Minimal (~1-2ms per test for timeout monitoring)
- **Parallel Execution**: cargo-nextest enables parallel test execution
- **Test Isolation**: Each test runs in its own timeout context

## Future Enhancements

- Per-test timeout configuration (override default 1s)
- Timeout statistics and reporting
- Integration with CI/CD timeout monitoring

