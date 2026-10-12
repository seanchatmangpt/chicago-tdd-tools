# NATO Assurance Matrix (src/nato)

Module documentation for the NATO high-assurance court suite living in
`src/nato/`. Ports the court set defined in
`/Users/sac/strata/docs/nato/NATO_N01_N20.md` (N01–N20 Rust courts) onto
`chicago-tdd-tools`' type-level, poka-yoke discipline. Evidence classes
targeted: EAL7 / DO-178C Level-A / DO-333 / STANAG 4107.

## The Honest Scoping Law

Spec figures (2^64 SMT, 10M dudect, 100M SEU) are **evidence-class
targets** — lanes implement the bounded/calibrated equivalent and record
the mapping from spec mandate to executed proof in each receipt. A court
that cannot meet the spec scale **says so** and delivers the strongest
sound substitute (exhaustive small-domain + calibrated sampling + static
analysis), **never a silent weakening**.

Every receipt therefore carries three fields for scope: `spec_scale`
(what the mandate asks), `delivered_scale` (what actually ran), and
`follow_up` (documented path to full scale, or `null` when the delivered
court is itself exhaustive/complete for its claim).

## Court matrix (N01–N20)

Provenance: every court below is ported from the "Rust courts (N01-N20)"
section of `/Users/sac/strata/docs/nato/NATO_N01_N20.md`.

| Court | Name | Pillar | Delivered scope |
|---|---|---|---|
| N01 | MILS non-interference | isolation | Bounded 2^16 exhaustive small-domain + 10^7 dual-tenant sweep proving `dState(TA)/dInput(TB) == 0`. Full Kani/Z3 2^64 NOT-SHIPPED; documented follow-up. |
| N02 | Coordinate orthogonality | isolation | Spec-exact: exhaustive bit-spill census — every field write perturbs no neighbor field. |
| N03 | Bell-LaPadula / Biba lattice | isolation | Exhaustive over the 25x65,536-entry transition-table space: no-write-down, no-read-up. |
| N04 | Zero ambient authority | authority | Source-tier symbol/AST audit of the final artifact: zero env/fs/unmetered-clock imports on state paths. Binary-tier audit deferred (disclosed). |
| N05 | dudect timing | timing | Two tiers: smoke (small calibrated N in CI) and full (Welch t-test, `|t| < 4.5`, valid-vs-refusal timing; spec 10M, delivered calibrated-N disclosed). |
| N06 | Branchless disassembly | timing | `.s`/objdump census over the compiled artifact: zero conditional branches on state paths; unmatched symbols recorded as typed skips, never silent passes. |
| N07 | Cache side-channel | timing | Prime+Probe-style concurrent eviction vs sequencer; latency invariance is **reported, not gated** (host-noise floor disclosed in receipt). |
| N09 | SEU single-bit | resilience | Prefix-stream tiers: bounded randomized single-bit flips during ALU exec; 100.000% fail-closed-or-halt, 0% SDC on the executed prefix; calibrated-N disclosed. |
| N10 | SEU double-bit | resilience | Adjacent-lane double flips vs forbidden-bit check; same prefix-stream tiering as N09. |
| N11 | Torn-write / bus-disconnect | resilience | Fuzz over the ported 64B CAS ring: every torn write recovers to a valid prior state. |
| N13 | WCET | timing | Static per-opcode table + calibrated measurement. Units are `ticks≈ns on aarch64` — never "cycles"; the measured table is published in the receipt. |
| N16 | Stack bound | resource | Zero-recursion proof (call-graph analysis) + 32KiB ladder: depth-tiered deepest-frame bound, total <= 32KiB. |
| N17 | ML-DSA-65 KAT | crypto | PIPELINE-KAT: known-answer tests against the signer. **NOT the official ACVP corpus**; the drop-in path to official vectors is documented in the receipt. |
| N18 | Zero-entropy diagnostics | crypto | Entropy refusal court: Shannon entropy of all emitted error strings == 0 bits about internal state; strings that leak state are refused at construction. |
| N19 | Reproducible build | supply chain | 2-run same-host SHA-256 identity of the build artifact. 3-host cross-environment identity NOT-RUN (disclosed follow-up). |
| N20 | Dossier generator | supply chain | Aggregates N01–N19 receipts into one dossier, sealed with ML-DSA-65 over the manifest digest. |

Note on numbering: N08, N12, N14, N15 remain strata-side lanes (Hamming
profile, clock-glitch, CAST-32A thrash, TLC liveness) and are not part of
this module's delivered set; receipts exist only for the courts listed
above.

## The positive-control gate law

Every court must prove it can fail: a **positive control** (an injected
defect — a mutant, a leaked string, a torn write) must drive the court to
a `Refused` receipt. A court that **cannot refuse cannot admit**: if a
mutant survives the court, the court's verdict is void for that run and
the run records `Refused(positive_control)` — we never adjudicate past a
surviving mutant. This is the generate-and-kill discipline applied to
the assurance layer itself.

## Tiering rules

| Tier | Where | When it runs |
|---|---|---|
| tier-1 | `test-unit` (inline `#[cfg(test)]` / `tests/`) | Every `just test-unit`, every commit |
| tier-2 | `test-nato-tier2` (named subset: `n16_workload_100k_transitions_on_32kib_thread`, `n07_conservation_under_eviction_storm_full`) | Explicit invocation; nightly / pre-release |
| tier-3 | `nato-courts-full` (release profile) | Release gate only |

Rule: a court's tier is fixed by its runtime cost, not its importance —
the tier-1 slice must be a sound (weaker but true) prefix of the tier-2/3
run, and the mapping is recorded in the receipt so no tier silently
claims another tier's strength.

## Receipt contract

Each court writes a `CourtReceipt` as JSON to `docs/receipts/N{01..20}.json`
(one file per court). Required fields:

```json
{
  "court": "N01",
  "verdict": "Admitted",
  "spec_scale": "2^64 SMT dual-tenant state space",
  "delivered_scale": "2^16 exhaustive + 10^7 randomized",
  "follow_up": "Kani/Z3 full-domain harness (not shipped)",
  "positive_control": {"mutant": "N01-leak-inject", "verdict": "Refused"},
  "subject": {"crate": "chicago-tdd-tools", "sha256": "..."},
  "command": "just test-nato-tier2",
  "exit": 0,
  "timestamp_utc": "2026-10-12T00:00:00Z"
}
```

`verdict` is one of `Admitted` | `Refused` | `NotRun`. A missing receipt
file degrades the N20 dossier to `Refused`, never a silent pass.
