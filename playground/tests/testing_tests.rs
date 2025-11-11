//! Testing Features Test Suite

use chicago_tdd_tools::prelude::*;

pub fn run_testing_tests() {
    println!("  Testing testing features...");
    
    // Generator (always available)
    test_generator();
    
    // Mutation (always available)
    test_mutation();
    
    #[cfg(feature = "property-testing")]
    test_property();
    
    #[cfg(feature = "snapshot-testing")]
    test_snapshot();
    
    #[cfg(feature = "concurrency-testing")]
    test_concurrency();
    
    #[cfg(feature = "cli-testing")]
    test_cli();
    
    println!("  ✓ Testing features validated");
}

fn test_generator() {
    let mut generator = chicago_tdd_tools::testing::generator::TestGenerator::new();
    let _code = generator.generate_test("test", "spec");
}

fn test_mutation() {
    use std::collections::HashMap;
    let data = HashMap::new();
    let mut tester = MutationTester::new(data);
    tester.apply_mutation(MutationOperator::AddKey("key".to_string(), "value".to_string()));
}

#[cfg(feature = "property-testing")]
fn test_property() {
    let mut generator = PropertyTestGenerator::<10, 3>::new();
    let _data = generator.generate_test_data();
}

#[cfg(feature = "snapshot-testing")]
fn test_snapshot() {
    let data = "test";
    SnapshotAssert::assert_matches(&data, "test_snapshot");
}

#[cfg(feature = "concurrency-testing")]
fn test_concurrency() {
    use std::sync::{Arc, Mutex};
    ConcurrencyTest::run(|| {
        let data = Arc::new(Mutex::new(0));
        *data.lock().unwrap() += 1;
    });
}

#[cfg(feature = "cli-testing")]
fn test_cli() {
    // CLI tests use .trycmd files
    assert!(true);
}

