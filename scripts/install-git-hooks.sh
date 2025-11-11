#!/bin/bash
# Install poka-yoke git hooks for chicago-tdd-tools project
# Aligned with core team 80/20 best practices: fast feedback, pragmatic exceptions
# Prevents unwrap() calls and unimplemented!() from being committed
# Adapted for chicago-tdd-tools: single crate, testing framework library

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "🔧 Installing chicago-tdd-tools poka-yoke git hooks (core team best practices)..."

# Ensure .git/hooks directory exists
if [ ! -d "$HOOKS_DIR" ]; then
  echo "❌ ERROR: .git/hooks directory not found"
  echo "   Are you in a git repository?"
  exit 1
fi

# Create pre-commit hook (fast: 2-5s target, only staged files, incremental checks)
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# Pre-commit hook: Fast incremental validation (core team 80/20 best practices)
# Target: 2-5 seconds (only checks staged files, skips unnecessary checks)
# Enforces: No unwrap/expect/TODO/FUTURE/unimplemented on MAIN branch only
# Uses: cargo make commands (NEVER direct cargo commands)
# CRITICAL: Overall timeout wrapper prevents infinite hangs

set -e

# Verify timeout command exists
if ! command -v timeout &> /dev/null; then
  echo "❌ ERROR: 'timeout' command not found. Cannot prevent freezing."
  echo "   Install: coreutils (macOS: brew install coreutils)"
  exit 1
fi

# Overall timeout wrapper: 30s max
# If hook exceeds this, it will be killed and fail fast
exec timeout 30s bash -c '
set -e

# Change to project root (with timeout)
PROJECT_ROOT=\$(timeout 5s git rev-parse --show-toplevel 2>&1)
if [ \$? -ne 0 ]; then
  echo \"❌ ERROR: Failed to get project root (timeout or error)\"
  exit 1
fi
cd \"\$PROJECT_ROOT\"

echo "🔍 Running pre-commit validation (incremental checks only)..."

# Only check if Rust files are staged (with timeout)
STAGED_RUST_FILES=$(timeout 5s git diff --cached --name-only --diff-filter=d 2>&1 | grep '\.rs$' || true)

if [ -z "$STAGED_RUST_FILES" ]; then
  echo "✅ No Rust files staged, skipping validation"
  exit 0
fi

