# How-To: Output in Different Formats

**Problem:** I want to view or export playground output in a specific format (JSON, YAML, TOML, Table, TSV).

**Solution:** Use the `--format` flag with any playground command.

---

## Quick Solutions

### I want a human-readable summary
```bash
./target/release/playground core stat --format table
```

### I want configuration file format
```bash
./target/release/playground core stat --format yaml
```

### I want to process the data programmatically
```bash
./target/release/playground core stat --format json
```

### I want to import into Excel/Sheets
```bash
./target/release/playground core stat --format tsv
```

### I want Rust config format (like Cargo.toml)
```bash
./target/release/playground core stat --format toml
```

---

## Complete Format Reference

### 1. JSON Format

**Use when:** Processing with scripts, APIs, jq

**Command:**
```bash
./target/release/playground core stat --format json
```

**Example output:**
```json
{
  "features": [
    "fixtures",
    "builders",
    "assert",
    "macros",
    "state"
  ],
  "examples": [
    "fixtures",
    "builders",
    "assert"
  ]
}
```

**With jq (JSON Query):**
```bash
# Get features only
./target/release/playground core stat | jq '.features'

# Get first feature
./target/release/playground core stat | jq '.features[0]'

# Count features
./target/release/playground core stat | jq '.features | length'

# Filter features
./target/release/playground core stat | jq '.features[] | select(. | contains("assert"))'
```

**Best for:**
- ✅ Shell scripts
- ✅ Pipeline processing
- ✅ API responses
- ✅ Automation

---

### 2. YAML Format

**Use when:** Configuration files, documentation, human-readable

**Command:**
```bash
./target/release/playground core stat --format yaml
```

**Example output:**
```yaml
features:
  - fixtures
  - builders
  - assert
  - macros
  - state
examples:
  - fixtures
  - builders
  - assert
```

**Processing with yq:**
```bash
# Convert YAML to JSON
./target/release/playground core stat --format yaml | yq -o json

# Get specific field
./target/release/playground core stat --format yaml | yq '.features'

# Extract first item
./target/release/playground core stat --format yaml | yq '.features[0]'
```

**Best for:**
- ✅ Kubernetes manifests
- ✅ Configuration files
- ✅ Documentation examples
- ✅ DevOps workflows

**Why YAML is readable:**
- Indentation shows structure
- No brackets needed
- Reads like English

---

### 3. TOML Format

**Use when:** Configuration files, Rust projects, settings

**Command:**
```bash
./target/release/playground core stat --format toml
```

**Example output:**
```toml
features = ["fixtures", "builders", "assert", "macros", "state"]
examples = ["fixtures", "builders", "assert"]
```

**Using in Cargo.toml:**
```toml
[package]
name = "my-project"
version = "0.1.0"

# Include playground features
[[test-features]]
features = ["fixtures", "builders", "assert"]
```

**Best for:**
- ✅ Rust config files
- ✅ Settings and options
- ✅ Declarative configuration
- ✅ Cargo.toml-like formats

---

### 4. Table Format

**Use when:** Quick visual inspection, human viewing

**Command:**
```bash
./target/release/playground core stat --format table
```

**Example output:**
```
FEATURES        EXAMPLES
──────────────  ──────────────
fixtures        fixtures
builders        builders
assert          assert
macros          (empty)
state           (empty)
type_level      (empty)
const_assert    (empty)
alert           (empty)
```

**Best for:**
- ✅ Quick terminal inspection
- ✅ Presentations
- ✅ Visual reporting
- ✅ Casual browsing

**Why Table format?**
- Instant visual clarity
- Column alignment
- No special tools needed
- Perfect for terminal

---

### 5. TSV Format

**Use when:** Spreadsheets, data analysis, Excel import

**Command:**
```bash
./target/release/playground core stat --format tsv
```

**Example output:**
```
features	examples
fixtures	fixtures
builders	builders
assert	assert
macros
state
type_level
const_assert
alert
```

**Excel Import Workflow:**
1. Generate TSV:
```bash
./target/release/playground core stat --format tsv > data.tsv
```

2. Open in Excel:
   - File → Open → Select .tsv file
   - Click "OK" to import with default settings

3. Excel automatically:
   - Detects tab separators
   - Creates columns
   - Formats as spreadsheet

**Processing with shell tools:**
```bash
# Convert TSV to CSV
./target/release/playground core stat --format tsv | tr '\t' ',' > data.csv

# Count rows
./target/release/playground core stat --format tsv | wc -l

# Get specific column (using awk)
./target/release/playground core stat --format tsv | awk -F'\t' '{print $1}'
```

