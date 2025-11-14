#!/bin/bash
# Prerequisite Verification Script
# FMEA Mitigation: M1 (RPN 140 → 20)
# Ensures all required tools are available

set -e

MISSING_TOOLS=()
WARNINGS=()

echo "🔍 Verifying chicago-tdd-tools prerequisites..."
echo ""

# Function to check if a command exists
check_command() {
    local cmd=$1
    local required=$2
    local install_hint=$3

    if command -v "$cmd" &> /dev/null; then
        local version=$(eval "$cmd --version 2>&1 | head -n 1" || echo "unknown")
        echo "✅ $cmd: $version"
        return 0
    else
        if [ "$required" = "true" ]; then
            echo "❌ $cmd: NOT FOUND (required)"
            MISSING_TOOLS+=("$cmd|$install_hint")
            return 1
        else
            echo "⚠️  $cmd: NOT FOUND (optional)"
            WARNINGS+=("$cmd|$install_hint")
            return 0
        fi
    fi
}

echo "📦 Required Tools:"
echo "─────────────────"
check_command "cargo" "true" "Install Rust: https://rustup.rs/"
check_command "cargo-make" "true" "cargo install cargo-make"
check_command "timeout" "true" "Install coreutils (Linux: apt-get install coreutils, macOS: brew install coreutils)"

echo ""
echo "📦 Build Tools (required for tests):"
echo "────────────────────────────────────"
check_command "cargo-nextest" "true" "cargo install cargo-nextest"

echo ""
echo "📦 Optional Tools (recommended):"
echo "─────────────────────────────────"
check_command "docker" "false" "Install Docker: https://docs.docker.com/get-docker/"
check_command "cargo-audit" "false" "cargo install cargo-audit --features=fix"
check_command "cargo-llvm-cov" "false" "cargo install cargo-llvm-cov"
check_command "cargo-mutants" "false" "cargo install cargo-mutants"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for missing required tools
if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo ""
    echo "❌ MISSING REQUIRED TOOLS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    for tool_info in "${MISSING_TOOLS[@]}"; do
        IFS='|' read -r tool hint <<< "$tool_info"
        echo "Tool: $tool"
        echo "Install: $hint"
        echo ""
    done
    echo "Please install the missing tools and run this script again."
    exit 1
fi

# Show warnings for optional tools
if [ ${#WARNINGS[@]} -gt 0 ]; then
    echo ""
    echo "⚠️  OPTIONAL TOOLS NOT FOUND"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "These tools are optional but recommended:"
    echo ""
    for tool_info in "${WARNINGS[@]}"; do
        IFS='|' read -r tool hint <<< "$tool_info"
        echo "• $tool"
        echo "  Install: $hint"
        echo ""
    done
fi

echo ""
echo "✅ All required prerequisites are installed!"
echo ""
echo "🚀 Next steps:"
echo "   • Run tests: cargo make test-unit"
echo "   • Run all checks: cargo make pre-commit"
echo "   • Install pre-commit hook: bash scripts/install-hooks.sh"
echo ""
echo "🎯 FMEA Impact: RPN 140 → 20 (86% risk reduction)"
echo ""

exit 0
