# Andon Signals (Visual Problem Indicators) - Multi-Step Workflow

## Purpose

This command guides agents to treat compiler errors, test failures, and warnings as Andon signals - visual indicators that something is wrong and work should stop. Andon means "lantern" or "sign" - a visual signal that alerts to problems. Experts stop and fix problems immediately when signals appear.

## Workflow Overview

```
Step 1: Monitor Andon Signals (with Measurement) → Step 2: Stop When Signal Appears → Step 3: Investigate Root Cause → Step 4: Fix Root Cause → Step 5: Verify Signal Cleared (with Measurement & Control)
```

## Step-by-Step Instructions

### Step 1: Monitor Andon Signals

**Action**: Watch for visual signals that indicate problems.

**Andon signal types**:

1. **Compiler errors** - Red signals, must stop
   - Pattern: `error[E...]: <description>`
   - Severity: **CRITICAL** - Cannot proceed

2. **Compiler warnings** - Yellow signals, should stop
   - Pattern: `warning: <description>`
   - Severity: **HIGH** - Should fix before proceeding

3. **Test failures** - Red signals, must stop
   - Pattern: `test ... FAILED`
   - Severity: **CRITICAL** - Cannot proceed

4. **Linting errors** - Yellow/red signals, should stop
   - Pattern: Clippy warnings/errors
   - Severity: **HIGH** - Should fix before proceeding

5. **Performance regressions** - Yellow signals, investigate
   - Pattern: Tests taking longer than expected
   - Severity: **MEDIUM** - Investigate if significant

**Action**: Set up signal monitoring

```bash
# Monitor compilation signals
cargo make check
# Look for: error[...] or warning: patterns

# Monitor test signals
cargo make test
# Look for: test ... FAILED patterns

# Monitor linting signals
cargo make lint
# Look for: warning: or error: patterns
```

**Principle**: "Andon signals are visual management" - Make problems immediately visible, don't hide them.

#### 1.1: Collect Baseline Data (DMAIC Measurement)

**Action**: Measure current signal frequency to establish baseline.

**Data to collect**:
- **Signal count**: How many signals exist?
- **Signal frequency**: How often do signals appear?
- **Signal types**: What types of signals (errors, warnings, failures)?
- **Signal severity**: What is the severity distribution?

**Action**: Collect baseline data

```bash
# Count compiler errors
cargo make check 2>&1 | grep -c "error\["
# Output: 3 errors

# Count compiler warnings
cargo make check 2>&1 | grep -c "warning:"
# Output: 5 warnings

# Count test failures
cargo make test 2>&1 | grep -c "FAILED"
# Output: 2 failures

# Count linting errors
cargo make lint 2>&1 | grep -c "error:"
# Output: 1 linting error
```

**Example baseline data**:
```markdown
## Baseline Data

**Total Signals**: 11
**By Type**:
- Compiler errors: 3 (27%)
- Compiler warnings: 5 (45%)
- Test failures: 2 (18%)
- Linting errors: 1 (9%)

**By Severity**:
- CRITICAL: 5 (errors + failures)
- HIGH: 6 (warnings + linting)
- MEDIUM: 0
```

---

### Step 2: Stop When Signal Appears

**Action**: Immediately stop work when an Andon signal appears.

#### 2.1: Recognize Signal Severity

**Action**: Determine signal severity and response.

**Signal severity levels**:

- **CRITICAL (Red)** - Must stop immediately
  - Compiler errors
  - Test failures
  - **Response**: Stop all work, fix immediately

- **HIGH (Yellow)** - Should stop
  - Compiler warnings
  - Linting errors
  - **Response**: Stop current work, fix before proceeding

- **MEDIUM (Yellow)** - Investigate
  - Performance warnings
  - Code quality warnings
  - **Response**: Investigate, fix if significant

**Action**: Classify signal

```markdown
## Andon Signal Classification

### Critical Signals (Stop Immediately)
- [ ] Compiler error: `error[E0425]: cannot find function`
- [ ] Test failure: `test test_name ... FAILED`

### High Signals (Stop and Fix)
- [ ] Compiler warning: `warning: unused variable`
- [ ] Linting error: `clippy::unwrap_used`

### Medium Signals (Investigate)
- [ ] Performance warning: Test taking longer than expected
- [ ] Code quality warning: Complexity too high
```

#### 2.2: Stop the Line

**Action**: Stop current work when signal appears.

**Stop the line principles**:
- **Don't ignore** - Never ignore Andon signals
- **Don't proceed** - Don't continue work with signals present
- **Don't hide** - Don't suppress warnings/errors
- **Fix immediately** - Address signal before continuing

**Example response**:
```bash
# Signal appeared: Compiler error
cargo make check
# Output: error[E0425]: cannot find function `test_function`

# STOP: Do not proceed with other work
# ACTION: Fix compiler error immediately
```

---

### Step 3: Investigate Root Cause

