# Merger Matrix (Fortune-5 Validation Harness)

Landed state as of 2026-10-11, post-Wave-2 (MX3-MX7 phase fixtures verified on
disk at write time; every cited file existence-checked).

## Status

Post-Wave-2. Three tiers: landed / in flight / roadmap.

### Landed

Fixtures: `examples/merger_matrix/` — a self-contained semantic fixture set
modeling a stock merger (`ex:AcmeCorp` acquiring `ex:BetaHoldings` via
`ex:MergerSub`), hand-authored source of truth, no `owl:imports`, closed-world
SHACL (`sh:closed true`).

| Path | Layer |
|---|---|
| `examples/merger_matrix/shapes/merger-core.shacl.ttl` | deal topology, closed class |
| `examples/merger_matrix/shapes/deal-terms.shacl.ttl` | exchange ratio, price, quorum |
| `examples/merger_matrix/shapes/materiality.shacl.ttl` | MAE thresholds |
| `examples/merger_matrix/shapes/party.shacl.ttl` | Target/Acquirer/Sub roles |
| `examples/merger_matrix/rules/vote-threshold.n3` | 66.67% supermajority |
| `examples/merger_matrix/rules/hhi-bands.n3` | DOJ/FTC 2023 HHI bands |
| `examples/merger_matrix/rules/netting.n3` | mutual netting |
| `examples/merger_matrix/deontic/*.ttl` | LKIF Obligation/Permission/Violation/Remedy |
| `examples/merger_matrix/data/deal.ttl`, `data/parties.ttl` | deal + party data |
| `examples/merger_matrix/tax/stepup.ttl`, `tax/withholding.ttl` | tax basis step-up, withholding |
| `examples/merger_matrix/settlement/fractional.ttl`, `settlement/netting_sets.ttl` | fractional settlement, netting sets |
| `examples/merger_matrix/collateral/simm.ttl` | SIMM collateral |
| `examples/merger_matrix/contracts/merger-agreement.ttl`, `contracts/credit-agreement.ttl` | contract layer |
| `examples/merger_matrix/loans/portfolio.ttl` | loan portfolio |
| `examples/merger_matrix/capital/structures.ttl` | capital structures |
| `examples/merger_matrix/analytics/hhi.ttl`, `analytics/simm-outputs.ttl` | analytics |
| `examples/merger_matrix/models/master.powl` + `phase{1..10}_*.powl` (11 files) | POWL phase models, all 10 phases |
| `examples/merger_matrix/conformance/*.ocel.json` (19 files) | OCEL64 streams: phases 1-8 positive+violation pairs, phase 9 audit + phase 10 seal (positive-only), master trace |
| `crates/merger-guests/` | hermetic WASM guest crate: 8 leaf kernels behind a uniform `mgi_*` stateless ABI |
| `src/nato/hamming.rs` | N08 Hamming weight/distance profile court (port; partial-scale honest disclosure) |
| `src/nato/tick.rs` | N12 clock-glitch resilience court (wall-monotonic surrogate; partial-scale disclosure) |
| `src/nato/thrash.rs` | N14 CAST-32A multi-core interference court |
| `src/nato/liveness.rs` | N15 liveness court port |
| `src/nato/registry.rs` | `COURT_NAMES` N01..N20 — registry complete, 20/20 CTT-side |
| `tests/merger_conformance_tests.rs` | merger conformance suite (W1): contract pipeline + phase courts (all phase blocks landed; fresh run 25 passed / 0 failed / 0 ignored — every phase golden and violation replays through the fixture bridge) |
| `src/conformance/` (loader, validate, session, pn64, net, report, guest, guest_seal, guest_telemetry, guest_yield, bcinr, state, model, error) | conformance kernel |
| `src/ledger/` (dossier, proof, receipt, signing) | ledger |

### In flight

- merger-guests WASM artifact build — `crates/merger-guests` is landed as
  source (workspace member, `Cargo.toml` `members` line 463); the wasm32
  cdylib artifact itself is not built in this checkout.
- temprun ABI re-base — merger-guests targets a uniform `mgi_*` stateless
  ABI; re-basing onto the temprun leaf ABI exercised by
  `crates/ctt-wasm-guest` (`just guest-build`/`guest-audit`/`guest-test`) is
  not landed.
- BEAM live-court execution — BEAM-side agents driving the sagas; only
  doc-level BEAM references landed (`src/conformance/guest_yield.rs`
  cooperative-yield protocol vs. BEAM scheduler); no BEAM agent code in this
  repo.

### Roadmap

- Pillar certs for pillars not yet covered by landed courts.
- Master seal over the full matrix.

## Architecture flow

Each arrow annotated with landing status (E-M16 probe).

```
Turtle RDF (examples/merger_matrix/**.ttl/.n3)        [LANDED]
  → graphlaw membrane (graphlaw.wasm, external pinned artifact; no graphlaw code in this repo)  [EXTERNAL, PINNED per E-M16]
  → pplan sagas (no pplan code in this repo)          [EXTERNAL / IN FLIGHT]
  → temprun leaves (crates/ctt-wasm-guest; built via `just guest-build`; merger-specific leaves in crates/merger-guests, artifact build in flight)  [LANDED / IN FLIGHT]
  → CTT conformance (src/conformance/, replayed against OCEL64 via ConformanceSession; merger suite at tests/merger_conformance_tests.rs)  [LANDED]
  → sealed dossier (src/ledger/dossier.rs seal_dossier / verify_sealed_file; emitted via `just emit-dossier`)  [LANDED]
```

