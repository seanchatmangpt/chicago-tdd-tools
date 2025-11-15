# How to Generate JSON Output for Automation

**Quick reference** for extracting JSON output from the playground for scripting and automation.

## Why JSON Output?

JSON output is perfect for:
- Automation and scripting
- CI/CD integration
- Processing with tools like `jq`
- Machine-readable results
- Dashboard integration
- Monitoring and alerting

## Quick Commands

```bash
# Default: JSON output
cargo run -- core stat

# Parse with jq
cargo run -- core stat | jq '.features[] | .name'

# Chain commands
cargo run -- core list | jq '.examples'
cargo run -- core exec --names "fixtures" | jq '.status'
```

## Output Formats

### Status Output (JSON)

```bash
cargo run -- core stat
```

Returns:

```json
{
  "category": "core",
  "features": [
    {
      "name": "fixtures",
      "status": "enabled",
      "description": "Test fixtures with setup/teardown"
    },
    {
      "name": "builders",
      "status": "enabled",
      "description": "Fluent builders for test data"
    }
  ]
}
```

### List Output (JSON)

```bash
cargo run -- core list
```

Returns:

```json
{
  "category": "core",
  "examples": [
    {
      "name": "fixtures",
      "description": "Core test fixtures"
    },
    {
      "name": "builders",
      "description": "Fluent builders for test data"
    }
  ]
}
```

### Execution Output (JSON)

```bash
cargo run -- core exec --names "fixtures"
```

Returns:

```json
{
  "example": "fixtures",
  "status": "success",
  "duration_ms": 45,
  "message": "Test fixtures demonstrated successfully",
  "details": {
    "fixtures_created": 3,
    "assertions_passed": 15
  }
}
```

### Multiple Execution Output (JSON Array)

```bash
cargo run -- core exec --names "fixtures builders assertions"
```

Returns:

```json
[
  {
    "example": "fixtures",
    "status": "success",
    "duration_ms": 45
  },
  {
    "example": "builders",
    "status": "success",
    "duration_ms": 32
  },
  {
    "example": "assertions",
    "status": "success",
    "duration_ms": 28
  }
]
```

## Parsing JSON with jq

Extract specific fields:

```bash
# Get all example names
cargo run -- core list | jq '.examples[].name'

# Get success status
cargo run -- core exec --names "fixtures" | jq '.status'

# Filter by status
cargo run -- core exec --names "fixtures builders" | jq 'map(select(.status == "success"))'

# Get duration
cargo run -- core exec --names "fixtures" | jq '.duration_ms'

# Pretty print
cargo run -- core stat | jq '.'
```

## Common jq Recipes

### List all enabled features

```bash
cargo run -- core stat | jq '.features[] | select(.status == "enabled") | .name'
```

### Check execution results

```bash
cargo run -- test exec --names "prop mut" | jq 'map({name: .example, result: .status})'
```

### Extract error details

```bash
cargo run -- test exec --names "prop" | jq 'if .status == "error" then .error_message else "success" end'
```

### Count assertions passed

```bash
cargo run -- valid exec --names "cov" | jq '.details.assertions_passed'
```

## Scripting Examples

### Bash Script: Run and Check Status

```bash
#!/bin/bash

RESULT=$(cargo run -- core exec --names "fixtures" | jq '.status')

if [ "$RESULT" = '"success"' ]; then
    echo "✅ Fixtures example passed"
    exit 0
else
    echo "❌ Fixtures example failed"
    exit 1
fi
```

### Bash Script: Run All and Report

```bash
#!/bin/bash

echo "Running all core examples..."

RESULTS=$(cargo run -- core exec --names "fixtures builders assertions macros")

echo "$RESULTS" | jq -r '.[] | "\(.example): \(.status)"'

FAILURES=$(echo "$RESULTS" | jq 'map(select(.status != "success")) | length')

if [ "$FAILURES" -eq 0 ]; then
    echo "✅ All passed"
    exit 0
else
    echo "❌ $FAILURES failed"
    exit 1
fi
```

### Python Script: Parse Results

