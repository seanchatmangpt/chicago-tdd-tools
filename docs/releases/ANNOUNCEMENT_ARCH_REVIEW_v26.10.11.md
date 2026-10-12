# Architectural Review: Unified Release v26.10.11 — Announcement Artifact (DRAFT)

> **Status: DRAFT** — Date: 2026-10-11. Operator-issued release text (third release text in the
> v26.10.11 train), transcribed to the repo as the dated announcement artifact.
> Claims per PR1 audit (`~/.claude/plans/compressed-zooming-diffie.md`); known corrections:
> ash_pplan real version is 26.10.8-1, not 26.10.3; `NATO_STANAG_4107_DOSSIER.json` is generated
> by `just emit-dossier` (G1/G3 landing); `cq` is the operator alias of
> `ctdd_ledger dossier-inspect` (G9 landing). As-announced text below is transcribed untouched;
> corrections land as follow-up edits, never by silent rewrite. Per-claim verdicts in the
> Claim Status Ledger at the end (do not publish).

---

This release marks the structural consolidation of the Forward Deployment OS: probabilistic execution, unmetered host authority, and interpretive glue scripts are formally replaced by closed algebraic conservation laws.

Below is an architectural review of the release synchronization across the stack, mapping the implementation details against the core tenets of the platform:

---

## Architectural Review: Unified Release v26.10.11

```text
              ┌─────────────────────────────────────────────────────────┐
              │             User Specification (Pure RDF / .ttl)        │
              └────────────────────────────┬────────────────────────────┘
                                           │
                                           ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ ash_graphlaw (v26.10.8)                                                                │
│ • SHACL / N3 / Datalog evaluation isolated in graphlaw.wasm                            │
│ • Emits: AshGraphLaw.Admitted + Digest-Bound Semantic Evidence                         │
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
                                           ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ ash_pplan & ash_a2a (v26.10.3)                                                         │
│ • Compiles RDF P-Plan into Ash Reactor Directed Acyclic Graphs (DAGs)                  │
│ • Projects protocol-agnostic Agent Cards (JSON-RPC / SSE / gRPC)                       │
│ • Binds agent invocation to monotonic actuation leases (Φ_lease)                       │
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
                                           ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ BEAM Execution Layer & temprun (v26.10.11)                                             │
│ • In-Process WebAssembly leaf execution (zero OS daemon / no /var/run/docker.sock)     │
│ • Empty import set (imports == ∅): No ambient clock, filesystem, or sockets            │
│ • 2,000-reduction cooperative scheduling yielding to BEAM schedulers                   │
│ • Pure in-memory synthetic VFS & ggen-synthesized OTP process mailboxes                │
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
                                           ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ chicago-tdd-tools & wasm4pm / bcinr (v26.10.11)                                        │
│ • Evaluates execution logs against normative Petri nets & POWL trees                   │
│ • Branchless dual-64 (pn64) state replayer (≤12 cycles, hot-path zero allocations)     │
│ • NATO Courts (N01–N20): MILS isolation, Welch t-test timing (|t| < 4.5), SEU traps    │
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
                                           ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Cryptographic Settlement & Dossier Sealing                                             │
│ • IEEE OCEL 2.0 log stream ──► BLAKE3 Merkle Tree ──► ML-DSA-65 (FIPS 204) Signature  │
│ • Strict enforcement of: ΔS_admitted + Φ_lease ≡ R_receipt (mod Σ_PQC)                 │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Key Subsystem Validations

| Layer / Package | Version | Verification Invariant | Failure Mode |
| --- | --- | --- | --- |
| **`ash_graphlaw`** | `26.10.8` | SHACL shape conformity & N3 entailment via WASI sandboxed `graphlaw.wasm`. | Semantic admission reject; zero downstream compute scheduled. |
| **`ash_pplan` / `ash_a2a**` | `26.10.3` | P-Plan graph translation to Ash Reactor steps; signed capability cards with $\mathbf{\Phi}_{\text{lease}}$. | Lease quota breach; saga rolls back through strict compensation steps. |
| **`temprun`** | `26.10.11` | In-memory guest with `imports == ∅`; $\le 2{,}000$ reduction budget; in-memory ETS/actor routing. | Reduction limit or syscall attempt triggers immediate VM trap & heap zeroization. |
| **`wasm4pm` / `bcinr**` | `26.10.11` | Register-level token simulation via branchless `pn64` replayer ($\le 12$ clock cycles). | Model-Log mismatch: trajectory deviation immediately fails closed. |
| **`chicago-tdd-tools`** | `26.10.11` | NATO qualification matrix (`N01–N20`): MILS non-interference, Welch $t$-test ($\vert{}t\vert{} < 4.5$), SEU fault injection. | Non-zero exit code on compliance assertion; build pipeline halts; no receipt generated. |
| **Cryptographic Core** | FIPS 204 | BLAKE3 Merkle tree aggregation over OCEL 2.0 events, signed via ML-DSA-65. | Cryptographic verification failure under $\mathbf{\Sigma}_{\text{PQC}}$. |

---

## Verification and Operational Next Steps

1. **Verify Hex & Crates Manifests**
Confirm that all dependent mix and cargo definitions pin exact versions without floating bounds:

```text
* Elixir: {:ash_graphlaw, "26.10.8"}, {:temprun, "26.10.11"}, {:ash_pplan, "26.10.3"}, {:ash_a2a, "26.10.3"}
* Rust: chicago-tdd-tools = { version = "26.10.11", features = ["conformance", "nato"] }, wasm4pm = "26.10.11", bcinr-cmca = "26.10.11"
```

2. **Execute Clean-Room Conformance Run**
Run the compiled verification suite across all 20 defense courts to generate the signed dossier:

```bash
cargo test -p chicago-tdd-tools --release --features "conformance nato" -- --nocapture
```

3. **Inspect Output Receipt Dossier**
Validate the generated FIPS 204 attestation envelope:

```bash
cq inspect docs/nato/NATO_STANAG_4107_DOSSIER.json \
  --verify-pqc-signature ML-DSA-65 \
  --assert-merkle-root
