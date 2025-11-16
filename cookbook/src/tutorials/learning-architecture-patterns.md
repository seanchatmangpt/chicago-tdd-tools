# Learning Architecture Patterns: 60-Minute Mastery

> 🎓 **TUTORIAL** | Master the 5 fundamental architecture patterns

This tutorial guides you through the 5 architecture patterns that organize your codebase.

**Time**: ~60 minutes | **Difficulty**: Intermediate | **Prerequisites**: [Testing Patterns](learning-testing-patterns.md)

---

## Module Overview

| Pattern | Time | Focus |
|---------|------|-------|
| Pattern 6: Generic Base Layer | 15 min | Eliminating duplication with abstractions |
| Pattern 7: Extension Layer | 12 min | Safe extensibility without modification |
| Pattern 8: Composition Over Duplication | 12 min | DRY principle in action |
| Pattern 9: Single Source of Truth | 12 min | Keeping data consistent |
| Pattern 10: Capability Grouping | 9 min | Organizing large modules |

---

## Part 1: Generic Base Layer (15 minutes)

**Goal**: Eliminate code duplication through generic abstractions

### The Problem: Code Duplication

Imagine you have multiple types that behave similarly:

```rust
// ❌ Duplicate code everywhere
struct FileStorage {
    path: String,
}

impl FileStorage {
    fn get(&self, key: &str) -> Result<String> { ... }
    fn set(&self, key: &str, value: String) -> Result<()> { ... }
    fn delete(&self, key: &str) -> Result<()> { ... }
}

struct DatabaseStorage {
    connection: Connection,
}

impl DatabaseStorage {
    fn get(&self, key: &str) -> Result<String> { ... }
    fn set(&self, key: &str, value: String) -> Result<()> { ... }
    fn delete(&self, key: &str) -> Result<()> { ... }
}

// Duplicated 9 times!
```

### The Solution: Generic Abstraction

```rust
// ✅ Define generic behavior once
pub trait KeyValueStore {
    fn get(&self, key: &str) -> Result<String>;
    fn set(&self, key: &str, value: String) -> Result<()>;
    fn delete(&self, key: &str) -> Result<()>;
}

// Both implement the same trait
struct FileStorage { ... }
impl KeyValueStore for FileStorage { ... }

struct DatabaseStorage { ... }
impl KeyValueStore for DatabaseStorage { ... }

// Code that works with BOTH
fn backup_all_data(store: &dyn KeyValueStore) {
    // Works with file, database, or any implementation!
}
```

### When to Use This Pattern

✅ **Use when**: You have similar code in multiple places
✅ **Use when**: You want to swap implementations
✅ **Use when**: You want to test with fakes/stubs

### Checkpoint Question

You have `RedisCache` and `MemoryCache` with nearly identical code.

**What should you do?**

Answer: Extract a `Cache` trait and implement it for both.

---

## Part 2: Extension Layer (12 minutes)

**Goal**: Allow safe extensions without modifying existing code

### The Problem: Modifying Core Code

```rust
// Original code
pub struct HttpServer {
    fn handle_request(&self, req: Request) {
        // Handle request
    }
}

// ❌ To add logging, you modify the core:
impl HttpServer {
    fn handle_request(&self, req: Request) {
        println!("Request: {:?}", req);  // Logging added
        // Handle request
    }
}

// ❌ To add authentication, you modify again:
impl HttpServer {
    fn handle_request(&self, req: Request) {
        println!("Request: {:?}", req);  // Still here
        // Check auth
        // Handle request
    }
}

// ❌ Code gets messy fast!
```

### The Solution: Extension Layer

```rust
// Core code - never changes
pub struct HttpServer { ... }

// Extension layer - add features here
pub struct LoggingHttpServer {
    inner: HttpServer,
}

impl LoggingHttpServer {
    fn handle_request(&self, req: Request) {
        println!("Request: {:?}", req);
        self.inner.handle_request(req);
    }
}

// Another extension layer
pub struct AuthHttpServer {
    inner: LoggingHttpServer,
}

impl AuthHttpServer {
    fn handle_request(&self, req: Request) {
        if !req.is_authenticated() {
            return Err(Unauthorized);
        }
        self.inner.handle_request(req);
    }
}

// Usage: Stack them!
let server = HttpServer::new();
let logged = LoggingHttpServer::new(server);
let secured = AuthHttpServer::new(logged);
```

