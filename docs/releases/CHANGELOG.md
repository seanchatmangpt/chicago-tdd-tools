# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [26.10.12] - 2026-10-11
### Added
- **Temprun guest bridge** (BEAM gen_statem worker + wasmi host bridge, landed
  in strata (host binary `temprun_step`; rebased to guest ABI v2 in strata `59ed80c`): 4-byte
  little-endian
  framed port protocol with cooperative reduction yields; CTT-side the guest
  ABI surface (`src/conformance/guest.rs`) and `crates/ctt-wasm-guest/` are
  part of this train. Tracked in `docs/jira/v26.10.11/000-temprun-guest.md`.
### Removed
- **`ctdd_ledger` / `cq` CLI binaries** (`src/bin/ctdd_ledger.rs`, `src/bin/cq.rs`)
  and the `ledger-cli` feature + clap dependency, excised in the V2 cleanup.
  The domain surface remains in the library (`src/ledger/`, notably
  `dossier_wire.rs`); the CLI verbs are re-landed in the external `affi`
  binary (Phase D M2: `receipt validate`, `receipt chain-verify`,
  `dossier seal`, `dossier verify` behind the `ctdd` feature), and the
  Justfile dossier recipes (`verify-ledger`, `emit-dossier`, `dossier-verify`,
  `dossier-inspect`) route through the `affi` binary.
### Changed

- **Wave-6 closeout** (`b8dac24`/`934a70c`): CHANGELOG wave-4 entries
  landed (they had been claimed but not written); docs/lanes/ boundary
  law; pillar-law authority note (consts + mapping test are the law;
  17 persisted receipts are the synthetic control set); findings
  register SK6/SX6 synced to PARTIAL; SK2/SK3 restored to QUEUED.
- `tests/dossier_cli_tests.rs` is a library-only court
  (`--features ledger,nato`); stale bin/feature references in
  `docs/features/LEDGER.md`, `docs/features/MERGER_MATRIX.md` and
  `src/ledger/dossier_wire.rs` doc comments updated to the excised state.

## [26.10.12] continued - 2026-10-11
Endgame audit-wave convergence: 20-lane adversarial audit with repairs,
all with on-disk test receipts. Waves: 1 (`8ab234a` + `1fc1c72`);
2 (`5189603`); 3 (`70575e8` + `e465c1a`); 4 (`2c80a9e`, W-NTW +
W-ABI2); strata rebase (`59ed80c`); 5 (`451f4a8`); 6 (`b8dac24` +
`0f33671` + `934a70c` + `e0411dd`).

### Added
- **Prelude assert macros**: prelude gains 8 new assert macros
  (`src/core/macros/`).
- **Justfile hardening**: `AFFIDAVIT_DIR` override (A134);
  `release-validate-version` now checks all 3 lockstep version sites and
  fails on unread versions.
- **T10b measured collision pins**: OCEL collision thresholds measured and
  pinned (142/238/209, generators named); `powl_fixture_bridge` gains
  `TraceTooLong(K>255)` plus 3 new tests; `import_json` "not lossless"
  disclosure.
- **NATO real-run receipts**: N08/N12/N14/N15 receipts re-minted from real
  runs (byte-identical, real `subject_sha`, positive controls); dossier
  emitter no longer clobbers them (persists 16 control-set courts only).
- **Docs/table completions**: diataxis reference tables completed
  (0xE005/0xE103 rows); line cites refreshed (CONFORMANCE/LEDGER/
  beam-membrane/ARCH_REVIEW); docs index adds diataxis/nato/receipts + 6
  files; ARCHITECTURE adds assurance modules; QG-05 + affi commands made
  runnable.
- **Tests**: validation/builders +16 tests (budget-exceeded error paths,
  yaml/toml, fake-data builders); +2 guest-telemetry R44 tests; real
  type-level arithmetic tests + `should_panic` bound test.
- **Swarm Wave tests** (`5189603`): `Wave::execute` happy/fail-fast/Andon
  paths covered (0 to 3 tests); `task_assignments` made bounded with
  release-on-completion plus `clear_assignments_for`; swarm module doc
  de-overclaimed (no gossip/consensus; idle-fraction heuristic).
- **Property validator + determinism** (`5189603`): +5 tests (test_counter,
  seeded determinism both arms, `max_shrink_iters` plumb-through);
  validator-closure `property_all_data_valid` made non-vacuous; exact LCG
  pin replaces a tautology (18 to 23 tests).
- **Core law coverage** (`5189603`): +23 tests across contract, fail_fast
  and invariants (phases 3/4/6/7/8/10 both arms, `PhaseResult`, finalize,
  52-variant Display sample).

### Changed
- **Zero-import guest dep cleanup**: `ctt-wasm-guest` uuid "js" dependency
  deleted (imports==[] census PASS); panic=abort disclosure corrected
  (unwind kept for SEU `catch_unwind`).
- **Cross-repo toolchain fix**: `ConstSizeValid` rewritten as a 2-param const
  trait (no associated-const equality), compiles under affidavit's pinned
  nightly — cross-repo edge unblocked.
- **TempDir process-unique fallback** (`8ab234a`): the non-uuid fallback used
  a constant name "id-0", so two parallel `TempDir::new` calls collided with
  `AlreadyExists` (witnessed as a flaky `test_temp_dir_creation` panic); the
  fallback now derives a monotonic per-process counter plus pid.
  Receipt: `cargo test --lib --all-features core::test_utils` → 12 passed,
  0 failed.
- **Docs de-overclaim**: sector_stacks/CLAUDE.md/docs corrected
  (production-grade, 100+, scaffold ggen claim, Admitted|Refused|NotRun
  serialization); fabricated CTRT tokens corrected; bins gain module docs
  and named `EXIT_FAILURE` consts.
- **Wave-2 de-overclaim** (`5189603`): HYPER_ADVANCED_MICROKERNEL.md 8
  fabricated symbol cites replaced with real APIs, version line marked
  historical; sector_stacks/ARCHITECTURE/GETTING_STARTED/FMEA claims
  corrected; advanced_features example emits stdout; weaver runbook
  strings corrected.
- **Pre-commit hook BRE fix** (`e465c1a`): `scripts/hooks/pre-commit` used
  `\.unwrap\(\)`, whose `.` matched any `.unwrap*` prefix and whose BRE
  group mis-parsed under `grep`; ugrep hard-errors on the pattern. Rewritten
  as `[.]unwrap[(` with the installed copy refreshed; self-test proof
  documented in the hook.
- **Testcontainers retry sleeps** (`e465c1a`): retry backoff sleeps now use
  `CONTAINER_RETRY_INITIAL_DELAY_MS`; 2 dead delay consts removed
  (`src/integration/testcontainers/mod.rs`).
- **fail_fast ReceiptData docs** (`e465c1a`): field docs state the real
  shape contract honestly (`src/core/fail_fast.rs`).
- **Annealing doctrine landed** (`70575e8` + `e465c1a`):
  `docs/process/ANNEALING_DOCTRINE.md` (entropy-cascade failure model, ERRC
  asymmetry, mechanical-annealing saturation law, TRIZ bindings) + repo
  clauses in root `CLAUDE.md` (affi-sovereign CLI, court vacuity sweeps,
  skeptic lanes never grade their own builders); TTL `AnnealingVibration`
  admitted to the doctrine projection (primitives=27, IN_SYNC).
- **NATO dossier re-sealed** (`5189603`): dossier re-sealed over the real
  N08/N12/N14/N15 receipts (previously control-set stand-ins); receipts-parity
  closure witnessed — re-minted receipts byte-identical, `dossier verify` ok
  over the real receipt set (`e465c1a`).
- **Examples docs** (`e465c1a`): `examples/README.md` gains a star-toml
  section and a conformance feature-flag row.
- **W-NTW court registry** (`2c80a9e`): `impl Court` for all 15 harness
  courts N05-N19; `COURT_REGISTRY` `[CourtEntry; 20]` canonical order;
  falsifier `registry_sweep_runs_all_twenty_and_matches_real_receipts`
  (honest typed skips, on-disk field agreement for the emitter-carve-out
  courts, never persists); pillar alignment to emitter receipts.
- **Guest ABI v2 core** (`2c80a9e`): `SEAL_PREIMAGE_V2_LEN = 58` (v1
  reserved field consumed by the 32B module-digest lane);
  `ABI_VERSION = 2`; errnos 14-16; telemetry migrated; M2 falsifier
  (distinct digests -> distinct seals). Strata `59ed80c` rebased the
  temprun bridge to v2.
- **Clippy `--all-targets` zeroed** (`451f4a8`): ~100 errors across 5
  test/example files fixed — `tests/adversarial_as4.rs` (38: file-level
  allows with JUSTIFICATION for FFI cast/pointer plumbing + mechanical
  stale-code fixes), `tests/blake3_receipt_tests.rs` (~50: literal
  separators, doc_markdown backticks, justified low-byte cast),
  `tests/merger_conformance_tests.rs`, `examples/conformance_quickstart.rs`
  and `tests/prelude_surface_tests.rs` (13 more).
- **N06 census target dir** (`451f4a8`): `scripts/n06_disasm_audit.sh`
  census target moved to `target/n06-lanes` (rust-cache-coverable); the
  `/tmp` default was cold on every CI run against the 300s recipe SLA —
  warm census now 0.62s. Lane-artifact default moved to `docs/lanes/`.
- **N06 skip-law documented** (`451f4a8`): typed skip -> `Verdict::Fail`
  stated at 3 sites (`docs/features/NATO_COURTS.md`, execution how-to,
  `src/nato/disasm.rs` `DisasmCourt` docs).
- **Falsifier determinism note** (`451f4a8`): N05 statistical / N06 host /
  N19 artifact sweep determinism disclosed; N19 flip command now eprintln'd.
- **Telemetry doc reflow** (`451f4a8`): `write_event_words` docs split into
  two coherent paragraphs.
- **Debris removed**: `operator_registry.rs.backup`, `wip/`,
  `check_output.txt`; `__pycache__/` gitignored.

### Fixed
- **Merger/guests errno ABI**: negative leaf codes remapped to positive ABI
  errnos (`ERR_LENGTH`/`ERR_BAD_SCALE`/`ERR_NO_SOURCES`); `expect` replaced
  with typed `ERR_TRUNCATED`; clippy clean.
- **Guest telemetry R44 bounds**: `write_event_words` routed through the R44
  `out_span_oob` law (`ERR_CAP_TOO_SMALL`).
- **WCET sort comparators** use `total_cmp`.
- **MCP typed errors** (`5189603`): `build()` returns `Result` (typed
  `McpTestError::Configuration`); stdio piping reports typed `anyhow`
  errors; snapshot tests feature-gated; error_tests bounded to 60s with
  child reaping (hang fixed).
- **spec-harness `generate_receipt` bin** (`5189603`): rewritten against
  the current `SpecConformanceReceipt` (was 6 compile errors).
- **MemBoundGuard mutex serialization** (`5189603`): parallel tests shared
  the `LINEAR_MEMORY_END` static, causing a nondeterministic 1-fail flake;
  access now serialized; deterministic guest suite 662/0.
- **git_hook_installer duplicate-main** (`5189603`): duplicate main
  under all-features fixed.
- **mu-kernel ASCII env fold** (`5189603`): Unicode uppercase of "mu"
  broke strict-mode env derivation; folded to ASCII.
- Wave-2 gates (`5189603`): `cargo test --lib --all-features` 1134/0;
  `just lint` clean; guest suite 662/0 deterministic; merger 25/0; mcp
  testing suite green (no hang); spec-harness 21/0 + all-targets compile;
  test-conformance 46/0; test-nato 139/0; doctests 117/0.
- Gates (at `1fc1c72`): `cargo test --lib --all-features` 1093 passed / 0
  failed; `just lint` clean; merger 25/0; ledger_tests 25/0;
  conformance_tests 23/0;
  guest_abi 14/0; `just release-validate-version`: consistent 26.10.12
  (3 sites).
- Wave-5 gates (`451f4a8`): nato lib 853/0; nato_courts_tests 15/0; blake3
  17/0; merger 25/0; adversarial_as4 4/0; `just lint` clean; n06 census
  0.62s warm on `target/n06-lanes`; receipts writer set = lawful 5
  (re-enumerated).

### Security
- **Forgeable TaskReceipt embedded-key signature retired**: receipt
  signatures are now HMAC-only with `validate_with_key`; keys larger than
  255 bytes are refused. Migration note: any consumer verifying
  `TaskReceipt` signatures via the retired embedded-key path must switch to
  `validate_with_key` with a caller-held key.
- **Key custody hardening** (`src/ledger/signing/keys.rs`): `fsync` on
  (`sync_all` + parent-dir sync) and `Zeroizing` seed locals; `receipt.rs`
  gains a compile-time const proof `COURT_RECORD_LEN` composition.

## [26.10.11] - 2026-10-11

### Added
- **Conformance module** (`src/conformance/`): PN64 branchless Petri-net transition kernel ported
  from strata-protocol (`pn64.rs`, @ 0de13a3); POWL model/loader/validator (`model.rs`,
  `loader.rs`, `validate.rs`); `ConformanceSession` trajectory verification with reduction budgets
  (`session.rs`); OCEL64 ingestion bridge (`session.rs`).
- **OCEL importers** (`src/observability/ocel/`): `import_json.rs` and `import_xml.rs` behind the
  `ocel-ingest-json` / `ocel-ingest-xml` features, feeding the OCEL64 bridge (`ocel64_bridge.rs`,
  `idmap_bridge.rs`).
- **Ledger module** (`src/ledger/`): strict 320/352-byte court-receipt wire format (`receipt.rs`)
  with deserialize-time BLAKE3 digest verification; Merkle dossier roots and per-court inclusion
  proofs (`dossier.rs`, `proof.rs`); ML-DSA-65 signer/sealer/keys (`signing/`); `ctdd_ledger` CLI
  (`src/bin/ctdd_ledger.rs`).
- **NATO court framework** (`src/nato/`): adversarial court framework with positive-control gate
  (`control.rs`), Pillar I–V classification (Security/Timing/Resilience/Proof/Dossier), honest
  scoping records (`disclosure.rs`), and deterministic LCG-strided probes (`rng.rs`, `cache.rs`).
- **Compile-fail suite** (`tests/compile_fail/`, `tests/trybuild.rs`): trybuild-based compile-fail
  courts; new test suites `prelude_surface_tests.rs`, `cli_proof_tests.rs`,
  `observability_fixtures_tests.rs`, `ocel_ingest_tests.rs`.
- **Hermetic WASM guest** (`crates/ctt-wasm-guest/`): cdylib crate with a zero-import ABI
  (imports==∅), cooperative reduction-yield protocol, guest-produces/host-seals ML-DSA-65 receipt
  division, zero-import audit court, and native-ABI fault-injection courts.
- **Conformance-test macro** (`src/core/macros/conformance_test.rs`) and `assurance.yml` workflow
  (`.github/workflows/assurance.yml`).

- **NATO guest-import audit court** (`src/nato/guest_audit.rs`): W4 zero-import census parsing the
  guest artifact's wasm import section directly from the binary (no `wasm-tools`/`wasm-objdump`, no
  `std::process`), with honest `Skipped`/`ArtifactMissing` status per the N06 law.
