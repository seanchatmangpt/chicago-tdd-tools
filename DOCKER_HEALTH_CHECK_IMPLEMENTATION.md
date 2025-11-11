# Docker Health Check Implementation Plan

This document outlines the implementation plan to add proactive Docker health checks throughout the chicago-tdd-tools project to prevent the failures cataloged in `DOCKER_DETECTION_FAILURES.md`.

## Implementation Goals

1. **Proactive Detection**: Detect Docker unavailability immediately, not after operations fail
2. **Clear Error Messages**: Distinguish Docker unavailability from other container errors
3. **Fail-Fast Principle**: Fail immediately when Docker is unavailable
4. **Chicago TDD Compliance**: Verify observable behavior (Docker daemon running), not just command execution

## Implementation Strategy

### Phase 1: Library-Level Docker Health Checks

#### 1.1: Add Docker Health Check Function

**Location**: `src/integration/testcontainers/mod.rs`

**Implementation**:
- Add `check_docker_available()` function that verifies Docker daemon is actually running
- Use `docker info` or `docker ps` with proper error handling
- Verify Docker daemon is responding, not just command execution
- Return clear error messages when Docker is stopped

**Code Structure**:
```rust
/// Check if Docker daemon is actually running and responding
///
/// This function verifies Docker daemon is running by checking:
/// 1. Docker command exists
/// 2. Docker daemon is responding (not just command execution)
/// 3. Docker daemon is accessible
///
/// # Returns
///
/// `Ok(())` if Docker daemon is running and responding
/// `Err(TestcontainersError)` if Docker is stopped or unavailable
pub fn check_docker_available() -> TestcontainersResult<()> {
    use std::process::Command;
    use std::time::Duration;
    
    // Check Docker command exists
    let docker_check = Command::new("docker")
        .args(["info"])
        .output();
    
    match docker_check {
        Ok(output) => {
            if output.status.success() {
                // Verify Docker daemon is responding by checking output
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("Server Version") || stdout.contains("Docker Root Dir") {
                    Ok(())
                } else {
                    Err(TestcontainersError::InvalidConfig(
                        "Docker daemon is not responding correctly".to_string()
                    ))
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(TestcontainersError::InvalidConfig(format!(
                    "Docker daemon is not running. Error: {}",
                    stderr
                )))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Err(TestcontainersError::InvalidConfig(
                    "Docker command not found. Please install Docker.".to_string()
                ))
            } else {
                Err(TestcontainersError::InvalidConfig(format!(
                    "Failed to check Docker availability: {}",
                    e
                )))
            }
        }
    }
}
```

#### 1.2: Add Docker Check to ContainerClient::new()

**Location**: `src/integration/testcontainers/mod.rs`

**Implementation**:
- Call `check_docker_available()` in `ContainerClient::new()`
- Return error immediately if Docker is unavailable
- Provide clear error message

**Code Changes**:
```rust
impl ContainerClient {
    /// Create a new container client
    ///
    /// # Errors
    ///
    /// Returns error if Docker daemon is not running or unavailable
    pub fn new() -> TestcontainersResult<Self> {
        check_docker_available()?;
        Ok(Self)
    }
}
```

#### 1.3: Add Docker Check to GenericContainer Methods

**Location**: `src/integration/testcontainers/mod.rs`

**Implementation**:
- Add Docker check to `GenericContainer::new()`
- Add Docker check to `GenericContainer::with_env()`
- Add Docker check to `GenericContainer::with_ports()`
- Or rely on `ContainerClient::new()` check (preferred)

**Code Changes**:
- If `ContainerClient::new()` checks Docker, container methods can rely on that
- Otherwise, add `check_docker_available()?` at start of each method

#### 1.4: Improve Error Messages

**Location**: `src/integration/testcontainers/mod.rs`

**Implementation**:
- Add `DockerUnavailable` variant to `TestcontainersError`
- Use clear error messages that distinguish Docker unavailability
- Provide actionable error messages (e.g., "Start Docker Desktop" or "Start Docker daemon")

**Error Variant**:
```rust
#[derive(Error, Debug)]
pub enum TestcontainersError {
    // ... existing variants ...
    /// Docker daemon is not running or unavailable
    #[error("Docker daemon is not running or unavailable: {0}")]
    DockerUnavailable(String),
    // ... other variants ...
}
```

### Phase 2: Improve Test Infrastructure

#### 2.1: Improve docker_available() Function

**Location**: `tests/common.rs`

**Implementation**:
- Replace `is_ok()` check with proper Docker daemon verification
- Use `docker info` instead of `docker ps` for better verification
- Verify Docker daemon is responding, not just command execution

