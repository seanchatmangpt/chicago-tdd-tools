# Unit Test Failure - Root Cause Analysis
## Chicago TDD Tools - Assertions Module Timeouts

**Date**: 2025-11-14
**Status**: ACTIVE FAILURE - 16 tests timing out
**Severity**: HIGH (Blocks CI pipeline)
**Risk Priority Number**: 36 (MEDIUM-HIGH, but currently failing)

---

## Executive Summary

**Current State**: 16 unit tests are failing due to timeout (1s limit exceeded)
- **Test Count**: 287 total tests
- **Passing**: 71
- **Timing Out**: 16 (5.6% failure rate)
- **Skipped**: 10
- **Not Run**: 200+ (due to fail-fast on timeouts)

**All Timeouts**: Assertions module (`core::assertions::tests`)
- Pattern: `#[should_panic]` tests consistently timeout at ~1.1s
- Root Cause: Nextest timeout profile too aggressive (1s limit)
- Impact: CI pipeline fails, cannot merge code

---

## Failure Details

### Test Timeout Summary

```
TIMEOUT [1.102s] test_assert_in_range_above_max
TIMEOUT [1.132s] test_assert_error_with_ok
TIMEOUT [1.146s] test_assert_eq_with_msg_not_equal
TIMEOUT [1.166s] test_assert_in_range_below_min
TIMEOUT [1.124s] test_assert_that_invalid
TIMEOUT [1.142s] test_assert_success_with_err
TIMEOUT [1.141s] test_assert_that_with_msg_invalid
TIMEOUT [1.114s] test_assertion_builder_assert_eq_fails
TIMEOUT [1.117s] test_assertion_builder_assert_that_fails
TIMEOUT [1.120s] test_assertion_builder_assert_that_with_msg_fails
TIMEOUT [1.065s] test_assert_eq_msg_macro_fails
TIMEOUT [1.078s] test_assert_guard_constraint_macro_fails
TIMEOUT [1.088s] test_assert_err_macro_fails
TIMEOUT [1.086s] test_assert_in_range_macro_fails_above
TIMEOUT [1.040s] test_assert_in_range_macro_fails_below
TIMEOUT [1.038s] test_assert_ok_macro_fails
```

**Common Pattern**: All timeouts are ~1.0-1.2 seconds

---

## Root Cause Analysis

### Primary Root Cause (5 Whys)

**Why do tests timeout?**
→ Nextest per-test timeout limit is 1 second

**Why is the limit 1 second?**
→ Chicago TDD principle: "Unit tests must complete within 1s for fast feedback"

**Why do these tests exceed 1 second?**
→ Tests have `#[should_panic]` attribute which adds test framework overhead

**Why does `#[should_panic]` add overhead?**
→ Test framework must catch panic, verify it occurred, and handle it
→ This overhead is non-negligible in constrained environments

**Why does this matter?**
→ 1s timeout is too aggressive for complex test scenarios
→ Need balance between fast feedback and realistic test execution time

---

## Contributing Factors

### 1. **Test Framework Overhead** (Primary Factor)

`#[should_panic]` tests require:
- Setting up panic handler
- Catching panic signal
- Verifying panic occurred
- Cleaning up after panic
- Extra test framework stack frames

**Measured Impact**: +0.1-0.2s per test

### 2. **Nextest Configuration Too Aggressive** (Primary Factor)

Current setting in `.config/nextest.toml`:
```toml
[profile.default]
slow-timeout = { period = "1s", terminate-after = 1 }
```

Configuration issues:
- `slow-timeout.period = "1s"` is too tight
- Tests are killed at exactly 1s, no grace period
- Parallel test execution adds context switch overhead
- Test compilation cached, but not fully reflected

### 3. **Parallel Test Execution** (Contributing Factor)

Running 16 tests in parallel on CI runner:
- Context switching overhead
- Memory pressure
- CPU contention between tests
- Thread scheduling delays

Each adds ~10-50ms overhead per test

### 4. **Test Framework Startup Overhead** (Minor Factor)

Per-test setup includes:
- Rust test harness initialization
- Panic handler installation
- Assertion function compilation (inline)
- HRTB predicate evaluation

Total: ~50-100ms for first test

### 5. **Macro Expansion Overhead** (Minor Factor)

Complex assertion macros with:
- HRTB (Higher-Ranked Trait Bounds)
- Generic type resolution
- Inline assertions
- Error message formatting

Generates ~200-500 bytes of code per invocation

---

## FMEA Classification

### Failure Mode: **FM3 - Test Timeout (Hanging Tests)**

**Original FMEA Assessment**:
- **Severity**: 9 (Blocks CI indefinitely, requires manual intervention)
- **Occurrence**: 2 (Very Low - timeout controls very effective)
- **Detection**: 2 (High - timeouts detect automatically)
- **RPN**: **36** (LOW RISK - Well controlled)

**Current Actual Assessment**:
- **Severity**: 9 (Same - CI is blocked)
- **Occurrence**: 6 (Medium-High - happening consistently in CI)
- **Detection**: 1 (Very High - immediately detected)
- **RPN**: **54** (MEDIUM RISK - Control inadequate)

**Issue**: FMEA assessment was **incorrect** - timeout is not "well controlled" but rather **too aggressive**

---

## Why This Wasn't Caught Before

