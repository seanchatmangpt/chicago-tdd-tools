# Release Notes: v26.6.29

## Summary

v26.6.29 is a **security, cryptographic, and integration testing release** that establishes end-to-end process verification. It replaces the default hashing mechanism with cryptographically secure BLAKE3 chaining, introduces a universal chain validation suite to prove tamper-evidence and replayability, and delivers a comprehensive real-world configuration verification suite using `star-toml` as a collaborator workload. Additionally, this release achieves a clean-room state with **zero non-metric violations** under the forensic admissibility scan.

> **Doctrine:** *If the process evidence is not cryptographically bound and tamper-evident under BLAKE3 replay laws, the test has proved nothing.*

## Changes

### 1. Cryptographic BLAKE3 Chaining (`receipt-validation` feature)
Migrates the framework's process verification layer from standard hashing to secure, replay-verified BLAKE3 chain signatures.

- **`Blake3ReceiptEntry`**: A trait defining previous hash prepending, content hashing, and stored hash extraction.
- **`RawReceiptEntry`**: A 57-byte structural adapter mapping directly to the `bcinr-powl` format, supporting `replay_ptr()` decoding (extracting bytes 49..57).
- **`Blake3ChainValidator`**: Implements universal validation laws:
  - `validate_chain()`: Recomputes and asserts `BLAKE3(prev_hash || content_bytes) == stored_hash` for all log entries.
  - `assert_chain_valid()` and `assert_tamper_evident()`: Ensures that any mutated bytes in any entry break the chain and invalidate downstream digests.
- **OCEL Hashing Upgrade**: Wires the `ocel-generation` feature to use `blake3::Hasher` within `seal_run()`, making mined process logs cryptographically sealed.

See **[tests/blake3_receipt_tests.rs](../../tests/blake3_receipt_tests.rs)** (17 tests verifying replay determinism, tamper-evidence, mutation detection, and overflow flags).

### 2. E2E Integration Suite: `star-toml` Workload (`examples/star-toml`)
A real collaborator TDD integration example proving config-admission laws against the crates.io `star-toml` package.

- **Load & Validate**: Leverages `TrustedLoader` to layer TOML files (Default, Dev, Prod) with dynamic environment overrides.
- **Interface Guards**: Validates port ranges, workers constraints, non-empty fields, and cross-field TLS paths rules.
- **Verification Tiers**: 60 E2E tests written under **[tests/star_toml_e2e.rs](../../tests/star_toml_e2e.rs)** across 4 execution tiers:
  - **Tier 1**: Feature Coverage (Parsing, Merging, Validation, Alerts, Exit Codes).
  - **Tier 2**: Boundary & Corner cases (Empty configs, range bounds, bad syntax, unicode).
  - **Tier 3**: Cross-feature combinations (Layered errors, dynamic overrides).
  - **Tier 4**: Real-world application scenarios (Prod/Dev profiles, scale-out).
- **Gap Feedback**: Includes a detailed **[examples/star-toml/GAP_REPORT.md](../../examples/star-toml/GAP_REPORT.md)** converting feedback loops into trackable Jira/DoD tickets (ST-201 to ST-206).

See **[examples/star-toml/README.md](../../examples/star-toml/README.md)**.

### 3. Forensic Compliance & Clean-Room Scan
Hardens the repository against LLM-cheats and stubbed implementations by configuring the forensic admissibility scanner (`anti-llm-cheat-lsp`).

- **`.ignore`**: Excludes temporary/scratch files, doc archives, and playground targets from diagnostic sweeps.
- **`anti.toml`**: Configures domain terms, test paths, and suppression targets.
- **LSP Code Refactoring**: Modified rules (Oracle, Hollow, Receipts, Version, Placeholder) to support configuration-defined allowed suppression paths.
- **Result**: Zero non-metric scan violations.

## Installation

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "26.6.29", features = ["testing-extras", "ocel-generation", "receipt-validation"] }
```

## Upgrade Notes

Backward compatible. All new cryptographic validation and integration structures are additive. Crate version aligned to `26.6.29`. OCEL evidence lifecycle builds on the updated `wasm4pm-compat 26.6.26` package.
