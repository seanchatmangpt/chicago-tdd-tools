# CLI Architecture: clap-noun-verb 3.7.1 in the Playground

Deep dive into how the playground uses **clap-noun-verb 3.7.1** to build a composable, type-safe CLI with zero boilerplate.

## Overview

The playground CLI (`playg`) demonstrates modern CLI design principles:

```
┌─────────────────────────────────────────────────┐
│  main.rs: clap_noun_verb::run()                 │
│  (Single line entry point)                      │
└─────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────┐
│  clap_noun_verb::run()                          │
│  ├─ Parse arguments (clap)                      │
│  ├─ Auto-discover verbs via linkme              │
│  ├─ Route to correct verb handler               │
│  └─ Serialize output to requested format        │
└─────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────┐
│  src/cli/*.rs (Verb Handlers)                   │
│  ├─ core.rs      (core noun verbs)              │
│  ├─ test.rs      (test noun verbs)              │
│  ├─ gh.rs        (GitHub Actions noun verbs)    │
│  ├─ obs.rs       (observability noun verbs)     │
│  ├─ valid.rs     (validation noun verbs)        │
│  └─ integ.rs     (integration noun verbs)       │
│                                                  │
│  Each file contains #[verb] functions that      │
│  are automatically registered at compile-time   │
└─────────────────────────────────────────────────┘
```

## How It Works: Step by Step

### 1. Function Signature → CLI Arguments (Type Inference)

clap-noun-verb uses Rust's type system to infer command arguments:

```rust
// playground/src/cli/gh.rs

#[verb]
fn stat(verbose: usize) -> Result<GhStatus> {
    //     ↓
    // Creates: playg gh stat -v, -vv, -vvv
    // (usize type infers count action)
}

#[verb]
fn check(fix: bool, verbose: usize) -> Result<Vec<String>> {
    //      ↓      ↓
    // Creates: playg gh check --fix -v -vv -vvv
    // (bool type infers flag, usize infers count)
}

#[verb]
fn list(format: Option<String>) -> Result<Vec<String>> {
    //     ↓
    // Creates: playg gh list --format <value>?
    // (Option<T> infers optional argument)
}
```

#### Type Inference Rules

| Rust Type | CLI Argument |
|-----------|--------------|
| `String` | Required: `--name <VALUE>` |
| `Option<T>` | Optional: `--name <VALUE>` |
| `bool` | Flag: `--flag` |
| `usize` | Count: `-v -vv -vvv` |
| `Vec<T>` | Multiple: `--items a b c` |
| Custom types (with `#[arg(...)]`) | Configured via attributes |

### 2. Automatic Registration via linkme

All verb functions are automatically registered at compile time:

```rust
// playground/src/cli/mod.rs

pub mod core;      // ← Import modules
pub mod test;      //
pub mod valid;     //    ↓
pub mod obs;       //    Each module contains
pub mod integ;     //    #[verb] functions
pub mod gh;        //    ↓
                   //    linkme auto-discovers them!
```

The `#[verb]` macro uses `linkme` to create a compile-time registry:

```
Compile time:
  1. Find all #[verb] functions
  2. Create distributed slice entries (linkme)
  3. Build command registry in binary

Runtime:
  1. clap parses arguments
  2. linkme-registered verbs are available
  3. Route to correct handler based on noun + verb
```

### 3. Return Type → JSON Output (Auto-Serialization)

All return types must implement `Serialize` for JSON output:

```rust
#[derive(Serialize)]
struct GhStatus {
    workflows: Vec<WorkflowStatus>,
    total_workflows: usize,
    valid_workflows: usize,
    invalid_workflows: usize,
}

#[verb]
fn stat(verbose: usize) -> Result<GhStatus> {
    Ok(GhStatus { /* ... */ })
    // Automatically serialized to JSON by clap-noun-verb
}

// Output:
// $ playg gh stat
// {"workflows":[...],"total_workflows":5,"valid_workflows":5,"invalid_workflows":0}
```

### 4. Format Conversion (Multiple Output Formats)

clap-noun-verb automatically converts Serialize types to multiple formats:

```bash
# Provided automatically by clap-noun-verb:

$ playg gh stat                       # JSON (default)
$ playg gh stat --format json         # JSON
$ playg gh stat --format yaml         # YAML
$ playg gh stat --format toml         # TOML
$ playg gh stat --format table        # ASCII Table
$ playg gh stat --format tsv          # Tab-Separated Values
```

No additional code needed—just derive `Serialize` on your return type!

## Code Organization

### Entry Point: main.rs