**Action**: Understand why the signal appeared.

#### 3.1: Read Signal Message

**Action**: Carefully read the signal message.

**What to look for**:
- **Error message** - What went wrong?
- **Location** - Where did it occur?
- **Context** - What was happening when it occurred?

**Example**:
```
error[E0425]: cannot find function `test_function` in this scope
  --> src/test.rs:10:5
   |
10 |     test_function();
   |     ^^^^^^^^^^^^ not found in this scope
```

**Analysis**:
- **What**: Function `test_function` not found
- **Where**: `src/test.rs:10:5`
- **Why**: Function doesn't exist or not imported

#### 3.2: Trace Root Cause

**Action**: Use root cause analysis to find why signal appeared.

**Questions to ask**:
- Why did this signal appear?
- What changed that caused it?
- Is this a symptom of a deeper problem?

**Example root cause analysis**:
```markdown
## Root Cause Analysis

**Signal**: Compiler error - function not found
**Why #1**: Function `test_function` doesn't exist
**Why #2**: Function was removed during refactoring
**Why #3**: Tests weren't updated after refactoring
**Root Cause**: Missing test update after refactoring
```

**Reference**: See [Root Cause Analysis](./root-cause-analysis.md) for detailed 5 Whys process

#### 3.3: Verify Root Cause

**Action**: Confirm root cause hypothesis.

**Verification**:
- Does fixing root cause clear the signal?
- Does data support root cause hypothesis?
- Are there other contributing factors?

---

### Step 4: Fix Root Cause

**Action**: Address the underlying cause, not just the symptom.

#### 4.1: Fix the Problem

**Action**: Implement fix that addresses root cause.

**Fix principles**:
- **Fix root cause** - Not just symptom
- **Fix completely** - Don't leave partial fixes
- **Fix safely** - Don't introduce new problems

**Example fix**:
```rust
// Root cause: Function removed, test not updated
// Fix: Add missing function or update test

// Option 1: Add missing function
fn test_function() {
    // Implementation
}

// Option 2: Update test to use correct function
// test_function() -> actual_function()
```

#### 4.2: Verify Fix

**Action**: Ensure fix resolves the signal.

**Verification steps**:
1. Fix the problem
2. Re-run signal check
3. Verify signal cleared

**Example**:
```bash
# Fix applied
# Re-check signal
cargo make check
# Expected: No errors, signal cleared ✅
```

---

### Step 5: Verify Signal Cleared

**Action**: Confirm signal is resolved and won't return.

#### 5.1: Verify Signal Cleared

**Action**: Run checks to confirm signal gone.

**Verification**:
- ✅ Compiler errors cleared: `cargo make check`
- ✅ Test failures cleared: `cargo make test`
- ✅ Warnings cleared: `cargo make lint`
- ✅ No new signals appeared

**Example**:
```bash
# Verify all signals cleared
cargo make check   # No errors ✅
cargo make test    # All tests pass ✅
cargo make lint    # No warnings ✅
```

#### 5.2: Prevent Signal Return

**Action**: Add controls to prevent signal from returning.

**Prevention methods**:
- **Tests** - Add tests to catch regression
- **Linting** - Enable linting rules in CI
- **Documentation** - Document why fix was needed
- **Code review** - Review to prevent similar issues

**Example**:
```rust
// Add test to prevent regression
chicago_test!(test_function_exists, {
    // Test that would fail if function removed again
    assert!(test_function().is_ok());
});
```

#### 5.3: Measure Improvement (DMAIC Measurement)

**Action**: Measure improvement against baseline data.

**Measurement**:
- Re-count signals after fixes
- Compare to baseline
- Calculate improvement percentage
- Verify success criteria met

**Action**: Measure improvement

```bash
# Re-count signals after fixes
cargo make check 2>&1 | grep -c "error\["
# Output: 0 errors (down from 3)

cargo make test 2>&1 | grep -c "FAILED"
# Output: 0 failures (down from 2)

# Calculate improvement
# Baseline: 11 signals total
# After fixes: 0 signals
# Improvement: 100% (11/11 signals cleared)
```

**Example improvement measurement**:
```markdown
## Improvement Measurement

**Baseline**: 11 signals total
**After Fixes**: 0 signals
**Improvement**: 100% (11/11 signals cleared)

**By Type**:
- Compiler errors: 3 → 0 (100% improvement)
- Compiler warnings: 5 → 0 (100% improvement)
- Test failures: 2 → 0 (100% improvement)
- Linting errors: 1 → 0 (100% improvement)

**Success Criteria Met**: ✅
- All signals cleared ✅
- No new signals appeared ✅
```

#### 5.4: Establish Controls (DMAIC Control)

**Action**: Set up controls to prevent signals from returning.

**Controls**:
- **Automated checks**: Run checks automatically in CI
- **Pre-commit hooks**: Run checks before commits
- **Monitoring**: Track signal frequency over time
- **Alerts**: Set up alerts if signals appear

