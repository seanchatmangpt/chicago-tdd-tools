# Andon Signals in GitHub Actions CI/CD Pipeline

**Andon** (Japanese: 暗点 or "lantern") signals are visual indicators that alert to problems and stop the line. In our CI/CD pipeline, Andon signals represent different severity levels of failures that require immediate attention.

## Signal Types and Severity Levels

### 🔴 CRITICAL (Red) - Stop Immediately

**Meaning**: Work cannot proceed. These signals block merge and require immediate fix.

**Examples**:
- **Test failures** - Unit tests failing on any OS/Rust version
- **Compiler errors** - Code that doesn't compile
- **Production panics** - `unwrap()`, `expect()`, or `panic!()` in production code
- **Format violations** - Code not formatted with rustfmt

**Response**:
1. ⏸️ Stop all work immediately
2. 🔍 Investigate root cause (see Step 3 below)
3. 🔧 Fix root cause (not just symptom)
4. ✅ Verify signal cleared by re-running CI

---

### 🟡 HIGH (Yellow) - Stop and Fix

**Meaning**: Work should pause to fix these issues before continuing.

**Examples**:
- **Lint warnings** - Clippy warnings about code quality
- **Format issues** - Code style inconsistencies

**Response**:
1. ⏹️ Stop current work
2. 🔍 Understand the warning
3. 🔧 Fix using suggested approach
4. ✅ Verify fix clears the signal

**Quick Fixes**:
```bash
# Format issues
just fmt

# Lint warnings
just lint  # Review warnings
# Fix code or add #[allow] with justification
```

---

### 🟠 MEDIUM (Orange) - Investigate

**Meaning**: These are warning signals that need investigation but don't block merge.

**Examples**:
- **Coverage below threshold** - Tests don't cover 70% of code
- **Performance regressions** - Tests taking longer than expected

**Response**:
1. 📊 Review the metric
2. 🔍 Investigate if significant
3. 📈 Improve if it aligns with goals
4. 📝 Document decision if intentional

---

## CI Pipeline Jobs and Their Signals

| Job | Signal | Severity | What it Checks | Response |
|-----|--------|----------|---|---|
| **🟡 Lint Check** | Clippy warnings | HIGH | Code quality, best practices | `just lint` + fix |
| **🟡 Format Check** | rustfmt issues | HIGH | Code style consistency | `just fmt` |
| **🔴 Unit Tests** | Test failures | CRITICAL | Functionality works correctly | Fix failing test logic |
| **🔴 Production Safety** | unwrap/expect | CRITICAL | No panics in production | Use `?` operator or error handling |
| **🟠 Code Coverage** | Coverage below 70% | MEDIUM | Test comprehensiveness | Add tests if important |

---

## The Five-Step Andon Response Workflow

### Step 1: Monitor Andon Signals

**Action**: Watch CI pipeline for red/yellow/orange indicators.

**How**: Check your GitHub Actions workflow runs. Signals appear in:
- Job names (with emoji: 🔴🟡🟠)
- Step outputs (showing which signal was triggered)
- CI summary (aggregated Andon status)

### Step 2: Stop When Signal Appears

**Action**: Immediately stop other work when you see a signal.

**Remember**:
- 🔴 CRITICAL → Stop immediately
- 🟡 HIGH → Stop current task
- 🟠 MEDIUM → Investigate before continuing

### Step 3: Investigate Root Cause

**Action**: Use 5 Whys to understand why the signal appeared.

**Example - Test Failure**:
```
Signal: Tests failing on Windows
Why 1: A specific test always fails on Windows
Why 2: Test assumes Unix path separators
Why 3: Test uses hardcoded "/" instead of Path
Why 4: Path handling wasn't tested on Windows
Why 5: No cross-platform test coverage

Root Cause: Missing cross-platform test coverage
```

