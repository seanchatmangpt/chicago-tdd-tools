# chicago-tdd-tools

[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A Rust testing framework that enforces Chicago-style TDD (Classicist,
real-collaborator testing) through compile-time guarantees, plus three
feature-gated assurance surfaces for proving that processes and artifacts
actually conformed: a POWL process-conformance kernel (`conformance`), a
tamper-evident court-receipt wire format (`ledger`), and a 20-court NATO
assurance matrix (`nato`).

If it compiles, the AAA (Arrange-Act-Assert) structure is enforced; the
assurance features make "it passed" a checkable artifact instead of a claim.

---

## Use as a dependency

`chicago-tdd-tools` is published to crates.io (current version `26.10.12`,
[Cargo.toml:3](Cargo.toml)); a `path` or `git` dependency also works.

```toml
# crates.io
[dev-dependencies]
chicago-tdd-tools = { version = "26.10.12", features = ["testing-extras"] }
tokio = { version = "1", features = ["macros", "rt", "rt-multi-thread"] }

# path / git forms are also supported:
# chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["testing-extras"] }
# chicago-tdd-tools = { git = "https://github.com/seanchatmangpt/chicago-tdd-tools",
#                        tag = "v26.10.12" }
```

Only the core/prelude (`test!`, `async_test!`, `fixture_test!`, builders,
assertions) is enabled by default; no external services are required.

A minimal example that compiles against the default feature set:

```rust
// tests/smoke.rs
use chicago_tdd_tools::prelude::*;

test!(chicago_tdd_tools_is_usable, {
    let x = 2 + 2;
    assert_eq!(x, 4);
});
```

`test!` expands to a plain `#[test]` fn, so the file runs with
`cargo test --test smoke`. For the unified OTEL/Weaver harness:

```rust
// feature = "otel" (or "weaver", which implies it)
use chicago_tdd_tools::observability::ObservabilityTest;

let test = ObservabilityTest::new()?;
// ... send spans, inspect validation results ...
```

---

## Testing discipline

Chicago-style (Classicist) TDD: verify behavior with **real collaborators**,
not mocks. The framework's enforcement layers:

- **Type-first design** — state machines are encoded at the type level; an
  invalid test state is a compile error, not a CI failure.
- **Poka-yoke lint posture** — `Cargo.toml` denies `clippy::all`,
  `pedantic`, `nursery`, `cargo`, and `-D warnings`; `unsafe_code` is denied
  (Cargo.toml:355-368).
- **Build-time prevention** — `just pre-commit` runs `timeout-check fmt lint
  test-unit dead-code-check` (Justfile:494); the `pre-commit-staged-checks`
  recipe additionally sweeps staged diffs for `.unwrap()`, `.expect()`,
  `todo!`, and `unimplemented!` (Justfile:487).
- **Result-based errors** — no panics in production code; use `?`, `match`,
  or the alert macros (`alert_critical!`, `alert_warning!`, `alert_info!`,
  `alert_success!`, `alert_debug!`, crate root).

Core macros (always available, crate root / prelude):

```rust
use chicago_tdd_tools::prelude::*;

test!(test_sync_behavior, {
    let input = vec![1, 2, 3, 4, 5];
    let sum: i32 = input.iter().sum();
    assert_eq!(sum, 15);
});

async_test!(test_async_operation, {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let result = 2 + 3;
    assert_eq!(result, 5);
});

fixture_test!(test_with_fixture, fixture, {
    let counter = fixture.test_counter();
    assert!(counter < u64::MAX);
});
```

Assertion helpers: `assert_ok!`, `assert_err!`, `assert_in_range!`,
`assert_eq_msg!`, plus `assert_within_tick_budget!` for tick-budget
performance checks. The `conformance` feature adds `conformance_test!` and
`assert_conforms!(session, receipt)`, which asserts BOTH the trajectory
conforms AND the test receipt is algebraically closed
(src/core/macros/conformance_test.rs:71-86).

---

## Feature flags

All flags below are quoted from `[features]` in [Cargo.toml](Cargo.toml)
(HEAD `bf7ea75`). Default: `logging` only.

| Feature | Implies / pulls | Purpose |
|---|---|---|
| `default` | `logging` | Alert helpers wired to the `log` crate |
| `testing-extras` | property + snapshot + fake-data | The 80% bundle |
| `testing-full` | all testing features | Maximum testing coverage |
| `property-testing` | `proptest`, `arbitrary` | `testing::property` module |
| `snapshot-testing` | `insta` | `testing::snapshot`, `SnapshotAssert` |
| `fake-data` | `fake` | `builders::FakeDataBuilder` |
| `mutation-testing` | (no deps) | Mutation-scoring types (reported metric, not an enforced gate) |
| `concurrency-testing` | `loom` | `testing::concurrency`, `LoomModel` |
| `parameterized-testing` | `rstest` | Parameterized test macros |
| `cli-testing` | `trycmd` | `testing::cli`, `CliTest`, golden files |
| `cli-proof` | `tempfile` | CLI harness + sabotage fixtures |
| `async` | `tokio` | Async performance-measurement utilities |
| `benchmarking` | (no deps) | Criterion benchmarking support |
| `otel` | `tokio` | `observability::otel` validation |
| `weaver` | `otel` + OTEL crates + `tempfile` | Weaver live-check |
| `observability-full` | `otel` + `weaver` | Full observability stack |
| `testcontainers` | `testcontainers` crate | Docker integration testing |
| `integration-full` | `testcontainers` + `weaver` | Integration + observability |
| `ocel-generation` | `wasm4pm-compat`, `dashmap`, `tracing`, `blake3` | OCEL 2.0 test logs |
| `ocel-generation-discovery` | `ocel-generation` | Process discovery / graduation candidates |
| `ocel-ingest` | `ocel-generation` | OCEL-to-OCEL64 word packing bridge |
| `ocel-ingest-json` | `ocel-ingest` | OCEL 2.0 JSON import (fail-closed) |
| `ocel-ingest-xml` | `ocel-ingest` | Roundtrip-first OCEL XML reader |
| `conformance` | (zero deps) | POWL replay kernel (below) |
| `ledger` | `receipt-validation`, `blake3` | Court-receipt wire + chain/Merkle laws |
| `nato` | `ledger`, `conformance`, `ml-dsa`, `zeroize`, `getrandom` | N01-N20 court matrix |
| `all-assurance` | `conformance` + `ledger` + `nato` | Full defense-assurance surface |
| `wasm-guest` | `conformance` | Zero-import WASM guest (telemetry also needs `ocel-ingest`) |
| `guest-abi-v1` | (no deps) | Legacy ABI v1 shim (probe -> 1, 34-byte seal preimage) |
| `receipt-validation` | `blake3` | BLAKE3 receipt-chain replay validation |
| `powl-bcinr` | `conformance` | **UNWIRED by design** — see limitations |
| `logging` | `log` | Default; alert/log integration |
| `uuid-v4` | `uuid` | v4 UUID ids (host-side only) |

---

## Assurance surface 1: conformance (POWL replay)

Load a POWL process model (fail-closed, positioned parse errors), attach a
stream of OCEL64 event words (activity id in bits [7:0], tick in bits
[15:8], ticks strictly `prev + 1`), and replay it cooperatively against a
branchless PN64 net. Deviations are typed values, never strings.

Feature-gated trim of the compile-checked
[examples/conformance_quickstart.rs](examples/conformance_quickstart.rs):

```rust
use chicago_tdd_tools::conformance::{ConformanceSession, ReportOutcome, YieldStatus};

// 1. Fail-closed load: parse errors are positioned and typed.
let mut session = ConformanceSession::from_powl_file(&model_path).expect("parse POWL fixture");

// 2. Attach an OCEL64 word stream: tids 0,1,2 at ticks 1,2,3.
session.attach_stream(&words).expect("attach stream");

// 3. Cooperative reductions: step in bounded slices, resuming until complete.
let report = loop {
    match session.step_budget(2) {
        YieldStatus::Yielded { remaining_events } => {
            println!("yielded: {remaining_events} event(s) remaining");
        }
        YieldStatus::Complete(report) => break report,
        YieldStatus::Violation(err) => panic!("conformance violation: {err}"),
    }
};

// 4. Structured verdict: every event replayed, no violation. The verdict is
//    computed by the library, never asserted by the caller.
assert_eq!(report.outcome, ReportOutcome::Complete);
```

The POWL DSL itself (verbatim from the example):

```text
# Three-atom linear process: handle -> fork -> approve
activity handle  = log.handle
activity fork    = robot.fork
activity approve = ledger.approve

sequence {
    handle
    fork
    approve
}
```

Typed `Deviation`s: `VerbMismatch`, `NotEnabled`, `ForbiddenBit`,
`OutOfOrder`, `Unfinished`, with `open_deviations()` / `explain(idx)` /
`is_conforming()` accessors. Full error model (18 typed variants), DSL
grammar, and session API:
[docs/features/CONFORMANCE.md](docs/features/CONFORMANCE.md).

Run it:

```bash
cargo run --example conformance_quickstart --features conformance
# or: just test-conformance
```

---

## Assurance surface 2: ledger (court-receipt wire format)

A strict, fixed-size court-receipt wire format with BLAKE3 chain and Merkle
dossier laws. Constants verified in
[src/ledger/receipt.rs:89-99](src/ledger/receipt.rs):

- `MAGIC = b"CTRT"`, `VERSION = 1`
- 320-byte court receipt (`COURT_RECEIPT_V1_LEN`), 352-byte chained record
  (`COURT_RECORD_LEN` = receipt + 32-byte BLAKE3 chain hash), with a
  compile-time const-assert that the lengths compose.
- Dossier sealing: ML-DSA-65 (`SIGNATURE_LEN = 3309`,
  `PUBLIC_KEY_LEN = 1952`), context `chicago-ledger-dossier-v1`, with
  subject-binding-before-signature verification order.
- Typed exit codes (`LedgerExitCode`): `Ok = 0`, then per-class codes
  10-19 (`MalformedReceipt = 10` through `DossierInvalid = 19`).

The ledger surface is a **pure library**: there are no CLI verbs in this
crate. The former `ctdd_ledger` binary and its `ledger-cli` feature were
excised in v26.10.11; CLI verbs (receipt validate / chain-verify, dossier
seal / verify / emit / inspect) route through the external `affi` binary
(`~/affidavit`). The `just emit-dossier` / `just dossier-verify` recipes in
this repo's Justfile (Justfile:276/291) invoke that external binary. Full
JSON schema and exit-code table:
[docs/features/LEDGER.md](docs/features/LEDGER.md).

---

## Assurance surface 3: nato (N01-N20 court matrix)

`nato` (implies `ledger` + `conformance`) delivers a 20-court assurance
matrix — `COURT_NAMES: [&str; 20]`
([src/nato/registry.rs:88](src/nato/registry.rs)) — targeting EAL7 /
DO-178C Level-A / DO-333 / STANAG 4107 evidence classes.

- **Positive-control-first law**: every court must prove it can fail. A
  court that cannot refuse cannot admit — if a mutant survives the court,
  the verdict is void and the run records `Refused(positive_control)`.
- **Honest-scoping law**: every receipt carries a typed `ScopingRecord`
  (`spec_mandate`, `delivered_evidence`, `is_full_spec_mandate`, `bound`)
  so honest scoping is a required field, not a comment.
- Receipts are written to `docs/receipts/N{01..20}.json` only when the
  opt-in gate `CTT_EMIT_RECEIPTS=1` is set (hermetic by default).
- N20 aggregates N01-N19 receipts into one dossier sealed with ML-DSA-65;
  a missing receipt file degrades the dossier to `Refused`, never a silent
  pass. The emitted artifact is
  `docs/nato/NATO_STANAG_4107_DOSSIER.json` (generated; run
  `just emit-dossier`).

Run: `just test-nato` (tier-1); `just test-nato-tier2` and
`just nato-courts-full` for the higher tiers. Per-court delivered scope and
the receipt contract: [docs/features/NATO_COURTS.md](docs/features/NATO_COURTS.md)
and [src/nato/README.md](src/nato/README.md).

---

## Process mining (OCEL)

`ocel-generation` turns test execution into OCEL 2.0 event logs via the
published [`wasm4pm-compat`](https://crates.io/crates/wasm4pm-compat)
crate, which enforces a one-way evidence lifecycle (`Raw` -> `Admitted` ->
`Receipted`); invalid evidence cannot reach the receipted state. The
`ocel-ingest` family packs `OcelLog`s into fixed OCEL64 words; the JSON and
XML importers are fail-closed (a bad document is refused whole, never
partially imported).

```rust
use chicago_tdd_tools::observability::ocel::OcelCollector;

let collector = OcelCollector::new();
// events accumulate during the run, are admitted (case-id, timestamp
// monotonicity, object-reference validation), then sealed into a
// receipted log with a digest you can pin in CI
```

More: [docs/OCEL.md](docs/OCEL.md), the packed-tier disclosures in
[docs/features/CONFORMANCE.md](docs/features/CONFORMANCE.md).

---

## Build system: use `just`

Always use `just`, never raw `cargo` — the Justfile handles workspace
members, enforces timeout SLAs, and keeps the environment consistent.

| Command | What it does |
|---|---|
| `just check` | Fast compilation check |
| `just test` / `just test-unit` | Unit tests (nextest) |
| `just pre-commit` | `timeout-check fmt lint test-unit dead-code-check` (run before every commit) |
| `just lint` | Clippy + dylint |
| `just fmt` | Formatting |
| `just ci-local` | CI simulation |
| `just test-all` | Unit + Docker integration |
| `just test-integration` | Docker/Weaver integration tests (`WEAVER_ALLOW_SKIP=1` to skip Weaver) |
| `just test-conformance` | Conformance kernel tests |
| `just test-ledger` | Ledger law tests |
| `just test-nato` | NATO courts, tier-1 |
| `just test-nato-tier2` / `just nato-courts-full` | Higher NATO tiers |
| `just test-ocel-ingest` | OCEL64 bridge tests |
| `just verify-hermetic` | Locked + offline build/test matrix over conformance/ledger/nato |
| `just check-mutation-gate` | Mutation-score gate |
| `just emit-dossier` / `just dossier-verify` | Emit/verify the NATO dossier via external `affi` |
| `just weaver-bootstrap` / `just weaver-smoke` | Weaver CLI + registry setup / smoke test |
| `just test-property` / `test-mutation` / `test-snapshot` | Advanced testing recipes |
| `just snapshot-review` / `accept` / `reject` | Snapshot management |
| `just coverage` | Coverage report |
| `just docs` | Generate Rustdoc |

Setup: `cargo install just cargo-nextest`. The repo pins a nightly
toolchain ([rust-toolchain.toml](rust-toolchain.toml)).

---

## Examples

Runnable examples live in [examples/](examples/); each is registered in
Cargo.toml with its feature gates. Entries include `basic_test`,
`macro_examples`, `property_testing` (`property-testing`),
`snapshot_testing` (`snapshot-testing`), `concurrency_testing`
(`concurrency-testing`), `cli_testing` (`cli-testing`),
`testcontainers_example` (`testcontainers`), `otel_weaver_testing`
(`otel`,`weaver`), and `conformance_quickstart` (`conformance`). Catalog
with categories: [examples/README.md](examples/README.md).

---

## Honest limitations and disclosures

- **CLI verbs need `affi`.** This crate is library-only; every CLI verb
  (dossier emit/inspect, receipt validation) lives in the external `affi`
  binary (`~/affidavit`). The former `ctdd_ledger` binary and `ledger-cli`
  feature were excised in v26.10.11.
- **`powl-bcinr` is unwired by design.** The feature exists (declared as
  `powl-bcinr = ["conformance"]`) but the bridge is compiled out
  (`#[cfg(any())]` in src/conformance/bcinr.rs:44) and `validate_bcinr`
  (validate.rs:166) is a stub. Enabling it contributes zero items.
- **NATO dossier receipts are not all real-run.** Of the 20 receipt files,
  16 are synthetic control-stub receipts (`cr-control-*`, subject
  `"control-subject"`) exercising the dossier generator; four courts (N08,
  N12, N14, N15) mint real-run receipts via their `CTT_EMIT_RECEIPTS=1`
  emitters. Spec-scale figures (2^64 SMT, 10M dudect, 100M SEU) are
  evidence-class targets; courts deliver the bounded/calibrated equivalent
  and record the mapping in each `ScopingRecord`.
- **The packed OCEL64 tier discards identity and attributes.** Attributes
  do not survive the roundtrip; event ids regenerate as `e_{n}`, case ids
  as `case_{n}`, and absolute timestamps are not carried (per-case tick
  deltas only). Use the rich `OcelLog` tier when attributes matter.
- **Packed-tier id capacity has a birthday bound.** The id map uses a
  16-bit blake3-truncated collision sentinel: about 302 names gives ~50%
  collision probability; fewer than 83 names keeps p below 5%. A collision
  is a typed refusal (`IdMapError::Collision`), never silent.
- **Conformance session is single-shot per stream.** The stream field is
  private (no public getter), and `attach_stream` on a `Complete`/`Faulted`
  session stays frozen — attaching does not unfreeze a terminal phase.
- **Operator-gated items.** NATO receipt emission (`CTT_EMIT_RECEIPTS=1`)
  and Weaver-based runs are environment-gated; tier-3 courts
  (`nato-courts-full`) run at release gates, not per-commit.
- **No `just install-hooks` recipe exists.** The git hooks referenced in
  [CLAUDE.md](CLAUDE.md) are enforced via the `pre-commit` /
  `pre-commit-staged-checks` recipes and CI instead.
- **`just test` runs the lib suite only.** It does not compile `tests/`
  integration files; use `cargo test --test <name>` for a specific file.

---

## Documentation

- [docs/features/CONFORMANCE.md](docs/features/CONFORMANCE.md) — conformance
  kernel reference (error model, DSL grammar, session API, OCEL64 bridge)
- [docs/features/NATO_COURTS.md](docs/features/NATO_COURTS.md) — court
  matrix, tiering rules, receipt contract
- [docs/features/LEDGER.md](docs/features/LEDGER.md) — wire format, JSON
  schema, exit codes
- [docs/getting-started/GETTING_STARTED.md](docs/getting-started/GETTING_STARTED.md)
- [docs/reference/ARCHITECTURE.md](docs/reference/ARCHITECTURE.md)
- [docs/releases/CHANGELOG.md](docs/releases/CHANGELOG.md) — full change history
- [src/nato/README.md](src/nato/README.md) — NATO module documentation
- [examples/README.md](examples/README.md)

## Contributing

Issues and docs feedback: [GitHub
Issues](https://github.com/seanchatmangpt/chicago-tdd-tools/issues). Code
contributions: follow
[docs/process/CODE_REVIEW_CHECKLIST.md](docs/process/CODE_REVIEW_CHECKLIST.md)
and run `just pre-commit`.

## License

MIT