- **Guest ABI consolidation**: canonical ABI errno table (`ERR_*` 0..=13,
  `src/conformance/guest.rs`) is the single source for yield-status vs errno separation;
  `guest_telemetry::write_event_words` is a never-truncating unsafe ABI writer with deterministic
  errno on refusal; `key_id` unified to the first 8 bytes of BLAKE3 over the signer public key
  (`src/ledger/signing/sealer.rs`).
- **Release docs (v26.10.11, DRAFT)**: `docs/releases/ANNOUNCEMENT_v26.10.11.md`,
  `docs/releases/ANNOUNCEMENT_FORWARD_DEPLOYMENT_OS_v26.10.11.md`,
  `docs/releases/ANNOUNCEMENT_ARCH_REVIEW_v26.10.11.md`, and the temprun-guest integration ticket
  `docs/jira/v26.10.11/000-temprun-guest.md`.
- **NATO court ports N08/N12/N14/N15** (`src/nato/`): the registry is now complete at 20/20
  CTT-side (`registry.rs` `COURT_NAMES`) — `hamming.rs` (N08 Hamming weight/distance profile,
  partial-scale honest disclosure), `tick.rs` (N12 clock-glitch resilience, wall-monotonic
  surrogate), `thrash.rs` (N14 CAST-32A multi-core interference), `liveness.rs` (N15 liveness).
