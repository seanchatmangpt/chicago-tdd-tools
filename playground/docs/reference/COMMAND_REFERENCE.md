# Technical Reference: Complete Command Guide

**Reference Type:** Information-oriented | **Updated:** 2025-11-15

Complete documentation of all playground commands, options, and arguments.

---

## Command Structure

```
playground [OPTIONS] <NOUN> [VERB] [VERB_OPTIONS]
```

### Components

- **`playground`** - Binary name
- **`[OPTIONS]`** - Global options (apply to all commands)
- **`<NOUN>`** - Module to operate on (core, testing, validation, etc.)
- **`[VERB]`** - Action to perform (stat, list, exec)
- **`[VERB_OPTIONS]`** - Verb-specific options

---

## Global Options

These options work with all commands:

### `--help` / `-h`
Display help information.

```bash
# Show general help
./target/release/playground --help

# Show help for a noun
./target/release/playground core --help

# Show help for a verb
./target/release/playground core stat --help
```

### `--version` / `-V`
Display version information.

```bash
./target/release/playground --version
```

---

## Nouns (Modules)

The playground is organized into these modules:

### `core`
Core testing features: fixtures, builders, assertions, macros, state, type-level assertions, const assertions, alerts.

```bash
./target/release/playground core stat
./target/release/playground core list
./target/release/playground core exec
```

**Key features:**
- Fixtures for test setup
- Test data builders
- Assertion utilities
- Type-level state machines
- Compile-time assertions
- Alert/logging utilities

---

### `testing`
Advanced testing techniques: property-based testing, mutation testing, snapshot testing, concurrency testing, parameterized testing, CLI testing.

```bash
./target/release/playground testing stat
./target/release/playground testing list
./target/release/playground testing exec
```

**Key features:**
- Property-based testing (proptest)
- Mutation testing operators
- Snapshot testing (insta)
- Concurrency testing (loom)
- Parameterized tests
- CLI testing (trycmd)

---

### `validation`
Quality and constraint validation: coverage analysis, guard constraints, Jobs To Be Done, performance benchmarking.

```bash
./target/release/playground validation stat
./target/release/playground validation list
./target/release/playground validation exec
```

**Key features:**
- Test coverage analysis
- Guard constraint checking
- JTBD requirement validation
- Performance metrics
- Benchmarking tools

---

### `observability`
Telemetry and observability: OpenTelemetry (OTEL) integration, Weaver validation, unified observability API.

```bash
./target/release/playground observability stat
./target/release/playground observability list
./target/release/playground observability exec
```

**Key features:**
- OTEL span validation
- Metrics collection
- Weaver integration
- Semantic conventions
- Telemetry testing

---

### `integration`
Integration testing support: testcontainers, Docker container management, service helpers.

```bash
./target/release/playground integration stat
./target/release/playground integration list
./target/release/playground integration exec
```

**Key features:**
- Docker testcontainers
- Service helpers (Postgres, Redis)
- Container lifecycle management
- Wait conditions
- Health checks

---

## Verbs (Actions)

Each noun supports these verbs:

### `stat`
Display status and statistics for the module.

```bash
playground <NOUN> stat [OPTIONS]
```

**Options:**
- `-v, --verbose` - Increase verbosity (use multiple times: -v, -vv, -vvv)
- `-f, --format <FORMAT>` - Output format: json, yaml, toml, table, tsv (default: json)

**Example:**
```bash
./target/release/playground core stat
./target/release/playground core stat -v
./target/release/playground core stat --format yaml
./target/release/playground core stat -vv --format table
```

**Output Structure:**
```json
{
  "features": ["list", "of", "features"],
  "examples": ["list", "of", "examples"]
}
```

---

### `list`
List available examples and features for the module.

```bash
playground <NOUN> list [OPTIONS]
```

**Options:**
- `-v, --verbose` - Increase verbosity
- `-f, --format <FORMAT>` - Output format: json, yaml, toml, table, tsv (default: json)

**Example:**
```bash
./target/release/playground core list
./target/release/playground core list --format table
./target/release/playground testing list -v
```

**Output:**
Returns a list of available examples with metadata.

---

### `exec`
Execute a demonstration or validation for the module.

```bash
playground <NOUN> exec [OPTIONS]
```

**Options:**
- `-n, --names <NAMES>` - Specific examples to execute (comma-separated)
- `-o, --output <OUTPUT>` - Output file path
- `-v, --verbose` - Increase verbosity

**Example:**
```bash
./target/release/playground core exec
./target/release/playground core exec --names fixtures,builders
./target/release/playground validation exec --output results.txt -vv
```

---

## Common Usage Patterns

### Display Patterns

**Show module status (JSON):**
```bash
./target/release/playground core stat
```

**Show module status (human-readable):**
```bash
./target/release/playground core stat --format table
```

**Show with maximum detail:**
```bash
./target/release/playground core stat -vvv
```

**Export as YAML:**
```bash
./target/release/playground core stat --format yaml > core.yaml
```

---

### Information Gathering

**List all examples in a module:**
```bash
./target/release/playground testing list
```

