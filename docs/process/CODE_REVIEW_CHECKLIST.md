# Code Review Checklist - Chicago TDD Tools

## Root Cause Prevention: Clippy Compliance

**Root Cause**: Missing CI/CD pipeline allowed 48 clippy errors to accumulate in codebase.

**Prevention**: CI/CD pipeline now enforces clippy checks automatically. Code reviews must verify clippy compliance.

## Pre-Review Checklist

- [ ] **Clippy passes**: Run `cargo make lint` - must pass with exit code 0
- [ ] **Tests pass**: Run `cargo make test` - all tests must pass
- [ ] **Format check**: Run `cargo make fmt` - code must be formatted
- [ ] **No unwrap/expect**: Check for `unwrap()`/`expect()` in production code (use `#[allow]` with justification if needed)

## Code Quality Checklist

- [ ] **Clippy compliance**: No clippy warnings/errors (CI will fail if present)
- [ ] **Error handling**: All error paths handled (no silent failures)
- [ ] **Type safety**: Types encode invariants (use newtypes where appropriate)
- [ ] **Documentation**: Public APIs have doc comments with `# Errors` sections
- [ ] **Tests**: New code has tests (AAA pattern, behavior verification)

## Clippy Allow Patterns

When `#[allow(clippy::...)]` is used, verify:
- [ ] Justification comment explains why allow is necessary
- [ ] Allow is scoped to smallest possible scope
- [ ] Pattern matches project standards (see SPR Guide)

**Common allows**:
- `expect_used`: Mutex operations (with "Mutex should never be poisoned" justification)
- `panic`: Test helpers (with "Test helper - panic is appropriate" justification)
- `unwrap_used`: Test code only (with "Test code - unwrap is acceptable" justification)

## CI/CD Integration

- [ ] **CI will enforce**: All checks run automatically on commit/PR
- [ ] **No bypassing**: Cannot merge if clippy fails
- [ ] **Fast feedback**: Run `cargo make pre-commit` before pushing

## Root Cause Prevention

**Remember**: The root cause was missing CI/CD enforcement. Now that CI/CD is in place:
- Clippy errors cannot accumulate (CI fails on warnings)
- Code quality is enforced automatically
- Prevention is built into the process

**If clippy fails in CI**: Fix errors immediately, don't bypass checks.

