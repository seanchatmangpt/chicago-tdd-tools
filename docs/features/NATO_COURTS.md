# NATO Courts

What the NATO assurance matrix is, how the positive-control gate law works,
what each court delivers, and how to run the courts at each tier. Everything
here is cited from files on disk in `src/nato/` and the `Justfile`.

## What the matrix is

`src/nato/` is the NATO high-assurance court suite for `chicago-tdd-tools`.
It ports the court set defined in the strata-side spec
(`/Users/sac/strata/docs/nato/NATO_N01_N20.md`, courts N01–N20) onto this
crate's type-level, poka-yoke discipline. Evidence classes targeted:
EAL7 / DO-178C Level-A / DO-333 / STANAG 4107.

A court is a pillar-classified adversarial evaluator
(`crate::nato::court::Court`). Every run is receipted: the framework mints a
`CourtReceipt` with a stable SHA-256 digest over a canonical string, so a
court verdict is an artifact you can replay, not a stdout line.

## The positive-control gate law

Every court must prove it can fail. A **positive control** — an injected
defect (a mutant, a leaked string, a torn write) — runs FIRST. If the probe
fails, the court's `run` is never called and the receipt records
`Verdict::Refused` with `ControlResult::GateRefused`. A court that cannot
refuse cannot admit. The exact gate, from `src/nato/court.rs`:

```rust
pub fn run_court(court: &dyn Court, candidate: &CandidateRef, subject_sha: &str) -> CourtReceipt {
    let probe = court.positive_control();
    let control_result = control::run_probe(probe);

    let outcome = if control::probe_passed(&control_result) {
        court.run(candidate)
    } else {
        CourtOutcome {
            verdict: Verdict::Refused,
            metrics: Vec::new(),
            scoping: ScopingRecord::refused_scope(),
        }
    };

    CourtReceipt::record(
        court.id(),
        court.pillar(),
        court.name(),
        subject_sha,
        outcome,
        control_result,
    )
}
```

The `Court` trait (same file) fixes the shape every implementation follows:

```rust
pub trait Court {
    /// Dyn-safe court identifier accessor. Concrete courts define the
    /// identity as associated consts (`const ID` / `const PILLAR`) and
    /// forward these methods to them.
    fn id(&self) -> CourtId;

    /// Dyn-safe pillar accessor (see [`Court::id`]).
    fn pillar(&self) -> Pillar;

    /// Canonical court name.
    fn name(&self) -> &'static str;

    /// The mandatory positive-control probe.
    fn positive_control(&self) -> ControlProbe;

    /// Run the court's adversarial evaluation of the candidate.
    fn run(&self, candidate: &CandidateRef) -> CourtOutcome;
}
```

`ControlProbe` is `pub type ControlProbe = fn() -> bool;` — cheap and
deterministic; `true` clears the court to run.

## CourtReceipt JSON shape

Defined by the serde structs in `src/nato/receipt.rs`
(`CourtReceipt`, `ControlResult`, `MetricValue`) and
`src/nato/disclosure.rs` (`ScopingRecord`). `CourtReceipt::to_json()`
emits pretty JSON of exactly this shape:

```json
{
  "receipt_id": "cr-N01-<16 hex of digest>",
  "court_id": "N01",
  "pillar": "Security",
  "name": "N01",
  "subject_sha": "…",
  "verdict": "Pass" ,
  "metrics": [["count", {"U64": 42}], ["ratio", {"F64": 0.5}]],
  "positive_control": {"MutantCaught": {"mutant_id": "…", "detection_channel": "…"}},
  "scoping": {
    "spec_mandate": "…",
    "delivered_evidence": "…",
    "is_full_spec_scale": true,
    "bound": ""
  },
  "digest": "<64 hex>",
  "timestamp_ms": 0
}
```

Notes on the shape, from the structs:

- `receipt_id` is `cr-{court}-{first 16 hex of digest}`; the `digest` is
  SHA-256 over the canonical string `id|sha|verdict|k1=v1;k2=v2` (metric
  values rendered as `tag:value`, e.g. `u64:42`), computed by
  `canonical_digest` in `src/nato/receipt.rs`.
