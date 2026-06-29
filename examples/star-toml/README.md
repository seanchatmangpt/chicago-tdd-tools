# Star TOML Integration & Testing Example

This directory contains the `star_toml` integration example demonstrating how to parse, validate, and layer TOML configurations using Chicago-style Test-Driven Development (TDD).

---

## Tutorial: Building, Running, and Testing `star-toml`

This tutorial provides step-by-step instructions to get started with the `star-toml` example in the Chicago TDD Tools framework.

### Prerequisites
- Rust 1.70 or newer
- `cargo` installed and available in your shell path
- No internet access is required (runs fully offline)

### Step 1: Build the Example Binary
To compile the `star_toml` example executable, run:
```bash
cargo build --example star_toml --all-features
```
This builds the binary and places it under `target/debug/examples/star_toml`.

### Step 2: Run the Example with Different Configurations
The `star_toml` example supports configuration layering via CLI flags and environment variables. Try running it with different profiles:

1. **Load the Default Profile:**
   ```bash
   cargo run --example star_toml -- --config examples/star-toml/samples/default.toml
   ```
   *Expected output:* You will see `INFO: Loading config layers`, `SUCCESS: Configuration admitted`, followed by the parsed config struct debug dump, serialised TOML, and the status messages `standing`, `q_config = 1`, and `admitted`.

2. **Layer the Development Profile (Overrides & Warnings):**
   ```bash
   cargo run --example star_toml -- --config examples/star-toml/samples/default.toml --config examples/star-toml/samples/dev.toml
   ```
   *Expected output:* Since the development profile overrides the port to `80` (which is `<= 1024`), you will see:
   `WARNING: prefer a port above 1024`
   but the configuration remains admitted (exit code `0`).

3. **Layer the Production Profile (TLS & Strict Checking):**
   ```bash
   cargo run --example star_toml -- --config examples/star-toml/samples/default.toml --config examples/star-toml/samples/prod.toml
   ```
   *Expected output:* This profile overrides `server.host` to `"0.0.0.0"`, `server.port` to `443`, and configures TLS. It will succeed cleanly since valid certificate and key paths are provided.

4. **Attempt to Load an Invalid Configuration (Refusal):**
   ```bash
   cargo run --example star_toml -- --config examples/star-toml/samples/invalid_port.toml
   ```
   *Expected output:* The binary exits with code `1`, displaying:
   `CRITICAL ERROR: Configuration refused!`
   along with details of the validation error indicating that port `99999` is out of the valid range.

5. **Layer via Environment Variables:**
   You can also override keys using environment variables prefixed with `STAR_TOML_`:
   ```bash
   STAR_TOML_WORKERS=8 cargo run --example star_toml -- --config examples/star-toml/samples/default.toml
   ```
   *Expected output:* The printed config will show `workers: 8` instead of the default `4`.

### Step 3: Run the Test Suites
The example includes both inline unit/integration tests and an end-to-end (E2E) integration test suite.

- **Run Example Unit & Feature Tests:**
  ```bash
  cargo test --example star_toml --all-features
  ```
  This runs the unit, property, snapshot, and performance tests defined inline inside `examples/star-toml/main.rs`.

- **Run End-to-End Integration Tests:**
  ```bash
  cargo test --test star_toml_e2e --all-features
  ```
  This runs all 64 integration tests spanning feature coverage, boundary/corner cases, cross-feature combinations, and real-world application scenarios.

---

## How-To: Adding Configuration Testing to Your Project

Follow these recipes to introduce Chicago-style configuration testing into another project.

### Recipe 1: Define the Configuration Schema & Validation Invariants
First, implement `serde::Deserialize` and `star_toml::Validate` on your configuration structures.

```rust
use serde::{Deserialize, Serialize};
use star_toml::{Validate, Validator, ConfigLifecycle};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: usize,
    pub timeout_seconds: u64,
}

impl Validate for DatabaseConfig {
    fn validate(&self, v: &mut Validator) {
        // Enforce non-empty database connection URL
        v.check_non_empty("url", &self.url);
        // Pool size must be within a safe concurrency range
        v.check_range("pool_size", self.pool_size, 1..=128);
        // Timeout must not be zero
        v.check_range("timeout_seconds", self.timeout_seconds, 1..=600);
    }
}

impl ConfigLifecycle for DatabaseConfig {}
```

### Recipe 2: Set Up the Trusted Config Loader
Configure your application startup path to load layers deterministically, falling back to environment variables.

```rust
fn load_app_config(config_path: Option<&str>) -> Result<DatabaseConfig, star_toml::Error> {
    let mut loader = star_toml::trusted();
    
    // 1. Layer base file if it exists
    if let Some(path) = config_path {
        loader = loader.layer_file(path);
    }
    
    // 2. Layer environment overrides (e.g. APP_POOL_SIZE)
    loader = loader.env_prefix("APP_");
    
    // 3. Load and validate
    let admitted = loader.load_admitted::<DatabaseConfig>()?;
    Ok(admitted.into_value())
}
```

