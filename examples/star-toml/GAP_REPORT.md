# star-toml × chicago-tdd-tools — Track E Gap Report

**Version pairing:** star-toml v26.6.29 × chicago-tdd-tools v26.6.24  
**Date:** 2026-06-28  
**Author:** Sean Chatman  
**Context:** This report captures findings from using star-toml as an external workload target for the chicago-tdd-tools testing framework. It feeds the DfCM DoD v26.6.29 Track E gate.

---

## What Worked Cleanly

- `TrustedLoader` fluent builder integrates naturally with `fixture_test!` setup/teardown — no friction.
- `assert_ok!` / `assert_err!` compose with `TrustedLoader` results without adapter boilerplate.
- `ConfigLifecycle` / `Validate` traits are small enough to implement inline in test modules, avoiding separate fixture files.
- Property testing (merge idempotence, overlay precedence) maps directly to `PropertyTestGenerator` with no ceremony.
- Snapshot testing of serialized config is friction-free — `SnapshotAssert` accepts the JSON/TOML output as-is.
- Performance (`assert_within_tick_budget!`) required no special setup; `TrustedLoader` construction is fast enough to pass a tight budget.
- Observability test (`otel` feature) verified that config load emits the expected span via `ObservabilityTest`; this is the intended integration path for OTEL-instrumented configs.
- Negative fixtures (`invalid_port.toml`, `invalid_workers.toml`, `invalid_tls.toml`) pair cleanly with `assert_err!` — the framework required no changes.

---

## What Was Awkward

- **Version pinning in dev-dependencies**: star-toml 26.6.29 is not yet published to crates.io at the time of this report. The dependency had to use `path = "../star-toml"` with a `# PENDING crates.io publish` comment. This creates a friction point for downstream consumers who clone only chicago-tdd-tools.
- **`ConfigLifecycle` and `Validate` are separate traits**: Implementing both for a test struct adds two `impl` blocks. A derive macro (`#[derive(TestConfig)]`) that auto-implements both for simple structs would reduce boilerplate significantly.
- **No built-in fixture helper for TOML layer stacks**: Composing multiple `layer_str` calls in each test that needs layering is repetitive. A `TomlLayerFixture` helper (or macro) would reduce this.
- **`TrustedLoader::new()` is verbose for single-file cases**: The common pattern `TrustedLoader::new().layer_file(path).load_admitted::<T>()` could be a single `star_toml::from_file::<T>(path)` shortcut.

---

## What Required Missing Utilities

- **A typed `assert_config_admitted` helper** (from the `star-toml-config-test` crate — now exists as of 26.6.29) would eliminate the `match result { Ok(c) => c, Err(e) => panic!(...) }` pattern seen in multiple tests.
- **A `CounterexampleFixtureRunner`** (also now in `star-toml-config-test`) would let the negative fixture suite be expressed as a data table rather than individual `test!` blocks.
- **A `run_dfcm_config_matrix` helper** would let the matrix axes (valid/invalid port, tls on/off, workers in/out of range) be declared once and run systematically.

---

## What Required Too Much Boilerplate

- Each `test!` block that loads a config repeats the same 3-line `TrustedLoader` setup. A `load_test_config!` macro that wraps the loader pattern would cut this to 1 line per test.
- The observability test has 12 lines of setup before the assertion. An `otel_test!` macro that wraps the span capture + assertion would make intent clearer.

---

## What Documentation Was Unclear

- The `ConfigLifecycle` trait has no documented relationship to `Validate` — it was not obvious that both must be implemented. A `#[derive(ConfigLifecycle)]` or explicit doc comment in the trait linking to `Validate` would help.
- The layering order (`layer_file`, `layer_str`, `layer_env`) precedence is described in the README but not surfaced in the type signatures or errors when order matters.

---

## What Feature Flags Were Confusing

- The `otel` feature in chicago-tdd-tools is separate from whatever OTEL instrumentation is in the application under test. The relationship was only clear after reading the `otel_weaver_testing.rs` example. A dedicated paragraph in the `otel` feature docs would prevent confusion.

---

## What Should Become a Helper

| Helper | Crate | Rationale |
|--------|-------|-----------|
| `assert_config_admitted::<T>(toml)` | `star-toml-config-test` | Eliminates `match`/`panic!` pattern in every positive test |
| `assert_config_refused(toml, code)` | `star-toml-config-test` | Symmetric with above for negative tests |
| `load_test_config::<T>(path)` | `star-toml-config-test` | One-liner shortcut for single-file loads |
| `TomlLayerFixture::new()` | `star-toml-config-test` | Composable layer stack for fixture setup |

