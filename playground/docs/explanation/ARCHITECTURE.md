# Explanation: Playground Architecture & Design

**Type:** Understanding-oriented | **Updated:** 2025-11-15

This document explains the design philosophy, architecture, and decision-making behind the playground CLI.

---

## What is the Playground?

The **Playground** is a demonstration application—a reference implementation that showcases all capabilities of the Chicago TDD testing framework.

### Three Purposes

1. **Educational** - Learn how to use Chicago TDD patterns in a real application
2. **Validation** - Verify that framework components work as designed
3. **Documentation** - Demonstrate best practices through code examples

### Why a Playground?

Rather than just writing documentation, we built a working application that:
- ✅ **Actually uses** all framework features
- ✅ **Demonstrates** patterns in context
- ✅ **Validates** that patterns work together
- ✅ **Serves as** canonical example code

---

## Architectural Philosophy

### 1. Noun-Verb Command Structure

The playground uses a **noun-verb** pattern (provided by `clap-noun-verb` crate):

```
playground [NOUN] [VERB]
    │        │      │
    │        │      └─ Action: stat, list, exec
    │        └────────── Domain: core, testing, validation, etc.
    └─────────────────── Application
```

**Why this design?**

**Traditional approach:**
```bash
playground core-stat
playground core-list
playground testing-stat
playground testing-list
```

**Noun-verb approach (ours):**
```bash
playground core stat
playground core list
playground testing stat
playground testing list
```

**Benefits:**
- ✅ **Discoverability** - Learn nouns first, then verbs
- ✅ **Consistency** - Same verbs work across all nouns
- ✅ **Scalability** - Add new nouns/verbs without explosion
- ✅ **Natural language** - Reads like English

### 2. Module Organization (The Five Nouns)

The playground mirrors the framework structure:

```
Chicago TDD Testing Framework
    ├── Core (fixtures, builders, assertions)
    ├── Testing (property, mutation, snapshot, concurrency)
    ├── Validation (coverage, guards, JTBD, performance)
    ├── Observability (OTEL, Weaver, telemetry)
    └── Integration (testcontainers, Docker)
```

Each module has:
- ✅ A domain area of the framework
- ✅ A CLI noun for exploration
- ✅ Example implementations
- ✅ Validation tests

**Why mirror the framework structure?**

Users can:
1. Explore in playground
2. Reference the matching crate docs
3. Apply in their own projects
4. Maintain consistent mental model

### 3. Three Verbs for Exploration

Each noun supports three verbs:

| Verb | Purpose | Use Case |
|------|---------|----------|
| `stat` | Show status & statistics | "What's available?" |
| `list` | List examples & details | "What can I learn?" |
| `exec` | Run demonstrations | "Show me how it works" |

**Design principle:** Progressive disclosure
- `stat` - Quick overview
- `list` - Detailed listing
- `exec` - Real demonstration

---

## Multi-Format Output Design

### The Problem

Playground needed to serve different audiences:

| Audience | Need | Format |
|----------|------|--------|
| Developers | Quick check | Table (human-readable) |
| CI/CD systems | Validation | JSON (machine-readable) |
| Documentation | Examples | YAML (clean, readable) |
| Data analysis | Export | TSV (spreadsheet-compatible) |
| Config files | Integration | TOML (Rust-native) |

### The Solution: Format Enum

```rust
pub enum OutputFormat {
    Json,
    Yaml,
    Toml,
    Table,
    Tsv,
}
```

**Why five formats?**

Each serves a specific purpose:
- **JSON** - Universal standard, precise types
- **YAML** - Human-readable, configuration
- **TOML** - Rust ecosystem standard
- **Table** - Terminal visualization
- **TSV** - Data exchange, spreadsheets

**How it works:**

1. Collect data into struct
2. Let user choose format
3. Serialize appropriately
4. Print output

```rust
let status = Status { features: vec![...], examples: vec![...] };
match format {
    OutputFormat::Json => println!("{}", serde_json::to_string(&status)?),
    OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&status)?),
    // ... other formats
}
```

---

## Command-Line Interface Design

### Flag Standardization

All commands support consistent flags:

```bash
# Common flags
playground <NOUN> <VERB> -v          # Verbosity
playground <NOUN> <VERB> --format    # Output format
playground <NOUN> <VERB> -h          # Help
```

**Clap-noun-verb macros enable this:**

```rust
#[verb]
fn stat(
    #[arg(short = 'v', action = "count")]
    verbose: usize,
    #[arg(short = 'f', long, default_value = "json")]
    format: String,
) -> Result<Status> {
    // Implementation
}
```

### Verbosity Design

Three levels of detail:

```
-v   (1)  →  Basic context
-vv  (2)  →  More information
-vvv (3)  →  Maximum detail
```

**Why action="count" instead of boolean flags?**

```bash
# Boolean approach (limited)
playground core stat --verbose      # Can't get levels
playground core stat --very-verbose # Clunky names

# Count approach (elegant)
playground core stat -v       # Level 1
playground core stat -vv      # Level 2
playground core stat -vvv     # Level 3
```

---

## Error Handling Strategy

### Result-Based, Not Panics

Every operation returns `Result<T>`:

```rust
// ❌ NOT: fn stat() -> Status
// ✅ YES:
fn stat() -> Result<Status> {
    // Can fail gracefully
}
```

**Why?**

- ✅ Composable error chains
- ✅ Graceful error messages
- ✅ Testable failure paths
- ✅ No unwrap/expect (Poka-Yoke principle)

### Exit Codes

- **0** - Success
- **1** - General error
- **2** - Command-line error
- **3** - Runtime error

---

## Integration with Chicago TDD

### How Playground Demonstrates Patterns

Each module showcases real patterns:

#### Core Module
- `fixtures` - Test setup and teardown
- `builders` - Test data construction
- `assertions` - Custom assertions
- `state` - Type-level state machines

#### Testing Module
- `property` - Property-based testing with proptest
- `snapshot` - Snapshot testing with insta
- `mutation` - Mutation operators
- `concurrency` - Deterministic concurrency tests

#### Validation Module
- `coverage` - Code coverage analysis
- `guards` - Constraint checking
- `jtbd` - Jobs-To-Be-Done validation
- `performance` - Benchmarking

### Reflection Pattern

Playground uses `std::reflect` patterns to:
- ✅ Discover available examples
- ✅ Enumerate module capabilities
- ✅ Display available features

```rust
// Examples:
let examples = core::examples();     // Get available examples
for example in examples {
    println!("- {}", example.name);
}
```

---

## Why Five Modules?

```
Chicago TDD Framework Structure
│
├─ Core (Foundation)
│  └─ Everything else builds on core
│
├─ Testing (Techniques)
│  └─ Advanced testing patterns
│
├─ Validation (Quality)
│  └─ Ensuring code quality
│
├─ Observability (Insights)
│  └─ Understanding system behavior
│
└─ Integration (Real-World)
   └─ Testing with real services
```

**Why this grouping?**

- **Logical progression** - Foundation → Techniques → Quality
- **Mirrors user journey** - Learn core → Apply techniques → Validate quality
- **Organized by concern** - Each module has clear purpose
- **Framework structure** - Matches crate organization

---

## Design Decisions

### Decision: Noun-Verb vs Subcommands

**Alternative:** Traditional subcommands
```bash
playground stat core
playground stat testing
```

**Chosen:** Noun-verb pattern
```bash
playground core stat
playground testing stat
```

**Why noun-verb?**

- Reads more naturally in English
- Discoverability: "What verbs can I do?" instead of "What subcommands exist?"
- Consistency across nouns
- Used by major CLI tools (git, kubectl)

---

### Decision: Five Formats vs Single Format

**Alternative:** Single JSON output
```bash
playground core stat        # Always JSON
jq . for processing         # User's responsibility
```

**Chosen:** Five format options
```bash
playground core stat --format table    # Human-readable
playground core stat --format json     # Machine-readable
playground core stat --format yaml     # Configuration
```

**Why multiple formats?**

- ✅ Serves different users (developers, scripts, docs)
- ✅ No need for external tools for basics
- ✅ One command works for multiple workflows
- ✅ Reduces barrier to entry

---

### Decision: Verbosity Levels