**Best for:**
- ✅ Excel/Sheets import
- ✅ Data analysis
- ✅ Database import
- ✅ Tabular data

---

## Practical Workflows

### Workflow 1: Quick Check During Development

```bash
# I need to quickly check available features
./target/release/playground core stat --format table

# Output is readable in terminal, no piping needed
```

### Workflow 2: Documentation with Examples

```bash
# Generate YAML for docs
./target/release/playground testing list --format yaml > testing_examples.yaml

# Include in README:
# See [testing_examples.yaml](testing_examples.yaml) for all examples
```

### Workflow 3: CI/CD Validation

```bash
#!/bin/bash
# Check if required features exist

features=$(./target/release/playground core stat --format json | jq '.features')

if echo "$features" | grep -q "fixtures"; then
    echo "✓ Fixtures available"
else
    echo "✗ Fixtures missing"
    exit 1
fi
```

### Workflow 4: Data Analysis in Spreadsheet

```bash
# Generate TSV report
./target/release/playground validation stat --format tsv > validation_report.tsv

# Open in Excel
open validation_report.tsv

# In Excel:
# - Format as table
# - Add charts
# - Sort/filter
# - Export for presentation
```

### Workflow 5: Format Conversion

```bash
# Convert JSON to YAML
./target/release/playground core stat --format json | jq . > temp.json
yq -P temp.json > output.yaml

# Or use yq directly for YAML input
./target/release/playground core stat --format yaml | yq -o json | jq .

# Or for TSV to CSV
./target/release/playground core stat --format tsv | tr '\t' ',' > data.csv
```

---

## Cheat Sheet

| Want | Command |
|------|---------|
| JSON (default) | `playground core stat` |
| YAML | `playground core stat --format yaml` |
| TOML | `playground core stat --format toml` |
| Table | `playground core stat --format table` |
| TSV | `playground core stat --format tsv` |

---

## Combining with Other Tools

### With jq (JSON processing)
```bash
# Filter features
playground core stat | jq '.features | map(select(. != ""))'

# Create report
playground core stat | jq '{module: "core", feature_count: (.features | length)}'
```

### With yq (YAML processing)
```bash
# Convert and filter
playground core stat --format yaml | yq '.features | map(select(. != ""))'

# Pretty print
playground core stat --format yaml | yq -P .
```

### With awk (Text processing)
```bash
# Count fields in TSV
playground core stat --format tsv | awk -F'\t' '{print NF}'

# Extract specific column
playground core stat --format tsv | awk -F'\t' '{print $1}'
```

### With grep (Text searching)
```bash
# Find format containing text
playground core stat --format yaml | grep "fixtures"

# Case-insensitive
playground core stat --format yaml | grep -i "FIXTURES"
```

---

## Format Pros and Cons

| Format | Pros | Cons | Best For |
|--------|------|------|----------|
| JSON | Universal, precise, `jq` support | Verbose | Scripts, APIs |
| YAML | Readable, clean, DevOps standard | Whitespace sensitive | Config, docs |
| TOML | Explicit, hierarchical, Rust native | Less standard | Rust projects |
| Table | Instant visual clarity | Not machine-readable | Quick checks |
| TSV | Excel-compatible, simple, minimal | Limited structure | Data analysis |

---

## Troubleshooting

### "Invalid format" error
```bash
# Wrong
./target/release/playground core stat --format xml

# Right - only these are valid: json, yaml, toml, table, tsv
./target/release/playground core stat --format json
```

### Table format missing columns
```bash
# Some columns might be empty, try:
./target/release/playground core stat -v --format table

# Or get more details:
./target/release/playground core stat --format yaml
```

### jq command not found
```bash
# Install jq
brew install jq          # macOS
sudo apt install jq      # Linux
choco install jq         # Windows
```

### yq command not found
```bash
# Install yq
brew install yq          # macOS
sudo apt install yq      # Linux
pip install yq           # Python
```

---

## Related Guides

- **[How-To: Increase Verbosity](increase-verbosity.md)** - Get more detailed output
- **[How-To: Filter Results](filter-results.md)** - Find specific data
- **[How-To: Use with Shell Scripts](use-with-scripts.md)** - Automation

---

**How-To Version:** 1.0.0 | **Updated:** 2025-11-15 | **Difficulty:** Beginner