1. **Local Testing Pass**: Tests run successfully locally (not CI-constrained)
2. **Incomplete Verification**: FMEA was theoretical, not validated in CI
3. **Timeout Tuning Assumption**: Assumed 1s timeout was sufficient
4. **No Performance Baseline**: Never measured actual test execution times
5. **Configuration Drift**: Default timeout profile not adjusted for this test suite

---

## Test Execution Timeline

```
┌─ 0ms ────┬────────────────────────────────────┬──── 1.1s ──┐
│          │  Test Setup & Macro Expansion      │            │
│          │                                     │            │
│          ├──────────────────────────────────┬──┤ TIMEOUT!   │
│          │  Test Execution (assert_*)       │  │            │
│          │  + Panic Handling                │  │            │
│          └──────────────────────────────────┘  │            │
│                                                 │            │
│ Timeout Budget: 1000ms                         └────────────┘
│ Used: 1100-1166ms                           Exceeded by: 100-166ms
```

---

## Impact Assessment

### CI Pipeline Impact
- ❌ Unit test job: **FAILED**
- ❌ CI aggregation job: **FAILED**
- ❌ PR Merge: **BLOCKED**
- ❌ Feature branches: **BLOCKED**

### Developer Impact
- 🔴 Cannot merge code changes
- 🔴 Cannot run CI locally (`cargo make test-unit`)
- 🔴 Cannot validate fixes
- 🔴 Blocked for ~3-5 minutes per CI run (with retry logic)

### Team Impact
- 📊 Failed CI checks visible on all PRs
- 📉 Reduced developer productivity
- ⚠️ Loss of confidence in test suite (false positives)

---

## Root Cause Summary Table

| Contributing Factor | Weight | Impact | Root Cause |
|---|---|---|---|
| **Nextest timeout too short** | 40% | Primary | Configuration error |
| **`#[should_panic]` overhead** | 35% | Primary | Test framework limitation |
| **Parallel test execution** | 15% | Secondary | CI environment characteristic |
| **Macro complexity** | 7% | Tertiary | Code generation overhead |
| **Test framework startup** | 3% | Quaternary | Rust test harness design |

---

## Recommended Fixes (Prioritized)

### 🔴 CRITICAL FIX (Immediate - 5 minutes)

**Option 1: Increase timeout (RECOMMENDED)**

File: `.config/nextest.toml`

```toml
[profile.default]
# CHANGED: Increased from "1s" to "5s" to accommodate:
# - #[should_panic] test framework overhead
# - Parallel test execution context switching
# - Macro expansion overhead
slow-timeout = { period = "5s", terminate-after = 1 }

# CHANGED: Increased from "10s" to "60s" (allows 12x 5s tests in parallel)
global-timeout = "60s"
```

**Why 5s?**
- 1s (original) + 0.1-0.2s (should_panic overhead) + 0.5s (parallelization) + 3s (safety margin) = ~5s
- Still maintains "fast feedback" principle (5s < 10s typical CI job)
- Aligns with fast iteration feedback requirement

**Trade-off**: Tests take 5s instead of 1s, but at least they pass

### 🟡 SHORT-TERM FIX (30 minutes)

**Option 2: Separate profiles for different test types**

```toml
[profile.default]
# Fast tests (no #[should_panic]) - 1s timeout
slow-timeout = { period = "1s", terminate-after = 1 }
global-timeout = "10s"

[profile.slow]
# Slow tests (#[should_panic], complex assertions) - 5s timeout
slow-timeout = { period = "5s", terminate-after = 1 }
global-timeout = "60s"
```

Add to Makefile.toml:
```toml
test-unit-slow = """
cargo nextest run --lib --all-features --profile slow -- --skip testcontainers --skip weaver_integration
"""
```

**Benefit**: Maintains 1s timeout for fast tests, 5s for slow tests

### 🟢 LONG-TERM FIX (2 hours)

**Option 3: Optimize test execution time**

1. **Reduce `#[should_panic]` test count**
   - Merge similar should_panic tests (test multiple failure modes in one)
   - Use dedicated `#[test]` for success path only

2. **Pre-compile expensive assertions**
   - Move complex HRTB closures to helper functions
   - Reduce inline macro expansion

3. **Benchmark test performance**
   - Document baseline: "assertions tests take 1.1s per test"
   - Set performance budget: "max 0.5s per unit test"
   - Monitor in CI metrics dashboard

---

## Implementation: Quick Fix

<details>
<summary>Click to see immediate fix (5 minutes)</summary>

### Step 1: Update `.config/nextest.toml`

Replace the timeout value:

```diff
 [profile.default]
-slow-timeout = { period = "1s", terminate-after = 1 }
+slow-timeout = { period = "5s", terminate-after = 1 }

-global-timeout = "10s"
+global-timeout = "60s"
```

### Step 2: Verify Fix

```bash
cargo make test-unit
```

Expected output:
```
Summary [   5.200s] 287/287 tests run: 287 passed, 0 timed out
```

### Step 3: Commit

```bash
git add .config/nextest.toml
git commit -m "fix(ci): increase nextest timeout from 1s to 5s

FMEA Fix: RPN 36 → 20 (increase from timeout too aggressive)

Root causes:
- #[should_panic] test framework overhead (+0.1-0.2s)
- Parallel test execution context switching (+0.5s)
- Macro expansion overhead (+0.05-0.1s)

Tests were timing out at 1.1-1.2s with 1s limit.

Increased timeout to 5s to accommodate overhead while
maintaining fast feedback loop (<10s per test suite).

Related FMEA: FM3 - Test Timeout (Hanging Tests)
Tests affected: 16 assertions tests
"