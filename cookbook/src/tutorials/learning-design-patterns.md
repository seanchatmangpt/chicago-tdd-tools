# Learning Design Patterns: 120-Minute Mastery

> 🎓 **TUTORIAL** | Master the 10 advanced design patterns

This tutorial guides you through the 10 design patterns that create safe, fast, maintainable systems.

**Time**: ~120 minutes | **Difficulty**: Advanced | **Prerequisites**: [Testing](learning-testing-patterns.md) + [Architecture](learning-architecture-patterns.md)

---

## Module Overview

| Pattern | Time | Focus |
|---------|------|-------|
| Pattern 11: Zero-Cost Abstractions | 12 min | Performance without sacrifice |
| Pattern 12: Type Safety with GATs | 12 min | Lifetimes and type safety |
| Pattern 13: Sealed Traits | 10 min | Prevent API misuse |
| Pattern 14: Compile-Time Validation | 12 min | Move errors left |
| Pattern 15: Type State Enforcement | 12 min | Encode state in types |
| Pattern 16: Fixture Lifecycle | 12 min | Safe resource management |
| Pattern 17: Builder-Driven Test Data | 12 min | Fluent test builders |
| Pattern 18: Timeout Defense | 12 min | Prevent hangs |
| Pattern 19: Feature Gate Slices | 12 min | Reliable feature flags |
| Pattern 20: Macro Pattern Enforcement | 12 min | Enforce patterns with code |

---

## Part 1: Zero-Cost Abstractions (12 minutes)

**Goal**: Performance through generics and compile-time specialization

### The Problem: Abstraction vs. Performance

```rust
// ❌ Abstraction adds runtime cost
fn process_data(data: &dyn Iterator<Item = i32>) {
    // Dynamic dispatch = function pointers = overhead
    for item in data {
        // ...
    }
}

// Result: Slower than necessary
```

### The Solution: Generics = Zero Cost

```rust
// ✅ Generic = compile specialization
fn process_data<I: Iterator<Item = i32>>(data: I) {
    // Compiler generates specialized code for EACH type
    // No function pointers, no indirection
    for item in data {
        // ...
    }
}

// Rust compiles:
// - process_data::<Vec<i32>::IntoIter>
// - process_data::<ArrayIter>
// - process_data::<CustomIterator>
// Each one optimized for its type!
```

### Key Principle

Rust compiler can **monomorphize** generics - it creates specialized versions for each type. This costs compilation time but **zero runtime cost**.

### When to Use Generics vs. Trait Objects

| Choice | Cost | Use When |
|--------|------|----------|
| **Generics** | Compile-time | Size known, type fixed |
| **Trait objects** | Runtime | Size unknown, type varies |

```rust
// ✅ Generics - no runtime cost
fn sort<T: Ord>(list: &mut [T]) { ... }

// ✅ Trait objects - when you need dynamic dispatch
fn apply_filter(items: &[i32], filter: &dyn Fn(i32) -> bool) { ... }
```

### Checkpoint Question

You have a function that processes different collection types.

**Should you use generics or trait objects?**

Answer: **Generics**. Let compiler specialize for each type.

---

## Part 2: Type Safety with GATs (12 minutes)

**Goal**: Use Generic Associated Types to prevent lifetime bugs

### The Problem: Complex Lifetimes

```rust
// ❌ Lifetime issues with references
trait DataProvider {
    fn get(&self) -> &str;
}

// Can this reference outlive the provider?
// The compiler can't tell!
```

### The Solution: GATs

```rust
// ✅ GATs bind lifetime to self
trait DataProvider {
    type Data<'a> where Self: 'a;

    fn get(&'a self) -> Self::Data<'a>;
}

// Now compiler KNOWS: returned data is tied to self's lifetime
```

### Why This Matters

```rust
// With GATs, this is impossible:
fn use_provider(provider: &Provider) {
    let data = provider.get();
    drop(provider);  // ❌ Compiler error!
    println!("{:?}", data);  // data would be dangling
}

// Without GATs, it might compile (dangerous!)
```

### When to Use GATs

✅ **Use when**: Returning references from traits
✅ **Use when**: Fixture providers with borrowed data
✅ **Use when**: APIs that care about lifetimes

### Checkpoint Question

You have a trait that returns a reference to internal data.

**How do you ensure it can't outlive the source?**

Answer: Use GATs to bind the returned reference lifetime to `self`.

---

## Part 3: Sealed Traits (10 minutes)

**Goal**: Prevent external implementations of your traits

### The Problem: API Misuse

```rust
// ❌ Public trait - anyone can implement!
pub trait Serializable {
    fn serialize(&self) -> String;
}

// External code implements it wrongly:
impl Serializable for String {
    fn serialize(&self) -> String {
        "nope".to_string()  // Wrong!
    }
}

// Your code breaks!
```

