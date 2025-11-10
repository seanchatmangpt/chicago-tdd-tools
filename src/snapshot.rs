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
}
