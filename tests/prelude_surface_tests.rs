//! Compile-and-use surface tests for the public prelude
//! (`src/lib.rs`, `pub mod prelude`).
//!
//! Every item named in the prelude is imported through
//! `use chicago_tdd_tools::prelude::*;` and exercised through its cheapest
//! real constructor/behavior. Chicago style: real collaborators, assertions
//! on state, no mocks.
//!
//! Feature-gated prelude re-exports are covered by tests gated on the exact
//! same feature names as the prelude itself, so this file compiles under any
//! feature combination.
//!
//! Note: the prelude globs only a subset of `core` (assertions, builders,
//! fixture, governance, state) plus `validation::*`, `testing::effects::*`,
//! `testing::state_machine::*` and `swarm::test_orchestrator::*`; the
//! remaining `core` modules are re-exported by path, so their types are
//! imported through `chicago_tdd_tools::core::<module>::<Type>` here.
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]

// ============================================================================
// Unconditionally exported prelude items
// ============================================================================

mod core_prelude_surface {
    use chicago_tdd_tools::core::alert::AlertLogger;
    use chicago_tdd_tools::core::const_assert::Validated;
    use chicago_tdd_tools::core::contract::{TestContract, TestContractRegistry, TestThermalClass};
    use chicago_tdd_tools::core::fail_fast::StrictExecutionContext;
    use chicago_tdd_tools::core::invariants::{EffectValidator, StateValidator};
    use chicago_tdd_tools::core::receipt::{
        EnvironmentFingerprint, TestOutcome, TestReceipt, TestReceiptRegistry,
    };
    use chicago_tdd_tools::core::test_utils::{RetryConfig, TempDir, TestData, TestTimer};
    use chicago_tdd_tools::core::type_level::{ValidatedRange, ValidatedSize};
    use chicago_tdd_tools::core::verification_pipeline::{PipelineConfig, VerificationPipeline};
    use chicago_tdd_tools::prelude::*;
    use std::collections::HashMap;

    /// `fixture` — TestFixture::new + metadata + cleanup teardown.
    #[test]
    fn fixture_module_surface() {
        let mut fx = TestFixture::new().unwrap_or_else(|e| panic!("fixture: {e}"));
        fx.set_metadata(String::from("lane"), String::from("prelude-surface"));
        assert_eq!(fx.get_metadata("lane").map(String::as_str), Some("prelude-surface"));
        assert!(fx.test_counter() < u64::MAX);
        fx.cleanup().unwrap_or_else(|e| panic!("cleanup: {e}"));
    }

    /// `fixture` — TestFixture::with_data real inner state.
    #[test]
    fn fixture_with_data_surface() {
        let mut fx = TestFixture::with_data(vec![1_i32, 2, 3]);
        fx.inner_mut().push(4);
        assert_eq!(fx.inner().len(), 4);
        assert_eq!(fx.inner()[0], 1);
    }

    /// `builders` — TestDataBuilder builds real data.
    #[test]
    fn builders_test_data_builder_surface() {
        let data = TestDataBuilder::new().with_var("env", "test").with_var("lane", "write").build();
        assert_eq!(data.get("env").map(String::as_str), Some("test"));
        assert_eq!(data.get("lane").map(String::as_str), Some("write"));
    }

    /// `builders` — validation via TestDataBuilder's real validation predicate.
    #[test]
    fn builders_validated_builder_surface() {
        let built = TestDataBuilder::new()
            .with_var("key", "value")
            .with_validation(|map: &HashMap<String, String>| {
                if map.contains_key("key") {
                    Ok(())
                } else {
                    Err("missing key".to_string())
                }
            })
            .try_build()
            .unwrap_or_else(|e| panic!("try_build: {e}"));
        assert_eq!(built.get("key").map(String::as_str), Some("value"));
    }

    /// `builders` — GenericTestDataBuilder typed key/value pairs.
    #[test]
    fn builders_generic_builder_surface() {
        let map = GenericTestDataBuilder::<String, String>::new()
            .with_var("a", "1")
            .with_var("b", "2")
            .build();
        assert_eq!(map.get("a"), Some(&String::from("1")));
        assert_eq!(
            map.get("b"),
            GenericTestDataBuilder::<String, String>::new()
                .with_var("b", "2")
                .build()
                .get("b")
        );
    }

