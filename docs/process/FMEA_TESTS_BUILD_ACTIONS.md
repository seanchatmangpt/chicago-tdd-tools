# FMEA: Tests, Build System, and GitHub Actions

**Document**: Failure Mode and Effects Analysis (FMEA)
**System**: Chicago TDD Tools - Testing, Build, and CI/CD Infrastructure
**Date**: 2025-11-14
**Purpose**: Proactive risk assessment to identify and prevent failures in tests, build system, and GitHub Actions

## FMEA Rating Scale

### Severity (S): Impact of failure
- **10**: Critical - System unusable, data loss, security breach
- **8-9**: High - Major functionality broken, blocks release
- **6-7**: Medium - Significant degradation, workarounds exist
- **4-5**: Low - Minor inconvenience, cosmetic issues
- **1-3**: Negligible - No real impact

### Occurrence (O): Probability of failure
- **10**: Very High - >30% of the time
- **8-9**: High - 10-30% of the time
- **6-7**: Medium - 1-10% of the time
- **4-5**: Low - 0.1-1% of the time
- **1-3**: Very Low - <0.1% of the time

### Detection (D): Ability to detect before impact
- **10**: Cannot detect - No detection mechanism
- **8-9**: Very Low - Detection rare or unreliable
- **6-7**: Low - Detection requires manual inspection
- **4-5**: Medium - Automated detection, some gaps
- **1-3**: High - Automated detection, reliable

### Risk Priority Number (RPN)
**RPN = Severity × Occurrence × Detection**

- **RPN > 200**: Critical - Immediate action required
- **RPN 100-200**: High - Action required soon
- **RPN 50-100**: Medium - Monitor and plan mitigation
- **RPN < 50**: Low - Accept or defer

---

## Part 1: Test System FMEA

### Test Failure Mode #1: Flaky Tests (Race Conditions)

**Failure Mode**: Tests fail intermittently due to race conditions or timing issues

**Effects**:
- False negatives block CI/CD pipeline
- Developer time wasted investigating non-issues
- Loss of trust in test suite
- Delayed releases

**Causes**:
- Concurrent test execution without proper synchronization
- Shared state between tests
- Timing dependencies (sleep/delays)
- External resource contention

**Current Controls**:
- Single-threaded test mode available (`test-single-threaded`)
- cargo-nextest for better timeout enforcement

**Ratings**:
- **Severity**: 8 (Blocks CI/CD, wastes time)
- **Occurrence**: 3 (Low - seen in testcontainers_tests.rs historically)
- **Detection**: 5 (Medium - requires multiple test runs)
- **RPN**: **120** (HIGH RISK)

**Recommended Actions**:
1. Add test retry logic in CI (1-2 retries for failed tests)
2. Implement test flakiness detection (track failure rates)
3. Add explicit synchronization to concurrent tests
4. Use deterministic test data generators (fixed seeds)
5. Document timing-sensitive tests with comments

---

### Test Failure Mode #2: Tests Pass Locally, Fail in CI

**Failure Mode**: Tests pass on developer machines but fail in GitHub Actions CI

**Effects**:
- CI pipeline blocked
- Developer confusion and frustration
- Delayed integration
- Hidden environment dependencies

**Causes**:
- Different OS (Linux CI vs macOS/Windows dev)
- Docker not available in CI
- Environment variable differences
- Resource constraints (CPU/memory limits in CI)
- Timing differences (CI slower than local)

**Current Controls**:
- `docker-check` task verifies Docker availability
- Timeout enforcement prevents hanging
- Feature flags (`testcontainers`, `weaver`) allow skipping integration tests

**Ratings**:
- **Severity**: 7 (Blocks CI, but workarounds exist)
- **Occurrence**: 5 (Medium - platform differences are common)
- **Detection**: 3 (High - CI catches immediately)
- **RPN**: **105** (HIGH RISK)

**Recommended Actions**:
1. Add CI environment matrix (test on multiple OSes)
2. Document required CI environment variables
3. Add pre-CI local check: `cargo make ci-local` that simulates CI
4. Use Docker containers for local development (match CI environment)
5. Add environment validation task to CI

---