**Tools**:
- Read CI job logs carefully
- Check git diff for recent changes
- Review error messages for patterns
- Look at which OS/Rust version combination fails

### Step 4: Fix Root Cause

**Action**: Fix the underlying problem, not just the symptom.

**Fix Types**:

| Signal Type | Symptom | Fix |
|---|---|---|
| Lint warning | Too many nesting levels | Refactor function into smaller pieces |
| Format issue | Wrong spacing | Run `just fmt` |
| Test failure | Assertion mismatch | Fix test logic or implementation |
| Production panic | `unwrap()` in code | Use `?` operator or error handling |
| Coverage low | 65% coverage | Add tests for untested paths |

**Verification**:
```bash
# Re-run the specific check
just lint          # For lint signals
just fmt           # For format signals
just test-unit     # For test signals
just pre-commit    # All local checks
just ci-local      # Simulate CI
```

### Step 5: Verify Signal Cleared

**Action**: Confirm signal is gone and won't return.

**Verification Steps**:
1. ✅ Local checks pass: `just pre-commit`
2. ✅ CI simulation passes: `just ci-local`
3. ✅ Push to branch and verify GitHub Actions passes
4. ✅ Commit message documents the fix
5. ✅ Add test if appropriate to prevent regression

---

## Common Andon Signals and Fixes

### 🔴 Test Failure: `assertion failed`

**Signal**:
```
test result::test_example ... FAILED
assertion failed: expected 10, got 5
```

**Steps to Fix**:
1. Read the assertion message carefully
2. Check test logic (did you test the right thing?)
3. Check implementation (is it correct?)
4. Run test locally: `just test-unit`
5. Add debug output if unclear
6. Commit fix with clear message

### 🟡 Lint Warning: `clippy::unwrap_used`

**Signal**:
```
warning: used `unwrap()` on a `Result` value
--> src/main.rs:42:13
```

**Steps to Fix**:
1. Review the code context
2. Replace with `?` operator if in a Result-returning function:
   ```rust
   // ❌ Before
   let value = operation().unwrap();

   // ✅ After
   let value = operation()?;
   ```
3. Or use error handling:
   ```rust
   let value = match operation() {
       Ok(v) => v,
       Err(e) => {
           alert_warning!("Operation failed: {}", e);
           default_value
       }
   };
   ```
4. Run lint check: `just lint`

### 🟡 Format Issue

**Signal**:
```
❌ ANDON SIGNAL DETECTED: Code is not formatted correctly
```

**Steps to Fix**:
1. Run formatter: `just fmt`
2. Review changes: `git diff`
3. Commit: `git add . && git commit -m "style: format code"`

### 🔴 Production Panic: `unwrap()` in src/

**Signal**:
```
❌ ANDON SIGNAL DETECTED: Production code contains panics!
Found 3 unwrap() calls in src/core/fixture.rs
```

**Steps to Fix**:
1. Open the file shown in the error
2. Locate each unwrap() call
3. Replace with proper error handling:
   - Use `?` if in Result-returning function
   - Use `if let` or `match` for error handling
   - Use `default_value` if recovery is possible
4. Test locally: `just test-unit`
5. Re-run safety check via CI

### 🟠 Coverage Below Threshold

**Signal**:
```
🟠 ANDON SIGNAL: Coverage below recommended 70% threshold
   Current coverage: 65%
   Status: ⚠️  WARNING (investigative signal)
```

**Steps to Address**:
1. Identify untested code paths
2. Add tests for critical paths
3. Run coverage locally: `just coverage`
4. Focus on error paths and edge cases
5. Verify coverage improves

---

## Preventing Andon Signals

### Install Pre-commit Hook

Catch signals locally before pushing to GitHub:

```bash
# Install hook (one-time setup)
cargo install just
./scripts/install-hooks.sh

# Or manual installation
just install-hooks
```

**What it does**:
- Runs format check
- Runs lint check
- Runs unit tests
- Checks for unwrap/expect
- Blocks commit if any fail

