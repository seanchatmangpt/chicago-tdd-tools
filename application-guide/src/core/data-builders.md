# Building Test Data with Data Builders

Data builders provide a fluent API for constructing complex test data structures.

## Why Data Builders?

Raw test data is hard to read and maintain:

```rust
// ❌ Hard to understand what this represents
let mut data = HashMap::new();
data.insert("key1".to_string(), "value1".to_string());
data.insert("key2".to_string(), "value2".to_string());
data.insert("order_id".to_string(), "ORD-001".to_string());
```

Data builders are readable and maintainable:

```rust
// ✅ Clear intent - building an order
let data = TestDataBuilder::new()
    .with_var("key1", "value1")
    .with_var("key2", "value2")
    .with_order_data("ORD-001", "100.00")
    .build_json()?;
```

## Basic Data Builder Usage

### Creating Simple Data

```rust
use chicago_tdd_tools::prelude::*;

test!(test_data_builder, {
    // Arrange: Build test data
    let builder = TestDataBuilder::new()
        .with_var("name", "Alice")
        .with_var("email", "alice@example.com");

    // Build as JSON
    let json_data = builder.build_json()?;
    assert!(json_data.is_object());
});
```

### Building JSON Data

The primary format is JSON:

```rust
let data = TestDataBuilder::new()
    .with_var("key", "value")
    .build_json()?;  // Returns serde_json::Value
```

All test data is built as JSON, which is flexible and works with most applications.

## Fluent Builder Pattern

Builders use method chaining for readability:

```rust
let data = TestDataBuilder::new()
    .with_var("user_id", "123")
    .with_var("name", "Bob")
    .with_var("email", "bob@example.com")
    .with_var("status", "active")
    .with_order_data("ORD-001", "250.99")
    .build_json()?;
```

Each method returns `Self`, allowing unlimited chaining.

## Builder Methods

### Basic Variables

```rust
.with_var(key, value)           // Add a string variable
```

### Complex Data

```rust
.with_order_data(id, amount)    // Add order information
```

### Error Handling

Always handle the Result from `build_*()`:

```rust
match TestDataBuilder::new()
    .with_var("key", "value")
    .build_json()
{
    Ok(data) => {
        // Use data
        assert!(data.is_object());
    }
    Err(e) => {
        alert_critical!("Failed to build data: {}", e);
        return Err(e.into());
    }
}
```

Or use `?`:

```rust
let data = TestDataBuilder::new()
    .with_var("key", "value")
    .build_json()?;  // Propagates error
```

## Real-World Example: Building User Data

```rust
test!(test_user_registration, {
    // Build user data
    let user_data = TestDataBuilder::new()
        .with_var("username", "alice_wonder")
        .with_var("email", "alice@example.com")
        .with_var("password", "secure_password_123")
        .with_var("first_name", "Alice")
        .with_var("last_name", "Wonder")
        .with_var("country", "US")
        .build_json()?;

    // Use in test
    let result = register_user(&user_data)?;

    // Verify
    assert_ok!(&result);
    assert_eq!(result.unwrap().email, "alice@example.com");
});
```

## Real-World Example: Building Order Data

```rust
test!(test_order_processing, {
    // Build order
    let order = TestDataBuilder::new()
        .with_order_data("ORD-12345", "499.99")
        .with_var("customer_id", "CUST-001")
        .with_var("shipping_address", "123 Main St")
        .with_var("payment_method", "credit_card")
        .build_json()?;

    // Process order
    let result = process_order(&order)?;

    // Verify
    assert_ok!(&result);
    assert_eq!(result.unwrap().status, "processed");
});
```

## Advanced: Composition

Build complex structures by combining builders:

```rust
test!(test_composition, {
    // Build related data
    let user = TestDataBuilder::new()
        .with_var("user_id", "123")
        .with_var("name", "Alice")
        .build_json()?;

    let order = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_var("user_id", "123")  // Link to user
        .build_json()?;

    // Both built, ready to use
    assert_eq!(user["user_id"], "123");
    assert_eq!(order["user_id"], "123");
});
```

## Boundary Conditions with Builders

Test edge cases:

```rust
test!(test_builder_boundaries, {
    // Empty data
    let empty = TestDataBuilder::new().build_json()?;
    assert!(empty.is_object());

    // Minimal data
    let minimal = TestDataBuilder::new()
        .with_var("id", "1")
        .build_json()?;
    assert_eq!(minimal["id"], "1");

    // Maximum data (many fields)
    let mut builder = TestDataBuilder::new();
    for i in 0..1000 {
        builder = builder.with_var(&format!("field_{}", i), &format!("value_{}", i));
    }
    let large = builder.build_json()?;
    assert!(large.is_object());
});
```

## Accessing Built Data

### Access as JSON

```rust
let data = TestDataBuilder::new()
    .with_var("name", "Alice")
    .with_var("age", "30")
    .build_json()?;

// Access fields
assert_eq!(data["name"], "Alice");
assert_eq!(data["age"], "30");
```

### Serialize to Struct

```rust
#[derive(Deserialize)]
struct User {
    name: String,
    age: u32,
}

let data = TestDataBuilder::new()
    .with_var("name", "Alice")
    .with_var("age", "30")
    .build_json()?;

let user: User = serde_json::from_value(data)?;
assert_eq!(user.name, "Alice");
assert_eq!(user.age, 30);
```

## Best Practices

✅ **Do:**
- Use descriptive variable names
- Chain methods for readability
- Handle errors with `?`
- Build all data before acting
- Use order data for order-specific fields

❌ **Don't:**
- Use unclear abbreviations
- Mix high-level and low-level builders
- Build data after acting (arrange first!)
- Ignore build errors

## Common Patterns

### Pattern: Reusable Builder Factory

```rust
fn create_valid_user_data() -> Result<serde_json::Value, String> {
    TestDataBuilder::new()
        .with_var("username", "test_user")
        .with_var("email", "test@example.com")
        .with_var("status", "active")
        .build_json()
}

test!(test_with_factory, {
    let user_data = create_valid_user_data()?;
    // Use pre-built data
});
```

### Pattern: Variation for Edge Cases

```rust
fn create_inactive_user_data() -> Result<serde_json::Value, String> {
    TestDataBuilder::new()
        .with_var("username", "inactive_user")
        .with_var("email", "inactive@example.com")
        .with_var("status", "inactive")  // Key difference
        .build_json()
}

test!(test_inactive_user, {
    let user_data = create_inactive_user_data()?;
    // Test inactive user handling
});
```

## Troubleshooting

### "Failed to build data: Invalid JSON"

Check for:
- Malformed variable values
- Type mismatches
- Missing required fields

```rust
// Debug by building step-by-step
let builder1 = TestDataBuilder::new().with_var("key1", "value1");
// builder1 is valid

let builder2 = builder1.with_var("key2", "value2");
// builder2 is valid

// etc.
```

## Next Steps

Learn assertions: [Assertions & Verification](assertions.md)

---

## Summary

| Aspect | Purpose |
|--------|---------|
| **Fluent API** | Readable data construction |
| **Chaining** | `.with_var()` returns `Self` |
| **Error Handling** | `build_json()` returns `Result` |
| **Composition** | Combine multiple builders |
| **Reusability** | Extract to helper functions |

