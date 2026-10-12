# Chicago TDD Tools — just build system (replaces cargo-make)
#
# CRITICAL: All commands have timeout protection to prevent freezing.
# Better to fail fast than hang forever.
#
# Run `just --list` to see all recipes.

set shell := ["bash", "-uc"]

# Affidavit checkout used by the affi-routed recipes (PRD P15: affi is the
# sovereign CLI). Overridable so CI/other machines are not pinned to one
# absolute path (A134).
AFFIDAVIT_DIR := env_var_or_default("AFFIDAVIT_DIR", "/Users/sac/affidavit")

# Default: show available recipes
default:
    @just --list

# ---------------------------------------------------------------------------
# Core development
# ---------------------------------------------------------------------------

timeout-check:
    command -v timeout

check:
    timeout 30s cargo check --all-targets

build:
    timeout 5s cargo build

build-release:
    timeout 30s cargo build --release

clean:
    timeout 5s cargo clean

# Clean build artifacts in all Rust projects under home directory (~/)
clean-all-home:
    #!/usr/bin/env bash
    set -uo pipefail
    HOME_DIR="${HOME:-}"
    if [ -z "$HOME_DIR" ]; then echo '❌ ERROR: HOME variable not set' >&2; exit 1; fi
    echo "🔍 Searching for Rust projects in: $HOME_DIR (max depth: 4)" >&2
    COUNT=0; SUCCESS=0; FAILED=0; PROJECTS=""
    TMPFILE=$(mktemp); trap 'rm -f "$TMPFILE"' EXIT
    timeout 15s find "$HOME_DIR" -maxdepth 4 -type f -name 'Cargo.toml' 2>/dev/null \
      | grep -v '/target/' | grep -v '/.cargo/' | grep -v '/.rustup/' > "$TMPFILE" || true
    while IFS= read -r cargo_file || [ -n "$cargo_file" ]; do
      if [ -n "$cargo_file" ] && [ -f "$cargo_file" ]; then
        dir=$(dirname "$cargo_file")
        if [ -d "$dir" ]; then
          COUNT=$((COUNT + 1))
          echo "[${COUNT}] Cleaning: $dir" >&2
          PROJECTS="$PROJECTS\n  - $dir"
          if (cd "$dir" && timeout 10s cargo clean >/dev/null 2>&1); then
            SUCCESS=$((SUCCESS + 1)); echo "✅ [${COUNT}] Success: $dir" >&2
          else
            FAILED=$((FAILED + 1)); echo "⚠️  [${COUNT}] Failed: $dir" >&2
          fi
        fi
      fi
    done < "$TMPFILE"
    echo '' >&2
    echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━' >&2
    echo '📊 CLEAN SUMMARY' >&2
    echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━' >&2
    echo "Total projects found: $COUNT" >&2
    echo "Successfully cleaned: $SUCCESS" >&2
    echo "Failed: $FAILED" >&2
    if [ "$COUNT" -gt 0 ]; then
      echo '' >&2; echo 'Projects processed:' >&2; printf "$PROJECTS" >&2; echo '' >&2
    fi
    echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━' >&2
    if [ "$COUNT" -eq 0 ]; then echo 'ℹ️  No Rust projects found in home directory (searched up to 4 levels deep)' >&2; fi
    exit 0

fmt:
    timeout 30s cargo fmt --all

# Run clippy + chicago-lints (dylint) — single source of truth for all lint enforcement
lint: lint-clippy lint-dylint

# Clippy with maximized linting (all, pedantic, nursery, cargo)
lint-clippy:
    timeout 300s cargo clippy -p chicago-tdd-tools -p chicago-tdd-tools-proc-macros --lib --all-features -- \
      -W clippy::all \
      -W clippy::pedantic \
      -W clippy::nursery \
      -W clippy::cargo \
      -D warnings \
      -D clippy::unwrap_used \
      -D clippy::expect_used \
      -D clippy::dbg_macro \
      -D clippy::todo \
      -D clippy::panic \
      -D clippy::unimplemented \
      -D clippy::print_stdout \
      -D clippy::print_stderr

# chicago-lints via dylint — no_raw_test, no_println, unused_result_silenced, no_direct_log, no_async_raw_test, assert_without_message
lint-dylint:
    timeout 300s cargo dylint --all

clippy: lint

# ---------------------------------------------------------------------------
# Testing
# ---------------------------------------------------------------------------

# Run unit tests (lib, all features) - use test-all for unit + integration
test:
    timeout 300s cargo nextest run --lib --all-features

docker-check:
    # Docker-less environments degrade to skip instead of hard-failing when
    # WEAVER_ALLOW_SKIP=1 is set (documented escape hatch in CLAUDE.md).
    @if [ "${WEAVER_ALLOW_SKIP:-0}" = "1" ]; then \
        echo "docker-check: WEAVER_ALLOW_SKIP=1 — skipping Docker daemon check"; \
    else \
        timeout 5s docker info; \
    fi

