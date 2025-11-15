# Tutorial: Understanding Output Formats

**Level:** Beginner | **Time:** 8 minutes | **Hands-on:** Yes

In this tutorial, you'll master the different ways the playground can display information—from human-friendly tables to machine-readable JSON.

## What You'll Learn

By the end of this tutorial:
- ✅ Understand the five output formats
- ✅ Know when to use each format
- ✅ Get hands-on experience with each format
- ✅ Choose the right format for your use case

## The Five Output Formats

The playground supports five formats, each optimized for different purposes:

```
┌─────────────────────────────────────────────────────┐
│ FORMAT     │ USE CASE                │ HUMAN-FRIENDLY│
├─────────────────────────────────────────────────────┤
│ JSON       │ Scripting, APIs         │ No (but readable)
│ YAML       │ Configuration files     │ Yes (clean)
│ TOML       │ Config & data files     │ Yes (declarative)
│ Table      │ Terminal viewing        │ Yes (structured)
│ TSV        │ Spreadsheets, data      │ No (tabular)
└─────────────────────────────────────────────────────┘
```

## 1. JSON Format (Default)

### What is JSON?

**JSON** (JavaScript Object Notation) is a structured, machine-readable format. It's the default because it's universal and perfect for programmatic access.

### Syntax Example

```json
{
  "features": [
    "fixtures",
    "builders",
    "assert"
  ],
  "examples": [
    "example1",
    "example2"
  ]
}
```

### When to Use JSON

✅ Scripting and automation
✅ APIs and data exchange
✅ Processing with tools like `jq`
✅ When you need precise data types
✅ Integration with other tools

### Try It

```bash
./target/release/playground core stat --format json
```

Or simply (JSON is the default):

```bash
./target/release/playground core stat
```

### Power Tip: Use jq for Filtering

JSON becomes powerful when combined with `jq`:

```bash
# Get just the features list
./target/release/playground core stat | jq .features

# Get the first feature
./target/release/playground core stat | jq '.features[0]'

# Count features
./target/release/playground core stat | jq '.features | length'
```

---

## 2. YAML Format

### What is YAML?

