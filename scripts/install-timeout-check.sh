#!/bin/bash
# Timeout Command Validation Script
# FMEA Mitigation: M2 (RPN 180 → 20)
# Ensures timeout command is available before running any tasks

set -e

echo "🔍 Checking for timeout command..."

# Check if timeout command exists
if command -v timeout &> /dev/null; then
    VERSION=$(timeout --version 2>&1 | head -n 1)
    echo "✅ timeout command found: $VERSION"
    exit 0
else
    echo "❌ timeout command not found"
    echo ""
    echo "The timeout command is REQUIRED for chicago-tdd-tools."
    echo "All cargo-make tasks have timeout protection to prevent hangs."
    echo ""
    echo "📦 Installation instructions:"
    echo ""

    # Detect OS and provide installation instructions
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "Linux (Debian/Ubuntu):"
        echo "  sudo apt-get install coreutils"
        echo ""
        echo "Linux (RHEL/CentOS):"
        echo "  sudo yum install coreutils"
        echo ""
        echo "Linux (Arch):"
        echo "  sudo pacman -S coreutils"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macOS:"
        echo "  brew install coreutils"
        echo ""
        echo "Note: On macOS, you may need to use 'gtimeout' instead"
        echo "      or add coreutils to PATH:"
        echo "      export PATH=\"/usr/local/opt/coreutils/libexec/gnubin:\$PATH\""
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
        echo "Windows (Git Bash):"
        echo "  timeout command should be available in Git Bash"
        echo "  If not, install Git for Windows: https://git-scm.com/download/win"
        echo ""
        echo "Windows (WSL):"
        echo "  sudo apt-get install coreutils"
    else
        echo "Unknown OS: $OSTYPE"
        echo "Please install GNU coreutils for your platform"
    fi

    echo ""
    echo "After installation, run this script again to verify."
    exit 1
fi
