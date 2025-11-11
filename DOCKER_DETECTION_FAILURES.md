# Docker Detection Failures - Complete Catalog

This document catalogs all specific failures where Docker being stopped was not detected by various systems in the chicago-tdd-tools project.

## Overview

Docker was stopped throughout development, but no system detected or reported this failure. This document catalogs each failure point and explains why detection failed.

## Failure Categories

1. [Testcontainers Library Code](#testcontainers-library-code)
2. [OTEL/Weaver Integration](#otelweaver-integration)
3. [Test Infrastructure](#test-infrastructure)
4. [Git Hooks](#git-hooks)
5. [Makefile Tasks](#makefile-tasks)
6. [Lint System](#lint-system)

---

## Testcontainers Library Code

### Failure: ContainerClient::new() doesn't check Docker

**Location**: `src/integration/testcontainers/mod.rs:131-141`

**Failure**: `ContainerClient::new()` creates a client without checking if Docker is running.

**Code**:
```131:141:src/integration/testcontainers/mod.rs
pub struct ContainerClient;

impl ContainerClient {
    /// Create a new container client
    pub fn new() -> Self {
        Self
    }

    /// Get a reference for compatibility (no-op in minimal implementation)
    pub fn client(&self) -> &Self {
        self
    }
}
```

**Why it failed**: No Docker health check before creating client. Client creation succeeds even if Docker is stopped.

**Impact**: False confidence - code appears to work until container operations fail.

### Failure: GenericContainer::new() doesn't check Docker

**Location**: `src/integration/testcontainers/mod.rs:176-186`

**Failure**: `GenericContainer::new()` attempts container creation without checking Docker availability.

**Code**:
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

**Why it failed**: 
- No Docker health check before `image.start()`
- Generic error message doesn't distinguish Docker unavailability
- Error handling maps all failures to `CreationFailed` without root cause analysis
- Assumes Docker is available without validation
- Reactive approach - waits for testcontainers-rs library to fail

**Root Cause Analysis**:
1. **Assumption-Based Design**: Assumes Docker is available without validation
2. **Reactive Error Handling**: Relies on testcontainers-rs library to fail, then maps error generically
3. **No Proactive Validation**: Doesn't check Docker before attempting container operations
4. **Generic Error Messages**: All failures mapped to `CreationFailed` without distinguishing Docker unavailability
5. **No Fail-Fast**: Doesn't detect Docker unavailability immediately

**Impact**: 
- Unclear error messages when Docker is stopped
- Difficult to diagnose root cause
- Wasted debugging time
- False confidence - code appears to work until container operations fail

### Failure: GenericContainer::with_env() doesn't check Docker

**Location**: `src/integration/testcontainers/mod.rs:209-226`

**Failure**: `GenericContainer::with_env()` attempts container creation without checking Docker.

**Code**:
```209:226:src/integration/testcontainers/mod.rs
pub fn with_env(
    _client: &ContainerClient,
    image: &str,
    tag: &str,
    env_vars: HashMap<String, String>,
) -> TestcontainersResult<Self> {
    let image = GenericImage::new(image, tag);
    // Build container request with all env vars
    // Move env_vars into the request (no need to clone since we consume the HashMap)
    let mut request: testcontainers::core::ContainerRequest<GenericImage> = image.into();
    for (key, value) in env_vars {
        request = request.with_env_var(key, value);
    }
    let container = request.start().map_err(|e| {
        TestcontainersError::CreationFailed(format!("Failed to start container: {e}"))
    })?;

    Ok(Self { container })
}
```

**Why it failed**: Same as `GenericContainer::new()` - no Docker check before container operations.

**Impact**: Same as above - unclear errors, difficult diagnosis.

### Failure: GenericContainer::with_ports() doesn't check Docker

**Location**: `src/integration/testcontainers/mod.rs:241-255`

**Failure**: `GenericContainer::with_ports()` attempts container creation without checking Docker.

**Code**:
```241:255:src/integration/testcontainers/mod.rs
pub fn with_ports(
    _client: &ContainerClient,
    image: &str,
    tag: &str,
    ports: &[u16],
) -> TestcontainersResult<Self> {
    let mut image = GenericImage::new(image, tag);
    for port in ports {
        image = image.with_exposed_port(ContainerPort::Tcp(*port));
    }
    let container = image.start().map_err(|e| {
        TestcontainersError::CreationFailed(format!("Failed to start container: {e}"))
    })?;

    Ok(Self { container })
}
```

**Why it failed**: Same pattern - no Docker check before container operations.

**Impact**: Same as above.

---

## OTEL/Weaver Integration

### Failure: WeaverValidator::start() doesn't check Docker

**Location**: `src/observability/weaver/mod.rs:99-129`

**Failure**: `WeaverValidator::start()` checks weaver binary and registry path, but not Docker.

**Code**:
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

**Why it failed**: 
- Checks weaver binary availability
- Checks registry path exists
- **Does NOT check Docker availability**
- If testcontainers is used with weaver, Docker check should happen first
- Assumes Docker is available if testcontainers is used, but doesn't validate this assumption

**Root Cause Analysis**:
1. **Incomplete Validation**: Validates weaver binary and registry path, but not Docker
2. **Assumption-Based**: Assumes Docker is available if testcontainers is used, but doesn't validate
3. **Missing Dependency Check**: Doesn't check Docker even though it's a dependency when testcontainers is used
4. **No Fail-Fast**: Doesn't detect Docker unavailability before starting weaver process
5. **Inconsistent Validation**: Validates some dependencies (weaver binary, registry path) but not others (Docker)

**Impact**: 
- Weaver integration fails silently if Docker is stopped
- Unclear error messages
- Difficult to diagnose root cause
- False confidence - weaver binary and registry checks pass, but Docker is stopped

### Failure: WeaverLiveCheck::start() doesn't check Docker

**Location**: `src/observability/weaver/types.rs:318-353`

**Failure**: `WeaverLiveCheck::start()` checks weaver binary but not Docker.

**Code**:
```318:353:src/observability/weaver/types.rs
pub fn start(&self) -> Result<Child, String> {
    // Check Weaver binary availability first (may trigger runtime download)
    Self::check_weaver_available()?;
    use std::process::Command;

    // Find weaver binary path
    let weaver_binary = Self::find_weaver_binary()
        .ok_or_else(|| "Weaver binary not found after check".to_string())?;

    let mut cmd = Command::new(&weaver_binary);

    cmd.args(["registry", "live-check"]);

    if let Some(ref registry) = self.registry_path {
        cmd.args(["--registry", registry]);
    }

    cmd.args(["--otlp-grpc-address", &self.otlp_grpc_address]);
    cmd.args(["--otlp-grpc-port", &self.otlp_grpc_port.to_string()]);
    cmd.args(["--admin-port", &self.admin_port.to_string()]);
    cmd.args(["--inactivity-timeout", &self.inactivity_timeout.to_string()]);
    cmd.args(["--format", &self.format]);

    if let Some(ref output) = self.output {
        cmd.args(["--output", output]);
    }

    cmd.spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Weaver binary not found in PATH. Install with: ./scripts/install-weaver.sh or cargo install weaver".to_string()
            } else {
                format!("Failed to start Weaver live-check: {e}. Ensure Weaver is installed and in PATH.")
            }
        })
}
```

**Why it failed**: 
- Checks weaver binary availability
- **Does NOT check Docker availability**
- Assumes Docker is available if testcontainers is used

**Impact**: Same as above - unclear errors, difficult diagnosis.

---

## Test Infrastructure

### Failure: docker_available() uses insufficient check

**Location**: `tests/common.rs:14-16`

**Failure**: `docker_available()` uses `is_ok()` which doesn't verify Docker daemon is actually running.

**Code**:
```14:16:tests/common.rs
pub fn docker_available() -> bool {
    std::process::Command::new("docker").arg("ps").output().is_ok()
}
```

**Why it failed**:
1. `is_ok()` only checks if command executed successfully, not if Docker daemon is running
2. If Docker is stopped, `docker ps` might:
   - Return exit code 0 with an error message
   - Hang indefinitely
   - Return success but Docker daemon isn't actually running
3. Doesn't verify Docker daemon is responding

**Impact**:
- False positives - `docker_available()` returns `true` even when Docker is stopped
- Tests may skip or fail silently
- False confidence in test results

### Failure: require_docker() relies on insufficient check

**Location**: `tests/common.rs:36-40`

**Failure**: `require_docker()` relies on `docker_available()` which uses insufficient check.

**Code**:
```36:40:tests/common.rs
pub fn require_docker() {
    if !docker_available() {
        panic!("Docker is required for this test but is not available. Please ensure Docker is running.");
    }
}
```

**Why it failed**: 
- Relies on `docker_available()` which doesn't verify Docker daemon is running
- May panic incorrectly or not panic when Docker is stopped
- Doesn't provide clear error message about Docker daemon state

**Impact**: 
- Tests may pass when Docker is stopped
- Tests may fail with unclear errors
- Difficult to diagnose root cause

### Failure: Tests don't check Docker in library code

**Location**: All test files using testcontainers

**Failure**: Tests call `require_docker()` but library code doesn't check Docker before operations.

**Evidence**:
- `tests/testcontainers/tests.rs` - Multiple tests call `require_docker()` but library code doesn't check
- `tests/testcontainers/integration.rs` - Integration tests call `require_docker()` but library code doesn't check
- `tests/testcontainers/weaver.rs` - Weaver tests call `require_docker()` but library code doesn't check

**Why it failed**: 
- Tests check Docker availability, but library code doesn't
- If tests don't run or checks fail silently, Docker unavailability isn't detected
- Library code assumes Docker is available without validation

**Impact**: 
- False confidence when tests don't run
- Silent failures when Docker is stopped
- Difficult to diagnose root cause

---

## Git Hooks

### Failure: Pre-commit hook doesn't check Docker

**Location**: `scripts/install-git-hooks.sh:22-257`

**Failure**: Pre-commit hook doesn't check Docker availability before running tests.

**Code**: Pre-commit hook runs `cargo make test-unit` which excludes testcontainers, but doesn't check Docker for other test types.

**Why it failed**:
- Pre-commit hook focuses on fast checks (2-5s target)
- Excludes testcontainers tests (too slow)
- **Does NOT check Docker availability**
- If integration tests are run manually, Docker isn't checked first
- Assumes Docker is available if needed, but doesn't validate this assumption

**Root Cause Analysis**:
1. **Design Decision**: Pre-commit hook excludes testcontainers tests for speed (2-5s target)
2. **Missing Validation**: No Docker check even though comments acknowledge Docker requirement
3. **Assumption-Based**: Assumes Docker is available if needed, but doesn't verify
4. **No Fail-Fast**: Doesn't detect Docker unavailability early in development workflow

**Impact**:
- Pre-commit passes even when Docker is stopped
- Manual integration test runs fail with unclear errors
- Difficult to diagnose root cause
- False confidence - pre-commit passes but integration tests fail later

### Failure: Pre-push hook doesn't check Docker

**Location**: `scripts/install-git-hooks.sh:260-428`

**Failure**: Pre-push hook doesn't check Docker availability before running tests.

**Code**:
```404:409:scripts/install-git-hooks.sh
# Gate 4: Tests (unit tests only for speed)
# Note: Testcontainers integration tests excluded (too slow, require Docker)
# Run 'cargo make test-integration' manually if needed
echo "Gate 4/5: Unit tests..."
if ! cargo make test-unit 2>&1 | tail -20; then
  echo "❌ ERROR: Unit tests failed"
```

**Why it failed**:
- Pre-push hook runs `cargo make test-unit` which excludes testcontainers
- **Does NOT check Docker availability**
- Comments indicate integration tests should be run manually, but Docker isn't checked
- Assumes Docker is available if needed, but doesn't validate this assumption

**Root Cause Analysis**:
1. **Design Decision**: Pre-push hook excludes testcontainers tests for speed
2. **Missing Validation**: No Docker check even though comments acknowledge Docker requirement
3. **Assumption-Based**: Assumes Docker is available if needed, but doesn't verify
4. **No Fail-Fast**: Doesn't detect Docker unavailability before push
5. **Manual Process**: Relies on manual integration test runs, but doesn't check Docker for those

**Impact**:
- Pre-push passes even when Docker is stopped
- Manual integration test runs fail with unclear errors
- Difficult to diagnose root cause
- False confidence - pre-push passes but integration tests fail later
- Wasted time - developer pushes code, then discovers Docker is stopped

---

## Makefile Tasks

### Failure: test-integration task doesn't check Docker

**Location**: `Makefile.toml:99-113`

**Failure**: `test-integration` task runs testcontainers tests but doesn't check Docker first.

**Code**:
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

**Why it failed**:
- Description says "requires Docker" but doesn't check Docker availability
- Assumes Docker is available and relies on test failures to indicate problems
- No proactive Docker health check before running tests
- Reactive approach - waits for test failures instead of proactive validation

**Root Cause Analysis**:
1. **Documentation vs Implementation Gap**: Description acknowledges Docker requirement but implementation doesn't validate it
2. **Reactive Design**: Relies on test failures to indicate Docker unavailability
3. **No Fail-Fast**: Doesn't detect Docker unavailability before running tests
4. **Assumption-Based**: Assumes Docker is available without validation

**Impact**:
- Tests fail with unclear errors when Docker is stopped
- Difficult to diagnose root cause
- Wasted debugging time
- False confidence - task description says Docker is required but doesn't validate

### Failure: test-all task doesn't check Docker

**Location**: `Makefile.toml:115-117`

**Failure**: `test-all` task runs both unit and integration tests but doesn't check Docker.

**Code**:
```115:117:Makefile.toml
[tasks.test-all]
description = "Run all tests (unit + integration)"
dependencies = ["test-unit", "test-integration"]
```

**Why it failed**: 
- Depends on `test-integration` which doesn't check Docker
- No Docker check before running integration tests
- Assumes Docker is available without validation

**Root Cause Analysis**:
1. **Dependency Chain**: Depends on `test-integration` which doesn't check Docker
2. **No Validation**: No Docker check in dependency chain
3. **Assumption-Based**: Assumes Docker is available without validation

**Impact**: Same as above - unclear errors, difficult diagnosis.

---

## Lint System

### Failure: Clippy doesn't check Docker (and shouldn't)

**Location**: N/A

**Failure**: Clippy doesn't check Docker availability.

**Why it failed**: 
- Clippy is a static analysis tool - it doesn't check runtime dependencies
- **This is correct behavior** - Clippy shouldn't check Docker
- However, this means Docker unavailability isn't detected by lint system

**Root Cause Analysis**:
1. **Tool Limitation**: Clippy is a static analysis tool, not a runtime validation tool
2. **Correct Behavior**: Clippy shouldn't check Docker - this is by design
3. **Gap in Validation**: Lint system doesn't detect Docker unavailability, but this is expected
4. **Not a Bug**: This is not a bug, but a gap in the overall validation system

**Impact**: 
- Lint passes even when Docker is stopped
- No early detection of Docker unavailability
- False confidence in code quality
- Gap in validation system - Docker unavailability not detected by lint

**Note**: This is not a bug - Clippy shouldn't check Docker. However, it means Docker unavailability isn't detected by the lint system, which is a gap in the overall validation system. The solution is to add Docker checks in other parts of the validation system (library code, tests, git hooks, Makefile), not in Clippy.

**Why Clippy Shouldn't Check Docker**:
1. **Static Analysis**: Clippy analyzes code statically, not runtime behavior
2. **Separation of Concerns**: Runtime dependency checks belong in runtime validation, not static analysis
3. **Performance**: Adding Docker checks to Clippy would slow down linting
4. **False Positives**: Clippy runs in CI/CD where Docker may not be available, causing false positives

**Solution**: Add Docker checks in appropriate places (library code, tests, git hooks, Makefile), not in Clippy.

---

## Summary of Failures

### Library Code Failures
1. `ContainerClient::new()` - No Docker check
2. `GenericContainer::new()` - No Docker check
3. `GenericContainer::with_env()` - No Docker check
4. `GenericContainer::with_ports()` - No Docker check
5. `WeaverValidator::start()` - No Docker check
6. `WeaverLiveCheck::start()` - No Docker check

### Test Infrastructure Failures
1. `docker_available()` - Insufficient check (uses `is_ok()`)
2. `require_docker()` - Relies on insufficient check
3. Tests don't check Docker in library code

### Git Hook Failures
1. Pre-commit hook - No Docker check
2. Pre-push hook - No Docker check

### Makefile Failures
1. `test-integration` task - No Docker check
2. `test-all` task - No Docker check

### Lint System
1. Clippy - Doesn't check Docker (correct behavior, but gap in validation)

## Root Cause Summary

**Primary Root Cause**: No proactive Docker health checks in library code. Only reactive checks in tests that may not work correctly when Docker is stopped.

**Contributing Factors**:
1. Insufficient test checks (`is_ok()` instead of Docker daemon verification)
2. No library-level validation
3. Generic error handling
4. No git hook validation
5. No Makefile validation
6. Silent failures
7. Assumption-based design

## Next Steps

See `DOCKER_HEALTH_CHECK_IMPLEMENTATION.md` for implementation plan to fix these failures.

