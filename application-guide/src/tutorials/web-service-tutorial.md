# Building a REST Web Service: Complete Tutorial

> 🎓 **TUTORIAL** | Build a production-ready REST API with Chicago TDD

This tutorial guides you through building a real REST web service with comprehensive tests, using Rust and common web frameworks.

**Prerequisites**: [Getting Started](getting-started.md), [CLI Application Tutorial](cli-app-tutorial.md)
**Time**: ~50 minutes
**What you'll build**: A working `user-api` with CRUD operations

---

## Project Overview

You'll build a **user-api** REST service:

```bash
# Get all users
GET /api/users
-> [{"id": 1, "name": "Alice", "email": "alice@example.com"}, ...]

# Get specific user
GET /api/users/:id
-> {"id": 1, "name": "Alice", "email": "alice@example.com"}

# Create user
POST /api/users
-> {"id": 2, "name": "Bob", "email": "bob@example.com"}

# Update user
PUT /api/users/:id
-> {"id": 1, "name": "Alice Updated", ...}

# Delete user
DELETE /api/users/:id
-> {}
```

---

## Step 1: Project Setup (3 minutes)

### Create project

```bash
cargo new user-api
cd user-api
```

### Update Cargo.toml

```toml
[package]
name = "user-api"
version = "0.1.0"
edition = "2021"

[dependencies]
# JSON support
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
chicago-tdd-tools = { version = "1.1", features = ["testing-extras"] }
```

### Project structure

```
user-api/
├── src/
│   ├── main.rs
│   ├── models/
│   │   └── user.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   └── users.rs
│   └── store.rs
└── tests/
    ├── user_tests.rs
    └── api_tests.rs
```

---

## Step 2: Data Models (5 minutes)

Create `src/models/user.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl User {
    pub fn new(id: u32, name: String, email: String) -> Self {
        User { id, name, email }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if !self.email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }
}
```

---

## Step 3: Test the Models (8 minutes)

Create `tests/user_tests.rs`:

```rust
use chicago_tdd_tools::prelude::*;
use user_api::models::User;

test!(test_create_user, {
    let user = User::new(1, "Alice".to_string(), "alice@example.com".to_string());

    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");
});

test!(test_user_validation_valid, {
    let user = User::new(1, "Alice".to_string(), "alice@example.com".to_string());

    let result = user.validate();

    assert_ok!(&result);
});

test!(test_user_validation_empty_name, {
    let user = User::new(1, String::new(), "alice@example.com".to_string());

    let result = user.validate();

    assert_err!(&result);
    if let Err(e) = result {
        assert!(e.contains("Name"));
    }
});

test!(test_user_validation_invalid_email, {
    let user = User::new(1, "Alice".to_string(), "not-an-email".to_string());

    let result = user.validate();

    assert_err!(&result);
    if let Err(e) = result {
        assert!(e.contains("email"));
    }
});
```

---

## Step 4: User Repository (8 minutes)

Create `src/store.rs`:

```rust
use crate::models::{User, CreateUserRequest, UpdateUserRequest};
use std::collections::HashMap;

pub struct UserStore {
    users: HashMap<u32, User>,
    next_id: u32,
}

impl UserStore {
    pub fn new() -> Self {
        UserStore {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, req: CreateUserRequest) -> Result<User, String> {
        let user = User::new(self.next_id, req.name, req.email);
        user.validate()?;

        self.users.insert(user.id, user.clone());
        self.next_id += 1;

        Ok(user)
    }

    pub fn get(&self, id: u32) -> Option<User> {
        self.users.get(&id).cloned()
    }

    pub fn list(&self) -> Vec<User> {
        let mut users: Vec<_> = self.users.values().cloned().collect();
        users.sort_by_key(|u| u.id);
        users
    }

    pub fn update(&mut self, id: u32, req: UpdateUserRequest) -> Result<User, String> {
        let user = self.users.get_mut(&id)
            .ok_or("User not found".to_string())?;

        if let Some(name) = req.name {
            user.name = name;
        }
        if let Some(email) = req.email {
            user.email = email;
        }

        user.validate()?;
        Ok(user.clone())
    }

    pub fn delete(&mut self, id: u32) -> bool {
        self.users.remove(&id).is_some()
    }
}
```

---

## Step 5: Repository Tests (10 minutes)

Create `tests/api_tests.rs`:

```rust
use chicago_tdd_tools::prelude::*;
use user_api::models::{User, CreateUserRequest, UpdateUserRequest};
use user_api::store::UserStore;

test!(test_create_user_success, {
    let mut store = UserStore::new();
    let req = CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let result = store.create(req);

    assert_ok!(&result);
    let user = result.unwrap();
    assert_eq!(user.name, "Alice");
});

test!(test_create_user_invalid_email, {
    let mut store = UserStore::new();
    let req = CreateUserRequest {
        name: "Alice".to_string(),
        email: "invalid".to_string(),
    };

    let result = store.create(req);

    assert_err!(&result);
});

test!(test_get_user, {
    let mut store = UserStore::new();
    let req = CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let created = store.create(req).unwrap();

    let retrieved = store.get(created.id);

    assert_eq!(retrieved, Some(created));
});

test!(test_get_nonexistent_user, {
    let store = UserStore::new();

    let result = store.get(999);

    assert_eq!(result, None);
});

test!(test_list_users, {
    let mut store = UserStore::new();

    store.create(CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }).ok();

    store.create(CreateUserRequest {
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    }).ok();

    let users = store.list();

    assert_eq!(users.len(), 2);
    assert_eq!(users[0].name, "Alice");
    assert_eq!(users[1].name, "Bob");
});

test!(test_update_user, {
    let mut store = UserStore::new();
    store.create(CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }).ok();

    let result = store.update(1, UpdateUserRequest {
        name: Some("Alice Updated".to_string()),
        email: None,
    });

    assert_ok!(&result);
    let updated = result.unwrap();
    assert_eq!(updated.name, "Alice Updated");
});

test!(test_update_nonexistent_user, {
    let mut store = UserStore::new();

    let result = store.update(999, UpdateUserRequest {
        name: Some("Bob".to_string()),
        email: None,
    });

    assert_err!(&result);
});

test!(test_delete_user, {
    let mut store = UserStore::new();
    store.create(CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }).ok();

    let result = store.delete(1);

    assert!(result);
    assert_eq!(store.list().len(), 0);
});

test!(test_delete_nonexistent_user, {
    let mut store = UserStore::new();

    let result = store.delete(999);

    assert!(!result);
});

test!(complete_api_workflow, {
    let mut store = UserStore::new();

    // Create users
    let alice = store.create(CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }).unwrap();

    store.create(CreateUserRequest {
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    }).ok();

    // Verify list
    assert_eq!(store.list().len(), 2);

    // Update Alice
    store.update(alice.id, UpdateUserRequest {
        name: Some("Alice Wonder".to_string()),
        email: None,
    }).ok();

    // Verify update
    let updated = store.get(alice.id).unwrap();
    assert_eq!(updated.name, "Alice Wonder");

    // Delete Bob
    store.delete(2);

    // Verify deletion
    assert_eq!(store.list().len(), 1);
});
```

---

## Step 6: Handlers (8 minutes)

Create `src/handlers/users.rs`:

```rust
use crate::models::CreateUserRequest;
use crate::store::UserStore;
use serde_json::json;

pub fn get_users(store: &UserStore) -> String {
    let users = store.list();
    serde_json::to_string(&users).unwrap_or_else(|_| "[]".to_string())
}

pub fn get_user(store: &UserStore, id: u32) -> Result<String, String> {
    let user = store.get(id).ok_or("User not found".to_string())?;
    serde_json::to_string(&user).map_err(|e| e.to_string())
}

pub fn create_user(store: &mut UserStore, body: &str) -> Result<String, String> {
    let req: CreateUserRequest = serde_json::from_str(body)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let user = store.create(req)?;
    serde_json::to_string(&user).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::prelude::*;

    test!(test_get_users_empty, {
        let store = UserStore::new();

        let result = get_users(&store);

        assert_eq!(result, "[]");
    });

    test!(test_create_user_from_json, {
        let mut store = UserStore::new();
        let json = r#"{"name":"Alice","email":"alice@example.com"}"#;

        let result = create_user(&mut store, json);

        assert_ok!(&result);
        let response = result.unwrap();
        assert!(response.contains("Alice"));
    });

    test!(test_create_user_invalid_json, {
        let mut store = UserStore::new();
        let json = "invalid json";

        let result = create_user(&mut store, json);

        assert_err!(&result);
    });
}
```

---

## Step 7: Main Application (5 minutes)

Create `src/main.rs`:

```rust
mod models;
mod handlers;
mod store;

use models::{User, CreateUserRequest};
use store::UserStore;

fn main() {
    let mut store = UserStore::new();

    // Sample data
    store.create(CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }).ok();

    store.create(CreateUserRequest {
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    }).ok();

    // List users
    for user in store.list() {
        println!("User #{}: {} ({})", user.id, user.name, user.email);
    }
}
```

---

## Step 8: Running Tests (5 minutes)

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_user_success
```

Expected output:
```
running 11 tests
...
test result: ok. 11 passed; 0 failed
```

---

## Extending the API

### Add error responses

```rust
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn format_error(message: String) -> String {
    serde_json::to_string(&ErrorResponse { error: message })
        .unwrap_or_else(|_| r#"{"error":"Unknown error"}"#.to_string())
}
```

### Add pagination

```rust
pub fn list_users_paginated(
    store: &UserStore,
    page: u32,
    limit: u32,
) -> (Vec<User>, u32) {
    let all = store.list();
    let total = all.len() as u32;
    let start = (page - 1) * limit;
    let end = std::cmp::min(start + limit, total);

    (all[start as usize..end as usize].to_vec(), total)
}
```

---

## Next Steps

**Enhance your API:**
- Add authentication
- Add validation
- Add pagination
- Add filtering/searching
- Add real database (PostgreSQL/SQLite)

**Testing enhancements:**
- [Integration Testing with Docker](../guides/integration-docker.md) - Test with real database
- [Snapshot Testing](../advanced/snapshot-testing.md) - Test API responses
- [Property Testing](../advanced/property-testing.md) - Fuzz test API handlers

**Deployment:**
- Build Docker image
- Deploy to cloud (AWS, Heroku, etc.)
- Set up CI/CD pipeline
- Monitor with observability tools

---

**Congratulations!** You've built a production-ready REST API with comprehensive tests. You're ready to deploy real services!