- **Merger-guests crate** (`crates/merger-guests/`): hermetic WASM guest crate with 8 leaf kernels
  behind a uniform `mgi_*` stateless ABI (wasm32 artifact build in flight).
- **Merger conformance suite** (`tests/merger_conformance_tests.rs`): W1 contract pipeline + phase
  courts over the landed merger-matrix fixtures (fresh run: 25 passed / 0 failed / 0 ignored — all
  phase blocks green).
- **Merger Justfile recipes**: `merger-guest-build`, `merger-guest-test`, `merger-conformance`.

### Changed
- **Docs wave**: 15 dated plan/verification docs archived to `docs/archive/2026-10/` and
  `cookbook/archive/`; fabricated-API claims removed from docs; `cargo make` → `just` migrations;
  stale counts corrected.

### Fixed
- **ctdd_ledger CLI** (`src/bin/ctdd_ledger.rs`): explicit `exit_code(&LedgerError) -> u8` domain
  mapping (failures map to typed exit codes rather than a blanket 2), and `ChainRecord` now
  correctly implements `Blake3ReceiptEntry` for BLAKE3 chain verification.
- **N11 torn-write / slot+ring resilience court**: ported 64B CAS ring subject — every torn write
  recovers to a valid prior state; torn guest inputs refused typed (never admitted) in
  `crates/ctt-wasm-guest/tests/fault_injection.rs`.

