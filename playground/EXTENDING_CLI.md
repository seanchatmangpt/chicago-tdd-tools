# Extending the CLI: Adding New clap-noun-verb Commands

Complete guide for adding new commands to the playground CLI using **clap-noun-verb 3.7.1**.

## Quick Start: Add a Simple Command in 3 Steps

### Step 1: Create a New Module

Create `playground/src/cli/myfeature.rs`:

```rust
//! My feature noun commands

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct MyOutput {
    pub message: String,
    pub count: usize,
}

/// Show status of my feature
#[verb]
pub fn stat() -> Result<MyOutput> {
    Ok(MyOutput {
        message: "Everything looks good!".to_string(),
        count: 42,
    })
}
```

### Step 2: Register the Module

In `playground/src/cli/mod.rs`, add:

```rust
pub mod myfeature;  // ← Add this line (order doesn't matter)
```

### Step 3: Test It!

```bash
cd playground

# Build
cargo build --bin playg

# Test
cargo run --bin playg -- myfeature stat

# See JSON output
cargo run --bin playg -- myfeature stat | jq .

# Try different formats
cargo run --bin playg -- myfeature stat --format yaml
cargo run --bin playg -- myfeature stat --format table
```

That's it! No boilerplate, no enums, no match statements.

---

## Complete Example: Building a Package Manager Info Tool

Let's build a realistic example: a CLI for package management with multiple commands.

### File: playground/src/cli/pkg.rs

```rust
//! Package management noun commands
//!
//! Demonstrates:
//! - Multiple verbs with different arguments
//! - Type inference from function signatures
//! - Error handling with Result<T>
//! - Multiple output formats (automatic)

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

// ============================================================================
// Output Types (must all implement Serialize)
// ============================================================================

#[derive(Serialize)]
pub struct PkgStatus {
    pub packages: Vec<PackageInfo>,
    pub total: usize,
    pub outdated: usize,
}

#[derive(Serialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub latest: String,
    pub needs_update: bool,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub results: Vec<PackageEntry>,
    pub total_found: usize,
}

#[derive(Serialize)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct InstallResult {
    pub success: bool,
    pub package: String,
    pub version: String,
    pub message: String,
}

// ============================================================================
// Verb Handlers (automatically registered by #[verb])
// ============================================================================

/// List installed packages
///
/// Shows all installed packages with version information.
/// Use -v to show detailed version info, -vv for debug output.
#[verb]
pub fn list(verbose: usize) -> Result<PkgStatus> {
    let packages = vec![
        PackageInfo {
            name: "serde".to_string(),
            version: "1.0.0".to_string(),
            latest: "1.0.0".to_string(),
            needs_update: false,
        },
        PackageInfo {
            name: "tokio".to_string(),
            version: "1.35.0".to_string(),
            latest: "1.36.0".to_string(),
            needs_update: true,
        },
    ];

    if verbose > 0 {
        eprintln!("Found {} packages", packages.len());
    }

    if verbose > 1 {
        eprintln!("Packages: {:?}", packages);
    }

    Ok(PkgStatus {
        total: packages.len(),
        outdated: packages.iter().filter(|p| p.needs_update).count(),
        packages,
    })
}

/// Search for packages
///
/// Search the package registry by name or keyword.
/// Use --limit to control number of results.
#[verb]
pub fn search(
    #[arg(index = 0)]  // Positional argument
    query: String,

    #[arg(long, default_value = "10")]  // Named argument with default
    limit: usize,
) -> Result<SearchResult> {
    // Simulate searching
    let results = vec![
        PackageEntry {
            name: format!("{}-1", query),
            version: "1.0.0".to_string(),
            description: "A great package".to_string(),
        },
        PackageEntry {
            name: format!("{}-2", query),
            version: "2.0.0".to_string(),
            description: "Another package".to_string(),
        },
    ]
    .into_iter()
    .take(limit)
    .collect();

    Ok(SearchResult {
        total_found: results.len(),
        results,
    })
}

/// Install a package
///
/// Install a package by name, optionally specifying a version.
/// Supports both simple and complex dependency resolution.
#[verb]
pub fn install(
    #[arg(index = 0)]  // Package name is first positional arg
    package: String,

    #[arg(long)]  // Optional version flag
    version: Option<String>,

    #[arg(short = 'f')]  // Short flag for force
    force: bool,
) -> Result<InstallResult> {
    let version = version.unwrap_or_else(|| "latest".to_string());

    let message = if force {
        format!("Force installed {} at {}", package, version)
    } else {
        format!("Installed {} version {}", package, version)
    };

    Ok(InstallResult {
        success: true,
        package,
        version,
        message,
    })
}

/// Update packages
///
/// Update installed packages to their latest versions.
/// Use --all to update everything at once.
#[verb]
pub fn update(
    #[arg(long)]  // Optional package name
    package: Option<String>,

    #[arg(long)]  // Update all packages
    all: bool,

    #[arg(short = 'v', action = "count")]  // Verbose count flag
    verbose: usize,
) -> Result<String> {
    if verbose > 0 {
        eprintln!("Updating packages...");
    }

    let msg = match (package, all) {
        (Some(pkg), _) => format!("Updated {}", pkg),
        (None, true) => "Updated all packages".to_string(),
        (None, false) => "Use --package or --all to specify what to update".to_string(),
    };

    Ok(msg)
}

/// Show package status
///
/// Shows statistics about packages: installed count, outdated count, etc.
#[verb]
pub fn info(
    #[arg(index = 0, required = false)]  // Optional package name
    package: Option<String>,
) -> Result<String> {
    match package {
        Some(pkg) => Ok(format!("Detailed info for {}", pkg)),
        None => Ok("Overview of package system".to_string()),
    }
}

// ============================================================================
// Helper Functions (optional, not exposed as verbs)
// ============================================================================

fn simulate_registry_search(query: &str, limit: usize) -> Vec<PackageEntry> {
    vec![]
}
```