### Why This Matters

- **Original code never changes** → No bugs introduced
- **Easy to test** → Test each layer separately
- **Composable** → Mix and match features
- **Reversible** → Remove a layer anytime

### Checkpoint Question

You need to add timeout handling to a database connection.

**Using the Extension Layer pattern, how would you do it?**

Answer: Create a `TimeoutConnection` wrapper that wraps the original `Connection`.

---

## Part 3: Composition Over Duplication (12 minutes)

**Goal**: Use composition instead of copying code

### The Problem: Copy-Paste Duplication

```rust
// ❌ Copy-pasting code
struct Logger {
    buffer: Vec<String>,
}

impl Logger {
    fn log(&mut self, msg: &str) {
        self.buffer.push(msg.to_string());
    }
}

struct FileWriter {
    buffer: Vec<String>,  // DUPLICATE: Same buffer
    path: String,
}

impl FileWriter {
    fn write(&mut self, data: &str) {
        self.buffer.push(data.to_string());  // DUPLICATE: Same logic
    }
}

// Now you need to fix a bug in the buffer logic...
// You have to fix it in TWO places! 😞
```

### The Solution: Composition

```rust
// ✅ Create a shared component
pub struct StringBuffer {
    data: Vec<String>,
}

impl StringBuffer {
    fn append(&mut self, s: &str) {
        self.data.push(s.to_string());
    }
}

// Compose it into both
struct Logger {
    buffer: StringBuffer,
}

struct FileWriter {
    buffer: StringBuffer,  // Same component
    path: String,
}

// Now: Fix bug in StringBuffer once, both are fixed! ✅
```

### Composition vs. Inheritance

| Aspect | Composition | Inheritance |
|--------|-------------|-------------|
| **Reusability** | ✅ Mix and match | ❌ Rigid hierarchy |
| **Maintainability** | ✅ Changes in one place | ❌ Changes everywhere |
| **Flexibility** | ✅ Swap parts easily | ❌ Can't swap |
| **Testability** | ✅ Test component alone | ❌ Test whole tree |

**Rule of thumb**: If you're copy-pasting code, use composition instead.

### Checkpoint Question

You have `JsonParser` and `XmlParser` with 50 lines of duplicate validation code.

**What should you do?**

Answer: Extract a `Validator` component and use it in both.

---

## Part 4: Single Source of Truth (12 minutes)

**Goal**: Keep data consistent by having one canonical source

### The Problem: Data Inconsistency

```rust
// ❌ Multiple copies of the same data
struct UserCache {
    users: HashMap<u32, User>,  // Copy of database
}

struct UserService {
    cache: UserCache,
    database: Database,  // Original
}

// Update database
service.database.update_user(123, new_data)?;

// Oops! Cache is now stale
// service.cache.users[&123] is out of date!

// Different code paths get different data 😞
```

### The Solution: One Source of Truth

```rust
// ✅ Truth in one place
struct UserService {
    database: Database,  // Only source of truth
}

// Cache is derived from database
fn get_user(&self, id: u32) -> Result<User> {
    // Get from database (single source)
    self.database.fetch(id)
}

// Update
fn update_user(&mut self, id: u32, data: UserUpdate) -> Result<()> {
    // Update only the source
    self.database.update(id, data)
}

// Always consistent! ✅
```

### When to Apply

✅ **Apply when**: Data appears in multiple places
✅ **Apply when**: Synchronization is complex
✅ **Apply when**: Consistency matters (payment systems, authorization, etc.)

### Examples

| Domain | Truth | Derived |
|--------|-------|---------|
| **E-commerce** | Database (orders) | Cache, indices, reports |
| **Auth** | Database (permissions) | Session tokens, caches |
| **Analytics** | Raw events | Aggregations, reports |

### Checkpoint Question

You have user data in both database and cache.

**Which is the single source of truth?**

Answer: Database. Cache is derived/cached from it. Update database first, invalidate cache.

