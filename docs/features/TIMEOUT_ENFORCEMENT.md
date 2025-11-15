# Timeout Enforcement - SPR

Multi-layered timeout enforcement system ensuring all unit tests complete within 1s. Integration tests excluded from normal iteration, use separate 30s timeout profile.

## Chicago TDD Principles for Timeouts

**Core Principle**: "Better to break fast than freeze forever" - Timeouts prevent infinite hangs and ensure fast feedback.

### Key Principles

1. **Fast Feedback Loop**: Unit tests must complete within 1s for rapid iteration during development
2. **Defense in Depth**: Multiple timeout layers ensure enforcement even if one layer fails
3. **Fail Fast**: Timeouts should fail immediately with clear error messages, not hang indefinitely
4. **Consistent Configuration**: Timeout behavior should be consistent across all test macros
5. **80/20 Approach**: Integration tests excluded from normal iteration (slow, require Docker)

### Timeout SLAs

- **Unit Tests**: 1s per test execution (actual: ~0.05s, well under SLA)
- **Integration Tests**: 30s per test execution (for Docker operations, network calls)

## Architecture

**Layer 1: Test-Level Timeouts (tokio::time::timeout)**: 
- `async_test!`, `fixture_test!`, and `weaver_test!` macros wrap test bodies with `tokio::time::timeout(Duration::from_secs(1))` for unit tests
- Use `async_test_with_timeout!`, `fixture_test_with_timeout!`, or `weaver_test_with_timeout!` for custom timeouts (e.g., 30s for integration tests)
- Tokio's timeout cancels future if it doesn't complete within the specified duration

**Layer 2: Test Runner Timeouts (cargo-nextest)**: 
- `.config/nextest.toml` defines two profiles:
  - **Default Profile** (unit tests, 1s timeout): `slow-timeout = { period = "1s", terminate-after = 1 }`, excludes testcontainers integration tests
  - **Integration Profile** (testcontainers tests, 30s timeout): `slow-timeout = { period = "30s", terminate-after = 1 }`, used only for testcontainers integration tests
- cargo-nextest monitors test execution and kills slow tests immediately

**Layer 3: Process-Level Timeouts (Unix timeout command)**: 
- All test tasks wrapped with `timeout` command in Makefile.toml
- Unit tests: `timeout 10s` (allows for parallel execution while maintaining per-test SLA)
- Integration tests: `timeout 30s`
- Unix `timeout` command kills entire process if it exceeds the timeout

**Note**: Synchronous test macros (`test!`, `otel_test!`) rely on cargo-nextest profile timeouts rather than test-level timeouts. This allows cargo-nextest to apply the correct timeout based on the profile used (1s for unit tests, 30s for integration tests).

## Benefits

**Defense in Depth**: Multiple layers ensure timeout enforcement even if one layer fails. **Fast Test Execution**: cargo-nextest provides parallel execution and better performance. **Clear Error Messages**: Each layer provides specific timeout violation messages. **SLA Compliance**: Guaranteed 1s maximum test execution time for unit tests, 30s for integration tests.

## Usage

**Unit Tests Only**: `cargo make test` (fast iteration, excludes testcontainers, uses 1s timeout). `cargo make test-unit` (explicit). **Integration Tests Only**: `cargo make test-integration` (testcontainers, requires Docker, uses 30s timeout). **All Tests**: `cargo make test-all` (unit + integration). **With Verbose Output**: `cargo make test-verbose` (unit tests only).

### Test Macro Timeout Configuration

**Synchronous Tests** (`test!`, `otel_test!`):
- No test-level timeout (relies on cargo-nextest profiles)
- Unit tests: Use default profile (1s timeout)
- Integration tests: Use integration profile (30s timeout)

**Async Tests** (`async_test!`, `fixture_test!`, `weaver_test!`):
- Default: 1s timeout for unit tests
- For integration tests: Use `*_with_timeout!` variants with 30s timeout
- Example: `async_test_with_timeout!(test_name, 30, { /* test body */ })`

## Testcontainers Integration Tests

**Excluded from Normal Iteration**: Testcontainers integration tests excluded from `test` and `test-unit` tasks because they require Docker, are slow (typically 5-30s per test), are not needed for fast feedback during development.

**Running Integration Tests**: Use `cargo make test-integration` when you need to verify testcontainers functionality. Uses `integration` profile (30s timeout), requires `--features testcontainers` to be enabled, requires Docker to be running.

**Mocking Testcontainers**: For fast testing without Docker, disable the `testcontainers` feature: `cargo make test --no-default-features`. When feature is disabled, testcontainers stubs return `TestcontainersError::InvalidConfig`, which can be tested without Docker.

## External Command Timeout Pattern

**Root Cause Fix**: All external command calls (docker, git, etc.) must have timeout protection to prevent hanging when dependencies are unavailable.

### Pattern Implementation

