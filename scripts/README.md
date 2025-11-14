# Scripts Directory

This directory contains FMEA (Failure Mode and Effects Analysis) mitigation scripts for chicago-tdd-tools.

## 🎯 80/20 Risk Reduction Scripts

These scripts implement the top 4 quick wins from FMEA analysis, reducing overall RPN by **53%** (48.4 → 22.6) in just **3.5 hours** of implementation.

---

## Scripts Overview

### 1. `install-hooks.sh` - Pre-Commit Hook Installer
**FMEA Mitigation**: Q6 (RPN 144 → 20, 86% reduction)
**Effort**: 1 hour
**ROI**: 124 RPN/hour

**Purpose**: Automates quality checks before every commit

**Usage**:
```bash
bash scripts/install-hooks.sh
```

**What it does**:
- Installs Git pre-commit hook
- Runs `cargo make pre-commit` automatically on every commit
- Checks: fmt, lint, test-unit, dead-code-check
- Expected duration: ~20 seconds per commit

**Benefits**:
- Catch issues before CI (faster feedback)
- Prevent bad code from being committed
- Consistent quality enforcement

---

### 2. `install-timeout-check.sh` - Timeout Command Validation
**FMEA Mitigation**: M2 (RPN 180 → 20, 89% reduction)
**Effort**: 30 minutes
**ROI**: 320 RPN/hour (HIGHEST ROI!)

**Purpose**: Ensures timeout command is available before running tasks

**Usage**:
```bash
bash scripts/install-timeout-check.sh
```

**What it does**:
- Checks if `timeout` command exists
- Provides OS-specific installation instructions
- Prevents hangs in cargo-make tasks

**Benefits**:
- Fail-fast if timeout unavailable
- Clear installation instructions
- Prevents infinite hangs

**Installation Help**:
- **Linux**: `sudo apt-get install coreutils`
- **macOS**: `brew install coreutils`
- **Windows**: Available in Git Bash

---

### 3. `setup-docker-validation.sh` - Docker Availability Check
**FMEA Mitigation**: I1 (RPN 210 → 30, 86% reduction)
**Effort**: 1 hour
**ROI**: 180 RPN/hour

**Purpose**: Ensures Docker is running before integration tests

**Usage**:
```bash
bash scripts/setup-docker-validation.sh
```

**What it does**:
- Checks if Docker command exists
- Verifies Docker daemon is running
- Provides installation/troubleshooting instructions

**Benefits**:
- Prevents silent test skipping
- Guarantees integration test coverage
- Clear error messages

**Integration Tests Require Docker**:
- Testcontainers tests
- Weaver integration tests
- OTEL observability tests

---

### 4. `verify-prerequisites.sh` - Comprehensive Tool Check
**FMEA Mitigation**: M1 (RPN 140 → 20, 86% reduction)
**Effort**: 1 hour
**ROI**: 120 RPN/hour

**Purpose**: Verifies all required and optional tools are installed

**Usage**:
```bash
bash scripts/verify-prerequisites.sh
```

**What it checks**:

**Required Tools**:
- ✅ `cargo` - Rust compiler
- ✅ `cargo-make` - Build task runner
- ✅ `timeout` - Timeout protection
- ✅ `cargo-nextest` - Test runner

**Optional Tools**:
- ⚠️ `docker` - Container runtime (for integration tests)
- ⚠️ `cargo-audit` - Security auditing
- ⚠️ `cargo-llvm-cov` - Code coverage
- ⚠️ `cargo-mutants` - Mutation testing

**Benefits**:
- One command to check everything
- Clear installation instructions
- Distinguishes required vs optional

---

## Quick Start

### New Contributors

```bash
# 1. Verify prerequisites
bash scripts/verify-prerequisites.sh

# 2. Install pre-commit hooks
bash scripts/install-hooks.sh

# 3. Run tests
cargo make test-unit
```

### CI/CD Integration

These scripts are automatically called by `Makefile.toml`:

```toml
# Verify prerequisites before any task
[tasks.verify-prerequisites]
script = "bash scripts/verify-prerequisites.sh"

# Docker check for integration tests
[tasks.docker-check]
script = "bash scripts/setup-docker-validation.sh"

# Timeout validation
[tasks.timeout-check]
command = "bash scripts/install-timeout-check.sh"
```

---

## FMEA Impact Summary