- `verdict` is `Pass` | `Fail` | `Refused`. `Refused` means the court did
  not evaluate the candidate at all.
- `positive_control` is one of `MutantCaught { mutant_id, detection_channel }`,
  `MutantSurvived { mutant_id }`, `GateRefused { reason }`.
- `scoping` is the honest-scoping disclosure (see below); the validator
  (`ScopingRecord::validate`) rejects a partial disclosure that does not
  name both delivered evidence and its bound.
- `timestamp_ms` is a fixed `0`: receipts are ordered by digest, not
  wall-clock; lanes that need real time inject it.
- `pillar` is `Security` | `Timing` | `Resilience` | `Proof` | `Dossier`
  (machine names `security`/`timing`/`resilience`/`proof`/`dossier`).

### Receipt contract: `docs/receipts/N{01..20}.json`

Per the receipt contract in `src/nato/README.md`, each court writes its
`CourtReceipt` as JSON to `docs/receipts/N{01..20}.json` — one file per
court — and `verdict` in that serialized form is one of
`Admitted` | `Refused` | `NotRun`. A missing receipt file degrades the N20
dossier to `Refused`, never a silent pass. (`docs/receipts/` is populated
by court runs under the `nato` feature **and** the opt-in gate
`CTT_EMIT_RECEIPTS=1`; it is not checked in as static content.)

## The N01–N20 court matrix

Provenance: every court below is ported from the "Rust courts (N01–N20)"
section of the strata spec. Table mirrored from `src/nato/README.md`,
including the honest scoping wording.