### Test Failure Mode #3: Test Timeout (Hanging Tests)

**Failure Mode**: Tests hang indefinitely, never completing

**Effects**:
- CI pipeline blocked indefinitely
- Resource waste (CI runners stuck)
- Developer blocked waiting for results
- Manual intervention required

**Causes**:
- Deadlocks in concurrent code
- Infinite loops
- Waiting for external resources that never respond
- Docker container startup failures

**Current Controls**:
- Timeout enforcement on ALL test tasks (10s unit, 30s integration)
- cargo-nextest with per-test timeouts
- `timeout` command wraps all cargo commands

**Ratings**:
- **Severity**: 9 (Blocks CI indefinitely, requires manual intervention)
- **Occurrence**: 2 (Very Low - timeout controls very effective)
- **Detection**: 2 (High - timeouts detect automatically)
- **RPN**: **36** (LOW RISK - Well controlled)

**Recommended Actions**:
1. Monitor timeout occurrences (log when timeouts trigger)
2. Add timeout alerts to CI (notify on timeout)
3. Document expected test duration in comments
4. Add per-test timeout configuration in nextest

---

### Test Failure Mode #4: Missing Test Coverage

**Failure Mode**: Critical code paths have no test coverage

**Effects**:
- Bugs reach production
- Regression risks increase
- Refactoring becomes risky
- Quality degradation over time

**Causes**:
- New features added without tests
- Edge cases not considered
- Error paths not tested
- Integration scenarios missing

**Current Controls**:
- Coverage tasks available (`coverage`, `coverage-report`)
- Manual process (not enforced)

**Ratings**:
- **Severity**: 8 (Bugs reach production, quality degradation)
- **Occurrence**: 6 (Medium - easy to miss coverage)
- **Detection**: 7 (Low - requires manual coverage check)
- **RPN**: **336** (CRITICAL RISK)

**Recommended Actions**:
1. **CRITICAL**: Add coverage enforcement to CI (fail if coverage drops)
2. Set minimum coverage threshold (e.g., 80%)
3. Add coverage badges to README
4. Require coverage increase for PRs touching existing code
5. Add coverage report to PR comments automatically

---

### Test Failure Mode #5: Docker Not Available

**Failure Mode**: Integration tests fail because Docker daemon is not running

**Effects**:
- Integration tests cannot run
- Incomplete test coverage
- Integration issues not detected until deployment

**Causes**:
- Docker daemon stopped or crashed
- Docker not installed on CI runner
- Docker socket permissions issues
- Docker service degradation

**Current Controls**:
- `docker-check` task fails fast if Docker unavailable
- Integration tests depend on `docker-check`
- Tests can be skipped with feature flags

**Ratings**:
- **Severity**: 6 (Integration tests skipped, but detected)
- **Occurrence**: 4 (Low - Docker usually stable)
- **Detection**: 2 (High - docker-check detects immediately)
- **RPN**: **48** (LOW RISK)

**Recommended Actions**:
1. Add Docker health check to CI setup phase
2. Document Docker requirements clearly in README
3. Add fallback: mock Docker tests for when Docker unavailable
4. Monitor Docker availability in CI metrics

---

### Test Failure Mode #6: Test Data Corruption

**Failure Mode**: Tests modify shared test data, affecting other tests

**Effects**:
- Test order dependency (tests pass/fail based on run order)
- Flaky test failures
- Difficult to debug issues
- Loss of test isolation

**Causes**:
- Shared mutable state
- Tests modifying fixtures in-place
- Global variables
- File system modifications not cleaned up

**Current Controls**:
- TestFixture design provides isolation
- Per-test unique counters
- Resource cleanup in Drop implementations

**Ratings**:
- **Severity**: 7 (Test isolation broken, flaky failures)
- **Occurrence**: 4 (Low - TestFixture design prevents this)
- **Detection**: 6 (Low - requires careful observation)
- **RPN**: **168** (HIGH RISK)

**Recommended Actions**:
1. Audit tests for shared mutable state
2. Enforce test isolation in code review checklist
3. Add test for test isolation (verify tests pass in any order)
4. Document test data best practices
5. Use read-only test data where possible

