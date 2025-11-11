# Root Cause Analysis: Docker Detection Failure

## Problem Definition

**What**: Docker has been stopped the whole time, and testcontainers, otel, weaver, tests, git hooks, and lint never detected or reported this failure.

**Where**: 
- `src/integration/testcontainers/mod.rs` - ContainerClient and GenericContainer creation
- `src/observability/weaver/mod.rs` - Weaver integration
- `tests/common.rs` - Docker availability checks
- `Makefile.toml` - Test tasks
- `.git/hooks/pre-commit` - Pre-commit validation

**When**: Throughout development - Docker was stopped but no system detected it

**Impact**: 
- False confidence - tests may have appeared to pass when Docker was unavailable
- Silent failures - errors may have been masked or ignored
- Wasted time - developer time spent debugging when Docker was simply stopped
- Violates Chicago TDD - tests should verify observable behavior and fail fast

## 5 Whys Analysis

### Why #1: Why didn't testcontainers detect Docker was stopped?

**Answer**: `ContainerClient::new()` and `GenericContainer::new()` don't check Docker availability before attempting container operations. They call `image.start()` which fails if Docker is stopped, but the error is generic ("Failed to start container") and doesn't clearly indicate Docker is stopped.

**Evidence**: 
- `src/integration/testcontainers/mod.rs:176-186` - `GenericContainer::new()` directly calls `image.start()` without checking Docker
- Error handling maps all failures to `TestcontainersError::CreationFailed` without distinguishing Docker unavailability
- Library code assumes Docker is available and relies on testcontainers-rs library to handle errors

**Code Reference**:
```176:186:src/integration/testcontainers/mod.rs
pub fn new(
    _client: &ContainerClient,
    image: &str,
    tag: &str,
) -> TestcontainersResult<Self> {
    let image = GenericImage::new(image, tag);
    let container = image.start().map_err(|e| {
        TestcontainersError::CreationFailed(format!("Failed to start container: {e}"))
    })?;

    Ok(Self { container })
}
```

### Why #2: Why didn't otel/weaver detect Docker was stopped?

**Answer**: Weaver integration doesn't check Docker at all. It only checks if the weaver binary is available. If tests use testcontainers with weaver, they should check Docker first, but the weaver integration itself doesn't check Docker.

**Evidence**:
- `src/observability/weaver/mod.rs:99-129` - `WeaverValidator::start()` checks weaver binary and registry path, but not Docker
- `src/observability/weaver/types.rs:318-353` - `WeaverLiveCheck::start()` checks weaver binary but not Docker
- Weaver integration assumes Docker is available if testcontainers is used, but doesn't validate this assumption

**Code Reference**:
```99:129:src/observability/weaver/mod.rs
pub fn start(&mut self) -> WeaverValidationResult<()> {
    // Check Weaver binary availability
    Self::check_weaver_available()?;

    // Verify registry path exists
    if !self.registry_path.exists() {
        return Err(WeaverValidationError::RegistryNotFound(
            self.registry_path.display().to_string(),
        ));
    }

    // Create Weaver live-check instance
    let registry_str = self.registry_path.to_str().ok_or_else(|| {
        WeaverValidationError::ValidationFailed("Registry path is not valid UTF-8".to_string())
    })?;

    let live_check = WeaverLiveCheck::new()
        .with_registry(registry_str.to_string())
        .with_otlp_port(self.otlp_grpc_port)
        .with_admin_port(self.admin_port)
        .with_inactivity_timeout(DEFAULT_INACTIVITY_TIMEOUT_SECONDS) // 5 minutes (longer for tests)
        .with_format("json".to_string()) // Use JSON format for parsing
        .with_output("./weaver-reports".to_string()); // Output to directory for parsing

    // Start Weaver live-check process
    let process = live_check.start().map_err(WeaverValidationError::ProcessStartFailed)?;

    self.live_check = Some(live_check);
    self.process = Some(process);

    Ok(())
}
```

### Why #3: Why didn't tests detect Docker was stopped?

**Answer**: Tests use `require_docker()` which checks `docker ps`, but:
1. The check uses `is_ok()` which doesn't verify Docker daemon is actually running - it just checks if the command executed
2. If Docker is stopped, `docker ps` might still return success (exit code 0) but with an error message, or it might hang
3. The check is only in tests, not in library code - library code doesn't validate Docker before operations

**Evidence**:
- `tests/common.rs:14-16` - `docker_available()` uses `Command::new("docker").arg("ps").output().is_ok()` which doesn't verify Docker daemon is running
- Tests call `require_docker()` but library code doesn't check Docker before container operations
- The check only verifies command execution, not Docker daemon availability

