# Fix Plan: End User Failures with OTEL and Weaver

## Root Cause Analysis Summary

**Root Cause**: Missing end-user testing workflow - Code was tested for correctness but not for usability from an end-user perspective.

**Immediate Symptoms**:
1. Error messages reference non-existent script
2. Error messages lose detailed context
3. No guidance on where to get registry path
4. Port conflicts cause silent failures
5. Environment variable pollution
6. Potential infinite recursion
7. Missing actionable next steps

---

## Implementation Plan

### Phase 1: Fix Immediate Usability Issues (Symptoms)

#### Fix 1: Update BinaryNotFound Error Message
**File**: `src/observability/weaver/mod.rs`
**Issue**: Error message references non-existent script, loses error context
**Fix**:
- Change `BinaryNotFound` to accept `String` parameter
- Preserve detailed error message from `check_weaver_available()`
- Update error message to reference actual installation methods

**Changes**:
```rust
// Line 18: Update error variant
#[error("Weaver binary not found: {0}. Install with: cargo install weaver-cli or download from https://github.com/open-telemetry/weaver/releases")]
BinaryNotFound(String), // Add String parameter

// Line 95: Preserve error message
pub fn check_weaver_available() -> WeaverValidationResult<()> {
    WeaverLiveCheck::check_weaver_available()
        .map_err(|e| WeaverValidationError::BinaryNotFound(e)) // Preserve error message
}
```

#### Fix 2: Improve RegistryNotFound Error Message
**File**: `src/observability/weaver/mod.rs`
**Issue**: Users don't know where to get registry path
**Fix**: Add helpful context with link to semantic conventions registry

**Changes**:
```rust
// Line 104-108: Add helpful context
if !self.registry_path.exists() {
    return Err(WeaverValidationError::RegistryNotFound(format!(
        "Registry path does not exist: {}. Get the OpenTelemetry semantic conventions registry from: https://github.com/open-telemetry/semantic-conventions",
        self.registry_path.display()
    )));
}
```

#### Fix 3: Fix Environment Variable Pollution
**File**: `src/observability/weaver/mod.rs`
**Issue**: `send_test_span_to_weaver()` sets global environment variable
**Fix**: Save and restore original value

**Changes**:
```rust
// Line 196: Save original value
let original_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
let base_endpoint = endpoint.trim_end_matches("/v1/traces").trim_end_matches('/');
std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", base_endpoint);

// ... existing code ...

// Before shutdown: Restore original value
if let Some(original) = original_endpoint {
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", original);
} else {
    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
}
```

#### Fix 4: Prevent Infinite Recursion
**File**: `src/observability/weaver/types.rs`
**Issue**: `check_weaver_available()` can recursively call itself
**Fix**: Add recursion guard

**Changes**:
```rust
// Add static recursion guard
use std::sync::atomic::{AtomicBool, Ordering};
static DOWNLOAD_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

// In check_weaver_available():
if DOWNLOAD_IN_PROGRESS.swap(true, Ordering::Acquire) {
    return Err("Weaver download already in progress".to_string());
}
// ... download logic ...
DOWNLOAD_IN_PROGRESS.store(false, Ordering::Release);
```

#### Fix 5: Add Port Availability Check
**File**: `src/observability/weaver/mod.rs`
**Issue**: Ports might be in use, causing silent failures
**Fix**: Check ports before starting Weaver

**Changes**:
```rust
// Add port availability check in start()
use std::net::TcpListener;

// Check OTLP port
if TcpListener::bind(format!("{}:{}", LOCALHOST, self.otlp_grpc_port)).is_err() {
    return Err(WeaverValidationError::ValidationFailed(format!(
        "OTLP port {} is already in use. Choose a different port with with_config()",
        self.otlp_grpc_port
    )));
}

// Check admin port
if TcpListener::bind(format!("{}:{}", LOCALHOST, self.admin_port)).is_err() {
    return Err(WeaverValidationError::ValidationFailed(format!(
        "Admin port {} is already in use. Choose a different port with with_config()",
        self.admin_port
    )));
}
```

---

### Phase 2: Add End-User Testing Workflow (Root Cause Fix)

#### Fix 6: Create End-User Scenario Tests
**File**: `tests/end_user_scenarios/weaver_fresh_install.rs` (new)
**Purpose**: Test from end-user perspective (fresh install, no prior knowledge)

**Tests to Add**:
1. `test_weaver_fresh_install_scenario` - Simulate fresh install
2. `test_weaver_error_messages_actionable` - Verify error messages are actionable
3. `test_weaver_setup_instructions_work` - Verify setup instructions work
4. `test_weaver_registry_path_guidance` - Verify registry path guidance is helpful