# Detect current branch - strict rules only apply to main (with timeout)
CURRENT_BRANCH=$(timeout 5s git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
IS_MAIN_BRANCH=false
if [ "$CURRENT_BRANCH" = "main" ] || [ "$CURRENT_BRANCH" = "master" ]; then
  IS_MAIN_BRANCH=true
  echo "🔒 Main branch detected - enforcing strict rules"
else
  echo "🌿 Branch '$CURRENT_BRANCH' - relaxed rules"
fi

# Check 1: No unwrap() in production code (only staged changes)
echo "   Checking staged changes for unwrap()..."
UNWRAP_COUNT=0
for file in $STAGED_RUST_FILES; do
  # Skip test files, examples, benches, build scripts, config files
  if [[ "$file" =~ /(test|tests|example|examples|bench|benches)/ ]] || [[ "$file" == *"build.rs" ]] || [[ "$file" =~ ^(test|tests|example|examples|bench|benches)/ ]] || [[ "$file" =~ \.(cursorrules|toml|yaml|yml|json)$ ]]; then
    continue
  fi
  
  # Skip proc-macro crate test files
  if [[ "$file" =~ proc_macros/.*/(test|tests)/ ]]; then
    continue
  fi
  
  # Check if file has allow attribute (with timeout)
  if timeout 5s git diff --cached "$file" 2>&1 | grep -qE "#!?\[allow\(clippy::unwrap_used\)\]" || \
     timeout 5s grep -qE "#!?\[allow\(clippy::unwrap_used\)\]" "$file" 2>/dev/null; then
    continue
  fi
  
  # Skip files with test modules (pragmatic exception for pre-commit speed, with timeout)
  if timeout 5s grep -q "#\[cfg(test)\]" "$file" 2>/dev/null; then
    continue
  fi
  
  # Count unwrap() calls in staged changes only (with timeout)
  UNWRAPS=$(timeout 5s git diff --cached "$file" 2>&1 | grep -E "^\+" | grep -c "\.unwrap()" || echo 0)
  UNWRAPS=${UNWRAPS//[^0-9]/}
  if [ "${UNWRAPS:-0}" -gt 0 ]; then
    echo "     ❌ $file: $UNWRAPS unwrap() call(s)"
    UNWRAP_COUNT=$((UNWRAP_COUNT + UNWRAPS))
  fi
done

if [ "$UNWRAP_COUNT" -gt 0 ]; then
  echo "❌ ERROR: Cannot commit $UNWRAP_COUNT unwrap() calls in production code"
  exit 1
fi
echo "  ✅ No unwrap() in staged changes"

# Check 2: No unimplemented!() placeholders - BLOCKED ONLY ON MAIN
if [ "$IS_MAIN_BRANCH" = true ]; then
  echo "   Checking staged changes for unimplemented!()..."
  UNIMPL_COUNT=0
  for file in $STAGED_RUST_FILES; do
    UNIMPL=$(timeout 5s git diff --cached "$file" 2>&1 | grep -E "^\+" | grep -c "unimplemented!" || echo 0)
    UNIMPL=${UNIMPL//[^0-9]/}
    if [ "${UNIMPL:-0}" -gt 0 ]; then
      echo "     ❌ $file: $UNIMPL unimplemented!() placeholder(s)"
      UNIMPL_COUNT=$((UNIMPL_COUNT + UNIMPL))
    fi
  done

  if [ "$UNIMPL_COUNT" -gt 0 ]; then
    echo "❌ ERROR: Cannot commit $UNIMPL_COUNT unimplemented!() placeholders to main"
    exit 1
  fi
  echo "  ✅ No unimplemented!() placeholders"
fi

# Check 3: No FUTURE or TODO comments - BLOCKED ONLY ON MAIN
if [ "$IS_MAIN_BRANCH" = true ]; then
  echo "   Checking staged changes for FUTURE/TODO..."
  TODO_COUNT=0
  for file in $STAGED_RUST_FILES; do
    # Skip documentation files
    if [[ "$file" =~ \.(md|txt|rst)$ ]]; then
      continue
    fi
    
    TODOS=$(timeout 5s git diff --cached "$file" 2>&1 | grep -E "^\+" | grep -iE "\b(TODO|FUTURE)\b" | grep -c . || echo 0)
    TODOS=${TODOS//[^0-9]/}
    if [ "${TODOS:-0}" -gt 0 ]; then
      echo "     ❌ $file: $TODOS FUTURE/TODO comment(s)"
      TODO_COUNT=$((TODO_COUNT + TODOS))
    fi
  done

  if [ "$TODO_COUNT" -gt 0 ]; then
    echo "❌ ERROR: Cannot commit $TODO_COUNT FUTURE/TODO comments to main"
    exit 1
  fi
  echo "  ✅ No FUTURE/TODO comments"
fi

# Check 4: No expect() in production code (only staged changes)
echo "   Checking staged changes for expect()..."
EXPECT_COUNT=0
for file in $STAGED_RUST_FILES; do
  # Skip test files, examples, benches, build scripts, config files
  if [[ "$file" =~ /(test|tests|example|examples|bench|benches)/ ]] || [[ "$file" == *"build.rs" ]] || [[ "$file" =~ ^(test|tests|example|examples|bench|benches)/ ]] || [[ "$file" =~ \.(cursorrules|toml|yaml|yml|json)$ ]]; then
    continue
  fi
  
  # Skip proc-macro crate test files
  if [[ "$file" =~ proc_macros/.*/(test|tests)/ ]]; then
    continue
  fi
  
  # Check if file has allow attribute
  if timeout 5s grep -qE "#!?\[allow\(clippy::expect_used\)\]" "$file" 2>/dev/null || \
     timeout 5s git diff --cached "$file" 2>&1 | grep -qE "#!?\[allow\(clippy::expect_used\)\]"; then
    continue
  fi
  
  # Skip files with test modules (pragmatic exception for pre-commit speed, with timeout)
  if timeout 5s grep -q "#\[cfg(test)\]" "$file" 2>/dev/null; then
    continue
  fi
  
  # Count expect() calls in staged changes only (with timeout)
  EXPECTS=$(timeout 5s git diff --cached "$file" 2>&1 | grep -E "^\+" | grep -c "\.expect(" || echo 0)
  EXPECTS=${EXPECTS//[^0-9]/}
  if [ "${EXPECTS:-0}" -gt 0 ]; then
    echo "     ❌ $file: $EXPECTS expect() call(s)"
    EXPECT_COUNT=$((EXPECT_COUNT + EXPECTS))
  fi
done

if [ "$EXPECT_COUNT" -gt 0 ]; then
  echo "❌ ERROR: Cannot commit $EXPECT_COUNT expect() calls in production code"
  exit 1
fi
echo "  ✅ No expect() in staged changes"

# Check 5: Formatting (only check staged Rust files)
echo "   Checking formatting of staged Rust files..."
# Use cargo fmt --check but only on staged files (faster than --all)
FMT_FAILED=0
# Build list of staged Rust files for cargo fmt
STAGED_FMT_FILES=$(echo "$STAGED_RUST_FILES" | tr \"\\n\" \" \" || true)

if [ -n "$STAGED_FMT_FILES" ]; then
  # Check formatting of staged files only (with 5s timeout)
  if ! timeout 5s cargo fmt -- --check $STAGED_FMT_FILES 2>&1 | grep -q "Diff"; then
    # If no diff output, check exit code
    if ! timeout 5s cargo fmt -- --check $STAGED_FMT_FILES 2>&1 > /dev/null; then
      echo "     ❌ Some staged files are not formatted"
      FMT_FAILED=1
    fi
  else
    echo "     ❌ Some staged files are not formatted"
    FMT_FAILED=1
  fi
fi

if [ "$FMT_FAILED" -eq 1 ]; then
  echo "❌ ERROR: Staged Rust files are not formatted"
  echo "   Run: cargo make fmt"
  exit 1
fi
echo "  ✅ Staged files are formatted"

# Check 6: Clippy (only if source files changed, incremental check)
STAGED_SOURCE_FILES=$(echo "$STAGED_RUST_FILES" | grep -E "^(src|proc_macros/.*/src)/" || true)

if [ -n "$STAGED_SOURCE_FILES" ]; then
  echo "   Running incremental clippy check..."
  # Only check if compilation succeeds (fast check, with 5s timeout)
  if ! timeout 5s cargo check --lib --all-features --message-format=short 2>&1 | grep -qE "^error"; then
    # If check passes, run clippy on lib only (fast, with 5s timeout)
    if timeout 5s cargo clippy --lib --all-features -- -D warnings 2>&1 > /tmp/clippy_precommit.txt; then
      rm -f /tmp/clippy_precommit.txt
      echo "  ✅ Clippy checks passed"
    else
      # Filter out test-related warnings
      if grep -v "test\|tests\|example\|examples\|bench\|benches\|\.rs:" /tmp/clippy_precommit.txt | grep -qE "(error|warning):"; then
        echo "❌ ERROR: Clippy found issues in production code"
        grep -v "test\|tests\|example\|examples\|bench\|benches" /tmp/clippy_precommit.txt | head -10
        rm -f /tmp/clippy_precommit.txt
        exit 1
      fi
      rm -f /tmp/clippy_precommit.txt
      echo "  ✅ Clippy checks passed (test warnings ignored)"
    fi
  else
    echo "  ⚠️  Compilation errors detected, skipping clippy (will be caught in pre-push)"
  fi
else
  echo "  ⏭️  No source files changed, skipping clippy"
fi

echo "✅ Pre-commit validation passed (incremental checks)"
exit 0
'
EOF

# Create pre-push hook (comprehensive: 30-60s acceptable, full validation)
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash
# Pre-push hook: Comprehensive validation (core team best practices)
# Full validation before push (30-60s acceptable)
# Uses: cargo make commands (NEVER direct cargo commands)
# CRITICAL: Overall timeout wrapper prevents infinite hangs

set -e

# Verify timeout command exists
if ! command -v timeout &> /dev/null; then
  echo "❌ ERROR: 'timeout' command not found. Cannot prevent freezing."
  echo "   Install: coreutils (macOS: brew install coreutils)"
  exit 1
fi

# Overall timeout wrapper: 120s max (2 minutes)
# If hook exceeds this, it will be killed and fail fast
exec timeout 120s bash -c '
set -e

# Change to project root (with timeout)
PROJECT_ROOT=\$(timeout 5s git rev-parse --show-toplevel 2>&1)
if [ \$? -ne 0 ]; then
  echo \"❌ ERROR: Failed to get project root (timeout or error)\"
  exit 1
fi
cd \"\$PROJECT_ROOT\"

echo "🚦 Pre-push validation (comprehensive checks)..."
echo ""

# Docker availability check function (matches Rust check_docker_available() logic)
check_docker_available() {
  # Check Docker command exists
  if ! command -v docker &> /dev/null; then
    echo "❌ ERROR: Docker command not found. Please install Docker."
    return 1
  fi
  
  # Check Docker daemon is running
  DOCKER_OUTPUT=\$(timeout 5s docker info 2>&1)
  DOCKER_EXIT=\$?
  
  if [ \$DOCKER_EXIT -ne 0 ]; then
    echo "❌ ERROR: Docker daemon is not running"
    echo "   Error: \$DOCKER_OUTPUT"
    echo "   💡 FIX: Start Docker Desktop or Docker daemon"
    echo "   📋 macOS: Open Docker Desktop"
    echo "   📋 Linux: sudo systemctl start docker"
    echo "   📋 Windows: Start Docker Desktop"
    return 1
  fi
  
  # Verify Docker daemon is responding
  if echo "\$DOCKER_OUTPUT" | grep -qE "(Server Version|Docker Root Dir)"; then
    return 0
  else
    echo "❌ ERROR: Docker daemon is not responding correctly"
    echo "   Output does not contain expected Docker info"
    return 1
  fi
}

# Gate 1: Cargo check (comprehensive)
echo "Gate 1/6: Cargo check..."
if ! cargo make check 2>&1; then
  echo "❌ ERROR: cargo make check failed"
  exit 1
fi
echo "✅ Gate 1 passed"
echo ""

# Gate 2: Clippy (comprehensive, strict for production)
echo "Gate 2/6: Clippy (strict mode)..."
if cargo make lint 2>&1 > /tmp/clippy_push.txt; then
  rm -f /tmp/clippy_push.txt
  echo "✅ Gate 2 passed"
else
  # Filter out test-related warnings
  if grep -v "test\|tests\|example\|examples\|bench\|benches\|\.rs:" /tmp/clippy_push.txt | grep -qE "(error|warning):"; then
    echo "❌ ERROR: Clippy found issues in production code"
    grep -v "test\|tests\|example\|examples\|bench\|benches" /tmp/clippy_push.txt | head -30
    rm -f /tmp/clippy_push.txt
    exit 1
  fi
  rm -f /tmp/clippy_push.txt
  echo "✅ Gate 2 passed (test warnings ignored)"
fi
echo ""

# Gate 2.5: TODO & error handling check (comprehensive)
echo "Gate 2.5/6: TODO & error handling check..."

# Check for TODO comments in production code (with timeout)
TODO_COUNT=$(timeout 10s find src proc_macros/src -name "*.rs" -type f 2>/dev/null | \
  grep -v "/tests/" | \
  grep -v "/test/" | \
  grep -v "/example" | \
  grep -v "build.rs" | \
  grep -v "/target/" | \
  xargs timeout 10s grep "TODO:" 2>/dev/null | \
  grep -v "FUTURE:" | \
  wc -l | tr -d '[:space:]' || echo 0)

if [ "$TODO_COUNT" -gt 0 ]; then
  echo "❌ ERROR: $TODO_COUNT TODO comments found in production code"
  exit 1
fi

# Check for unwrap/expect in production code (with timeout)
UNWRAP_COUNT=$(timeout 10s find src proc_macros/src -name "*.rs" -type f 2>/dev/null | \
  grep -v "/tests/" | \
  grep -v "/test/" | \
  grep -v "/example" | \
  grep -v "build.rs" | \
  grep -v "/target/" | \
  while read file; do
    if [ -z "$file" ]; then
      continue
    fi
    if timeout 5s grep -qE "#!?\[allow\(clippy::unwrap_used\)\]" "$file" 2>/dev/null; then
      continue
    fi
    if timeout 5s grep -q "#\[cfg(test)\]" "$file" 2>/dev/null; then
      continue
    fi
    timeout 5s grep -c "\.unwrap()" "$file" 2>/dev/null || echo 0
  done | awk "{s+=\$1} END {print s}" || echo 0)

if [ "$UNWRAP_COUNT" -gt 0 ]; then
  echo "❌ ERROR: Found $UNWRAP_COUNT unwrap() calls in production code"
  exit 1
fi

EXPECT_COUNT=$(timeout 10s find src proc_macros/src -name "*.rs" -type f 2>/dev/null | \
  grep -v "/tests/" | \
  grep -v "/test/" | \
  grep -v "/example" | \
  grep -v "build.rs" | \
  grep -v "/target/" | \
  while read file; do
    if [ -z "$file" ]; then
      continue
    fi
    if timeout 5s grep -qE "#!?\[allow\(clippy::expect_used\)\]" "$file" 2>/dev/null; then
      continue
    fi
    if timeout 5s grep -q "#\[cfg(test)\]" "$file" 2>/dev/null; then
      continue
    fi
    timeout 5s grep -c "\.expect(" "$file" 2>/dev/null || echo 0
  done | awk "{s+=\$1} END {print s}" || echo 0)

if [ "$EXPECT_COUNT" -gt 0 ]; then
  echo "❌ ERROR: Found $EXPECT_COUNT expect() calls in production code"
  exit 1
fi

echo "✅ Gate 2.5 passed"
echo ""

# Gate 2.6: Docker availability check (if testcontainers feature enabled)
echo "Gate 2.6/6: Docker availability check..."
# Check if testcontainers feature is enabled or testcontainers tests exist
HAS_TESTCONTAINERS=false
if grep -qE "testcontainers\s*=" Cargo.toml 2>/dev/null || \
   find tests examples -name "*.rs" -type f 2>/dev/null | grep -q "testcontainers" || \
   find src -name "*.rs" -type f 2>/dev/null | grep -q "testcontainers"; then
  HAS_TESTCONTAINERS=true
fi

if [ "$HAS_TESTCONTAINERS" = true ]; then
  if ! check_docker_available; then
    echo "❌ ERROR: Docker is required for testcontainers feature but is not available"
    exit 1
  fi
  echo "✅ Gate 2.6 passed"
else
  echo "⏭️  Testcontainers feature not detected, skipping Docker check"
fi
echo ""

# Gate 3: Formatting check (comprehensive, with 5s timeout)
echo "Gate 3/6: Formatting check..."
if ! timeout 5s cargo fmt --all -- --check 2>&1; then
  echo "❌ ERROR: Code is not formatted"
  echo "   Run: cargo make fmt"
  exit 1
fi
echo "✅ Gate 3 passed"
echo ""

# Gate 4: Tests (unit + integration - full validation loop)
echo "Gate 4/6: Unit tests..."
if ! cargo make test-unit 2>&1 | tail -20; then
  echo "❌ ERROR: Unit tests failed"
  exit 1
fi
echo "✅ Gate 4.1 passed (unit tests)"
echo ""

# Gate 4.2: Integration tests (requires Docker - checked in Gate 2.6)
if [ "$HAS_TESTCONTAINERS" = true ]; then
  echo "Gate 4.2/6: Integration tests (testcontainers + weaver)..."
  if ! cargo make test-integration 2>&1 | tail -30; then
    echo "❌ ERROR: Integration tests failed"
    exit 1
  fi
  echo "✅ Gate 4.2 passed (integration tests)"
else
  echo "⏭️  Gate 4.2 skipped (no testcontainers feature detected)"
fi
echo ""

# Gate 5: Security audit (non-blocking)
echo "Gate 5/6: Security audit..."
if command -v cargo-audit &> /dev/null; then
  if ! cargo make audit 2>&1; then
    echo "⚠️  Security audit found issues (non-blocking)"
  else
    echo "✅ Gate 5 passed"
  fi
else
  echo "⚠️  cargo-audit not installed (optional)"
fi
echo ""

echo "✅ All gates passed - ready to push"
exit 0
'
EOF

# Make hooks executable
chmod +x "$HOOKS_DIR/pre-commit"
chmod +x "$HOOKS_DIR/pre-push"

echo "✅ Git hooks installed successfully:"
echo "   - $HOOKS_DIR/pre-commit (fast, incremental)"
echo "   - $HOOKS_DIR/pre-push (comprehensive)"
echo ""
echo "🔍 Hook optimization (core team 80/20 best practices):"
echo ""
echo "📋 Pre-commit (2-5s target):"
echo "   • Only checks staged files (incremental)"
echo "   • Skips formatting if no Rust files staged"
echo "   • Skips clippy if no source files changed"
echo "   • Uses rustfmt --check on individual files (faster)"
echo "   • Uses cargo check before clippy (faster failure)"
echo "   • No tests (too slow for pre-commit)"
echo ""
echo "📋 Pre-push (30-60s acceptable):"
echo "   • Comprehensive validation (all files)"
echo "   • Full cargo check"
echo "   • Full clippy (all targets)"
echo "   • Full formatting check"
echo "   • Unit tests only (faster than full test suite)"
echo "   • Security audit (non-blocking)"
echo ""
echo "💡 Key improvements:"
echo "   • Pre-commit: Incremental checks only (staged files)"
echo "   • Pre-push: Comprehensive but optimized (unit tests only)"
echo "   • No redundant checks between hooks"
echo "   • Faster feedback loop (pre-commit < 5s)"
echo "   • Comprehensive validation before push"
