# OCEL 2.0 Generation in Chicago TDD Tools

`chicago-tdd-tools` automatically transforms test execution into Object-Centric Event Logs (OCEL 2.0).

## The Core Insight

A test run **is** an OCEL 2.0 event log:
- **Test activities** (Started, Completed, Setup) are OCEL events.
- **Fixtures, artifacts, schemas, and receipts** are OCEL objects.
- **Governance signals** (Diagnostics) are provenance-bearing events.

## Zero Code Changes

OCEL generation requires no changes to your test code. By enabling the `ocel-generation` feature, the framework's internal `DiagnosticSink` pipeline is instrumented to collect and admit events into a standard OCEL log.

## Features

### 1. Automatic Event Emission
Every macro execution (`assert_admitted!`, `assert_no_drift!`, etc.) produces OCEL events as a side effect. These events relate multiple objects (e.g., an Artifact and its governing Schema) simultaneously.

### 2. Evidence Lifecycle
Events are admitted through the `wasm4pm-compat` Evidence lifecycle, ensuring that the generated log is valid and has proper provenance.

### 3. Log Sealing
At the end of a run, the log is sealed with a BLAKE3 digest and can be exported as a `.ocel.jsonl` file for consumption by standard process mining tools.

### 4. Self-Teaching Loop
When `ocel-generation-discovery` is enabled, the framework can automatically discover process models from your test runs and use them to enforce conformance in future runs via `assert_conformant_auto!`.

## Configuration

Enable the feature in your `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "26.6.12", features = ["ocel-generation"] }
```

---
*chicago-tdd-tools Observability Documentation*