### The Solution: Sealed Traits

```rust
// ✅ Private sealing module
mod sealed {
    pub trait Sealed {}
}

pub trait Serializable: sealed::Sealed {
    fn serialize(&self) -> String;
}

// Only WE can implement Sealed
impl sealed::Sealed for MyType {}
impl Serializable for MyType { ... }

// External code CANNOT implement Serializable
// because they can't implement sealed::Sealed!
```

### Why This Matters

- **Prevents misuse** → Forced to use correct implementations
- **API evolution** → Can change internals safely
- **Documentation** → Readers know this is sealed

### Pattern

```rust
mod sealed {
    pub trait Sealed {}
}

pub trait PublicTrait: sealed::Sealed {
    fn public_method(&self);
}

// Only internal types can implement
pub struct InternalType;
impl sealed::Sealed for InternalType {}
impl PublicTrait for InternalType { ... }
```

### Checkpoint Question

You have an important trait that must have specific implementations.

**How do you prevent user code from breaking it?**

Answer: Seal the trait so only your code can implement it.

---

## Part 4: Compile-Time Validation (12 minutes)

**Goal**: Catch errors during compilation, not at runtime

### The Problem: Runtime Errors

```rust
// ❌ Errors at runtime
fn process(config: &str) -> Result<Data, Error> {
    let parsed = parse_config(config)?;  // Might fail at runtime
    validate_config(parsed)?;  // Another runtime check
    Ok(build_data(parsed))
}

// Errors are found after deployment 😞
```

### The Solution: Phantom Types

```rust
// ✅ Errors at compile time
struct Config<S> {
    data: String,
    _state: PhantomData<S>,
}

// Parsing: Raw → Parsed
impl Config<Raw> {
    fn parse(s: &str) -> Result<Config<Parsed>, Error> {
        let data = validate(s)?;
        Ok(Config { data, _state: PhantomData })
    }
}

// Building: Only works with Parsed
impl Config<Parsed> {
    fn build(self) -> Data {
        // Can only call this on Parsed, never Raw
        Data::new(&self.data)
    }
}

// ✅ This is impossible:
let raw = Config::<Raw>::from("config");
raw.build();  // ❌ Compiler error! Use .parse() first
```

### Key Idea

**Encode requirements in types**. Make it impossible to violate at compile-time.

### Common Examples

| Pattern | Types | Purpose |
|---------|-------|---------|
| **Builder pattern** | Raw → Built | Ensure all fields set |
| **State machines** | Idle → Running | Prevent invalid operations |
| **Type tokens** | Foo<_> with phantom | Track type information |

### Checkpoint Question

You have an API that requires:
1. Initialize
2. Configure
3. Start

**How do you prevent wrong order?**

Answer: Use phantom types to track state:
- `Service<Uninitialized>` → `.init()` → `Service<Initialized>`
- `Service<Initialized>` → `.configure()` → `Service<Configured>`
- `Service<Configured>` → `.start()` → `Service<Running>`

---

## Part 5: Type State Enforcement (12 minutes)

**Goal**: Use the type system to enforce valid state transitions

### The Problem: Invalid States

```rust
// ❌ Can access before initialization
struct Connection {
    socket: Option<Socket>,  // None until connected
}

impl Connection {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        match self.socket {
            Some(ref mut s) => s.write(data),
            None => Err("Not connected"),  // Runtime error!
        }
    }
}

// Code can call send() before connect() - runtime error!
```

### The Solution: Type State

```rust
// ✅ Types enforce state
struct Connection<State> { ... }

// States
pub struct Disconnected;
pub struct Connected;

// Only Disconnected can connect
impl Connection<Disconnected> {
    fn connect(mut self, addr: &str) -> Result<Connection<Connected>> {
        self.socket = Some(Socket::new(addr)?);
        Ok(Connection { ... })
    }
}

// Only Connected can send
impl Connection<Connected> {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.socket.write(data)?;  // socket is guaranteed Some
        Ok(())
    }
}

// ✅ Impossible to call send() before connect():
let conn = Connection::new();  // Disconnected
conn.send(b"data")?;  // ❌ Compiler error!

let conn = conn.connect("127.0.0.1")?;  // Now Connected
conn.send(b"data")?;  // ✅ Allowed!
```

### Benefits

- **Compile-time safety** → No panics from invalid states
- **No runtime checks** → No Option/Result overhead
- **Clear API** → Code documents valid transitions
- **Impossible states** → Some states just can't happen

### Checkpoint Question

You have a database transaction that must:
1. Begin
2. Execute
3. Commit/Rollback

**How do you prevent calling Commit before Begin?**

Answer: Use type states:
- `Tx<NotStarted>` → `.begin()` → `Tx<Started>`
- `Tx<Started>` → `.commit()` → `Tx<Finished>`

