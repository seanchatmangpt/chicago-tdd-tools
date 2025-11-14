# SLA Reference - SPR

Comprehensive reference for all timeout SLAs in chicago-tdd-tools project.

## Overview

All tasks have explicit timeout SLAs to prevent hangs and ensure fast feedback. Timeouts are enforced at multiple layers: task-level (Unix `timeout` command), test-level (ntest crate, tokio::time::timeout), and test-runner (cargo-nextest profiles).

## Test SLAs

**Unit Tests**: 1s per test execution. Actual: ~0.05s (well under SLA). Profile: `default` in `.config/nextest.toml`. Tasks: `test`, `test-unit`. Note: Excludes testcontainers integration tests (too slow, require Docker).

**Integration Tests**: 30s per test execution. Profile: `integration` in `.config/nextest.toml`. Tasks: `test-integration`. Note: Requires Docker, only run when needed.

## Build SLAs

**Check**: 5s SLA, ~0.8s actual. Task: `check`. **Build (Debug)**: 5s SLA. Task: `build`. **Build (Release)**: 30s SLA (release builds slower). Task: `build-release`. **Clean**: 5s SLA. Task: `clean`.

## Code Quality SLAs

**Formatting**: 5s SLA, ~0.6s actual. Task: `fmt`. Applied via cargo-make tasks and CI pipelines.

**Linting (Clippy)**: 5s SLA, ~1.2s actual. Task: `lint`. Executed by `cargo make pre-commit`, `cargo make ci`, and CI workflows.

## Coverage SLAs

**Note**: Coverage tasks are manual only, NOT part of commit/push verification.

**Coverage (cargo-llvm-cov)**: 30s SLA. Task: `coverage`. **Coverage Report (HTML)**: 30s SLA. Task: `coverage-report`. **Coverage (cargo-tarpaulin)**: 30s SLA. Task: `coverage-tarpaulin`.

## Security SLAs

**Audit**: 15s SLA (network operations can take longer). Task: `audit`. Note: Fetches advisory database from network.

**Audit Outdated**: 15s SLA (network operations can take longer). Task: `audit-outdated`. Note: Queries crates.io index.

## Documentation SLAs

**Docs (with open)**: 20s SLA (documentation generation can take longer). Task: `docs`. **Docs Build**: 20s SLA. Task: `docs-build`.

## Workflow SLAs

**Pre-Commit**: ~10s expected total (fmt: 5s, lint: 5s, test-unit: 1s). Task: `pre-commit`.

**Pre-Push**: ~60s expected total (check: 5s, lint: 5s, TODO/error handling: <1s, fmt: 5s, test-unit: 1s, audit: 15s). Recommended task sequence: `pre-commit`, `check`, `lint`, `audit`.

**CI Pipeline**: ~120s expected total (fmt: 5s, lint: 5s, test-unit: 1s, audit-all: 30s). Task: `ci`.

**Release**: ~180s expected total (ci: 120s, docs-build: 20s). Task: `release`.

**Development Workflow**: ~15s expected total (check: 5s, fmt: 5s, test-unit: 1s). Task: `dev`.

**Full Validation**: ~20s expected total (build: 5s, test: 1s, lint: 5s). Task: `all`.

## Git Hook SLAs

_Removed_: Git hook installer has been retired; developers run cargo-make tasks directly (`pre-commit`, `check`, `lint`, `audit`, etc.).

## Rationale

**1s for Unit Tests**: Fast feedback loop, forces test optimization, prevents slow tests from accumulating, actual execution ~0.05s (well under SLA).

**30s for Integration Tests**: Docker container startup takes 5-10s, network operations can take time, integration tests excluded from normal iteration, only run when needed.

**5s for General Tasks**: Fast enough for normal operations, actual times 0.6-1.2s (well under SLA), prevents hangs without being too restrictive.

**15s for Security Audit**: Network operations (fetching advisories, querying crates.io), can take 5-10s on slow networks, non-blocking in pre-push (warning only).

**20s for Documentation**: Documentation generation can take 10-20s, not part of normal iteration, only run for releases.

**30s for Coverage**: Coverage analysis computationally expensive, can take 10-30s for larger projects, manual task only, not part of commit/push verification.

## Monitoring

**Verify SLAs**: `time cargo make test-unit`, `time cargo make check`, `time cargo make lint`, `time cargo make pre-commit`, `time cargo make ci`.

## Summary

**Key Associations**: Unit Tests = 1s = Fast Feedback. Integration Tests = 30s = Docker Operations. Build Tasks = 5s = Normal Operations. Security Audit = 15s = Network Operations. Documentation = 20s = Generation Time. Coverage = 30s = Analysis Time.

**Pattern**: All tasks have explicit timeout SLAs. Timeouts enforced at multiple layers (task-level, test-level, runner-level). Actual times well under SLA. Timeouts prevent hangs and ensure fast feedback.