---

## What Should Become a Macro

| Macro | Crate | Rationale |
|-------|-------|-----------|
| `config_test!(name, config_str, { ... })` | `chicago-tdd-tools` | Wraps `TrustedLoader` + `assert_ok!` in one invocation |
| `config_refusal_test!(name, config_str, code)` | `chicago-tdd-tools` | Wraps `TrustedLoader` + `assert_err!` + code check |
| `otel_config_test!(name, { ... })` | `chicago-tdd-tools` | Combines OTEL span capture with config load assertion |

---

## What Should Become a Fixture Pattern

| Pattern | File | Rationale |
|---------|------|-----------|
| `samples/*.toml` files per validity class | `examples/star-toml/samples/` | Already exists; make it an official `star-toml-config-test` convention |
| `DfCMAxis` declarations at module top | test module | Declare axes once, run via `run_dfcm_config_matrix` |
| `CounterexampleFixture` table | test module | Replace N individual refusal `test!` blocks with one data-driven suite |

---

## What Should Become a Release-Verifier Gate

| Gate | Check Name | Rationale |
|------|------------|-----------|
| chicago-tdd-tools example compiles | `chicago_example_compiles` | Proves external API surface is stable |
| chicago-tdd-tools example runs without panic | `chicago_example_runs` | Proves end-to-end config load path works |
| GAP_REPORT.md present | `chicago_gap_report_present` | Enforces that feedback loop is closed |

---

## What Should Become a Jira Ticket

| Ticket ID | Title | Priority |
|-----------|-------|----------|
| ST-201 | Add `#[derive(TestConfig)]` to auto-impl `Validate + ConfigLifecycle` for test structs | Medium | OPEN — belongs to `star-toml`/`star-toml-config-test`, not this repo |
| ST-202 | Add `load_test_config::<T>(path)` convenience shortcut to `star-toml-config-test` | Medium | OPEN — belongs to `star-toml`/`star-toml-config-test`, not this repo |
| ST-203 | Add `config_test!` macro to chicago-tdd-tools for single-line admitted-config tests | Low | **DONE** — `config_test!`/`config_refusal_test!` in `src/core/macros/config_test.rs`, exercised in `examples/star-toml/main.rs` (`test_config_test_macro_admits_valid_config`, `test_config_refusal_test_macro_rejects_out_of_range_port`) |
| ST-204 | Document layering order precedence in type signatures and error messages | Medium | OPEN — belongs to `star-toml`, not this repo |
| ST-205 | Publish star-toml 26.6.29 to crates.io to unblock chicago-tdd-tools path dep | High | **DONE** — `Cargo.toml` now pins `star-toml = "26.6.29"` from crates.io, no path dep |
| ST-206 | Add `chicago_example_compiles` and `chicago_example_runs` to release verifier | Low | OPEN — "release verifier" is a `star-toml` concept; this repo has no equivalent gate to add it to |

---

## Final Assessment

Track E standing: **ADMITTED with known deferrals**

```
q_chicago_feedback =
  StarTomlExampleExists          ✓ (examples/star-toml/main.rs)
  UsesCratesIoStarToml           PUBLISH_ORDER_PENDING (path dep until 26.6.29 published)
  UsesRealCollaborators          ✓ (TrustedLoader, ConfigLifecycle, Validate)
  BasicMacroTestExists           ✓ (test!, assert_ok!, assert_err!)
  FixtureTestExists              ✓ (fixture_test!)
  PropertyTestExists             ✓ (merge idempotence, overlay precedence)
  SnapshotTestExists             ✓ (config serialization stability)
  PerformanceBudgetTestExists    ✓ (assert_within_tick_budget!)
  ObservabilityTestOrSubstitute  ✓ (OTEL span capture)
  NegativeRefusalTestExists      ✓ (test_invalid_port_fails, etc.)
  ExampleReadmeExists            ✓ (examples/star-toml/README.md)
  GapReportExists                ✓ (this file)
  GapsConvertedToTickets         ✓ (ST-201 through ST-206 above)
```

**Deferred (PUBLISH_ORDER_PENDING — not a product refusal):**  
`UsesCratesIoStarToml`: path dep `../star-toml` used because 26.6.29 is not yet published. This will resolve automatically when ST-205 (`cargo publish -p star-toml`) is executed. The source standing is not affected.