```rust
//! Chicago TDD Tools Playground CLI

use clap_noun_verb::Result;
use playground::cli;  // Import to trigger linkme auto-discovery

fn main() -> Result<()> {
    clap_noun_verb::run()  // That's it! Parse args and dispatch.
}
```

**Key points:**
- Single line to run the entire CLI
- `use playground::cli` triggers linkme discovery
- `clap_noun_verb::run()` handles parsing and routing

### Module Registration: src/cli/mod.rs

```rust
//! CLI commands (noun handlers)
//!
//! Each module defines verbs for a specific noun.
//! linkme automatically discovers all #[verb] functions.

pub mod core;      // playg core [stat|list|exec]
pub mod test;      // playg test [stat|list|exec]
pub mod valid;     // playg valid [stat|exec]
pub mod obs;       // playg obs [stat|otel|weav]
pub mod integ;     // playg integ [stat|contain]
pub mod gh;        // playg gh [stat|list|check|runs|open]
```

**Key points:**
- Simply importing modules triggers linkme auto-discovery
- No `pub fn main() { ... }` needed
- No explicit verb registration needed

### Verb Handlers: src/cli/gh.rs (Example)

```rust
//! GitHub Actions noun commands
//!
//! Demonstrates clap-noun-verb best practices:
//! - Type inference from function signatures
//! - Automatic serialization to JSON
//! - Multiple output format support

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct WorkflowStatus {
    name: String,
    path: String,
    jobs: usize,
    valid: bool,
}

#[derive(Serialize, Debug)]
struct GhStatus {
    workflows: Vec<WorkflowStatus>,
    total_workflows: usize,
    valid_workflows: usize,
    invalid_workflows: usize,
}

/// Show GitHub Actions status
///
/// Returns workflow statistics with validation results.
/// Use -v or -vv for verbose output.
#[verb]
fn stat(verbose: usize) -> Result<GhStatus> {
    let workflows = discover_workflows();
    let valid_count = workflows.iter().filter(|w| w.valid).count();

    if verbose > 0 {
        println!("📊 GitHub Actions Status");
        for workflow in &workflows {
            let icon = if workflow.valid { "✅" } else { "❌" };
            println!("{} {} (jobs: {})", icon, workflow.name, workflow.jobs);
        }
    }

    Ok(GhStatus {
        total_workflows: workflows.len(),
        valid_workflows: valid_count,
        invalid_workflows: workflows.len() - valid_count,
        workflows,
    })
    // Automatically serialized by clap-noun-verb
}

/// List all GitHub Actions workflows
#[verb]
fn list(format: Option<String>) -> Result<Vec<String>> {
    // ...
}

fn discover_workflows() -> Vec<WorkflowStatus> {
    // Implementation details...
}
```

**Key points:**
- `#[verb]` macro automatically registers function as verb handler
- Arguments inferred from function signature (`verbose: usize`, `format: Option<String>`)
- Return type (`Result<T>` where `T: Serialize`) automatically becomes JSON output
- No argument parsing code needed
- Doc comments show up in `--help`

## Feature: Advanced Argument Configuration

For more control over argument handling, use `#[arg(...)]` attributes:

```rust
use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

#[verb]
fn advanced(
    // Short flag with alias
    #[arg(short = 'p', alias = "port")]
    port: Option<u16>,

    // Positional argument
    #[arg(index = 0)]
    target: String,

    // Multiple values
    #[arg(multiple)]
    tags: Vec<String>,

    // Environment variable fallback
    #[arg(env = "DEBUG", default_value = "false")]
    debug: bool,
) -> Result<Output> {
    // ...
}
```

Available attributes match clap's builder API. See `clap-noun-verb` docs for complete reference.

## Feature: Async Operations

Execute async code in verb handlers using `run_async()`:

```rust
use clap_noun_verb::async_verb::run_async;

#[verb]
fn fetch(url: String) -> Result<Data> {
    run_async(async {
        let data = reqwest::get(&url).await?;
        Ok(data)
    })
}
```

## Testing Your CLI

### Build the CLI

```bash
cd playground
cargo build --bin playg
```

### Test Individual Commands

```bash
# Test stat command
cargo run --bin playg -- gh stat

# Test with verbose output
cargo run --bin playg -- gh stat -v
cargo run --bin playg -- gh stat -vv

# Test list command
cargo run --bin playg -- gh list

# Test with different formats
cargo run --bin playg -- gh list --format json
cargo run --bin playg -- gh list --format yaml
cargo run --bin playg -- gh list --format table
```

### Debug Information

