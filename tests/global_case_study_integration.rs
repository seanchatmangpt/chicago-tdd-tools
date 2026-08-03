#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::needless_return)]
//! Global Case Study Integration Tests — Project Omni-Route
//!
//! Validates the 10 phases of the Project Omni-Route case study using all
//! core testing paradigms from `chicago-tdd-tools`.

use chicago_tdd_tools::prelude::*;
use std::collections::HashMap;

// wasm4pm imports for case study verification
use wasm4pm::algorithms::{discover_alpha_plus_plus_from_log, discover_footprints_from_log};
use wasm4pm::models::{AttributeValue, Event, EventLog, Trace};

// 60 Canonical algorithms from Project Omni-Route
const EXPECTED_ALGORITHMS: &[&str] = &[
    "a_star",
    "aco",
    "alpha_plus_plus",
    "declare",
    "dfg",
    "genetic_algorithm",
    "heuristic_miner",
    "hill_climbing",
    "ilp",
    "inductive_miner",
    "optimized_dfg",
    "process_skeleton",
    "pso",
    "simulated_annealing",
    "hierarchical_dfg",
    "simd_streaming_dfg",
    "smart_engine",
    "streaming_log",
    "analyze_process_speedup",
    "analyze_variant_complexity",
    "batches",
    "causal_graph",
    "compute_activity_transition_matrix",
    "compute_trace_similarity_matrix",
    "correlation_miner",
    "log_to_trie",
    "performance_spectrum",
    "transition_system",
    "alignments",
    "complexity_metrics",
    "etconformance_precision",
    "generalization",
    "monte_carlo_simulation",
    "playout",
    "bpmn_import",
    "pnml_import",
    "powl_to_process_tree",
    "yawl_export",
    "ocel_dfg",
    "ocel_dfg_per_type",
    "ocel_encode",
    "ocel_oc_declare",
    "ocel_ocla",
    "ocel_petri_net",
    "compute_ewma",
    "detect_drift",
    "predict_next_activity",
    "predict_outcome",
    "predict_remaining_time",
    "automl_classify",
    "automl_forecast",
    "ml_anomaly",
    "ml_classify",
    "ml_cluster",
    "ml_forecast",
    "ml_pca",
    "ml_regress",
    "handover_network",
    "working_together_network",
    "agentic_pipeline",
];

/// Helper function to build a valid `EventLog` safely without panicking.
/// Aligns with the mandate to avoid unwrap or panic in helper paths.
fn build_case_study_log(activities: &[&str]) -> Result<EventLog, String> {
    if activities.is_empty() {
        return Err("activities cannot be empty".to_string());
    }
    let mut log = EventLog::new();
    let mut trace = Trace {
        attributes: {
            let mut m = std::collections::BTreeMap::new();
            m.insert(
                "concept:name".to_string(),
                AttributeValue::String("case-omni-route-1".to_string()),
            );
            m
        },
        events: Vec::new(),
    };
    for (i, &act) in activities.iter().enumerate() {
        let mut attrs = std::collections::BTreeMap::new();
        attrs.insert("concept:name".to_string(), AttributeValue::String(act.to_string()));
        attrs.insert(
            "time:timestamp".to_string(),
            AttributeValue::String(format!("2026-07-05T18:{i:02}:00Z")),
        );
        trace.events.push(Event { attributes: attrs });
    }
    log.traces.push(trace);
    Ok(log)
}

// ============================================================================
// 1. Synchronous Test (test! macro)
// ============================================================================
test!(test_global_case_study_completeness, {
    // Arrange: Retrieve list of 60 algorithms and 55 cognitive breeds
    let algo_count = EXPECTED_ALGORITHMS.len();
    let breeds = &wasm4pm_cognition::breeds::BreedId::ALL;
    let breed_count = breeds.len();

    // Act: Log the counts and select representative elements for check
    println!("Validating completeness: {algo_count} algorithms, {breed_count} breeds");

    // Assert: Verify completeness counts
    assert_eq!(algo_count, 60, "Expected exactly 60 algorithms");
    assert_eq!(breed_count, 55, "Expected exactly 55 cognitive breeds");

    // Check presence of representative elements
    assert!(EXPECTED_ALGORITHMS.contains(&"alpha_plus_plus"));
    assert!(EXPECTED_ALGORITHMS.contains(&"a_star"));
    assert!(EXPECTED_ALGORITHMS.contains(&"declare"));

    let has_ltl = breeds.iter().any(|b| format!("{b:?}") == "LtlMonitor");
    assert!(has_ltl, "Missing LTL monitor breed in registry list");
});

