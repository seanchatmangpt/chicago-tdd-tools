# Tutorial: Getting Started with Playground CLI

**Level:** Beginner | **Time:** 10 minutes | **Hands-on:** Yes

Welcome to the Playground! This tutorial will walk you through the basics of using the Chicago TDD Tools playground CLI—a powerful command-line tool for exploring and validating the testing framework.

## What You'll Learn

By the end of this tutorial, you will:
- ✅ Understand what the playground CLI is and why you'd use it
- ✅ Run your first playground command
- ✅ Explore different modules (core, testing, validation, observability, integration)
- ✅ View results in different formats (JSON, YAML, Table)
- ✅ Get help for any command

## Prerequisites

- Rust toolchain installed (or access to a machine with Rust)
- Basic familiarity with command-line interfaces
- The chicago-tdd-tools repository cloned locally

## What is the Playground?

The **Playground** is an interactive CLI tool that demonstrates all capabilities of the Chicago TDD testing framework. Think of it as a showcase application—it's built using Chicago TDD patterns and lets you explore each component interactively.

### Use Cases

- 🎓 **Learning:** Understand how different testing patterns work
- 🔍 **Exploration:** Discover available features and modules
- 🧪 **Validation:** Verify that specific testing components work as expected
- 📊 **Analysis:** Get detailed information about test capabilities
- 🐛 **Debugging:** Understand test failures and behavior

## Building the Playground

### Step 1: Build the Binary

In your terminal, navigate to the playground directory and build it:

```bash
cd playground
cargo build --release
```

This creates an executable at `target/release/playground` (or `playground.exe` on Windows).

### Step 2: Verify Installation

Check that the build succeeded by asking for help:

```bash
./target/release/playground --help
```

You should see output like:

```
Playground CLI - Chicago TDD Testing Framework Explorer

USAGE:
    playground <NOUN> [OPTIONS] <VERB>

NOUNS:
    core            Core testing features (fixtures, builders, assertions)
    testing         Advanced testing techniques
    validation      Quality and constraint validation
    observability   Telemetry and observability
    integration     Integration testing support

VERBS:
    stat            Show status and statistics
    list            List available examples
    exec            Execute a demonstration

OPTIONS:
    -h, --help           Print help information
    -V, --version        Print version
```

Perfect! Your playground is ready.

## Your First Command

### Step 3: Check Core Features Status

Let's explore the **core** module to see what features are available:

```bash
./target/release/playground core stat
```

You should see JSON output with all available core features:

```json
{
  "features": [
    "fixtures",
    "async",
    "builders",
    "assert",
    "macros",
    "state",
    "type_level",
    "const_assert",
    "alert"
  ],
  "examples": [
    "fixtures",
    "builders",
    "assert",
    "macros",
    "state",
    "type_level",
    "const_assert",
    "alert"
  ]
}
```

**What just happened?**
- You ran the `playground` binary
- Specified the **noun** `core` (which module to explore)
- Specified the **verb** `stat` (what action to perform)
- Received structured JSON output listing features

## Understanding the Command Structure

Playground uses a **noun-verb** pattern for commands:

```
playground [NOUN] [OPTIONS] [VERB]
     │        │      │       │
     │        │      │       └─ Action to perform (stat, list, exec)
     │        │      └──────── Flags and options
     │        └────────────── Subject/Module (core, testing, etc.)
     └─────────────────────── Program name
```

### Common Nouns (Modules)

| Noun | Purpose |
|------|---------|
| `core` | Core testing features: fixtures, builders, assertions |
| `testing` | Advanced techniques: property, snapshot, mutation, concurrency |
| `validation` | Quality metrics: coverage, guards, JTBD, performance |
| `observability` | Telemetry: OpenTelemetry, Weaver |
| `integration` | Integration testing: testcontainers, Docker support |

### Common Verbs (Actions)

