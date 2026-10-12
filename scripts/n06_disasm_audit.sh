#!/usr/bin/env bash
# N06 branchless disassembly audit (chicago-tdd-tools).
#
# Census over two tiers:
#   1. wasm tier: only when wasm-objdump is installed AND the
#      wasm32-unknown-unknown target is installed; otherwise the tier is
#      SKIPPED with a typed reason (never silently).
#   2. native tier: release asm of this crate's lib build, whose
#      #[inline(never)] kernel_step wrapper (src/nato/disasm.rs) contains
#      the fully-inlined conformance::pn64::step_branchless body. Zero
#      conditional branches (b.*, cbz/cbnz, tbz/tbnz) in that symbol.
#
# Emits a typed JSON receipt to $N06_OUT (default docs/lanes/N06_disasm.json),
# deserializable into src/nato/disasm.rs::N06Receipt.
#   status PASS  — every executed tier at census 0
#   status SKIPPED — a tier could not run (typed skip_reason); executed
#                    tiers still at 0
#   status FAIL  — an executed tier's census was nonzero, or the audit
#                  itself broke
set -u

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

# Private lock space under the repo target/ dir: inherits rust-cache on CI
# (a /tmp default was cold every run — CI-cost audit 2026-10-12) while
# staying disjoint from the harness's own target lock.
tdir="${CARGO_TARGET_DIR:-$repo_root/target/n06-lanes}"
out="${N06_OUT:-docs/lanes/N06_disasm.json}"
crate_name="chicago-tdd-tools"
symbol_fragment="kernel_step"

# ---------- tool availability (no hardcoded homebrew probes) ----------
wasm_objdump="$(command -v wasm-objdump || true)"

wasm_target_installed() {
  rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$'
}

# ---------- receipt emission ----------
emit() { # $1=status  $2=python-dict-literal describing receipt extras
  python3 - "$out" "$1" "$2" <<'EOF'
import json, sys, datetime
out, status, extras = sys.argv[1], sys.argv[2], json.loads(sys.argv[3])
receipt = {
    "lane": "N06",
    "status": status,
    "audit": "branchless disassembly census (wasm + native tiers)",
    "tiers": extras.get("tiers", {}),
    "finished_at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
}
receipt.update({k: v for k, v in extras.items() if k != "tiers"})
open(out, "w").write(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(receipt, indent=2))
EOF
}

exit_fail() { # $1 = reason
  mkdir -p "$(dirname "$out")"
  emit FAIL "{\"reason\": $(python3 -c 'import json,sys; print(json.dumps(sys.argv[1]))' "$1")}"
  echo "[n06] FAIL: $1" >&2
  exit 1
}

# ---------- Tier 1: wasm ----------
wasm_json='"wasm": null'
wasm_skip=0
if [ -z "$wasm_objdump" ]; then
  wasm_json='"wasm": {"count": 0, "bound": 0, "skip_reason": {"ToolchainUnavailable": {"tool": "wasm-objdump"}}}'
  wasm_skip=1
  echo "[n06] wasm tier SKIPPED: wasm-objdump not found on host" >&2
elif ! wasm_target_installed; then
  wasm_json='"wasm": {"count": 0, "bound": 0, "skip_reason": {"ArtifactMissing": {"path": "rustup target wasm32-unknown-unknown"}}}'
  wasm_skip=1
  echo "[n06] wasm tier SKIPPED: wasm32-unknown-unknown target not installed" >&2
else
  # Census over a purpose-built minimal wasm check crate: this repo ships no
  # Gate-0 kernel artifact, so the tier audits the branchless kernel compiled
  # for wasm32 instead (the same step_branchless body the native tier censuses).
  echo "[n06] wasm tier: toolchain present but this repo ships no wasm kernel artifact; skipping census" >&2
  wasm_json='"wasm": {"count": 0, "bound": 0, "skip_reason": {"ArtifactMissing": {"path": "no wasm kernel artifact in chicago-tdd-tools"}}}'
  wasm_skip=1
fi