## [26.6.29] - 2026-06-28

### Added
- **Cryptographic BLAKE3 Chaining & Validation** (feature `receipt-validation`): Adds the
  `Blake3ReceiptEntry` trait, `RawReceiptEntry` struct (providing a 57-byte adapter mapping to
  `bcinr-powl`), `ReceiptChainBuilder` helper, and the `Blake3ChainValidator` to programmatically
  assert the validity, replayability, and tamper-evidence of process receipt logs. Wires
  `seal_run()` to use `blake3::Hasher` for securing mined OCEL log digests under the
  `ocel-generation` feature.
- **Star-TOML TDD Integration Workload** (`examples/star-toml/`): Comprehensive
  config-admissibility testing example using the `star-toml` package as a real collaborator target
  (layered loader, validation rules, alerts, exit behaviors).
- **Comprehensive E2E Suite** (`tests/star_toml_e2e.rs`): 60 opaque-box tests mapped across 4 tiers
  (Feature Coverage, Boundaries, Cross-Feature Combinations, and Real-World Scenarios).
- **DoD Gap Report** (`examples/star-toml/GAP_REPORT.md`): Closed the loop on Track E requirements,
  converting feedback issues (ST-201 to ST-206) into actionable Jira/DoD tickets.

### Changed
- Bumped workspace packages and dependency versions to `26.6.29`.
- Upgraded `wasm4pm-compat` to version `26.6.26`.

### Fixed
- Fixed all `anti-llm-cheat-lsp` admissibility scan violations, achieving a zero non-metric
  diagnostic state across the entire codebase.

## [26.6.121] - 2026-06-13