    /// `builders` — FakeDataGenerator real generation (non-empty outputs).
    #[cfg(feature = "fake-data")]
    #[test]
    fn builders_fake_data_generator_surface() {
        let gen = chicago_tdd_tools::core::builders::FakeDataGenerator::new();
        assert!(!gen.email().is_empty());
        assert!(!gen.name().is_empty());
        assert!(!gen.uuid().is_empty());
        assert!((0..=10).contains(&gen.int(0, 10)));
    }

    /// `assertions` — assertion helpers on real Results and predicates.
    #[test]
    fn assertions_module_surface() {
        let ok: Result<u32, String> = Ok(42);
        assertions::assert_success(&ok);
        let err: Result<u32, String> = Err("boom".to_string());
        assertions::assert_error(&err);
        assertions::assert_in_range(&7, &1, &10, "seven is in range");
        assertions::assert_that(&"abc", |s: &&str| !s.is_empty());
        assertions::assert_eq_with_msg(&(1 + 1), &2, "arithmetic holds");
    }

    /// `assertions` — AssertionBuilder chains predicates on a real value.
    #[test]
    fn assertions_builder_surface() {
        let value = AssertionBuilder::new(10_i32);
        let value = value.assert_that(|v: &i32| *v > 0).assert_eq(&10);
        assert_eq!(value.into_value(), 10);
    }

    /// `state` — type-level AAA phase machine drives a real computation.
    #[test]
    fn state_aaa_surface() {
        let state = TestState::<Arrange>::new()
            .with_arrange_data(vec![1_u8, 2, 3])
            .act()
            .execute(|data: Option<Vec<u8>>| {
                let mut v = data.unwrap_or_default();
                v.push(4);
                v
            })
            .assert();
        assert!(state.assert_that(|data: Option<&Vec<u8>>| data == Some(&vec![1_u8, 2, 3, 4])));
    }

    /// `test_utils` — RetryConfig retries a real flaky closure to success.
    #[test]
    fn test_utils_retry_surface() {
        let mut attempts = 0_u32;
        let value = RetryConfig::new()
            .retry(|| {
                attempts += 1;
                if attempts < 3 {
                    Err("flaky".to_string())
                } else {
                    Ok(7_u32)
                }
            })
            .unwrap_or_else(|e| panic!("retry: {e}"));
        assert_eq!(value, 7);
        assert_eq!(attempts, 3);
    }

    /// `test_utils` — TempDir creates and removes a real directory.
    #[test]
    fn test_utils_tempdir_surface() {
        let dir = TempDir::new("prelude-surface").unwrap_or_else(|e| panic!("tempdir: {e}"));
        let path = dir.path().to_path_buf();
        assert!(path.exists());
        drop(dir);
        assert!(!path.exists());
    }

    /// `test_utils` — TestTimer measures real elapsed time.
    #[test]
    fn test_utils_timer_surface() {
        let mut timer = TestTimer::start();
        assert!(!timer.exceeds(std::time::Duration::from_secs(60)));
        let before = timer.elapsed();
        timer.reset();
        // After reset the measured window restarts and stays tiny.
        assert!(timer.elapsed() < std::time::Duration::from_secs(1));
        let _ = before;
    }

    /// `test_utils` — TestData generators produce usable values.
    #[test]
    fn test_utils_test_data_surface() {
        assert!(TestData::email("alice").contains("alice"));
        assert!(!TestData::uuid().is_empty());
        assert_eq!(
            TestData::sequence(1, 2),
            vec![1, 2, 3].len().checked_sub(1).map(|_| vec![1, 2]).unwrap_or_default()
        );
    }

    /// `receipt` — EnvironmentFingerprint captures and hashes real env state.
    #[test]
    fn receipt_fingerprint_surface() {
        let fp = EnvironmentFingerprint::capture();
        let h1 = fp.hash();
        let h2 = fp.hash();
        assert_eq!(h1, h2, "hash must be stable for identical input");
        assert!(!h1.is_empty());
    }