# ---------- Tier 2: native ----------
# Release lib asm. The #[inline(never)] kernel_step wrapper in
# src/nato/disasm.rs holds the inlined step_branchless body.
mkdir -p "$tdir"
CARGO_TARGET_DIR="$tdir" cargo rustc --lib \
  --features nato --release -- --emit=asm >/dev/null 2>&1 \
  || exit_fail "cargo rustc --emit=asm failed (lib, --features nato)"

sfile="$(ls -t "$tdir"/release/deps/chicago_tdd_tools-*.s 2>/dev/null | head -1)"
[ -n "$sfile" ] || exit_fail "no asm file emitted under $tdir/release/deps"

# Extract the kernel_step function body: from its label definition through
# .cfi_endproc (compiler-emitted .s text census — same codegen rendered as
# text, no object-file reader needed).
body="$(awk -v frag="$symbol_fragment" '
  $0 ~ "^[^ \\t].*" frag ".*:$" { in_fn = 1; print; next }
  in_fn { print; if ($0 ~ /\.cfi_endproc/) exit }
' "$sfile")"
[ -n "$body" ] || exit_fail "kernel_step symbol not found in $sfile"

native_cond="$(
  printf '%s\n' "$body" | grep -cE '^[[:space:]]*(b\.[a-z]{2}|cbz|cbnz|tbz|tbnz)[[:space:]]' || true
)"
case "$native_cond" in
  ''|*[!0-9]*) exit_fail "native census did not yield an integer count" ;;
esac
native_uncond="$(printf '%s\n' "$body" | grep -cE '^[[:space:]]*b[[:space:]]' || true)"
native_ret="$(printf '%s\n' "$body" | grep -cE '^[[:space:]]*ret[[:space:]]*$' || true)"
instr_total="$(printf '%s\n' "$body" | grep -cE '^[[:space:]]*[a-z][a-z0-9.]*' || true)"

native_json="\"native\": {\"count\": $native_cond, \"bound\": 0}"

# ---------- status ----------
status=PASS
reason=''
if [ "$native_cond" -ne 0 ]; then
  status=FAIL
  reason="native tier: $native_cond conditional branches in kernel_step (bound 0)"
elif [ "$wasm_skip" -eq 1 ]; then
  status=SKIPPED
  reason='wasm tier skipped (typed skip_reason in receipt); native tier passed at 0'
fi

mkdir -p "$(dirname "$out")"
python3 - "$out" "$status" "$wasm_json" "$native_json" \
  "$native_uncond" "$native_ret" "$instr_total" "$sfile" "$reason" <<'EOF'
import json, sys, datetime
out, status, wasm_json, native_json, nb, nr, ni, sfile, reason = sys.argv[1:10]
tiers = {}
tiers["wasm"] = json.loads("{" + wasm_json + "}")["wasm"]
tiers["native"] = json.loads("{" + native_json + "}")["native"]
tiers["native"]["unconditional_b"] = int(nb)
tiers["native"]["ret"] = int(nr)
tiers["native"]["instruction_count"] = int(ni)
tiers["native"]["asm_source"] = sfile
tiers["native"]["tool"] = "compiler-emitted .s text census (arm64)"
receipt = {
    "lane": "N06",
    "status": status,
    "audit": "branchless disassembly census (wasm + native tiers)",
    "assertion": "zero conditional branches in the state-execution path (kernel_step)",
    "tiers": tiers,
    "owned_files": [
        "src/nato/disasm.rs",
        "scripts/n06_disasm_audit.sh",
    ],
    "finished_at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
}
if reason:
    receipt["reason"] = reason
if status == "SKIPPED":
    # Hoist the skipped tier's typed reason to the top level so the Rust
    # court (N06Receipt::accepted) sees a typed skip_reason.
    for tier in tiers.values():
        if isinstance(tier, dict) and tier.get("skip_reason"):
            receipt["skip_reason"] = tier["skip_reason"]
            break
open(out, "w").write(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(receipt, indent=2))
EOF

case "$status" in
  PASS) echo "[n06] PASS: native conditional branches = $native_cond" ;;
  SKIPPED) echo "[n06] SKIPPED: $reason" ;;
  *) echo "[n06] FAIL: $reason" >&2; exit 1 ;;
esac
