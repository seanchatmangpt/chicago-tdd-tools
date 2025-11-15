# Playground Documentation

Welcome to the Chicago TDD Tools Playground documentation. This guide is organized using the **Diataxis framework**, which structures knowledge into four modes:

## 🎓 Learning Path

**Start here if you're new to the playground.**

1. **[Getting Started Tutorial](tutorials/getting-started.md)** - Install and run your first example (5 min)
2. **[Understanding the Playground](explanation/playground-philosophy.md)** - Learn why the playground exists (5 min)
3. **[Noun-Verb Pattern Guide](explanation/noun-verb-pattern.md)** - Understand the CLI structure (5 min)

## 📚 Documentation Sections

### 🎯 Tutorials (Learning-Oriented)

Step-by-step guides to get you up and running. Perfect for beginners.

- **[Getting Started](tutorials/getting-started.md)** - Installation, first command, next steps
- **[Running Core Examples](tutorials/running-core-examples.md)** - Learn fixtures, builders, assertions
- **[Running Feature Examples](tutorials/running-feature-examples.md)** - Explore advanced features
- **[Copying Examples to Your Project](tutorials/copying-examples.md)** - Adapt playground examples for real work

### 🛠️ How-To Guides (Problem-Oriented)

Practical guides to solve specific problems. Choose based on what you want to accomplish.

- **[How to Run Core Feature Examples](how-to/core-features.md)** - Fixtures, builders, assertions, macros
- **[How to Run Testing Examples](how-to/testing-features.md)** - Property, mutation, snapshot, concurrency
- **[How to Run Validation Examples](how-to/validation-features.md)** - Coverage, guards, JTBD, performance
- **[How to Run Observability Examples](how-to/observability-features.md)** - OTEL, Weaver integration
- **[How to Run Integration Examples](how-to/integration-features.md)** - Docker, testcontainers
- **[How to Add New Examples](how-to/adding-examples.md)** - Extend the playground
- **[How to Use Playground with Feature Flags](how-to/feature-flags.md)** - Enable/disable features
- **[How to Generate JSON Output for Automation](how-to/json-output.md)** - Use playground in scripts

### 📖 Reference (Information-Oriented)

Complete technical reference for the playground. Look up details as needed.

- **[CLI Command Reference](reference/cli-commands.md)** - All commands and options
- **[Feature Matrix](reference/feature-matrix.md)** - Which examples use which features
- **[Directory Structure](reference/directory-structure.md)** - File organization and module layout
- **[Example Inventory](reference/example-inventory.md)** - Complete list of all examples
- **[Configuration Options](reference/configuration.md)** - Environment variables, settings
- **[JSON Output Schema](reference/json-schema.md)** - Output format for automation

### 💡 Explanation (Understanding-Oriented)

Deep dives into concepts and design decisions. Read to understand the "why."

- **[Playground Philosophy](explanation/playground-philosophy.md)** - Why the playground exists, design goals
- **[Noun-Verb Pattern](explanation/noun-verb-pattern.md)** - Command structure and design
- **[Feature Organization](explanation/feature-organization.md)** - How features are grouped
- **[AAA Pattern in Examples](explanation/aaa-pattern.md)** - Arrange-Act-Assert structure
- **[Example Lifecycle](explanation/example-lifecycle.md)** - How examples are discovered and executed
- **[Testing Philosophy](explanation/testing-philosophy.md)** - Chicago TDD principles in the playground

## 🔍 Quick Navigation by Use Case

### "I want to..."

- **See the playground in action** → [Getting Started Tutorial](tutorials/getting-started.md)
- **Run core examples** → [Running Core Examples Tutorial](tutorials/running-core-examples.md) or [How to Run Core Features](how-to/core-features.md)
- **Understand the CLI** → [Noun-Verb Pattern](explanation/noun-verb-pattern.md)
- **Copy an example to my project** → [Copying Examples Tutorial](tutorials/copying-examples.md)
- **Find all CLI commands** → [CLI Command Reference](reference/cli-commands.md)
- **Understand feature organization** → [Feature Organization](explanation/feature-organization.md)
- **Add new examples** → [How to Add Examples](how-to/adding-examples.md)
- **Automate with JSON output** → [How to Generate JSON Output](how-to/json-output.md)
- **See all available examples** → [Example Inventory](reference/example-inventory.md)
- **Check which features an example uses** → [Feature Matrix](reference/feature-matrix.md)

## 📊 Documentation Framework

This documentation follows the **[Diataxis framework](https://diataxis.fr/)**, which organizes documentation into four key modes based on user intent:

| Mode | Purpose | Use When | Example |
|------|---------|----------|---------|
| **Tutorial** | Learning-oriented | You're new and want to get started | "Getting Started" |
| **How-To** | Problem-oriented | You know what you want and need steps | "How to Run Core Examples" |
| **Reference** | Information-oriented | You need to look up facts | "CLI Command Reference" |
| **Explanation** | Understanding-oriented | You want to understand concepts | "Noun-Verb Pattern Design" |

## 🚀 Common Starting Points

### For Beginners
1. [Getting Started Tutorial](tutorials/getting-started.md)
2. [Running Core Examples Tutorial](tutorials/running-core-examples.md)
3. [Noun-Verb Pattern Explanation](explanation/noun-verb-pattern.md)

### For Experienced Rust Developers
1. [Playground Philosophy](explanation/playground-philosophy.md)
2. [Feature Organization](explanation/feature-organization.md)
3. [Example Inventory](reference/example-inventory.md)
4. [How to Add Examples](how-to/adding-examples.md)

### For Integration/Automation
1. [How to Generate JSON Output](how-to/json-output.md)
2. [CLI Command Reference](reference/cli-commands.md)
3. [JSON Output Schema](reference/json-schema.md)

### For Learning Chicago TDD
1. [Playground Philosophy](explanation/playground-philosophy.md)
2. [Testing Philosophy](explanation/testing-philosophy.md)
3. [AAA Pattern in Examples](explanation/aaa-pattern.md)
4. [Running Testing Examples Tutorial](tutorials/running-feature-examples.md)

## ❓ FAQ

**Q: What's the difference between tutorials and how-to guides?**
A: Tutorials teach you from scratch. How-to guides assume you understand basics and solve specific problems.

**Q: Where do I find all available commands?**
A: See [CLI Command Reference](reference/cli-commands.md).

**Q: Can I copy examples from the playground?**
A: Yes! See [Copying Examples to Your Project](tutorials/copying-examples.md).

**Q: How do I add my own examples?**
A: See [How to Add Examples](how-to/adding-examples.md).

**Q: What does "noun-verb" mean?**
A: See [Noun-Verb Pattern](explanation/noun-verb-pattern.md).

## 🔗 Related Documentation

- **Main Framework**: [Chicago TDD Tools Documentation](../../docs/README.md)
- **Framework Examples**: [Framework Examples](../../examples/)
- **Cookbook**: [Alexander-Style Patterns](../../cookbook/README.md)
- **Project Charter**: [Playground Charter](../PROJECT_CHARTER.md)

## 📝 Contributing to Documentation

See the main project's [Documentation Style Guide](../../docs/process/DOCUMENTATION_STYLE_GUIDE.md) for how to contribute documentation improvements.

---

**Last Updated**: 2025-11-15 | **Framework**: Diataxis
