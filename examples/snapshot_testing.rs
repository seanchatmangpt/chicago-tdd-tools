//! # Snapshot Testing Example - Comprehensive Guide
//!
//! Demonstrates snapshot testing using `insta` for Chicago TDD. Snapshot testing is ideal
//! for testing complex data structures and ensuring output stability.
//!
//! ## Tutorial: Getting Started
//!
//! This example demonstrates snapshot testing:
//!
//! 1. **String Snapshots**: Test string output with `assert_matches()`
//! 2. **JSON Snapshots**: Test JSON data with `assert_json_matches()`
//! 3. **Debug Snapshots**: Test debug representations with `assert_debug_matches()`
//! 4. **Custom Settings**: Configure snapshot paths and settings
//!
//! **Run tests**: `cargo test --features snapshot-testing --example snapshot_testing`
//!
//! ## Explanation: Concepts
//!
//! **Snapshot Testing**: Captures output on first run and compares it on subsequent runs.
//! If output changes, the test fails and shows a diff. This is ideal for:
//! - Complex data structures
//! - API responses
//! - Generated code
//! - Configuration files
//!
//! **Snapshot Lifecycle**:
//! 1. First run: Creates snapshot file (`.snap` extension)
//! 2. Subsequent runs: Compares output to snapshot
//! 3. On failure: Shows diff and allows review
//! 4. Update: Use `cargo insta review` to accept changes
//!
//! **SnapshotAssert**: Wrapper around `insta` that provides Chicago TDD-compatible API.
//! Methods match Chicago TDD patterns and integrate with test framework.
//!
//! **Custom Settings**: Configure snapshot paths, file extensions, and other settings
//! using `SnapshotAssert::with_settings()`.
//!
//! ## How-to: Common Tasks
//!
//! - Test string output: See `test_snapshot_string`
//! - Test JSON data: See `test_snapshot_json`
//! - Test debug representation: See `test_snapshot_debug`
//! - Use custom settings: See `test_snapshot_with_settings`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `SnapshotAssert`: Snapshot assertion wrapper
//!
//! **Key Functions**:
//! - `SnapshotAssert::assert_matches(data, name)` - Assert string snapshot matches
//! - `SnapshotAssert::assert_json_matches(data, name)` - Assert JSON snapshot matches
//! - `SnapshotAssert::assert_debug_matches(data, name)` - Assert debug snapshot matches
//! - `SnapshotAssert::with_settings(config, test)` - Configure settings for snapshot
//!
//! **Key Concepts**:
//! - **Snapshot**: Captured output stored in `.snap` files
//! - **Review**: Process of accepting or rejecting snapshot changes
//! - **Diff**: Comparison between current output and snapshot

#[cfg(feature = "snapshot-testing")]
#[allow(unused_imports)] // Example code - imports shown for demonstration
use chicago_tdd_tools::prelude::*;
#[cfg(feature = "snapshot-testing")]
#[allow(unused_imports)] // Example code - imports shown for demonstration
use chicago_tdd_tools::snapshot::SnapshotAssert;

#[cfg(feature = "snapshot-testing")]
fn main() {
    chicago_tdd_tools::alert_info!("Snapshot Testing Example");
    chicago_tdd_tools::alert_info!(
        "Run: cargo test --features snapshot-testing --example snapshot_testing"
    );
}

#[cfg(not(feature = "snapshot-testing"))]
fn main() {
    chicago_tdd_tools::alert_info!(
        "Snapshot testing feature not enabled. Enable with: --features snapshot-testing"
    );
}

#[cfg(feature = "snapshot-testing")]
#[cfg(test)]
mod tests {
    use super::*;

    // Example: String snapshot testing
    //
    // ## How-to: Test String Output
    //
    // Use `SnapshotAssert::assert_matches()` to test string output. On first run,
    // creates a snapshot file. On subsequent runs, compares output to snapshot.
    //
    // ## Reference
    //
    // - **Function**: `SnapshotAssert::assert_matches(data, name)`
    // - **Parameters**:
    //   - `data`: String or `&str` to snapshot
    //   - `name`: Snapshot name (used for file naming)
    // - **Behavior**: Creates snapshot on first run, compares on subsequent runs
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::snapshot::SnapshotAssert;
    //
    // let data = "Hello, World!";
    // SnapshotAssert::assert_matches(&data, "my_snapshot");
    // ```
    #[test]
    fn test_snapshot_string() {
        // Arrange: Create test data
        let data = "Hello, Chicago TDD!";

        // Act: Assert snapshot matches
        // Assert: Verify snapshot (creates snapshot on first run, compares on subsequent runs)
        SnapshotAssert::assert_matches(&data, "test_snapshot_string");
    }

