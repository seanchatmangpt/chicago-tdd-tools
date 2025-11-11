//! Observability Features Test Suite

pub fn run_observability_tests() {
    println!("  Testing observability features...");
    
    #[cfg(feature = "otel")]
    test_otel();
    
    #[cfg(feature = "weaver")]
    test_weaver();
    
    println!("  ✓ Observability features validated");
}

#[cfg(feature = "otel")]
fn test_otel() {
    use chicago_tdd_tools::observability::otel::test_helpers;
    let span = test_helpers::create_test_span("test");
    assert_eq!(span.name, "test");
}

#[cfg(feature = "weaver")]
fn test_weaver() {
    use chicago_tdd_tools::observability::weaver::WeaverValidator;
    use std::path::PathBuf;
    let validator = WeaverValidator::new(PathBuf::from("registry/"));
    assert!(!validator.is_running());
}