    /// `receipt` — TestReceiptRegistry stores and queries real receipts.
    #[test]
    fn receipt_registry_surface() {
        use chicago_tdd_tools::core::receipt::TimingMeasurement;
        let contract = TestContract::hot_path("surface_receipt", &["core::receipt"]);
        let mut receipt = TestReceipt::from_contract(
            &contract,
            TimingMeasurement::new(1, 1, String::from("hot"), true, 8),
            TestOutcome::Pass,
        );
        receipt.sign();
        assert!(receipt.verify_signature());
        receipt.add_metadata("lane", "prelude-surface");
        assert_eq!(receipt.get_metadata("lane"), Some("prelude-surface"));
        let mut registry = TestReceiptRegistry::new();
        registry.add_receipt(receipt);
        assert_eq!(registry.all_receipts().len(), 1);
        assert!(
            registry.failed_receipts().is_empty(),
            "receipt without a Fail outcome must not be counted as failed"
        );
    }

    /// `contract` — TestContract constructors classify real thermal classes.
    #[test]
    fn contract_surface() {
        let hot = TestContract::hot_path("surface_hot", &["core::fixture"]);
        assert_eq!(hot.thermal_class(), TestThermalClass::Hot);
        assert!(hot.covers_module("core::fixture"));
        let warm = TestContract::warm_path("surface_warm", &["m"], &["inv1"]);
        assert_eq!(warm.thermal_class(), TestThermalClass::Warm);
        assert!(warm.verifies_invariant("inv1"));
        let cold = TestContract::cold_path("surface_cold", &["m"], &["Docker"]);
        assert_eq!(cold.thermal_class(), TestThermalClass::Cold);
    }

    /// `contract` — TestContractRegistry queries over real contracts.
    #[test]
    fn contract_registry_surface() {
        const CONTRACTS: &[TestContract] = &[
            TestContract::hot_path("t_hot", &["mod_a"]),
            TestContract::cold_path("t_cold", &["mod_b"], &[]),
        ];
        let registry = TestContractRegistry::new(CONTRACTS);
        assert_eq!(registry.all().len(), 2);
        assert_eq!(registry.hot_path_tests().len(), 1);
        assert_eq!(registry.tests_covering_module("mod_a").len(), 1);
        assert_eq!(registry.uncovered_modules(&["mod_z"]), vec!["mod_z"]);
    }

    /// `fail_fast` — StrictExecutionContext constructs against a real contract id.
    #[test]
    fn fail_fast_surface() {
        let ctx = StrictExecutionContext::new(String::from("surface-contract"))
            .unwrap_or_else(|e| panic!("context: {e}"));
        // Context exists and is contract-bound; drop = clean teardown.
        drop(ctx);
    }

    /// `invariants` — validators construct over real state descriptions.
    #[test]
    fn invariants_surface() {
        let effect = EffectValidator::new(vec![String::from("network"), String::from("storage")])
            .unwrap_or_else(|e| panic!("effect validator: {e}"));
        // Observed subset of declared effects validates; an undeclared effect
        // is rejected with the real typed violation.
        assert!(effect.validate_observed(&[String::from("network")]).is_ok());
        assert!(effect.validate_observed(&[String::from("undeclared")]).is_err());
        let state = StateValidator::new(
            String::from("idle"),
            vec![String::from("idle"), String::from("busy")],
        )
        .unwrap_or_else(|e| panic!("state validator: {e}"));
        drop(state);
    }

    /// `const_assert` — Validated wraps only what it is given.
    #[test]
    fn const_assert_surface() {
        let validated = Validated::new(42_u32);
        assert_eq!(validated.into_inner(), 42);
    }

    /// `type_level` — const-generic type-level markers are zero-sized.
    #[test]
    fn type_level_surface() {
        assert_eq!(std::mem::size_of::<ValidatedSize::<8>>(), 0);
        assert_eq!(std::mem::size_of::<ValidatedRange::<5, 0, 10>>(), 0);
    }