Arrow-by-arrow:

1. **Turtle RDF → graphlaw membrane** — fixtures are the hand-authored source
   of truth (examples/merger_matrix/README.md). graphlaw.wasm is real and
   pinned per the E-M16 probe, but lives outside this repo; no graphlaw source
   in-repo.
2. **graphlaw → pplan sagas** — saga orchestration is external (ash_pplan);
   in-repo, nothing consumes the fixtures as sagas yet. IN FLIGHT.
3. **pplan → temprun leaves** — the general WASM guest
   (`crates/ctt-wasm-guest`) is landed and exercised by `guest-build`,
   `guest-audit` (zero-import audit court, `src/nato/guest_audit.rs`), and
   `guest-test` (native-ABI contract + fault-injection suites). The
   merger-specific guest crate `crates/merger-guests` is landed as source with
   8 leaf kernels; its wasm32 artifact build is in flight. The a2a lease is
   the SagaControl epoch per E-M16 (external to this repo).
4. **temprun leaves → CTT conformance** — `src/conformance/` compiles POWL
   models and replays them against OCEL64 streams
   (`ConformanceSession::from_powl_file`). Kernel landed; all 10 merger phase
   models and their OCEL fixtures landed post-Wave-2; the W1 suite
   (`tests/merger_conformance_tests.rs`) drives the pipeline.
5. **CTT conformance → sealed dossier** — `just emit-dossier` gates on green
   courts, emits `docs/nato/NATO_STANAG_4107_DOSSIER.json` via
   the `affi` `dossier emit` verb (landed in `~/affidavit`, Phase D M2;
   routed via `just emit-dossier`); verified by
   `affi dossier inspect ... --verify-pqc-signature ML-DSA-65 --assert-merkle-root`
   (`just dossier-verify`).

## The execution equation (ΔS + Φ ≡ R)

The harness enforces "state change + proof ≡ receipt" at the seal boundary.
Landed enforcement points:

- **Seal-time verification** (E-M19): every sealed dossier file carries one
  352-byte court record; `verify_sealed_file` checks the sealed output against
  the local key store, and `dossier-inspect --verify-pqc-signature ML-DSA-65
  --assert-merkle-root` re-derives the Merkle root over the sealed court
  records. Without the proof, the dossier is not a receipt.
- **Emit gating**: `emit-dossier` refuses (typed error) when the receipts dir
  is empty — a dossier cannot exist without per-court receipts
  (`CTT_EMIT_RECEIPTS=1` in-process court receipts).
- **Honest v1/v2 scoping**: v1 = in-repo seal boundary only (ledger + NATO
  courts). v2 (not landed here): lease/epoch verification (SagaControl epoch
  per E-M16) inside the conformance kernel itself; no lease/epoch code exists
  in `src/ledger/` or `src/conformance/guest_seal.rs` as of 2026-10-11.
- **Honest fixture scoping (W1, asserted as typed courts)**: the landed OCEL
  fixtures carry POWL names in `ocel:activity` with no
  `attributes["powl"]`, so `import_ocel_json` refuses them with a typed
  `ImportError::MissingField` — pinned by `import_refuses_landed_fixtures_typed`;
  phases 1 and 2 exceed the linear net's 16-place budget (`TooManyPlaces`),
  so no golden replay exists for them (the refusal is the court); phase 5's
  landed golden double-fires loop atoms and faults mid-trace (the
  model-derived golden is the conforming trajectory); phase 7's fixtures use
  activity names absent from the model's declaration block (first replayed
  event `Undeclared`); phases 9 and 10 landed positive-only (no violation
  fixtures yet). The W1 suite's phase-block coverage was written against the
  pre-Wave-2 fixture set; the post-Wave-2 phase 3/6/8/9/10 fixtures are not
  yet covered by suite courts.

## Commands that exist

All recipes verified in `Justfile`:

| Command | What it does |
|---|---|
| `just emit-dossier` | court tests → receipts → `affi dossier-emit` (typed refusal on empty receipts dir) |
| `just dossier-verify` | `affi dossier-inspect` with PQC (ML-DSA-65) + Merkle-root assertion |
| `just verify-ledger SOURCE` | `affi validate` on a source |
| `just guest-build` | builds the WASM guest (wasm32-unknown-unknown) |
| `just guest-audit` | zero-import audit court over the built artifact |
| `just guest-test` | native-ABI contract + fault-injection suites |

Merger-specific Justfile recipes exist: `merger-guest-build` (wasm32 cdylib
for `crates/merger-guests`), `merger-guest-test` (`cargo test -p
merger-guests`), and `merger-conformance` (the W1 suite: `cargo test --test
merger_conformance_tests --features conformance,ocel-ingest-json`; fresh run
25 passed / 0 failed / 0 ignored — all phase blocks green). The fixtures themselves are additionally validated by
external validators (`pyshacl`, `eye`) per the example README.

## See Also

- `examples/merger_matrix/README.md`
- `docs/features/NATO_COURTS.md`
- `docs/features/CONFORMANCE.md`
- `docs/features/LEDGER.md`
- `src/nato/README.md` (receipt contract)