### File: playground/src/cli/mod.rs

Update to include the new module:

```rust
//! CLI commands (noun handlers)

pub mod core;
pub mod test;
pub mod valid;
pub mod obs;
pub mod integ;
pub mod gh;
pub mod pkg;  // ← Add this line
```

### Testing Your New Commands

```bash
cd playground
cargo build --bin playg

# Test list
cargo run --bin playg -- pkg list

# Test with verbose output
cargo run --bin playg -- pkg list -v
cargo run --bin playg -- pkg list -vv

# Test search
cargo run --bin playg -- pkg search "serde"
cargo run --bin playg -- pkg search "serde" --limit 5

# Test install
cargo run --bin playg -- pkg install "tokio"
cargo run --bin playg -- pkg install "tokio" --version "1.36.0"
cargo run --bin playg -- pkg install "tokio" --version "1.36.0" -f

# Test update
cargo run --bin playg -- pkg update --package "tokio"
cargo run --bin playg -- pkg update --all

# Test info
cargo run --bin playg -- pkg info "serde"

# Test different formats
cargo run --bin playg -- pkg list --format json
cargo run --bin playg -- pkg list --format yaml
cargo run --bin playg -- pkg list --format table
cargo run --bin playg -- pkg list --format tsv

# Get help
cargo run --bin playg -- pkg --help
cargo run --bin playg -- pkg list --help
cargo run --bin playg -- pkg search --help
```

---

## Common Patterns

### Pattern 1: Positional Arguments

```rust
#[verb]
pub fn example(
    #[arg(index = 0)]
    first: String,

    #[arg(index = 1)]
    second: String,
) -> Result<String> {
    Ok(format!("{} and {}", first, second))
}

// Usage:
// playg feature example hello world
// ↑ noun  ↑ verb   ↑ arg1 ↑ arg2
```

### Pattern 2: Optional Arguments

```rust
#[verb]
pub fn example(
    required: String,
    optional: Option<String>,
) -> Result<Output> {
    let opt = optional.unwrap_or_else(|| "default".to_string());
    Ok(Output { required, optional: opt })
}

// Usage:
// playg feature example value
// playg feature example value --optional custom
```

### Pattern 3: Flags (Boolean)

```rust
#[verb]
pub fn example(
    #[arg(short = 'f', long)]
    flag1: bool,

    #[arg(short = 'v')]
    flag2: bool,
) -> Result<String> {
    Ok(format!("Flags: {}, {}", flag1, flag2))
}

// Usage:
// playg feature example              # Both false
// playg feature example -f           # flag1 = true
// playg feature example --flag1 -v   # Both true
```

### Pattern 4: Count Flags (-v, -vv, -vvv)

```rust
#[verb]
pub fn example(
    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<String> {
    Ok(format!("Verbosity: {}", verbose))
}

// Usage:
// playg feature example      # verbose = 0
// playg feature example -v   # verbose = 1
// playg feature example -vv  # verbose = 2
// playg feature example -vvv # verbose = 3
```

### Pattern 5: Multiple Values

```rust
#[verb]
pub fn example(
    #[arg(multiple = true)]
    items: Vec<String>,
) -> Result<Output> {
    Ok(Output { items })
}

// Usage:
// playg feature example --items a b c
// playg feature example --items a --items b --items c
```

