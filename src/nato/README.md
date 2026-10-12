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

Every receipt therefore carries three fields for scope: `spec_mandate`
(what the mandate asks), `delivered_evidence` (what actually ran), and
`bound` (documented path to full scale, or `null` when the delivered
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
| N08 | Hamming weight/distance profile | security | HIGH (`0xFFFF`) vs LOW (`0x0000`) payload-class kernel chains over the PN64 committed marking, identical LCG seed; tier-1 bin-diff gate + tier-2 100k-samples-per-class (`--ignored`); input-independent Hamming profile = no power-analysis leak. `src/nato/hamming.rs`. |
| N09 | SEU single-bit | resilience | Prefix-stream tiers: bounded randomized single-bit flips during ALU exec; 100.000% fail-closed-or-halt, 0% SDC on the executed prefix; calibrated-N disclosed. |
| N10 | SEU double-bit | resilience | Adjacent-lane double flips vs forbidden-bit check; same prefix-stream tiering as N09. |
| N11 | Torn-write / bus-disconnect | resilience | Fuzz over the ported 64B CAS ring: every torn write recovers to a valid prior state. |
| N12 | Clock-glitch | resilience | `HwTick::mono_checked` monotonicity over jittered/stuck/huge-jump tick sources; jitter-injection regression refused, forward chains admit; wedged backstep clock caught by positive control (honest host note: no counter register on this host). Tier-3 calibration `--ignored`. `src/nato/tick.rs`. |
| N13 | WCET | timing | Static per-opcode table + calibrated measurement. Units are `ticks≈ns on aarch64` — never "cycles"; the measured table is published in the receipt. |
| N14 | CAST-32A thrash | resilience | 4-producer/4-consumer MPMC pipeline vs storm/spinner adversaries on the landed 64B CAS ring; typed `ConsensusError` refusals only (zero SDC, zero loss), watchdog liveness leg, run-to-run conservation determinism. Tier-1 small legs + tier-3 `n14_cast32a_thrash_full` (`--ignored`). `src/nato/thrash.rs`. |
| N15 | TLC liveness | proof | Starvation-freedom of the RONR admission surface: exhaustive reachable-marking enumeration + round-robin fair scheduler with typed `StarvationMonitor` (enabled-then-fired within TOTAL/32); unfair-scheduler counterexample must fire. TLC/Erlang itself NOT-SHIPPED (disclosed in `ScopingRecord`). `src/nato/liveness.rs`. |
| N16 | Stack bound | resource | Zero-recursion proof (call-graph analysis) + 32KiB ladder: depth-tiered deepest-frame bound, total <= 32KiB. |
| N17 | ML-DSA-65 KAT | crypto | PIPELINE-KAT: known-answer tests against the signer. **NOT the official ACVP corpus**; the drop-in path to official vectors is documented in the receipt. |
| N18 | Zero-entropy diagnostics | crypto | Entropy refusal court: Shannon entropy of all emitted error strings == 0 bits about internal state; strings that leak state are refused at construction. |
| N19 | Reproducible build | supply chain | 2-run same-host SHA-256 identity of the build artifact. 3-host cross-environment identity NOT-RUN (disclosed follow-up). |
| N20 | Dossier generator | supply chain | Aggregates N01–N19 receipts into one dossier, sealed with ML-DSA-65 over the manifest digest. |

Note on numbering (E-M18): all 20 courts are delivered CTT-side and the
matrix is 20/20 (`src/nato/registry.rs` `COURT_NAMES`). N08
[Hamming profile](hamming.rs), N12 [clock-glitch](tick.rs), N14
[CAST-32A thrash](thrash.rs), N15 [liveness](liveness.rs) landed
CTT-side; the Erlang/OTP tiers remain strata-side. These four courts'
receipts are real-run emitted (via the `CTT_EMIT_RECEIPTS=1` emitters),
unlike the synthetic control-stub receipts described under the
[receipt contract](#receipt-contract).

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

Each court writes a `CourtReceipt` (see `src/nato/receipt.rs`) as JSON to
`docs/receipts/N{01..20}.json` (one file per court) when the opt-in gate
`CTT_EMIT_RECEIPTS=1` is set (hermetic by default otherwise).

`verdict` is one of `Pass` | `Fail` | `Refused`. A receipt carries:
`receipt_id`, `court_id`, `pillar`, `name`, `subject_sha`, `verdict`,
`metrics`, `positive_control`, `scoping` (a `ScopingRecord`), `digest`,
and `timestamp_ms`. `scoping` maps the spec mandate to the delivered
evidence: `spec_mandate`, `delivered_evidence`, `is_full_spec_mandate`,
and `bound`.

The positive-control field is one of:

- `MutantCaught { mutant_id, detection_channel }` — the injected defect
  was caught, naming the channel that caught it;
- `MutantSurvived { mutant_id }` — the court is void for that run;
- `GateRefused { reason }` — the court refused at its gate.

A real emitted receipt, quoted verbatim from `docs/receipts/N08.json`:

```json
{
  "receipt_id": "cr-N08-e6c8d884e346e9e6",
  "court_id": "N08",
  "pillar": "Security",
  "name": "N08",
  "subject_sha": "b92e311b8d24773c28065b8d4fe522204b9d081f280cfcb7323d44d95cdfa776",
  "verdict": "Pass",
  "metrics": [
    [
      "n_per_class",
      {
        "U64": 10000
      }
    ],
    [
      "welch_t_weight",
      {
        "F64": 0.0
      }
    ],
    [
      "welch_t_distance",
      {
        "F64": 0.0
      }
    ],
    [
      "max_bin_diff_weight",
      {
        "F64": 0.0
      }
    ],
    [
      "max_bin_diff_distance",
      {
        "F64": 0.0
      }
    ]
  ],
  "positive_control": {
    "MutantCaught": {
      "mutant_id": "observe_leaky",
      "detection_channel": "welch_t"
    }
  },
  "scoping": {
    "spec_mandate": "N08: Hamming weight/distance profile across successive states is input-independent",
    "delivered_evidence": "receipt minted from a real tier-1 run: 10,000-step chained kernel profile court per payload class (HIGH 0xFFFF / LOW 0x0000), Welch t + 1%-per-bin histogram comparison, plus the firing leaky-observer positive control",
    "is_full_spec_mandate": false,
    "bound": "calibrated Tier-1 prefix: 10k samples per class vs the 100k spec scale"
  },
  "digest": "e6c8d884e346e9e6c22518ee9400789ddbe8a7db7fe2781103033024929a372c",
  "timestamp_ms": 0
}
```

### Dossier control reality

Of the 20 receipt files, 16 (N01, N20, and 14 others) are **synthetic
control-stub receipts** (`cr-control-*` receipt ids, subject
`"control-subject"`) — they exercise the dossier generator, not the
courts. Four courts — N08 (`src/nato/hamming.rs`), N12
(`src/nato/tick.rs`), N14 (`src/nato/thrash.rs`), N15
(`src/nato/liveness.rs`) — carry **real-run receipts** minted by their
`CTT_EMIT_RECEIPTS=1` emitters. A missing receipt file degrades the N20
dossier to `Refused`, never a silent pass.
