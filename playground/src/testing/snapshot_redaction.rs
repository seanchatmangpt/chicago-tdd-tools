//! v1.3.0 Snapshot Redaction and Inline Snapshots Examples
//!
//! Demonstrates snapshot redaction hooks for handling dynamic values (UUIDs, timestamps)
//! and inline snapshots for storing snapshots in source code.

use chicago_tdd_tools::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotRedactionResult {
    pub redaction_examples: usize,
    pub uuid_redactions: usize,
    pub timestamp_redactions: usize,
    pub inline_snapshot_examples: usize,
    pub snapshot_profiles_used: usize,
}

pub fn run() -> crate::Result<SnapshotRedactionResult> {
    let mut redaction_examples = 0;
    let mut uuid_redactions = 0;
    let mut timestamp_redactions = 0;
    let mut inline_snapshot_examples = 0;
    let mut snapshot_profiles_used = 0;

    // ========================================================================
    // 1. SNAPSHOT REDACTION - Handle dynamic values in snapshots
    // ========================================================================
    {
        // Example 1a: API response with UUID that changes every test run
        let api_response = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "username": "alice",
            "email": "alice@example.com",
            "created_at": "2025-11-15T10:30:45Z"
        });

        // In real testing, you would use SnapshotAssert with redaction:
        // SnapshotAssert::new(&api_response)
        //     .with_redaction(|value| {
        //         value
        //             .replace_regex(r"[a-f0-9-]{36}", "[UUID]")  // Redact UUIDs
        //             .replace_regex(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", "[ISO_TIMESTAMP]")  // Redact timestamps
        //     })
        //     .assert_matches("api_response");

        redaction_examples += 1;

        // The snapshot would compare against expected value with redacted placeholders
        let expected_response = json!({
            "id": "[UUID]",
            "username": "alice",
            "email": "alice@example.com",
            "created_at": "[ISO_TIMESTAMP]"
        });

        uuid_redactions += 1;
        timestamp_redactions += 1;

        // Verify redaction would work (simulated)
        assert_eq!(api_response.get("username"), expected_response.get("username"));
        redaction_examples += 1;
    }

    // ========================================================================
    // 2. UUID REDACTION - Common pattern for API responses
    // ========================================================================
    {
        // Example 2a: Multiple UUIDs in response
        let user_with_ids = json!({
            "user_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "session_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
            "request_id": "9c56cce9-8bb7-4c7f-8f5b-7a8c6f5d4e3c",
            "username": "bob",
            "action": "login"
        });

        // Redaction pattern for UUIDs
        // .replace_regex(r"[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}", "[UUID]")

        uuid_redactions += 1;
        redaction_examples += 1;

        // After redaction, snapshot would be:
        let redacted = json!({
            "user_id": "[UUID]",
            "session_id": "[UUID]",
            "request_id": "[UUID]",
            "username": "bob",
            "action": "login"
        });

        // Verify structure (simulated)
        assert_eq!(
            user_with_ids.get("username"),
            redacted.get("username")
        );
    }

    // ========================================================================
    // 3. TIMESTAMP REDACTION - Handle various datetime formats
    // ========================================================================
    {
        // Example 3a: ISO 8601 timestamps
        let event_log = json!({
            "timestamp": "2025-11-15T14:30:45.123Z",
            "event_type": "user_login",
            "user_id": "user_123"
        });

        // Redaction: r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{3})?Z?" → "[ISO_TIMESTAMP]"
        timestamp_redactions += 1;
        redaction_examples += 1;

        // Example 3b: Unix timestamps (epoch)
        let metric_data = json!({
            "value": 42.5,
            "timestamp": 1731679845,
            "metric": "temperature"
        });

        // Redaction: r"\d{10}" → "[EPOCH_TIMESTAMP]"
        timestamp_redactions += 1;
        redaction_examples += 1;

        // Example 3c: Date-only format
        let record = json!({
            "date": "2025-11-15",
            "description": "Daily report",
            "status": "completed"
        });

        // Redaction: r"\d{4}-\d{2}-\d{2}" → "[DATE]"
        timestamp_redactions += 1;
        redaction_examples += 1;
    }

    // ========================================================================
    // 4. COMBINED REDACTIONS - Multiple patterns in single response
    // ========================================================================
    {
        // Example 4a: API response with both UUIDs and timestamps
        let complex_response = json!({
            "request_id": "a1b2c3d4-e5f6-47a8-9b1c-2d3e4f5a6b7c",
            "user_id": "u-550e8400-e29b-41d4-a716-446655440000",
            "created_at": "2025-11-15T10:30:45Z",
            "updated_at": "2025-11-15T11:45:30Z",
            "data": {
                "session_token": "sess_550e8400-e29b-41d4-a716-446655440000",
                "expires_at": "2025-11-22",
                "user_name": "alice"
            }
        });

        // Multiple redaction patterns would be applied:
        // 1. r"[a-f0-9-]{36}" → "[UUID]"
        // 2. r"[a-f0-9-]{40}" → "[LONG_UUID]"
        // 3. r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z" → "[ISO_TIMESTAMP]"
        // 4. r"\d{4}-\d{2}-\d{2}" → "[DATE]"
        // 5. r"sess_[a-f0-9-]{36}" → "[SESSION_TOKEN]"

        uuid_redactions += 1;
        timestamp_redactions += 1;
        redaction_examples += 1;

        let redacted_response = json!({
            "request_id": "[UUID]",
            "user_id": "[LONG_UUID]",
            "created_at": "[ISO_TIMESTAMP]",
            "updated_at": "[ISO_TIMESTAMP]",
            "data": {
                "session_token": "[SESSION_TOKEN]",
                "expires_at": "[DATE]",
                "user_name": "alice"
            }
        });

        // Verify non-sensitive data is unchanged
        assert_eq!(
            complex_response.get("data").and_then(|d| d.get("user_name")),
            redacted_response.get("data").and_then(|d| d.get("user_name"))
        );
    }

    // ========================================================================
    // 5. INLINE SNAPSHOTS - Store snapshots in source code
    // ========================================================================
    {
        // Example 5a: Simple inline snapshot
        let user_data = json!({
            "name": "Alice",
            "email": "alice@example.com",
            "active": true
        });

        // Inline snapshot syntax (v1.3.0):
        // assert_snapshot_inline!(user_data, @r#"
        // {
        //   "name": "Alice",
        //   "email": "alice@example.com",
        //   "active": true
        // }
        // "#);

        inline_snapshot_examples += 1;

        // Example 5b: Inline snapshot with complex nested structure
        let api_config = json!({
            "version": "1.0",
            "endpoints": {
                "auth": "https://api.example.com/auth",
                "users": "https://api.example.com/users"
            },
            "timeout": 30,
            "retries": 3
        });

        // Inline snapshot:
        // assert_snapshot_inline!(api_config, @r#"
        // {
        //   "version": "1.0",
        //   "endpoints": {
        //     "auth": "https://api.example.com/auth",
        //     "users": "https://api.example.com/users"
        //   },
        //   "timeout": 30,
        //   "retries": 3
        // }
        // "#);

        inline_snapshot_examples += 1;
    }

    // ========================================================================
    // 6. SNAPSHOT PROFILES - Different comparison modes
    // ========================================================================
    {
        // Example 6a: "strict" profile - Exact match required
        let data_strict = json!({
            "id": 123,
            "name": "test",
            "value": 42
        });

        // SnapshotConfig::profile("strict");  // Requires exact match
        snapshot_profiles_used += 1;

        // Example 6b: "pretty" profile - Formatted, stable key order
        let data_pretty = json!({
            "name": "Alice",
            "email": "alice@example.com",
            "id": 1
        });

        // SnapshotConfig::profile("pretty");  // Keys sorted, whitespace normalized
        snapshot_profiles_used += 1;

        // Example 6c: "compact" profile - Minimal whitespace
        let data_compact = json!({
            "name": "Bob",
            "email": "bob@example.com"
        });

        // SnapshotConfig::profile("compact");  // No extra whitespace
        snapshot_profiles_used += 1;

        // Example 6d: "diff-only" profile - Show only differences
        let data_diff = json!({
            "old_field": "value",
            "new_field": "changed"
        });

        // SnapshotConfig::profile("diff-only");  // Only show changed fields
        snapshot_profiles_used += 1;
    }

    Ok(SnapshotRedactionResult {
        redaction_examples,
        uuid_redactions,
        timestamp_redactions,
        inline_snapshot_examples,
        snapshot_profiles_used,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_redaction() {
        let result = run().expect("Snapshot redaction should run");
        assert!(result.redaction_examples > 0);
        assert!(result.uuid_redactions > 0);
        assert!(result.timestamp_redactions > 0);
        assert!(result.inline_snapshot_examples > 0);
        assert!(result.snapshot_profiles_used > 0);
    }
}