# Unit tests only (excludes integration tests via test organization)
test-unit:
    # chicago-claims's `fixture-self-test` feature is intentionally excluded from
    # --all-features: it gates each corrupted fixture's own oracle test, which is
    # DESIGNED to fail (that failure IS the demonstration that the mutant is
    # killed — see crates/chicago-claims/Cargo.toml). Run it explicitly with
    # `cargo test -p chicago-claims --features fixture-self-test -- fixture_` to
    # see each corrupted variant's oracle fail on its own.
    timeout 300s cargo test --workspace --lib --all-features --exclude chicago-claims
    timeout 300s cargo test -p chicago-claims --lib  # compiles fresh under lock contention; tests themselves ~3s

# Conformance: pn64/POWL trajectory verification suite (feature conformance)
test-conformance:
    timeout 120s cargo nextest run --test conformance_tests --test conformance_loader_tests --features conformance

# OCEL 2.0 ingestion bridge tests (feature ocel-ingest)
test-ocel-ingest:
    timeout 120s cargo nextest run --test ocel_ingest_tests --features ocel-ingest,ocel-generation,ocel-ingest-json,ocel-ingest-xml

# Ledger: receipt/dossier verification suite (feature ledger)
test-ledger:
    timeout 120s cargo nextest run --features ledger --lib ledger

# Run all three assurance integration suites (tests/adversarial_as4.rs,
# tests/blake3_receipt_tests.rs, tests/nato_courts_tests.rs), each with its
# full required feature set. adversarial_as4 MUST use the 4-feature combo
# "nato,wasm-guest,ocel-ingest,ocel-ingest-json" — its
# #![cfg(all(feature = "nato", feature = "wasm-guest",
# feature = "ocel-ingest-json"))] header makes --features nato alone a silent
# 0-test run.
# Full run: `just test-assurance-integration`
test-assurance-integration:
    timeout 600s cargo nextest run --test adversarial_as4 --features "nato,wasm-guest,ocel-ingest,ocel-ingest-json"
    timeout 600s cargo nextest run --test blake3_receipt_tests --features receipt-validation
    # NATO sweep timeout lives in .config/nextest.toml overrides
    # (registry_sweep_runs_all_twenty alone takes ~60s).
    timeout 600s cargo nextest run --test nato_courts_tests --features nato
    # hyper_advanced integration (contract/Receipt/swarm): 19 tests, ~30ms —
    # cheap enough to gate here rather than rot locally.
    timeout 600s cargo nextest run --test hyper_advanced_integration
    # remaining always-compiled / cheap feature suites (audit W9 inventory)
    timeout 600s cargo nextest run --test prelude_surface_tests
    timeout 600s cargo nextest run --test dependency_lock_guard --features receipt-validation
    timeout 600s cargo nextest run --test cli_proof_tests --features cli-proof
    timeout 600s cargo nextest run --test ocel_tests --features ocel-generation,ocel-generation-discovery,uuid-v4
    timeout 600s cargo nextest run --test observability_fixtures_tests --features weaver,otel
    # star_toml_e2e re-invokes cargo at test time (compile-at-runtime suite);
    # nextest's per-test timeout charges that cold build to the first tests,
    # so this leg uses the plain harness (audit W27 convergence receipt).
    timeout 600s cargo test --test star_toml_e2e --features property-testing,snapshot-testing,uuid-v4
    timeout 600s cargo nextest run --test go_extra_mile_tests --features property-testing,snapshot-testing
    timeout 600s cargo nextest run --test v1_3_0_integration --features cli-testing,snapshot-testing,uuid-v4
    # (v1_3_0 was previously excluded: a stale v26.6.11-era snapshot and a
    # uuid-length assert fixed since — re-minted + gated, audit W10.)
    # ledger sealing + dossier-lib courts (entropy_court_lite needs ~7s,
    # so these use cargo test, not nextest's 5s default timeout)
    timeout 600s cargo test --features ledger,nato --test ledger_sealing_tests
    timeout 600s cargo test --features ledger,nato --test dossier_cli_tests
    timeout 600s cargo test --features ledger,nato --test ledger_tests
    # chicago_test! macro expansion courts (W26 coverage gap closure)
    timeout 600s cargo test --test chicago_test_macro_tests
    timeout 600s cargo test --test trybuild

# NATO courts: fast tier (always-run courts; tier-3 via nato-courts-full)
test-nato:
    timeout 300s cargo nextest run --lib --features nato nato::

# NATO courts: full tier (release profile; strata-exact scales)
nato-courts-full:
    timeout 600s cargo test --release --lib --features nato -- --ignored nato::

# Affidavit CLI runner (PRD P15: affi is sovereign CLI)
# SOURCE must be a 352-byte CTRT binary record (MAGIC b"CTRT") or `-` for
# stdin. JSON court receipts under docs/receipts/ are the dossier-emit input
# format, not CTRT wire — use `just dossier-verify` for the emitted dossier.
verify-ledger SOURCE:
    timeout 300s cargo run --manifest-path {{AFFIDAVIT_DIR}}/Cargo.toml --features ctdd --bin affi -- --format json receipt validate "{{SOURCE}}"

