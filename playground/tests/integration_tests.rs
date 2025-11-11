//! Integration Features Test Suite

pub fn run_integration_tests() {
    println!("  Testing integration features...");
    
    #[cfg(feature = "testcontainers")]
    test_testcontainers();
    
    println!("  ✓ Integration features validated");
}

#[cfg(feature = "testcontainers")]
fn test_testcontainers() {
    use chicago_tdd_tools::integration::testcontainers::*;
    let client = ContainerClient::new();
    // Note: Actual container creation requires Docker
    // This validates the API is available
    assert!(client.client().version().is_ok());
}