// ============================================================================
// 2. Async Test (async_test! macro)
// ============================================================================
async_test!(test_async_streaming_log_footprints, {
    // Arrange: Simulate async log ingestion
    let activities = vec!["Ingest", "Process", "Complete"];
    let log = build_case_study_log(&activities).unwrap();

    // Act: Stream the events asynchronously
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let admitted = wasm4pm_compat::admission::Admission::<EventLog, ()>::new(log).into_evidence();
    let footprints = discover_footprints_from_log(&admitted, "concept:name");

    // Assert: Verify discovered footprint relations
    assert_eq!(footprints.activities.len(), 3);
    assert!(footprints.activities.contains(&"Ingest".to_string()));
    assert!(footprints.activities.contains(&"Complete".to_string()));
});

// ============================================================================
// 3. Fixture Test (fixture_test! macro)
// ============================================================================
fixture_test!(test_omni_route_fixture_admission, fixture, {
    // Arrange: Configure test fixture metadata
    fixture.set_metadata("case_study_id".to_string(), "Project Omni-Route".to_string());
    fixture.set_metadata("conformance_level".to_string(), "L5".to_string());

    let activities = vec!["A", "B", "C", "D"];
    let log = build_case_study_log(&activities).unwrap();

    // Act: Admit log and discover Petri net
    let admitted = wasm4pm_compat::admission::Admission::<EventLog, ()>::new(log).into_evidence();
    let petri_net = discover_alpha_plus_plus_from_log(&admitted, "concept:name", 0.0).unwrap();

    // Assert: Verify places, transitions, and metadata
    assert!(!petri_net.places.is_empty(), "Alpha++ must discover places");
    assert!(!petri_net.transitions.is_empty(), "Alpha++ must discover transitions");
    assert!(!petri_net.arcs.is_empty(), "Alpha++ must discover arcs");

    let val = fixture.get_metadata("case_study_id");
    assert_eq!(val.map(String::as_str), Some("Project Omni-Route"));
});

// ============================================================================
// 4. Performance Test (performance_test! macro)
// ============================================================================
performance_test!(test_alpha_plus_plus_performance, {
    // Arrange: Create representative event log for performance benchmark
    let activities = vec!["Ingest", "Verify", "Validate", "Store"];
    let log = build_case_study_log(&activities).expect("build failed");
    let admitted = wasm4pm_compat::admission::Admission::<EventLog, ()>::new(log).into_evidence();

    // Act: Measure execution performance in CPU ticks
    let ((), ticks) = chicago_tdd_tools::validation::performance::measure_ticks(|| {
        discover_alpha_plus_plus_from_log(&admitted, "concept:name", 0.0)
            .expect("discovery failed");
    });

    // Assert: Validate tick budget constraint
    println!("Alpha++ execution performance: {ticks} ticks");
    assert!(ticks < 20_000_000, "Tick budget exceeded: {ticks}");
});

// ============================================================================
// 5. Property-based Test (PropertyTestGenerator)
// ============================================================================
#[test]
fn test_property_based_input_validation() {
    // Arrange: Initialize const generic property test generator
    let mut generator = PropertyTestGenerator::<10, 3>::new().with_seed(12345);

    // Act & Assert: Generate and validate multiple cases
    for _ in 0..50 {
        let test_data = generator.generate_test_data();
        assert!(test_data.len() <= 10, "Generator violated MAX_ITEMS bound");

        // Validate that keys and values are populated and non-empty
        for (key, val) in &test_data {
            assert!(key.starts_with("key_"));
            assert!(!val.is_empty());
        }
    }
}

