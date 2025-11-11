//! Core Features Test Suite

use chicago_tdd_tools::prelude::*;

pub fn run_core_tests() {
    println!("  Testing core features...");
    
    // Fixtures
    test_fixtures();
    
    // Builders
    test_builders();
    
    // Assertions
    test_assertions();
    
    // State
    test_state();
    
    // Type level
    test_type_level();
    
    // Const assert
    test_const_assert();
    
    // Alert
    test_alert();
    
    println!("  ✓ Core features validated");
}

fn test_fixtures() {
    let fixture = TestFixture::new().unwrap();
    assert!(fixture.test_counter() >= 0);
}

fn test_builders() {
    let data = TestDataBuilder::new()
        .with_var("key", "value")
        .build_json()
        .unwrap();
    assert!(data.is_object());
}

fn test_assertions() {
    let result: Result<u32, String> = Ok(42);
    assert_success(&result);
}

fn test_state() {
    let state = TestState::<Arrange>::new();
    let _act_state = state.act();
}

fn test_type_level() {
    const ARRAY: chicago_tdd_tools::core::type_level::SizeValidatedArray<8, 8> =
        chicago_tdd_tools::core::type_level::SizeValidatedArray::new([0u8; 8]);
    assert_eq!(ARRAY.size(), 8);
}

fn test_const_assert() {
    chicago_tdd_tools::core::const_assert::const_assert(true);
}

fn test_alert() {
    alert_info!("Test alert");
}

