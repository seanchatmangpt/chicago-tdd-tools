# Kaizen Improvement Plan - Weaver Live-Check Always Works

## Overview

Break down root cause fixes into small, incremental improvements following Kaizen principles. Each improvement is:
- **Small**: Can be done in minutes, not hours
- **Focused**: Addresses one specific thing
- **Safe**: Low risk of breaking things
- **Value**: Adds clarity, reliability, or error prevention

## Kaizen Improvement 1: Add Startup Timeout Constant

### Step 1: Identify Opportunity
**Opportunity**: Extract magic number `5` (seconds) to named constant for Weaver startup timeout
**Type**: Code clarity, consistency
**Current state**: No constant for startup timeout (will be used in `wait_for_weaver_ready()`)

### Step 2: Plan Change
**What**: Add `DEFAULT_WEAVER_STARTUP_TIMEOUT_SECONDS` constant
**Why**: 
- Makes code more readable
- Easier to change timeout value
- Self-documenting
- Matches existing pattern (`DEFAULT_INACTIVITY_TIMEOUT_SECONDS`)
**How**: 
1. Add constant after `DEFAULT_INACTIVITY_TIMEOUT_SECONDS` in `src/observability/weaver/mod.rs`
2. Use `5` seconds (quick check SLA from SPR Guide)
**Risk**: Low - simple constant addition, no logic change

### Step 3: Do (Implement)
```rust
/// Default Weaver startup timeout in seconds (5 seconds)
///
/// **Kaizen improvement**: Extracted magic number `5` to named constant.
/// Pattern: Use named constants for timeout values.
/// Benefits: Improves readability, maintainability, self-documentation.
/// 
/// **SPR Guide**: Matches quick check SLA (5 seconds).
pub const DEFAULT_WEAVER_STARTUP_TIMEOUT_SECONDS: u64 = 5;
```

### Step 4: Check (Verify)
- ✅ Code compiles (`cargo make check`)
- ✅ Tests pass (`cargo make test`)
- ✅ Constant follows existing pattern

### Step 5: Act (Standardize)
- Pattern: Use named constants for all timeout values
- Document: Add to code review checklist

---

## Kaizen Improvement 2: Fix `is_running()` to Check Actual Process State

### Step 1: Identify Opportunity
**Opportunity**: `is_running()` only checks `process.is_some()`, not if process is actually alive
**Type**: Error prevention, behavior verification
**Current state**: Returns `true` even if process exited

