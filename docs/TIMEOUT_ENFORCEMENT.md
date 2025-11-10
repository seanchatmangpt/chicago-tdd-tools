# Test Timeout Enforcement - 1s SLA Compliance

This document describes the multi-layered timeout enforcement system ensuring all unit tests complete within 1 second. Integration tests (testcontainers) are excluded from normal iteration and use a separate 30s timeout profile.

**For comprehensive SLA reference**: See [SLA_REFERENCE.md](./SLA_REFERENCE.md) for all timeout SLAs across the project.

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

**Configuration**: `.config/nextest.toml` defines two profiles:

**Default Profile** (unit tests, 1s timeout):
- `slow-timeout = { period = "1s", terminate-after = 1 }`
- `test-timeout = "1s"`
- Excludes testcontainers integration tests (too slow, require Docker)

**Integration Profile** (testcontainers tests, 30s timeout):
- `slow-timeout = { period = "30s", terminate-after = 1 }`
- `test-timeout = "30s"`
- Used only for testcontainers integration tests

**How it works**: cargo-nextest monitors test execution and kills slow tests immediately. Unit tests use the default profile, integration tests use the integration profile.

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
# Unit tests only (fast iteration, excludes testcontainers)
# Uses 1s timeout, excludes slow integration tests
cargo make test

# Unit tests only (explicit)
cargo make test-unit

# Integration tests only (testcontainers, requires Docker)
# Uses 30s timeout, requires Docker to be running
cargo make test-integration

# All tests (unit + integration)
cargo make test-all

# With verbose output (unit tests only)
cargo make test-verbose
```

### Testcontainers Integration Tests

**Excluded from Normal Iteration**: Testcontainers integration tests are excluded from `test` and `test-unit` tasks because they:
- Require Docker to be running
- Are slow (typically 5-30s per test)
- Are not needed for fast feedback during development

**Running Integration Tests**: Use `cargo make test-integration` when you need to verify testcontainers functionality. This task:
- Uses the `integration` profile (30s timeout)
- Requires `--features testcontainers` to be enabled
- Requires Docker to be running

**Mocking Testcontainers**: For fast testing without Docker, disable the `testcontainers` feature:
```bash
# Run tests without testcontainers feature (uses stubs)
cargo test --no-default-features
```

When the feature is disabled, testcontainers stubs return `TestcontainersError::InvalidConfig`, which can be tested without Docker.

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

**Unit Tests Timing Out**:
1. Check test complexity - optimize slow operations
2. Review async operations - ensure they're not blocking
3. Verify timeout configuration in `.config/nextest.toml` (default profile)
4. Check for infinite loops or deadlocks

**Integration Tests Timing Out**:
1. Verify Docker is running: `docker ps`
2. Check container startup time - may need longer timeout
3. Verify integration profile timeout in `.config/nextest.toml` (30s)
4. Consider mocking testcontainers for faster testing (disable feature)

### Testcontainers Tests Not Running

If testcontainers tests aren't running:
1. Verify feature is enabled: `cargo make test-integration --features testcontainers`
2. Check Docker is running: `docker ps`
3. Verify test file exists: `tests/testcontainers_expert_tests.rs`
4. Check feature gate: Tests are gated with `#[cfg(all(feature = "testcontainers", test))]`

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

## Testcontainers Exclusion Rationale

**80/20 Approach**: Testcontainers integration tests are excluded from normal iteration because:
1. **Speed**: Unit tests run in <1s, integration tests take 5-30s
2. **Dependency**: Integration tests require Docker (not always available)
3. **Frequency**: Unit tests run frequently during development, integration tests run occasionally
4. **Mocking**: Testcontainers can be mocked by disabling the feature (stubs return errors)

**When to Run Integration Tests**:
- Before committing major changes to testcontainers module
- In CI/CD pipelines (can use `test-all` task)
- When debugging testcontainers-specific issues
- Before releases (comprehensive validation)

**Normal Development**: Use `cargo make test` (unit tests only) for fast feedback. Integration tests are assumed to work (as per 80/20 principle).

## Future Enhancements

- Per-test timeout configuration (override default 1s)
- Timeout statistics and reporting
- Integration with CI/CD timeout monitoring
- Conditional integration test execution (skip if Docker unavailable)

