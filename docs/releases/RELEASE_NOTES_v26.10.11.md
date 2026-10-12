# Release Notes — chicago-tdd-tools v26.10.11 (2026-10-11)

## Headline

Trajectory conformance becomes a shipped capability: a vendored branchless `pn64` Petri-net kernel, a POWL normative-model layer with a fail-closed loader, `ConformanceSession` trajectory verification with cooperative reduction budgets, an OCEL 2.0 ingestion bridge, a cryptographic ledger (strict 352-byte court receipts, BLAKE3 Merkle dossiers, ML-DSA-65 sealing), a 16-court NATO assurance matrix (N01–N07, N09–N11, N13, N16–N20; N08/N12/N14/N15 land strata-side), a hermetic zero-import WASM guest, and the pure-Rust `ctdd_ledger` / `cq` CLI binaries.

Related ecosystem artifacts this train: the Strata `temprun` integration (BEAM `gen_statem` worker + wasmi host bridge) and the DRAFT announcements under `docs/releases/ANNOUNCEMENT_*.md` (claims audited; publication is an operator act).

## Added

- `conformance` feature: `src/conformance/` — pn64 kernel (ported from strata-protocol @ 0de13a3), POWL model/loader/validator, `ConformanceSession` (from_powl_file, step_budget → `YieldStatus`), reports with typed deviation taxonomy, guest ABI modules (`guest*`).
- `ocel-ingest{-json,-xml}` features: idmap/ocel64 bridges, JSON/XML importers (attribute-lossy at the packed tier, disclosed).
- `ledger` feature: `CourtReceiptV1` wire (320/352 B, deserialize-time verification), BLAKE3 chains reusing `Blake3ReceiptEntry`, Merkle dossier roots + inclusion proofs, ML-DSA-65 signer/sealer/keys (keys never committed), `ctdd_ledger` + `cq` bins with typed exit codes 10–18.
- `nato` feature: court framework (positive-control gate first; `Refused` receipts, never adjudication past a surviving mutant), courts N01–N07/N09–N11/N13/N16–N20 with Honest-Scoping records, LCG determinism, torn-write CAS-ring subject port.
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

## Verification receipts (this train)

- `just check` / `just lint` / `just test-unit`: exit 0 — 991 lib tests passed, 13 ignored (tier-3 by design).
- `cargo test --features all-assurance --lib`: 735 passed.
- Zero-import proof: `{"imports":[],"status":"PASS"}` on `target/wasm32-unknown-unknown/release/ctt_wasm_guest.wasm` (304 KB).
- Courts: 735-run assurance lib suite green; nato fast tier 106+14; fault-injection 9 (SEU/torn/seal-boundary, SDC=0).
- `release-validate-version`: "✅ Version consistent: 26.10.11".

## Known scope boundaries (honest disclosures)

- 16 of 20 courts CTT-side; N08/N12/N14/N15 land strata-side (`src/nato/README.md`).
- N17 is PIPELINE-KAT (self-pinned vectors), not official ACVP; drop-in upgrade path documented.
- N19 reproducible-build: same-host 2-run identity; 3-host NOT-RUN.
- WASM guest ABI v1; the temprun bridge re-base (errno-packed i64 → out-buffer i32) is a tracked work order (`docs/jira/v26.10.11/000-temprun-guest.md`).
- Tag `v26.10.11` and crates.io publication await explicit operator go.
