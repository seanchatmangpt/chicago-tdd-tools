# Release Notes — chicago-tdd-tools v26.10.12 (2026-10-11)

> Train doc for the v26.10.11 → v26.10.12 release train; filename retained.
> Workspace version is 26.10.12 (Cargo.toml @ bump lane B1,
> `release-validate-version` green).

## Headline

Trajectory conformance becomes a shipped capability: a vendored branchless `pn64` Petri-net kernel, a POWL normative-model layer with a fail-closed loader, `ConformanceSession` trajectory verification with cooperative reduction budgets, an OCEL 2.0 ingestion bridge, a cryptographic ledger (320-byte court receipts / 352-byte chained records, BLAKE3 Merkle dossiers, ML-DSA-65 sealing), a 20-court NATO assurance matrix (N01–N20 complete CTT-side: N08 hamming, N12 tick, N14 thrash, N15 liveness ports landed in `src/nato/`), a hermetic zero-import WASM guest, and the `ledger` domain surface (the `ctdd_ledger`/`cq` CLI binaries were excised post-train; CLI verbs re-landed in the external `affi` binary (Phase D M2, behind the `ctdd` feature; see the CHANGELOG `[26.10.12]` notes).

Related ecosystem artifacts this train: the Strata `temprun` integration (BEAM `gen_statem` worker + wasmi host bridge) and the DRAFT announcements under `docs/releases/ANNOUNCEMENT_*.md` (claims audited; publication is an operator act).

## Added

- `conformance` feature: `src/conformance/` — pn64 kernel (ported from strata-protocol @ 0de13a3), POWL model/loader/validator, `ConformanceSession` (from_powl_file, step_budget → `YieldStatus`), reports with typed deviation taxonomy, guest ABI modules (`guest*`).
- `ocel-ingest{-json,-xml}` features: idmap/ocel64 bridges, JSON/XML importers (attribute-lossy at the packed tier, disclosed).
- `ledger` feature: `CourtReceiptV1` wire (320/352 B, deserialize-time verification), BLAKE3 chains reusing `Blake3ReceiptEntry`, Merkle dossier roots + inclusion proofs, ML-DSA-65 signer/sealer/keys (keys never committed). Library-only this train: the typed exit-code vocabulary (10–18) lives in `src/ledger` as `LedgerExitCode`, ready for the future `affi` CLI; the `ctdd_ledger`/`cq` CLI binaries were excised post-train (see the CHANGELOG `[26.10.12]` notes).
- `nato` feature: court framework (positive-control gate first; `Refused` receipts, never adjudication past a surviving mutant), courts N01–N20 (complete registry, 20/20 CTT-side — N08 hamming, N12 tick, N14 thrash, N15 liveness ported post-train) with Honest-Scoping records, LCG determinism, torn-write CAS-ring subject port.
- `wasm-guest` feature + `crates/ctt-wasm-guest`: zero-import cdylib (audit-proven: `imports == []`), reduction-yield protocol, guest-produces/host-seals receipt division, fault-injection courts.
- Test suites: conformance (37), ocel-ingest (11), ledger laws (9) + sealing laws, nato fast tier (14), guest ABI (13) + fault-injection (9+2 tier-3), compile-fail trybuild suite (5 cases), prelude-surface (46), cli-proof (34), observability-fixtures (12).
- CI: `assurance.yml` (path-gated PR jobs + nightly full tier + guest job); Justfile recipe set (test-conformance, test-nato, nato-courts-full, verify-ledger, verify-hermetic, check-mutation-gate, emit-dossier, guest-*).
- Docs: `docs/features/{CONFORMANCE,NATO_COURTS,LEDGER,MERGER_MATRIX}.md`, E6-audited README sections, DRAFT announcements with claim ledgers.

## Changed

- Docs wave: 15 dated planning docs archived (`docs/archive/2026-10/`, `cookbook/archive/`); fabricated API claims removed; `cargo make` → `just`; stale counts corrected against measured ground truth (38k LOC, 500+ tests).
- Dependency hygiene for hermeticity: `tokio` optional (feature-gated), `uuid` optional (`uuid-v4`), `chrono` `wasmbind` off — the wasm guest stays zero-import.

## Fixed

- ctdd_ledger typed exit codes + `ChainRecord` stored-hash offset (320..352).
- OCEL64 decode: case-timeline accumulation, missing `ReceiptMissing` activity arm.
- Receipt layout law: chain hash at [320..352], zero trailer [304..320).
- Key-id law unified to BLAKE3 (was SHA-256/BLAKE3 split).
- N07 endless-chain harness reconciled to the kernel's fixpoint law (proved, not guessed).

## Verification receipts (fresh runs, 2026-10-11 post-convergence)

- `just test-unit`: exit 0 — 1041 lib tests passed, 18 ignored (tier-3 by design); plus merger-guests 122, an integration suite at 38 passed / 2 ignored, and 146 doc tests.
- `cargo test --features all-assurance --lib`: 777 passed, 18 ignored.
- `cargo test --features nato --lib`: 777 passed, 18 ignored (of which `nato::`-filtered: 135 passed).
- `cargo test -p merger-guests`: 122 + 14 + 3 passed, 1 ignored — GREEN.
- `cargo test --test merger_conformance_tests --features conformance,ocel-ingest-json`: 25 passed, 0 failed, 0 ignored (all phase blocks landed and green through the fixture bridge).
- Zero-import proof: `{"imports":[],"status":"PASS"}` on a fresh wasm32 release build of `ctt_wasm_guest` (79 KB), re-verified by the `zero_import` census tests (3/3 green).
- Fault-injection 9 (SEU/torn/seal-boundary, SDC=0) — as receipted at train time, not re-run in this refresh.
- `release-validate-version`: "✅ Version consistent: 26.10.11" (as-of-date at train time; re-run green at 26.10.12 after bump lane B1).

## Known scope boundaries (honest disclosures)

- All 20 courts CTT-side: registry 20/20 (`src/nato/registry.rs` `COURT_NAMES`); N08 (Hamming profile, partial-scale), N12 (clock-glitch, wall-monotonic surrogate), N14 (CAST-32A thrash), N15 (liveness) are post-train ports. The `src/nato/README.md` scoping note has been updated post-port: 20/20 delivered CTT-side; no Erlang/OTP tier for N08/N12/N14/N15 is planned or exists in strata (strata's Erlang tiers are N21–N24) — CTT's Rust courts are the only implementation.
- N17 is PIPELINE-KAT (self-pinned vectors), not official ACVP; drop-in upgrade path documented.
- N19 reproducible-build: same-host 2-run identity; 3-host NOT-RUN.
- WASM guest ABI v1; the temprun bridge re-base (errno-packed i64 → out-buffer i32) is a tracked work order (`docs/jira/v26.10.11/000-temprun-guest.md`).
- Workspace bump landed (a166e25 → 26.10.11; superseded by the 26.10.12 bump, lane B1); crates.io publication and tag await explicit operator go.