| Verb | Purpose |
|------|---------|
| `stat` | Show status and statistics |
| `list` | List available examples or features |
| `exec` | Execute a demonstration or validation |

## Step 4: List Examples

Now let's see what examples are available for testing techniques:

```bash
./target/release/playground testing list
```

This shows available examples in the `testing` module. Each example demonstrates a real testing pattern.

## Step 5: Explore Validation

Let's check what validation tools are available:

```bash
./target/release/playground validation stat
```

This reveals quality assurance features like coverage analysis and constraint validation.

## Step 6: Different Output Formats

By default, commands output JSON. But you can request different formats!

### JSON Format (Default)

```bash
./target/release/playground core stat
```

### YAML Format

```bash
./target/release/playground core stat --format yaml
```

Output will look like:

```yaml
features:
  - fixtures
  - async
  - builders
  - assert
  - macros
  - state
  - type_level
  - const_assert
  - alert
examples:
  - fixtures
  - builders
  - ...
```

### Table Format

```bash
./target/release/playground core stat --format table
```

Displays results in an easy-to-read table format.

### Other Formats

Try these too:
- `--format toml` - TOML configuration format
- `--format tsv` - Tab-separated values (great for spreadsheets)

Choose the format that works best for your use case!

## Step 7: Increase Verbosity

Need more details? Use the verbose flag:

```bash
./target/release/playground core stat -v
./target/release/playground core stat -vv
./target/release/playground core stat -vvv
```

Each `-v` increases the detail level:
- `-v` → Basic details
- `-vv` → More information
- `-vvv` → Maximum verbosity

## Common Patterns

### Check Status of All Modules

```bash
# Core module status
./target/release/playground core stat

# Testing module status
./target/release/playground testing stat

# Validation module status
./target/release/playground validation stat

# Observability module status
./target/release/playground observability stat

# Integration module status
./target/release/playground integration stat
```

### List Examples in Each Module

```bash
./target/release/playground core list
./target/release/playground testing list
./target/release/playground validation list
./target/release/playground observability list
./target/release/playground integration list
```

### Get Help

```bash
# General help
./target/release/playground --help

# Help for a specific noun
./target/release/playground core --help

# Help for a specific action
./target/release/playground core stat --help
```

## Next Steps

Now that you understand the basics, explore these topics:

1. **[Output Formats Tutorial](output-formats-intro.md)** - Deep dive into formatting options
2. **[How-To: Output in Different Formats](../how-to/output-in-different-formats.md)** - Practical examples
3. **[Command Reference](../reference/COMMAND_REFERENCE.md)** - Complete command documentation
4. **[Architecture Explanation](../explanation/ARCHITECTURE.md)** - Understand the design

## Troubleshooting

### Command Not Found

If you get "command not found" error:
- Make sure you're in the correct directory
- Use `./target/release/playground` (with path) instead of just `playground`
- On Windows, use `.\target\release\playground.exe`

### Unknown Noun/Verb

```
Error: Unknown noun 'foo'
```

This means that noun doesn't exist. Use `--help` to see available nouns.

### Version Information

Want to check which version you're running?

```bash
./target/release/playground --version
```

## Summary

You've now learned:
- ✅ What the playground CLI is
- ✅ How to build and run it
- ✅ The noun-verb command structure
- ✅ How to explore different modules
- ✅ How to use different output formats
- ✅ How to increase verbosity

## Playground Power User Tips

- **Pipe to jq:** `./target/release/playground core stat | jq .features`
- **Export to file:** `./target/release/playground core stat --format yaml > output.yaml`
- **Use in scripts:** All output is machine-readable, perfect for automation
- **Check exit codes:** Commands exit with 0 on success, non-zero on failure

---

**Next:** Read [Output Formats Tutorial](output-formats-intro.md) to master formatting options!

---

**Tutorial Version:** 1.0.0 | **Updated:** 2025-11-15 | **Difficulty:** Beginner