### Pattern 6: Environment Variable Fallback

```rust
#[verb]
pub fn example(
    #[arg(env = "CUSTOM_VAR", default_value = "default")]
    config: String,
) -> Result<String> {
    Ok(format!("Config: {}", config))
}

// Usage:
// CUSTOM_VAR=value playg feature example
// playg feature example                        # Uses "default"
```

### Pattern 7: Custom Value Parser

```rust
fn parse_port(s: &str) -> Result<u16, String> {
    let port: u16 = s.parse()
        .map_err(|_| "not a number".to_string())?;
    if port < 1024 {
        return Err("port must be >= 1024".to_string());
    }
    Ok(port)
}

#[verb]
pub fn server(
    #[arg(value_parser = parse_port, default_value = "8080")]
    port: u16,
) -> Result<String> {
    Ok(format!("Server on port {}", port))
}
```

### Pattern 8: Default Values

```rust
#[verb]
pub fn example(
    #[arg(long, default_value = "10")]
    timeout: u64,

    #[arg(long, default_value = "localhost")]
    host: String,
) -> Result<Output> {
    Ok(Output { timeout, host })
}

// Usage:
// playg feature example                           # Uses defaults
// playg feature example --timeout 30 --host "0.0.0.0"
```

### Pattern 9: Argument Groups (Exclusive)

```rust
#[verb]
pub fn example(
    #[arg(group = "format")]
    json: bool,

    #[arg(group = "format")]
    yaml: bool,

    #[arg(group = "format")]
    toml: bool,
) -> Result<String> {
    // Can only specify one of: --json, --yaml, --toml
    Ok("Choose one format".to_string())
}

// Usage:
// playg feature example --json
// playg feature example --yaml
// playg feature example --json --yaml  # ERROR: conflicting values
```

### Pattern 10: Requiring Dependent Arguments

```rust
#[verb]
pub fn example(
    #[arg(long, requires = "username")]
    password: Option<String>,

    #[arg(long)]
    username: Option<String>,
) -> Result<String> {
    // If --password is set, --username must also be set
    Ok("Authenticated".to_string())
}

// Usage:
// playg feature example --username admin --password secret  # OK
// playg feature example --password secret                    # ERROR: requires --username
```

---

## Best Practices

### ✅ Do This

```rust
// 1. Always implement Serialize for return types
#[derive(Serialize)]
pub struct MyOutput {
    pub status: String,
}

// 2. Use Result<T> for error handling
#[verb]
pub fn good() -> Result<MyOutput> {
    let value = do_something()?;  // Propagate errors
    Ok(MyOutput { status: value })
}

// 3. Write clear doc comments
/// Do something important
///
/// Detailed explanation of what this does and how to use it.
/// Use -v for verbose output.
#[verb]
pub fn meaningful() -> Result<String> {
    Ok("Done".to_string())
}

// 4. Use appropriate argument types
#[verb]
pub fn typed(
    count: usize,              // Will be -v -vv -vvv
    optional: Option<String>,  // Optional --optional
    flag: bool,                // Boolean --flag
) -> Result<String> {
    Ok(format!("{}:{}:{}", count, optional.is_some(), flag))
}

// 5. Validate input early
#[verb]
pub fn validated(port: u16) -> Result<String> {
    if port < 1024 {
        return Err("Port must be >= 1024".into());  // Custom error
    }
    Ok(format!("Port {}", port))
}
```

### ❌ Don't Do This

```rust
// 1. Don't forget Serialize
pub struct BadOutput {  // ← Missing #[derive(Serialize)]
    status: String,
}

// 2. Don't panic or unwrap
#[verb]
pub fn bad() -> Result<String> {
    let file = std::fs::read("file.txt").expect("should work");  // ← Panics!
    Ok(String::from_utf8(file).unwrap())  // ← Panics!
}

// 3. Don't skip documentation
#[verb]
pub fn unclear() -> Result<String> {  // ← No doc comment!
    Ok("something".to_string())
}

// 4. Don't use incorrect types
#[verb]
pub fn confusing(
    verbose: bool,  // ← Should be usize for count
    format: String, // ← Should be Option<String>
) -> Result<String> {
    Ok("confusing".to_string())
}

// 5. Don't swallow errors silently
#[verb]
pub fn silent() -> Result<String> {
    match do_something() {
        Ok(val) => Ok(val),
        Err(_) => Ok("default".to_string()),  // ← User won't know about error!
    }
}
```

---