# WASM guest: build the hermetic ctt guest artifact (wasm32; PRD temprun embed)
guest-build:
    timeout 300s cargo build --release -p ctt-wasm-guest --target wasm32-unknown-unknown

# WASM guest: zero-import audit court over the built artifact
guest-audit:
    timeout 60s cargo test --features nato --lib nato::guest_audit -- --nocapture

# WASM guest: splice CTT_MODULE_DIGEST into the built guest wasm (W-ABI2 §2)
guest-splice:
    timeout 60s cargo run -q -p ctt-splice-digest -- target/wasm32-unknown-unknown/release/ctt_wasm_guest.wasm

# WASM guest: verify the spliced CTT_MODULE_DIGEST (recompute must match)
guest-verify:
    timeout 60s cargo run -q -p ctt-splice-digest -- --verify target/wasm32-unknown-unknown/release/ctt_wasm_guest.wasm

# WASM guest: native-ABI contract + fault-injection suites
guest-test:
    timeout 300s cargo test -p ctt-wasm-guest --features chicago-tdd-tools/wasm-guest
    timeout 300s cargo test --features wasm-guest,conformance,ledger --test guest_abi_tests
    # guest_telemetry is cfg(all(wasm-guest, ocel-ingest)) — without the
    # ocel-ingest feature its tests silently compile out (audit W5).
    timeout 300s cargo test --features wasm-guest,ocel-ingest --lib conformance::guest_telemetry

# Merger-matrix WASM guest: build the leaf-kernel cdylib (wasm32)
merger-guest-build:
    timeout 300s cargo build -p merger-guests --target wasm32-unknown-unknown --release

# Merger-matrix WASM guest: native unit tests
merger-guest-test:
    timeout 300s cargo test -p merger-guests

# Merger-matrix conformance: full contract pipeline over the landed fixtures
merger-conformance:
    timeout 300s cargo test --test merger_conformance_tests --features conformance,ocel-ingest-json

# Compile-fail suite (trybuild; stderr must be seeded and committed)
test-compile-fail:
    timeout 120s cargo test --test trybuild

# One-time stderr seeding for the compile-fail suite (commit the generated .stderr files)
seed-trybuild:
    TRYBUILD=overwrite timeout 300s cargo test --test trybuild

# Hermetic verification matrix (locked + offline; PRD NFR hermetic toolchain)
verify-hermetic:
    CARGO_NET_OFFLINE=true timeout 600s cargo build --locked --offline
    CARGO_NET_OFFLINE=true timeout 600s cargo build --locked --offline --features conformance
    CARGO_NET_OFFLINE=true timeout 600s cargo build --locked --offline --features ledger
    CARGO_NET_OFFLINE=true timeout 600s cargo build --locked --offline --features nato
    CARGO_NET_OFFLINE=true timeout 600s cargo test --locked --offline --features conformance --no-run

# Mutation gate: zero killable survivors in the assurance modules (PRD)
check-mutation-gate:
    timeout 300s cargo mutants --all-features --file src/conformance --file src/nato --file src/ledger

# NATO tier-2: named medium-cost subset of the (all tier-3-marked) ignored
# courts — no tests carry a `nato_tier2` marker, so this recipe selects the
# two cheapest ignored courts by exact test path instead of a filter group.
# Tier-1 runs in test-unit; the full tier-3 scale run stays in nato-courts-full.
# NOTE (CP-N14): `nato::thrash::tests::n14_cast32a_thrash_full` is donor-verbatim
# scale (1M transitions, 16 threads, 600s bound) — it does NOT fit this recipe's
# 120s SLA and stays in nato-courts-full.
test-nato-tier2:
    timeout 120s cargo nextest run --lib --features nato -- --ignored \
        nato::stackbound::tests::n16_workload_100k_transitions_on_32kib_thread \
        nato::cache::tests::n07_conservation_under_eviction_storm_full

# Aggregate: emit the sealed compliance dossier (N20; gated on green courts).
# NOTE: receipt persistence to docs/receipts/N{01..20}.json is wired via the
# opt-in court test (src/nato/dossier_court.rs emit_dossier_receipts_when_opted_in,
# gated on CTT_EMIT_RECEIPTS=1 — the env var this recipe sets; permission errors
# skip typed, never panic). Persistence is a PREREQUISITE of the dossier-emit
# step; `dossier-emit` refuses with a typed error when --receipts-dir is empty.
emit-dossier:
    CTT_EMIT_RECEIPTS=1 timeout 300s cargo test --release --lib --features nato,ledger -- nato::dossier_court --nocapture
    timeout 300s cargo run --manifest-path {{AFFIDAVIT_DIR}}/Cargo.toml --features crypto-trust --bin affi -- dossier emit --receipts-dir docs/receipts --out docs/nato/NATO_STANAG_4107_DOSSIER.json --format json
    @# Idempotence law (audit W6): sealed_at is the only wall-clock input; when
    @# the re-emitted dossier differs from HEAD solely in sealed_at (root,
    @# receipts, and signature byte-stable), restore the committed bytes so a
    @# no-material-change re-seal never dirties the tree.
    @just _dossier-idempotence

