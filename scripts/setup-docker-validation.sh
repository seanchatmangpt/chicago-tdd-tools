#!/bin/bash
# Docker Validation Script
# FMEA Mitigation: I1 (RPN 210 → 30)
# Ensures Docker is running before integration tests

set -e

echo "🐳 Checking Docker availability..."

# Check if Docker command exists
if ! command -v docker &> /dev/null; then
    echo "❌ Docker command not found"
    echo ""
    echo "Docker is REQUIRED for integration tests."
    echo ""
    echo "📦 Installation instructions:"
    echo ""
    echo "Linux:"
    echo "  curl -fsSL https://get.docker.com | sh"
    echo "  sudo usermod -aG docker \$USER"
    echo "  newgrp docker"
    echo ""
    echo "macOS:"
    echo "  brew install --cask docker"
    echo "  # Or download Docker Desktop from https://www.docker.com/products/docker-desktop"
    echo ""
    echo "Windows:"
    echo "  # Download Docker Desktop from https://www.docker.com/products/docker-desktop"
    echo ""
    exit 1
fi

echo "✅ Docker command found"

# Check if Docker daemon is running
if ! timeout 5s docker info &> /dev/null; then
    echo "❌ Docker daemon is not running"
    echo ""
    echo "Docker is installed but the daemon is not running."
    echo ""
    echo "🔧 Start Docker:"
    echo ""
    echo "Linux (systemd):"
    echo "  sudo systemctl start docker"
    echo "  sudo systemctl enable docker"
    echo ""
    echo "macOS:"
    echo "  # Open Docker Desktop application"
    echo ""
    echo "Windows:"
    echo "  # Open Docker Desktop application"
    echo ""
    echo "💡 Alternative: Skip integration tests"
    echo "  cargo make test-unit  # Unit tests only (no Docker required)"
    echo ""
    exit 1
fi

echo "✅ Docker daemon is running"

# Get Docker version
DOCKER_VERSION=$(docker version --format '{{.Server.Version}}' 2>&1)
echo "✅ Docker version: $DOCKER_VERSION"

# Check if user can run Docker without sudo (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if groups | grep -q docker; then
        echo "✅ User is in docker group"
    else
        echo "⚠️  User is not in docker group"
        echo "   Add user to docker group: sudo usermod -aG docker \$USER"
        echo "   Then logout and login again"
    fi
fi

echo ""
echo "🎉 Docker is ready for integration tests!"
exit 0