**Code Reference**:
```14:16:tests/common.rs
pub fn docker_available() -> bool {
    std::process::Command::new("docker").arg("ps").output().is_ok()
}
```

**Problem**: `is_ok()` only checks if the command executed successfully, not if Docker daemon is actually running. If Docker is stopped:
- `docker ps` might return exit code 0 with an error message
- `docker ps` might hang indefinitely
- The check doesn't verify Docker daemon is responding

### Why #4: Why didn't git hooks detect Docker was stopped?

**Answer**: Pre-commit hook doesn't check Docker availability before running tests. It runs `cargo make test-unit` which excludes testcontainers tests, but if integration tests are run, Docker isn't checked first.

**Evidence**:
- `scripts/install-git-hooks.sh:404` - Pre-push hook runs `cargo make test-unit` which excludes testcontainers, but doesn't check Docker for other test types
- Pre-commit hook doesn't check Docker at all - it only runs unit tests
- No Docker health check in pre-commit or pre-push validation

**Code Reference**:
```404:409:scripts/install-git-hooks.sh
# Gate 4: Tests (unit tests only for speed)
# Note: Testcontainers integration tests excluded (too slow, require Docker)
# Run 'cargo make test-integration' manually if needed
echo "Gate 4/5: Unit tests..."
if ! cargo make test-unit 2>&1 | tail -20; then
  echo "❌ ERROR: Unit tests failed"
```

**Problem**: Git hooks assume Docker is available if integration tests are run, but don't validate this assumption. If Docker is stopped and integration tests are run manually, they fail with unclear errors.

### Why #5: Why didn't Makefile.toml detect Docker was stopped?

**Answer**: Test tasks in `Makefile.toml` don't check Docker availability before running integration tests. They rely on tests to check Docker, but if tests don't run or checks fail silently, Docker unavailability isn't detected.

**Evidence**:
- `Makefile.toml:99-113` - `test-integration` task runs testcontainers tests but doesn't check Docker first
- Tasks assume Docker is available and rely on test failures to indicate problems
- No proactive Docker health check before running integration tests

**Code Reference**:
```99:113:Makefile.toml
[tasks.test-integration]
description = "Run testcontainers integration tests only (30s timeout, requires Docker)"
command = "timeout"
args = [
  "30s",
  "cargo",
  "nextest",
  "run",
  "--test",
  "testcontainers",
  "--profile",
  "integration",
  "--features",
  "testcontainers",
]
```

**Problem**: Makefile tasks assume Docker is available and rely on test failures to indicate problems. If Docker is stopped, tests fail with unclear errors, and the root cause (Docker stopped) isn't immediately obvious.

## Root Cause

**Root Cause**: No proactive Docker health checks in library code. Only reactive checks in tests that may not work correctly when Docker is stopped. Library code attempts operations without validating Docker is running, and test checks are insufficient (use `is_ok()` instead of verifying Docker daemon is actually running).

**Contributing Factors**:

1. **Insufficient test checks**: `docker_available()` uses `is_ok()` which doesn't verify Docker daemon is running
2. **No library-level validation**: Library code doesn't check Docker before container operations
3. **Generic error handling**: Container creation errors don't distinguish Docker unavailability
4. **No git hook validation**: Pre-commit doesn't check Docker before running tests
5. **No Makefile validation**: Test tasks don't check Docker before running integration tests
6. **Silent failures**: Tests may skip or fail silently when Docker is unavailable
7. **Assumption-based design**: Code assumes Docker is available without validating this assumption

## Verification

**Test Root Cause Hypothesis**: If we add proactive Docker health checks in library code and improve test checks to verify Docker daemon is actually running, Docker unavailability will be detected immediately.

**Verification Steps**:

1. **Add Docker health check in library code**: `ContainerClient::new()` should verify Docker daemon is running before creating containers
2. **Improve test checks**: `docker_available()` should verify Docker daemon is actually running (not just command execution)
3. **Add Docker check in git hooks**: Pre-commit and pre-push hooks should check Docker before running tests
4. **Add Docker check in Makefile**: Test tasks should check Docker before running integration tests
5. **Improve error messages**: Container creation errors should clearly indicate Docker is stopped

**Expected Outcome**: Docker unavailability will be detected immediately with clear error messages, preventing false confidence and wasted debugging time.

## Prevention

**Prevention Methods**:

1. **Library-level validation**: Add Docker health checks in library code before container operations
2. **Improved test checks**: Verify Docker daemon is running, not just command execution
3. **Git hook validation**: Check Docker before running tests in pre-commit and pre-push
4. **Makefile validation**: Check Docker before running integration tests
5. **Clear error messages**: Distinguish Docker unavailability from other container errors
6. **Fail-fast principle**: Detect Docker unavailability immediately, not after operations fail
7. **Proactive checks**: Validate dependencies before operations, not after failures

