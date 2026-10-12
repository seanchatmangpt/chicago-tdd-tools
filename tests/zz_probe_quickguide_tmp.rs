// TEMPORARY saturation probe for docs/getting-started/QUICK_GUIDE.md — delete after use.
// Compiles guide snippets verbatim (modulo required boilerplate) against crate @ HEAD.

// === Section: OTEL Test (lines 156-169) ===
mod otel_snippet {
    use chicago_tdd_tools::{otel_test, prelude::*};

    otel_test!(test_otel_span_validation, {
        let span = chicago_tdd_tools::otel::test_helpers::create_test_span("test.operation");
        let helper = chicago_tdd_tools::otel::OtelTestHelper::new();
        helper.assert_spans_valid(&[span.clone()]);
        assert_eq!(span.name, "test.operation");
    });
}

// === Section: State Machine Testing (Track 4, lines 511-519) ===
mod state_machine_snippet {
    use chicago_tdd_tools::prelude::*;
    use std::marker::PhantomData;

    struct Init;
    impl State for Init {
        fn name() -> &'static str {
            "Init"
        }
    }
    struct Connected;
    impl State for Connected {
        fn name() -> &'static str {
            "Connected"
        }
    }
    struct Connect;
    impl Transition<Init, Connected> for Connect {
        fn execute() -> Result<(), String> {
            Ok(())
        }
    }

    #[allow(unused)]
    fn compile_check() {
        let sm: StateMachine<Init> = StateMachine::new();
        let _sm = sm.transition::<Connected, Connect>();
    }

    // phantom to silence unused warnings on imports
    #[allow(dead_code)]
    struct _P(PhantomData<()>);
}

// === Section: Effect-Typed Tests (Track 3, lines 502-508) ===
mod effects_snippet {
    use chicago_tdd_tools::prelude::*;

    #[allow(unused)]
    fn compile_check() {
        let _pure: Effects<Pure> = Effects::new();
        let _network: Effects<NetworkRead> = Effects::new();
    }
}

// === Section: τ-Aware Thermal Testing (Track 2, lines 486-498) ===
mod thermal_snippet {
    use chicago_tdd_tools::prelude::*;

    fn critical_function() -> i32 {
        42
    }

    #[allow(unused)]
    fn compile_check() {
        let relaxed_config = HotPathConfig {
            max_ticks: 1000,
            enforce_no_alloc: false,
            enforce_no_syscall: false,
        };
        let hot_test = HotPathTest::new(relaxed_config);
        let (value, ticks) = hot_test.run(|| critical_function()).unwrap();
        assert!(ticks <= 1000);
        assert_eq!(value, 42);
    }
}

// === Section: Test Receipts (Track 5, lines 522-534) ===
mod receipts_snippet {
    use chicago_tdd_tools::prelude::*;

    const CONTRACT: TestContract = TestContract::hot_path("test_critical", &["module::critical"]);

    #[allow(unused)]
    fn compile_check(timing: chicago_tdd_tools::core::receipt::TimingMeasurement) {
        let mut receipt = TestReceipt::from_contract(&CONTRACT, timing, TestOutcome::Pass);
        receipt.sign();
        receipt.add_metadata("deploy_env", "production");
        let mut registry = TestReceiptRegistry::new();
        registry.add_receipt(receipt);
        let violations = registry.tau_violations();
        let _ = violations.len();
    }
}

// === Section: Hyper-Advanced Track 1 (lines 468-481) ===
mod track1_snippet {
    use chicago_tdd_tools::prelude::*;

    const CONTRACT: TestContract = TestContract::hot_path("test_critical", &["module::critical"]);

    #[allow(unused)]
    fn compile_check() {
        let registry = TestContractRegistry::new(&[CONTRACT]);
        let uncovered = registry.uncovered_modules(&["module::critical", "module::missing"]);
        let _ = uncovered.len();
    }
}
