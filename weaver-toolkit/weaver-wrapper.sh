#!/usr/bin/env bash
# weaver-wrapper.sh — standalone, vendorable convenience wrapper around the
# `weaver` CLI (https://github.com/open-telemetry/weaver).
#
# Portable on purpose: no dependency on this repo's Cargo/Justfile layout, so
# any project can copy this single file (or `git subtree`/`curl` it in) and
# get bootstrap + live-check start/stop/check without pulling in
# chicago-tdd-tools as a Rust dependency.
#
# Usage:
#   ./weaver-wrapper.sh bootstrap   # download weaver binary + semconv registry
#   ./weaver-wrapper.sh check       # static registry validation
#   ./weaver-wrapper.sh live-start  # start an ephemeral OTLP live-check listener
#   ./weaver-wrapper.sh live-stop   # stop it and collect the JSON report
#   ./weaver-wrapper.sh version
#
# Configuration (env vars, all optional):
#   WEAVER_VERSION      weaver release tag to install (default: 0.24.2)
#   WEAVER_REGISTRY_URL semantic-conventions registry git url
#                        (default: https://github.com/open-telemetry/semantic-conventions.git)
#   WEAVER_REGISTRY_REF git ref/tag to clone (default: v1.42.0)
#   WEAVER_HOME          install root for the binary + cloned registry
#                        (default: ./.weaver relative to CWD)
#   WEAVER_REGISTRY_PATH override: use an existing local registry instead of cloning
#   WEAVER_OTLP_GRPC_PORT default 4319 (kept off the common 4317 default so a
#                          live-check session never collides with a real,
#                          in-cluster OTLP collector)
#   WEAVER_ADMIN_PORT     default 4321
#   WEAVER_REPORTS_DIR    default ./weaver-reports

set -euo pipefail

log() { printf '👉 %s\n' "$1"; }
die() { printf '🚨 %s\n' "$1" >&2; exit 1; }

command -v timeout >/dev/null 2>&1 || die "Required command 'timeout' not found. Install coreutils (e.g. brew install coreutils)."

WEAVER_VERSION="${WEAVER_VERSION:-0.24.2}"
WEAVER_REGISTRY_URL="${WEAVER_REGISTRY_URL:-https://github.com/open-telemetry/semantic-conventions.git}"
WEAVER_REGISTRY_REF="${WEAVER_REGISTRY_REF:-v1.42.0}"
WEAVER_HOME="${WEAVER_HOME:-$(pwd)/.weaver}"
WEAVER_BIN_DIR="${WEAVER_HOME}/bin"
WEAVER_BIN_PATH="${WEAVER_BIN_DIR}/weaver"
WEAVER_REGISTRY_PATH="${WEAVER_REGISTRY_PATH:-${WEAVER_HOME}/registry}"
WEAVER_OTLP_GRPC_PORT="${WEAVER_OTLP_GRPC_PORT:-4319}"
WEAVER_ADMIN_PORT="${WEAVER_ADMIN_PORT:-4321}"
WEAVER_REPORTS_DIR="${WEAVER_REPORTS_DIR:-$(pwd)/weaver-reports}"
WEAVER_PID_FILE="${WEAVER_HOME}/live-check.pid"

detect_platform() {
  local arch os download_arch download_os
  arch="$(uname -m)"
  os="$(uname -s)"
  case "${arch}" in
    x86_64|amd64) download_arch="x86_64" ;;
    arm64|aarch64) download_arch="aarch64" ;;
    *) die "Unsupported architecture: ${arch}" ;;
  esac
  case "${os}" in
    Linux) download_os="unknown-linux-gnu" ;;
    Darwin) download_os="apple-darwin" ;;
    *) die "Unsupported operating system: ${os}" ;;
  esac
  printf '%s-%s' "${download_arch}" "${download_os}"
}