### Recipe 3: Write a Snapshot Test for Default Configs
Freeze the parsed default configuration structure to prevent silent drift in default values.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::testing::snapshot::SnapshotAssert;

    #[test]
    fn test_default_config_snapshot() {
        let loader = star_toml::trusted()
            .layer_file("config/default.toml");
        let admitted = loader.load_admitted::<DatabaseConfig>().unwrap();
        let serialized = toml::to_string(admitted.value()).unwrap();
        
        SnapshotAssert::with_settings(|settings| {
            settings.set_snapshot_path("../tests/snapshots");
            settings.set_prepend_module_to_snapshot(false);
        }, || {
            SnapshotAssert::assert_matches(&serialized, "database_default_config_snapshot");
        });
    }
}
```

### Recipe 4: Write Property-Based Invariant Tests
Ensure configuration merging rules remain consistent across any arbitrary keys.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::testing::property::PropertyTestGenerator;

    chicago_tdd_tools::test!(test_merge_idempotence, {
        let mut gen = PropertyTestGenerator::<5, 2>::new().with_seed(42);
        
        for _ in 0..50 {
            let data = gen.generate_test_data();
            let mut base = toml_1::Value::Table(
                data.into_iter().map(|(k, v)| (k, toml_1::Value::String(v))).collect()
            );
            let overlay = base.clone();
            
            star_toml::deep_merge(&mut base, overlay.clone());
            assert_eq!(base, overlay, "Merging identical configurations must be idempotent");
        }
    });
}
```

---

## Explanation: Why `star-toml` is a Great Chicago TDD Target

This section explores the core testing philosophy behind using `star-toml` as a primary target for Chicago-style TDD (Classicist TDD).

### 1. Real Collaborators vs. Mocks
Chicago-style TDD strongly advocates for testing with **real collaborators** rather than mock objects or mock filesystems.
In the `star_toml` integration tests, we do not mock the filesystem or stub the TOML parser. Instead, we use real sample files (`default.toml`, `dev.toml`, `prod.toml`) and environment variables. The integration tests invoke the compiled binary directly via `Command` subprocess execution.
This guarantees that the actual:
- OS environment variable resolution
- TOML syntax parser (`toml_1`)
- Deep-merge logic
- Invariant validators
all work in unison exactly as they will in production. Mocks frequently drift from real implementation behaviors; real collaborators ensure that if the test suite passes, the production configuration layer is guaranteed to load correctly.

### 2. Configuration as Behavior
We treat configuration not just as passive data, but as **executable system behavior**.
An invalid configuration (e.g. port out of range, TLS enabled without certificates) is a runtime defect that should fail instantly (fail-fast). Rather than letting the application start up in a compromised or broken state, the validator enforces type-level and value-level correctness.
By defining the config lifecycle explicitly via `ConfigLifecycle` and `Validate`, validation is integrated into the core type admission loop. Testing configuration as behavior means we assert proper exit codes (`0` for success, non-zero for refusal) and stderr diagnostic alerts under invalid states.

### 3. Property Testing for Invariants
Layered configurations rely heavily on deep-merging strategies (merging tables, overriding scalar values, and resolving environment prefixes). Writing individual unit tests for every possible permutation of merged tables is unfeasible.
By utilizing **Property-Based Testing**, we verify that key algebraic invariants hold true under all inputs:
- **Idempotence**: Merging configuration layer `A` with itself produces exactly `A`.
- **Overlay Precedence**: Merging base `A` with overlay `B` ensures that for any shared keys, `B`'s values overwrite `A`'s.
This catches subtle merging edge cases (like handling empty tables or conflicting types) automatically without writing manual edge-case tests.

### 4. Snapshots for Stable Projections
When configurations scale to hundreds of parameters, manually writing `assert_eq!(config.field, expected)` assertions leads to verbose, fragile tests that are painful to maintain.
**Snapshot Testing** captures a stable, serialized representation (a projection) of the admitted configuration. If a default value changes, or if nested structure flattening behaves differently, the snapshot assertion fails, presenting a clear diff of the changes. This provides high-coverage regression testing with single-line assertions.

### 5. Performance Budgets
Configuration loading runs synchronously during application boot, blocking incoming traffic or startup readiness checks. It must remain extremely fast even under large files or heavy environment overlays.
Chicago TDD includes **performance budget validation** by measuring CPU ticks (`measure_ticks`) directly in tests. The suite asserts that loading and validation remain strictly within a performance budget (e.g., `< 10,000,000` ticks). If a change introduces slow reflection or redundant parsing loops, the test suite catches the performance regression.

### 6. Observability
Configuration status (admitted, warned, or refused) is a critical diagnostic event.
The example integrates OpenTelemetry (OTEL) tracking. Under the hood, unified spans trace the duration of config parsing, log warning alerts, and capture structural validation failures. By verifying the observability tracing invariants in the test suite itself, we ensure production operators will have complete diagnostics if a system fails to start.

---

## Reference: Commands, Features, and Test Registry

A comprehensive reference of the testing surface area of `star-toml`.

### Command Cheat Sheet
- **Build example binary:**
  ```bash
  cargo build --example star_toml --all-features
  ```
- **Run inline unit/property/performance tests:**
  ```bash
  cargo test --example star_toml --all-features
  ```
