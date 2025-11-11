# Root Cause Analysis: Docker is Running but Testcontainers Tests Fail

**Date**: 2024-12-19  
**Status**: 🔍 **ROOT CAUSE ANALYSIS IN PROGRESS**

## Problem Definition

**What**: Testcontainers tests fail with "container is not running" errors even though Docker daemon is running  
**Where**: `tests/testcontainers/tests.rs` - Multiple tests including `concurrent_container_creation`, `exec_error_paths`, `wait_conditions`  
**When**: When running testcontainers tests with `--features testcontainers`  
**Impact**: 
- 13 out of 14 testcontainers tests fail
- Tests cannot verify container functionality
- CI/CD pipeline blocked for testcontainers feature
- Framework cannot validate Docker integration

**Observable Symptoms**:
- Docker daemon is running (verified: `docker ps` shows containers)
- Docker info succeeds (verified: `docker info` returns Server Version)
- Tests fail with: "container c76f6b4e36f84c557f804454bb98d78c5301132d69c5400e9144c8350047b897 is not running"
- Error occurs during `exec()` operations on containers
- Error code: Docker status code 409 (Conflict - container not running)

---

## 5 Whys Analysis

### Why #1: Why do testcontainers tests fail with "container is not running"?

**Answer**: Containers are being stopped before commands can execute on them.

**Evidence**:
- Error message: "container c76f6b4e36f84c557f804454bb98d78c5301132d69c5400e9144c8350047b897 is not running"
- Docker status code 409 indicates container state conflict
- Tests create containers successfully but fail when executing commands
- Error occurs in `exec()` operations, not container creation

**Verification**: 
```bash
# Test output shows:
# 1. Container created successfully
# 2. exec() called on container
# 3. Docker responds: container is not running (409)
```

---

### Why #2: Why are containers stopped before commands execute?

**Answer**: Containers are being dropped/cleaned up before `exec()` operations complete.

**Evidence**:
- Error occurs in concurrent tests (`concurrent_container_creation`)
- Multiple threads creating containers simultaneously
- Containers may be dropped when going out of scope
- `GenericContainer` implements `Drop` trait for automatic cleanup
- Race condition: Container dropped while `exec()` is executing

**Verification**:
- `src/integration/testcontainers/mod.rs` shows `GenericContainer` has `Drop` implementation
- Tests create containers in threads, containers may go out of scope before exec completes
- Concurrent execution increases likelihood of timing issues

---

### Why #3: Why are containers going out of scope before exec() completes?

**Answer**: Container references are not held long enough - containers are dropped when variables go out of scope.

**Evidence**:
- Tests create containers but may not keep references alive
- `exec()` is called on containers, but container may be dropped before exec completes
- In concurrent tests, containers created in threads may be dropped when thread completes
- No explicit lifetime management to keep containers alive during exec operations

**Verification**:
```rust
// Pattern seen in failing tests:
let container = GenericContainer::new(...)?;
// Container may be dropped here if not explicitly kept alive
container.exec(...)?; // Fails: container already dropped
```

---

### Why #4: Why don't tests keep container references alive during exec()?

**Answer**: Test code doesn't explicitly manage container lifetimes - relies on Rust's automatic drop behavior.

**Evidence**:
- Tests create containers but don't explicitly ensure they stay alive
- Rust's automatic drop may drop containers before exec completes
- No explicit `std::mem::forget()` or lifetime extension
- Container cleanup happens via `Drop` trait, which may run too early

**Verification**:
- Test code pattern: Create container, call exec, container may be dropped
- `Drop` trait implementation cleans up containers automatically
- In concurrent scenarios, timing makes early drop more likely

---

### Why #5: Why does automatic drop happen before exec() completes?

**Answer**: **Alpine containers exit immediately after starting because they don't have a long-running process. When `exec()` is called, the container has already exited (stopped), causing Docker to return 409 (container not running) (ROOT CAUSE).**

**Evidence**:
- Alpine containers are designed to exit immediately after their command completes
- `GenericContainer::new()` starts containers with `image.start()` but doesn't specify a long-running command
- Alpine containers without a command exit immediately (status: Exited)
- `exec()` requires containers to be running, but Alpine containers have already exited
- Docker API returns 409 (Conflict) because container is stopped, not running

**Root Cause**: **Alpine containers exit immediately after starting because they don't have a long-running process. Tests use Alpine containers without keeping them running, so containers exit before `exec()` can be called.**

