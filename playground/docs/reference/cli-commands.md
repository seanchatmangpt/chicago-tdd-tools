# CLI Command Reference

**Complete reference** for all playground commands and options.

## Command Structure

```
playg <NOUN> <VERB> [OPTIONS]
     └──────┬──────┘ └──────┬────────┘
       Feature         Action
       Category
```

Nouns (feature categories): `core`, `test`, `valid`, `obs`, `integ`
Verbs (actions): `stat`, `list`, `exec`

## Global Commands

### Help

```bash
cargo run -- --help
cargo run -- -h
```

Shows general help.

### Version

```bash
cargo run -- --version
cargo run -- -V
```

Shows version information.

## Core Features Commands

### Show Status

```bash
cargo run -- core stat
```

**Output:** JSON with enabled/disabled features.

### List Examples

```bash
cargo run -- core list
```

**Output:** Available examples in core category.

### Execute Examples

```bash
cargo run -- core exec --names "fixtures"
cargo run -- core exec --names "fixtures builders assertions"
```

**Options:**
- `--names <NAMES>` - Space-separated example names

**Output:** JSON execution result(s).

## Testing Features Commands

### Show Status

```bash
cargo run -- test stat
```

### List Examples

```bash
cargo run -- test list
```

### Execute Examples

```bash
cargo run -- test exec --names "prop"
cargo run -- test exec --names "prop mut snap conc"
```

## Validation Features Commands

### Show Status

```bash
cargo run -- valid stat
```

### List Examples

```bash
cargo run -- valid list
```

### Execute Examples

```bash
cargo run -- valid exec --names "cov"
cargo run -- valid exec --names "cov guard jtbd perf"
```

## Observability Features Commands

### Show Status

```bash
cargo run -- obs stat
```

### List Examples

```bash
cargo run -- obs list
```

### Execute OTEL Example

```bash
cargo run -- obs otel
```

Requires `otel` feature.

### Execute Weaver Example

```bash
cargo run -- obs weav
```

Requires `weaver` feature.

### Bootstrap Weaver

```bash
cargo run -- obs bootstrap
```

Downloads Weaver CLI and registry (first time only).

### Weaver Smoke Test

```bash
cargo run -- obs smoke
```

Tests Weaver installation.

## Integration Features Commands

### Show Status

```bash
cargo run -- integ stat
```

### List Examples

```bash
cargo run -- integ list
```

### Execute Docker Example

```bash
cargo run -- integ contain
```

Requires Docker and `testcontainers` feature.

## Execution Options

### Multiple Examples

Run several at once:

```bash
cargo run -- core exec --names "fixtures builders assertions macros"
```

### With Feature Flags

```bash
# Specific feature
cargo run --features property-testing -- test exec --names "prop"

# All features
cargo run --all-features -- test exec --names "prop mut snap"

# No optional features
cargo run --no-default-features -- core exec --names "fixtures"
```

### Output Redirection

Capture JSON output:

```bash
# Save to file
cargo run -- core stat > output.json

# Parse with jq
cargo run -- core stat | jq '.features'

# Suppress cargo output (stderr)
cargo run -- core stat 2>/dev/null | jq '.'
```

## Quick Command Reference

| Goal | Command |
|------|---------|
| Check what's available | `cargo run -- core list` |
| See feature status | `cargo run -- core stat` |
| Run a single example | `cargo run -- core exec --names "fixtures"` |
| Run multiple examples | `cargo run -- core exec --names "fixtures builders"` |
| Run all core examples | `cargo run -- core exec --names "fixtures builders assertions macros state const alert"` |
| Run testing examples | `cargo run -- test exec --names "prop mut snap"` |
| Run with all features | `cargo run --all-features -- test exec --names "prop"` |
| Get JSON output | `cargo run -- core stat` |
| Filter with jq | `cargo run -- core stat \| jq '.features'` |
| Save results | `cargo run -- core stat > results.json` |

## Available Examples by Category

### Core Examples