**Rust Code Pattern** (for external commands in Rust code):
```rust
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const COMMAND_TIMEOUT_MILLIS: u64 = 500;

// Spawn command in thread to enable timeout
let (tx, rx) = mpsc::channel();
let _handle = thread::spawn(move || {
    let output = Command::new("docker").args(["info"]).output();
    tx.send(output).ok();
});

// Wait for result with timeout
let result = match rx.recv_timeout(Duration::from_millis(COMMAND_TIMEOUT_MILLIS)) {
    Ok(result) => result,
    Err(_) => {
        // Timeout - command hung (likely dependency unavailable)
        return Err(Error::Timeout(format!(
            "Command timed out after {}ms",
            COMMAND_TIMEOUT_MILLIS
        )));
    }
};
```

**Makefile Pattern** (for external commands in Makefile.toml):
```toml
[tasks.docker-check]
command = "timeout"
args = ["5s", "docker", "info"]
ignore_errors = false
```

### Examples

**testcontainers/mod.rs** - `check_docker_available()`:
- Uses thread/mpsc pattern with 500ms timeout
- Prevents hanging when Docker daemon is not running
- Returns `DockerUnavailable` error within timeout period

**test_common.inc** - `docker_available()`:
- Uses same thread/mpsc pattern with 500ms timeout
- Consistent timeout pattern across codebase

**Makefile.toml** - `docker-check` task:
- Uses shell `timeout` command with 5s timeout
- Process-level timeout protection

### When to Apply

- **External command calls**: docker, git, curl, etc.
- **Network operations**: HTTP requests, socket connections
- **Any operation that could hang indefinitely**: File I/O on network mounts, database connections, etc.

### Benefits

- **Prevents hangs**: Commands fail fast when dependencies unavailable
- **Fast feedback**: Errors returned within timeout period (500ms-5s)
- **Consistent behavior**: Same timeout pattern across all external commands
- **Fail-fast principle**: Better to break fast than freeze forever

### Timeout Durations

- **Quick checks** (docker info, git status): 500ms (Rust) or 5s (shell)
- **Compilation**: 10s
- **Unit tests**: 1s per test
- **Integration tests**: 30s per test
- **Long operations**: 60s

See: [Root Cause Analysis - Docker Check Freeze](../analysis/ROOT_CAUSE_ANALYSIS_DOCKER_CHECK_FREEZE.md) for details on the timeout fix.

## Fallback

If cargo-nextest is not available, use: `cargo make test-cargo` (falls back to standard cargo test).

## Configuration Files

`.config/nextest.toml`: cargo-nextest timeout configuration. `Makefile.toml`: Process-level timeout wrappers. `src/core/macros/test.rs`: Test macro timeout enforcement. `Cargo.toml`: ntest dependency (not currently used, kept for backward compatibility).

## Troubleshooting

**Unit Tests Timing Out**: Check test complexity (optimize slow operations), review async operations (ensure they're not blocking), verify timeout configuration in `.config/nextest.toml` (default profile), check for infinite loops or deadlocks.

**Integration Tests Timing Out**: Verify Docker is running (`docker ps`), check container startup time (may need longer timeout), verify integration profile timeout in `.config/nextest.toml` (30s), consider mocking testcontainers for faster testing (disable feature), use `*_with_timeout!` macro variants with 30s timeout for async integration tests.

**Testcontainers Tests Not Running**: Verify feature is enabled (`cargo make test-integration --features testcontainers`), check Docker is running (`docker ps`), verify test file exists (`tests/testcontainers_expert_tests.rs`), check feature gate (tests are gated with `#[cfg(all(feature = "testcontainers", test))]`).

**Timeout Not Enforced**: Verify cargo-nextest is installed (`cargo install cargo-nextest`), check `.config/nextest.toml` exists and is valid, check macro expansion (`cargo expand --test test_name`), for async tests verify `tokio::time::timeout` is being applied.

## Performance Impact

**Overhead**: Minimal (~1-2ms per test for timeout monitoring). **Parallel Execution**: cargo-nextest enables parallel test execution. **Test Isolation**: Each test runs in its own timeout context.

## Testcontainers Exclusion Rationale

**80/20 Approach**: Testcontainers integration tests excluded from normal iteration because: Speed (unit tests run in <1s, integration tests take 5-30s), Dependency (integration tests require Docker, not always available), Frequency (unit tests run frequently during development, integration tests run occasionally), Mocking (testcontainers can be mocked by disabling the feature, stubs return errors).

**When to Run Integration Tests**: Before committing major changes to testcontainers module, in CI/CD pipelines (can use `test-all` task), when debugging testcontainers-specific issues, before releases (comprehensive validation).

**Normal Development**: Use `cargo make test` (unit tests only) for fast feedback. Integration tests are assumed to work (as per 80/20 principle).

## Summary

**Key Associations**: Timeout Enforcement = Multi-Layer = Defense in Depth. Unit Tests = 1s SLA = Fast Feedback. Integration Tests = 30s SLA = Docker Operations. Testcontainers = Excluded = 80/20 Approach.

**Pattern**: All tests have timeout protection at multiple layers (test-level for async tests, runner-level, process-level). Unit tests use 1s timeout for fast feedback. Integration tests use 30s timeout for Docker operations. Testcontainers excluded from normal iteration for speed.

**Chicago TDD Alignment**: Timeouts enforce "better to break fast than freeze forever" principle, enable fast feedback loop (1s unit tests), maintain defense in depth (multiple layers), and support 80/20 approach (integration tests excluded from normal iteration).