    // Example: JSON snapshot testing
    //
    // ## How-to: Test JSON Data
    //
    // Use `SnapshotAssert::assert_json_matches()` to test JSON data. Formats JSON
    // with pretty printing for readable snapshots.
    //
    // ## Reference
    //
    // - **Function**: `SnapshotAssert::assert_json_matches(data, name)`
    // - **Parameters**:
    //   - `data`: JSON value (e.g., `serde_json::Value`)
    //   - `name`: Snapshot name
    // - **Behavior**: Formats JSON with pretty printing
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::snapshot::SnapshotAssert;
    // use serde_json::json;
    //
    // let data = json!({"key": "value"});
    // SnapshotAssert::assert_json_matches(&data, "my_json_snapshot");
    // ```
    #[test]
    fn test_snapshot_json() {
        // Arrange: Create JSON test data
        let data = serde_json::json!({
            "name": "Chicago TDD Tools",
            "version": "1.0.0",
            "features": ["snapshot-testing", "property-testing"]
        });

        // Act: Assert JSON snapshot matches
        // Assert: Verify JSON snapshot
        SnapshotAssert::assert_json_matches(&data, "test_snapshot_json");
    }

    // Example: Debug representation snapshot testing
    //
    // ## How-to: Test Debug Representation
    //
    // Use `SnapshotAssert::assert_debug_matches()` to test debug representations.
    // Useful for complex data structures that implement `Debug`.
    //
    // ## Reference
    //
    // - **Function**: `SnapshotAssert::assert_debug_matches(data, name)`
    // - **Parameters**:
    //   - `data`: Value implementing `Debug`
    //   - `name`: Snapshot name
    // - **Behavior**: Uses `Debug` trait for snapshot representation
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::snapshot::SnapshotAssert;
    //
    // let data = vec![1, 2, 3];
    // SnapshotAssert::assert_debug_matches(&data, "my_debug_snapshot");
    // ```
    #[test]
    fn test_snapshot_debug() {
        // Arrange: Create complex data structure
        let data = vec![("key1", "value1"), ("key2", "value2"), ("key3", "value3")];

        // Act: Assert debug snapshot matches
        // Assert: Verify debug representation snapshot
        SnapshotAssert::assert_debug_matches(&data, "test_snapshot_debug");
    }

    // Example: Snapshot testing with custom settings
    //
    // ## How-to: Configure Snapshot Settings
    //
    // Use `SnapshotAssert::with_settings()` to configure snapshot paths, file extensions,
    // and other settings. Settings apply only within the closure.
    //
    // ## Reference
    //
    // - **Function**: `SnapshotAssert::with_settings(config, test)`
    // - **Parameters**:
    //   - `config`: Closure that configures `insta::Settings`
    //   - `test`: Closure that runs snapshot assertions
    // - **Settings**: `set_snapshot_path()`, `set_snapshot_suffix()`, etc.
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::snapshot::SnapshotAssert;
    //
    // SnapshotAssert::with_settings(
    //     |settings| {
    //         settings.set_snapshot_path("custom_snapshots");
    //     },
    //     || {
    //         SnapshotAssert::assert_matches(&"data", "my_snapshot");
    //     },
    // );
    // ```
    #[test]
    fn test_snapshot_with_settings() {
        // Arrange: Create test data
        let data = "test_with_custom_settings";

        // Act: Assert snapshot with custom settings
        // Assert: Verify snapshot with custom path
        SnapshotAssert::with_settings(
            |settings| {
                settings.set_snapshot_path("custom_snapshots");
            },
            || {
                SnapshotAssert::assert_matches(&data, "test_custom_path");
            },
        );
    }

    // ========================================================================
    // v1.4.0 ENHANCEMENTS - Enhanced Fixtures, Complex Structures, Better Organization
    // ========================================================================

