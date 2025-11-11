# Kaizen Improvement Plan: Extract Docker Error Detection Strings

## Step 1: Identify Improvement Opportunity ✅

**Opportunity**: Extract Docker error detection strings to named constants

**Current State**:
- Strings `"Cannot connect to the Docker daemon"`, `"docker daemon"`, and `"connection refused"` are duplicated 3 times
- Same detection logic repeated in `new()`, `with_env()`, and `with_ports()` methods
- Violates DRY (Don't Repeat Yourself) principle

**Improvement Criteria**:
- ✅ **Small**: Extract 3 strings to constants (5 minutes)
- ✅ **Focused**: One specific improvement (reduce duplication)
- ✅ **Safe**: No logic changes, just refactoring
- ✅ **Value**: Easier to maintain, single source of truth

## Step 2: Plan Change

**What**: Extract Docker error detection strings to named constants

**Why**: 
- Reduces duplication (DRY principle)
- Easier to maintain (change in one place)
- Self-documenting (constant names explain purpose)
- Consistent error detection across all methods

**How**:
1. Add constants after `check_docker_available()` function:
   ```rust
   /// Docker error message patterns that indicate Docker daemon is unavailable
   const DOCKER_CONNECTION_ERROR_PATTERNS: &[&str] = &[
       "Cannot connect to the Docker daemon",
       "docker daemon",
       "connection refused",
   ];
   ```

2. Create helper function to check if error indicates Docker unavailable:
   ```rust
   fn is_docker_unavailable_error(error_msg: &str) -> bool {
       DOCKER_CONNECTION_ERROR_PATTERNS.iter().any(|pattern| error_msg.contains(pattern))
   }
   ```

3. Replace duplicated detection logic in 3 places:
   - `GenericContainer::new()` (line 255-257)
   - `GenericContainer::with_env()` (line 308-310)
   - `GenericContainer::with_ports()` (line 349-351)

**Risk**: Low - simple refactoring, no logic changes

**Safety Checks**:
- ✅ No logic changes (same detection logic, just extracted)
- ✅ Tests exist for container creation
- ✅ Change is isolated (only affects error detection)
- ✅ Can be easily reverted if needed

## Step 3: Do (Implement)

**Implementation Steps**:
1. Add constants and helper function
2. Replace duplicated code with helper function call
3. Verify compilation

## Step 4: Check (Verify)

**Verification**:
- ✅ Code compiles: `cargo make check`
- ✅ Tests pass: `cargo make test`
- ✅ Functionality preserved: Same error detection behavior
- ✅ Improvement achieved: Code more maintainable, less duplication

## Step 5: Act (Standardize)

**Standardization**:
- Document pattern: Extract repeated string patterns to constants
- Apply pattern: Look for other duplicated strings in codebase
- Establish standard: Use constants for error message patterns

## Expected Outcome

**Before**:
```rust
if error_msg.contains("Cannot connect to the Docker daemon") 
    || error_msg.contains("docker daemon") 
    || error_msg.contains("connection refused") {
    // ... (repeated 3 times)
}
```

**After**:
```rust
if is_docker_unavailable_error(&error_msg) {
    // ... (single implementation)
}
```

**Benefits**:
- ✅ Single source of truth for error detection
- ✅ Easier to maintain (change in one place)
- ✅ More readable (function name explains purpose)
- ✅ Consistent error detection across all methods


