# Documentation Style Guide

**Purpose**: Establish consistent documentation standards to eliminate Mura (unevenness) and improve user experience.

## Core Principles

1. **User-First**: Write for users, not for developers
2. **Actionable**: Every guide should enable users to accomplish tasks
3. **Consistent**: Same patterns, terminology, and structure across all docs
4. **Complete**: Cover all relevant use cases without redundancy
5. **Current**: Keep synchronized with code changes

## Documentation Structure

### Heading Hierarchy

**Maximum Depth**: 4 levels (`####`)

```
# Title (H1) - Document title
## Section (H2) - Major sections
### Subsection (H3) - Subsections
#### Detail (H4) - Detailed topics
```

**Guidelines**:
- Use H1 only for document title
- H2 for major sections (Getting Started, Patterns, API Reference)
- H3 for subsections (Basic Usage, Advanced Patterns)
- H4 for detailed topics (specific examples, edge cases)
- Avoid H5 and deeper - flatten structure instead

### Document Types

#### Getting Started Guides
- **Purpose**: Help users get started quickly
- **Structure**: Installation → First Test → Common Patterns → Next Steps
- **Style**: Tutorial-style, step-by-step, full examples
- **Examples**: `GETTING_STARTED.md`, `QUICK_GUIDE.md`

#### Reference Documentation
- **Purpose**: Complete API and technical reference
- **Structure**: Organized by module/feature → API details → Examples
- **Style**: Technical, detailed, signature-focused
- **Examples**: `API_REFERENCE.md`, `ARCHITECTURE.md`

#### Feature Guides
- **Purpose**: Deep dive into specific features
- **Structure**: Overview → Quick Start → Patterns → Best Practices → Troubleshooting
- **Style**: Comprehensive, pattern-focused, real-world examples
- **Examples**: `WEAVER_LIVE_CHECK.md`, `cli-testing-guide.md`

## Code Examples

### Format Standards

#### Getting Started Guides
**Format**: Full examples with imports and context

```rust
use chicago_tdd_tools::prelude::*;

test!(my_test, {
    // Arrange
    let value = 42;
    
    // Act
    let result = value * 2;
    
    // Assert
    assert_eq!(result, 84);
});
```

#### Quick Guides
**Format**: Minimal examples, assume imports via prelude

```rust
test!(my_test, {
    let value = 42;
    assert_eq!(value * 2, 84);
});
```

#### API Reference
**Format**: Signature + minimal example

```rust
// Signature
pub fn assert_eq<T: PartialEq + Debug>(left: T, right: T) -> Result<()>

// Example
assert_eq!(42, 42)?;
```

### Code Example Guidelines

1. **Always Runnable**: Examples should compile and run
2. **Feature Flags**: Include `#[cfg(feature = "...")]` when needed
3. **Error Handling**: Show `Result` handling in examples
4. **Comments**: Use AAA pattern comments (Arrange-Act-Assert) in getting started guides
5. **Imports**: Show imports in getting started, assume prelude in quick guides

## Terminology

### Standard Terms

| Concept | Standard Term | Variants to Avoid |
|---------|--------------|-------------------|
| Test macro | `test!` macro | test macro, test!(), test macro function |
| Assertion | `assert_eq!` | assert, assertion macro, assert_eq |
| Feature flag | `feature` | feature flag, cfg feature, feature gate |
| Test fixture | `TestFixture` | fixture, test fixture, fixture object |
| Async test | `async_test!` | async test, async test macro |

### Usage Guidelines

1. **First Mention**: Use full term with backticks: `` `test!` macro ``
2. **Subsequent Mentions**: Can use shorter form: `test!`
3. **Code References**: Always use backticks: `` `assert_eq!` ``
4. **File Names**: Use backticks: `` `Cargo.toml` ``

## Version References

### Current Version
- **Current**: 1.1.2 (published to crates.io)
- **Always use current version** in examples and installation instructions
- **Exception**: Future feature docs can reference planned versions with clear status

### Version Format
```toml
chicago-tdd-tools = { version = "1.1.2", features = ["feature-name"] }
```

### Version-Specific Docs
- **Naming**: `v1.2.0-feature-name.md` for version-specific docs
- **Status**: Mark as "(Planned)" if describing future features
- **Current Version**: Always mention current version in planned docs

## Build Commands

### Standard: Always Use `cargo make`