    // Example: Using enhanced test fixtures (v1.4.0)
    //
    // ## How-to: Use Test Fixtures
    //
    // v1.4.0 encourages reusable test fixtures for snapshot testing. Fixtures follow
    // Chicago TDD Arrange-Act-Assert pattern and provide consistent test data.
    // Create your own fixtures for reusable test data structures.
    //
    // ## Reference
    //
    // - **Pattern**: Create fixture functions that return consistent test data
    // - **Benefits**: Reusable, maintainable, consistent test data
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::snapshot::SnapshotAssert;
    //
    // fn nested_json_fixture() -> serde_json::Value {
    //     serde_json::json!({
    //         "users": [{"id": 1, "name": "Alice"}],
    //         "metadata": {"count": 1}
    //     })
    // }
    //
    // let data = nested_json_fixture();
    // SnapshotAssert::assert_json_matches(&data, "nested_data");
    // ```
    #[test]
    fn test_snapshot_with_fixtures() {
        // Arrange: Create reusable fixtures (v1.4.0 pattern)
        fn nested_json_fixture() -> serde_json::Value {
            serde_json::json!({
                "users": [
                    {"id": 1, "name": "Alice", "tags": ["admin", "user"]},
                    {"id": 2, "name": "Bob", "tags": ["user"]}
                ],
                "metadata": {"count": 2, "version": "1.0.0"}
            })
        }

        fn simple_json_fixture() -> serde_json::Value {
            serde_json::json!({
                "name": "test",
                "value": 42
            })
        }

        let nested_data = nested_json_fixture();
        let simple_data = simple_json_fixture();

        // Act & Assert: Verify snapshots with fixture data
        SnapshotAssert::assert_json_matches(&nested_data, "test_fixture_nested_json");
        SnapshotAssert::assert_json_matches(&simple_data, "test_fixture_simple_json");
    }

    // Example: Testing complex structures (v1.4.0)
    //
    // ## How-to: Complex Structures
    //
    // v1.4.0 improves support for complex data structures including nested JSON,
    // enums, maps, and custom structs. Better handling of edge cases and formatting.
    //
    // ## Reference
    //
    // - **Complex Types**: Nested structs, enums with variants, maps, arrays
    // - **Better Formatting**: Improved debug representation for complex types
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::snapshot::SnapshotAssert;
    //
    // #[derive(Debug)]
    // struct Inner { value: i32, name: String }
    // #[derive(Debug)]
    // struct Outer { inner: Inner, count: usize }
    //
    // let complex = Outer { inner: Inner { value: 42, name: "test".to_string() }, count: 10 };
    // SnapshotAssert::assert_debug_matches(&complex, "complex_struct");
    // ```
    #[test]
    fn test_snapshot_complex_struct() {
        // Arrange: Create complex nested structure (v1.4.0)
        #[derive(Debug)]
        struct Inner {
            value: i32,
            name: String,
        }

        #[derive(Debug)]
        struct Outer {
            inner: Inner,
            count: usize,
        }

        #[derive(Debug)]
        enum TestEnum {
            Variant1,
            Variant2(String),
            Variant3 { field: i32 },
        }

        use std::collections::BTreeMap;

        let complex_struct =
            Outer { inner: Inner { value: 42, name: "test".to_string() }, count: 10 };

        let enum_variants = vec![
            TestEnum::Variant1,
            TestEnum::Variant2("test".to_string()),
            TestEnum::Variant3 { field: 42 },
        ];

        let mut map_data = BTreeMap::new();
        map_data.insert("key1".to_string(), "value1".to_string());
        map_data.insert("key2".to_string(), "value2".to_string());
        map_data.insert("key3".to_string(), "value3".to_string());

        // Act & Assert: Verify complex structures
        SnapshotAssert::assert_debug_matches(&complex_struct, "test_complex_nested_struct");
        SnapshotAssert::assert_debug_matches(&enum_variants, "test_enum_variants");
        SnapshotAssert::assert_debug_matches(&map_data, "test_map_structure");
    }