**Get examples with details:**
```bash
./target/release/playground testing list -v
```

**Export examples list:**
```bash
./target/release/playground testing list --format tsv > examples.tsv
```

---

### Execution Patterns

**Run all examples in a module:**
```bash
./target/release/playground validation exec
```

**Run specific examples:**
```bash
./target/release/playground core exec --names fixtures,builders
```

**Run and save output:**
```bash
./target/release/playground integration exec --output results.log -v
```

---

## Output Formats Reference

### JSON (Default)
```bash
./target/release/playground core stat --format json
```

Structure:
```json
{
  "features": ["feature1", "feature2"],
  "examples": ["example1", "example2"]
}
```

Best for: Scripting, APIs, processing with `jq`

---

### YAML
```bash
./target/release/playground core stat --format yaml
```

Structure:
```yaml
features:
  - feature1
  - feature2
examples:
  - example1
  - example2
```

Best for: Configuration files, documentation, readability

---

### TOML
```bash
./target/release/playground core stat --format toml
```

Structure:
```toml
features = ["feature1", "feature2"]
examples = ["example1", "example2"]
```

Best for: Rust config files, settings

---

### Table
```bash
./target/release/playground core stat --format table
```

Structure:
```
FEATURES   EXAMPLES
─────────  ─────────
feature1   example1
feature2   example2
```

Best for: Terminal inspection, visual clarity

---

### TSV (Tab-Separated Values)
```bash
./target/release/playground core stat --format tsv
```

Structure:
```
features    examples
feature1    example1
feature2    example2
```

Best for: Spreadsheets, Excel import, data analysis

---

## Verbosity Levels

### No Verbosity (Default)
```bash
./target/release/playground core stat
```
Shows basic information only.

### Single Verbosity (-v)
```bash
./target/release/playground core stat -v
```
Shows additional context and details.

### Double Verbosity (-vv)
```bash
./target/release/playground core stat -vv
```
Shows more comprehensive information, including metadata.

### Triple Verbosity (-vvv)
```bash
./target/release/playground core stat -vvv
```
Shows maximum detail, including diagnostic information.

---

## Exit Codes

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | Command completed successfully |
| 1 | General error | Invalid arguments, command failed |
| 2 | Command-line error | Missing required argument |
| 3 | Runtime error | File not found, I/O error |

**Check exit code:**
```bash
./target/release/playground core stat
echo $?  # Prints exit code (0 = success)
```

---

## Complete Command Examples

### Example 1: Quick Status Check
```bash
./target/release/playground core stat --format table
```

### Example 2: Export Documentation
```bash
./target/release/playground testing list --format yaml > testing_examples.yaml
```

### Example 3: Validate in CI/CD
```bash
status=$(./target/release/playground core stat --format json)
features=$(echo "$status" | jq '.features | length')
if [ "$features" -lt 5 ]; then
  echo "ERROR: Not enough features"
  exit 1
fi
```

### Example 4: Run Specific Tests
```bash
./target/release/playground validation exec --names coverage,guards -v
```

### Example 5: Generate Report
```bash
./target/release/playground integration stat --format tsv > integration_report.tsv
```

### Example 6: Maximum Detail Output
```bash
./target/release/playground observability stat -vvv --format yaml
```

---

## Combining Commands

### Pipe Output
```bash
# Process JSON with jq
./target/release/playground core stat | jq '.features'

# Convert and format
./target/release/playground core stat --format yaml | yq -o json
```

### Save to File
```bash
# Save YAML
./target/release/playground core stat --format yaml > core_status.yaml

# Save with exec output
./target/release/playground validation exec -o validation_results.txt
```

### Chain Commands
```bash
# Check status and then run if successful
./target/release/playground core stat > /dev/null && ./target/release/playground core exec
```

---

## Troubleshooting

### Command Not Found
```
Error: command not found: playground
```

**Solution:** Use full path to binary:
```bash
./target/release/playground core stat
```

### Unknown Noun
```
Error: Unknown noun 'foo'
```

**Solution:** Use valid noun (core, testing, validation, observability, integration):
```bash
./target/release/playground core stat
```

### Invalid Format
```
Error: Invalid format 'xml'
```

**Solution:** Use valid format (json, yaml, toml, table, tsv):
```bash
./target/release/playground core stat --format yaml
```

### jq Not Found
```
command not found: jq
```

**Solution:** Install jq:
```bash
brew install jq     # macOS
apt install jq      # Linux
```

---

## Performance Characteristics

| Command | Typical Time | Notes |
|---------|-------------|-------|
| `stat` | <100ms | Instant status |
| `list` | <200ms | Scanning examples |
| `exec` | 1-30s | Depends on examples |

---

## Related Documentation

- **[Tutorial: Getting Started](../tutorials/GETTING_STARTED.md)** - Learn the basics
- **[How-To: Output Formats](../how-to/output-in-different-formats.md)** - Format usage guide
- **[Explanation: Architecture](../explanation/ARCHITECTURE.md)** - Design details

---

**Reference Version:** 1.0.0 | **Updated:** 2025-11-15