**Correct**:
```bash
cargo make test
cargo make check
cargo make test-integration
```

**Incorrect**:
```bash
cargo test
cargo check
cargo build
```

### Rationale
- `cargo make` handles proc-macro crates correctly
- Includes timeouts to prevent hanging
- Ensures consistency across environments
- Enforces quality gates

### Exceptions
- When documenting cargo-make itself
- When showing fallback commands (with explanation)

## Cross-References

### Required Sections

All major documentation should include:

1. **Quick Navigation** (at top): Links to related getting started guides
2. **See Also** (at bottom): Links to related documentation, external resources

### Link Format

**Internal Links**:
```markdown
[Link Text](relative/path/to/file.md)
[Link Text with Anchor](relative/path/to/file.md#anchor)
```

**External Links**:
```markdown
[Link Text](https://example.com)
```

### Cross-Reference Guidelines

1. **Getting Started Guides**: Link to API Reference, User Guide, Architecture
2. **Feature Guides**: Link to related features, API Reference, examples
3. **API Reference**: Link to getting started, user guide, architecture
4. **Always Include**: Next steps section with relevant links

## Writing Style

### Tone
- **Friendly but Professional**: Approachable, not casual
- **Direct**: Get to the point quickly
- **Confident**: Use declarative statements, not "you might want to"

### Structure
- **Lead with Value**: What problem does this solve?
- **Show, Don't Tell**: Code examples over explanations
- **Progressive Disclosure**: Basic → Advanced → Edge Cases

### Language
- **Active Voice**: "Create a test" not "A test is created"
- **Imperative Mood**: "Run the test" not "You should run the test"
- **Present Tense**: "The macro expands to..." not "The macro will expand to..."

## File Naming

### Conventions

1. **General Docs**: `FEATURE_NAME.md` (e.g., `WEAVER_LIVE_CHECK.md`)
2. **Version-Specific**: `vVERSION-feature-name.md` (e.g., `v1.2.0-coverage-strategy.md`)
3. **Process Docs**: `PROCESS_NAME.md` (e.g., `CODING_STANDARDS.md`)
4. **Reference Docs**: `REFERENCE_TYPE.md` (e.g., `API_REFERENCE.md`)

### Guidelines
- Use UPPERCASE for major documents
- Use lowercase-with-hyphens for feature guides
- Include version in filename only for version-specific content
- Be descriptive but concise

## Checklist for New Documentation

- [ ] Heading hierarchy limited to 4 levels
- [ ] Code examples match format for document type
- [ ] Version numbers are current (1.1.2)
- [ ] Build commands use `cargo make`
- [ ] Cross-references included (Quick Navigation + See Also)
- [ ] Terminology is consistent
- [ ] Examples are runnable
- [ ] Feature flags included where needed
- [ ] Error handling shown in examples
- [ ] Next steps section included

## Maintenance

### When to Update
- After code changes that affect APIs
- When adding new features
- When fixing bugs that affect usage
- During release preparation

### Review Process
1. Check against this style guide
2. Verify all examples compile
3. Test all links
4. Verify version numbers
5. Check build commands

## Examples

### Good Example (Getting Started)

```markdown
## Your First Test

Create a test file `tests/my_test.rs`:

```rust
use chicago_tdd_tools::prelude::*;

test!(my_first_test, {
    // Arrange
    let value = 42;
    
    // Act
    let result = value * 2;
    
    // Assert
    assert_eq!(result, 84);
});
```

Run the test:

```bash
cargo make test
```

**See Also**:
- [Quick Guide](QUICK_GUIDE.md) - More patterns
- [API Reference](../reference/API_REFERENCE.md) - Complete API
```

### Bad Example (Issues)

```markdown
## Your First Test

You might want to create a test. Here's how:

```rust
// Missing imports
test!(my_first_test, {
    assert_eq!(42 * 2, 84);  // No AAA pattern
});
```

Run with `cargo test`.  // Wrong command
```

## Summary

**Key Principles**: User-first, actionable, consistent, complete, current

**Structure**: 4-level headings max, document-type-appropriate format

**Code Examples**: Full in getting started, minimal in quick guides, signature-focused in API reference

**Terminology**: Consistent terms, backticks for code, current version (1.1.2)

**Commands**: Always `cargo make`, never direct `cargo` commands

**Cross-References**: Quick Navigation + See Also sections required

**Maintenance**: Update with code changes, verify examples compile, test links