**Action**: Create todo list for controls (10+ items)

```markdown
## Andon Signal Control Todos (10+ items)

**Automated Checks**:
- [ ] Add CI check: Run `cargo make check` on every commit
- [ ] Add CI check: Run `cargo make test` on every commit
- [ ] Add CI check: Run `cargo make lint` on every commit
- [ ] Configure CI to fail if signals appear

**Pre-commit Controls**:
- [ ] Add pre-commit hook: Run checks before commit
- [ ] Configure hook to prevent commit if signals appear
- [ ] Verify pre-commit hooks work correctly
- [ ] Document hook usage

**Monitoring Controls**:
- [ ] Set up signal frequency tracking dashboard
- [ ] Configure alerts if signal count > 0
- [ ] Review signal trends weekly
- [ ] Document signal patterns

**Standards Controls**:
- [ ] Add standard: No signals allowed before commit
- [ ] Add standard: Fix signals immediately when they appear
- [ ] Update team documentation with standards
- [ ] Verify standards are followed
```

**Execution**:
1. Create todos using `todo_write` tool (10+ items minimum)
2. Execute todos one by one (implement controls)
3. Mark todos as completed as controls are implemented
4. Verify each control works before moving to next
5. Continue until all controls implemented

**Principle**: Implement controls to prevent signals, don't just document them. Todos track progress, controls prevent recurrence.

#### 5.5: Monitor for New Signals (DMAIC Control)

**Action**: Continue monitoring for new signals with systematic tracking.

**Monitoring**:
- Run checks regularly
- Don't ignore warnings
- Fix signals immediately
- Track signal frequency over time
- Set up alerts for regression

**Action**: Set up monitoring

```bash
# Monitor signal frequency
# Run daily: 
#   cargo make check 2>&1 | grep -c "error\["
#   cargo make test 2>&1 | grep -c "FAILED"
# Alert if signal count > 0

# Track trends
# Week 1: 11 signals (baseline)
# Week 2: 0 signals (after fixes)
# Week 3: 0 signals (controls working)
# Week 4: 0 signals (sustained)
```

---

## Complete Workflow Example

```bash
# Step 1: Monitor Andon Signals
cargo make check
# Signal appeared: error[E0425]: cannot find function

# Step 2: Stop When Signal Appears
# STOP: Do not proceed with other work
# ACTION: Fix compiler error immediately

# Step 3: Investigate Root Cause
# Root cause: Function removed during refactoring, test not updated

# Step 4: Fix Root Cause
# Fix: Add missing function or update test
# Applied fix

# Step 5: Verify Signal Cleared
cargo make check  # No errors ✅
cargo make test   # All tests pass ✅
# Signal cleared, work can proceed
```

## Andon Signal Response Matrix

| Signal Type | Severity | Response | Example |
|------------|----------|----------|---------|
| Compiler error | CRITICAL | Stop immediately, fix now | `error[E0425]` |
| Test failure | CRITICAL | Stop immediately, fix now | `test ... FAILED` |
| Compiler warning | HIGH | Stop current work, fix before proceeding | `warning: unused` |
| Linting error | HIGH | Stop current work, fix before proceeding | `clippy::unwrap_used` |
| Performance warning | MEDIUM | Investigate, fix if significant | Test timeout |

## Integration with Other Commands

- **[Root Cause Analysis](./root-cause-analysis.md)** - Use 5 Whys in Step 3 to find root cause
- **[DMAIC Problem Solving](./dmaic-problem-solving.md)** - Use DMAIC measurement and control steps integrated into this workflow
- **[Gemba Walk](./gemba-walk.md)** - Go to source in Step 3 to investigate
- **[Poka-Yoke Design](./poka-yoke-design.md)** - Use type system in Step 4 to prevent signals

## Expert Insights

**Why this matters**: Ignoring signals leads to accumulating problems. Experts treat every signal as important and fix them immediately.

**Key principle**: "Stop the line" - When an Andon signal appears, stop work and fix the problem immediately. Don't proceed with problems present.

**Remember**: Andon signals are visual management. They make problems immediately visible. Don't hide them, don't ignore them, fix them.

**Andon culture**: In Lean manufacturing, any worker can stop the production line if they see a problem. In coding, any developer should stop and fix problems when signals appear. This prevents defects from propagating.

**DfLSS alignment**: Andon signals help prevent both defects (quality) and waste (efficiency) - stopping problems early prevents rework (waste) and defects from propagating (quality). This aligns with DfLSS (Design for Lean Six Sigma) principles. Don't conflate DfLSS with DFSS (Design for Six Sigma) - DFSS only addresses quality, missing critical waste elimination. See [Root Cause Analysis - DfLSS vs DFSS](./root-cause-analysis.md#dflss-vs-dfss-critical-distinction) for why conflating DfLSS with DFSS is a huge error.

