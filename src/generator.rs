//! Test Code Generator
//!
//! Generates test code from specifications.
//! Uses const fn for compile-time test data generation.

/// Test generator
pub struct TestGenerator {
    /// Generated tests
    tests: Vec<String>,
}

impl TestGenerator {
    /// Create new test generator
    pub fn new() -> Self {
        Self { tests: vec![] }
    }

    /// Generate test from specification
    pub fn generate_test(&mut self, name: &str, spec: &str) -> String {
        format!(
            "#[test]\nfn {}() {{\n    // Generated from: {}\n    // Test implementation needed\n}}\n",
            name, spec
        )
    }

    /// Get all generated tests
    pub fn get_tests(&self) -> &[String] {
        &self.tests
    }
}

impl Default for TestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a test array at compile time
///
/// Uses const fn to generate arrays of any size at compile time.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::generator::generate_test_array;
///
/// const TEST_DATA: [u8; 10] = generate_test_array::<10>();
/// assert_eq!(TEST_DATA.len(), 10);
/// assert_eq!(TEST_DATA[0], 0);
/// assert_eq!(TEST_DATA[1], 1);
/// ```
pub const fn generate_test_array<const N: usize>() -> [u8; N] {
    let mut array = [0u8; N];
    let mut i = 0;
    while i < N {
        array[i] = (i % 256) as u8;
        i += 1;
    }
    array
}

/// Generate a test array with a pattern at compile time
///
/// Generates an array where each element follows a pattern based on its index.
pub const fn generate_test_array_pattern<const N: usize>(pattern: u8) -> [u8; N] {
    let mut array = [0u8; N];
    let mut i = 0;
    while i < N {
        array[i] = pattern.wrapping_add(i as u8);
        i += 1;
    }
    array
}

/// Compile-time validation helper
///
/// Validates a condition at compile time using const assertions.
pub const fn const_assert(condition: bool) {
    if !condition {
        panic!("Compile-time assertion failed");
    }
}

/// Compile-time validation helper with message
pub const fn const_assert_msg(condition: bool, _msg: &'static str) {
    if !condition {
        panic!("Compile-time assertion failed");
    }
}