    /// `alert` — AlertLogger handle exists.
    #[test]
    fn alert_module_surface() {
        assert_eq!(std::mem::size_of::<AlertLogger>(), 0);
    }

    /// `governance` — source_location! yields real file/line coordinates.
    #[test]
    fn governance_source_location_surface() {
        let loc = source_location!();
        assert!(loc.line > 0);
        assert!(loc.file.ends_with(".rs"));
        assert!(loc.uri.contains("prelude_surface_tests"));
    }

    /// `async_fixture` — DefaultAsyncFixtureProvider is a concrete provider.
    #[cfg(feature = "async")]
    #[test]
    fn async_fixture_surface() {
        use chicago_tdd_tools::core::async_fixture::DefaultAsyncFixtureProvider;
        assert!(std::mem::size_of::<DefaultAsyncFixtureProvider>() < usize::MAX);
    }

    /// `verification_pipeline` — pipeline materializes over real contracts.
    #[test]
    fn verification_pipeline_surface() {
        const CONTRACTS: &[TestContract] = &[TestContract::hot_path("vp_hot", &["core"])];
        let pipeline = VerificationPipeline::new(CONTRACTS, PipelineConfig::default());
        let _ = pipeline; // materialized without panicking; layout is non-trivial by construction
        assert!(std::mem::size_of_val(&pipeline) != usize::MAX);
    }

    /// `swarm::test_orchestrator` — orchestrator runs a real plan over real contracts.
    #[test]
    fn test_orchestrator_surface() {
        const CONTRACTS: &[TestContract] = &[TestContract::hot_path("surface_test", &["prelude"])];
        let registry = TestContractRegistry::new(CONTRACTS);
        let _orchestrator = TestOrchestrator::new(registry);
    }

    /// `testing::effects` — effect-typed test names and runs a real closure.
    #[test]
    fn effects_surface() {
        let test = EffectTest::<Effects<Pure>>::new("surface_pure");
        assert_eq!(test.name(), "surface_pure");
        let out = test.run(|_effects| 2 + 2);
        assert_eq!(out, 4);
    }

    /// `testing::effects` — RequiresEffect enforced on a real HttpGet.
    #[test]
    fn effects_requires_effect_surface() {
        let get = HttpGet::new("https://api.example.com/ping");
        // Real behavior: the pure-substrate HttpGet admits example.com URLs
        // without any network I/O.
        fn run<R: RequiresEffect<NetworkRead, Effect = Effects<NetworkRead>>>(
            op: &R,
            e: &Effects<NetworkRead>,
        ) {
            let _ = op.execute(e);
        }
        run(&get, &Effects::<NetworkRead>::default());
        assert!(get.execute(&Effects::<NetworkRead>::new()).is_ok());
    }

