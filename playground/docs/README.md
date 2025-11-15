# Playground CLI Documentation

Welcome to the playground documentation! This documentation is organized using the **Diátaxis** framework, which provides four distinct types of documentation for different needs.

---

## Start Here 🚀

**New to the playground?** → [Getting Started Tutorial](tutorials/GETTING_STARTED.md)

**Want to do something specific?** → [How-To Guides Index](how-to/INDEX.md)

**Need exact details?** → [Command Reference](reference/COMMAND_REFERENCE.md)

**Want to understand the design?** → [Architecture Explanation](explanation/ARCHITECTURE.md)

---

## Documentation Structure (Diátaxis)

This documentation follows the **Diátaxis** systematic approach, which organizes content into four pillars:

```
                    LEARNING
                       |
        TUTORIALS ----+---- HOW-TO GUIDES
        (Courses)     |     (Tasks)
                      |
                 UNDERSTANDING
                      |
        EXPLANATION --+----- REFERENCE
        (Discussion)  |      (Lookup)
                      |
                  INFORMATION
```

### 📚 [Tutorials](tutorials/) - Learning-Oriented

**Goal:** Help you learn and gain hands-on experience

**When to use:** You're new to playground and want to learn the basics

**Start with:**
- [Getting Started](tutorials/GETTING_STARTED.md) - Your first playground commands
- [Output Formats](tutorials/output-formats-intro.md) - Understanding format options

---

### 🔧 [How-To Guides](how-to/) - Goal-Oriented

**Goal:** Help you accomplish specific tasks and solve problems

**When to use:** You want to do something specific (e.g., "Export as YAML")

**Guides:**
- [How-To Index](how-to/INDEX.md) - All guides listed by task
- [Output in Different Formats](how-to/output-in-different-formats.md) - Format usage
- [Increase Verbosity](how-to/increase-verbosity.md) - Get more detail
- [Filter Results](how-to/filter-results.md) - Find specific data
- [Use with Shell Scripts](how-to/use-with-scripts.md) - Automation recipes

---

### 📖 [Technical Reference](reference/) - Information-Oriented

**Goal:** Provide complete and precise documentation for lookup

**When to use:** You need to know exactly what a command does or what options are available

**Reference:**
- [Command Reference](reference/COMMAND_REFERENCE.md) - All commands, options, and syntax
- [CLI Architecture](reference/CLI_ARCHITECTURE.md) - Module and command structure
- [Output Formats](reference/OUTPUT_FORMATS.md) - Format specifications

---

### 💡 [Explanation](explanation/) - Understanding-Oriented

**Goal:** Help you understand the "why" behind design decisions

**When to use:** You want to understand design philosophy, architecture, or tradeoffs

**Explanations:**
- [Architecture](explanation/ARCHITECTURE.md) - System design and philosophy
- [Design Decisions](explanation/DESIGN_DECISIONS.md) - Why we built it this way
- [Noun-Verb Pattern](explanation/NOUN_VERB_DESIGN.md) - CLI structure explained
- [Multi-Format Strategy](explanation/OUTPUT_FORMAT_DESIGN.md) - Format design rationale

---

## Navigation Guide

### By User Type

**🎓 I'm a student / new developer**
1. Start with [Getting Started Tutorial](tutorials/GETTING_STARTED.md)
2. Try [Output Formats Tutorial](tutorials/output-formats-intro.md)
3. Explore specific use cases in [How-To Guides](how-to/INDEX.md)

**💼 I'm a developer integrating playground into my project**
1. Skim [Command Reference](reference/COMMAND_REFERENCE.md)
2. Read relevant [How-To Guides](how-to/INDEX.md)
3. Check [Architecture](explanation/ARCHITECTURE.md) for design context

**🤖 I'm setting up automation / CI/CD**
1. Read [How-To: Use with Shell Scripts](how-to/use-with-scripts.md)
2. Reference [Command Reference](reference/COMMAND_REFERENCE.md)
3. Check exit codes and output formats in [Technical Reference](reference/)

**📚 I'm documenting or creating examples**
1. Check [How-To: Output in Different Formats](how-to/output-in-different-formats.md)
2. See example outputs in [Command Reference](reference/COMMAND_REFERENCE.md)
3. Understand philosophy in [Architecture](explanation/ARCHITECTURE.md)

**❓ I'm debugging or troubleshooting**
1. Look up command in [Command Reference](reference/COMMAND_REFERENCE.md)
2. Check [How-To: Increase Verbosity](how-to/increase-verbosity.md)
3. See [How-To Guides](how-to/INDEX.md) for problem-solving approaches

