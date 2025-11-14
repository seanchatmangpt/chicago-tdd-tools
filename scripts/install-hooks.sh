#!/bin/bash
# Pre-Commit Hook Installation Script
# FMEA Mitigation: Q6 (RPN 144 → 20)
# Automates quality checks before every commit

set -e

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo ".")
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "🔧 Installing pre-commit hooks for chicago-tdd-tools..."
echo ""

# Check if we're in a git repository
if [ ! -d "$REPO_ROOT/.git" ]; then
    echo "❌ Not in a git repository"
    echo "   Run this script from within the chicago-tdd-tools repository"
    exit 1
fi

echo "📂 Repository root: $REPO_ROOT"
echo "📂 Hooks directory: $HOOKS_DIR"
echo ""

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Backup existing pre-commit hook if it exists
if [ -f "$HOOKS_DIR/pre-commit" ]; then
    echo "⚠️  Existing pre-commit hook found"
    BACKUP="$HOOKS_DIR/pre-commit.backup.$(date +%Y%m%d_%H%M%S)"
    cp "$HOOKS_DIR/pre-commit" "$BACKUP"
    echo "✅ Backed up to: $BACKUP"
    echo ""
fi

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'HOOK'
#!/bin/bash
# Pre-Commit Hook - Chicago TDD Tools
# FMEA Mitigation: Q6 (RPN 144 → 20)
# Automatically runs quality checks before every commit

set -e

echo ""
echo "🔍 Running pre-commit quality checks..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Change to repository root
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

# Check if cargo-make is installed
if ! command -v cargo-make &> /dev/null; then
    echo "❌ cargo-make is not installed"
    echo ""
    echo "Install it with: cargo install cargo-make"
    echo ""
    echo "Or skip this check with: git commit --no-verify"
    exit 1
fi

# Run pre-commit checks
echo "Running: cargo make pre-commit"
echo ""

if cargo make pre-commit; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ All pre-commit checks passed!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    exit 0
else
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "❌ Pre-commit checks failed!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Fix the issues above and try again."
    echo ""
    echo "💡 To bypass this check (not recommended):"
    echo "   git commit --no-verify"
    echo ""
    exit 1
fi
HOOK

# Make hook executable
chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Pre-commit hook installed successfully!"
echo ""
echo "📋 What happens now:"
echo "   • Every 'git commit' will run: cargo make pre-commit"
echo "   • Checks include: fmt, lint, test-unit, dead-code-check"
echo "   • Expected duration: ~20 seconds"
echo ""
echo "💡 Tips:"
echo "   • Fix issues before committing (faster feedback)"
echo "   • Bypass check with: git commit --no-verify (not recommended)"
echo "   • Manual check: cargo make pre-commit"
echo ""
echo "🎯 FMEA Impact: RPN 144 → 20 (86% risk reduction)"
echo ""

# Test the hook (optional)
read -p "🧪 Test the pre-commit hook now? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "Testing pre-commit hook..."
    bash "$HOOKS_DIR/pre-commit" || true
fi

echo ""
echo "✅ Installation complete!"