cmd_bootstrap() {
  mkdir -p "${WEAVER_BIN_DIR}"

  if [ -x "${WEAVER_BIN_PATH}" ]; then
    log "weaver binary already present at ${WEAVER_BIN_PATH}"
  else
    local platform download_url tmp_dir archive extracted
    platform="$(detect_platform)"
    download_url="https://github.com/open-telemetry/weaver/releases/download/v${WEAVER_VERSION}/weaver-${platform}.tar.xz"
    log "downloading weaver ${WEAVER_VERSION} for ${platform}"
    tmp_dir="$(mktemp -d)"
    archive="${tmp_dir}/weaver.tar.xz"
    if command -v curl >/dev/null 2>&1; then
      timeout 60s curl -LsSf "${download_url}" -o "${archive}"
    elif command -v wget >/dev/null 2>&1; then
      timeout 60s wget -q "${download_url}" -O "${archive}"
    else
      rm -rf "${tmp_dir}"
      die "Neither curl nor wget is available for download"
    fi
    timeout 30s tar -xJf "${archive}" -C "${tmp_dir}"
    extracted="$(find "${tmp_dir}" -type f -name weaver -print -quit)"
    if [ -z "${extracted}" ]; then
      rm -rf "${tmp_dir}"
      die "Failed to locate weaver binary in downloaded archive"
    fi
    mv "${extracted}" "${WEAVER_BIN_PATH}"
    chmod +x "${WEAVER_BIN_PATH}"
    rm -rf "${tmp_dir}"
    log "weaver binary installed to ${WEAVER_BIN_PATH}"
  fi

  if [ -n "${WEAVER_REGISTRY_PATH_OVERRIDE:-}" ]; then
    log "using externally supplied registry at ${WEAVER_REGISTRY_PATH}"
  elif [ -d "${WEAVER_REGISTRY_PATH}/.git" ]; then
    log "registry already present at ${WEAVER_REGISTRY_PATH}"
  else
    [ -d "${WEAVER_REGISTRY_PATH}" ] && rm -rf "${WEAVER_REGISTRY_PATH}"
    timeout 60s git clone --depth 1 --branch "${WEAVER_REGISTRY_REF}" --single-branch \
      "${WEAVER_REGISTRY_URL}" "${WEAVER_REGISTRY_PATH}"
    log "semantic-convention registry cloned to ${WEAVER_REGISTRY_PATH}"
  fi

  timeout 5s "${WEAVER_BIN_PATH}" --version || die "weaver binary installed but 'weaver --version' failed"
  log "weaver bootstrap complete"
}

registry_model_dir() {
  # Modern semconv registries nest the schema under model/.
  if [ -d "${WEAVER_REGISTRY_PATH}/model" ]; then
    printf '%s' "${WEAVER_REGISTRY_PATH}/model"
  else
    printf '%s' "${WEAVER_REGISTRY_PATH}"
  fi
}

cmd_check() {
  [ -x "${WEAVER_BIN_PATH}" ] || die "weaver binary not found, run: $0 bootstrap"
  "${WEAVER_BIN_PATH}" registry check -r "$(registry_model_dir)"
}

cmd_live_start() {
  [ -x "${WEAVER_BIN_PATH}" ] || die "weaver binary not found, run: $0 bootstrap"
  mkdir -p "${WEAVER_REPORTS_DIR}"
  log "starting weaver live-check on grpc:${WEAVER_OTLP_GRPC_PORT} admin:${WEAVER_ADMIN_PORT}"
  # Redirect the backgrounded process's stdout/stderr to a log file instead
  # of inheriting this script's own fds. Without this, a caller that
  # captures this script's output (e.g. `subprocess.run(capture_output=True)`
  # from a driving process, or any `$(...)`/pipe) never sees EOF on that
  # pipe -- the still-running weaver process holds the write end open even
  # after this script itself has exited -- so the caller hangs until its own
  # timeout instead of returning once `live-start` completes.
  "${WEAVER_BIN_PATH}" registry live-check -r "$(registry_model_dir)" \
    --otlp-grpc-port "${WEAVER_OTLP_GRPC_PORT}" \
    --admin-port "${WEAVER_ADMIN_PORT}" \
    --format json --output http \
    > "${WEAVER_REPORTS_DIR}/live-check.log" 2>&1 &
  echo $! > "${WEAVER_PID_FILE}"
  log "weaver live-check pid $(cat "${WEAVER_PID_FILE}") (report will land in ${WEAVER_REPORTS_DIR})"
}

cmd_live_stop() {
  mkdir -p "${WEAVER_REPORTS_DIR}"
  curl -s -X POST "http://127.0.0.1:${WEAVER_ADMIN_PORT}/stop" \
    > "${WEAVER_REPORTS_DIR}/report.json" || true
  if [ -f "${WEAVER_PID_FILE}" ]; then
    kill "$(cat "${WEAVER_PID_FILE}")" 2>/dev/null || true
    rm -f "${WEAVER_PID_FILE}"
  fi
  pkill -f 'weaver registry live-check' 2>/dev/null || true
  log "weaver live-check stopped, report at ${WEAVER_REPORTS_DIR}/report.json"
}

cmd_version() {
  [ -x "${WEAVER_BIN_PATH}" ] || die "weaver binary not found, run: $0 bootstrap"
  "${WEAVER_BIN_PATH}" --version
}

main() {
  local sub="${1:-}"
  case "${sub}" in
    bootstrap) cmd_bootstrap ;;
    check) cmd_check ;;
    live-start) cmd_live_start ;;
    live-stop) cmd_live_stop ;;
    version) cmd_version ;;
    *)
      cat >&2 <<EOF
usage: $0 <bootstrap|check|live-start|live-stop|version>
EOF
      exit 1
      ;;
  esac
}

main "$@"
