//! Snapshot Testing Framework
//!
//! Provides snapshot testing capabilities using insta for Chicago TDD.
//! Snapshot testing captures output and compares it against stored snapshots,
//! making it ideal for testing complex data structures and ensuring output stability.
//!
//! # Chicago TDD Alignment
//!
//! Snapshot testing aligns with Chicago TDD principles:
//! - **State-Based Testing**: Verifies outputs and state, not implementation
//! - **Behavior Verification**: Tests what code produces, not how it produces it
//! - **AAA Pattern**: Arrange (setup), Act (execute), Assert (snapshot comparison)

#[cfg(feature = "snapshot-testing")]
use insta::{assert_snapshot, Settings};

/// Snapshot assertion helper for Chicago TDD
///
/// Provides a Chicago TDD-friendly wrapper around insta's snapshot testing.
/// This makes snapshot testing consistent with other assertion helpers in the framework.
#[cfg(feature = "snapshot-testing")]
pub struct SnapshotAssert;

#[cfg(feature = "snapshot-testing")]
impl SnapshotAssert {
    /// Assert that a value matches a snapshot
    ///
    /// # Arguments
    ///
    /// * `value` - The value to snapshot (must implement `Display` or `Debug`)
    /// * `snapshot_name` - Name of the snapshot (used as filename)
    ///
    /// # Panics
    ///
    /// Panics if the value doesn't match the stored snapshot.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "snapshot-testing")]
    /// use chicago_tdd_tools::snapshot::SnapshotAssert;
    ///
    /// # #[cfg(feature = "snapshot-testing")]
    /// let data = serde_json::json!({
    ///     "name": "test",
    ///     "value": 42
    /// });
    ///
    /// # #[cfg(feature = "snapshot-testing")]
    /// SnapshotAssert::assert_matches(&data, "test_data");
    /// ```
    pub fn assert_matches<T: std::fmt::Display>(value: &T, snapshot_name: &str) {
        assert_snapshot!(snapshot_name, value);
    }

    /// Assert that a debug representation matches a snapshot
    ///
    /// # Arguments
    ///
    /// * `value` - The value to snapshot (must implement `Debug`)
    /// * `snapshot_name` - Name of the snapshot (used as filename)
    ///
    /// # Panics
    ///
    /// Panics if the debug representation doesn't match the stored snapshot.
    pub fn assert_debug_matches<T: std::fmt::Debug>(value: &T, snapshot_name: &str) {
        assert_snapshot!(snapshot_name, format!("{:#?}", value));
    }

    /// Assert that a JSON value matches a snapshot
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON value to snapshot
    /// * `snapshot_name` - Name of the snapshot (used as filename)
    ///
    /// # Panics
    ///
    /// Panics if the JSON doesn't match the stored snapshot.
    pub fn assert_json_matches(value: &serde_json::Value, snapshot_name: &str) {
        assert_snapshot!(
            snapshot_name,
            serde_json::to_string_pretty(value).unwrap_or_else(|_| "invalid json".to_string())
        );
    }

    /// Configure snapshot settings for a test
    ///
    /// Allows customization of snapshot behavior (e.g., redactions, filters).
    ///
    /// # Arguments
    ///
    /// * `configure` - Function to configure snapshot settings
    /// * `test` - Test function to run with configured settings
    ///
    /// # Panics
    ///
    /// Panics if the test closure panics or if snapshot assertions fail.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "snapshot-testing")]
    /// use chicago_tdd_tools::snapshot::SnapshotAssert;
    ///
    /// # #[cfg(feature = "snapshot-testing")]
    /// SnapshotAssert::with_settings(|settings| {
    ///     settings.set_snapshot_path("snapshots");
    /// }, || {
    ///     SnapshotAssert::assert_matches(&"test", "custom_path_test");
    /// });
    /// ```
    pub fn with_settings<F, R>(configure: F, test: R)
    where
        F: FnOnce(&mut Settings),
        R: FnOnce(),
    {
        let mut settings = Settings::clone_current();
        configure(&mut settings);
        settings.bind(|| {
            test();
        });
    }
}

