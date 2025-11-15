# Noun-Verb Pattern Guide

**Understanding** the command structure used in the playground.

## What Is Noun-Verb?

The playground uses a **noun-verb command pattern** inspired by natural language:

```
noun  verb  options
─────────────────────
core  stat  [no options]
core  exec  --names "fixtures"
test  list  [no options]
obs   otel  [no options]
```

**Noun** = What (feature category)
**Verb** = Action (what to do with it)

## Why Noun-Verb?

### Advantage 1: Intuitive

Commands read like sentences:

```
"Show me the status of core features"
→ cargo run -- core stat

"Execute the fixtures example"
→ cargo run -- core exec --names "fixtures"

"List testing examples"
→ cargo run -- test list
```

Commands match how humans think about them.

### Advantage 2: Discoverable

Commands follow a predictable pattern:

```
<noun> stat   → Show status
<noun> list   → List available items
<noun> exec   → Execute something
```

Once you learn the pattern, all commands make sense.

### Advantage 3: Scalable

Easy to add new nouns without confusion:

```
core  - Core testing features
test  - Testing features
valid - Validation features
obs   - Observability features
integ - Integration features
```

Each noun is a category. Each category follows the same verb pattern.

### Advantage 4: Scriptable

Commands are easy to parse and script:

```bash
# Extract all feature names
cargo run -- core list | jq '.examples[].name'

# Check status of all features
cargo run -- core stat | jq '.features[] | .status'

# Run and check results
cargo run -- core exec --names "fixtures" | jq '.status'
```

Perfect for CI/CD integration.

## Noun Categories

### Nouns Available

| Noun | Purpose | Examples |
|------|---------|----------|
| `core` | Core testing features (always available) | fixtures, builders, assertions |
| `test` | Advanced testing features | property, mutation, snapshot |
| `valid` | Validation features | coverage, guards, JTBD |
| `obs` | Observability features | OTEL, Weaver |
| `integ` | Integration features | Docker, testcontainers |

### Example Nouns

```bash
cargo run -- core stat
cargo run -- test stat
cargo run -- valid stat
cargo run -- obs stat
cargo run -- integ stat
```

Each one shows what features are available in that category.

## Verbs

### Standard Verbs

| Verb | Purpose | Output |
|------|---------|--------|
| `stat` | Show status | JSON with features/status |
| `list` | List items | JSON with available examples |
| `exec` | Execute | JSON with results |

### Verb Examples

```bash
# stat - Show status
cargo run -- core stat

# list - List available
cargo run -- core list

# exec - Execute
cargo run -- core exec --names "fixtures"
```

## Examples of Noun-Verb Commands

### Learning Core Features

```bash
# What core features exist?
cargo run -- core stat

# What examples can I run?
cargo run -- core list

# Run the fixtures example
cargo run -- core exec --names "fixtures"

# Run multiple examples
cargo run -- core exec --names "fixtures builders assertions"
```

### Learning Testing Features

```bash
# Are testing features available?
cargo run -- test stat

# What testing examples exist?
cargo run -- test list

# Run property testing
cargo run -- test exec --names "prop"

# Run multiple
cargo run -- test exec --names "prop mut snap conc"
```

### Running Everything

```bash
# Check core status
cargo run -- core stat

# Check testing status
cargo run -- test stat

# Check validation status
cargo run -- valid stat

# Run all core examples
cargo run -- core exec --names "fixtures builders assertions macros state const alert"

# Run all testing examples
cargo run --all-features -- test exec --names "prop mut snap conc cli gen"

# Run all validation examples
cargo run --all-features -- valid exec --names "cov guard jtbd perf"
```

## Pattern Grammar

### Format

```
cargo run -- <noun> <verb> [OPTIONS]
```

### Noun Rules

- Always comes first
- Identifies the category
- Determines available verbs

### Verb Rules

- Always comes second
- Specifies the action
- Some verbs accept options

### Options Rules

- Come after verb
- Start with `--` (long options)
- Format: `--name "value"` or `--name value`

## Special Commands

### Observability Special Cases

Some observability operations have special names:

```bash
# Bootstrap Weaver (download Weaver CLI)
cargo run -- obs bootstrap

# Run Weaver smoke test
cargo run -- obs smoke

# OTEL validation
cargo run -- obs otel

# Weaver validation
cargo run -- obs weav
```

These are still noun-verb, but the "verb" is the action name.

## Feature Flags with Noun-Verb

### With Specific Features

```bash
# Property-based testing
cargo run --features property-testing -- test stat

# Multiple features
cargo run --features "property-testing,snapshot-testing" -- test list
```

### With All Features

```bash
cargo run --all-features -- test exec --names "prop mut snap"
```

### With No Optional Features

```bash
cargo run --no-default-features -- core stat
```

Feature flags come **before** the noun-verb command.

## Discovering Commands

### See All Features

```bash
cargo run -- core stat
cargo run -- test stat
cargo run -- valid stat
cargo run -- obs stat
cargo run -- integ stat
```

### See All Examples in a Category

```bash
cargo run -- core list
cargo run -- test list
cargo run -- valid list
```

### Get General Help

```bash
cargo run -- --help
cargo run -- -h
```

## Scripting with Noun-Verb

### Bash Scripts

```bash
#!/bin/bash
# Check if fixtures example passes
RESULT=$(cargo run -- core exec --names "fixtures" | jq '.status')
if [ "$RESULT" = '"success"' ]; then
    echo "✅ Fixtures passed"
else
    echo "❌ Fixtures failed"
fi
```

### Python Scripts

```python
import subprocess
import json

# Run command
result = subprocess.run(
    ["cargo", "run", "--", "core", "exec", "--names", "fixtures"],
    capture_output=True,
    text=True
)

# Parse JSON
data = json.loads(result.stdout)
print(f"Status: {data['status']}")
```

### JSON Parsing

```bash
# Get all feature names
cargo run -- core stat | jq -r '.features[].name'

# Get examples in test category
cargo run -- test list | jq -r '.examples[].name'

# Check if specific example passed
cargo run -- core exec --names "fixtures" | jq '.status == "success"'
```

## Common Patterns

### Pattern: Discover → Understand → Execute

```bash
# 1. Discover
cargo run -- core stat

# 2. Understand
cargo run -- core list

# 3. Execute
cargo run -- core exec --names "fixtures"
```

### Pattern: Check Status → List Options → Execute

```bash
# 1. Check
cargo run -- test stat

# 2. List
cargo run -- test list

# 3. Execute
cargo run -- test exec --names "prop"
```

### Pattern: Script Results

```bash
# Get JSON
cargo run -- core stat > features.json

# Process
jq '.features[] | select(.status == "enabled")' < features.json

# Act
while read -r feature; do
    echo "Testing: $feature"
done
```

## Comparison: Other Patterns

### vs. Positional Arguments

Noun-verb is more intuitive than positional:

```
❌ playground fixtures run
✅ core exec --names "fixtures"
```

### vs. Subcommands

Noun-verb works well as subcommands:

```
✅ cargo run -- core stat
✅ git remote add <name> <url>
✅ docker container ls
```

### vs. Flags Only

Noun-verb is clearer than flags-only:

```
❌ cargo run -- --feature core --action stat
✅ cargo run -- core stat
```

## Advantages for Users

### Clear Intent

Your command clearly expresses what you want:

```
"I want to execute the property testing example"
→ cargo run -- test exec --names "prop"
```

### Discoverable

`stat` and `list` help you explore:

```
cargo run -- test stat  # What's available?
cargo run -- test list  # Show me examples
```

### Consistent

Once you learn the pattern, all commands make sense:

```
<any-noun> stat
<any-noun> list
<any-noun> exec --names "..."
```

### Extensible

Adding new categories doesn't break the pattern:

```
cargo run -- core stat      # Works
cargo run -- test stat      # Works
cargo run -- newcategory stat  # Also works!
```

## Next Steps

- **Learn the philosophy** → [Playground Philosophy](playground-philosophy.md)
- **See all commands** → [CLI Command Reference](../reference/cli-commands.md)
- **Run examples** → [Getting Started](../tutorials/getting-started.md)

---

Noun-verb makes the playground intuitive and discoverable.
