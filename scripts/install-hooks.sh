#!/bin/bash
# Install Git hooks for Chicago TDD Tools
# FMEA Fix: Automatic hook installation (RPN: 180 → 36)
#
# This script installs pre-commit hooks to prevent unwrap/expect in production code.
# Run this once after cloning the repository: ./scripts/install-hooks.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "🔧 Installing Git hooks for Chicago TDD Tools..."

# Check if we're in a git repository
if [ ! -d "$REPO_ROOT/.git" ]; then
    echo "❌ ERROR: Not in a git repository (.git directory not found)"
    echo "Run this script from the repository root: ./scripts/install-hooks.sh"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Install pre-commit hook
HOOK_SOURCE="$SCRIPT_DIR/hooks/pre-commit"
HOOK_DEST="$HOOKS_DIR/pre-commit"

if [ ! -f "$HOOK_SOURCE" ]; then
    echo "❌ ERROR: Hook source not found: $HOOK_SOURCE"
    exit 1
fi

# Check if hook already exists
if [ -f "$HOOK_DEST" ]; then
    echo "⚠️  Pre-commit hook already exists at: $HOOK_DEST"
    read -p "Overwrite? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Skipping hook installation"
        exit 0
    fi
fi

# Copy and make executable
cp "$HOOK_SOURCE" "$HOOK_DEST"
chmod +x "$HOOK_DEST"

echo "✅ Installed pre-commit hook: $HOOK_DEST"
echo ""
echo "📋 Hook capabilities:"
echo "  - Prevents .unwrap() in production code"
echo "  - Prevents .expect() in production code"
echo "  - Allows unwrap/expect in tests, examples, benches"
echo "  - Allows #[allow(clippy::unwrap_used)] with justification"
echo ""
echo "🔧 To bypass (DISCOURAGED): SKIP_UNWRAP_CHECK=1 git commit"
echo "🗑️  To uninstall: rm .git/hooks/pre-commit"
echo ""
echo "✅ Git hooks installed successfully!"