### Added
- **OCEL 2.0 process mining** (`observability::ocel`, feature `ocel-generation`): turns test
  execution into Object-Centric Event Logs. New modules `collector`, `discovery`, `projections`,
  `types`, `wasm4pm`; types `TestOcelEvent`, `OcelLog`, `TestObject`, `TestActivity`,
  `TestObjectType`; `OcelCollector` (a `DiagnosticSink` that captures events), `seal_run()` (seals
  a run into a receipted log + digest), and `project_admission_events()`. Built on the published
  [`wasm4pm-compat 26.6.11`](https://crates.io/crates/wasm4pm-compat) crate, which enforces the
  one-way `Raw → Admitted → Receipted` evidence lifecycle.
- **Process discovery** (feature `ocel-generation-discovery`): `ProcessModelStore` and
  `graduate_for_discovery()` surface `GraduationCandidate`s where the mined process diverges from
  the declared model.
- **Governance module** (`core::governance`): diagnostic/severity primitives for the agent
  governance loop — `Severity`, `DiagnosticCategory`, `DiagnosticCode`, `Diagnostic`,
  `DiagnosticSink`, `TaskReceipt`; a global diagnostic `channel` (`register_sink`,
  `emit_diagnostic`, `set_run_id`, `close_channel`, …); admission `laws` (`AdmissionMetadata`,
  `SubstrateDelta`, `ContributionKind`); and `sector` stacks (`SectorStack`, `MergeStrategy`).
- **Wave orchestration** (`swarm::wave`): N-phase sequential waves with M parallel tasks — `Wave`,
  `WavePhase`, `WaveStatus`, `WaveReceipt`, `PhaseReceipt`, and `ResidualClass` failure
  classification.
- **Full YAWL operator registry** (`operator_registry`): all **43** YAWL workflow control patterns
  registered via `OperatorRegistry`/`OperatorDescriptor`/`OperatorProperties`, each characterized
  by its control-flow law and `GuardType`s; `global_registry()` accessor.
- **`chicago-tdd-lsp`** crate: editor guard that emits `CTDD-DEV-001` when `chicago-tdd-tools`
  appears in `[dependencies]` instead of `[dev-dependencies]` (built on lsp-max).
- Feature flags `ocel-generation` and `ocel-generation-discovery`.
- Documentation: `docs/OCEL.md` (OCEL 2.0 generation guide) and `docs/governance_architecture.md`
  (agent governance architecture).

### Changed
- Migrated OCEL evidence handling from the in-tree vendored `wasm4pm-compat` to the published
  `26.6.11` crate (associated-type `Admit`, `Witness`/`WitnessFamily`,
  `Admission::new().into_evidence()` as the sole path to admitted evidence).
- Consolidated workspace version to `26.6.121`.
- `HttpGet::execute()` (`testing::effects`) now returns an explicit feature-missing error instead
  of silently succeeding when no HTTP client feature is enabled — test-double semantics.
- Wired the testcontainers `Container` type-state machine to a real `GenericContainer`
  (start/exec/get_host_port/stop).
- Tuned build timeouts for the heavier `testcontainers`/`bollard` dependency graph: `fmt` 5s→30s,
  `check` 30s→60s.
- Excluded `chicago-tdd-lsp` from the root workspace build so the root `check`/CI never resolves
  `lsp-max` against the published `wasm4pm-compat` (which lacks the `fresh_names` module the local
  source provides).

### Fixed
- Eliminated **93 stubs and cheats** (69 + 24) across `core`, `testing`, `observability`, and
  `integration` — placeholders replaced with real implementations (verification-pipeline phases 3 &
  4, dog-fooding string-literal scanner, proc-macro panic removal, weaver fixture `Default` panic
  removal, and more).
- Resolved all `--all-features` clippy errors across the `ocel`, `otel`, `weaver`, `property`, and
  `testcontainers` modules (0 warnings under `all`/`pedantic`/`nursery`/`cargo`).
- Fixed proc_macros doc-test imports (test-unit now runs `--lib`).
- Added missing `lint`/`fmt`/`test-unit` task stubs to sub-crate `Makefile.toml` files so the
  workspace runner resolves every root task.

### Security
- Upgraded `testcontainers` `^0.25 → ^0.27` to clear `astral-tokio-tar` advisories.

## [26.6.11] - 2026-06-11

### Changed
- Bumped crate version of chicago-tdd-tools, chicago-tdd-tools-proc-macros, and
  chatman-spec-harness to `26.6.11`.
- Aligned `git2` dependency version in `spec-harness` to `^0.20` to match root `Cargo.toml`.
- Modified dependency on `chicago-tdd-tools` in `playground/Cargo.toml` to add `path = ".."`
  referencing the local copy.

### Removed
- Removed the non-existent `spec_conformance` test target in `spec-harness/Cargo.toml` to avoid
  target discovery errors.

## [1.4.0] - 2025-01-16

### Added - Production-Grade Verification Infrastructure

This release extends the μ-kernel verification substrate with production-grade sector
  implementations, fail-fast hardening, and complete verification pipeline phases. The framework
  now supports real-world workflows with deterministic guarantees and cryptographic proofs.

#### Fail-Fast Hardening Infrastructure (`core::fail_fast`)
- **StrictExecutionContext**: Zero-tolerance execution context with 12-phase verification pipeline
- **47 Invariant Violations**: Comprehensive invariant checking across all phases
- **PhaseResult**: Unified result type for phase execution (Ok or Violation)
- **PhaseLabel**: 12 distinct phases from Contract Definition to Quality Dashboard
- **Receipt Validation**: Self-validating receipts with version and checksum
- Fail-fast semantics: any invariant violation causes immediate test failure
- No degradation, no warnings ignored, no partial success

#### Advanced Verification Phases 7-12
- **Phase 7: Verification Pipeline** - Verify all configured phases executed
- **Phase 8: Continuous Learning** - Validate learner state consistency
- **Phase 9: Distributed Consensus** - Verify 2/3 Byzantine quorum
- **Phase 10: Time-Travel Debugging** - Validate snapshot integrity
- **Phase 11: Performance Prophet** - Verify prediction self-checks
- **Phase 12: Quality Dashboard** - Verify dashboard consistency
- Complete end-to-end verification from contract definition to quality metrics

#### Sector-Grade Reference Stacks (`sector_stacks`)
- **Academic Publishing Workflow** (`sector_stacks::academic`): Complete paper review lifecycle
  - 6 workflow stages: Submission → Desk Review → Reviewer Assignment → Review Collection →
  Decision → Notification
  - Deterministic decision algorithm (3.5+ avg review score = Accept)
  - All-or-nothing decision logic (any rejection → Rejected)
  - 5+ knowledge hooks per major operation
  - 3 theorems mapped to property-based tests
- **Enterprise Claims Processing** (`sector_stacks::claims`): Insurance claims workflow
  - 6 workflow stages: Validation → Fraud Detection → Entitlements → Settlement → Payment → Receipt
  - 7 knowledge hooks per operation
  - 5 guard types: Legality, Budget, Chronology, Causality, Recursion
  - 100+ synthetic test claims configuration
  - Deterministic fraud detection and settlement calculation
- **OperationReceipt**: Generic receipt structure with cryptographic merkle roots
- **SectorOperation Trait**: Extensible trait for sector-specific operations
- Property-based tests proving determinism and reproducibility

#### RDF Integration (`sector_stacks::rdf`)
- **SectorOntology**: Core RDF ontology data structures
- **WorkflowStage**: RDF-driven workflow stage definitions
- **RdfOperationValidator**: Runtime validation against RDF ontologies
- RDF ontologies as single source of truth for workflow definitions
- Oxigraph integration moved to playground (optional tooling, not core dependency)
- 13 new RDF tests (6 ontology + 7 validation)

#### Core Ontology & Operator Registry
- **Operator Registry**: Global singleton for pattern registration
- **Guard System**: 5 guard types with property tracking
- **12 YAWL Patterns**: Registered patterns with guard definitions
- **ggen Templates**: Rust code and LaTeX documentation generation
- Complete class hierarchy with comprehensive properties
- Single source of truth for workflow patterns

#### Spec Harness (`spec-harness`)
- **Theorem Registry**: Executable theorem-to-test mapping
- **Receipt Generation**: Merkle-rooted proofs for theorem conformance
- **23 Passing Tests**: 100% theorem coverage (17/17 theorems)
- Machine-checkable specification with cryptographic receipts

#### Paper as Self-Hosting RDF Instance
- **Auto-Regeneration**: LaTeX documentation generated from RDF
- **CI Pipeline**: Automated paper regeneration and verification
- **RDF Instance**: Paper represented as RDF for semantic querying
- Complete auditability and reproducibility

#### Swarm Protocol (`swarm`)
- **Distributed Multi-Sector Coordination**: Agent-driven task coordination
- **Task Receipt System**: Cryptographic task receipts
- **Knowledge Hooks**: Composition for multi-sector orchestration
- **Public Task Ledger**: Transparent task tracking

#### Snapshot Testing Improvements (`testing::snapshot`)
- **Enhanced Test Fixtures**: Reusable test data structures
- **Improved Test Organization**: Better AAA pattern alignment
- **Complex Structure Support**: Better handling of nested JSON, enums, and maps
- **Sensitive Data Redaction**: Enhanced redaction capabilities for testing

### Added - Documentation & Examples
- **Phase Summaries**: Comprehensive documentation for Phases 1-4
- **RDF Integration Guide**: Complete guide for RDF-driven validation
- **Sector Stack Examples**: Production-grade workflow implementations
- **Cookbook Updates**: Enhanced patterns and troubleshooting guides
- **Application Guide**: Complete mdbook-based application guide

### Changed
- **RDF Architecture**: Oxigraph moved to playground, core library remains lightweight
- **Module Organization**: Sector stacks integrated into main library
- **Test Organization**: Enhanced snapshot testing with better fixtures
- **Documentation Structure**: Improved organization with phase-based summaries

### Performance
- **Sector Operations**: Deterministic execution with <1ms overhead per operation
- **Receipt Generation**: ~100 μs per receipt (SHA-256 signing)
- **RDF Validation**: Minimal overhead for runtime validation
- **Fail-Fast Pipeline**: <1% overhead for typical test suites

### Migration
- All changes are backward compatible
- Existing tests continue working without modification
- Sector stacks are opt-in (use `sector_stacks` module)
- RDF integration is optional (use `rdf` feature or playground)

### See Also
- [Phase 1 Summary](../archive/PHASE_1_SUMMARY.md)
- [Phase 2 Summary](../archive/PHASE_2_SUMMARY.md)
- [Phase 3 Summary](../archive/PHASE_3_SUMMARY.md)
- [Phase 4 Summary](../archive/PHASE_4_SUMMARY.md)
- [RDF Integration Summary](../process/RDF_INTEGRATION_SUMMARY.md)
- [Release Notes](archive/releases-v1x/ANNOUNCEMENT_v1.4.0.md)

## [1.3.0] - 2025-11-16

### Added - Hyper-Advanced μ-Kernel Verification Substrate

This release transforms Chicago TDD Tools from a "great testing framework" into a **canonical
  verification substrate for A = μ(O)** (Artifacts equal micro-operator of Observations). Six
  hyper-advanced tracks provide compile-time contracts, timing discipline, effect typing, state
  machine validation, cryptographic proofs, and agent-driven orchestration.

#### Track 1: Test Contracts as First-Class Types (`core::contract`)
- **TestContract**: Const-evaluable test descriptors with coverage/invariant declarations
- **TestContractRegistry**: Registry with gap analysis (`uncovered_modules`, `uncovered_invariants`)
- **ResourceEnvelope**: Memory/CPU/network resource constraints
- **Thermal Classification**: Hot/Warm/Cold path designation for timing budgets
- Compile-time coverage analysis prevents gaps in test coverage
- Zero overhead (const evaluation)

#### Track 2: τ-Aware Test Harness (`validation::thermal`)
- **HotPathTest**: Enforces Chatman Constant (τ ≤ 8 ticks) for critical paths
- **WarmPathTest**: Standard paths with heap allocation (τ ≤ 100 ticks)
- **ColdPathTest**: Integration tests without timing constraints
- **HotPathConfig**: Configurable constraints (`max_ticks`, `enforce_no_alloc`,
  `enforce_no_syscall`)
- **ThermalTestError**: Budget violation errors with detailed diagnostics
- RDTSC/CNTVCT cycle-accurate timing integration
- Relaxed configs for test environments (strict enforcement in production)
- ~10 cycle measurement overhead

#### Track 3: Effect-Typed Tests (`testing::effects`)
- **EffectTest<E>**: Type-safe effect constraints via phantom types
- **Effect Markers**: `Pure`, `NetworkRead`, `NetworkWrite`, `StorageRead`, `StorageWrite`,
  `Privileged`
- **EffectCoverageRegistry**: Track which effects are tested across test suite
- **RequiresEffect<E>**: Compile-time enforcement that operations match declared effects
- Zero overhead (phantom types)

#### Track 4: Type-Directed State Machine Testing (`testing::state_machine`)
- **StateMachine<S>**: Phantom-typed states with compile-time valid transition enforcement
- **Transition<From, To>**: Trait defining valid state transitions
- **Schedule**: Concurrent state exploration schedules
- **ModelChecker**: Deterministic concurrent state machine testing
- Invalid transitions caught at compile time
- Zero overhead (phantom types)

#### Track 5: Proof-Carrying Test Receipts (`core::receipt`)
- **TestReceipt**: Cryptographic provenance with SHA-256 signatures (placeholder for Ed25519)
- **TestReceiptRegistry**: Governance query API (Γₜ) for deployment decisions
- **EnvironmentFingerprint**: Captured execution environment (OS, arch, Rust version)
- **TimingMeasurement**: τ compliance tracking
- **TestOutcome**: Pass/Fail/Skip with metadata
- Serialization support (JSON) for storage/audit trails
- Governance queries: `tau_violations()`, `failed_receipts()`, `query_by_metadata()`
- ~100 μs per receipt (SHA-256 signing)
- ~2 KB per receipt

#### Track 6: Swarm-Native Test Orchestrator (`swarm::test_orchestrator`)
- **TestOrchestrator**: Agent-driven test scheduling with priority/QoS
- **TestPlan**: Test execution plans with resource budgets
- **QoSClass**: Priority levels (Premium/Standard/BestEffort)
- **ResourceBudget**: CPU/memory/network constraints
- **TestPlanningAPI**: Coverage gap analysis and thermal filtering
- **ΔΣ Analysis**: `suggest_tests_for_change()` suggests minimal test sets for code changes
- ~1 ms per plan
- ~1 KB per plan

### Added - Testing & Documentation
- **Integration Tests** (`tests/hyper_advanced_integration.rs`): 19 comprehensive test cases
  covering all 6 tracks
- **Complete Workflow Test**: End-to-end demonstration of Contract → Test → Receipt → Orchestration
  → Governance
- **Comprehensive Documentation** (`docs/features/HYPER_ADVANCED_MICROKERNEL.md`): 400+ line guide
  with theory, examples, and migration path
- **Example Program** (`examples/hyper_advanced_microkernel.rs`): Runnable demonstration of all 6
  tracks
- **Example Documentation** (`examples/hyper_advanced_microkernel.md`): Tutorial with expected
  output

### Changed
- **Prelude Exports**: Added hyper-advanced types to prelude for easy access
  - `testing::effects::*`
  - `testing::state_machine::*`
  - `swarm::test_orchestrator::*`
  - (Track 1, 2, 5 already exported via `core::*` and `validation::*`)
- **Module Organization**: New modules integrated into capability groups
  - `core::contract` (Track 1)
  - `core::receipt` (Track 5)
  - `validation::thermal` (Track 2)
  - `testing::effects` (Track 3)
  - `testing::state_machine` (Track 4)
  - `swarm::test_orchestrator` (Track 6)

### Performance
- **Total Overhead**: <1% for typical test suites
- **Compile-time Features**: Zero overhead (contracts, effects, state machines)
- **Runtime Features**: Minimal overhead (thermal: ~10 cycles, receipts: ~100 μs, orchestrator: ~1
  ms)

### Theory
- **A = μ(O)**: Canonical equation for verification substrate
  - **A (Artifacts)**: Test receipts, contracts, proofs
  - **μ (Micro-operator)**: μ-kernel substrate transforming observations
  - **O (Observations)**: Timing, effects, state transitions
- **Chatman Constant (τ ≤ 8)**: Maximum sustainable cognitive load for hot paths, derived from
  μ-kernel timing discipline
- **Poka-Yoke Design**: Compile-time prevention of invalid states via phantom types
- **80/20 Thinking**: Second idea = sweet spot (demonstrated in 19 integration tests)

### Migration
- All changes are backward compatible
- Existing tests continue working without modification
- Gradual adoption path: start with Track 2 (thermal testing) for immediate value
- Relaxed configs available for test environments while maintaining strict production enforcement

### See Also
- [Feature Documentation](../features/HYPER_ADVANCED_MICROKERNEL.md)
- [Integration Tests](../../tests/hyper_advanced_integration.rs)
- [Example Program](../../examples/hyper_advanced_microkernel.rs)
- [Release Notes](archive/releases-v1x/ANNOUNCEMENT_v1.3.0.md)

## [1.2.0] - 2025-11-14

### Added
- **Coverage Enforcement**: Mandatory 85% line coverage enforcement (up from 70% warning in v1.1.0)
  - Production code target: 90%+ coverage
  - Test utilities target: 80%+ coverage
  - CI/CD enforcement as hard requirement (blocking merges)
  - Coverage strategy documented in `docs/coverage/v1.2.0-coverage-strategy.md`

### Changed
- **Coverage Threshold**: Increased from 70% (warning) to 85% (enforced)
  - 85% coverage catches 95% of bugs (Codecov data)
  - Aligns with industry best practices for testing frameworks
  - Achievable without excessive test maintenance burden
- **CI/CD**: Coverage checks now block merges if threshold not met
  - Hard requirement instead of warning-only
  - Supports Poka-Yoke design (error-proofing) through comprehensive error path testing

### Documentation
- Added coverage strategy documentation (`docs/coverage/v1.2.0-coverage-strategy.md`)
- Updated coverage enforcement guidelines
- Documented rationale for 85% threshold (coverage science, Chicago TDD context)

## [1.1.2] - 2025-11-14

### Fixed
- **Testcontainers**: Fixed compilation errors in testcontainers module
  - Added `Debug` derive for error types
  - Fixed string comparison logic in port tests
  - Improved Docker availability checking with timeout protection and retry logic
  - Enhanced error messages for Docker unavailability scenarios
- **Config Module**: Fixed config module test isolation
  - Changed working directory to isolate config tests
  - Added `tempfile` as dev-dependency for config module tests
  - Declared config module to resolve dead code warnings
- **Test Assertions**: Fixed `assert_that` usage to `assert_that_with_msg` for message arguments
  - Updated all test files to use correct assertion macro
  - Improved test error messages with descriptive assertions

### Enhanced
- **CI/CD Pipeline**: Comprehensive improvements to GitHub Actions workflows
  - FMEA improvements: OIDC authentication, path filters, artifact validation
  - Andon signals implementation for quality monitoring
  - Enhanced error handling and reporting
- **Testcontainers**: Improved Docker availability checking
  - Added timeout protection (5s) to prevent hanging when Docker is unavailable
  - Implemented retry logic with exponential backoff for parallel test execution
  - Better error messages with actionable guidance for Docker setup
- **Documentation**: Added comprehensive guides
  - Timeout enforcement documentation (`docs/features/TIMEOUT_ENFORCEMENT.md`)
  - Observability testing guide (`docs/observability/observability-testing-guide.md`)
  - OTEL/Weaver integration guide (`docs/observability/otel-weaver-guide.md`)
  - CLI testing guide (`docs/testing/cli-testing-guide.md`)
  - Coverage strategy documentation (`docs/coverage/v1.2.0-coverage-strategy.md`)

### Changed
- **Build System**: Improved build reliability
  - Enhanced timeout handling in testcontainers Docker checks
  - Better error messages for common failure scenarios
- **Code Quality**: Applied kaizen improvements
  - Extracted magic numbers to named constants (Docker check timeout, retry counts)
  - Improved error message consistency and clarity
  - Enhanced code documentation with kaizen improvement notes

### Documentation
- Added timeout enforcement guide explaining timeout SLAs and patterns
- Added comprehensive observability testing guides (OTEL and Weaver)
- Added CLI testing guide with examples
- Added coverage strategy documentation
- Updated code comments with kaizen improvement documentation

## [1.1.1] - 2025-11-14

### Enhanced
- **`test!` Macro**: Added `Result<(), E>` return type support
  - Tests can now return `Result<(), E>` and use the `?` operator for error propagation
  - Backward compatible: existing tests returning `()` continue to work unchanged
  - Automatic conversion between `()` and `Result<(), E>` return types
  - Improved ergonomics for testing fallible operations
- **`assert_fail!` Macro**: New convenience macro for testing error paths
  - Calls a function and asserts it returns `Err`, then returns the error value
  - Enables concise error path testing without intermediate variables
  - Works seamlessly with `test!` macro's new `Result` return type support
  - Example: `let error = assert_fail!(fallible_function());`

### Documentation
- Added examples demonstrating `Result` return type usage in `test!` macro
- Updated `async_test!` macro documentation to clarify Result return type support
- Added `assert_fail!` macro documentation and examples

## [1.1.0] - 2025-11-10

### Added
- **Weaver Integration**: OpenTelemetry live validation with Weaver (`weaver` feature)
  - `WeaverValidator` for lifecycle management (start/stop)
  - `send_test_span_to_weaver()` helper function for testing
  - Static schema validation via `validate_schema_static()`
  - Automatic Weaver binary download during build (when `weaver` feature enabled)
- **OTEL Validation**: OpenTelemetry span/metric validation (`otel` feature)
  - `SpanValidator` for span validation
  - `MetricValidator` for metric validation
  - Type-safe OTEL types (TraceId, SpanId, SpanContext, etc.)
- **Testcontainers Support**: Docker container integration testing (`testcontainers` feature)
  - Generic container support
  - Port mapping, environment variables, command execution
  - Wait conditions (HTTP health checks, log messages)
  - Automatic cleanup via `Drop` trait
- **Module Reorganization**: Modules organized into capability groups
  - `core/`: Core testing infrastructure
  - `testing/`: Advanced testing techniques
  - `validation/`: Quality & validation
  - `observability/`: Telemetry & observability
  - `integration/`: Integration testing
  - Backward compatibility maintained (all modules re-exported at crate root)
- **Dog Fooding**: Framework tests itself using its own tools
  - All framework tests use `chicago_test!` macros
  - Framework validates its own ergonomics through self-testing

### Changed
- Module organization: Modules moved into capability groups for better discoverability
- Build system: All commands use `cargo make` with timeout protection
- Documentation: Comprehensive documentation updates (README, guides, architecture)
- Test framework: All tests migrated to use `chicago_test!` macro
- Code quality: Extracted magic numbers to named constants for improved readability and
  maintainability
- Tooling: Retired the custom git hook installer; `cargo make` tasks (`pre-commit`, `check`,
  `lint`, `audit`) are now the canonical workflow

### Fixed
- Documentation: Updated outdated reports to reflect actual implementation status
- Test framework: Fixed test organization and consistency
- Dead code: Removed duplicate `andon.rs` module (393 lines) - `alert.rs` is the correct
  implementation

### Documentation
- Added comprehensive README with Chicago TDD principles and dog fooding
- Added architecture documentation
- Added user guides and quick start
- Added SPR (Sparse Priming Representation) methodology guide
- Added dog fooding documentation

## [1.0.0] - 2024-01-01

### Added
- Initial release
- Core testing framework
- Fixtures, builders, assertions
- Test macros
- Property-based testing
- Mutation testing

