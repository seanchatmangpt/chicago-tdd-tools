# Merger Matrix (Fortune-5 Validation Harness)

DRAFT — grounded only in files verified on disk 2026-10-11.

## Status

Phase B in progress. Three tiers: landed / in flight / roadmap.

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

The fixtures bind to landed kernels: the conformance kernel
(`src/conformance/` — loader, validate, session, pn64, net, report, guest,
guest_seal, guest_telemetry, guest_yield, bcinr, state, model, error), the
ledger (src/ledger/ — dossier, proof, receipt, signing), and the NATO courts
(`src/nato/`).

### In flight

Not present on disk as of 2026-10-11; cited for scoping only.

- Phase POWL models for the merger phases (signing → closing → integration);
  the kernel consumer exists (`src/conformance/session.rs`:
  `ConformanceSession::from_powl_file`), the merger phase models do not.
- Merger OCEL fixtures (OCEL64 streams; bridge modules landed at
  `src/observability/ocel/ocel64_bridge.rs`, merger-specific streams do not).
- A merger-guests crate (the hermetic guest crate that exists is
  `crates/ctt-wasm-guest` — general, not merger-specific).
- BEAM agents driving the sagas (only doc-level BEAM references landed:
  `src/conformance/guest_yield.rs` cooperative-yield protocol vs. BEAM
  scheduler; no BEAM agent code in this repo).
- N08/N12/N14/N15 court ports into `src/nato/` (module list verified; the
  ports are not present).

### Roadmap

- Pillar certs for pillars not yet covered by landed courts.
- Master seal over the full matrix.

## Architecture flow

Each arrow annotated with landing status (E-M16 probe).

```
Turtle RDF (examples/merger_matrix/**.ttl/.n3)        [LANDED]
  → graphlaw membrane (graphlaw.wasm, external pinned artifact; no graphlaw code in this repo)  [EXTERNAL, PINNED per E-M16]
  → pplan sagas (no pplan code in this repo)          [EXTERNAL / IN FLIGHT]
  → temprun leaves (crates/ctt-wasm-guest; built via `just guest-build`)  [LANDED]
  → CTT conformance (src/conformance/, replayed against OCEL64 via ConformanceSession)  [LANDED]
  → sealed dossier (src/ledger/dossier.rs seal_dossier / verify_sealed_file; emitted via `just emit-dossier`)  [LANDED]
```

Arrow-by-arrow:

1. **Turtle RDF → graphlaw membrane** — fixtures are the hand-authored source
   of truth (examples/merger_matrix/README.md). graphlaw.wasm is real and
   pinned per the E-M16 probe, but lives outside this repo; no graphlaw source
   in-repo.
2. **graphlaw → pplan sagas** — saga orchestration is external (ash_pplan);
   in-repo, nothing consumes the fixtures as sagas yet. IN FLIGHT.
3. **pplan → temprun leaves** — the WASM guest
   (`crates/ctt-wasm-guest`) is landed and exercised by `guest-build`,
   `guest-audit` (zero-import audit court, `src/nato/guest_audit.rs`), and
   `guest-test` (native-ABI contract + fault-injection suites). The a2a lease
   is the SagaControl epoch per E-M16 (external to this repo).
4. **temprun leaves → CTT conformance** — `src/conformance/` compiles POWL
   models and replays them against OCEL64 streams
   (`ConformanceSession::from_powl_file`). Landed kernel, in-flight merger
   models/streams.
5. **CTT conformance → sealed dossier** — `just emit-dossier` gates on green
   courts, emits `docs/nato/NATO_STANAG_4107_DOSSIER.json` via
   `ctdd_ledger dossier-emit`; verified by
   `ctdd_ledger dossier-inspect ... --verify-pqc-signature ML-DSA-65 --assert-merkle-root`.

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

## Commands that exist

All recipes verified in `Justfile`:

| Command | What it does |
|---|---|
| `just emit-dossier` | court tests → receipts → `dossier-emit` (typed refusal on empty receipts dir) |
| `just dossier-verify` | `dossier-inspect` with PQC (ML-DSA-65) + Merkle-root assertion |
| `just ctdd-ledger` | builds the `ctdd_ledger` release binary |
| `just verify-ledger SOURCE` | `ctdd_ledger validate` on a source |
| `just guest-build` | builds the WASM guest (wasm32-unknown-unknown) |
| `just guest-audit` | zero-import audit court over the built artifact |
| `just guest-test` | native-ABI contract + fault-injection suites |

No merger-specific commands exist yet. Nothing in the Justfile consumes
`examples/merger_matrix/` — the fixtures are validated by external validators
(`pyshacl`, `eye`) per the example README until the in-flight items land.

## See Also

- `examples/merger_matrix/README.md`
- `docs/features/NATO_COURTS.md`
- `docs/features/CONFORMANCE.md`
- `docs/features/LEDGER.md`
- `src/nato/README.md` (receipt contract)