---

## Part 2: Build System FMEA

### Build Failure Mode #1: Task Timeout Expiration

**Failure Mode**: Build tasks timeout before completing

**Effects**:
- Build fails despite valid code
- CI blocked unnecessarily
- Developer frustration
- False negatives

**Causes**:
- Timeout too short for task
- Slow CI runners
- Network issues (downloading dependencies)
- Compile-time code generation delays

**Current Controls**:
- Different timeouts for different tasks (5s check, 30s build-release)
- Timeout values tuned based on experience
- Timeout-check task verifies timeout command exists

**Ratings**:
- **Severity**: 7 (Build fails unnecessarily, blocks progress)
- **Occurrence**: 3 (Very Low - timeouts well-tuned)
- **Detection**: 2 (High - timeout errors clear)
- **RPN**: **42** (LOW RISK)

**Recommended Actions**:
1. Monitor timeout occurrences (track which tasks timeout)
2. Add CI performance metrics (track build duration trends)
3. Consider dynamic timeout adjustment based on CI load
4. Document timeout tuning rationale in Makefile.toml

---

### Build Failure Mode #2: Dependency Resolution Failure

**Failure Mode**: Cargo cannot resolve dependencies or downloads fail

**Effects**:
- Build fails completely
- CI blocked
- Development blocked
- Cannot install or run project

**Causes**:
- Crates.io unavailable or degraded
- Network issues
- Dependency version conflicts
- Yanked dependencies

**Current Controls**:
- Cargo.lock pins exact versions
- 15s timeout on audit tasks (network operations)
- Cargo caching in CI (actions/cache@v4)

**Ratings**:
- **Severity**: 10 (Build completely blocked)
- **Occurrence**: 2 (Very Low - Crates.io very reliable)
- **Detection**: 1 (High - Cargo error messages clear)
- **RPN**: **20** (LOW RISK)

**Recommended Actions**:
1. Add dependency mirror/cache for critical dependencies
2. Monitor Crates.io status automatically
3. Add retry logic for network operations
4. Document dependency resolution troubleshooting

---

### Build Failure Mode #3: Clippy Lint Failures

**Failure Mode**: Clippy lint check fails, blocking commit/CI

**Effects**:
- CI blocked
- Commit blocked
- Developer must fix lints
- Potential delay in integration

**Causes**:
- New clippy warnings introduced
- Code doesn't follow lint standards
- `#[allow]` attributes missing where needed

**Current Controls**:
- Clippy runs in pre-commit (`cargo make pre-commit`)
- CI enforces clippy (`cargo make lint`)
- `-D warnings` treats warnings as errors (Poka-Yoke)
- Documentation of SPR lint standards

**Ratings**:
- **Severity**: 5 (Blocks commit, but fixable)
- **Occurrence**: 6 (Medium - developers forget to run pre-commit)
- **Detection**: 2 (High - caught by pre-commit or CI)
- **RPN**: **60** (MEDIUM RISK)

**Recommended Actions**:
1. Add Git pre-commit hook (automatic, not manual)
2. Add IDE integration (clippy warnings in editor)
3. Add quick fix suggestions in CI output
4. Document common clippy fixes in SPR Guide

---

### Build Failure Mode #4: Build Artifact Corruption

**Failure Mode**: Build artifacts are corrupted or incomplete

**Effects**:
- Tests run against wrong code
- Release artifacts broken
- Deployment failures
- Runtime errors in production

**Causes**:
- Partial builds not cleaned
- Out-of-date target/ directory
- Incremental compilation bugs
- Disk space issues

**Current Controls**:
- `cargo clean` task available
- `clean-all-home` for comprehensive cleanup
- CI builds from clean state each time

**Ratings**:
- **Severity**: 9 (Broken releases, production issues)
- **Occurrence**: 2 (Very Low - Cargo incremental compilation reliable)
- **Detection**: 5 (Medium - may not be detected until runtime)
- **RPN**: **90** (MEDIUM RISK)

**Recommended Actions**:
1. Add build artifact validation (checksum verification)
2. Add `cargo clean` to pre-commit workflow
3. Monitor disk space on CI runners
4. Add release artifact smoke tests