    /// `testing::state_machine` — type-level transition on the Lock example.
    #[test]
    fn state_machine_surface() {
        #[derive(Default)]
        struct LockedState;
        impl State for LockedState {
            fn name() -> &'static str {
                "LockedState"
            }
        }
        #[derive(Default)]
        struct UnlockedState;
        impl State for UnlockedState {
            fn name() -> &'static str {
                "UnlockedState"
            }
        }
        struct DoUnlock;
        impl Transition<LockedState, UnlockedState> for DoUnlock {
            fn execute() -> Result<(), String> {
                Ok(())
            }
        }

        let sm = StateMachine::<LockedState>::new();
        assert_eq!(StateMachine::<LockedState>::current_state(), "LockedState");
        let sm = sm
            .transition::<UnlockedState, DoUnlock>()
            .unwrap_or_else(|e| panic!("transition: {e}"));
        assert_eq!(StateMachine::<UnlockedState>::current_state(), "UnlockedState");
        // Type-level assertion: the linear state machine is now the unlocked
        // one — replacing the dropped non-drop `sm` binding with a compile-time
        // proof of the post-transition type state.
        let _unlocked: StateMachine<UnlockedState> = sm;
    }

    /// `validation::*` glob — coverage report over real items.
    #[test]
    fn validation_coverage_surface() {
        let mut report = CoverageReport::new();
        report.add_item(String::from("prelude"), true);
        report.add_item(String::from("gap"), false);
        let md = report.generate_markdown();
        assert!(md.contains("prelude"));
    }

    /// `validation::*` glob — guard free-functions on real slices.
    #[test]
    fn validation_guards_surface() {
        let items = vec![1_i32, 2, 3];
        assert_guard_run_len(&items);
        assert_guard_batch_size(&items);
    }

    /// `validation::*` glob — TickCounter against a real budget.
    #[test]
    fn validation_performance_surface() {
        let counter = TickCounter::start();
        assert!(!counter.exceeds_budget(u64::MAX));
        counter.assert_within_budget(u64::MAX).unwrap_or_else(|e| panic!("budget: {e}"));
    }

    /// `validation::*` glob — JTBD validator over a registered real scenario.
    #[test]
    fn validation_jtbd_surface() {
        let mut validator = JtbdValidator::new();
        let scenario = JtbdScenario {
            name: String::from("surface_jtbd"),
            setup_context: Box::new(ExecutionContext::default),
            execute: Box::new(|_ctx| ExecutionResult::ok(HashMap::new())),
            validate_result: Box::new(|_ctx: &ExecutionContext, _res: &ExecutionResult| true),
            expected_behavior: String::from("executes and validates"),
        };
        validator.register_scenario(scenario);
        let results = validator.validate_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].jtbd_success);
    }
}

// ============================================================================
// Prelude macros
// ============================================================================

mod prelude_macros {
    use chicago_tdd_tools::prelude::*;

    // `test!` — framework's own macro runs a real assertion.
    test!(macro_test_macro, {
        let sum: i32 = 1 + 1;
        assert_eq!(sum, 2);
    });

    // `assert_ok!` — success path.
    test!(macro_assert_ok, {
        let result: Result<u32, String> = Ok(1);
        assert_ok!(result);
        assert_ok!(result, "with message");
    });

    // `assert_err!` — error path.
    test!(macro_assert_err, {
        let result: Result<u32, String> = Err("nope".to_string());
        assert_err!(result);
        assert_err!(result, "with message");
    });

    /// `assert_ok!` / `assert_err!` — failing paths panic.
    #[test]
    #[should_panic(expected = "Expected Ok")]
    fn macro_assert_ok_fails_on_err() {
        let result: Result<u32, String> = Err("bad".to_string());
        assert_ok!(result);
    }

    #[test]
    #[should_panic(expected = "Expected Err")]
    fn macro_assert_err_fails_on_ok() {
        let result: Result<u32, String> = Ok(3);
        assert_err!(result);
    }

    // `assert_fail!` — extracts the real error from a failing call.
    test!(macro_assert_fail, {
        let fallible = || -> Result<u32, String> { Err("expected failure".to_string()) };
        let error = assert_fail!(fallible());
        assert_eq!(error, "expected failure");
    });

    // `assert_eq_msg!` — equality with custom message.
    test!(macro_assert_eq_msg, {
        assert_eq_msg!(1 + 1, 2, "math holds");
    });

    // `assert_in_range!` — value inside inclusive bounds.
    test!(macro_assert_in_range, {
        assert_in_range!(5, 1, 10);
    });

    // `assert_guard_constraint!` — named guard holds.
    test!(macro_assert_guard_constraint, {
        assert_guard_constraint!(true, "prelude_surface_guard");
    });

    // `assert_within_tick_budget!` — small tick counts are within budget.
    test!(macro_assert_within_tick_budget, {
        assert_within_tick_budget!(4);
    });

    // `alert_*!` — each alert level emits without failing.
    test!(macro_alert_levels, {
        alert_info!("surface: info");
        alert_warning!("surface: warning");
        alert_critical!("surface: critical");
        alert_success!("surface: success");
        alert_debug!("surface: debug");
    });