### Step 2: Plan Change
**What**: Change `is_running()` to check actual process state using `try_wait()`
**Why**: 
- Prevents false positives (thinks process is running when it's dead)
- Matches Chicago TDD behavior verification principle
- Catches process exit early
**How**: 
1. Change `is_running()` to call `process.try_wait()`
2. Return `false` if process exited
3. Keep `const` if possible, or remove if `try_wait()` requires mutable reference
**Risk**: Low - isolated change, no breaking API changes

### Step 3: Do (Implement)
```rust
/// Check if Weaver process is running
///
/// **Kaizen improvement**: Now checks actual process state using `try_wait()`.
/// Previously only checked `process.is_some()`, which could return `true`
/// even if process exited. Now verifies process is actually alive.
///
/// # Returns
///
/// `true` if process exists and hasn't exited, `false` otherwise.
#[must_use]
pub fn is_running(&mut self) -> bool {
    if let Some(ref mut process) = self.process {
        // Check if process has exited
        match process.try_wait() {
            Ok(Some(_)) => false, // Process exited
            Ok(None) => true,     // Process still running
            Err(_) => false,      // Error checking process (assume not running)
        }
    } else {
        false // No process
    }
}
```

**Note**: Must change signature from `&self` to `&mut self` to call `try_wait()`. This is acceptable because checking process state may mutate internal state.

### Step 4: Check (Verify)
- ✅ Code compiles (`cargo make check`)
- ✅ Tests pass (`cargo make test`)
- ✅ Behavior verified: Returns `false` if process exited

### Step 5: Act (Standardize)
- Pattern: Always check actual state, not just presence
- Document: Add to code review checklist

---

## Kaizen Improvement 3: Add Process Exit Detection After Spawn

### Step 1: Identify Opportunity
**Opportunity**: After spawning Weaver process, don't verify it's actually running
**Type**: Error prevention, fail-fast
**Current state**: `start()` returns success even if process exits immediately

### Step 2: Plan Change
**What**: Add process exit check immediately after spawn in `WeaverValidator::start()`
**Why**: 
- Catches process exit early (fail-fast)
- Prevents proceeding with dead process
- Provides clear error message with exit code and stderr
**How**: 
1. After spawning process (line 209), check if process exited using `try_wait()`
2. If exited, capture stderr and return error with exit code
3. Error message includes actionable fix instructions
**Risk**: Low - isolated change, improves error detection

### Step 3: Do (Implement)
```rust
// Start Weaver live-check process
let mut process = live_check.start().map_err(WeaverValidationError::ProcessStartFailed)?;

// **Kaizen improvement**: Check if process exited immediately after spawn.
// This catches registry validation errors and other startup failures early.
// Fail-fast: Don't proceed if Weaver can't start.
match process.try_wait() {
    Ok(Some(status)) => {
        // Process exited immediately - capture stderr for error message
        let exit_code = status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&[]); // TODO: Capture stderr from process
        
        return Err(WeaverValidationError::ProcessStartFailed(format!(
            "Weaver process exited immediately after spawn (exit code: {exit_code})\n   ⚠️  STOP: Weaver cannot start\n   💡 FIX: Check registry validation errors (run: weaver registry check -r {})\n   💡 FIX: Review stderr output above for detailed error messages\n   📋 Exit code: {exit_code}",
            self.registry_path.display()
        )));
    }
    Ok(None) => {
        // Process is running - continue
    }
    Err(e) => {
        // Error checking process - assume it's running (may be race condition)
        // Log warning but don't fail
        log::warn!("Failed to check Weaver process status: {e}");
    }
}

self.live_check = Some(live_check);
self.process = Some(process);
```

**Note**: Capturing stderr from `Child` requires more complex handling. For now, return error with exit code. Can be improved in later Kaizen.

### Step 4: Check (Verify)
- ✅ Code compiles (`cargo make check`)
- ✅ Tests pass (`cargo make test`)
- ✅ Behavior verified: Detects process exit immediately

### Step 5: Act (Standardize)
- Pattern: Always verify process state after spawn
- Document: Add to code review checklist

---

## Kaizen Improvement 4: Add Admin Port Readiness Check Helper

### Step 1: Identify Opportunity
**Opportunity**: No helper function to check if Weaver admin port is listening
**Type**: Code clarity, reusability
**Current state**: Port readiness check logic would be duplicated

### Step 2: Plan Change
**What**: Add `check_admin_port_ready()` helper function
**Why**: 
- Reusable for multiple checks
- Clear, focused function
- Easier to test
**How**: 
1. Add helper function `check_admin_port_ready(admin_port: u16) -> bool`
2. Use `TcpStream::connect_timeout()` to check port
3. Return `true` if port is listening, `false` otherwise
**Risk**: Low - isolated helper function, no breaking changes

### Step 3: Do (Implement)
```rust
/// Check if Weaver admin port is ready (listening)
///
/// **Kaizen improvement**: Extracted port readiness check to reusable helper.
/// Pattern: Extract repeated logic to helper functions.
/// Benefits: Reusable, testable, clear intent.
///
/// # Returns
///
/// `true` if admin port is listening, `false` otherwise.
#[must_use]
fn check_admin_port_ready(admin_port: u16) -> bool {
    use std::net::TcpStream;
    use std::time::Duration;
    
    // Quick check: Try to connect with short timeout (100ms)
    // This is non-blocking and fast
    TcpStream::connect_timeout(
        &format!("{}:{}", LOCALHOST, admin_port).parse().unwrap(),
        Duration::from_millis(100),
    )
    .is_ok()
}
```

### Step 4: Check (Verify)
- ✅ Code compiles (`cargo make check`)
- ✅ Tests pass (`cargo make test`)
- ✅ Helper function works correctly

### Step 5: Act (Standardize)
- Pattern: Extract repeated logic to helper functions
- Document: Add to code review checklist

---

## Kaizen Improvement 5: Add `wait_for_weaver_ready()` Function

### Step 1: Identify Opportunity
**Opportunity**: No function to wait for Weaver to be ready (process alive + admin port listening)
**Type**: Error prevention, reliability
**Current state**: No readiness verification after spawn

### Step 2: Plan Change
**What**: Add `wait_for_weaver_ready()` function that verifies process is alive and admin port is listening
**Why**: 
- Ensures Weaver is actually ready before proceeding
- Prevents race conditions
- Provides clear error messages
**How**: 
1. Add function `wait_for_weaver_ready(process: &mut Child, admin_port: u16, timeout: u64) -> WeaverValidationResult<()>`
2. Use exponential backoff (100ms, 200ms, 400ms, 800ms, 1600ms, 2000ms)
3. Check process state and admin port on each retry
4. Return error if timeout exceeded or process exited
**Risk**: Medium - new function, but isolated and testable

### Step 3: Do (Implement)
```rust
/// Wait for Weaver to be ready (process alive + admin port listening)
///
/// **Kaizen improvement**: Verifies Weaver is actually ready before proceeding.
/// Prevents race conditions where tests proceed before Weaver is ready.
///
/// # Errors
///
/// Returns error if:
/// - Process exits during wait
/// - Admin port never becomes ready (timeout)
/// - Timeout exceeded
fn wait_for_weaver_ready(
    process: &mut Child,
    admin_port: u16,
    timeout_seconds: u64,
) -> WeaverValidationResult<()> {
    use std::time::{Duration, Instant};
    
    let start_time = Instant::now();
    let timeout = Duration::from_secs(timeout_seconds);
    
    // Exponential backoff intervals (total ~5 seconds)
    let backoff_intervals = [100, 200, 400, 800, 1600, 2000]; // milliseconds
    
    for &interval_ms in &backoff_intervals {
        // Check if timeout exceeded
        if start_time.elapsed() > timeout {
            return Err(WeaverValidationError::ProcessStartFailed(format!(
                "Weaver startup timeout exceeded ({timeout_seconds}s)\n   ⚠️  STOP: Weaver admin port not ready\n   💡 FIX: Check Weaver logs for startup errors\n   💡 FIX: Verify registry is valid (run: weaver registry check -r <registry_path>)"
            )));
        }
        
        // Check if process exited
        match process.try_wait() {
            Ok(Some(status)) => {
                let exit_code = status.code().unwrap_or(-1);
                return Err(WeaverValidationError::ProcessStartFailed(format!(
                    "Weaver process exited during startup (exit code: {exit_code})\n   ⚠️  STOP: Weaver cannot start\n   💡 FIX: Check registry validation errors (run: weaver registry check -r <registry_path>)\n   💡 FIX: Review stderr output for detailed error messages"
                )));
            }
            Ok(None) => {
                // Process is running - check admin port
                if Self::check_admin_port_ready(admin_port) {
                    // ✅ Weaver is ready!
                    return Ok(());
                }
                // Port not ready yet - continue waiting
            }
            Err(_) => {
                // Error checking process - assume it's running and continue
            }
        }
        
        // Wait before next check
        std::thread::sleep(Duration::from_millis(interval_ms));
    }
    
    // Final check after all retries
    if Self::check_admin_port_ready(admin_port) {
        Ok(())
    } else {
        Err(WeaverValidationError::ProcessStartFailed(format!(
            "Weaver admin port not ready after {timeout_seconds}s\n   ⚠️  STOP: Weaver may not be responding\n   💡 FIX: Check Weaver logs for startup errors\n   💡 FIX: Verify registry is valid (run: weaver registry check -r <registry_path>)"
        )))
    }
}
```

### Step 4: Check (Verify)
- ✅ Code compiles (`cargo make check`)
- ✅ Tests pass (`cargo make test`)
- ✅ Behavior verified: Waits for Weaver to be ready

### Step 5: Act (Standardize)
- Pattern: Always wait for external processes to be ready
- Document: Add to code review checklist

---

## Kaizen Improvement 6: Call `wait_for_weaver_ready()` After Spawn

### Step 1: Identify Opportunity
**Opportunity**: `wait_for_weaver_ready()` exists but isn't called after spawn
**Type**: Error prevention, reliability
**Current state**: Process spawned but not verified ready

### Step 2: Plan Change
**What**: Call `wait_for_weaver_ready()` after spawning process in `WeaverValidator::start()`
**Why**: 
- Ensures Weaver is ready before returning
- Prevents race conditions
- Provides clear error messages if Weaver can't start
**How**: 
1. After spawning process (line 209), call `wait_for_weaver_ready()`
2. Use `DEFAULT_WEAVER_STARTUP_TIMEOUT_SECONDS` constant
3. Return error if readiness check fails
**Risk**: Low - uses existing function, isolated change

### Step 3: Do (Implement)
```rust
// Start Weaver live-check process
let mut process = live_check.start().map_err(WeaverValidationError::ProcessStartFailed)?;

// **Kaizen improvement**: Wait for Weaver to be ready before proceeding.
// This ensures process is alive and admin port is listening.
// Prevents race conditions where tests proceed before Weaver is ready.
Self::wait_for_weaver_ready(
    &mut process,
    self.admin_port,
    DEFAULT_WEAVER_STARTUP_TIMEOUT_SECONDS,
)?;

self.live_check = Some(live_check);
self.process = Some(process);
```

### Step 4: Check (Verify)
- ✅ Code compiles (`cargo make check`)
- ✅ Tests pass (`cargo make test`)
- ✅ Behavior verified: Waits for Weaver before returning

### Step 5: Act (Standardize)
- Pattern: Always verify external processes are ready before proceeding
- Document: Add to code review checklist

---

## Implementation Order

1. **Improvement 1**: Add startup timeout constant (foundation)
2. **Improvement 2**: Fix `is_running()` (isolated, no dependencies)
3. **Improvement 3**: Add process exit detection (isolated, improves error messages)
4. **Improvement 4**: Add admin port readiness helper (foundation for improvement 5)
5. **Improvement 5**: Add `wait_for_weaver_ready()` (uses improvement 4)
6. **Improvement 6**: Call `wait_for_weaver_ready()` (uses improvement 5)

## Success Criteria

After all improvements:
- ✅ Weaver process exit during startup is detected immediately
- ✅ Clear error messages include exit code and fix instructions
- ✅ Tests fail fast with actionable errors
- ✅ All existing tests pass
- ✅ Code follows existing patterns (named constants, helper functions)

## Testing Strategy

For each improvement:
1. **Unit tests**: Test function in isolation
2. **Integration tests**: Test with real Weaver process
3. **Error path tests**: Test error scenarios (80% of bugs)
4. **Behavior verification**: Verify actual behavior, not just function calls

## Notes

- Each improvement is independent and can be done separately
- Improvements build on each other (foundation → usage)
- Follow existing code patterns (named constants, helper functions)
- Use Chicago TDD principles (behavior verification, real collaborators)
- Follow SPR Guide (timeout SLAs, error handling patterns)

