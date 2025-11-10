# Service Level Agreement (SLA) Reference

This document provides a comprehensive reference for all timeout SLAs in the chicago-tdd-tools project.

## Overview

All tasks have explicit timeout SLAs to prevent hangs and ensure fast feedback. Timeouts are enforced at multiple layers:
1. **Task-level**: Unix `timeout` command wraps each task
2. **Test-level**: ntest crate and tokio::time::timeout for individual tests
3. **Test-runner**: cargo-nextest profiles with timeout configuration
4. **Git hooks**: Individual checks wrapped with timeouts

## Test SLAs

### Unit Tests
- **SLA**: 1s per test execution
- **Actual**: ~0.05s (well under SLA)
- **Profile**: `default` in `.config/nextest.toml`
- **Tasks**: `test`, `test-unit`
- **Note**: Excludes testcontainers integration tests (too slow, require Docker)

### Integration Tests
- **SLA**: 30s per test execution
- **Profile**: `integration` in `.config/nextest.toml`
- **Tasks**: `test-integration`
- **Note**: Requires Docker, only run when needed

## Build SLAs

### Check
- **SLA**: 5s
- **Actual**: ~0.8s
- **Task**: `check`

### Build (Debug)
- **SLA**: 5s
- **Task**: `build`

### Build (Release)
- **SLA**: 30s (release builds are slower)
- **Task**: `build-release`

### Clean
- **SLA**: 5s
- **Task**: `clean`

## Code Quality SLAs

### Formatting
- **SLA**: 5s
- **Actual**: ~0.6s
- **Task**: `fmt`
- **Git Hook**: Pre-commit and pre-push (5s timeout)

### Linting (Clippy)
- **SLA**: 5s
- **Actual**: ~1.2s
- **Task**: `lint`
- **Git Hook**: Pre-commit (5s timeout for cargo check and clippy)

## Coverage SLAs

**Note**: Coverage tasks are manual only, NOT part of commit/push verification.

### Coverage (cargo-llvm-cov)
- **SLA**: 30s
- **Task**: `coverage`

### Coverage Report (HTML)
- **SLA**: 30s
- **Task**: `coverage-report`

### Coverage (cargo-tarpaulin)
- **SLA**: 30s
- **Task**: `coverage-tarpaulin`

## Security SLAs

### Audit
- **SLA**: 15s (network operations can take longer)
- **Task**: `audit`
- **Note**: Fetches advisory database from network

### Audit Outdated
- **SLA**: 15s (network operations can take longer)
- **Task**: `audit-outdated`
- **Note**: Queries crates.io index

## Documentation SLAs

### Docs (with open)
- **SLA**: 20s (documentation generation can take longer)
- **Task**: `docs`

### Docs Build
- **SLA**: 20s
- **Task**: `docs-build`

## Workflow SLAs

### Pre-Commit
- **Expected Total**: ~10s
  - fmt: 5s
  - lint: 5s
  - test-unit: 1s
- **Task**: `pre-commit`
- **Git Hook**: Pre-commit hook (individual checks have 5s timeouts)

### Pre-Push
- **Expected Total**: ~60s
  - check: 5s
  - lint: 5s
  - TODO/error handling: <1s
  - fmt: 5s
  - test-unit: 1s
  - audit: 15s
- **Git Hook**: Pre-push hook (individual checks use cargo make with timeouts)

### CI Pipeline
- **Expected Total**: ~120s
  - fmt: 5s
  - lint: 5s
  - test-unit: 1s
  - audit-all: 30s (audit + audit-outdated)
- **Task**: `ci`

### Release
- **Expected Total**: ~180s
  - ci: 120s
  - docs-build: 20s
- **Task**: `release`

### Development Workflow
- **Expected Total**: ~15s
  - check: 5s
  - fmt: 5s
  - test-unit: 1s
- **Task**: `dev`

### Full Validation
- **Expected Total**: ~20s
  - build: 5s
  - test: 1s
  - lint: 5s
- **Task**: `all`

## Git Hook SLAs

### Pre-Commit Hook
- **Target**: 2-5s (incremental checks only)
- **Individual Checks**:
  - unwrap/expect/TODO checks: <1s (grep operations)
  - formatting: 5s timeout
  - clippy: 5s timeout (cargo check + clippy)
- **Note**: Only checks staged files, skips unnecessary checks

### Pre-Push Hook
- **Target**: 30-60s (comprehensive validation)
- **Gates**:
  - Gate 1: Cargo check (5s via cargo make)
  - Gate 2: Clippy (5s via cargo make)
  - Gate 2.5: TODO/error handling (<1s)
  - Gate 3: Formatting (5s timeout)
  - Gate 4: Unit tests (1s via cargo make)
  - Gate 5: Security audit (15s via cargo make)

## Rationale

### Why 1s for Unit Tests?
- Fast feedback loop for developers
- Forces test optimization
- Prevents slow tests from accumulating
- Actual execution is ~0.05s, well under SLA

### Why 30s for Integration Tests?
- Docker container startup takes 5-10s
- Network operations can take time
- Integration tests are excluded from normal iteration
- Only run when needed (not in pre-commit/pre-push)

### Why 5s for General Tasks?
- Fast enough for normal operations
- Actual times are 0.6-1.2s, well under SLA
- Prevents hangs without being too restrictive

### Why 15s for Security Audit?
- Network operations (fetching advisories, querying crates.io)
- Can take 5-10s on slow networks
- Non-blocking in pre-push (warning only)

### Why 20s for Documentation?
- Documentation generation can take 10-20s
- Not part of normal iteration
- Only run for releases

### Why 30s for Coverage?
- Coverage analysis is computationally expensive
- Can take 10-30s for larger projects
- Manual task only, not part of commit/push verification

## Monitoring

To verify SLAs are being met:
```bash
# Check actual execution times
time cargo make test-unit
time cargo make check
time cargo make lint

# Check git hook times
time .git/hooks/pre-commit
time .git/hooks/pre-push
```

## Future Enhancements

- Add timing output to tasks to track SLA compliance
- Add SLA violation alerts
- Consider per-test timeout configuration
- Add timeout statistics and reporting