**YAML** (YAML Ain't Markup Language) is a human-friendly format popular in configuration files and documentation.

### Syntax Example

```yaml
features:
  - fixtures
  - builders
  - assert
examples:
  - example1
  - example2
```

### Why YAML is Readable

Notice how YAML:
- Uses indentation instead of brackets
- Doesn't require quotes for strings
- Reads like English prose
- Shows structure clearly

### When to Use YAML

✅ Configuration files
✅ Documentation and examples
✅ When humans will read the file
✅ DevOps and infrastructure
✅ Kubernetes manifests

### Try It

```bash
./target/release/playground core stat --format yaml
```

### Comparison: JSON vs YAML

**JSON** (compact, structured):
```json
{"features":["fixtures","builders"],"examples":["example1"]}
```

**YAML** (readable, scannable):
```yaml
features:
  - fixtures
  - builders
examples:
  - example1
```

---

## 3. TOML Format

### What is TOML?

**TOML** (Tom's Obvious, Minimal Language) is designed for configuration files. It's similar to YAML but more explicit.

### Syntax Example

```toml
features = ["fixtures", "builders", "assert"]
examples = ["example1", "example2"]
```

### When to Use TOML

✅ Configuration files (like Cargo.toml)
✅ Application settings
✅ When you need explicit structure
✅ Rust/TOML ecosystem
✅ Settings with clear hierarchy

### Try It

```bash
./target/release/playground core stat --format toml
```

### Format Characteristics

- Square brackets for arrays
- Equals signs for assignments
- Clear, hierarchical structure
- Perfect for config files

---

## 4. Table Format

### What is Table Format?

**Table** format displays data in a visual, columnar format—like a spreadsheet in your terminal.

### Example Output

```
FEATURES        EXAMPLES
─────────────   ─────────────
fixtures        example1
builders        example2
assert          example3
macros          ...
```

### When to Use Table

✅ Quick visual inspection
✅ Terminal/console viewing
✅ When you're reading directly
✅ Presentation and reporting
✅ Not for scripting (use JSON/YAML instead)

### Try It

```bash
./target/release/playground core stat --format table
```

### Why Table Format?

- **Instant Readability:** See all data at a glance
- **Column Alignment:** Easy to scan vertically
- **Terminal Friendly:** Works in any terminal
- **No Special Tools:** No need for `jq` or other utilities

### Comparison: Machine-Readable vs Human-Readable

```bash
# Machine-readable (JSON)
./target/release/playground core stat --format json | wc -l

# Human-readable (Table)
./target/release/playground core stat --format table
```

---

## 5. TSV Format

### What is TSV?

**TSV** (Tab-Separated Values) is a simple columnar format where values are separated by tabs. Perfect for spreadsheets.

### Syntax Example

```
features     examples
fixtures     example1
builders     example2
assert       example3
```

### When to Use TSV

✅ Spreadsheets (Excel, Google Sheets)
✅ Data analysis
✅ Simple columnar data
✅ Import into databases
✅ Lightweight data exchange

### Try It

```bash
./target/release/playground core stat --format tsv
```

### TSV in Practice

1. **Generate TSV file:**
```bash
./target/release/playground core stat --format tsv > core_features.tsv
```

2. **Open in Excel/Sheets:**
- Double-click the .tsv file
- Or: File → Open → Select .tsv

3. **Process in spreadsheet:**
- Sort by features
- Filter examples
- Create charts

---

## Choosing the Right Format

### Decision Tree

```
Do you need to process it programmatically?
  ├─ YES → Use JSON
  │
Do you want it human-readable?
  ├─ YES → Do you need columns/tables?
  │  ├─ YES → Use Table
  │  └─ NO → Use YAML
  │
Do you need to import to spreadsheet?
  └─ YES → Use TSV
```

### Format Selection Guide

| Task | Best Format | Reason |
|------|-------------|--------|
| Shell script | JSON | Pipe to `jq` |
| Configuration | YAML or TOML | Human-friendly |
| Quick check | Table | Instant viewing |
| Spreadsheet | TSV | Direct import |
| Documentation | YAML | Readable examples |
| API response | JSON | Standard format |
| DevOps | YAML | Industry standard |

---

## Real-World Examples

### Example 1: Share Results in Documentation

You want to document core features in your README:

```bash
./target/release/playground core stat --format yaml > docs/core-features.yaml
```

Then include in documentation:

```markdown
## Available Core Features

See `core-features.yaml` for the complete list:

```yaml
features:
  - fixtures
  - builders
  # ...
```

### Example 2: Automate Test Validation

A CI/CD script needs to verify all modules are available:

```bash
#!/bin/bash
status=$(./target/release/playground core stat --format json)
features=$(echo "$status" | jq '.features | length')
if [ "$features" -lt 5 ]; then
  echo "ERROR: Not enough features found"
  exit 1
fi
```

### Example 3: Generate Report for Team

Create a spreadsheet for stakeholders:

```bash
./target/release/playground validation stat --format tsv > validation_report.tsv
# Open in Excel → Format as table → Add charts → Share
```

---

## Format Conversion

Once you have output, you can convert between formats:

```bash
# JSON to YAML
./target/release/playground core stat --format json | yq -P

# Any format to JSON (then use jq)
./target/release/playground core stat --format yaml | yq -o json | jq .
```

Install tools for conversions:
- `yq` - YAML/JSON conversion: `brew install yq`
- `jq` - JSON queries: `brew install jq`

---

## Hands-On Practice

Try these commands in order:

```bash
# 1. View default (JSON)
./target/release/playground core stat

# 2. View as YAML
./target/release/playground core stat --format yaml

# 3. View as Table
./target/release/playground core stat --format table

# 4. Save TOML to file
./target/release/playground core stat --format toml > core.toml

# 5. Query JSON with jq
./target/release/playground core stat | jq '.features[0:3]'
```

---

## Summary

You've now learned:
- ✅ Five output formats and when to use each
- ✅ JSON for scripting
- ✅ YAML for configuration
- ✅ TOML for explicit structure
- ✅ Table for quick viewing
- ✅ TSV for spreadsheets
- ✅ Real-world use cases
- ✅ Format conversion techniques

## Next Steps

1. **[How-To: Output in Different Formats](../how-to/output-in-different-formats.md)** - Practical recipes
2. **[Command Reference](../reference/COMMAND_REFERENCE.md)** - All flags and options
3. **[How-To: Use with Scripts](../how-to/use-with-scripts.md)** - Automation patterns

---

**Tutorial Version:** 1.0.0 | **Updated:** 2025-11-15 | **Difficulty:** Beginner