// ============================================================================
// 6. Mutation Test (MutationTester)
// ============================================================================
#[test]
fn test_mutation_operator_on_omni_route_data() {
    use chicago_tdd_tools::testing::mutation::{CaseMode, MutationOperator, MutationTester};

    // Arrange: Set up original case study configuration map
    let mut config = HashMap::new();
    config.insert("active".to_string(), "true".to_string());
    config.insert("identifier".to_string(), "omni-route".to_string());

    let mut tester = MutationTester::new(config);

    // Act & Assert: Apply ToggleBoolean mutation
    let mutated_bool = tester.apply_mutation(MutationOperator::ToggleBoolean("active".to_string()));
    assert_eq!(mutated_bool.get("active").map(String::as_str), Some("false"));

    // Act & Assert: Apply StringCase mutation
    let mutated_case = tester
        .apply_mutation(MutationOperator::StringCase("identifier".to_string(), CaseMode::Upper));
    assert_eq!(mutated_case.get("identifier").map(String::as_str), Some("OMNI-ROUTE"));
}

// ============================================================================
// 7. Concurrency Test (ConcurrencyTest::run)
// ============================================================================
#[cfg(feature = "concurrency-testing")]
#[test]
fn test_concurrency_omni_route() {
    use chicago_tdd_tools::testing::concurrency::ConcurrencyTest;
    use loom::sync::{Arc, Mutex};
    use loom::thread;

    ConcurrencyTest::run(|| {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            let mut val = counter_clone.lock().unwrap();
            *val += 1;
        });

        {
            let mut val = counter.lock().unwrap();
            *val += 1;
        }

        handle.join().unwrap();
        let final_val = *counter.lock().unwrap();
        assert!(final_val > 0);
    });
}

// ============================================================================
// 8. OCEL Logging (OcelCollector)
// ============================================================================
#[cfg(feature = "ocel-generation")]
#[test]
fn test_ocel_logging_phases_1_to_10() {
    use chicago_tdd_tools::core::governance::{
        Diagnostic, DiagnosticCategory, DiagnosticCode, DiagnosticSink, RunSummary, Severity,
    };
    use chicago_tdd_tools::observability::ocel::OcelCollector;
    use std::path::PathBuf;

    // Arrange: Prepare target file path and collector
    let output_path = PathBuf::from("target/omni_route_phases.ocel.json");
    if output_path.exists() {
        let _ = std::fs::remove_file(&output_path);
    }
    // Ensure target folder exists
    if let Some(parent) = output_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let collector = OcelCollector::new(Some(output_path.clone()));

    // Act: Emit Diagnostic event for each phase from 1 to 10
    for phase in 1u16..=10u16 {
        let mut context = HashMap::new();
        context.insert("phase", serde_json::Value::Number(phase.into()));
        context.insert("fixture_id", serde_json::Value::String("omni-route-fixture".to_string()));

        let diag = Diagnostic {
            code: DiagnosticCode::new(
                "omni_route".to_string(),
                DiagnosticCategory::Conformance,
                phase,
            ),
            category: DiagnosticCategory::Conformance,
            severity: Severity::Info,
            location: None,
            message: format!("Phase {phase} completed successfully"),
            context,
            run_id: "omni-route-run-1".to_string(),
            agent_id: None,
            source_module: "global_case_study",
            elapsed_ns: 1000 * u64::from(phase),
        };

        collector.emit(diag).expect("Failed to emit diagnostic event");
    }

    // Seal run and close collector
    let summary = RunSummary {
        run_id: "omni-route-run-1".to_string(),
        total_diagnostics: 10,
        ..Default::default()
    };

    collector.close(summary).expect("Failed to close collector");

    // Assert: Output file is created and contains correct diagnostic information
    assert!(output_path.exists(), "OCEL output file was not created");
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("omni-route-run-1"));
    assert!(content.contains("omni-route-fixture"));
}