**Code Changes**:
```rust
/// Check if Docker is available for testcontainers tests
///
/// This helper verifies Docker daemon is actually running and responding,
/// not just that the docker command executed successfully.
///
/// # Returns
///
/// `true` if Docker daemon is running and responding, `false` otherwise.
pub fn docker_available() -> bool {
    use std::process::Command;
    
    // Use docker info to verify daemon is running
    let output = match Command::new("docker")
        .args(["info"])
        .output()
    {
        Ok(output) => output,
        Err(_) => return false,
    };
    
    // Verify command succeeded and daemon is responding
    if !output.status.success() {
        return false;
    }
    
    // Verify Docker daemon is actually responding
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("Server Version") || stdout.contains("Docker Root Dir")
}
```

#### 2.2: Improve require_docker() Function

**Location**: `tests/common.rs`

**Implementation**:
- Use improved `docker_available()` function
- Provide clear error message about Docker daemon state
- Suggest how to start Docker

**Code Changes**:
```rust
/// Require Docker to be available, panic if not
///
/// Integration tests that require Docker should use this function.
/// If Docker is not available, the test will fail with a clear error message.
///
/// # Panics
///
/// Panics if Docker is not available, with a message indicating Docker is required
/// and how to start Docker.
pub fn require_docker() {
    if !docker_available() {
        panic!(
            "Docker is required for this test but Docker daemon is not running.\n\
             Please ensure Docker is running:\n\
             - macOS: Start Docker Desktop\n\
             - Linux: Start Docker daemon (sudo systemctl start docker)\n\
             - Windows: Start Docker Desktop"
        );
    }
}
```

### Phase 3: Add Git Hook Validation

#### 3.1: Add Docker Check to Pre-commit Hook

**Location**: `scripts/install-git-hooks.sh`

**Implementation**:
- Add Docker availability check before running tests
- Only check if testcontainers tests might be run
- Provide clear error message if Docker is unavailable

**Code Changes**:
```bash
# Check Docker availability if testcontainers tests might be run
# (Pre-commit excludes testcontainers, but check anyway for consistency)
if command -v docker &> /dev/null; then
  if ! timeout 5s docker info &> /dev/null; then
    echo "⚠️  WARNING: Docker daemon is not running"
    echo "   Integration tests may fail if Docker is required"
  fi
fi
```

#### 3.2: Add Docker Check to Pre-push Hook

**Location**: `scripts/install-git-hooks.sh`

**Implementation**:
- Add Docker availability check before running tests
- Fail if Docker is required but unavailable
- Provide clear error message

**Code Changes**:
```bash
# Check Docker availability before running tests
if command -v docker &> /dev/null; then
  if ! timeout 5s docker info &> /dev/null; then
    echo "❌ ERROR: Docker daemon is not running"
    echo "   Integration tests require Docker to be running"
    echo "   Start Docker Desktop or Docker daemon before pushing"
    exit 1
  fi
else
  echo "⚠️  WARNING: Docker command not found"
  echo "   Integration tests may fail if Docker is required"
fi
```

### Phase 4: Add Makefile Validation

#### 4.1: Add Docker Check Task

**Location**: `Makefile.toml`

**Implementation**:
- Add `docker-check` task that verifies Docker is available
- Use in `test-integration` and `test-all` tasks
- Provide clear error message if Docker is unavailable

**Code Changes**:
```toml
[tasks.docker-check]
description = "Check if Docker daemon is running and available"
command = "timeout"
args = ["5s", "docker", "info"]
ignore_errors = false

[tasks.test-integration]
description = "Run testcontainers integration tests only (30s timeout, requires Docker)"
dependencies = ["docker-check"]
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

[tasks.test-all]
description = "Run all tests (unit + integration)"
dependencies = ["test-unit", "docker-check", "test-integration"]
```

### Phase 5: Add Weaver Integration Docker Checks

#### 5.1: Add Docker Check to WeaverValidator::start()

**Location**: `src/observability/weaver/mod.rs`

**Implementation**:
- Add Docker check if testcontainers feature is enabled
- Check Docker before starting weaver process
- Provide clear error message if Docker is unavailable

**Code Changes**:
```rust
pub fn start(&mut self) -> WeaverValidationResult<()> {
    // Check Weaver binary availability
    Self::check_weaver_available()?;

    // Check Docker availability if testcontainers feature is enabled
    #[cfg(feature = "testcontainers")]
    {
        use crate::testcontainers::check_docker_available;
        check_docker_available().map_err(|e| {
            WeaverValidationError::ValidationFailed(format!(
                "Docker daemon is not running. Weaver integration requires Docker. Error: {}",
                e
            ))
        })?;
    }

    // ... rest of implementation ...
}
```

## Implementation Priority

1. **High Priority**: Library-level Docker checks (Phase 1)
   - Prevents false confidence
   - Provides clear error messages
   - Fail-fast principle

2. **High Priority**: Improve test infrastructure (Phase 2)
   - Fixes insufficient checks
   - Provides better error messages
   - Chicago TDD compliance

3. **Medium Priority**: Git hook validation (Phase 3)
   - Early detection in development workflow
   - Prevents pushing broken code