```python
#!/usr/bin/env python3

import subprocess
import json
import sys

# Run playground command
result = subprocess.run(
    ["cargo", "run", "--", "core", "exec", "--names", "fixtures"],
    capture_output=True,
    text=True
)

# Parse JSON
data = json.loads(result.stdout)

# Check status
if data["status"] == "success":
    print(f"✅ {data['example']} passed in {data['duration_ms']}ms")
    sys.exit(0)
else:
    print(f"❌ {data['example']} failed")
    sys.exit(1)
```

### Python Script: Batch Execution

```python
#!/usr/bin/env python3

import subprocess
import json

examples = ["fixtures", "builders", "assertions", "macros"]
results = []

for example in examples:
    cmd = ["cargo", "run", "--", "core", "exec", "--names", example]
    result = subprocess.run(cmd, capture_output=True, text=True)
    data = json.loads(result.stdout)
    results.append({
        "example": data["example"],
        "status": data["status"],
        "duration": data["duration_ms"]
    })

# Print summary
total_duration = sum(r["duration"] for r in results)
successes = len([r for r in results if r["status"] == "success"])

print(f"Total: {successes}/{len(results)} passed in {total_duration}ms")

for r in results:
    status = "✅" if r["status"] == "success" else "❌"
    print(f"  {status} {r['example']}: {r['duration']}ms")
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Playground Tests
on: [push]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run playground
        run: |
          cd playground
          cargo run -- core exec --names "fixtures builders assertions" > results.json

      - name: Check results
        run: |
          STATUS=$(jq '.[] | select(.status != "success") | .status' < results.json)
          if [ -n "$STATUS" ]; then
            echo "❌ Some tests failed"
            exit 1
          fi
          echo "✅ All tests passed"
```

### GitLab CI Example

```yaml
test_playground:
  script:
    - cd playground
    - cargo run -- test exec --names "prop mut snap" > results.json
    - jq 'map(select(.status == "success")) | length' results.json
  artifacts:
    reports:
      junit: results.json
```

## Webhook Integration

### Send Results to Slack

```bash
#!/bin/bash

RESULT=$(cargo run -- core exec --names "fixtures" | jq '.status')

curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
  -H 'Content-Type: application/json' \
  -d "{\"text\":\"Playground test: $RESULT\"}"
```

### Send to Monitoring System

```bash
#!/bin/bash

DURATION=$(cargo run -- core exec --names "fixtures" | jq '.duration_ms')

# Send to monitoring system
curl -X POST http://monitoring.local/metrics \
  -H 'Content-Type: application/json' \
  -d "{\"metric\":\"playground_duration\",\"value\":$DURATION}"
```

## JSON Schema Reference

### Execution Result

```json
{
  "example": "string",              // Example name
  "status": "success|error",        // Execution status
  "duration_ms": number,            // Execution time
  "message": "string",              // Human message
  "details": {                      // Feature-specific details
    "fixtures_created": number,
    "assertions_passed": number,
    // ... more details
  }
}
```

### Status Response

```json
{
  "category": "string",             // Feature category
  "features": [
    {
      "name": "string",             // Feature name
      "status": "enabled|disabled",
      "description": "string"
    }
  ]
}
```

## Troubleshooting

**Q: "jq: command not found"**
A: Install jq:
```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq

# Windows (chocolatey)
choco install jq
```

**Q: "Invalid JSON output"**
A: Ensure you're capturing only JSON:
```bash
# Wrong: Captures cargo output too
cargo run -- core exec --names "fixtures" | jq '.'

# Right: Use 2>/dev/null to suppress warnings
cargo run -- core exec --names "fixtures" 2>/dev/null | jq '.'
```

**Q: "Python script can't parse JSON"**
A: Ensure clean output:
```python
# Capture only stdout, not stderr
result = subprocess.run(
    cmd,
    capture_output=True,
    text=True,
    stderr=subprocess.DEVNULL
)
```

## Best Practices

1. **Capture stderr separately** - JSON is on stdout
2. **Use jq for parsing** - More reliable than string parsing
3. **Validate status field** - Always check `.status`
4. **Parse duration** - Track performance trends
5. **Log results** - Archive JSON for analysis

## Next Steps

- **Copy to your project** → [Copying Examples](../tutorials/copying-examples.md)
- **See all commands** → [CLI Command Reference](../reference/cli-commands.md)
- **Learn CLI patterns** → [Noun-Verb Pattern](../explanation/noun-verb-pattern.md)

---

Automate testing validation with JSON output and standard tools.