### Local CI Simulation

Before pushing, simulate the full CI:

```bash
# Runs all checks locally (faster than GitHub Actions)
just ci-local
```

**Equivalent to**:
- fmt check
- lint check
- unit tests on current OS
- unwrap/expect check
- coverage check

### Best Practices

1. **Commit frequently**: Smaller commits are easier to debug
2. **Test before pushing**: Run `just pre-commit` before every commit
3. **Read warnings**: Don't ignore clippy warnings - they catch real issues
4. **Fix immediately**: Don't accumulate signals
5. **Understand the fix**: Don't just blindly apply suggestions

---

## CI Pipeline Signal Flow

```
┌─────────────────────────────────────────┐
│  Code pushed to GitHub                  │
└────────────────┬────────────────────────┘
                 ↓
        ┌────────────────┐
        │ 🟡 Lint Check  │
        └────────┬───────┘
                 ↓ (if fail → 🔴 CRITICAL)
        ┌────────────────┐
        │ 🟡 Format Check │
        └────────┬───────┘
                 ↓ (if fail → 🔴 CRITICAL)
        ┌────────────────────┐
        │ 🔴 Unit Tests      │ (multi-OS)
        └────────┬───────────┘
                 ↓ (if fail → 🔴 CRITICAL)
        ┌──────────────────────────────┐
        │ 🔴 Production Safety Check   │
        │ (no unwrap/expect)           │
        └────────┬─────────────────────┘
                 ↓ (if fail → 🔴 CRITICAL)
        ┌────────────────────┐
        │ 🟠 Coverage Check  │
        └────────┬───────────┘
                 ↓ (if warning → 🟠 MEDIUM)
        ┌────────────────────────┐
        │ Andon Signal Summary    │
        └────────┬───────────────┘
                 ↓
        ┌─────────────────────────────────┐
        │ All Signals Clear? → Ready ✅  │
        │ Signal Active? → STOP 🔴       │
        └─────────────────────────────────┘
```

---

## Andon Signal Response Checklist

When you see an Andon signal:

- [ ] **Read** the error message carefully
- [ ] **Understand** what went wrong (root cause)
- [ ] **Locate** the problematic code
- [ ] **Fix** the underlying issue (not symptom)
- [ ] **Verify** locally with `just pre-commit`
- [ ] **Test** with `just ci-local`
- [ ] **Commit** with descriptive message
- [ ] **Verify** GitHub Actions passes
- [ ] **Add test** if needed to prevent regression
- [ ] **Document** if it's a complex fix

---

## When Signals Keep Appearing

If the same signal keeps appearing:

1. **Pattern Analysis**: What's the pattern?
2. **Root Cause**: Use 5 Whys to find root cause
3. **System Fix**: Can we prevent this type of problem?
4. **Automation**: Add check to `Justfile` or CI if possible
5. **Documentation**: Document the solution for others

**Examples**:
- Repeated format issues → Add pre-commit hook
- Repeated lint warnings → Review clippy rules
- Repeated test failures → Better test isolation

---

## Resources

- 📚 **Andon Signals Workflow**: See `andon-signals-workflow.md` for 5-step process
- 📚 **Root Cause Analysis**: See `docs/process/SPR_GUIDE.md`
- 🔗 **Pre-commit Setup**: `./scripts/install-hooks.sh`
- 📖 **Build Commands**: `just --list`

---

## Philosophy

Andon signals are **visual management**. They make problems immediately visible, so:

1. **Problems don't hide** - Signals are obvious
2. **Work stops for problems** - Can't merge with signals
3. **Root causes are fixed** - Prevents problem recurrence
4. **Quality is built-in** - Signals prevent defects early

**Key Principle**: Stop the line when signals appear. Fix immediately. Resume work only when signals are clear.

This prevents accumulation of problems and ensures quality is the default, not an afterthought.