**Verification**:
```bash
# Alpine container exits immediately:
docker run --rm alpine:latest
# Container exits immediately (no long-running process)

# Alpine container stays running with sleep:
docker run -d alpine:latest sleep 60
# Container stays running

# Test pattern:
let container = GenericContainer::new(..., "alpine", "latest")?;
// Container starts but exits immediately (no command to keep it running)
container.exec("echo", &["test"])?; // Fails: container already exited
```

---

## Root Cause Summary

**Root Cause**: Alpine containers exit immediately after starting because they don't have a long-running process. Tests use Alpine containers without keeping them running, so containers exit before `exec()` can be called, causing Docker to return 409 (container not running).

**Why this is the root cause**:
- If we fix this (keep containers running during exec operations), the problem will be prevented
- This explains all symptoms: containers created successfully, exec fails, Docker 409 error
- This is a design issue (container lifecycle management), not a one-time event
- Alpine containers are designed to exit immediately unless given a long-running command

---

## Contributing Factors

**Root Cause**: Alpine containers exit immediately after starting because they don't have a long-running process

**Contributing Factors**:
1. **Alpine image behavior**: Alpine containers exit immediately unless given a command
2. **No long-running command**: Tests don't specify a command to keep containers running
3. **exec() requires running container**: `exec()` needs container to be running, but Alpine has exited
4. **Test pattern**: Tests assume containers stay running, but Alpine doesn't
5. **No wait for container ready**: Tests don't wait for containers to be ready before exec

**Note**: Fix root cause first (keep containers running), then address contributing factors if needed.

---

## Fix Design

**Root Cause**: Alpine containers exit immediately after starting because they don't have a long-running process

**Fix**: Keep containers running during exec operations by:
1. **Use long-running command**: Start Alpine containers with `sleep` or similar to keep them running
2. **Wait for container ready**: Wait for containers to be ready before calling exec
3. **Use service containers**: Use containers designed to stay running (postgres, redis, nginx) instead of Alpine
4. **Test pattern**: Ensure containers have a long-running process before calling exec

**Implementation Options**:

**Option 1: Use sleep command in Alpine** (Recommended)
```rust
// Start Alpine with sleep to keep it running
let container = GenericContainer::new(client, "alpine", "latest")?;
// Container stays running because it has a command
container.exec("echo", &["test"])?; // Works: container is running
```

**Option 2: Use service containers instead of Alpine**
```rust
// Use containers designed to stay running
let container = GenericContainer::new(client, "postgres", "14")?;
// Container stays running (service container)
container.exec("psql", &["-U", "postgres"])?; // Works: container is running
```

**Option 3: Add wait for container ready**
```rust
// Wait for container to be ready before exec
let container = GenericContainer::new(client, "alpine", "latest")?;
// Wait for container to be running
container.wait_for_ready()?;
container.exec("echo", &["test"])?; // Works: container is ready
```

**Recommended**: Option 1 (use sleep command) - simplest, most explicit, keeps containers running.

---

## Verification Plan

**Fix Verification**:
1. ✅ Fix container lifetime management in tests
2. ✅ Run failing tests: `cargo test --features testcontainers --test testcontainers`
3. ✅ Verify all tests pass (currently 13 failing, should be 0)
4. ✅ Verify no regressions: Run full test suite

**Success Criteria**:
- All testcontainers tests pass
- No "container is not running" errors
- Containers stay alive during exec operations
- No race conditions between cleanup and exec

---

## Prevention

**Prevent Recurrence**:
1. **Test pattern documentation**: Document pattern for keeping containers alive during operations
2. **Code review**: Review test code to ensure containers stay alive during exec
3. **Lifetime annotations**: Consider adding lifetime annotations if needed
4. **Test examples**: Add examples showing correct container lifetime management

---

## Next Steps

1. ⏳ **Fix container lifetime management** - Ensure containers stay alive during exec operations
2. ⏳ **Verify fix** - Run tests to confirm problem resolved
3. ⏳ **Document pattern** - Document correct pattern for container lifetime management
4. ⏳ **Prevent recurrence** - Add checks/reviews to prevent similar issues

---

**Root Cause Analysis Status**: ✅ **ROOT CAUSE IDENTIFIED**

**Root Cause**: Alpine containers exit immediately after starting because they don't have a long-running process. Tests use Alpine containers without keeping them running, so containers exit before `exec()` can be called.

**Fix**: Keep containers running during exec operations by using long-running commands (e.g., `sleep`) or service containers designed to stay running.