---

### Build Failure Mode #5: Unwrap/Expect in Production Code

**Failure Mode**: Production code contains `.unwrap()` or `.expect()` calls that panic at runtime

**Effects**:
- Runtime panics in production
- Service crashes
- Data loss
- Poor user experience

**Causes**:
- Developers forget to handle errors properly
- Code copied from examples/tests
- Refactoring introduces unwrap
- Lack of code review

**Current Controls**:
- `check-unwrap-staged` and `check-expect-staged` tasks
- Pre-commit validation (blocks commit)
- Manual process (developer must run pre-commit)

**Ratings**:
- **Severity**: 9 (Production crashes, data loss)
- **Occurrence**: 5 (Medium - easy to introduce accidentally)
- **Detection**: 4 (Medium - caught if pre-commit run, missed otherwise)
- **RPN**: **180** (HIGH RISK)

**Recommended Actions**:
1. **HIGH PRIORITY**: Add automatic Git pre-commit hook (not manual)
2. Add CI check for unwrap/expect (catch if pre-commit skipped)
3. Add clippy deny for unwrap_used/expect_used
4. Document error handling patterns in SPR Guide
5. Add code review checklist item

---

## Part 3: GitHub Actions FMEA

### GitHub Actions Failure Mode #1: Workflow Only Runs on Main Branch

**Failure Mode**: CI workflow doesn't run on feature branches (claude/* branches)

**Effects**:
- Issues not detected until PR
- Late feedback loop
- Integration problems discovered late
- Wasted developer time fixing issues post-PR

**Causes**:
- Workflow configured for `branches: [main, master]` only
- No wildcard branch pattern
- Intentional to save CI minutes

**Current Controls**:
- None - this is current behavior

**Ratings**:
- **Severity**: 7 (Late feedback, wasted time)
- **Occurrence**: 10 (Very High - always happens on feature branches)
- **Detection**: 8 (Very Low - no indication until PR)
- **RPN**: **560** (CRITICAL RISK)

**Recommended Actions**:
1. **CRITICAL**: Update workflow to run on all branches
2. Add branch pattern: `branches: ['**']`
3. Or add pattern for feature branches: `branches: [main, master, 'claude/**']`
4. Consider: Run subset of checks on feature branches, full on main
5. Monitor CI cost/usage after change

---

### GitHub Actions Failure Mode #2: CI Cache Corruption

**Failure Mode**: GitHub Actions cache becomes corrupted or stale

**Effects**:
- Build failures due to stale dependencies
- Inconsistent build behavior
- CI slower (cache miss)
- Difficult to debug issues

**Causes**:
- Cache key collision
- Cargo.lock changes not reflected in cache
- Partial cache writes
- GitHub Actions cache service issues

**Current Controls**:
- Cache key includes `${{ hashFiles('**/Cargo.lock') }}`
- Restore-keys provide fallback
- actions/cache@v4 (latest version)

**Ratings**:
- **Severity**: 6 (Build issues, difficult to debug)
- **Occurrence**: 3 (Very Low - caching mostly reliable)
- **Detection**: 6 (Low - looks like random failures)
- **RPN**: **108** (HIGH RISK)

**Recommended Actions**:
1. Add cache verification step (validate cache contents)
2. Add manual cache invalidation workflow
3. Monitor cache hit rates
4. Add cache size limits
5. Document cache troubleshooting

---

### GitHub Actions Failure Mode #3: Cargo-make Not Installed

**Failure Mode**: CI fails because cargo-make installation fails or is missing

**Effects**:
- All builds fail
- CI completely blocked
- Cannot run any tasks

**Causes**:
- Crates.io unavailable
- cargo install fails
- Network issues
- Cargo-make yanked or unavailable

**Current Controls**:
- `cargo install cargo-make` in each workflow step
- No caching of cargo-make binary
- No version pinning

**Ratings**:
- **Severity**: 10 (CI completely blocked)
- **Occurrence**: 2 (Very Low - cargo install very reliable)
- **Detection**: 1 (High - fails immediately, clear error)
- **RPN**: **20** (LOW RISK)