- **Run E2E integration test suite:**
  ```bash
  cargo test --test star_toml_e2e --all-features
  ```
- **Verify single test case (e.g., production profile):**
  ```bash
  cargo test --test star_toml_e2e test_r4_production_profile
  ```

### Cargo Features Used
- `property-testing`: Enables the `proptest` and `PropertyTestGenerator` logic for configuration merge property assertions.
- `snapshot-testing`: Enables `insta` snapshot assertions for config file output stability.
- `otel`: Enables the unified OpenTelemetry tracing and span checks for configuration loads.

### Test Registry

#### Inline Tests (`examples/star-toml/main.rs`)
| Test Name | Feature Verified | Verification Method |
|---|---|---|
| `test_basic_load_succeeds` | Basic load of `default.toml` | `assert_ok!` |
| `test_fixture_based` | Temporary file loader lifecycle | `chicago_tdd_tools::fixture_test!` |
| `test_property_idempotence` | Deep-merge idempotency invariant | `PropertyTestGenerator` |
| `test_property_overriding_behavior` | Merge overriding correctness | `ProptestStrategy` (proptest) |
| `test_snapshot_config` | Stability of default config projection | `SnapshotAssert` |
| `test_load_performance` | Parser execution tick budget | `measure_ticks` (< 10M ticks) |
| `test_observability` | Unified OTEL provider capability | `ObservabilityTest` |
| `test_invalid_port_fails` | Boundary error rejection | `assert_err!` |

#### E2E Integration Tests (`tests/star_toml_e2e.rs`)
The E2E suite consists of 64 tests divided into the following categories:

1. **Tier 1: Feature Coverage (25 tests)**
   - **F1: TOML Loading & Parsing**: `test_f1_load_valid_toml_file`, `test_f1_load_multiple_configs`, `test_f1_parse_syntax_error`, `test_f1_parse_env_var_expansion`, `test_f1_parse_nonexistent_file`
   - **F2: Layering & Merging**: `test_f2_merge_scalar_override`, `test_f2_merge_nested_table`, `test_f2_merge_env_override`, `test_f2_merge_determinism`, `test_f2_merge_overlapping_keys`
   - **F3: Configuration Validation**: `test_f3_validate_valid_config`, `test_f3_validate_missing_required_field`, `test_f3_validate_port_out_of_range`, `test_f3_validate_workers_out_of_range`, `test_f3_validate_tls_paths_required`
   - **F4: Progress Alerts/Logging**: `test_f4_progress_alert_success`, `test_f4_progress_alert_warning_port`, `test_f4_progress_alert_info_loading`, `test_f4_progress_alert_error_refusal`, `test_f4_progress_alert_custom_format`
   - **F5: Accepted/Refused Behavior**: `test_f5_accepted_standing_verification`, `test_f5_refused_invalid_port`, `test_f5_refused_invalid_workers`, `test_f5_refused_invalid_tls`, `test_f5_refused_exit_code_non_zero`

2. **Tier 2: Boundary & Corner Cases (25 tests)**
   - File states: `test_b2_empty_toml`, `test_b2_whitespace_only_toml`, `test_b2_read_only_config_file`, `test_b2_directory_path_as_file`, `test_b2_large_file_parsing`
   - Numeric limits: `test_b2_port_lower_boundary_zero`, `test_b2_port_upper_boundary_65536`, `test_b2_workers_lower_boundary_zero`, `test_b2_workers_upper_boundary_1025`, `test_b2_workers_exact_max_1024`, `test_b2_workers_exact_min_1`, `test_b2_port_advisory_boundary_1023`, `test_b2_port_advisory_boundary_1024`
   - Syntactic/Type errors: `test_b2_missing_key_delimiters`, `test_b2_unclosed_string`, `test_b2_non_utf8_toml`, `test_b2_float_instead_of_integer`, `test_b2_boolean_instead_of_integer`, `test_b2_mixing_array_and_table`, `test_b2_log_level_case_sensitivity`
   - Env weirdness: `test_b2_override_empty_env`, `test_b2_weird_env_reference`, `test_b2_multiple_same_key_overrides`
   - Security bounds: `test_b2_path_traversal_attempt`, `test_b2_special_characters_app_name`

3. **Tier 3: Cross-Feature Combinations (5 tests)**
   - `test_c3_merge_valid_and_invalid`
   - `test_c3_merge_env_override_invalid`
   - `test_c3_merge_tls_components`
   - `test_c3_multiple_errors_from_different_layers`
   - `test_c3_merge_env_var_expansion_invalid_format`

4. **Tier 4: Real-World Application Scenarios (5 tests)**
   - Profiles: `test_r4_production_profile`, `test_r4_development_profile`
   - Deployment contexts: `test_r4_local_override_flow`, `test_r4_ci_environment_strict`, `test_r4_microservice_scaling`

5. **Specialist Integration Tests (4 tests)**
   - Merging properties: `test_property_based_merge_determinism`
   - Stability verification: `test_snapshot_normalized_merged_output`
   - Perf boundaries: `test_performance_budget_validation_ticks`
   - Trace assertions: `test_observability_configuration_load`