| Court | Name | Pillar | Delivered scope |
|---|---|---|---|
| N01 | MILS non-interference | security | Bounded 2^16 exhaustive small-domain + 10^7 dual-tenant sweep proving `dState(TA)/dInput(TB) == 0`. Full Kani/Z3 2^64 NOT-SHIPPED; documented follow-up. |
| N02 | Coordinate orthogonality | proof | Spec-exact: exhaustive bit-spill census — every field write perturbs no neighbor field. |
| N03 | Bell-LaPadula / Biba lattice | security | Exhaustive over the 25x65,536-entry transition-table space: no-write-down, no-read-up. |
| N04 | Zero ambient authority | dossier (disclosed; the 5-value enum has no Authority variant) | Source-tier symbol/AST audit of the final artifact: zero env/fs/unmetered-clock imports on state paths. Binary-tier audit deferred (disclosed). In-code pillar: `Pillar::Dossier`, pinned by `pillar_mapping_matches_the_matrix`. |
| N05 | dudect timing | timing | Two tiers: smoke (small calibrated N in CI) and full (Welch t-test, `\|t\| < 4.5`, valid-vs-refusal timing; spec 10M, delivered calibrated-N disclosed). |
| N06 | Branchless disassembly | timing | `.s`/objdump census over the compiled artifact: zero conditional branches on state paths; unmatched symbols recorded as typed skips, never silent passes; a typed skip is adjudicated `Verdict::Fail` with a validating `ScopingRecord` disclosure. |
| N07 | Cache side-channel | timing | Prime+Probe-style concurrent eviction vs sequencer; latency invariance is **reported, not gated** (host-noise floor disclosed in receipt). |
| N08 | Hamming weight/distance profile | security | Test-court lane (`src/nato/hamming.rs`, cfg(test) adjudication — no dyn `Court` impl): HIGH (`0xFFFF`) vs LOW (`0x0000`) payload-class kernel chains over the PN64 committed marking, identical LCG seed so payload is the only variable. Tier-1 bin-diff gate plus tier-2 100k-samples-per-class run (`--ignored`); input-independent Hamming profile = no power-analysis leak. |
| N09 | SEU single-bit | resilience | Prefix-stream tiers: bounded randomized single-bit flips during ALU exec; 100.000% fail-closed-or-halt, 0% SDC on the executed prefix; calibrated-N disclosed. |
| N10 | SEU double-bit | resilience | Adjacent-lane double flips vs forbidden-bit check; same prefix-stream tiering as N09. |
| N11 | Torn-write / bus-disconnect | resilience | Fuzz over the ported 64B CAS ring: every torn write recovers to a valid prior state. |
| N12 | Clock-glitch | resilience | Hardware-tick law (`src/nato/tick.rs`): `HwTick::mono_checked` monotonicity over jittered/stuck/huge-jump tick sources. Jitter-injection regression is refused while forward chains admit; frozen clock gives bounded liveness; PN64 transition ordering law holds; wedged backstep clock is caught by positive control (honest host note: no counter register on this host). Tier-3 `n12_clock_resolution_calibration` is `--ignored` calibration. |
| N13 | WCET | timing | Static per-opcode table + calibrated measurement. Units are `ticks≈ns on aarch64` — never "cycles"; the measured table is published in the receipt. |
| N14 | CAST-32A thrash | resilience | Multi-core interference on the landed 64B CAS ring (`src/nato/thrash.rs`): 4-producer/4-consumer MPMC pipeline vs storm/spinner adversary threads. Three legs — typed `ConsensusError` refusals only (zero SDC, zero loss), liveness under watchdog deadlock bound, run-to-run conservation determinism. Tier-1 small-config legs plus `--ignored` tier-3 donor-verbatim `n14_cast32a_thrash_full`; positive control is a wedged shim that must hit `DeadlockBound`. |
| N15 | TLC liveness | proof | Starvation-freedom of the RONR admission surface (`src/nato/liveness.rs`), property-equivalent per the spec's own "Elixir property equivalent" sanction: exhaustive reachable-marking enumeration + round-robin fair-scheduler simulation with an explicit typed `StarvationMonitor` (enabled-then-fired within `TOTAL/32`), unfair-scheduler starvation counterexample, adoption-absorbing diamond, tier-3 full-net trap-pinning. TLC/Erlang itself NOT-SHIPPED (disclosed in the `ScopingRecord`). |
| N16 | Stack bound | dossier (disclosed; the 5-value enum has no Resource variant, per the N04 precedent) | Zero-recursion proof (call-graph analysis) + 32KiB ladder: depth-tiered deepest-frame bound, total <= 32KiB. |
| N17 | ML-DSA-65 KAT | security (disclosed; the 5-value enum has no Crypto variant, per the N04 precedent) | PIPELINE-KAT: known-answer tests against the signer. **NOT the official ACVP corpus**; the drop-in path to official vectors is documented in the receipt. |
| N18 | Zero-entropy diagnostics | security (disclosed; the 5-value enum has no Crypto variant, per the N04 precedent) | Entropy refusal court: Shannon entropy of all emitted error strings == 0 bits about internal state; strings that leak state are refused at construction. |
| N19 | Reproducible build | proof | 2-run same-host SHA-256 identity of the build artifact. 3-host cross-environment identity NOT-RUN (disclosed follow-up). |
| N20 | Dossier generator | dossier (disclosed; the 5-value enum has no Supply-chain variant) | Aggregates N01–N19 receipts into one dossier, sealed with ML-DSA-65 over the manifest digest. |

Numbering note: all 20 courts N01–N20 are delivered CTT-side
(`src/nato/registry.rs` `COURT_NAMES` is 20/20). N08 Hamming profile
(`src/nato/hamming.rs`), N12 clock-glitch (`src/nato/tick.rs`), N14 CAST-32A
thrash (`src/nato/thrash.rs`), and N15 liveness (`src/nato/liveness.rs`)
landed CTT-side as test-court lanes; the Erlang/OTP tiers of those four
remain strata-side. Pillar law is the landed `Court::PILLAR` consts, pinned by the
`pillar_mapping_matches_the_matrix` test; the emitter-carve-out receipts
(`docs/receipts/N{08,12,14,15}.json`) witness it for the four real-run
courts — the other 17 persisted receipts are the synthetic control set
(`sample_control_set()`), not pillar evidence.
N06 specifically: the court probe adjudicates the real
`scripts/n06_disasm_audit.sh` output and maps a typed skip to `Verdict::Fail`
+ disclosure, while `docs/receipts/N06.json` is the synthetic control receipt
minted by `emit_dossier_receipts_when_opted_in` — the live run's actual receipt
lands at `docs/lanes/N06_disasm.json` (or `$N06_OUT`), not in `docs/receipts/`.
Where the spec pillar has no enum variant, the in-code mapping is disclosed
in the row (N16 Resource->`Dossier`, N17/N18 Crypto->`Security`, per the N04
Authority->`Dossier` precedent).

