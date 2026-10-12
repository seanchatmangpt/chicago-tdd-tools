# CC-0042 — `#[chicago_test]` ticket-gate fixture

Minimal real ticket file consumed by `tests/chicago_test_macro_tests.rs`.
The gate (`proc_macros/src/chicago_test.rs`) only checks existence of the
workspace-relative path, and `extract_ticket_id` parses `CC-0042` from the
filename stem.