## Chicago TDD Violations

This failure violates several Chicago TDD principles as defined in `.cursor/rules/chicago-tdd-standards.mdc`:

### 1. Behavior Verification Violation

**Principle**: "Verify Outputs: Test results and invariants, not implementation details"

**Violation**: `docker_available()` uses `is_ok()` which only checks if the command executed successfully, not if Docker daemon is actually running. This violates the principle of verifying observable behavior.

**Expected**: Tests should verify Docker daemon is responding (observable behavior), not just command execution (implementation detail).

**Code Evidence**:
```14:16:tests/common.rs
pub fn docker_available() -> bool {
    std::process::Command::new("docker").arg("ps").output().is_ok()
}
```

**Fix**: Verify Docker daemon is responding by checking `docker info` output contains "Server Version" or "Docker Root Dir", not just command execution success.

### 2. Fail Fast Violation

**Principle**: "Test Results are Truth: Never trust claims without test verification"

**Violation**: Library code attempts container operations without validating Docker is running first. Operations fail after attempting to start containers, not immediately when Docker is unavailable.

**Expected**: Operations should fail immediately when Docker is unavailable, not after attempting container operations.

**Code Evidence**:
```176:186:src/integration/testcontainers/mod.rs
pub fn new(
    _client: &ContainerClient,
    image: &str,
    tag: &str,
) -> TestcontainersResult<Self> {
    let image = GenericImage::new(image, tag);
    let container = image.start().map_err(|e| {
        TestcontainersError::CreationFailed(format!("Failed to start container: {e}"))
    })?;

    Ok(Self { container })
}
```

**Fix**: Check Docker availability before attempting container operations. Fail immediately with clear error message.

### 3. State Verification Violation

**Principle**: "Classicist Approach: State-based tests, not interaction-based"

**Violation**: Tests don't verify Docker daemon state before operations. They assume Docker is available without verifying the actual state.

**Expected**: Tests should verify Docker daemon state (running/stopped) before operations, not assume availability.

**Code Evidence**: Tests call `require_docker()` which uses insufficient check that doesn't verify Docker daemon state.

**Fix**: Verify Docker daemon state (running and responding) before operations.

### 4. Real Collaborators Violation

**Principle**: "Real Collaborators: Use real objects, minimize mocks"

**Violation**: Tests use command execution checks instead of real Docker daemon verification. The check doesn't actually verify Docker daemon is running.

**Expected**: Tests should use real Docker daemon checks, not just command execution checks.

**Code Evidence**: `docker_available()` checks command execution, not Docker daemon state.

**Fix**: Use real Docker daemon verification (`docker info` with output verification).

### 5. Error Handling Violation

**Principle**: "Verify Outputs: Test results and invariants, not implementation details"

**Violation**: Error messages don't clearly indicate Docker is stopped. Generic errors like "Failed to start container" don't distinguish Docker unavailability from other failures.

**Expected**: Errors should clearly indicate root cause (Docker stopped), not generic failures.

**Code Evidence**:
```176:186:src/integration/testcontainers/mod.rs
let container = image.start().map_err(|e| {
    TestcontainersError::CreationFailed(format!("Failed to start container: {e}"))
})?;
```

**Fix**: Add `DockerUnavailable` error variant and provide clear, actionable error messages.

### 6. Test Results are Truth Violation

**Principle**: "Test Results are Truth: Never trust claims without test verification"

**Violation**: Code assumes Docker is available without verification. Tests may pass when Docker is stopped because checks are insufficient.

**Expected**: Never assume Docker is available. Always verify Docker daemon is running before operations.

**Code Evidence**: Library code doesn't check Docker before operations. Tests use insufficient checks.

**Fix**: Add proactive Docker health checks in library code and improve test checks.

## Summary of Chicago TDD Violations

1. **Behavior Verification**: Tests verify command execution, not Docker daemon state
2. **Fail Fast**: Operations fail after attempting container operations, not immediately
3. **State Verification**: Tests don't verify Docker daemon state before operations
4. **Real Collaborators**: Tests use command execution checks, not real Docker daemon verification
5. **Error Handling**: Errors don't clearly indicate Docker is stopped
6. **Test Results are Truth**: Code assumes Docker is available without verification

All violations stem from the same root cause: insufficient Docker health checks that don't verify Docker daemon is actually running and responding.

## Next Steps

See `DOCKER_DETECTION_FAILURES.md` for detailed catalog of all failures, and `DOCKER_HEALTH_CHECK_IMPLEMENTATION.md` for implementation plan.

