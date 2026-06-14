# Chicago TDD Tools v26.6.121: "Process Truth & Governance"

> **Prove the process, don't just pass the test.** Mine test runs as OCEL 2.0 event logs, enforce governance laws, and orchestrate waves — on a clean, hardened build.

---

## 🎯 Highlights

- 🔬 **OCEL 2.0 Process Mining** — new `observability::ocel` module turns test execution into Object-Centric Event Logs with a one-way `Raw → Admitted → Receipted` evidence lifecycle (built on the published [`wasm4pm-compat 26.6.11`](https://crates.io/crates/wasm4pm-compat)). Zero test-code changes; enable the `ocel-generation` feature.
- ⚖️ **Agent Governance** — new `core::governance` adds diagnostic/severity types, a global diagnostic channel, admission laws, and sector stacks.
- 🌊 **Wave Orchestration** — new `swarm::wave` runs N-phase sequential waves with M parallel tasks and `ResidualClass` failure classification.
- 📋 **Full YAWL Operator Registry** — all **43** workflow control patterns registered, each with its control-flow law and guards.
- 🧩 **`chicago-tdd-lsp`** — editor guard emitting `CTDD-DEV-001` when the crate lands in `[dependencies]` instead of `[dev-dependencies]`.
- 🛡️ **Hardening** — 93 stubs/cheats eliminated, 445 tests green, 0 clippy warnings (`all`/`pedantic`/`nursery`/`cargo`), 6-gate pre-push.
- 🔒 **Security** — `testcontainers ^0.25 → ^0.27` clears `astral-tokio-tar` advisories.

> *If the code says it worked but the event log cannot prove a lawful process happened, then it did not work.*

---

## 📦 Installation

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "26.6.121", features = ["testing-extras", "ocel-generation"] }
```

Process-mining bundle adds `ocel-generation` (OCEL logs) / `ocel-generation-discovery` (process discovery).

---

## ✨ What's Changed

- **Process mining**: `OcelCollector`, `seal_run()`, `project_admission_events()`, `TestOcelEvent`/`OcelLog`/`TestObject` — see [docs/OCEL.md](../OCEL.md).
- **Governance**: `Severity`, `DiagnosticCode`, `Diagnostic`, `DiagnosticSink`, global `channel`, `SectorStack`/`MergeStrategy` — see [docs/governance_architecture.md](../governance_architecture.md).
- **Wave**: `Wave`, `WavePhase`, `WaveStatus`, `WaveReceipt`, `ResidualClass`.
- **Operator registry**: 43 YAWL patterns via `OperatorRegistry`/`global_registry()`.
- **Dependency**: migrated OCEL off the in-tree vendor to published `wasm4pm-compat 26.6.11`.
- **Effects**: `HttpGet::execute()` returns an explicit feature-missing error instead of silent success.
- **Build**: `fmt`/`check` timeouts tuned for the heavier dep graph; `chicago-tdd-lsp` excluded from root build.

**Backward compatible** — all new capabilities are additive and feature-gated.
