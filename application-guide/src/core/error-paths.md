# Error Path Testing

> 🔧 **HOW-TO** | Learn to test failure scenarios thoroughly

Error paths are where 80% of bugs hide. Chicago TDD emphasizes comprehensive error testing.

## Why Error Paths Matter

Most code focuses on the "happy path" (success). Bugs hide in error cases:

```rust
// Happy path is obvious
let parsed = "42".parse::<u32>()?;
assert_eq!(parsed, 42);

// Error path has subtle bugs
// What about "not_a_number"?
// What about negative "-42"?
// What about overflow "99999999999999999999"?
```

## Testing Both Paths

Every function should test both success and failure:

```rust
test!(test_complete_behavior, {
    // Success path
    let ok_result = parse_number::<u32>("42");
    assert_ok!(&ok_result);
    assert_eq!(ok_result.unwrap(), 42);

    // Error path
    let err_result = parse_number::<u32>("invalid");
    assert_err!(&err_result);
});
```

## Common Error Scenarios

### 1. Invalid Input

```rust
test!(test_invalid_input, {
    let result = validate_email("not_an_email");
    assert_err!(&result);

    let result = validate_email("");
    assert_err!(&result);

    let result = validate_email("@");
    assert_err!(&result);
});
```

### 2. Boundary Conditions

```rust
test!(test_boundaries, {
    // Minimum
    assert_ok!(&process(0));

    // Just above minimum
    assert_ok!(&process(1));

    // Maximum valid
    assert_ok!(&process(u32::MAX - 1));

    // Just past maximum
    assert_err!(&process(u32::MAX + 1));  // If checked
});
```

### 3. Resource Errors

```rust
test!(test_resource_errors, {
    // File doesn't exist
    let result = read_file("nonexistent.txt");
    assert_err!(&result);

    // Permission denied
    let result = write_file("/root/restricted.txt", "data");
    assert_err!(&result);

    // Out of memory (hard to test, but consider it)
});
```

### 4. State Errors

```rust
test!(test_state_errors, {
    let state = MyStateMachine::new();

    // Valid transition
    assert_ok!(&state.transition_to_active());

    // Invalid transition
    let already_active = MyStateMachine::new().transition_to_active();
    assert_err!(&already_active.transition_to_active());
});
```

## Error Messages

Test that error messages are helpful:

```rust
test!(test_error_messages, {
    let result = parse_number::<u32>("not_a_number");

    match result {
        Err(e) => {
            // Verify error message is clear
            assert!(e.to_string().contains("parse error"));
            assert!(e.to_string().contains("not_a_number"));
        }
        Ok(_) => panic!("Should have failed"),
    }
});
```

## Error Recovery

Test that code recovers from errors:

```rust
test!(test_error_recovery, {
    // First attempt fails
    let result1 = connect_to_database("invalid_url");
    assert_err!(&result1);

    // Code continues and retries with valid URL
    let result2 = connect_to_database("valid_url");
    assert_ok!(&result2);
});
```

## Real-World Example: Form Validation

```rust
test!(test_form_validation, {
    let validator = FormValidator::new();

    // Valid case
    let valid = validator.validate(&FormData {
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        password: "secure_password_123".to_string(),
    });
    assert_ok!(&valid);

    // Missing username
    let missing_username = validator.validate(&FormData {
        username: "".to_string(),
        email: "alice@example.com".to_string(),
        password: "secure_password_123".to_string(),
    });
    assert_err!(&missing_username);

    // Invalid email
    let invalid_email = validator.validate(&FormData {
        username: "alice".to_string(),
        email: "not_an_email".to_string(),
        password: "secure_password_123".to_string(),
    });
    assert_err!(&invalid_email);

    // Weak password
    let weak_password = validator.validate(&FormData {
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        password: "123".to_string(),  // Too short
    });
    assert_err!(&weak_password);
});
```

## Real-World Example: API Endpoint

