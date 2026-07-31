#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
BUILD_ROOT="${RUNNER_TEMP:-/tmp}/ctdd-ggen-build"
GGEN_REPO="$BUILD_ROOT/ggen"
GGEN_SHA="00a924e73acf03be1dd18968f797b3bb61fb8650"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$BUILD_ROOT/target}"
export CARGO_TARGET_DIR

fetch_exact() {
  local repo=$1
  local sha=$2
  local destination=$3
  if [[ -d "$destination/.git" ]] && [[ "$(git -C "$destination" rev-parse HEAD)" == "$sha" ]]; then
    return
  fi
  rm -rf "$destination"
  git init -q "$destination"
  git -C "$destination" remote add origin "https://github.com/seanchatmangpt/$repo"
  git -C "$destination" fetch -q --depth 1 origin "$sha"
  git -C "$destination" checkout -q FETCH_HEAD
  test "$(git -C "$destination" rev-parse HEAD)" = "$sha"
}

mkdir -p "$BUILD_ROOT"
fetch_exact ggen "$GGEN_SHA" "$GGEN_REPO"
fetch_exact lsp-max 7bcc1e16dec71ef5fb2cedea2dfd6cb5cde37f59 "$BUILD_ROOT/lsp-max"
fetch_exact lsp-types-max 6773e6017ca83c565785d0ec39d75304f62c3237 "$BUILD_ROOT/lsp-types-max"
fetch_exact wasm4pm 0bb134b29245517ac4969a9f1916f5432931c5d0 "$BUILD_ROOT/wasm4pm"
fetch_exact wasm4pm-compat e46155e209a750fda0218532d96ae17a9e10903e "$BUILD_ROOT/wasm4pm-compat"

cargo +nightly-2026-06-22 build \
  --manifest-path "$GGEN_REPO/Cargo.toml" \
  -p ggen-cli-lib \
  --bin ggen
GGEN="$CARGO_TARGET_DIR/debug/ggen"
test -x "$GGEN"

WORK=$(mktemp -d)
cleanup() { rm -rf "$WORK"; }
trap cleanup EXIT
cp -R "$ROOT/standards/chicago-tdd" "$WORK/cell"
rm -rf "$WORK/cell/generated" "$WORK/cell/.ggen-v2"

(
  cd "$WORK/cell"
  "$GGEN" sync run
  "$GGEN" receipt verify
)

PROJECTED="$WORK/cell/generated"
if [[ ! -d "$PROJECTED" ]]; then
  # ggen 26.7.61 writes output_file paths relative to the cell root even when
  # generation.output_dir is present. Normalize those real outputs into the
  # authored ownership root before byte comparison.
  PROJECTED="$WORK/projected"
  mkdir -p "$PROJECTED/src" "$PROJECTED/tests"
  for rel in \
    STANDARDS.md REFUSALS.md CHECKPOINTS.md Cargo.toml \
    src/standards.rs src/refusals.rs src/checkpoints.rs src/lib.rs \
    tests/contracts.rs
  do
    test -f "$WORK/cell/$rel"
    mkdir -p "$PROJECTED/$(dirname "$rel")"
    cp "$WORK/cell/$rel" "$PROJECTED/$rel"
  done
fi

diff -ru "$ROOT/standards/chicago-tdd/generated" "$PROJECTED"
test -f "$WORK/cell/.ggen-v2/receipt.json"
printf '%s\n' "real-ggen-standards: ALIVE @ $GGEN_SHA"
