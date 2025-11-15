# Copying Examples to Your Project Tutorial

**Estimated time**: 10-15 minutes

Learn how to take examples from the playground and adapt them for your own projects.

## Overview

The playground examples are designed to be:
- **Copyable** - Easy to copy and adapt
- **Self-contained** - Include everything needed
- **Well-documented** - Clear comments explaining intent
- **Production-ready** - Follow best practices

This tutorial shows how to extract and adapt playground examples for real projects.

## Prerequisites

- Complete [Getting Started Tutorial](getting-started.md)
- Completed [Running Core Examples Tutorial](running-core-examples.md)
- Your own Rust project with tests
- Understanding of the AAA pattern (Arrange-Act-Assert)

## Step 1: Identify an Example to Copy

Browse available examples:

```bash
cd playground
cargo run -- core list
```

Pick one that solves your problem. For example:
- `fixtures` - If you need test setup/teardown
- `builders` - If you need fluent test data construction
- `assertions` - If you need specialized assertion helpers

## Step 2: Find the Source Code

Playground examples are in `src/`:

```bash
# Find the core features examples
ls playground/src/core/

# Common files:
# - fixtures.rs (fixture examples)
# - builders.rs (builder examples)
# - assertions.rs (assertion examples)
# - macros.rs (test macro examples)
```

## Step 3: Review the Example

Read the source file to understand:

```bash
cat playground/src/core/fixtures.rs | head -50
```

Key things to note:
- What does it import?
- How does it arrange test data?
- What patterns does it follow?
- What macros does it use?

## Step 4: Copy the Example to Your Project

### Option A: Direct Copy (Simplest)

For a quick fixture example:

1. Create a test file in your project:
```bash
mkdir -p tests
touch tests/my_first_test.rs
```

2. Copy the example structure:
```rust
use chicago_tdd_tools::prelude::*;

#[test]
fn test_with_fixture() {
    // Copy the fixture test from playground
    fixture_test!(my_test, fixture, {
        // Arrange: Use fixture
        let counter = fixture.test_counter();

        // Act
        let result = counter + 1;

        // Assert
        assert!(result > counter);
    });
}
```

### Option B: Adapt to Your Code (Recommended)

If you're adapting a builder example to your types:

**Original playground example:**
```rust
let user = UserBuilder::new()
    .with_name("Alice")
    .with_email("alice@example.com")
    .build();
```

**Adapted to your project:**
```rust
// Create a test helper that builds YOUR types
struct TestUserBuilder {
    name: String,
    email: String,
}

impl TestUserBuilder {
    fn new() -> Self {
        Self {
            name: "default".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    fn build(self) -> MyUser {
        MyUser {
            name: self.name,
            email: self.email,
        }
    }
}

#[test]
fn test_with_adapted_builder() {
    let user = TestUserBuilder::new()
        .with_name("Bob")
        .build();

    assert_eq!(user.name, "Bob");
}
```

## Step 5: Add chicago-tdd-tools Dependency

Add the framework to your `Cargo.toml`:

```bash
cd your-project
cargo add chicago-tdd-tools --dev
```

Or manually add to `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = "1.1"
```

For specific features:

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.1", features = ["testing-extras"] }
```

## Step 6: Run Your Test

```bash
cargo test
```

## Step 7: Adapt to Your Needs

Common adaptations:

### Fixture for Your Types

```rust
fixture_test!(test_database, fixture, {
    // Use a fixture specifically for your app
    let db = fixture.create_database();

    // Your code
    db.insert(my_record);

    // Assert
    assert!(db.exists(my_record.id));
});
```

### Builder for Your Structs

```rust
// Adapt the builder pattern to your domain
struct ConfigBuilder {
    host: String,
    port: u16,
}

impl ConfigBuilder {
    fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
        }
    }

    fn with_host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    fn build(self) -> Config {
        Config {
            host: self.host,
            port: self.port,
        }
    }
}

#[test]
fn test_custom_config() {
    let config = ConfigBuilder::new()
        .with_host("example.com")
        .build();

    assert_eq!(config.host, "example.com");
}
```

### Assertion Helpers for Your Errors

```rust
// Adapt assertion patterns
test!(test_error_handling, {
    let result: Result<i32, String> = Err("Database failed".to_string());

    assert_err!(result);
    assert_eq!(result.err(), Some("Database failed".to_string()));
});
```

## Complete Example: Fixture + Builder + Assertions

Here's a real-world adaptation:

```rust
use chicago_tdd_tools::prelude::*;

// Your domain types
struct User {
    id: u64,
    name: String,
    email: String,
}

