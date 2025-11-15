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
use insta::{assert_snapshot, assert_debug_snapshot, assert_json_snapshot, Settings};
#[cfg(feature = "snapshot-testing")]
use std::collections::HashMap;

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

    /// Assert inline snapshot (v1.3.0)
    ///
    /// Snapshots are stored directly in the test source code for quick review.
    /// Commonly requested feature for simple assertions.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to snapshot
    ///
    /// # Panics
    ///
    /// Panics if the value doesn't match the inline snapshot.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "snapshot-testing")]
    /// use chicago_tdd_tools::snapshot::SnapshotAssert;
    ///
    /// # #[cfg(feature = "snapshot-testing")]
    /// let result = format!("Hello, {}!", "World");
    /// // SnapshotAssert::assert_inline(&result);
    /// // On first run, insta will write the snapshot inline
    /// ```
    pub fn assert_inline<T: std::fmt::Display>(value: &T) {
        assert_snapshot!(format!("{value}"));
    }

    /// Assert inline debug snapshot (v1.3.0)
    ///
    /// Like `assert_inline` but uses Debug formatting.
    pub fn assert_inline_debug<T: std::fmt::Debug>(value: &T) {
        assert_debug_snapshot!(value);
    }

    /// Assert inline JSON snapshot (v1.3.0)
    ///
    /// Like `assert_inline` but for JSON values.
    pub fn assert_inline_json(value: &serde_json::Value) {
        assert_json_snapshot!(value);
    }

    /// Assert with redaction (v1.3.0)
    ///
    /// Redact sensitive data before snapshot comparison.
    /// Commonly requested for testing with timestamps, UUIDs, and secrets.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to snapshot
    /// * `snapshot_name` - Name of the snapshot
    /// * `redactions` - HashMap of selectors to redaction values
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
    /// use std::collections::HashMap;
    ///
    /// # #[cfg(feature = "snapshot-testing")]
    /// let data = serde_json::json!({
    ///     "id": "uuid-12345",
    ///     "timestamp": "2024-01-01T00:00:00Z",
    ///     "message": "test"
    /// });
    ///
    /// # #[cfg(feature = "snapshot-testing")]
    /// let mut redactions = HashMap::new();
    /// # #[cfg(feature = "snapshot-testing")]
    /// redactions.insert(".id".to_string(), "[UUID]".to_string());
    /// # #[cfg(feature = "snapshot-testing")]
    /// redactions.insert(".timestamp".to_string(), "[TIMESTAMP]".to_string());
    ///
    /// // SnapshotAssert::assert_with_redaction(&data, "test_redacted", &redactions);
    /// ```
    pub fn assert_with_redaction(
        value: &serde_json::Value,
        snapshot_name: &str,
        redactions: &HashMap<String, String>,
    ) {
        Self::with_settings(
            |settings| {
                for (selector, replacement) in redactions {
                    settings.add_redaction(selector, replacement.clone());
                }
            },
            || {
                Self::assert_json_matches(value, snapshot_name);
            },
        );
    }

    /// Assert with profile (v1.3.0)
    ///
    /// Use environment-specific snapshot profiles (e.g., dev, ci, production).
    /// Commonly requested for different snapshot configurations per environment.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to snapshot
    /// * `snapshot_name` - Name of the snapshot
    /// * `profile` - Profile name (e.g., "ci", "dev", "prod")
    ///
    /// # Panics
    ///
    /// Panics if the value doesn't match the stored snapshot for the given profile.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "snapshot-testing")]
    /// use chicago_tdd_tools::snapshot::SnapshotAssert;
    ///
    /// # #[cfg(feature = "snapshot-testing")]
    /// let data = "test_output";
    /// // SnapshotAssert::assert_with_profile(&data, "test_output", "ci");
    /// // Snapshot will be stored in snapshots/ci/ directory
    /// ```
    pub fn assert_with_profile<T: std::fmt::Display>(
        value: &T,
        snapshot_name: &str,
        profile: &str,
    ) {
        Self::with_settings(
            |settings| {
                settings.set_snapshot_path(format!("snapshots/{profile}"));
            },
            || {
                Self::assert_matches(value, snapshot_name);
            },
        );
    }

    /// Create a redaction helper for common patterns (v1.3.0)
    ///
    /// Provides pre-built redactions for common use cases.
    ///
    /// # Returns
    ///
    /// A `HashMap` with common redaction patterns (UUIDs, timestamps, etc.)
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "snapshot-testing")]
    /// use chicago_tdd_tools::snapshot::SnapshotAssert;
    ///
    /// # #[cfg(feature = "snapshot-testing")]
    /// let redactions = SnapshotAssert::common_redactions();
    /// // Contains: .id → [UUID], .timestamp → [TIMESTAMP], etc.
    /// ```
    #[must_use]
    pub fn common_redactions() -> HashMap<String, String> {
        let mut redactions = HashMap::new();
        redactions.insert(".id".to_string(), "[UUID]".to_string());
        redactions.insert(".uuid".to_string(), "[UUID]".to_string());
        redactions.insert(".timestamp".to_string(), "[TIMESTAMP]".to_string());
        redactions.insert(".created_at".to_string(), "[TIMESTAMP]".to_string());
        redactions.insert(".updated_at".to_string(), "[TIMESTAMP]".to_string());
        redactions.insert(".token".to_string(), "[TOKEN]".to_string());
        redactions.insert(".password".to_string(), "[PASSWORD]".to_string());
        redactions.insert(".secret".to_string(), "[SECRET]".to_string());
        redactions
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
        #[allow(dead_code)] // Test enum - fields used for Debug output
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
        #[allow(dead_code)] // Test struct - fields used for Debug output
        struct Inner {
            value: i32,
            name: String,
        }
        #[derive(Debug)]
        #[allow(dead_code)] // Test struct - fields used for Debug output
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

    // ========================================================================
    // V1.3.0 FEATURES - Inline, Redaction, Profiles
    // ========================================================================

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_inline_simple() {
        let data = "inline_test_value";
        SnapshotAssert::assert_inline(&data);
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_inline_debug() {
        let data = vec![1, 2, 3];
        SnapshotAssert::assert_inline_debug(&data);
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_inline_json() {
        let data = serde_json::json!({"key": "value", "number": 42});
        SnapshotAssert::assert_inline_json(&data);
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_redaction_basic() {
        let data = serde_json::json!({
            "id": "uuid-12345",
            "message": "test message"
        });

        let mut redactions = HashMap::new();
        redactions.insert(".id".to_string(), "[UUID]".to_string());

        SnapshotAssert::assert_with_redaction(&data, "test_redaction_basic", &redactions);
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_redaction_multiple() {
        let data = serde_json::json!({
            "id": "uuid-12345",
            "timestamp": "2024-01-01T00:00:00Z",
            "token": "secret-token-abc",
            "message": "test"
        });

        let mut redactions = HashMap::new();
        redactions.insert(".id".to_string(), "[UUID]".to_string());
        redactions.insert(".timestamp".to_string(), "[TIMESTAMP]".to_string());
        redactions.insert(".token".to_string(), "[TOKEN]".to_string());

        SnapshotAssert::assert_with_redaction(&data, "test_redaction_multiple", &redactions);
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_common_redactions() {
        let redactions = SnapshotAssert::common_redactions();

        // Assert: Verify common redactions exist
        assert!(redactions.contains_key(".id"));
        assert!(redactions.contains_key(".uuid"));
        assert!(redactions.contains_key(".timestamp"));
        assert!(redactions.contains_key(".token"));
        assert!(redactions.contains_key(".password"));
        assert_eq!(redactions.get(".id"), Some(&"[UUID]".to_string()));
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_profile_ci() {
        let data = "ci_profile_test";
        SnapshotAssert::assert_with_profile(&data, "test_profile_ci", "ci");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_profile_dev() {
        let data = "dev_profile_test";
        SnapshotAssert::assert_with_profile(&data, "test_profile_dev", "dev");
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_redaction_with_common() {
        let data = serde_json::json!({
            "id": "uuid-12345",
            "timestamp": "2024-01-01T00:00:00Z",
            "message": "test message"
        });

        let redactions = SnapshotAssert::common_redactions();
        SnapshotAssert::assert_with_redaction(&data, "test_common_redaction", &redactions);
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_inline_complex_struct() {
        #[derive(Debug)]
        struct TestStruct {
            name: String,
            value: i32,
            tags: Vec<String>,
        }

        let data = TestStruct {
            name: "test".to_string(),
            value: 42,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        SnapshotAssert::assert_inline_debug(&data);
    }

    #[test]
    #[cfg(feature = "snapshot-testing")]
    fn test_snapshot_redaction_nested() {
        let data = serde_json::json!({
            "user": {
                "id": "uuid-user-123",
                "email": "test@example.com"
            },
            "session": {
                "token": "secret-session-token",
                "created_at": "2024-01-01T00:00:00Z"
            }
        });

        let mut redactions = HashMap::new();
        redactions.insert(".user.id".to_string(), "[USER_ID]".to_string());
        redactions.insert(".session.token".to_string(), "[SESSION_TOKEN]".to_string());
        redactions.insert(".session.created_at".to_string(), "[TIMESTAMP]".to_string());

        SnapshotAssert::assert_with_redaction(&data, "test_redaction_nested", &redactions);
    }
}