    // Example: Improved test organization (v1.4.0)
    //
    // ## How-to: Better Organization
    //
    // v1.4.0 improves test organization with better AAA pattern alignment.
    // Tests are clearer and more maintainable with enhanced fixtures.
    //
    // ## Reference
    //
    // - **AAA Pattern**: Clear Arrange-Act-Assert sections
    // - **Fixture Reuse**: Consistent test data across tests
    // - **Better Comments**: Enhanced documentation in test code
    //
    // # Examples
    //
    // ```rust
    // #[test]
    // fn test_well_organized() {
    //     // Arrange: Use fixtures for consistent data
    //     let data = nested_json_fixture();
    //
    //     // Act: Execute operation
    //     let result = process_data(&data);
    //
    //     // Assert: Verify with snapshot
    //     SnapshotAssert::assert_json_matches(&result, "processed_data");
    // }
    // ```
    #[test]
    fn test_snapshot_improved_organization() {
        // Arrange: Use fixtures for consistent test data (v1.4.0)
        fn input_fixture() -> serde_json::Value {
            serde_json::json!({
                "users": [{"id": 1, "name": "Alice"}],
                "metadata": {"count": 1}
            })
        }

        let input_data = input_fixture();

        // Act: Simulate processing (in real test, would call actual function)
        // For example: let processed = process_json(&input_data);
        let processed_data = input_data.clone(); // Simplified for example

        // Assert: Verify processed data with snapshot
        SnapshotAssert::assert_json_matches(&processed_data, "test_organized_processing");
    }

    // Example: Sensitive data redaction (v1.4.0)
    //
    // ## How-to: Redact Sensitive Data
    //
    // v1.4.0 enhances sensitive data redaction capabilities. Use fixtures
    // with sensitive data and apply redactions before snapshotting.
    //
    // ## Reference
    //
    // - **Function**: `SnapshotAssert::assert_with_redaction()`
    // - **Helper**: `SnapshotAssert::common_redactions()`
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::snapshot::SnapshotAssert;
    // use std::collections::HashMap;
    //
    // let sensitive = json!({"id": "uuid-123", "token": "secret"});
    // let redactions = SnapshotAssert::common_redactions();
    // SnapshotAssert::assert_with_redaction(&sensitive, "redacted", &redactions);
    // ```
    #[test]
    fn test_snapshot_sensitive_redaction() {
        // Arrange: Create sensitive data (v1.4.0)
        use std::collections::HashMap;

        let sensitive_data = serde_json::json!({
            "id": "uuid-12345",
            "timestamp": "2024-01-01T00:00:00Z",
            "token": "secret-token-abc",
            "message": "test"
        });

        let nested_sensitive = serde_json::json!({
            "user": {
                "id": "uuid-user-123",
                "email": "test@example.com"
            },
            "session": {
                "token": "secret-session-token",
                "created_at": "2024-01-01T00:00:00Z"
            }
        });

        // Get common redactions (v1.4.0)
        let mut redactions = SnapshotAssert::common_redactions();
        redactions.insert(".user.id".to_string(), "[USER_ID]".to_string());
        redactions.insert(".session.token".to_string(), "[SESSION_TOKEN]".to_string());

        // Act & Assert: Verify redacted snapshots
        SnapshotAssert::assert_with_redaction(
            &sensitive_data,
            "test_redacted_sensitive",
            &redactions,
        );
        SnapshotAssert::assert_with_redaction(
            &nested_sensitive,
            "test_redacted_nested",
            &redactions,
        );
    }

    // Example: Inline snapshots for complex structures (v1.4.0)
    //
    // ## How-to: Inline Snapshots
    //
    // v1.4.0 supports inline snapshots for complex structures. Snapshots are
    // stored directly in test source code for quick review.
    //
    // ## Reference
    //
    // - **Function**: `SnapshotAssert::assert_inline_debug()`
    // - **Function**: `SnapshotAssert::assert_inline_json()`
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::snapshot::SnapshotAssert;
    //
    // #[derive(Debug)]
    // struct Complex { inner: Inner, count: usize }
    // let data = Complex { inner: Inner { value: 42 }, count: 10 };
    // SnapshotAssert::assert_inline_debug(&data);
    // ```
    #[test]
    fn test_snapshot_inline_complex_struct() {
        // Arrange: Create complex structure (v1.4.0)
        #[derive(Debug)]
        struct Inner {
            value: i32,
            name: String,
        }

        #[derive(Debug)]
        struct Complex {
            inner: Inner,
            count: usize,
        }

        let complex = Complex { inner: Inner { value: 42, name: "test".to_string() }, count: 10 };

        let json_data = serde_json::json!({
            "users": [{"id": 1, "name": "Alice"}],
            "metadata": {"count": 1}
        });

        // Act & Assert: Verify inline snapshots
        // Note: On first run, insta will write snapshot inline
        SnapshotAssert::assert_inline_debug(&complex);
        SnapshotAssert::assert_inline_json(&json_data);
    }
}