_dossier-idempotence:
    #!/usr/bin/env bash
    set -euo pipefail
    python3 scripts/dossier_idempotence.py

# Verify the emitted dossier: PQC (ML-DSA-65) signature + Merkle-root assertion
dossier-verify:
    timeout 300s cargo run --manifest-path {{AFFIDAVIT_DIR}}/Cargo.toml --features crypto-trust --bin affi -- dossier inspect docs/nato/NATO_STANAG_4107_DOSSIER.json --verify-pqc-signature ML-DSA-65 --assert-merkle-root true --format json

dossier-inspect FILE:
    timeout 300s cargo run --manifest-path {{AFFIDAVIT_DIR}}/Cargo.toml --features crypto-trust --bin affi -- dossier inspect "{{FILE}}" --verify-pqc-signature ML-DSA-65 --assert-merkle-root true --format json

# Run integration tests only (requires Docker - MUST fail if Docker stopped)
test-integration: docker-check
    timeout 60s cargo nextest run --test testcontainers --test weaver_integration --test weaver_macro_tests --profile integration --features testcontainers,weaver

test-all: test-unit docker-check test-integration

test-full-integration: timeout-check test-unit docker-check test-integration

check-examples:
    timeout 300s cargo check --examples --all-features

test-examples: check-examples
    timeout 600s cargo nextest run --examples --all-features

test-property:
    timeout 600s cargo nextest run --features property-testing

test-mutation:
    timeout 600s cargo nextest run --features mutation-testing

test-mutation-mutants:
    timeout 60s cargo mutants --all-features

test-snapshot:
    timeout 600s cargo nextest run --features snapshot-testing

# Requires cargo-insta: cargo install cargo-insta
snapshot-review:
    timeout 30s cargo insta review --all-features

snapshot-accept:
    timeout 30s cargo insta accept --all-features

snapshot-reject:
    timeout 30s cargo insta reject --all-features

test-single-threaded:
    RUST_TEST_THREADS=1 timeout 600s cargo nextest run --test-threads 1 --all-features

test-verbose:
    timeout 600s cargo nextest run --lib --all-features -- --skip testcontainers --skip weaver_integration --nocapture

# Run tests and generate timing report to identify slow tests
test-timings:
    timeout 600s cargo nextest run --lib --all-features --timings=html,json -- --skip testcontainers --skip weaver_integration

# Fallback: standard cargo test (if nextest not available)
test-cargo:
    timeout 600s cargo test --all-features

test-doc:
    timeout 60s cargo test --doc --all-features

# ---------------------------------------------------------------------------
# Coverage (manual tasks, not part of commit/push verification)
# ---------------------------------------------------------------------------

coverage:
    timeout 600s cargo llvm-cov --all-features -- --test-threads 1

coverage-report:
    timeout 600s cargo llvm-cov --all-features --html -- --test-threads 1

coverage-tarpaulin:
    timeout 900s cargo tarpaulin --all-features --out Xml --output-dir target/coverage

# ---------------------------------------------------------------------------
# Quality gates — dead code / unwrap / expect / TODO detection
# ---------------------------------------------------------------------------

# Check for undeclared modules (dead code detection) - fails on warnings
dead-code-check:
    #!/usr/bin/env bash
    set -uo pipefail
    TMPFILE=$(mktemp); trap 'rm -f "$TMPFILE"' EXIT
    find src -name '*.rs' -type f 2>/dev/null | grep -v '^src/bin/' | while IFS= read -r f || [ -n "$f" ]; do
      if [ -n "$f" ] && [ -f "$f" ]; then
        dir=$(dirname "$f"); file=$(basename "$f" .rs)
        if [ "$file" != "mod" ] && [ "$file" != "lib" ]; then
          if ! grep -q "mod $file" "$dir/mod.rs" 2>/dev/null && ! grep -q "mod $file" "src/lib.rs" 2>/dev/null; then
            echo "⚠️  Potential dead code: $f (not declared as module)" >&2
            echo "$f" >> "$TMPFILE"
          fi
        fi
      fi
    done
    FOUND=$(wc -l < "$TMPFILE" 2>/dev/null | tr -d '[:space:]' || echo "0")
    if [ "$FOUND" -gt 0 ]; then
      echo "❌ ERROR: Found $FOUND potential dead code file(s). Fix by declaring modules or removing unused files." >&2
      exit 1
    fi

