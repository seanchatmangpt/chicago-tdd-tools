#!/bin/bash
# Install Git hooks for Chicago TDD Tools
# FMEA Fix: Automatic hook installation (RPN: 180 → 36)
#
# This script installs pre-commit and pre-push hooks:
# - Pre-commit: Prevents unwrap/expect in production code
# - Pre-push: Ensures examples compile and tests pass
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

# Install pre-push hook
HOOK_SOURCE_PUSH="$SCRIPT_DIR/hooks/pre-push"
HOOK_DEST_PUSH="$HOOKS_DIR/pre-push"

if [ ! -f "$HOOK_SOURCE_PUSH" ]; then
    echo "⚠️  Pre-push hook source not found: $HOOK_SOURCE_PUSH"
    echo "Skipping pre-push hook installation"
else
    # Check if hook already exists
    if [ -f "$HOOK_DEST_PUSH" ]; then
        echo "⚠️  Pre-push hook already exists at: $HOOK_DEST_PUSH"
        echo "   Existing hook will be backed up to: ${HOOK_DEST_PUSH}.backup"
        read -p "Overwrite? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Skipping pre-push hook installation"
        else
            # Backup existing hook
            cp "$HOOK_DEST_PUSH" "${HOOK_DEST_PUSH}.backup"
            cp "$HOOK_SOURCE_PUSH" "$HOOK_DEST_PUSH"
            chmod +x "$HOOK_DEST_PUSH"
            echo "✅ Installed pre-push hook: $HOOK_DEST_PUSH"
            echo "   (Backup saved to: ${HOOK_DEST_PUSH}.backup)"
        fi
    else
        cp "$HOOK_SOURCE_PUSH" "$HOOK_DEST_PUSH"
        chmod +x "$HOOK_DEST_PUSH"
        echo "✅ Installed pre-push hook: $HOOK_DEST_PUSH"
    fi
fi

echo ""
echo "📋 Hook capabilities:"
echo ""
echo "Pre-commit hook:"
echo "  - Prevents .unwrap() in production code"
echo "  - Prevents .expect() in production code"
echo "  - Allows unwrap/expect in tests, examples, benches"
echo "  - Allows #[allow(clippy::unwrap_used)] with justification"
echo ""
echo "Pre-push hook:"
echo "  - Ensures all examples compile successfully"
echo "  - Ensures all example tests pass"
echo "  - Prevents pushing broken examples"
echo ""
echo "🔧 To bypass (DISCOURAGED):"
echo "  - Pre-commit: SKIP_UNWRAP_CHECK=1 git commit"
echo "  - Pre-push: SKIP_EXAMPLES_CHECK=1 git push"
echo ""
echo "🗑️  To uninstall:"
echo "  - rm .git/hooks/pre-commit"
echo "  - rm .git/hooks/pre-push"
echo ""
echo "✅ Git hooks installed successfully!"