## Files on disk (src/nato/)

Framework (landed):

- `mod.rs`, `court.rs`, `receipt.rs`, `disclosure.rs`, `registry.rs`,
  `rng.rs`, `control.rs` — gate law, receipts, scoping, naming, RNG.

Court pillar files landing in this release (file present on disk):

- `mils.rs` (N01), `orthogonality.rs` (N02), `bell_lapadula.rs` (N03),
  `zero_ambient.rs` (N04), `timing.rs` (N05), `disasm.rs` (N06),
  `cache.rs` (N07), `hamming.rs` (N08), `seu.rs` (N09/N10),
  `torn/` (N11: `ring.rs`, `slot.rs`), `tick.rs` (N12), `wcet.rs` (N13),
  `thrash.rs` (N14), `liveness.rs` (N15), `stackbound.rs` (N16), `kat.rs` (N17),
  `entropy.rs` (N18), `reproducible.rs` (N19), `dossier_court.rs` (N20),
  `guest_audit.rs` (guest-wasm import audit lane), `transition_tables.rs`
  (RONR + DAO transition-table substrate for the N02/N03 courts).

Concrete courts define identity as inherent associated consts and forward
`id()`/`pillar()` to them, e.g. `MilSeparationCourt` (`mils.rs:424-429`):

```rust
impl MilSeparationCourt {
    pub const ID: CourtId = CourtId::N01;
    pub const PILLAR: Pillar = Pillar::Security;
}
```

## Honest scoping law

Spec figures (2^64 SMT, 10M dudect, 100M SEU) are **evidence-class
targets** — lanes implement the bounded/calibrated equivalent and record
the mapping from spec mandate to executed proof in each receipt. A court
that cannot meet the spec scale **says so** and delivers the strongest
sound substitute (exhaustive small-domain + calibrated sampling + static
analysis), **never a silent weakening**.

In code this is `ScopingRecord` (`src/nato/disclosure.rs`): every court
outcome carries `spec_mandate`, `delivered_evidence`,
`is_full_spec_scale`, and `bound`; a partial disclosure that omits either
the evidence or the bound fails `validate()`. NOT-SHIPPED (N01 full Kani/Z3
2^64), NOT-RUN (N19 3-host identity), deferred (N04 binary-tier), and
reported-not-gated (N07 latency invariance) disclosures above are part of
the delivered contract, not caveats.

## Tiering

| Tier | Command (verified in `Justfile`) | When |
|---|---|---|
| tier-1 | `just test-nato` | Every commit — `cargo nextest run --lib --features nato nato::` (300s timeout) |
| tier-2 | `just test-nato-tier2` | Explicit invocation — `two explicit tier-2 test filters: n16_workload_100k_transitions_on_32kib_thread + n07_conservation_under_eviction_storm_full` (120s) |
| tier-3 | `just nato-courts-full` | Release gate — `cargo test --release --lib --features nato -- --ignored nato::` (600s) |

Related recipes also present in the `Justfile`: the `nato` feature offline
build check, `cargo mutants` over `src/nato` (with `src/conformance` and
`src/ledger`), and the dossier recipe
(`cargo test --release --lib --features nato,ledger -- nato::dossier_court --nocapture`).

Rule: a court's tier is fixed by its runtime cost, not its importance —
the tier-1 slice must be a sound (weaker but true) prefix of the tier-2/3
run, and the mapping is recorded in the receipt so no tier silently claims
another tier's strength.
