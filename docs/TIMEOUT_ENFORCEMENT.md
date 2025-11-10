# Timeout Enforcement - SPR

Multi-layered timeout enforcement system ensuring all unit tests complete within 1s. Integration tests excluded from normal iteration, use separate 30s timeout profile.

## Architecture

**Layer 1: Test-Level Timeouts (ntest crate)**: `chicago_test!` macro automatically adds `#[ntest::timeout(1000)]` attribute. ntest spawns tests in separate threads with timeout monitoring. Tests exceeding 1s are terminated.

**Layer 2: Async Test Timeouts (tokio::time::timeout)**: `chicago_async_test!` and `chicago_fixture_test!` macros wrap test bodies with `tokio::time::timeout(Duration::from_secs(1))`. Tokio's timeout cancels future if it doesn't complete within 1s.

**Layer 3: Test Runner Timeouts (cargo-nextest)**: `.config/nextest.toml` defines two profiles. Default Profile (unit tests, 1s timeout): `slow-timeout = { period = "1s", terminate-after = 1 }`, `test-timeout = "1s"`, excludes testcontainers integration tests. Integration Profile (testcontainers tests, 30s timeout): `slow-timeout = { period = "30s", terminate-after = 1 }`, `test-timeout = "30s"`, used only for testcontainers integration tests. cargo-nextest monitors test execution and kills slow tests immediately.

**Layer 4: Process-Level Timeouts (Unix timeout command)**: All test tasks wrapped with `timeout 1s` command in Makefile.toml. Unix `timeout` command kills entire process if it exceeds 1s.

## Benefits

**Defense in Depth**: Multiple layers ensure timeout enforcement even if one layer fails. **Fast Test Execution**: cargo-nextest provides parallel execution and better performance. **Clear Error Messages**: Each layer provides specific timeout violation messages. **SLA Compliance**: Guaranteed 1s maximum test execution time.

## Usage

**Unit Tests Only**: `cargo make test` (fast iteration, excludes testcontainers, uses 1s timeout). `cargo make test-unit` (explicit). **Integration Tests Only**: `cargo make test-integration` (testcontainers, requires Docker, uses 30s timeout). **All Tests**: `cargo make test-all` (unit + integration). **With Verbose Output**: `cargo make test-verbose` (unit tests only).

## Testcontainers Integration Tests

**Excluded from Normal Iteration**: Testcontainers integration tests excluded from `test` and `test-unit` tasks because they require Docker, are slow (typically 5-30s per test), are not needed for fast feedback during development.

**Running Integration Tests**: Use `cargo make test-integration` when you need to verify testcontainers functionality. Uses `integration` profile (30s timeout), requires `--features testcontainers` to be enabled, requires Docker to be running.

**Mocking Testcontainers**: For fast testing without Docker, disable the `testcontainers` feature: `cargo test --no-default-features`. When feature is disabled, testcontainers stubs return `TestcontainersError::InvalidConfig`, which can be tested without Docker.

## Fallback

If cargo-nextest is not available, use: `cargo make test-cargo` (falls back to standard cargo test).

## Configuration Files

`.config/nextest.toml`: cargo-nextest timeout configuration. `Makefile.toml`: Process-level timeout wrappers. `src/macros.rs`: Test macro timeout enforcement. `Cargo.toml`: ntest dependency for synchronous test timeouts.

## Troubleshooting

**Unit Tests Timing Out**: Check test complexity (optimize slow operations), review async operations (ensure they're not blocking), verify timeout configuration in `.config/nextest.toml` (default profile), check for infinite loops or deadlocks.

**Integration Tests Timing Out**: Verify Docker is running (`docker ps`), check container startup time (may need longer timeout), verify integration profile timeout in `.config/nextest.toml` (30s), consider mocking testcontainers for faster testing (disable feature).

**Testcontainers Tests Not Running**: Verify feature is enabled (`cargo make test-integration --features testcontainers`), check Docker is running (`docker ps`), verify test file exists (`tests/testcontainers_expert_tests.rs`), check feature gate (tests are gated with `#[cfg(all(feature = "testcontainers", test))]`).

**Timeout Not Enforced**: Verify cargo-nextest is installed (`cargo install cargo-nextest`), check `.config/nextest.toml` exists and is valid, verify ntest dependency in `Cargo.toml`, check macro expansion (`cargo expand --test test_name`).

## Performance Impact

**Overhead**: Minimal (~1-2ms per test for timeout monitoring). **Parallel Execution**: cargo-nextest enables parallel test execution. **Test Isolation**: Each test runs in its own timeout context.

## Testcontainers Exclusion Rationale

**80/20 Approach**: Testcontainers integration tests excluded from normal iteration because: Speed (unit tests run in <1s, integration tests take 5-30s), Dependency (integration tests require Docker, not always available), Frequency (unit tests run frequently during development, integration tests run occasionally), Mocking (testcontainers can be mocked by disabling the feature, stubs return errors).

**When to Run Integration Tests**: Before committing major changes to testcontainers module, in CI/CD pipelines (can use `test-all` task), when debugging testcontainers-specific issues, before releases (comprehensive validation).

**Normal Development**: Use `cargo make test` (unit tests only) for fast feedback. Integration tests are assumed to work (as per 80/20 principle).

## Summary

**Key Associations**: Timeout Enforcement = Multi-Layer = Defense in Depth. Unit Tests = 1s SLA = Fast Feedback. Integration Tests = 30s SLA = Docker Operations. Testcontainers = Excluded = 80/20 Approach.

**Pattern**: All tests have timeout protection at multiple layers (test-level, async-level, runner-level, process-level). Unit tests use 1s timeout for fast feedback. Integration tests use 30s timeout for Docker operations. Testcontainers excluded from normal iteration for speed.