```
fixtures      - Core test fixtures
builders      - Fluent builders for test data
assertions    - Assertion helpers
macros        - Test macros (test!, async_test!, fixture_test!)
state         - Type-level state enforcement
const         - Compile-time assertions
alert         - Alert helper macros
```

### Testing Examples

```
prop          - Property-based testing
mut           - Mutation testing
snap          - Snapshot testing
conc          - Concurrency testing
cli           - CLI testing
gen           - Test code generation
```

### Validation Examples

```
cov           - Coverage analysis
guard         - Guard constraints
jtbd          - JTBD validation
perf          - Performance testing
```

### Observability Examples

```
otel          - OpenTelemetry validation
weav          - Weaver semantic conventions
bootstrap     - Download Weaver (first time)
smoke         - Weaver smoke test
```

### Integration Examples

```
contain       - Docker/testcontainers
```

## Error Messages

### "Unknown example"

```
cargo run -- core exec --names "unknown_example"
```

**Fix:** Check available examples with `cargo run -- core list`

### "Feature required"

```
Error: Feature 'property-testing' required
```

**Fix:** Enable feature:
```bash
cargo run --features property-testing -- test exec --names "prop"
```

### "Docker not running"

```
Error: Docker daemon not available
```

**Fix:** Start Docker or skip Docker tests:
```bash
export WEAVER_ALLOW_SKIP=1
cargo run -- obs weav
```

## Usage Examples

### Learn Core Features

```bash
# See what's available
cargo run -- core stat

# Run one example
cargo run -- core exec --names "fixtures"

# Run another
cargo run -- core exec --names "builders"

# Run all
cargo run -- core exec --names "fixtures builders assertions macros"
```

### Run Full Testing Suite

```bash
# Check what's available
cargo run -- test stat

# Run all testing examples
cargo run --all-features -- test exec --names "prop mut snap conc"
```

### Validation Workflow

```bash
# Check validation features
cargo run -- valid stat

# Run validation examples
cargo run -- valid exec --names "cov guard jtbd perf"
```

### Observability Setup

```bash
# Check if OTEL is available
cargo run -- obs stat

# Try OTEL example
cargo run --features otel -- obs otel

# Bootstrap Weaver (first time)
cargo run --features weaver -- obs bootstrap

# Run Weaver validation
cargo run --features weaver -- obs weav
```

### Integration Testing

```bash
# Check Docker support
cargo run -- integ stat

# Run with Docker
cargo run --features testcontainers -- integ contain
```

## Environment Variables

### RUST_LOG

Set log level:

```bash
RUST_LOG=debug cargo run -- core exec --names "fixtures"
RUST_LOG=info cargo run -- core stat
```

### WEAVER_ALLOW_SKIP

Skip Docker requirement for Weaver:

```bash
export WEAVER_ALLOW_SKIP=1
cargo run --features weaver -- obs weav
```

## Common Workflows

### "I want to run everything"

```bash
cargo run --all-features -- core exec --names "fixtures builders assertions macros state const alert"
cargo run --all-features -- test exec --names "prop mut snap conc cli gen"
cargo run --all-features -- valid exec --names "cov guard jtbd perf"
cargo run --all-features -- obs stat
cargo run --all-features -- integ stat
```

### "I want to learn gradually"

```bash
# Day 1: Core
cargo run -- core exec --names "fixtures"

# Day 2: More core
cargo run -- core exec --names "builders assertions"

# Day 3: Advanced
cargo run --all-features -- test exec --names "prop"

# Day 4: More advanced
cargo run --all-features -- test exec --names "mut snap"
```

### "I want to validate everything"

```bash
cargo run --all-features -- core exec --names "fixtures builders assertions macros"
cargo run --all-features -- valid exec --names "cov guard jtbd perf"
cargo run --all-features -- test exec --names "prop mut snap"
```

## Next Steps

- **How-to guides** → [How-To Guides](../how-to/)
- **Tutorials** → [Tutorials](../tutorials/)
- **Understand patterns** → [Explanation](../explanation/)

---

For detailed help on specific features, see the how-to guides.