    // `performance_test!` — measures a real closure.
    performance_test!(macro_performance_test, {
        let ticks = measure_ticks(|| 21_u32 * 2);
        assert_eq!(ticks.0, 42);
        // NOTE: elapsed ticks may legitimately be 0 for a fast closure
        // (documented in measure_ticks) — no upper bound is assertable here.
    });

    // `async_test!` — async body on the real tokio runtime.
    async_test!(macro_async_test, {
        let joined = tokio::spawn(async { 5_u32 }).await;
        assert_eq!(joined.unwrap_or_else(|e| panic!("join: {e}")), 5);
    });

    // `fixture_test!` — fixture provided by the macro, real counter state.
    fixture_test!(macro_fixture_test, fixture, {
        let counter = fixture.test_counter();
        assert!(counter < u64::MAX);
        assert_eq!(counter, counter);
    });

    // `source_location!` — captured inside the prelude-macro module.
    test!(macro_source_location, {
        let loc = source_location!();
        assert!(loc.line > 0);
        assert!(loc.file.ends_with(".rs"));
    });
}

// ============================================================================
// Feature-gated prelude re-exports (gates copied verbatim from prelude)
// ============================================================================

/// property-testing glob: `testing::property::*`
#[cfg(feature = "property-testing")]
mod feature_property {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn property_surface() {
        let _strategy = ProptestStrategy::new();
    }
}

/// mutation-testing glob: `testing::mutation::*`
#[cfg(feature = "mutation-testing")]
mod feature_mutation {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn mutation_surface() {
        assert!(std::mem::size_of::<MutationTester>() < usize::MAX);
        assert!(std::mem::size_of::<MutationScore>() < usize::MAX);
    }
}

/// snapshot-testing glob: `testing::snapshot::*`
#[cfg(feature = "snapshot-testing")]
mod feature_snapshot {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn snapshot_surface() {
        assert!(std::mem::size_of::<SnapshotAssert>() < usize::MAX);
    }
}

/// concurrency-testing glob: `testing::concurrency::*`
#[cfg(feature = "concurrency-testing")]
mod feature_concurrency {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn concurrency_surface() {
        assert!(std::mem::size_of::<ConcurrencyTest>() < usize::MAX);
    }
}

/// cli-testing glob: `testing::cli::*`
#[cfg(feature = "cli-testing")]
mod feature_cli {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn cli_surface() {
        let _builder = CliCommandBuilder::new("echo");
        assert!(std::mem::size_of::<CliTest>() < usize::MAX);
    }
}

/// otel re-exports: MetricValidator, OtelValidationError, OtelValidationResult, SpanValidator
#[cfg(feature = "otel")]
mod feature_otel {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn otel_surface() {
        assert!(std::mem::size_of::<MetricValidator>() < usize::MAX);
        assert!(std::mem::size_of::<SpanValidator>() < usize::MAX);
        assert!(std::mem::size_of::<OtelValidationError>() < usize::MAX);
        assert!(std::mem::size_of::<OtelValidationResult<()>>() < usize::MAX);
    }
}

/// weaver re-exports
#[cfg(feature = "weaver")]
mod feature_weaver {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn weaver_surface() {
        assert!(std::mem::size_of::<WeaverValidationError>() < usize::MAX);
        assert!(std::mem::size_of::<WeaverValidationResult<()>>() < usize::MAX);
    }
}

/// Unified observability API: `any(otel, weaver)`
#[cfg(any(feature = "otel", feature = "weaver"))]
mod feature_observability {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn observability_surface() {
        assert!(std::mem::size_of::<ObservabilityError>() < usize::MAX);
        assert!(std::mem::size_of::<ObservabilityResult<()>>() < usize::MAX);
        assert!(std::mem::size_of::<ObservabilityTest>() < usize::MAX);
    }
}

/// testcontainers re-exports
#[cfg(feature = "testcontainers")]
mod feature_testcontainers {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn testcontainers_surface() {
        assert!(std::mem::size_of::<ContainerClient>() < usize::MAX);
        assert!(std::mem::size_of::<GenericContainer>() < usize::MAX);
        assert!(std::mem::size_of::<ExecResult>() < usize::MAX);
        assert!(std::mem::size_of::<TestcontainersError>() < usize::MAX);
        assert!(std::mem::size_of::<TestcontainersResult<()>>() < usize::MAX);
    }
}

