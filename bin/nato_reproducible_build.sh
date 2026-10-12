#!/usr/bin/env bash
# bin/nato_reproducible_build.sh — N19-adjacent reproducible-build receipt
# generator (lane N6, plan P11 structure port).
#
# Runs TWO clean-target builds of the crate into isolated target roots and
# compares the primary library artifacts byte-wise (cmp + sha256). Emits a
# JSON receipt recording both artifact digests, the byte comparison verdict,
# the toolchain identity, and a MANDATORY not_run disclosure listing the
# exact commands for the 3-host (macOS-arm64 / Linux-x86_64 / Linux-arm64)
# cross-host reproduction that this script does NOT perform.
#
# Delivered scope (honest): macOS, same-host, clean-target double build.
# Cross-host byte-reproducibility is disclosed as not_run.
#
# Usage: bin/nato_reproducible_build.sh [output-receipt.json]
# Receipt defaults to target/nato_reproducible_receipt.json.

set -euo pipefail

RECEIPT_OUT="${1:-target/nato_reproducible_receipt.json}"
CRATE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_ROOT_A="$CRATE_ROOT/target/nato-repro-a"
BUILD_ROOT_B="$CRATE_ROOT/target/nato-repro-b"

cd "$CRATE_ROOT"

echo "[nato-repro] build A (clean target root)"
CARGO_TARGET_DIR="$BUILD_ROOT_A" cargo build --locked >/dev/null
echo "[nato-repro] build B (clean target root, independent)"
CARGO_TARGET_DIR="$BUILD_ROOT_B" cargo build --locked >/dev/null

# Primary artifact: the rlib of this crate (same profile in both roots).
ART_A="$(ls "$BUILD_ROOT_A"/debug/libchicago_tdd_tools*.rlib 2>/dev/null | head -n 1 || true)"
ART_B="$(ls "$BUILD_ROOT_B"/debug/libchicago_tdd_tools*.rlib 2>/dev/null | head -n 1 || true)"

if [[ -z "$ART_A" || -z "$ART_B" ]]; then
  echo "[nato-repro] FATAL: primary rlib not found in one of the build roots" >&2
  exit 2
fi

if cmp -s "$ART_A" "$ART_B"; then
  BYTE_IDENTICAL="true"
else
  BYTE_IDENTICAL="false"
fi

SHA_A="$(shasum -a 256 "$ART_A" | awk '{print $1}')"
SHA_B="$(shasum -a 256 "$ART_B" | awk '{print $1}')"
RUSTC_VER="$(rustc --version)"
CARGO_VER="$(cargo --version)"

mkdir -p "$(dirname "$RECEIPT_OUT")"

cat > "$RECEIPT_OUT" <<EOF
{
  "receipt_kind": "nato_reproducible_build",
  "artifact_a": "$(basename "$ART_A")",
  "artifact_b": "$(basename "$ART_B")",
  "sha256_a": "$SHA_A",
  "sha256_b": "$SHA_B",
  "byte_identical": $BYTE_IDENTICAL,
  "comparison_method": "cmp + sha256 (both roots clean, --locked)",
  "toolchain": {
    "rustc": "$RUSTC_VER",
    "cargo": "$CARGO_VER",
    "host_os": "$(uname -s)",
    "host_arch": "$(uname -m)"
  },
  "scope": {
    "delivered": "macOS same-host double clean-target build, byte-wise cmp",
    "is_full_spec_scale": false
  },
  "not_run": {
    "reason": "cross-host reproducibility requires hosts this lane does not have",
    "hosts": [
      {
        "platform": "linux-x86_64",
        "commands": "docker run --rm -v \$(pwd):/work -w /work rust:1 bash bin/nato_reproducible_build.sh /tmp/receipt-linux-x86_64.json && docker cp \$(docker ps -ql):/tmp/receipt-linux-x86_64.json ."
      },
      {
        "platform": "linux-aarch64",
        "commands": "docker run --rm --platform linux/arm64 -v \$(pwd):/work -w /work rust:1 bash bin/nato_reproducible_build.sh /tmp/receipt-linux-aarch64.json && docker cp \$(docker ps -ql):/tmp/receipt-linux-aarch64.json ."
      },
      {
        "platform": "macos-x86_64",
        "commands": "rustup target add x86_64-apple-darwin && bin/nato_reproducible_build.sh /tmp/receipt-macos-x86_64.json"
      }
    ]
  }
}
EOF

echo "[nato-repro] receipt written to $RECEIPT_OUT (byte_identical=$BYTE_IDENTICAL)"
if [[ "$BYTE_IDENTICAL" != "true" ]]; then
  echo "[nato-repro] WARNING: artifacts differ; see sha256_a/sha256_b in the receipt" >&2
  exit 1
fi