| Script | RPN Before | RPN After | Reduction | Effort | ROI |
|--------|------------|-----------|-----------|--------|-----|
| `install-timeout-check.sh` | 180 | 20 | 160 (89%) | 30 min | 320/hr |
| `setup-docker-validation.sh` | 210 | 30 | 180 (86%) | 1 hr | 180/hr |
| `install-hooks.sh` | 144 | 20 | 124 (86%) | 1 hr | 124/hr |
| `verify-prerequisites.sh` | 140 | 20 | 120 (86%) | 1 hr | 120/hr |
| **Total** | **674** | **90** | **584 (87%)** | **3.5 hr** | **193/hr** |

**Overall Project Impact**:
- Before: 6 critical risks (RPN ≥ 100)
- After: 2 critical risks
- **Reduction: 67% of critical risks eliminated**

---

## Troubleshooting

### Pre-Commit Hook Issues

**Problem**: Hook fails with "cargo-make not found"
**Solution**: Install cargo-make: `cargo install cargo-make`

**Problem**: Hook is too slow
**Solution**: This is expected (~20s). Run `cargo make pre-commit` manually to see progress

**Problem**: Want to bypass hook
**Solution**: `git commit --no-verify` (not recommended)

### Docker Issues

**Problem**: Docker command not found
**Solution**: Install Docker Desktop from https://www.docker.com/products/docker-desktop

**Problem**: Docker daemon not running
**Solution**:
- Linux: `sudo systemctl start docker`
- macOS/Windows: Open Docker Desktop app

**Problem**: Permission denied
**Solution**: Add user to docker group: `sudo usermod -aG docker $USER` (logout required)

### Timeout Issues

**Problem**: timeout command not found on macOS
**Solution**: `brew install coreutils` (may need to use `gtimeout`)

**Problem**: timeout command not found on Windows
**Solution**: Use Git Bash or WSL

---

## Development Workflow

### Recommended Setup (First Time)

```bash
# 1. Clone repository
git clone https://github.com/seanchatmangpt/chicago-tdd-tools.git
cd chicago-tdd-tools

# 2. Verify all prerequisites
bash scripts/verify-prerequisites.sh

# 3. Install missing tools (if any)
# Follow instructions from verify-prerequisites.sh

# 4. Install pre-commit hooks
bash scripts/install-hooks.sh

# 5. Run first test
cargo make test-unit
```

### Daily Workflow

```bash
# 1. Make changes
vim src/core/fixture.rs

# 2. Test locally (optional but recommended)
cargo make test-unit

# 3. Commit (pre-commit hook runs automatically)
git add .
git commit -m "feat: add feature"

# 4. Push (CI runs full validation)
git push
```

---

## Files Generated

### Git Hooks Created

- `.git/hooks/pre-commit` - Runs `cargo make pre-commit`
- `.git/hooks/pre-commit.backup.*` - Backup of previous hook (if exists)

### No System Files Modified

These scripts do not modify system files. They only:
- Create/update Git hooks in `.git/hooks/`
- Check for command existence
- Provide installation instructions

---

## Integration with Makefile.toml

All scripts are integrated into `Makefile.toml` tasks:

```bash
# Run prerequisite verification
cargo make verify-prerequisites

# Check Docker availability
cargo make docker-check

# Check timeout command
cargo make timeout-check

# Run all pre-commit checks
cargo make pre-commit
```

---

## Maintenance

### Updating Scripts

Scripts are version-controlled in `scripts/` directory:

```bash
# Make changes
vim scripts/verify-prerequisites.sh

# Test changes
bash scripts/verify-prerequisites.sh

# Commit
git add scripts/
git commit -m "chore: update prerequisite checks"
```

### Reinstalling Hooks

```bash
# Reinstall (backs up existing)
bash scripts/install-hooks.sh
```

---

## Related Documentation

- `/tmp/fmea_chicago_tdd_tools.md` - Full FMEA analysis
- `/tmp/fmea_quick_reference.txt` - Quick reference
- `/tmp/80_20_mitigation_plan.md` - Implementation plan
- `Makefile.toml` - Build tasks and integration
- `README.md` - Project overview

---

## Questions?

For issues or questions:
1. Check troubleshooting section above
2. Run `bash scripts/verify-prerequisites.sh` to diagnose
3. Open an issue on GitHub

---

**Version**: 1.0
**Last Updated**: 2025-11-14
**FMEA Document**: See `/tmp/fmea_chicago_tdd_tools.md`