**Recommended Actions**:
1. Cache cargo-make binary in ~/.cargo/bin/
2. Pin cargo-make version
3. Add fallback: pre-built cargo-make binary
4. Add health check for cargo-make installation

---

### GitHub Actions Failure Mode #4: Security Audit Failures

**Failure Mode**: Security audit fails due to vulnerabilities in dependencies

**Effects**:
- CI blocked (if audit is required)
- Security vulnerabilities unaddressed
- Difficult to update dependencies
- May need emergency patches

**Causes**:
- Dependency has newly disclosed vulnerability
- Transitive dependency vulnerability
- No patched version available yet

**Current Controls**:
- `cargo audit` runs in CI
- `continue-on-error: true` (audit failures are warnings)
- 15s timeout on audit

**Ratings**:
- **Severity**: 8 (Security risk, may need emergency fix)
- **Occurrence**: 4 (Low - vulnerabilities disclosed occasionally)
- **Detection**: 2 (High - cargo audit detects immediately)
- **RPN**: **64** (MEDIUM RISK)

**Recommended Actions**:
1. Add audit to PR checks (not just CI)
2. Add automated dependency update PRs (Dependabot/Renovate)
3. Monitor security advisories proactively
4. Document security response process
5. Add vulnerability severity threshold (block on high/critical)

---

### GitHub Actions Failure Mode #5: Workflow Timeout

**Failure Mode**: Entire CI workflow times out (GitHub Actions 6-hour limit)

**Effects**:
- CI never completes
- No feedback to developer
- Manual re-run required
- Blocks integration

**Causes**:
- Individual tasks hang (despite timeout)
- Too many tasks in sequence
- Slow CI runners
- Resource exhaustion

**Current Controls**:
- Individual task timeouts (5s-60s)
- Expected total CI time: ~120s (well under limit)
- Timeout-check task verifies timeout command

**Ratings**:
- **Severity**: 8 (CI blocked, no feedback)
- **Occurrence**: 1 (Very Low - total time well under limit)
- **Detection**: 3 (High - GitHub Actions timeout message)
- **RPN**: **24** (LOW RISK)

**Recommended Actions**:
1. Monitor total CI duration trends
2. Add alerts if CI duration increases significantly
3. Optimize slow tasks if duration grows
4. Document expected CI duration in workflow comments

---

### GitHub Actions Failure Mode #6: Matrix Build Failures

**Failure Mode**: CI doesn't test on multiple platforms (Linux only currently)

**Effects**:
- Platform-specific bugs not detected
- Broken builds on macOS/Windows
- User issues on non-Linux platforms
- Post-release bug fixes needed

**Causes**:
- No matrix strategy in workflow
- CI only configured for Linux
- Cost optimization (fewer runners)

**Current Controls**:
- None - CI only runs on ubuntu-latest

**Ratings**:
- **Severity**: 7 (Platform-specific bugs reach users)
- **Occurrence**: 5 (Medium - platform differences common)
- **Detection**: 9 (Very Low - only detected by users)
- **RPN**: **315** (CRITICAL RISK)

**Recommended Actions**:
1. **CRITICAL**: Add matrix strategy for multiple OSes
2. Test on: ubuntu-latest, macos-latest, windows-latest
3. Consider: Full tests on Linux, smoke tests on others (cost optimization)
4. Add platform-specific test documentation
5. Monitor cross-platform CI costs

---

## RPN Summary: Prioritized Action Plan

### Critical Risk (RPN > 200) - Immediate Action Required

1. **Test Coverage Enforcement (RPN: 336)**
   - Action: Add coverage enforcement to CI
   - Impact: Prevents bugs reaching production
   - Effort: Medium (2-4 hours)
   - Owner: Assign to next sprint

2. **Matrix Build Missing (RPN: 315)**
   - Action: Add multi-OS testing to CI
   - Impact: Catch platform-specific bugs
   - Effort: Low (1-2 hours)
   - Owner: Implement this week

3. **Workflow Doesn't Run on Feature Branches (RPN: 560)**
   - Action: Update workflow trigger to all branches
   - Impact: Earlier feedback, fewer integration issues
   - Effort: Minimal (15 minutes)
   - Owner: **Implement immediately**

