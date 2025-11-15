# How-To Guides: Playground CLI

Goal-oriented guides for solving specific problems and accomplishing common tasks.

## Quick Navigation

- **[Output in Different Formats](output-in-different-formats.md)** - JSON, YAML, TOML, Table, TSV
- **[Increase Verbosity](increase-verbosity.md)** - Get more detailed output
- **[Filter Results](filter-results.md)** - Find specific information
- **[Use with Shell Scripts](use-with-scripts.md)** - Automation and integration
- **[Debug Test Failures](debug-test-failures.md)** - Troubleshooting guide
- **[Export Results](export-results.md)** - Save output to files

---

## Guides by Use Case

### For Developers

- **I want to understand a testing pattern** → [How-To: Debug Test Failures](debug-test-failures.md)
- **I want to use playground in my test suite** → [How-To: Use with Shell Scripts](use-with-scripts.md)
- **I need to document test capabilities** → [How-To: Output in Different Formats](output-in-different-formats.md)

### For DevOps / Automation

- **I want to automate validation** → [How-To: Use with Shell Scripts](use-with-scripts.md)
- **I need to log test status** → [How-To: Output in Different Formats](output-in-different-formats.md) + [How-To: Increase Verbosity](increase-verbosity.md)
- **I want to monitor test health** → [How-To: Filter Results](filter-results.md)

### For Documentation

- **I want to include test examples** → [How-To: Output in Different Formats](output-in-different-formats.md)
- **I need to export test capabilities** → [How-To: Export Results](export-results.md)
- **I want to show verbose examples** → [How-To: Increase Verbosity](increase-verbosity.md)

### For Analysis & Reporting

- **I want to analyze test coverage** → [How-To: Filter Results](filter-results.md)
- **I need to generate reports** → [How-To: Export Results](export-results.md)
- **I want detailed statistics** → [How-To: Increase Verbosity](increase-verbosity.md)

---

## All Guides at a Glance

| Guide | Purpose | Difficulty | Time |
|-------|---------|-----------|------|
| [Output in Different Formats](output-in-different-formats.md) | Choose and use different output formats | Beginner | 5 min |
| [Increase Verbosity](increase-verbosity.md) | Get detailed output with -v flags | Beginner | 3 min |
| [Filter Results](filter-results.md) | Find specific information using tools | Intermediate | 8 min |
| [Use with Shell Scripts](use-with-scripts.md) | Integrate into bash/shell automation | Intermediate | 10 min |
| [Debug Test Failures](debug-test-failures.md) | Use playground for test debugging | Intermediate | 12 min |
| [Export Results](export-results.md) | Save output to files and formats | Beginner | 5 min |

---

## Problem Solver's Index

### "How do I...?"

**...change the output format?**
→ [How-To: Output in Different Formats](output-in-different-formats.md)

**...get more detailed information?**
→ [How-To: Increase Verbosity](increase-verbosity.md)

**...find specific features or examples?**
→ [How-To: Filter Results](filter-results.md)

**...use playground in a script?**
→ [How-To: Use with Shell Scripts](use-with-scripts.md)

**...understand why a test failed?**
→ [How-To: Debug Test Failures](debug-test-failures.md)

**...save the output to a file?**
→ [How-To: Export Results](export-results.md)

**...check if a feature is available?**
→ [How-To: Filter Results](filter-results.md)

**...integrate playground with CI/CD?**
→ [How-To: Use with Shell Scripts](use-with-scripts.md)

---

## Getting Help

- **Just starting?** → Read [Getting Started Tutorial](../tutorials/GETTING_STARTED.md) first
- **Need exact details?** → Check [Command Reference](../reference/COMMAND_REFERENCE.md)
- **Want to understand the design?** → See [Architecture Explanation](../explanation/ARCHITECTURE.md)
- **Something not listed?** → [Open an issue](https://github.com/seanchatmangpt/chicago-tdd-tools/issues)

---

## Format Guide at a Glance

Quick reference for output formats:

```
Use JSON when:   Scripting, APIs, processing with jq
Use YAML when:   Config files, documentation, readable output
Use TOML when:   Settings files, declarative config
Use Table when:  Quick visual inspection in terminal
Use TSV when:    Spreadsheets, data analysis, Excel import
```

---

## Common Recipes

### Quick Status Check
```bash
./target/release/playground core stat --format table
```

### Automate Feature Validation
```bash
#!/bin/bash
status=$(./target/release/playground core stat)
echo "Core module status: $status"
```

### Export Documentation
```bash
./target/release/playground core stat --format yaml > core_features.yaml
```

### Check Specific Features
```bash
./target/release/playground testing list | jq '.[] | select(.name | contains("snapshot"))'
```

### Generate Report
```bash
./target/release/playground validation stat --format tsv > report.tsv
# Open in Excel/Sheets
```

---

**Guide Index Version:** 1.0.0 | **Updated:** 2025-11-15

Need help? Check the [Getting Started Tutorial](../tutorials/GETTING_STARTED.md) or [Command Reference](../reference/COMMAND_REFERENCE.md).
