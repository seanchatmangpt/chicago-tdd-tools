# Release Notes: v26.6.121

## Summary

v26.6.121 is a **feature release** that adds a process-truth layer to the framework: test runs can now be mined as Object-Centric Event Logs (OCEL 2.0) and checked for lawful process conformance. It also introduces an agent **governance** substrate, **wave** orchestration, the complete **43-pattern YAWL operator registry**, and a new editor-guard crate — alongside a large hardening pass (93 stubs/cheats removed, a clean strict build, and a `testcontainers` security upgrade).

> **Doctrine:** *If the code says it worked but the event log cannot prove a lawful process happened, then it did not work.*

## Changes

### 1. OCEL 2.0 Process Mining (`observability::ocel`)
Transforms test execution into Object-Centric Event Logs with **zero changes to test code** — by enabling the `ocel-generation` feature, the framework's internal `DiagnosticSink` pipeline collects and admits events into a standard OCEL log.

- **Typed carriers**: `TestOcelEvent`, `OcelLog`, `TestObject`, `TestActivity`, `TestObjectType`.
- **`OcelCollector`**: a `DiagnosticSink` that captures test diagnostics as OCEL events.
- **Admission gate**: events are validated for lawful case IDs, timestamp monotonicity, and object references before they count — built on the published [`wasm4pm-compat 26.6.11`](https://crates.io/crates/wasm4pm-compat) one-way lifecycle `Raw → Admitted → Receipted`. `Admission::new(...).into_evidence()` is the only path to admitted evidence; there is no bypass.
- **`seal_run()`**: seals a run into a receipted `OcelLog` plus a stable hex digest you can pin in CI.
- **`project_admission_events()`**: projects an admitted log for downstream analysis.
- **Process discovery** (`ocel-generation-discovery`): `ProcessModelStore` and `graduate_for_discovery()` surface `GraduationCandidate`s where the mined process diverges from the declared model.

See **[docs/OCEL.md](../OCEL.md)**.

### 2. Agent Governance (`core::governance`)
A baseline for compile-time and runtime law enforcement and diagnostic routing.

- **Diagnostics**: `Severity`, `DiagnosticCategory`, `DiagnosticCode` (parseable, prefixed), `Diagnostic` (self-validating), `DiagnosticSink`, `TaskReceipt` (validate + sign).
- **Global channel**: `register_sink`, `register_domain`, `set_run_id`, `emit_diagnostic`, `on_test_started`/`on_test_completed`, `close_channel` → `RunSummary`.
- **Laws**: `AdmissionMetadata`, `SubstrateDelta`, `ContributionKind` (evolutionary vs. additive substrate changes).
- **Sectors**: `SectorStack`, `MergeStrategy`, `ProcessIntelligenceSector`.

See **[docs/governance_architecture.md](../governance_architecture.md)**.

### 3. Wave Orchestration (`swarm::wave`)
N-phase sequential waves with M parallel tasks, plus wave-state observability and failure classification.

- `Wave`, `WavePhase`, `WaveStatus` (`Queued`/`Executing`/`Completed`/`Failed`).
- `WaveReceipt`, `PhaseReceipt`, and `ResidualClass` for classifying failures that occur during a wave.

### 4. Full YAWL Operator Registry (`operator_registry`)
All **43** YAWL workflow control patterns are now registered, each characterized by its control-flow law.

- `OperatorRegistry`, `OperatorDescriptor`, `OperatorProperties`, `GuardType` (Legality, Budget, Chronology, Causality, Recursion).
- Query helpers: `count_by_category`, `count_deterministic`/`idempotent`/`type_preserving`/`bounded`, `operators_with_guard`, `operators_fully_deterministic`; `global_registry()`.

### 5. LSP Guard Crate (`chicago-tdd-lsp`)
A dev-dependency guard built on lsp-max: watches every `Cargo.toml` opened in the editor and emits **`CTDD-DEV-001`** whenever `chicago-tdd-tools` appears in `[dependencies]` instead of `[dev-dependencies]`. Excluded from the root workspace build so it never forces `lsp-max` resolution into this crate's dependency graph.

### 6. Hardening
- **93 stubs and cheats eliminated** (69 + 24) across core/testing/observability/integration — real implementations for verification-pipeline phases 3 & 4, the dog-fooding string-literal scanner, testcontainers state machine, and more.
- **Clean strict build**: 445 unit tests passing, 0 clippy warnings under `all`/`pedantic`/`nursery`/`cargo`, zero `unwrap`/`expect` in production code.
- **6-gate pre-push** validation (check → clippy → error-handling scan → fmt → unit tests → examples → docs), with timeouts tuned for the heavier `testcontainers`/`bollard` graph (`fmt` 5s→30s, `check` 30s→60s).
- `HttpGet::execute()` now returns an explicit feature-missing error instead of silently succeeding.

### 7. Security
- Upgraded `testcontainers` `^0.25 → ^0.27` to clear `astral-tokio-tar` advisories.

## Installation

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "26.6.121", features = ["testing-extras", "ocel-generation"] }
```

## Upgrade Notes

Backward compatible — existing `prelude` and crate-root imports continue to work. All new capabilities are additive and feature-gated (`ocel-generation`, `ocel-generation-discovery`). Process mining builds on the published `wasm4pm-compat 26.6.11` crate; no in-tree vendor is required.