```

---

## Claim Status Ledger (do not publish) — PR1 audit applied, 2026-10-11

As-announced transcription above is untouched; verdicts below are post-audit against the landed
state. Corrections to apply at publication: ash_pplan is 26.10.8-1 (not 26.10.3);
`NATO_STANAG_4107_DOSSIER.json` is generated by `just emit-dossier` (G1/G3 landing); `cq` is the
operator alias of `ctdd_ledger dossier-inspect` (G9 landing).

| Claim (table row / section) | Status | Evidence / Note |
| --- | --- | --- |
| `ash_graphlaw` 26.10.8 — SHACL/N3/Datalog via WASI `graphlaw.wasm` | UNBACKED (out-of-repo) | Out-of-repo per PR1; no CTT-side evidence surface. |
| `ash_pplan` / `ash_a2a` 26.10.3 — P-Plan→Reactor DAGs, leases | UNBACKED (out-of-repo) | Out-of-repo per PR1; version claim wrong — ash_pplan real version 26.10.8-1, not 26.10.3. |
| `temprun` 26.10.11 — `imports == ∅`, 2,000-reduction budget, ETS/actor routing | PARTIAL | PARTIAL per the Port-bridge truth: execution is `temprun_step` wasmi host binary via `packet: 4` BEAM Port bridge (wasmtime-CLI legacy path), not in-process BEAM Wasm; ETS/actor routing is roadmap. Zero-import and reduction-budget properties themselves are BACKED (guest.rs / guest_yield.rs). |
| `wasm4pm` / `bcinr` 26.10.11 — branchless `pn64` replayer ≤12 cycles | Version claims per PR1 | PR1 table 2: crates and versions verified out-of-repo; pn64 invariants BACKED in-repo (`src/conformance/pn64.rs`, `src/observability/ocel/ocel64_bridge.rs`). |
| `chicago-tdd-tools` 26.10.11 — NATO matrix N01–N20 (MILS, Welch, SEU) | BACKED | With scope note: N08, N12, N14, N15 run as strata-side courts, disclosed in `src/nato/README.md`; CTT implements N01–N07, N09–N11, N13, N16–N20. |
| Cryptographic Core — BLAKE3 Merkle over OCEL 2.0, ML-DSA-65 (FIPS 204) | BACKED | `src/ledger/signing/`; OCEL 2.0 ingest + BLAKE3 chain in `src/ledger/` / `src/observability/ocel/`. |
| Next step 1 — pinned manifests (ash_pplan/ash_a2a "26.10.3") | PARTIAL | Correction: `{:ash_pplan, "26.10.8-1"}`; exact-pin principle stands. |
| Next step 2 — clean-room conformance run command | BACKED (as given) | Standard cargo invocation against the landed `conformance`/`nato` feature gates. |
| Next step 3 — `cq inspect docs/nato/NATO_STANAG_4107_DOSSIER.json` | PARTIAL | `cq` = operator alias of `ctdd_ledger dossier-inspect` (G9 landing); the dossier JSON is generated by `just emit-dossier` (G1/G3 landing), not emitted by the test run directly. |
