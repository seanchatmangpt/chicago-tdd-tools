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
}