// Builder for tests
struct UserBuilder {
    id: u64,
    name: String,
    email: String,
}

impl UserBuilder {
    fn new() -> Self {
        Self {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    fn build(self) -> User {
        User {
            id: self.id,
            name: self.name,
            email: self.email,
        }
    }
}

// Fixture for setup/teardown
struct TestDatabase {
    users: Vec<User>,
}

impl TestDatabase {
    fn new() -> Self {
        Self {
            users: Vec::new(),
        }
    }

    fn add_user(&mut self, user: User) {
        self.users.push(user);
    }

    fn find_user(&self, id: u64) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
    }
}

// Test using all patterns
test!(test_user_creation_and_lookup, {
    // Arrange: Use builder and fixture
    let mut db = TestDatabase::new();
    let user = UserBuilder::new()
        .with_name("Alice")
        .build();

    db.add_user(user);

    // Act
    let found = db.find_user(1);

    // Assert with helpers
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Alice");
});
```

## Copying Pattern: Mutation Testing Example

Here's how to adapt a mutation testing example:

**From playground:**
```rust
test!(test_mutation_detection, {
    let mut data = HashMap::new();
    data.insert("key1", "value1");

    // Mutation: What if remove was used instead of insert?
    let mutated = data.remove("key1");

    // Test MUST catch this
    assert_eq!(mutated, Some("value1"));
});
```

**Adapted for your code:**
```rust
test!(test_my_cache_mutation, {
    let mut cache = MyCache::new();
    cache.insert("user_1", user_data);

    // Mutation: What if delete was called instead of insert?
    let result = cache.get("user_1");

    // Test catches if data wasn't stored
    assert!(result.is_some());
});
```

## Copying Pattern: Snapshot Testing

**From playground:**
```rust
test!(test_json_snapshot, {
    let data = MyData { name: "test" };
    assert_snapshot!(serde_json::to_string_pretty(&data)?);
});
```

**Adapted for your API responses:**
```rust
test!(test_api_response_format, {
    let response = api_call();
    let json = serde_json::to_string_pretty(&response)?;

    assert_snapshot!(json);
    // Update snapshots with: cargo make snapshot-accept
});
```

## Best Practices When Copying

1. **Keep the AAA pattern** - Maintain Arrange-Act-Assert structure
2. **Add comments** - Explain why, not what
3. **Use descriptive names** - Make test intent clear
4. **Avoid magic numbers** - Use named constants
5. **Keep tests focused** - One behavior per test
6. **Don't over-copy** - Adapt, don't just copy-paste

## Example: Wrong Way vs. Right Way

**❌ Wrong (just copied, no adaptation):**
```rust
test!(test_something, {
    let counter = fixture.test_counter();
    let incremented = counter + 1;
    assert_eq!(incremented, counter + 1);  // Obvious assertion
});
```

**✅ Right (adapted with clear intent):**
```rust
test!(test_request_counter_increments_on_success, {
    // Arrange: Create fresh request tracker
    let tracker = RequestTracker::new();
    let initial_count = tracker.total_requests();

    // Act: Make a successful request
    let _result = tracker.handle_request(Request::new());

    // Assert: Counter incremented by exactly 1
    assert_eq!(tracker.total_requests(), initial_count + 1);
});
```

## Troubleshooting

### "cannot find macro 'test!'"
Add to your test file:
```rust
use chicago_tdd_tools::prelude::*;
```

### "feature 'X' is required"
Add to your `Cargo.toml`:
```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.1", features = ["feature-name"] }
```

### "types don't match"
Adapt the example to your types:
```rust
// Wrong: Using playground's User type
let user = PlaygroundUser::new();

// Right: Using your User type
let user = YourUser::new();
```

## Summary Checklist

When copying an example:

- ✅ Find the example in `playground/src/`
- ✅ Understand what it demonstrates
- ✅ Copy the pattern, adapt the types
- ✅ Add `chicago-tdd-tools` to `dev-dependencies`
- ✅ Update imports to use `prelude::*`
- ✅ Adapt to your types and business logic
- ✅ Run `cargo test` to verify
- ✅ Keep the AAA pattern intact

## Next Steps

- **See all available examples** → [Example Inventory](../reference/example-inventory.md)
- **Learn more patterns** → [Feature Organization](../explanation/feature-organization.md)
- **Understand philosophy** → [Testing Philosophy](../explanation/testing-philosophy.md)
- **Explore how-to guides** → [How-To Guides](../how-to/)

---

**Ready?** Pick an example and start copying!
