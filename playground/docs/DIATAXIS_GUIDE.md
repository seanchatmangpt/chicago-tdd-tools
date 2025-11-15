# Playground Documentation: Diátaxis Framework

This documentation follows the **Diátaxis** systematic approach to technical documentation. All content is organized into four distinct pillars based on user needs and learning objectives.

---

## The Four Pillars

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

### 1. **Tutorials** 📚 (Learning-Oriented)

**Purpose:** Help users learn the basics and gain hands-on experience

**User Need:** "I'm new to this, teach me the fundamentals"

**Characteristics:**
- Step-by-step instructions
- Real, concrete examples
- Learning by doing
- Assumes minimal prior knowledge
- Builds confidence and competence

**When to use:** Getting started with playground CLI for the first time

**Location:** `docs/tutorials/`

---

### 2. **How-To Guides** 🔧 (Goal-Oriented)

**Purpose:** Help users accomplish specific tasks and solve problems

**User Need:** "I want to do X, show me how"

**Characteristics:**
- Problem-solution focused
- Practical and direct
- Assumes user has basic knowledge
- Action-oriented steps
- Address common use cases

**When to use:** Solving specific problems (e.g., "How to output in YAML format?")

**Location:** `docs/how-to/`

---

### 3. **Explanation** 💡 (Understanding-Oriented)

**Purpose:** Help users understand concepts, design decisions, and architecture

**User Need:** "Why is it this way? How does it work?"

**Characteristics:**
- Discussion and analysis
- Contextual information
- Design rationale
- Explores alternatives
- Educational and reflective

**When to use:** Understanding the philosophy and design of the playground

**Location:** `docs/explanation/`

---

### 4. **Technical Reference** 📖 (Information-Oriented)

**Purpose:** Provide complete, accurate API documentation and architecture details

**User Need:** "I need to know exactly what this does"

**Characteristics:**
- Comprehensive and precise
- Organized for lookup
- Complete parameter documentation
- Examples for most items
- Structured consistently

**When to use:** Looking up specific command syntax or API details

**Location:** `docs/reference/`

---

## Navigation Map

```
START HERE
    ↓
[New to CLI?] → tutorials/GETTING_STARTED.md
    ↓
[Want to do something?] → how-to/INDEX.md
    ↓
[Need exact details?] → reference/COMMAND_REFERENCE.md
    ↓
[Want to understand why?] → explanation/ARCHITECTURE.md
```

---

## Content Organization

### Tutorials Directory
- `GETTING_STARTED.md` - First steps with playground CLI
- `first-command.md` - Running your first command
- `understanding-modules.md` - Overview of playground modules
- `output-formats-intro.md` - Introduction to output formatting

### How-To Guides Directory
- `INDEX.md` - Index of all how-to guides
- `output-in-different-formats.md` - Using JSON, YAML, TOML, Table, TSV
- `increase-verbosity.md` - Getting more detailed output
- `filter-results.md` - Filtering and searching results
- `use-with-scripts.md` - Integrating playground with shell scripts
- `debug-test-failures.md` - Using playground for test debugging

### Technical Reference Directory
- `COMMAND_REFERENCE.md` - Complete command syntax
- `CLI_ARCHITECTURE.md` - Module and command structure
- `OUTPUT_FORMATS.md` - Format specifications
- `ERROR_MESSAGES.md` - Error reference guide

### Explanation Directory
- `ARCHITECTURE.md` - System design and philosophy
- `DESIGN_DECISIONS.md` - Why we built it this way
- `MODULE_STRATEGY.md` - Module organization strategy
- `CLAP_NOUN_VERB_INTEGRATION.md` - How clap-noun-verb works with playground
- `FORMATTING_STRATEGY.md` - Multi-format output design

---

## Writing Guidelines

### For Tutorials
- Write in second person: "You will..."
- Use concrete examples and real scenarios
- Include expected output
- One concept per tutorial
- End with "Next steps"

### For How-To Guides
- Start with problem statement
- Provide multiple approaches when relevant
- Show before/after examples
- Include troubleshooting section
- Link to related guides

### For Technical Reference
- Use consistent structure for all items
- Include examples for every command
- Document all parameters
- Note any limitations
- Provide cross-references

### For Explanation
- Use questions as structure
- Discuss trade-offs
- Compare with alternatives
- Explain design rationale
- Include diagrams where helpful

---

## Key Principles

1. **Separation of Concerns** - Each pillar serves a distinct purpose
2. **User-Centered** - Content organized by user needs, not system structure
3. **Progressive Disclosure** - Tutorials → How-to → Reference → Explanation
4. **Completeness** - All four pillars needed for comprehensive documentation
5. **Discoverability** - Clear navigation between related content

---

## Implementation Status

- [ ] **Tutorials** - Getting started content
- [ ] **How-To Guides** - Common tasks and problems
- [ ] **Technical Reference** - Complete API documentation
- [ ] **Explanation** - Design and architecture
- [ ] **Navigation** - Cross-linking and index pages
- [ ] **README Integration** - Link to Diátaxis docs from main README

---

## References

- **Diátaxis Official:** https://diataxis.fr/
- **Inspired by:** Linux kernel docs, Django documentation, Kubernetes docs

---

**Framework Version:** 1.0.0
**Last Updated:** 2025-11-15
**Status:** Framework Established