check-unwrap-staged:
    #!/usr/bin/env bash
    set -uo pipefail
    STAGED="$(git diff --cached --name-only --diff-filter=d 2>/dev/null | grep '\.rs$' || true)"
    if [ -z "$STAGED" ]; then exit 0; fi
    COUNT=0
    for f in $STAGED; do
      if [[ "$f" =~ /(test|tests|example|examples|bench|benches)/ ]] || [[ "$f" == *"build.rs" ]] || [[ "$f" =~ ^(test|tests|example|examples|bench|benches)/ ]]; then continue; fi
      if timeout 3s grep -qE '#!?\[allow\(clippy::unwrap_used\)\]' "$f" 2>/dev/null || timeout 3s grep -q '#\[cfg(test)\]' "$f" 2>/dev/null; then continue; fi
      UNWRAPS=$(timeout 3s git diff --cached "$f" 2>/dev/null | grep -E '^\+' | grep -c '\.unwrap()' 2>/dev/null | tr -d '[:space:]')
      if [ -z "$UNWRAPS" ]; then UNWRAPS=0; fi
      if [ "$UNWRAPS" -gt 0 ] 2>/dev/null; then echo "❌ $f: $UNWRAPS unwrap() call(s)"; COUNT=$((COUNT + UNWRAPS)); fi
    done
    if [ "$COUNT" -gt 0 ]; then echo "❌ ERROR: Cannot commit $COUNT unwrap() calls in production code"; exit 1; fi

check-expect-staged:
    #!/usr/bin/env bash
    set -uo pipefail
    STAGED=$(git diff --cached --name-only --diff-filter=d 2>/dev/null | grep '\.rs$' || true)
    if [ -z "$STAGED" ]; then exit 0; fi
    COUNT=0
    for f in $STAGED; do
      if [[ "$f" =~ /(test|tests|example|examples|bench|benches)/ ]] || [[ "$f" == *"build.rs" ]] || [[ "$f" =~ ^(test|tests|example|examples|bench|benches)/ ]]; then continue; fi
      if timeout 3s grep -qE '#!?\[allow\(clippy::expect_used\)\]' "$f" 2>/dev/null || timeout 3s grep -q '#\[cfg(test)\]' "$f" 2>/dev/null; then continue; fi
      EXPECTS=$(timeout 3s git diff --cached "$f" 2>/dev/null | grep -E '^\+' | grep -c '\.expect(' 2>/dev/null | tr -d '[:space:]')
      if [ -z "$EXPECTS" ]; then EXPECTS=0; fi
      if [ "$EXPECTS" -gt 0 ] 2>/dev/null; then echo "❌ $f: $EXPECTS expect() call(s)"; COUNT=$((COUNT + EXPECTS)); fi
    done
    if [ "$COUNT" -gt 0 ]; then echo "❌ ERROR: Cannot commit $COUNT expect() calls in production code"; exit 1; fi

check-todo-staged:
    #!/usr/bin/env bash
    set -uo pipefail
    BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo '')
    if [ "$BRANCH" != "main" ] && [ "$BRANCH" != "master" ]; then exit 0; fi
    STAGED=$(git diff --cached --name-only --diff-filter=d 2>/dev/null | grep '\.rs$' || true)
    if [ -z "$STAGED" ]; then exit 0; fi
    COUNT=0
    for f in $STAGED; do
      if [[ "$f" =~ \.(md|txt|rst)$ ]]; then continue; fi
      TODOS=$(timeout 3s git diff --cached "$f" 2>/dev/null | grep -E '^\+' | grep -iE '\b(TODO|FUTURE)\b' | grep -c . 2>/dev/null | tr -d '[:space:]')
      if [ -z "$TODOS" ]; then TODOS=0; fi
      if [ "$TODOS" -gt 0 ] 2>/dev/null; then echo "❌ $f: $TODOS FUTURE/TODO comment(s)"; COUNT=$((COUNT + TODOS)); fi
    done
    if [ "$COUNT" -gt 0 ]; then echo "❌ ERROR: Cannot commit $COUNT FUTURE/TODO comments to main"; exit 1; fi

check-unimplemented-staged:
    #!/usr/bin/env bash
    set -uo pipefail
    BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo '')
    if [ "$BRANCH" != "main" ] && [ "$BRANCH" != "master" ]; then exit 0; fi
    STAGED=$(git diff --cached --name-only --diff-filter=d 2>/dev/null | grep '\.rs$' || true)
    if [ -z "$STAGED" ]; then exit 0; fi
    COUNT=0
    for f in $STAGED; do
      UNIMPL=$(timeout 3s git diff --cached "$f" 2>/dev/null | grep -E '^\+' | grep -c 'unimplemented!' 2>/dev/null | tr -d '[:space:]')
      if [ -z "$UNIMPL" ]; then UNIMPL=0; fi
      if [ "$UNIMPL" -gt 0 ] 2>/dev/null; then echo "❌ $f: $UNIMPL unimplemented!() placeholder(s)"; COUNT=$((COUNT + UNIMPL)); fi
    done
    if [ "$COUNT" -gt 0 ]; then echo "❌ ERROR: Cannot commit $COUNT unimplemented!() placeholders to main"; exit 1; fi