**Example**:
```rust
#[cfg(feature = "weaver")]
#[test]
fn test_weaver_fresh_install_scenario() {
    // Simulate fresh install (no prior setup)
    // 1. Try to use WeaverValidator without setup
    // 2. Verify error message tells user what to do next
    // 3. Verify error message references existing resources
    
    let registry_path = std::path::PathBuf::from("/nonexistent/registry");
    let mut validator = WeaverValidator::new(registry_path);
    
    let result = validator.start();
    assert!(result.is_err());
    
    // Verify error message is actionable
    match result {
        Err(WeaverValidationError::RegistryNotFound(msg)) => {
            assert!(msg.contains("semantic-conventions"), 
                "Error message should tell user where to get registry");
        }
        _ => panic!("Expected RegistryNotFound error"),
    }
}

#[cfg(feature = "weaver")]
#[test]
fn test_weaver_error_messages_actionable() {
    // Verify error messages tell user what to do next
    // Verify error messages reference existing resources
    // Verify error messages preserve context
    
    // Test BinaryNotFound error
    let result = WeaverValidator::check_weaver_available();
    if let Err(WeaverValidationError::BinaryNotFound(msg)) = result {
        assert!(msg.contains("cargo install") || msg.contains("github.com"), 
            "Error message should tell user how to install");
    }
}
```

#### Fix 7: Add Documentation Verification Tests
**File**: `tests/end_user_scenarios/documentation_verification.rs` (new)
**Purpose**: Verify all referenced resources exist

**Tests to Add**:
1. `test_weaver_install_instructions_exist` - Verify installation instructions work
2. `test_weaver_error_messages_reference_existing_resources` - Verify error messages don't reference non-existent resources
3. `test_weaver_registry_path_guidance_exists` - Verify registry path guidance is correct

**Example**:
```rust
#[cfg(feature = "weaver")]
#[test]
fn test_weaver_error_messages_reference_existing_resources() {
    // Verify error messages don't reference non-existent scripts
    // Error message should NOT reference ./scripts/install-weaver.sh
    // Error message SHOULD reference cargo install or GitHub releases
    
    let result = WeaverValidator::check_weaver_available();
    if let Err(WeaverValidationError::BinaryNotFound(msg)) = result {
        assert!(!msg.contains("./scripts/install-weaver.sh"), 
            "Error message should not reference non-existent script");
        assert!(msg.contains("cargo install") || msg.contains("github.com"), 
            "Error message should reference existing installation methods");
    }
}
```

#### Fix 8: Add CI Validation
**File**: `Makefile.toml`
**Purpose**: Prevent recurrence by running end-user tests in CI

**Changes**:
```toml
[tasks.test-end-user-scenarios]
description = "Run end-user scenario tests (verify usability from end-user perspective)"
command = "timeout"
args = [
    "60s",
    "cargo",
    "nextest",
    "run",
    "--test",
    "end_user_scenarios",
    "--features",
    "weaver",
]
```

---

## Files to Modify

1. `src/observability/weaver/mod.rs` - Fix error messages, add port checks, fix env var pollution
2. `src/observability/weaver/types.rs` - Add recursion guard, improve error messages
3. `tests/end_user_scenarios/weaver_fresh_install.rs` - Add end-user scenario tests (new)
4. `tests/end_user_scenarios/documentation_verification.rs` - Add documentation verification tests (new)
5. `Makefile.toml` - Add CI validation task

---

## Validation

### Phase 1 Validation
- Run tests: `cargo make test --features weaver`
- Verify error messages are actionable
- Test port conflict detection
- Test environment variable restoration
- Verify no infinite recursion

### Phase 2 Validation
- Run end-user scenario tests: `cargo make test-end-user-scenarios`
- Verify fresh install scenario works
- Verify error messages are actionable
- Verify documentation is correct
- Verify setup instructions work

---

## Expected Outcomes

### Immediate (Phase 1)
- ✅ Clear, actionable error messages for all failure modes
- ✅ No environment variable pollution
- ✅ Port conflicts detected early
- ✅ No infinite recursion
- ✅ Better user experience with helpful guidance

### Long-term (Phase 2)
- ✅ End-user testing workflow catches usability issues before release
- ✅ CI validation prevents recurrence
- ✅ Documentation stays in sync with code
- ✅ Features are usable without internal knowledge
- ✅ Reduced end-user frustration and support burden

---

## Prevention Strategy

**Pattern**: End-User Testing Workflow

**When to Use**: Before releasing any feature to end users

**How to Use**:
1. Create `tests/end_user_scenarios/` directory
2. Add fresh install scenario tests
3. Add error message validation tests
4. Add documentation verification tests
5. Add setup instruction tests
6. Run tests before release
7. Add CI validation

**Benefits**:
- Catches usability issues before release
- Ensures error messages are actionable
- Verifies documentation is correct
- Prevents end-user frustration
- Reduces support burden

---

## Key Learning

**Always eat your own dog food** - Test features from an end-user perspective, not just internal correctness. End-user testing workflow catches usability issues before they reach production.

**Root Cause**: Missing end-user testing workflow allowed usability issues to reach production.

**Fix**: Add end-user testing workflow + fix immediate usability issues.

**Prevention**: CI validation ensures end-user tests run before release.