---

## Part 6-10: Summary of Advanced Patterns

The remaining 5 patterns build on what you've learned:

| # | Pattern | What It Does |
|-|---------|--------------|
| 16 | **Fixture Lifecycle** | Manage resources with sealed traits (combines patterns 13+2) |
| 17 | **Builder Test Data** | Fluent builders for test setup (applies compilation validation) |
| 18 | **Timeout Defense** | Multiple timeout strategies (zero-cost + compile-time) |
| 19 | **Feature Gates** | Type-safe feature flags (type state + sealed traits) |
| 20 | **Macro Enforcement** | Compile-time pattern checks via macros (ultimate compile-time validation) |

---

## Putting It Together: Complete Design

### Exercise: Build a Safe Transaction System

Requirements:
1. Transactions must go: Begin → Execute → Commit
2. Can't add statements after commit
3. Type-safe with zero runtime overhead
4. Can't be misused by users
5. Test data builder for testing

### Solution Using All Patterns

```rust
// Pattern 15 + 14: Type state
pub struct Transaction<State> {
    id: u32,
    statements: Vec<String>,
    _state: PhantomData<State>,
}

pub struct NotStarted;
pub struct Started;
pub struct Committed;

// Pattern 14 + 15: Compile-time validation
impl Transaction<NotStarted> {
    pub fn begin() -> Transaction<Started> {
        Transaction { ... }
    }
}

impl Transaction<Started> {
    pub fn add_statement(&mut self, sql: &str) -> Result<()> {
        self.statements.push(sql.to_string());
        Ok(())
    }

    pub fn commit(self) -> Transaction<Committed> {
        // Execute all statements
        Transaction { ... }
    }
}

// Pattern 13: Sealed to prevent misuse
mod sealed {
    pub trait Sealed {}
}

pub trait TransactionOps: sealed::Sealed {
    fn execute(&self) -> Result<()>;
}

impl sealed::Sealed for Transaction<Started> {}
impl TransactionOps for Transaction<Started> {
    fn execute(&self) -> Result<()> { ... }
}

// Pattern 17: Builder for tests
pub struct TransactionBuilder {
    statements: Vec<String>,
}

impl TransactionBuilder {
    pub fn new() -> Self { ... }
    pub fn add(mut self, sql: &str) -> Self {
        self.statements.push(sql.to_string());
        self
    }
    pub fn build(self) -> Transaction<Started> { ... }
}

// ✅ Usage is type-safe:
let mut tx = Transaction::begin();
tx.add_statement("INSERT ...")?;
let tx = tx.commit();  // Type changes to Committed

// ❌ Impossible:
let tx = Transaction::begin();
let tx = tx.commit();
tx.add_statement("SELECT");  // ❌ Compiler error! tx is Committed
```

---

## Summary: The 10 Design Patterns

| Pattern | Goal | Mechanism |
|---------|------|-----------|
| 11 | Zero-cost | Generics + monomorphization |
| 12 | Lifetimes | GATs (Generic Associated Types) |
| 13 | API safety | Sealed traits |
| 14 | Validation | Phantom types + state |
| 15 | State machines | Type states |
| 16 | Resources | Lifecycle traits |
| 17 | Test data | Fluent builders |
| 18 | Robustness | Timeout strategies |
| 19 | Features | Type-safe gates |
| 20 | Enforcement | Procedural macros |

---

## Next Steps

### Master One Pattern Per Week

1. **Week 1**: Zero-Cost & GATs
2. **Week 2**: Sealed & Compile-Time
3. **Week 3**: Type State & Lifecycle
4. **Week 4**: Builder, Timeout, Features, Macros

### Apply to Your Project

Choose the top 3 patterns that would improve your code safety and refactor this month.

---

## Checkpoint: Do You Know...?

- [ ] How generics create zero-cost abstractions?
- [ ] When to use GATs for lifetime safety?
- [ ] How sealed traits prevent API misuse?
- [ ] How to encode validation in types?
- [ ] How type states prevent invalid operations?
- [ ] How to manage resource lifecycles?
- [ ] How builders simplify test setup?
- [ ] Why timeouts matter?
- [ ] How to implement type-safe features?
- [ ] When to use macros for enforcement?

If yes to most, you've mastered Design Patterns! 🎉

---

**Congratulations!** You now understand the complete pattern language of Chicago TDD Tools.

## Final Advice

These patterns work together:

- **Testing patterns** help you verify code works
- **Architecture patterns** help you organize code
- **Design patterns** help you prevent bugs at compile-time

Master them all, and you'll write code that's:
✅ Tested thoroughly
✅ Well-organized
✅ Type-safe
✅ High-performance
✅ Hard to misuse

That's the Chicago TDD difference.