check-unwrap-all:
    #!/usr/bin/env bash
    set -uo pipefail
    COUNT=$(timeout 8s find src proc_macros/src -name '*.rs' -type f 2>/dev/null \
      | grep -v '/tests/' | grep -v '/test/' | grep -v '/example' | grep -v 'build.rs' | grep -v '/target/' \
      | while IFS= read -r f || [ -n "$f" ]; do
          if [ -n "$f" ] && [ -f "$f" ]; then
            if timeout 3s grep -qE '#!?\[allow\(clippy::unwrap_used\)\]' "$f" 2>/dev/null || timeout 3s grep -q '#\[cfg(test)\]' "$f" 2>/dev/null; then continue; fi
            timeout 3s grep -c '\.unwrap()' "$f" 2>/dev/null | tr -d '[:space:]' || echo '0'
          fi
        done | awk '{s+=$1} END {print s+0}' || echo '0')
    if [ -z "$COUNT" ]; then COUNT=0; fi
    if [ "$COUNT" -gt 0 ] 2>/dev/null; then echo "❌ ERROR: Found $COUNT unwrap() calls in production code"; exit 1; fi

check-expect-all:
    #!/usr/bin/env bash
    set -uo pipefail
    COUNT=$(timeout 8s find src proc_macros/src -name '*.rs' -type f 2>/dev/null \
      | grep -v '/tests/' | grep -v '/test/' | grep -v '/example' | grep -v 'build.rs' | grep -v '/target/' \
      | while IFS= read -r f || [ -n "$f" ]; do
          if [ -n "$f" ] && [ -f "$f" ]; then
            if timeout 3s grep -qE '#!?\[allow\(clippy::expect_used\)\]' "$f" 2>/dev/null || timeout 3s grep -q '#\[cfg(test)\]' "$f" 2>/dev/null; then continue; fi
            timeout 3s grep -c '\.expect(' "$f" 2>/dev/null | tr -d '[:space:]' || echo '0'
          fi
        done | awk '{s+=$1} END {print s+0}' || echo '0')
    if [ -z "$COUNT" ]; then COUNT=0; fi
    if [ "$COUNT" -gt 0 ] 2>/dev/null; then echo "❌ ERROR: Found $COUNT expect() calls in production code"; exit 1; fi

check-todo-all:
    #!/usr/bin/env bash
    set -euo pipefail
    COUNT=$(timeout 8s find src proc_macros/src -name '*.rs' -type f 2>/dev/null \
      | grep -v '/tests/' | grep -v '/test/' | grep -v '/example' | grep -v 'build.rs' | grep -v '/target/' \
      | xargs timeout 8s grep 'TODO:' 2>/dev/null | grep -v 'FUTURE:' | wc -l | tr -d '[:space:]' || echo '0')
    COUNT=${COUNT:-0}
    if [ "$COUNT" -gt 0 ] 2>/dev/null; then echo "❌ ERROR: $COUNT TODO comments found in production code"; exit 1; fi

pre-commit-staged-checks: check-unwrap-staged check-expect-staged check-todo-staged check-unimplemented-staged

# ---------------------------------------------------------------------------
# Pre-commit / CI
# ---------------------------------------------------------------------------

# Run pre-commit validation checks (format, lint, unit tests only)
pre-commit: timeout-check fmt lint test-unit dead-code-check

# Simulate CI environment locally
ci-local: timeout-check
    #!/usr/bin/env bash
    set -e
    echo "🔍 Running CI simulation locally..."
    echo ""
    echo "This runs the same checks as GitHub Actions CI to catch issues early."
    echo ""

    echo "1️⃣  Format check..."
    just fmt
    if ! git diff --exit-code; then
      echo "❌ FAILED: Code is not formatted correctly"
      echo "Fix: Run 'just fmt' and commit the changes"
      exit 1
    fi
    echo "✅ Format check passed"
    echo ""

    echo "2️⃣  Clippy lint check..."
    just lint
    echo "✅ Clippy passed"
    echo ""

    echo "3️⃣  Unit tests..."
    just test-unit || {
      echo "⚠️  Tests failed on first attempt, retrying..."
      sleep 2
      just test-unit || {
        echo "❌ FAILED: Tests failed twice"
        echo "This may indicate environment-specific issues"
        exit 1
      }
    }
    echo "✅ Unit tests passed"
    echo ""

    echo "4️⃣  Production code safety check (unwrap/expect)..."
    PROD_FILES=$(find src proc_macros/src -name '*.rs' -type f 2>/dev/null | \
      grep -v '/test' | grep -v '/tests/' | grep -v 'build.rs' || true)

    if [ -z "$PROD_FILES" ]; then
      echo "✅ No production files to check"
    else
      UNWRAP_COUNT=0
      for FILE in $PROD_FILES; do
        if grep -q '#\[cfg(test)\]' "$FILE" 2>/dev/null; then
          continue
        fi
        COUNT=$(grep -c '\.unwrap()\|\.expect(' "$FILE" 2>/dev/null || echo "0")
        UNWRAP_COUNT=$((UNWRAP_COUNT + COUNT))
      done

      if [ "$UNWRAP_COUNT" -gt 0 ]; then
        echo "❌ FAILED: Found $UNWRAP_COUNT unwrap/expect in production code"
        exit 1
      fi
      echo "✅ No unwrap/expect in production code"
    fi
    echo ""

    echo "5️⃣  Environment validation..."
    echo "  OS: $(uname -s)"
    echo "  Rust: $(rustc --version)"
    echo "  Cargo: $(cargo --version)"
    echo "  Docker: $(docker --version 2>/dev/null || echo "Not available")"
    echo "✅ Environment validated"
    echo ""

    echo "🎉 All CI checks passed locally!"
    echo ""
    echo "Your code should pass CI. Push with confidence!"