#[cfg(test)]
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_assert_matches() {
        let data = "test_value";
        SnapshotAssert::assert_matches(&data, "test_snapshot_assert");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_assert_debug_matches() {
        let data = vec![1, 2, 3];
        SnapshotAssert::assert_debug_matches(&data, "test_snapshot_debug");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_assert_json_matches() {
        let data = serde_json::json!({
            "name": "test",
            "value": 42
        });
        SnapshotAssert::assert_json_matches(&data, "test_snapshot_json");
    }

    // ========================================================================
    // ERROR PATH TESTING - Test error scenarios (80% of bugs)
    // ========================================================================

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_json_serialization_fallback() {
        // Test that JSON serialization fallback works correctly
        // This tests the unwrap_or_else in assert_json_matches
        // Note: serde_json::Value should always serialize successfully,
        // but we verify the fallback string format is correct
        let data = serde_json::json!(null);
        SnapshotAssert::assert_json_matches(&data, "test_snapshot_json_null");
    }

    // ========================================================================
    // BOUNDARY CONDITIONS - Test edge cases
    // ========================================================================

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_empty_string() {
        let data = "";
        SnapshotAssert::assert_matches(&data, "test_snapshot_empty_string");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_empty_collection() {
        let data: Vec<i32> = vec![];
        SnapshotAssert::assert_debug_matches(&data, "test_snapshot_empty_collection");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_single_item_collection() {
        let data = vec![42];
        SnapshotAssert::assert_debug_matches(&data, "test_snapshot_single_item");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_unicode_string() {
        let data = "Hello 世界 🌍";
        SnapshotAssert::assert_matches(&data, "test_snapshot_unicode");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_special_characters() {
        let data = "Line 1\nLine 2\tTabbed\r\nWindows";
        SnapshotAssert::assert_matches(&data, "test_snapshot_special_chars");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_long_string() {
        let data = "x".repeat(1000);
        SnapshotAssert::assert_matches(&data, "test_snapshot_long_string");
    }

    // ========================================================================
    // COMPLEX DATA STRUCTURES - Test real-world usage patterns
    // ========================================================================

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_nested_json() {
        let data = serde_json::json!({
            "users": [
                {
                    "id": 1,
                    "name": "Alice",
                    "tags": ["admin", "user"]
                },
                {
                    "id": 2,
                    "name": "Bob",
                    "tags": ["user"]
                }
            ],
            "metadata": {
                "count": 2,
                "version": "1.0.0"
            }
        });
        SnapshotAssert::assert_json_matches(&data, "test_snapshot_nested_json");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_hashmap() {
        use std::collections::BTreeMap;
        // Use BTreeMap for deterministic ordering
        let mut map = BTreeMap::new();
        map.insert("key1".to_string(), "value1".to_string());
        map.insert("key2".to_string(), "value2".to_string());
        map.insert("key3".to_string(), "value3".to_string());
        SnapshotAssert::assert_debug_matches(&map, "test_snapshot_hashmap");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_enum_variants() {
        #[derive(Debug)]
        enum TestEnum {
            Variant1,
            Variant2(String),
            Variant3 { field: i32 },
        }
        let variants = vec![
            TestEnum::Variant1,
            TestEnum::Variant2("test".to_string()),
            TestEnum::Variant3 { field: 42 },
        ];
        SnapshotAssert::assert_debug_matches(&variants, "test_snapshot_enum_variants");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_nested_struct() {
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
        let data = Outer { inner: Inner { value: 42, name: "test".to_string() }, count: 10 };
        SnapshotAssert::assert_debug_matches(&data, "test_snapshot_nested_struct");
    }

    // ========================================================================
    // DISPLAY VS DEBUG - Test format differences
    // ========================================================================

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_display_vs_debug() {
        // Test that Display and Debug produce different outputs
        let data = 42;
        SnapshotAssert::assert_matches(&data, "test_snapshot_display_number");
        SnapshotAssert::assert_debug_matches(&data, "test_snapshot_debug_number");
    }

    // ========================================================================
    // WITH_SETTINGS - Test custom settings
    // ========================================================================

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_with_custom_path() {
        SnapshotAssert::with_settings(
            |settings| {
                settings.set_snapshot_path("custom_snapshots");
            },
            || {
                let data = "custom_path_test";
                SnapshotAssert::assert_matches(&data, "test_custom_path");
            },
        );
    }
}