## Testing Your Commands

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list() {
        let result = list(0).expect("list failed");
        assert!(!result.packages.is_empty());
        assert_eq!(result.total, result.packages.len());
    }

    #[test]
    fn test_search() {
        let result = search("test".to_string(), 5).expect("search failed");
        assert!(result.total_found <= 5);
    }

    #[test]
    fn test_install() {
        let result = install(
            "my-package".to_string(),
            None,
            false,
        ).expect("install failed");
        assert!(result.success);
        assert_eq!(result.package, "my-package");
    }
}
```

### Integration Testing

Create `playground/tests/cli_pkg_integration.rs`:

```rust
use std::process::Command;

#[test]
fn test_pkg_list_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "playg", "--", "pkg", "list"])
        .output()
        .expect("failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("packages"));  // JSON output
}

#[test]
fn test_pkg_search_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "playg", "--", "pkg", "search", "test"])
        .output()
        .expect("failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_pkg_output_formats() {
    for format in &["json", "yaml", "table"] {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "playg", "--", "pkg", "list", "--format", format])
            .output()
            .expect("failed to execute command");

        assert!(output.status.success(), "format {} failed", format);
    }
}
```

---

## Troubleshooting

### Command not found after adding it

**Problem**: New verb isn't recognized after adding it

**Solution**: Make sure to:
1. Import the module in `src/cli/mod.rs`:
   ```rust
   pub mod myfeature;  // ← Must be here!
   ```

2. Rebuild the binary:
   ```bash
   cargo build --bin playg
   ```

3. Check the module is actually being imported by main.rs:
   ```rust
   // playground/src/main.rs
   use playground::cli;  // ← This imports the module
   ```

### Arguments not being parsed correctly

**Problem**: `--myarg` isn't recognized

**Solution**: Check that:
1. The argument name matches function parameter name:
   ```rust
   #[verb]
   fn mycommand(myarg: String) -> Result<String> {  // ← Parameter name
       // Usage: playg feature mycommand --myarg value
   }
   ```

2. Optional arguments must be `Option<T>`:
   ```rust
   #[arg(long)]  // ← Will be optional (--myarg <value>)
   myarg: Option<String>,
   ```

3. For positional arguments, use `#[arg(index = 0)]`:
   ```rust
   #[arg(index = 0)]
   myarg: String,
   // Usage: playg feature mycommand value
   ```

### Output not showing up

**Problem**: Command runs but output is empty

**Solution**: Make sure return type implements `Serialize`:
```rust
#[derive(Serialize)]  // ← Required!
pub struct MyOutput {
    pub status: String,
}

#[verb]
pub fn mycommand() -> Result<MyOutput> {
    Ok(MyOutput { status: "ok".to_string() })
}
```

### Format conversion not working

**Problem**: `--format yaml` doesn't work

**Solution**: clap-noun-verb's built-in format support is automatic. If it's not working:
1. Make sure return type implements `Serialize`
2. Try: `cargo run --bin playg -- myfeature mycommand --format json`
3. If JSON works but YAML doesn't, ensure you've enabled optional format dependencies

---

## Advanced Topics

### Using Custom Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PkgError {
    #[error("Package not found: {0}")]
    NotFound(String),

    #[error("Invalid version: {0}")]
    InvalidVersion(String),
}

#[verb]
pub fn find(name: String) -> Result<PackageInfo, PkgError> {
    if name.is_empty() {
        return Err(PkgError::NotFound("empty".to_string()));
    }
    Ok(PackageInfo { /* ... */ })
}
```

### Sharing State Across Commands

```rust
use std::sync::Arc;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref CACHE: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

#[verb]
pub fn cached() -> Result<String> {
    let mut cache = CACHE.lock().unwrap();
    cache.push("item".to_string());
    Ok(format!("Cache size: {}", cache.len()))
}
```

### Async Operations

```rust
use clap_noun_verb::async_verb::run_async;

#[verb]
pub fn fetch_remote(url: String) -> Result<String> {
    run_async(async {
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let text = response.text().await?;
        Ok(text)
    })
}
```

---

## Summary

Adding new clap-noun-verb commands is simple:

1. **Create a file** in `src/cli/`
2. **Write verb functions** with `#[verb]` macro
3. **Import the module** in `src/cli/mod.rs`
4. **Build and test** with `cargo run --bin playg`

No enums, no match statements, no boilerplate. Just functions!

For more details, see:
- **README.md** - Playground overview
- **CLI_ARCHITECTURE.md** - Deep dive into how it works
- **GH_CLI_COMMANDS.md** - Real example with GitHub Actions