```bash
# Show all available commands
cargo run --bin playg -- --help

# Show help for specific noun
cargo run --bin playg -- gh --help

# Show help for specific verb
cargo run --bin playg -- gh stat --help

# Get verbose output
cargo run --bin playg -- gh stat -vv
```

## Performance Characteristics

### Compile Time

- Attribute macros add minimal overhead (~5-10% typical)
- linkme processes are fast and scale linearly with verb count
- No code generation slowdowns

### Runtime

- Zero runtime dispatch cost for verb resolution
- JSON serialization is fast (uses standard serde ecosystem)
- Format conversion is lazy (only when requested)

### Binary Size

- Adding clap-noun-verb adds ~2-3 MB to debug build
- Release builds are optimized (debug info stripped)

## Comparison with Traditional clap

| Aspect | Traditional clap | clap-noun-verb |
|--------|-----------------|-----------------|
| Boilerplate | Enum + match arms | Just functions |
| Argument parsing | Manual | Inferred from signature |
| Output serialization | Manual | Automatic |
| Format support | Must implement manually | Built-in (JSON, YAML, TOML, Table, TSV) |
| Lines of code (avg command) | 50-100 | 15-30 |
| Extensibility | Requires enum modification | Add function + import |
| Type safety | Runtime checks | Compile-time checks |
| Error handling | Custom error types | `Result<T>` convention |

## Best Practices

### 1. Always Derive Serialize

```rust
#[derive(Serialize)]  // ← Required for JSON output
struct MyOutput {
    status: String,
    count: usize,
}

#[verb]
fn mycommand() -> Result<MyOutput> { /* ... */ }
```

### 2. Use Result<T> for Error Handling

```rust
// ✅ Good
#[verb]
fn good() -> Result<Output> {
    let file = std::fs::read("file.txt")?;  // Propagate with ?
    Ok(Output { data: file })
}

// ❌ Avoid
#[verb]
fn bad() -> Output {
    let file = std::fs::read("file.txt").expect("should work");  // Panics!
    Output { data: file }
}
```

### 3. Document with Doc Comments

```rust
/// Show system status
///
/// Returns detailed information about current system state.
/// Use -v for more details, -vv for maximum verbosity.
#[verb]
fn stat(verbose: usize) -> Result<Status> { /* ... */ }
```

Doc comments automatically appear in `--help` output.

### 4. Structure Output for JSON Clarity

```rust
#[derive(Serialize)]
struct Output {
    #[serde(rename = "status")]  // Custom field names
    status_code: usize,

    #[serde(skip)]  // Skip in JSON
    internal_state: String,
}
```

### 5. Use Validation During Parsing

```rust
#[verb]
fn process(
    #[arg(value_parser = parse_port)]  // Custom validator
    port: u16,
) -> Result<Output> {
    // Parser already validated port
    Ok(Output { port })
}

fn parse_port(s: &str) -> Result<u16, String> {
    let port: u16 = s.parse().map_err(|_| "invalid port")?;
    if port < 1024 {
        return Err("port must be >= 1024".to_string());
    }
    Ok(port)
}
```

## Debugging Tips

### Enable verbose output in your verb

```rust
#[verb]
fn mycommand(verbose: usize) -> Result<Output> {
    if verbose > 0 {
        eprintln!("Debug: processing...");
    }

    if verbose > 1 {
        eprintln!("Debug: detailed state: {:?}", state);
    }

    Ok(output)
}

// Test:
// $ playg myfeature mycommand -vv
// Debug: processing...
// Debug: detailed state: { ... }
// { JSON output follows }
```

### Print help

```bash
$ playg --help              # Show all nouns
$ playg gh --help           # Show all gh verbs
$ playg gh stat --help      # Show gh stat arguments
```

## Further Resources

- **README.md** - Playground overview
- **GH_CLI_COMMANDS.md** - Example command documentation
- **EXTENDING_CLI.md** - Guide for adding new commands
- **Official clap-noun-verb docs**: https://docs.rs/clap-noun-verb/
- **clap (underlying library)**: https://docs.rs/clap/

## Summary

The playground demonstrates how clap-noun-verb eliminates boilerplate while maintaining:

✅ Type safety (compiler enforces argument correctness)
✅ Composability (add commands by adding functions)
✅ JSON output (first-class support for automation)
✅ Multiple formats (YAML, TOML, Table, TSV built-in)
✅ Zero-cost abstractions (no runtime overhead)

This makes clap-noun-verb ideal for:
- CLI tools with many subcommands
- Machine-readable output (agents, API integration)
- Rapid CLI development
- Experimental tools and utilities
