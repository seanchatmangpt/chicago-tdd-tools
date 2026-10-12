# How To: RE/Timing Audit Playbook

Task-oriented recipes for auditing that a kernel is actually constant-time and actually
branch-free. All paths are landed code in this repo; commands run from the repo root.

## Audit timing invariance with the dudect court (`|t| < 4.5`)

The landed harness is `src/nato/timing.rs` (court N05). It implements the dudect decision rule
(Reparaz et al. 2017): alternating fixed valid vs fixed refusal-class testcases, per-batch
**median** of burst-timed samples, Welch's two-sample t-test, acceptance `|t| < 4.5`.

### Run the smoke court

```sh
cargo test --features nato --lib nato::timing -- --nocapture
```

Relevant tests:

- `n05_dudect_step_branchless_smoke` — the kernel under court is
  `conformance::pn64::step_branchless`, via `run_court(BurstConfig::SMOKE, kernel)`.
- `n05_positive_control_detects_leak_smoke` — `leaky_refusal` is a deliberately timing-leaky
  twin; the court **must** flag it (`|t| >= 4.5`) or every PASS it issues carries no bits.

### Run the full-scale court

```sh
cargo test --features nato --lib nato::timing -- --ignored --nocapture
```

`n05_dudect_step_branchless_full_scale` (`#[ignore = "nato-tier3"]`) runs the spec
calibration: `BurstConfig::FULL` = 100 batches x 2,000 iters x 1,024 reps =
204,800,000 transitions per class per run. Smoke is 10 x 100 x 256 = 256,000. Both clear the
`TOTAL_PER_CLASS_ADMISSION_MIN = 100_000` gate.

### Reuse the harness on your own kernel

`run_court(cfg, f)` takes any `FnMut(u64, u64) -> u64`. Your kernel must be paired-class:

- a fixed **valid** testcase that commits, and
- a **refusal** testcase differing only in the verb byte (same word otherwise), so any timing
  difference is data effect, not workload effect.

The landed fixed vectors (`PN_VALID`, `OCEL_VALID`, `OCEL_REFUSAL`, `INVARIANT_MASK`) show the
shape. Write your own equivalents and call `run_court` directly:

```rust,ignore
let outcome = chicago_tdd_tools::nato::timing::run_court(
    BurstConfig::SMOKE, my_kernel,
)?;
assert!(chicago_tdd_tools::nato::timing::passes(outcome.final_t()?));
```

### What the harness guarantees (and refuses to fake)

- **Clock-resolution soundness**: a ~1 ns kernel cannot be resolved by single executions, so
  each sample is a burst of `reps` executions timed as one interval. The fastest raw burst must
  exceed 20x the measured clock resolution and every batch span > 100x, else the court returns
  typed `TimingWindowTooCoarse` / `BatchSpanTooShort` instead of grading timer artifacts.
- **Auto-scaling**: `scale_reps_for_resolution` pilots the per-rep cost on the *valid* class
  (the cheap twin — the soundness floor binds on the fastest burst) and raises `reps` to the
  next power of two clearing the 20x floor with a 4x margin. It only ever raises reps, so the
  100k/class admission gate still holds.
- **Idle reruns**: `run_court` reruns up to 4 attempts total (500 ms idle cooldown between);
  all attempt t-statistics are reported in `CourtOutcome::attempts`. Machine noise vs genuine
  divergence is decided by the rerun, not by hope.
- **Behavioral sanity**: the valid class must commit and the refusal class must leave the word
  bit-exact unchanged, else `TimingCourtError::BehavioralMismatch` — a timing comparison
  between identical outputs is vacuous.

### The affidavit twin court

`~/affidavit/tests/ct_timing_invariance.rs` is landed (324 lines) and mirrors the same law:
`T_THRESHOLD = 4.5`, up to 1 + 3 idle reruns, medians, Welch t. It courts
`qg05_ctrt_deserialize_timing_invariance` and `qg05_ml_dsa65_sign_timing_invariance`, plus
`qg05_positive_control_leaky_twin_must_fire`. Sample counts are environment-tunable; run it
from the affidavit checkout:

```sh
cd ~/affidavit
cargo test --release --features crypto-trust --test ct_timing_invariance -- --nocapture
```

The `crypto-trust` feature gates the suite (without it the filter matches
0 tests — a silent no-op); the default sample count takes minutes, so use
`--release`, or set `QG05_SAMPLES` to shrink it.

## Audit branchless codegen with the disassembly census (N06)

Statistical timing evidence is necessary but not sufficient; the N06 script censuses the
actual machine code.

```sh
scripts/n06_disasm_audit.sh
```

What it does:

1. **Wasm tier**: runs only when `wasm-objdump` is installed AND the `wasm32-unknown-unknown`
   target is installed; otherwise the tier is SKIPPED with a typed reason in the receipt
   (`ToolchainUnavailable` or `ArtifactMissing`) — never silently. (For this repo the wasm tier
   currently records a typed `ArtifactMissing` skip: no Gate-0 kernel artifact ships here; the
   native tier covers the same `step_branchless` body.)
2. **Native tier**: emits release asm of the lib build
   (`cargo rustc --lib --features nato --release -- --emit=asm`), extracts the
   `#[inline(never)]` `kernel_step` symbol (from `src/nato/disasm.rs`, which holds the fully
   inlined `step_branchless` body), and censuses conditional branches
   (`b.*`, `cbz/cbnz`, `tbz/tbnz` on arm64). Bound: **zero**.

Output: typed JSON receipt at `$N06_OUT` (default `docs/lanes/N06_disasm.json`), deserializable
into `src/nato/disasm.rs::N06Receipt`. Statuses: `PASS` (every executed tier at census 0),
`SKIPPED` (a tier could not run, typed reason; executed tiers still 0), `FAIL`. A `SKIPPED` receipt is honest evidence, not a pass: the court
maps it to `Verdict::Fail` carrying the typed skip disclosure (a nonzero
census or the audit itself broke).

Override the target dir or output:

```sh
CARGO_TARGET_DIR=/tmp/n06_target N06_OUT=/tmp/N06.json scripts/n06_disasm_audit.sh
```

## Strip name sections from a wasm artifact

Custom name sections (wasm section id 0) are the RE surface: they carry function/locals names
that make disassembly trivially readable. `wasm-strip` (from wabt) removes them:

```sh
wasm-strip target/wasm32-unknown-unknown/release/ctt_wasm_guest.wasm
```

Verify the audit still passes after stripping (the import census walks real section headers;
stripping must not change the import verdict):

```sh
just guest-audit
```

Note on the landed audit: `walk_sections` in `src/nato/guest_audit.rs` skips custom sections
(id 0) by design and counts only the five counted sections (types, imports, functions, memory,
exports), so a name section's presence or absence is invisible to the W4 verdict — the
census is a measure of the import bound, not of name-section hygiene. Stripping is an
RE-hygiene step, verified separately (e.g. `wasm-objdump -h` before/after).

## Choosing an audit tier

| Question | Court | Command |
|---|---|---|
| Does the compiled code branch on secret-dependent data? | N06 disasm census | `scripts/n06_disasm_audit.sh` |
| Do valid and refusal classes time identically, statistically? | N05 dudect court | `cargo test --features nato --lib nato::timing` |
| Did the harness itself detect a known leak? | N05 positive control | `n05_positive_control_detects_leak_smoke` (runs in the default suite) |
| Same laws, affidavit-side records (CTRT decode, ML-DSA-65 sign)? | QG05 twin court | `cd ~/affidavit && cargo test --release --features crypto-trust --test ct_timing_invariance` |

Run the disasm census and the timing court together before signing off any constant-time
claim: the census proves the code shape, the dudect court proves the measured behavior, and
the positive control proves the measurer works.