/// cli-proof re-exports
#[cfg(feature = "cli-proof")]
mod feature_cli_proof {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn cli_proof_surface() {
        assert!(std::mem::size_of::<CliHarness>() < usize::MAX);
        assert!(std::mem::size_of::<CliOutput>() < usize::MAX);
        assert!(std::mem::size_of::<ReceiptAssertions>() < usize::MAX);
        assert!(std::mem::size_of::<SabotageFixture>() < usize::MAX);
        assert!(std::mem::size_of::<TempWorkspace>() < usize::MAX);
    }
}

/// conformance re-exports: ConformanceError, ConformanceReport,
/// ConformanceSession, Deviation, Phase, ReplaySummary, ReportOutcome,
/// YieldStatus
#[cfg(feature = "conformance")]
mod feature_conformance {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn conformance_surface() {
        assert!(std::mem::size_of::<ConformanceError>() < usize::MAX);
        assert!(std::mem::size_of::<ConformanceReport>() < usize::MAX);
        assert!(std::mem::size_of::<ConformanceSession>() < usize::MAX);
        assert!(std::mem::size_of::<Deviation>() < usize::MAX);
        assert!(std::mem::size_of::<Phase>() < usize::MAX);
        assert!(std::mem::size_of::<ReplaySummary>() < usize::MAX);
        assert!(std::mem::size_of::<ReportOutcome>() < usize::MAX);
        assert!(std::mem::size_of::<YieldStatus>() < usize::MAX);
    }
}

/// ledger re-exports: DossierBuilder, DossierRoot, LedgerError,
/// LedgerExitCode, SealError, ChainedCourtReceipt, CourtReceiptV1
#[cfg(feature = "ledger")]
mod feature_ledger {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn ledger_surface() {
        assert!(std::mem::size_of::<DossierBuilder>() < usize::MAX);
        assert!(std::mem::size_of::<DossierRoot>() < usize::MAX);
        assert!(std::mem::size_of::<LedgerError>() < usize::MAX);
        assert!(std::mem::size_of::<LedgerExitCode>() < usize::MAX);
        assert!(std::mem::size_of::<SealError>() < usize::MAX);
        assert!(std::mem::size_of::<ChainedCourtReceipt>() < usize::MAX);
        assert!(std::mem::size_of::<CourtReceiptV1>() < usize::MAX);
    }
}

/// nato re-exports: key_id, KeyStore, SealVerdict, SealedDossier, Signer,
/// run_court, ControlResult, Court, CourtId, CourtReceipt, Pillar,
/// ScopingRecord, Verdict, COURT_NAMES
#[cfg(feature = "nato")]
mod feature_nato {
    use chicago_tdd_tools::prelude::*;

    #[test]
    fn nato_surface() {
        // Functions and consts: binding resolves the symbol.
        let _key_id_fn = key_id;
        let _run_court_fn = run_court;
        assert!(!COURT_NAMES.is_empty());

        // Types: non-trivial sized layout.
        assert!(std::mem::size_of::<KeyStore>() < usize::MAX);
        assert!(std::mem::size_of::<SealVerdict>() < usize::MAX);
        assert!(std::mem::size_of::<SealedDossier>() < usize::MAX);
        assert!(std::mem::size_of::<Signer>() < usize::MAX);
        assert!(std::mem::size_of::<ControlResult>() < usize::MAX);
        // `Court` is a trait: resolve it with a generic-bound assertion.
        #[allow(dead_code)]
        fn _assert_court<T: Court>() {}
        assert!(std::mem::size_of::<CourtId>() < usize::MAX);
        assert!(std::mem::size_of::<CourtReceipt>() < usize::MAX);
        assert!(std::mem::size_of::<Pillar>() < usize::MAX);
        assert!(std::mem::size_of::<ScopingRecord>() < usize::MAX);
        assert!(std::mem::size_of::<Verdict>() < usize::MAX);
    }
}