**Alternative:** Boolean flags
```bash
playground core stat --verbose    # One level
playground core stat --quiet      # Less verbose
```

**Chosen:** Count-based verbosity
```bash
playground core stat      # Default
playground core stat -v   # Slightly verbose
playground core stat -vv  # More verbose
playground core stat -vvv # Maximum verbose
```

**Why levels?**

- More nuanced control
- Familiar to users (git, grep, etc.)
- Extensible (can add -vvvv if needed)
- Compact syntax

---

## Extensibility Design

### Adding New Modules

To add a new module (e.g., `async`):

```rust
// 1. Create module
pub mod async_testing;

// 2. Register with auto-discovery (linkme macro)
#[verb]
fn stat() -> Result<Status> { ... }

// 3. Works automatically
// playground async stat
// playground async list
// playground async exec
```

The `clap-noun-verb` crate handles:
- ✅ Auto-discovery via `linkme` macros
- ✅ Automatic help generation
- ✅ Command routing

### Adding New Verbs

Could add new verbs if needed:

```rust
// Would work for all nouns automatically
playground core bench       // Benchmarking
playground core debug       # Debugging
playground core validate    # Validation
```

**Why it's extensible:**

- Noun-verb pattern naturally scales
- `clap-noun-verb` handles auto-discovery
- No central registry needed
- Each module self-contained

---

## Testing the Playground

The playground itself is tested using Chicago TDD patterns:

```rust
// Use assert! macros
test!(test_core_stat, {
    let status = core::stat()?;
    assert!(status.features.len() > 0);
});

// Use fixtures
fixture_test!(test_with_data, fixture, {
    let result = run_command(&fixture);
    assert_ok!(result);
});
```

**Why test the testing framework?**

- ✅ Validates patterns work
- ✅ Demonstrates best practices
- ✅ Catches regressions
- ✅ Serves as documentation

---

## Performance Considerations

### Command Speed

```
stat   <100ms   Quick status
list   <200ms   Enumerate examples
exec   1-30s    Run demonstrations
```

**Why so fast?**

- No file I/O (by default)
- In-memory data structures
- Simple formatting
- Direct function calls

---

## Dependencies Philosophy

### Minimal Core

Core dependencies:
- `tokio` - Async runtime
- `serde` - Serialization
- `clap-noun-verb` - CLI framework
- `thiserror` - Error handling

**Why minimal?**

- Fast compilation
- Fewer security concerns
- Clear ownership
- Easy to understand

### Format-Specific Dependencies

Only included when needed:
- `serde_json` - JSON serialization (always)
- `serde_yaml` - YAML serialization
- `toml` - TOML parsing
- `prettytable-rs` - Table formatting
- `csv` - TSV formatting

---

## Future Evolution

### Potential Enhancements

1. **Interactive mode** - `playground interactive` for REPL-like experience
2. **Custom filters** - `--filter "features.contains('assert')"`
3. **Report generation** - `--report html/pdf`
4. **Real-time monitoring** - `--watch` for continuous checking
5. **Integration scripts** - Generate shell scripts for automation

### Design Ready For

- ✅ More output formats (XML, Protobuf, etc.)
- ✅ More complex filtering
- ✅ Interactive features
- ✅ Plugin system

---

## Key Principles Summary

| Principle | Implementation | Benefit |
|-----------|---|---|
| **Noun-Verb CLI** | `noun verb` structure | Natural, discoverability |
| **Multi-Format** | 5 output formats | Serves all users |
| **Result-Based** | No panics, use Result | Robust, testable |
| **Modular** | Mirror framework structure | Clear mapping |
| **Extensible** | Auto-discovery | Easy to add features |
| **Fast** | <100ms for status | Responsive |
| **Well-Tested** | Using Chicago TDD | Confidence |

---

## Related Documentation

- **[Getting Started Tutorial](../tutorials/GETTING_STARTED.md)** - Learn to use
- **[Command Reference](../reference/COMMAND_REFERENCE.md)** - Complete API
- **[Design Decisions](DESIGN_DECISIONS.md)** - Deep dive on decisions

---

**Architecture Version:** 1.0.0 | **Updated:** 2025-11-15
