#!/usr/bin/env bash

set -euo pipefail

if ! command -v timeout >/dev/null 2>&1; then
  echo "🚨 Required command 'timeout' not found. Install coreutils (e.g., brew install coreutils)" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PROFILE="${PROFILE:-debug}"
WEAVER_VERSION="${WEAVER_VERSION:-0.19.0}"
REGISTRY_URL="https://github.com/open-telemetry/semantic-conventions.git"
WEAVER_BIN_DIR="${PROJECT_ROOT}/target/${PROFILE}"
WEAVER_BIN_PATH="${WEAVER_BIN_DIR}/weaver"
REGISTRY_DIR="${PROJECT_ROOT}/registry"

log() {
  printf '👉 %s\n' "$1"
}

detect_platform() {
  local arch os download_arch download_os

  arch="$(uname -m)"
  os="$(uname -s)"

  case "${arch}" in
    x86_64|amd64)
      download_arch="x86_64"
      ;;
    arm64|aarch64)
      download_arch="aarch64"
      ;;
    *)
      echo "🚨 Unsupported architecture: ${arch}" >&2
      exit 1
      ;;
  esac

  case "${os}" in
    Linux)
      download_os="unknown-linux-gnu"
      ;;
    Darwin)
      download_os="apple-darwin"
      ;;
    *)
      echo "🚨 Unsupported operating system: ${os}" >&2
      exit 1
      ;;
  esac

  printf '%s-%s' "${download_arch}" "${download_os}"
}

ensure_directory() {
  local dir="$1"
  if [ ! -d "${dir}" ]; then
    log "Creating directory ${dir}"
    mkdir -p "${dir}"
  fi
}

download_weaver() {
  if [ -x "${WEAVER_BIN_PATH}" ]; then
    log "Weaver binary already present at ${WEAVER_BIN_PATH}"
    return
  fi

  ensure_directory "${WEAVER_BIN_DIR}"

  local platform
  platform="$(detect_platform)"
  local download_url="https://github.com/open-telemetry/weaver/releases/download/v${WEAVER_VERSION}/weaver-${platform}.tar.xz"

  log "Downloading Weaver ${WEAVER_VERSION} for ${platform}"
  local tmp_dir
  tmp_dir="$(mktemp -d)"

  trap 'rm -rf "${tmp_dir}"' EXIT

  local archive="${tmp_dir}/weaver.tar.xz"
  if command -v curl >/dev/null 2>&1; then
    timeout 60s curl -LsSf "${download_url}" -o "${archive}"
  elif command -v wget >/dev/null 2>&1; then
    timeout 60s wget -q "${download_url}" -O "${archive}"
  else
    echo "🚨 Neither curl nor wget is available for download" >&2
    exit 1
  fi

  timeout 30s tar -xJf "${archive}" -C "${tmp_dir}"

  local extracted
  extracted="$(find "${tmp_dir}" -type f -name weaver -print -quit)"
  if [ -z "${extracted}" ]; then
    echo "🚨 Failed to locate weaver binary in archive" >&2
    exit 1
  fi

  mv "${extracted}" "${WEAVER_BIN_PATH}"
  chmod +x "${WEAVER_BIN_PATH}"
  log "Weaver binary installed to ${WEAVER_BIN_PATH}"
}

clone_registry() {
  if [ -d "${REGISTRY_DIR}" ] && [ -d "${REGISTRY_DIR}/.git" ]; then
    log "Registry already present at ${REGISTRY_DIR}"
    return
  fi

  if [ -d "${REGISTRY_DIR}" ]; then
    log "Existing registry directory without git metadata found. Removing."
    rm -rf "${REGISTRY_DIR}"
  fi

  log "Cloning OpenTelemetry semantic conventions registry"
  timeout 60s git clone --depth 1 --single-branch "${REGISTRY_URL}" "${REGISTRY_DIR}"
}

main() {
  download_weaver
  clone_registry

  if [ -x "${WEAVER_BIN_PATH}" ]; then
    timeout 5s "${WEAVER_BIN_PATH}" --version || {
      echo "⚠️  Weaver binary installed but version command failed" >&2
      exit 1
    }
  fi

  if [ -d "${REGISTRY_DIR}" ]; then
    log "Semantic convention registry cloned to ${REGISTRY_DIR}"
  fi

  log "Weaver bootstrap complete"
}

main "$@"