---

## Part 5: Capability Grouping (9 minutes)

**Goal**: Organize large modules by capability, not by type

### The Problem: Type-Based Organization

```
project/
├── models/          # All data types
│   ├── user.rs
│   ├── order.rs
│   └── payment.rs
├── handlers/        # All HTTP handlers
│   ├── user_handler.rs
│   ├── order_handler.rs
│   └── payment_handler.rs
├── persistence/     # All database code
│   ├── user_repo.rs
│   ├── order_repo.rs
│   └── payment_repo.rs

// ❌ To understand "user" feature, you jump between 4 files!
// ❌ Each file is small but scattered
// ❌ Hard to find related code
```

### The Solution: Capability-Based Organization

```
project/
├── users/               # Everything for users
│   ├── model.rs
│   ├── handler.rs
│   ├── repository.rs
│   └── tests.rs
├── orders/              # Everything for orders
│   ├── model.rs
│   ├── handler.rs
│   ├── repository.rs
│   └── tests.rs
├── payments/            # Everything for payments
│   ├── model.rs
│   ├── handler.rs
│   ├── repository.rs
│   └── tests.rs

// ✅ To understand "users", all code is in `users/`!
// ✅ Related code is together
// ✅ Easy to find dependencies
```

### Benefits

- **Cohesion**: Related code together
- **Discoverability**: Find code easily
- **Modularity**: Move features together
- **Testability**: Test capability in one place

### Checkpoint Question

Your project has `models/`, `handlers/`, and `db/` directories.

**How would you reorganize for better capability grouping?**

Answer: Create `users/`, `orders/`, `products/` directories, each containing models, handlers, and db code.

---

## Putting It Together: Complete Exercise (15 minutes)

### Exercise: Building an Order Processing System

You're building an order system with these requirements:

1. Accept orders
2. Store them in database
3. Can switch between PostgreSQL and SQLite
4. Add logging without modifying core code
5. Keep inventory count accurate
6. Organize code clearly

### Design Using All 5 Patterns

**Pattern 6 (Generic Base)**: Create `Storage` trait
```rust
trait OrderStorage {
    fn save(&mut self, order: &Order) -> Result<()>;
    fn get(&self, id: u32) -> Result<Order>;
}

// Both implementations
struct PostgresStorage { ... }
struct SqliteStorage { ... }
```

**Pattern 7 (Extension Layer)**: Add logging
```rust
struct LoggingStorage {
    inner: Box<dyn OrderStorage>,
}
```

**Pattern 8 (Composition)**: Shared validation
```rust
struct OrderValidator { ... }
struct Order Service {
    storage: Box<dyn OrderStorage>,
    validator: OrderValidator,  // Composed
}
```

**Pattern 9 (Single Source)**: Inventory truth
```rust
struct InventoryService {
    database: Database,  // Single source
    // NOT: inventory_cache
}
```

**Pattern 10 (Capability)**: Organize by feature
```
order_service/
├── model.rs
├── handler.rs
├── storage.rs
├── validator.rs
└── tests.rs
```

---

## Summary: The 5 Architecture Patterns

| Pattern | Problem | Solution |
|---------|---------|----------|
| **6: Generic Base** | Duplication | Traits and abstraction |
| **7: Extension Layer** | Modification | Wrapping instead of changing |
| **8: Composition** | Copy-paste | Share components |
| **9: Single Source** | Inconsistency | One source of truth |
| **10: Capability** | Disorganization | Group by feature |

---

## Next Steps

### Apply These Patterns

Choose one pattern and apply it to your current project this week.

### Learn Design Patterns

Once you master architecture, learn [Design Patterns](learning-design-patterns.md) for type safety and optimization.

---

## Checkpoint: Do You Know...?

- [ ] How to use traits to eliminate duplication?
- [ ] How to extend code without modifying it?
- [ ] When to use composition over inheritance?
- [ ] How to keep data consistent?
- [ ] How to organize code by capability?

If yes to all, you've mastered Architecture Patterns! 🎉

---

**Congratulations!** You now understand how to organize production code. Next, master type safety and design with Design Patterns.