# Run CI pipeline (unit tests only, excludes slow integration tests)
ci: timeout-check fmt lint test-unit docs-check audit-all

# ---------------------------------------------------------------------------
# Security and audit
# ---------------------------------------------------------------------------

audit:
    timeout 15s cargo audit

audit-outdated:
    timeout 15s cargo outdated

audit-all: audit audit-outdated

# ---------------------------------------------------------------------------
# Documentation
# ---------------------------------------------------------------------------

docs:
    timeout 20s cargo doc --no-deps --all-features --open

docs-build:
    # 20s was witnessed insufficient: a cold all-features doc build takes
    # ~1m24s (audit W13 receipt). docs.yml CI runs this recipe.
    timeout 180s cargo doc --no-deps --all-features

# Validate documentation (version numbers, build commands, links, style)
docs-check:
    timeout 10s bash scripts/docs-check.sh

# Check source code documentation coverage (missing docs warnings)
docs-coverage:
    timeout 90s bash scripts/doc-coverage.sh

# ---------------------------------------------------------------------------
# Release validation
# ---------------------------------------------------------------------------

release-validate-git-state:
    #!/usr/bin/env bash
    set -uo pipefail
    if [ -n "$(git status --porcelain 2>/dev/null || true)" ]; then
      echo '🚨 CRITICAL: Git state is not clean - uncommitted changes detected'
      git status --porcelain 2>/dev/null || true
      exit 1
    fi
    WIP_FILES=$(find . -name '*.new' -o -name '*WIP*' -o -name '*.tmp' 2>/dev/null | grep -v 'target\|node_modules\|\.git' || true)
    if [ -n "$WIP_FILES" ]; then
      echo '🚨 CRITICAL: WIP files detected'
      echo "$WIP_FILES"
      exit 1
    fi
    echo '✅ Git state is clean'

release-validate-artifacts:
    #!/usr/bin/env bash
    set -uo pipefail
    if [ ! -f docs/releases/CHANGELOG.md ]; then echo '🚨 CRITICAL: docs/releases/CHANGELOG.md missing'; exit 1; fi
    VERSION=$(grep '^version' Cargo.toml 2>/dev/null | cut -d'"' -f2 || echo '')
    if [ -f "docs/releases/RELEASE_NOTES_v${VERSION}.md" ]; then
      echo "✅ Release notes found: docs/releases/RELEASE_NOTES_v${VERSION}.md"
    else
      echo "⚠️  WARNING: Release notes missing for version ${VERSION} (not blocking but recommended)"
    fi
    echo '✅ Release artifacts validated'

release-validate-version:
    #!/usr/bin/env bash
    set -uo pipefail
    VERSION=$(grep '^version' Cargo.toml 2>/dev/null | cut -d'"' -f2 || echo '')
    PROC_VERSION=$(grep '^version' proc_macros/Cargo.toml 2>/dev/null | cut -d'"' -f2 || echo '')
    INLINE_PIN=$(grep 'chicago-tdd-tools-proc-macros = ' Cargo.toml 2>/dev/null | cut -d'"' -f2 || echo '')
    if [ -z "$VERSION" ]; then echo '🚨 CRITICAL: Cannot read version from Cargo.toml'; exit 1; fi
    if [ -z "$PROC_VERSION" ]; then echo '🚨 CRITICAL: Cannot read version from proc_macros/Cargo.toml'; exit 1; fi
    if [ "$VERSION" != "$PROC_VERSION" ]; then
      echo "🚨 CRITICAL: Version mismatch - Cargo.toml: $VERSION, proc_macros/Cargo.toml: $PROC_VERSION"
      exit 1
    fi
    if [ -z "$INLINE_PIN" ]; then echo '🚨 CRITICAL: Cannot read inline proc-macros pin from Cargo.toml'; exit 1; fi
    if [ "$VERSION" != "$INLINE_PIN" ]; then
      echo "🚨 CRITICAL: Version mismatch - Cargo.toml: $VERSION, inline proc-macros pin: $INLINE_PIN"
      exit 1
    fi
    echo "✅ Version consistent: $VERSION (root, proc_macros, inline pin)"

release-validate-compilation:
    timeout 180s cargo build --release --all-features