```rust
test!(test_api_error_handling, {
    let client = ApiClient::new();

    // Success
    let result = client.get_user(123);
    assert_ok!(&result);
    assert_eq!(result.unwrap().id, 123);

    // User not found
    let result = client.get_user(999999);
    assert_err!(&result);

    // Invalid ID
    let result = client.get_user(-1);
    assert_err!(&result);

    // Network error (mock or integration test)
    let result = client.get_user(456);  // Server down
    assert_err!(&result);
});
```

## Error Handling Patterns

### Pattern: Check and Handle

```rust
test!(test_check_and_handle, {
    let result = risky_operation();

    match result {
        Ok(value) => {
            assert_eq!(value, expected);
        }
        Err(e) => {
            // Handle error
            assert!(e.to_string().len() > 0);
        }
    }
});
```

### Pattern: Map Error

```rust
test!(test_map_error, {
    let result = risky_operation()
        .map_err(|e| format!("Operation failed: {}", e));

    assert_err!(&result);
    if let Err(e) = result {
        assert!(e.contains("Operation failed"));
    }
});
```

### Pattern: Recover

```rust
test!(test_error_recovery, {
    let result = risky_operation()
        .or_else(|_| fallback_operation());

    // Should succeed via fallback
    assert_ok!(&result);
});
```

## Comprehensive Error Test

```rust
test!(test_comprehensive_errors, {
    // Arrange
    let test_cases = vec![
        ("valid_input", true),
        ("", false),
        ("too_long_" /* 100 chars */, false),
        ("special@chars#", false),
        ("123", true),
        ("-123", false),  // Negative not allowed
    ];

    // Act & Assert
    for (input, should_succeed) in test_cases {
        let result = validate_input(input);

        if should_succeed {
            assert_ok!(&result, "Input '{}' should be valid", input);
        } else {
            assert_err!(&result, "Input '{}' should be invalid", input);
        }
    }
});
```

## Best Practices

✅ **Do:**
- Test both success and error paths
- Use boundary conditions
- Test error messages
- Verify error recovery
- Document expected errors

❌ **Don't:**
- Only test the happy path
- Assume error handling is correct
- Ignore boundary conditions
- Skip error message verification
- Test implementation details of errors

## Error Testing Checklist

For each function, test:

- [ ] Happy path (normal input)
- [ ] Invalid input
- [ ] Boundary conditions (min, max, zero, -1)
- [ ] Empty/null values
- [ ] Resource errors (if applicable)
- [ ] State errors (if applicable)
- [ ] Error messages are clear
- [ ] Error recovery is possible

## Common Error Patterns

| Scenario | Test Case |
|----------|-----------|
| Missing input | Empty string, None, empty Vec |
| Invalid format | Wrong type, malformed data |
| Out of range | Negative when only positive allowed |
| Resource unavailable | File not found, connection refused |
| State violation | Invalid state transition |
| Timeout | Operation takes too long |

## Real-World Integration Example

```rust
test!(test_database_operations, {
    let db = Database::new();

    // Success: Insert and retrieve
    let user_id = db.insert_user("alice", "alice@example.com");
    let result = db.get_user(user_id);
    assert_ok!(&result);

    // Error: User not found
    let result = db.get_user(999999);
    assert_err!(&result);

    // Error: Duplicate email
    let result = db.insert_user("bob", "alice@example.com");  // Email taken
    assert_err!(&result);

    // Error: Invalid email
    let result = db.insert_user("carol", "not_an_email");
    assert_err!(&result);
});
```

## Next Steps

Learn advanced techniques: [Advanced Testing Techniques](../advanced/README.md)

---

## Summary

Chicago TDD prioritizes error testing because:
- Bugs hide in error paths
- Error handling is often incorrect
- Users encounter errors in real use

Always test:
- ✅ Happy path
- ✅ Error cases
- ✅ Boundary conditions
- ✅ Error messages
- ✅ Recovery

