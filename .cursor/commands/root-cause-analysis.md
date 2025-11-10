# Root Cause Analysis (5 Whys) - Multi-Step Workflow

## Purpose

This command guides agents through root cause analysis using the 5 Whys technique. Root cause analysis finds the underlying cause of problems, not just symptoms. Experts dig deeper to find root causes rather than fixing symptoms.

## Workflow Overview

```
Step 1: Define the Problem → Step 2: Ask Why #1 → Step 3: Ask Why #2-5 → Step 4: Verify Root Cause → Step 5: Fix Root Cause
```

## Step-by-Step Instructions

### Step 1: Define the Problem

**Action**: Clearly state the observable problem (symptom).

**Problem definition format**:
- **What**: What is the observable problem?
- **Where**: Where does it occur?
- **When**: When does it occur?
- **Impact**: What is the impact?

**Example problem definition**:
```markdown
## Problem Definition

**What**: Test fails with "assertion failed: expected 1000, got 999"
**Where**: `tests/concurrent_test.rs` - `test_concurrent_increment`
**When**: Approximately 30% of test runs, more frequent in CI
**Impact**: Blocks CI/CD pipeline, causes false negatives
```

**Principle**: Start with the observable symptom, not assumptions about cause.

---

### Step 2: Ask Why #1

**Action**: Ask why the problem occurred (first level).

**Why #1 question**: "Why did [problem] occur?"

**Answer format**:
- Direct cause of the symptom
- Observable fact, not assumption
- Can be verified

**Example**:
```markdown
## 5 Whys Analysis

**Problem**: Test fails with "expected 1000, got 999"

**Why #1**: Why did the test fail?
**Answer**: Counter value was 999 instead of expected 1000

**Verification**: 
- Test output shows actual value 999
- Expected value was 1000
- One increment operation didn't complete
```

---

### Step 3: Ask Why #2-5

**Action**: Continue asking why until root cause found.

**Process**:
- Ask "Why?" for each answer
- Continue until root cause found (usually 3-5 whys)
- Each answer should be deeper than previous
- Root cause is something that, if fixed, prevents the problem

**Example continued**:
```markdown
## 5 Whys Analysis (Continued)

**Why #2**: Why was counter 999 instead of 1000?
**Answer**: One increment operation didn't complete

**Why #3**: Why didn't one increment complete?
**Answer**: Race condition - two threads read same value before incrementing

**Why #4**: Why did race condition occur?
**Answer**: Mutex lock was released too early, allowing concurrent reads

**Why #5**: Why was mutex lock released too early?
**Answer**: Lock scope didn't include the entire increment operation (ROOT CAUSE)

**Root Cause**: Lock scope is too narrow - doesn't protect entire increment operation
```

**Key insight**: Root cause is usually a process or design issue, not a person or one-time event.

---

### Step 4: Verify Root Cause

**Action**: Confirm root cause hypothesis.

#### 4.1: Test Root Cause Hypothesis

**Action**: Verify that fixing root cause prevents the problem.

**Verification questions**:
- If we fix the root cause, will the problem be prevented?
- Does the data support the root cause hypothesis?
- Are there other contributing factors?

**Example verification**:
```rust
// Root cause hypothesis: Lock scope too narrow
// Test: Expand lock scope and verify problem prevented

// Before (root cause present)
let mut value = counter.lock().unwrap();
let current = *value;
drop(value); // Lock released too early
*value = current + 1; // Race condition possible

// After (root cause fixed)
let mut value = counter.lock().unwrap();
*value += 1; // Entire operation protected by lock
// Lock released after operation complete

// Verification: Run test 100 times, should have 0 failures
```

#### 4.2: Check for Contributing Factors

**Action**: Identify other factors that contribute to the problem.

**Contributing factors**:
- Factors that make problem more likely
- Factors that make problem worse
- Factors that prevent detection

**Example**:
```markdown
## Contributing Factors

**Root Cause**: Lock scope too narrow

**Contributing Factors**:
- Test runs in CI with multiple threads (makes race condition more likely)
- No flaky test detection (problem not caught early)
- Insufficient test coverage for concurrent code (problem not prevented)

**Note**: Fix root cause first, then address contributing factors
```

---

### Step 5: Fix Root Cause

**Action**: Implement fix that addresses root cause.

#### 5.1: Design Fix

**Action**: Design solution that addresses root cause.

**Fix criteria**:
- Addresses root cause (not just symptom)
- Prevents problem from recurring
- Doesn't introduce new problems
- Is maintainable