---

### By Task

| Task | Go To |
|------|-------|
| Get started with playground | [Getting Started Tutorial](tutorials/GETTING_STARTED.md) |
| Change output format | [Output in Different Formats How-To](how-to/output-in-different-formats.md) |
| Use playground in a script | [Use with Shell Scripts How-To](how-to/use-with-scripts.md) |
| Find specific features | [Filter Results How-To](how-to/filter-results.md) |
| Look up command syntax | [Command Reference](reference/COMMAND_REFERENCE.md) |
| Understand the design | [Architecture Explanation](explanation/ARCHITECTURE.md) |
| Learn about output formats | [Output Formats Tutorial](tutorials/output-formats-intro.md) |
| Troubleshoot issues | [How-To Index](how-to/INDEX.md) |

---

## Quick Reference

### Commands

```bash
# Show status
playground <NOUN> stat

# List examples
playground <NOUN> list

# Run examples
playground <NOUN> exec
```

### Nouns (Modules)
- `core` - Core testing features
- `testing` - Advanced testing techniques
- `validation` - Quality and constraint validation
- `observability` - Telemetry and observability
- `integration` - Integration testing support

### Format Options
```bash
--format json    # Machine-readable (default)
--format yaml    # Human-readable configuration
--format toml    # Rust configuration format
--format table   # Terminal table
--format tsv     # Spreadsheet format
```

### Verbosity Levels
```bash
# No flags     - Default output
-v            # Increase detail
-vv           # More detail
-vvv          # Maximum detail
```

---

## The Diátaxis Difference

Traditional documentation often mixes learning and reference content, making it hard to find what you need.

**Diátaxis separates these concerns:**

- **Tutorials** teach you concepts through guided practice
- **How-to guides** show you how to solve specific problems
- **Reference** gives you precise information for lookup
- **Explanation** helps you understand the "why"

Each type of content is:
- ✅ Organized for its specific purpose
- ✅ Written in its own style
- ✅ Optimized for its use case
- ✅ Linked to related content

---

## Contributing to Documentation

Want to improve the docs? Contributions are welcome!

- Fix typos: Edit the markdown file directly
- Add examples: Update the relevant how-to guide
- Add a new guide: Follow the structure of existing guides
- Improve explanations: Enhance the explanation section

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

## Feedback

Have questions or suggestions about the documentation?

- **Found an issue?** [Report it](https://github.com/seanchatmangpt/chicago-tdd-tools/issues)
- **Have an idea?** [Suggest it](https://github.com/seanchatmangpt/chicago-tdd-tools/discussions)
- **Want to contribute?** [Pull requests welcome](../../CONTRIBUTING.md)

---

## Documentation Map

```
docs/
├── README.md                          ← You are here
├── DIATAXIS_GUIDE.md                  ← Framework explanation
│
├── tutorials/
│   ├── GETTING_STARTED.md             ← Start here if new
│   └── output-formats-intro.md
│
├── how-to/
│   ├── INDEX.md                       ← Find by task here
│   ├── output-in-different-formats.md
│   ├── increase-verbosity.md
│   ├── filter-results.md
│   ├── use-with-scripts.md
│   ├── debug-test-failures.md
│   └── export-results.md
│
├── reference/
│   ├── COMMAND_REFERENCE.md           ← Look up commands here
│   ├── CLI_ARCHITECTURE.md
│   ├── OUTPUT_FORMATS.md
│   └── ERROR_MESSAGES.md
│
└── explanation/
    ├── ARCHITECTURE.md                ← Understand design here
    ├── DESIGN_DECISIONS.md
    ├── NOUN_VERB_DESIGN.md
    └── OUTPUT_FORMAT_DESIGN.md
```

---

## Quick Start (TL;DR)

```bash
# Build the playground
cd playground
cargo build --release

# Run your first command
./target/release/playground core stat

# Try different format
./target/release/playground core stat --format table

# Explore other modules
./target/release/playground testing stat
./target/release/playground validation list

# Get detailed output
./target/release/playground core stat -vvv

# Read the docs
open docs/tutorials/GETTING_STARTED.md
```

---

## More Information

- **Chicago TDD Framework**: [Main Repository](../../)
- **Framework Documentation**: [docs/](../../docs/)
- **API Reference**: [Reference](reference/)
- **Examples**: [examples/](../examples/)

---

**Documentation Framework:** Diátaxis
**Last Updated:** 2025-11-15
**Status:** Complete
