# Getting Started Tutorial

**Estimated time**: 5-10 minutes

This tutorial walks you through installing and running the Chicago TDD Tools Playground for the first time.

## Prerequisites

- Rust 1.70 or later (check with `rustc --version`)
- Cargo (comes with Rust)
- cargo-make (we'll install it)
- Git (to clone the repository)

## Step 1: Verify Rust Installation

```bash
rustc --version  # Should show 1.70 or later
cargo --version  # Should show recent version
```

If you don't have Rust installed, visit [rustup.rs](https://rustup.rs/).

## Step 2: Install cargo-make

cargo-make is required for the build system:

```bash
cargo install cargo-make
cargo make --version  # Verify installation
```

## Step 3: Clone the Repository

```bash
git clone https://github.com/seanchatmangpt/chicago-tdd-tools.git
cd chicago-tdd-tools/playground
```

## Step 4: Run Your First Command

The playground uses a noun-verb command pattern. Start simple:

```bash
cargo run -- --help
```

You should see:

```
Usage: playground <COMMAND>

Commands:
  core    Demonstrate core testing features
  test    Demonstrate testing features (optional)
  valid   Demonstrate validation features
  obs     Demonstrate observability features (optional)
  integ   Demonstrate integration features (optional)
  help    Print this message or the help of a subcommand
```

## Step 5: Check Status

```bash
cargo run -- core stat
```

This shows which core features are available and working.

**Output (JSON)**:
```json
{
  "features": [
    { "name": "fixtures", "status": "enabled" },
    { "name": "builders", "status": "enabled" },
    { "name": "assertions", "status": "enabled" },
    { "name": "macros", "status": "enabled" },
    { "name": "state", "status": "enabled" }
  ]
}
```

## Step 6: List Available Core Examples

```bash
cargo run -- core list
```

This shows all the examples you can run.

**Output**:
```
Available core examples:
  - fixtures      Core test fixtures
  - builders      Fluent builders for test data
  - assertions    Assertion helpers
  - macros        Test macros
  - state         Type-level state enforcement
```

## Step 7: Run Your First Example

Execute the fixtures example:

```bash
cargo run -- core exec --names "fixtures"
```

**Output**:
```json
{
  "example": "fixtures",
  "status": "success",
  "message": "Test fixtures demonstrated successfully",
  "details": {
    "fixtures_created": 3,
    "assertions_passed": 15,
    "duration_ms": 45
  }
}
```

## Step 8: Run Multiple Examples

You can run several examples at once:

```bash
cargo run -- core exec --names "fixtures builders assertions"
```

## Step 9: Explore Other Feature Groups

Now try other features:

```bash
# Validation features
cargo run -- valid stat
cargo run -- valid list

# Testing features (if available)
cargo run -- test stat
cargo run -- test list

# Observability (if Docker available)
cargo run -- obs stat
```

## Next Steps

- **Learn more about core features** → [Running Core Examples Tutorial](running-core-examples.md)
- **Understand the noun-verb pattern** → [Noun-Verb Pattern Guide](../explanation/noun-verb-pattern.md)
- **Copy examples to your project** → [Copying Examples Tutorial](copying-examples.md)
- **See all available commands** → [CLI Command Reference](../reference/cli-commands.md)

## Troubleshooting

### "cargo-make not found"
```bash
cargo install cargo-make
```

### "cannot find binary playg"
The playground runs with `cargo run`. If you want a `playg` command, build and install it:
```bash
cargo install --path .
playg core stat  # Now you can use playg directly
```

### "feature 'X' is required"
Install and run the playground with all features:
```bash
cargo run --all-features -- core stat
```

### Tests fail with "Docker not available"
Docker is optional. To skip Docker tests:
```bash
cargo test --features "testing-extras" --no-default-features
```

## Success Criteria

✅ You can run `cargo run -- core stat` successfully
✅ You can list examples with `cargo run -- core list`
✅ You can execute an example with `cargo run -- core exec --names "fixtures"`
✅ You understand the noun-verb command pattern (noun = feature group, verb = action)

## What's Next?

You're ready to:

1. **[Run more core examples](running-core-examples.md)** to understand fixtures, builders, assertions
2. **[Copy examples to your project](copying-examples.md)** to use them in real code
3. **[Explore testing features](running-feature-examples.md)** for advanced testing techniques

---

**Questions?** See [Noun-Verb Pattern](../explanation/noun-verb-pattern.md) or visit the [docs home](../README.md).