### High Risk (RPN 100-200) - Action Required Soon

4. **Flaky Tests (RPN: 120)**
   - Action: Add test retry logic, flakiness detection
   - Impact: Reduce false negatives
   - Effort: Medium (3-5 hours)
   - Owner: Plan for next sprint

5. **Tests Pass Locally, Fail in CI (RPN: 105)**
   - Action: Add CI environment matrix, documentation
   - Impact: Reduce environment surprises
   - Effort: Medium (2-3 hours)
   - Owner: Plan for next sprint

6. **CI Cache Corruption (RPN: 108)**
   - Action: Add cache validation, manual invalidation
   - Impact: Reduce mysterious failures
   - Effort: Low (1-2 hours)
   - Owner: Plan for next month

7. **Unwrap/Expect in Production (RPN: 180)**
   - Action: Add automatic Git hooks, CI enforcement
   - Impact: Prevent production panics
   - Effort: Low (1-2 hours)
   - Owner: Implement this week

8. **Test Data Corruption (RPN: 168)**
   - Action: Audit tests, enforce isolation
   - Impact: Eliminate test order dependencies
   - Effort: High (5-8 hours)
   - Owner: Plan for next sprint

### Medium Risk (RPN 50-100) - Monitor and Plan

9. **Build Artifact Corruption (RPN: 90)**
10. **Security Audit Failures (RPN: 64)**
11. **Clippy Lint Failures (RPN: 60)**

### Low Risk (RPN < 50) - Accept or Defer

12. All others (RPN < 50)

---

## Immediate Action Items (Next 2 Hours)

### 1. Fix Critical: Workflow Doesn't Run on Feature Branches (15 min)

**File**: `.github/workflows/ci.yml`

```yaml
on:
  push:
    branches: ['**']  # Run on all branches
  pull_request:
    branches: [main, master]
  workflow_dispatch:
```

### 2. Add Matrix Build for Multi-OS Testing (30 min)

**File**: `.github/workflows/ci.yml`

```yaml
jobs:
  test:
    name: Unit Tests
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      # ... existing steps ...
```

### 3. Add Coverage Enforcement (60 min)

**File**: `.github/workflows/ci.yml`

Add new job:

```yaml
  coverage:
    name: Test Coverage
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Generate coverage report
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info

      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo llvm-cov --all-features --summary-only | grep -oP 'lines\.\.\.\.\.\. \K[0-9.]+')
          if (( $(echo "$COVERAGE < 80.0" | bc -l) )); then
            echo "❌ ERROR: Coverage $COVERAGE% is below threshold 80%"
            exit 1
          fi
          echo "✅ Coverage $COVERAGE% meets threshold"
```

---

## Monitoring and Continuous Improvement

### Metrics to Track

1. **Test Flakiness Rate**: % of tests that fail intermittently
2. **CI Duration**: Total time for CI pipeline
3. **Coverage**: Test coverage percentage
4. **Security Audit Findings**: Number of vulnerabilities found
5. **Timeout Occurrences**: Frequency of task timeouts
6. **Cache Hit Rate**: GitHub Actions cache effectiveness

### Review Cadence

- **Weekly**: Review CI metrics, identify trends
- **Monthly**: Full FMEA review, update RPN values
- **Quarterly**: Deep dive on high-risk items, validate mitigations

### Success Criteria

- All Critical RPN items resolved within 1 month
- All High RPN items resolved within 3 months
- Medium RPN items monitored and planned
- CI duration stays under 5 minutes
- Test coverage > 80%
- Zero flaky tests in CI

---

## References

- [SPR Guide](SPR_GUIDE.md) - Development standards
- [Code Review Checklist](CODE_REVIEW_CHECKLIST.md) - Review guidelines
- [DMAIC Problem Solving](../../.cursor/commands/dmaic-problem-solving.md) - Problem solving methodology
- [Poka-Yoke Design](../../.cursor/commands/poka-yoke-design.md) - Error prevention patterns
- [Root Cause Analysis](../../.cursor/commands/root-cause-analysis.md) - 5 Whys methodology