4. **Medium Priority**: Makefile validation (Phase 4)
   - Prevents running tests when Docker is unavailable
   - Clear error messages

5. **Low Priority**: Weaver integration checks (Phase 5)
   - Only needed if testcontainers feature is enabled
   - Can rely on ContainerClient check

## Testing Strategy

### Unit Tests

1. **Test Docker Check Function**:
   - Test when Docker is running (should succeed)
   - Test when Docker is stopped (should fail with clear error)
   - Test when Docker command doesn't exist (should fail with clear error)

2. **Test ContainerClient::new()**:
   - Test when Docker is running (should succeed)
   - Test when Docker is stopped (should fail with clear error)

3. **Test docker_available()**:
   - Test when Docker is running (should return true)
   - Test when Docker is stopped (should return false)
   - Test when Docker command doesn't exist (should return false)

### Integration Tests

1. **Test Container Creation**:
   - Test container creation when Docker is running (should succeed)
   - Test container creation when Docker is stopped (should fail with clear error)

2. **Test Error Messages**:
   - Verify error messages clearly indicate Docker is stopped
   - Verify error messages provide actionable guidance

## Success Criteria

1. ✅ Docker unavailability is detected immediately
2. ✅ Clear error messages distinguish Docker unavailability from other errors
3. ✅ Fail-fast principle: operations fail immediately when Docker is unavailable
4. ✅ Chicago TDD compliance: verify observable behavior (Docker daemon running)
5. ✅ All failure points from `DOCKER_DETECTION_FAILURES.md` are addressed

## Rollout Plan

1. **Week 1**: Implement Phase 1 (Library-level checks)
2. **Week 1**: Implement Phase 2 (Test infrastructure)
3. **Week 2**: Implement Phase 3 (Git hooks)
4. **Week 2**: Implement Phase 4 (Makefile)
5. **Week 2**: Implement Phase 5 (Weaver integration)

## Monitoring

After implementation, monitor:
- Number of Docker-related errors
- Clarity of error messages
- Time to diagnose Docker issues
- False positive rate (Docker available but reported as unavailable)

## Future Improvements

1. **Docker Health Check Caching**: Cache Docker availability check to avoid repeated checks
2. **Docker Status Monitoring**: Monitor Docker daemon status during long-running tests
3. **Docker Startup Detection**: Detect when Docker starts during test execution
4. **Better Error Messages**: Provide platform-specific error messages (macOS, Linux, Windows)

## Prevention Strategies

To ensure Docker unavailability is detected immediately in the future:

### 1. Proactive Validation at All Layers

**Strategy**: Add Docker health checks at every layer where Docker is used:
- Library code (ContainerClient, GenericContainer)
- Test infrastructure (docker_available, require_docker)
- Git hooks (pre-commit, pre-push)
- Makefile tasks (test-integration, test-all)

**Benefit**: Docker unavailability detected immediately at any layer, not just after operations fail.

### 2. Fail-Fast Principle

**Strategy**: Check Docker availability before operations, not after failures.

**Benefit**: Immediate detection prevents wasted time debugging unclear errors.

### 3. Clear Error Messages

**Strategy**: Distinguish Docker unavailability from other errors with clear, actionable messages.

**Benefit**: Developers immediately know Docker is stopped and how to fix it.

### 4. Chicago TDD Compliance

**Strategy**: Verify observable behavior (Docker daemon running), not just command execution.

**Benefit**: Tests verify actual Docker daemon state, not just command success.

### 5. Validation Checklist

**Strategy**: Before any code that uses Docker:
- [ ] Check Docker availability in library code
- [ ] Verify Docker daemon is running (not just command execution)
- [ ] Provide clear error messages
- [ ] Fail fast when Docker is unavailable

**Benefit**: Systematic approach ensures Docker checks are not forgotten.

### 6. Continuous Monitoring

**Strategy**: Monitor Docker daemon status during long-running tests and operations.

**Benefit**: Detect Docker stopping during test execution, not just at start.

### 7. Documentation

**Strategy**: Document Docker requirements clearly in:
- README.md
- Error messages
- Task descriptions
- Test documentation

**Benefit**: Developers know Docker is required and how to start it.

### 8. CI/CD Integration

**Strategy**: Add Docker health checks to CI/CD pipelines before running integration tests.

**Benefit**: Detect Docker unavailability in CI/CD, not just locally.

### 9. Test Coverage

**Strategy**: Add tests that verify Docker health checks work correctly:
- Test when Docker is running (should succeed)
- Test when Docker is stopped (should fail with clear error)
- Test when Docker command doesn't exist (should fail with clear error)

**Benefit**: Ensure Docker checks work correctly and don't regress.

### 10. Regular Audits

**Strategy**: Regularly audit codebase for Docker usage without health checks.

**Benefit**: Catch new Docker usage without health checks before they cause problems.