**Example fix design**:
```markdown
## Fix Design

**Root Cause**: Lock scope too narrow

**Fix**: Expand lock scope to include entire increment operation

**Implementation**:
1. Keep lock for entire increment operation
2. Don't release lock until operation complete
3. Verify fix with tests

**Prevention**: Add test that would catch this pattern
```

#### 5.2: Implement Fix

**Action**: Implement the fix.

**Implementation steps**:
1. Make code changes
2. Verify compilation: `cargo make check`
3. Run tests: `cargo make test`
4. Verify fix: Run test multiple times

**Example implementation**:
```rust
// Fix: Expand lock scope
let mut value = counter.lock().unwrap();
*value += 1; // Entire operation protected by lock
// Lock released here, after operation complete
```

#### 5.3: Verify Fix

**Action**: Ensure fix prevents the problem.

**Verification**:
- ✅ Problem doesn't occur: Run test 100 times, 0 failures
- ✅ No regressions: Other tests still pass
- ✅ Root cause addressed: Lock scope now correct

**Example verification**:
```bash
# Verify fix prevents problem
for i in {1..100}; do
    cargo make test test_concurrent_increment
done
# Expected: 0 failures (problem prevented)

# Verify no regressions
cargo make test
# Expected: All tests pass
```

#### 5.4: Prevent Recurrence

**Action**: Add controls to prevent root cause from returning.

**Prevention methods**:
- **Tests** - Add test that would catch root cause
- **Code review** - Review to prevent similar issues
- **Documentation** - Document why fix was needed
- **Standards** - Establish pattern to follow

**Example prevention**:
```rust
// Add test to prevent root cause from returning
chicago_test!(test_lock_scope_covers_operation, {
    // Test that would fail if lock scope too narrow
    // This prevents root cause from returning
});

// Document pattern
/// Increment counter with proper lock scope.
/// 
/// **Root Cause Fix**: Lock scope covers entire increment operation.
/// Pattern: Keep lock for entire critical section, not just read.
fn increment_counter(counter: &Mutex<u32>) {
    let mut value = counter.lock().unwrap();
    *value += 1; // Entire operation protected
}
```

---

## Complete Workflow Example

```markdown
# Step 1: Define the Problem
Problem: Test fails with "expected 1000, got 999"

# Step 2: Ask Why #1
Why #1: Counter value was 999 instead of 1000

# Step 3: Ask Why #2-5
Why #2: One increment operation didn't complete
Why #3: Race condition - two threads read same value
Why #4: Mutex lock released too early
Why #5: Lock scope didn't include entire operation (ROOT CAUSE)

# Step 4: Verify Root Cause
Test: Expand lock scope, verify problem prevented
Result: Problem prevented ✅

# Step 5: Fix Root Cause
Fix: Expand lock scope to include entire increment
Verify: Run test 100 times, 0 failures ✅
Prevent: Add test to catch pattern
```

## 5 Whys Best Practices

**Guidelines**:
1. **Start with symptom** - Begin with observable problem
2. **Ask why, not who** - Focus on process, not blame
3. **Dig deep** - Usually need 3-5 whys to find root cause
4. **Verify root cause** - Test that fixing it prevents problem
5. **Fix root cause** - Not just symptom

**Common mistakes**:
- ❌ Stopping too early (fixing symptom, not cause)
- ❌ Blaming people instead of process
- ❌ Not verifying root cause
- ❌ Fixing symptoms instead of root cause

## Integration with Other Commands

- **[DMAIC Problem Solving](./dmaic-problem-solving.md)** - Use 5 Whys in Analyze step
- **[Gemba Walk](./gemba-walk.md)** - Go to source to verify root cause
- **[Andon Signals](./andon-signals.md)** - Use 5 Whys when signals appear
- **[Poka-Yoke Design](./poka-yoke-design.md)** - Use type system to prevent root cause

## Expert Insights

**Why this matters**: Fixing symptoms doesn't solve problems. Root cause analysis finds underlying causes that, when fixed, prevent problems from recurring.

**Key principle**: "Ask why five times" - Usually need to dig 3-5 levels deep to find root cause. Surface-level fixes don't prevent recurrence.

**Remember**: Root cause is usually a process or design issue, not a person or one-time event. Focus on fixing the process, not blaming people.

**5 Whys principle**: Each "why" should be deeper than the previous. Root cause is something that, if fixed, prevents the entire chain of problems.