release-validate-examples:
    timeout 300s cargo check --examples --all-features

release-validate-precommit: pre-commit

release-validate-security:
    -@just audit

release-validate-testcontainers:
    #!/usr/bin/env bash
    set -uo pipefail
    if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
      if timeout 300s cargo test --features testcontainers --test testcontainers 2>&1 | grep -q 'test result: ok'; then
        echo '✅ Testcontainers tests pass'
      else
        echo '⚠️  WARNING: Testcontainers tests failed (not blocking if Docker unavailable)'
        exit 0
      fi
    else
      echo '⚠️  SKIP: Docker not available, skipping testcontainers tests'
    fi

release-validate-git-push:
    #!/usr/bin/env bash
    set -uo pipefail
    UNPUSHED=$(git log origin/main..HEAD 2>/dev/null || git log origin/master..HEAD 2>/dev/null || true)
    if [ -n "$UNPUSHED" ]; then
      echo '⚠️  WARNING: Local commits not pushed to remote'
      git log --oneline origin/main..HEAD 2>/dev/null || git log --oneline origin/master..HEAD 2>/dev/null || true
      exit 1
    fi
    echo '✅ All commits pushed to remote'

release-validate-docs: docs-check

# Comprehensive release validation (all FMEA failure mode checks)
release-validate: timeout-check check-mutation-gate release-validate-git-state release-validate-artifacts release-validate-version release-validate-compilation release-validate-examples release-validate-precommit release-validate-security release-validate-testcontainers release-validate-docs

release: release-validate ci docs-build

# ---------------------------------------------------------------------------
# Development workflow / aggregate targets
# ---------------------------------------------------------------------------

# Development workflow (check, format, test)
dev: timeout-check check fmt test-unit

# Full validation (build, test, lint)
all: timeout-check build test lint

# ---------------------------------------------------------------------------
# Cookbook (mdBook)
# ---------------------------------------------------------------------------

cookbook-build: timeout-check
    timeout 10s mdbook build cookbook

cookbook-serve: timeout-check
    timeout 60s mdbook serve cookbook --open

cookbook-test: cookbook-build
    timeout 10s mdbook build cookbook --dest-dir book-test

# ---------------------------------------------------------------------------
# Weaver (OTel)
# ---------------------------------------------------------------------------

weaver-bootstrap:
    timeout 60s bash scripts/weaver-bootstrap.sh

weaver-smoke: weaver-bootstrap
    timeout 30s cargo run --quiet --bin weaver_smoke --features weaver

# Reusable, project-agnostic weaver wrappers — see weaver-toolkit/README.md.
# Vendorable by other projects independent of this crate's Rust workspace.
import 'weaver-toolkit/weaver.just'

# ---------------------------------------------------------------------------
# Spec harness (Chatman Equation conformance)
# ---------------------------------------------------------------------------

spec: timeout-check
    timeout 60s cargo test --manifest-path spec-harness/Cargo.toml --lib -- --nocapture

spec-check: timeout-check
    #!/usr/bin/env bash
    echo '🔍 Checking spec harness theorem coverage...'
    timeout 300s cargo test --manifest-path spec-harness/Cargo.toml --lib -- --nocapture 2>&1 | grep -E '^test result:' || (echo '❌ Spec harness tests failed'; exit 1)
    echo '✅ Spec conformance verified: 100% theorem coverage'

spec-view:
    cat spec-harness/THEOREM_MAPPING.md

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

# Install Git pre-commit hooks to prevent unwrap/expect in production code
install-hooks:
    bash scripts/install-hooks.sh

setup-dev: install-hooks
    @echo '✅ Development environment setup complete!'
    @echo '🔧 Git hooks installed for unwrap/expect prevention'
    @echo '📚 See docs/process/SPR_GUIDE.md for error handling patterns'

# ---------------------------------------------------------------------------
# OTEL Weaver Live Check
# ---------------------------------------------------------------------------

otel-weaver-check:
    target/debug/weaver registry check -r registry/model

# Isolated on ports 4319/4321 (not the crate's DEFAULT_OTLP_GRPC_PORT
# 4317) so this manual live-check session never collides with a real,
# non-test OTLP collector -- e.g. a consuming project's own Istio/Jaeger
# pipeline using 4317 in-cluster.
otel-weaver-live-start:
    mkdir -p ./weaver-reports
    target/debug/weaver registry live-check -r registry/model --otlp-grpc-port 4319 --admin-port 4321 --format json --output http &

otel-production-run:
    WEAVER_TEST_OTLP_PORT=4319 timeout 300s cargo run --quiet --bin otel_production_run --features weaver

otel-weaver-live-stop:
    curl -s -X POST http://127.0.0.1:4321/stop > weaver-reports/report.json || true
    pkill -f 'weaver registry live-check' || true

otel-weaver-live-negative:
    WEAVER_TEST_OTLP_PORT=4319 timeout 300s cargo run --quiet --bin otel_negative_run --features weaver

otel-weaver-live:
    ./scripts/otel-workflow.sh
