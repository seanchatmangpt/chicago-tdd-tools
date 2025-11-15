# How to Add New Examples to the Playground

**Step-by-step guide** for extending the playground with new examples.

## Overview

The playground uses a discoverable example system. Add examples by:
1. Creating a source file in `src/`
2. Implementing the example module
3. Registering in the CLI
4. Testing it works

## Step 1: Choose a Category

Examples live in feature group directories:

```
src/
├── core/          # Core features (fixtures, builders, assertions)
├── testing/       # Testing features (property, mutation, snapshot)
├── validation/    # Validation features (coverage, guards, JTBD)
├── observability/ # Observability features (OTEL, Weaver)
└── integration/   # Integration features (Docker, containers)
```

Pick the right category for your example.

## Step 2: Create the Example File

Create a new file in the appropriate category:

```bash
# For a core feature example
touch src/core/my_example.rs

# For a testing example
touch src/testing/my_example.rs
```

## Step 3: Write the Example

Follow the standard structure:

```rust
// src/core/my_example.rs

use chicago_tdd_tools::prelude::*;
use serde::{Deserialize, Serialize};

/// Demonstrates [feature name]
///
/// This example shows how to [brief description]
#[derive(Serialize, Deserialize)]
pub struct MyExampleResult {
    pub feature_count: usize,
    pub assertions_passed: usize,
    pub duration_ms: u128,
}

pub fn run() -> crate::Result<MyExampleResult> {
    let start = std::time::Instant::now();

    // Arrange: Set up test data
    let feature_count = 5;

    // Act: Execute the example
    let mut assertions = 0;

    // Example 1: Basic usage
    test!(test_basic, {
        assertions += 1;
        assert_eq!(1 + 1, 2);
    });

    // Example 2: With fixture
    fixture_test!(test_with_fixture, fixture, {
        assertions += 1;
        let data = fixture.test_counter();
        assert!(data >= 0);
    });

    // More examples as needed...

    let duration_ms = start.elapsed().as_millis();

    Ok(MyExampleResult {
        feature_count,
        assertions_passed: assertions,
        duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_runs() {
        let result = run().expect("Example should run");
        assert!(result.assertions_passed > 0);
    }
}
```

## Step 4: Register in Module

Add your example to the category's `mod.rs`:

```rust
// src/core/mod.rs

pub mod my_example;

pub use my_example::*;
```

## Step 5: Add to CLI

Update `src/cli/mod.rs` or the relevant category's CLI handler to discover your example:

```rust
// src/cli/core.rs

pub fn list_examples() -> Vec<(&'static str, &'static str)> {
    vec![
        ("fixtures", "Core test fixtures"),
        ("builders", "Fluent builders for test data"),
        ("assertions", "Assertion helpers"),
        ("my_example", "Demonstrates my_example feature"),  // Add this line
    ]
}

pub fn exec_example(name: &str) -> crate::Result<serde_json::Value> {
    let result = match name {
        "fixtures" => crate::core::fixtures::run()?,
        "builders" => crate::core::builders::run()?,
        "assertions" => crate::core::assertions::run()?,
        "my_example" => crate::core::my_example::run()?,  // Add this line
        _ => return Err(format!("Unknown example: {}", name).into()),
    };

    Ok(serde_json::to_value(result)?)
}
```

## Step 6: Test Locally

```bash
# Build the project
cargo build

# Run your example
cargo run -- core exec --names "my_example"

# Should see JSON output:
# {
#   "example": "my_example",
#   "status": "success",
#   ...
# }

# Run tests
cargo test
```

## Step 7: Test with CLI

List it:

```bash
cargo run -- core list
```

Should show your example.

Execute it:

```bash
cargo run -- core exec --names "my_example"
```

Should execute successfully.

## Example Template: Core Feature

For core features (always available):

```rust
// src/core/my_feature.rs

use chicago_tdd_tools::prelude::*;
use serde::{Deserialize, Serialize};

/// Shows how to use [feature]
#[derive(Serialize, Deserialize)]
pub struct MyFeatureResult {
    pub examples_demonstrated: usize,
    pub assertions_passed: usize,
}

pub fn run() -> crate::Result<MyFeatureResult> {
    let mut examples = 0;
    let mut assertions = 0;

    // Example 1: Basic usage
    {
        examples += 1;
        test!(test_1, {
            // Your code here
            assertions += 1;
        });
    }

    // Example 2: Intermediate usage
    {
        examples += 1;
        fixture_test!(test_2, fixture, {
            // Your code here
            assertions += 1;
        });
    }

    Ok(MyFeatureResult {
        examples_demonstrated: examples,
        assertions_passed: assertions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runs_successfully() {
        let result = run().expect("Should run");
        assert!(result.assertions_passed > 0);
    }
}
```

## Example Template: Feature-Gated Example

For optional features:

```rust
// src/testing/my_feature.rs

use chicago_tdd_tools::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MyFeatureResult {
    pub test_count: usize,
}

#[cfg(feature = "property-testing")]
pub fn run() -> crate::Result<MyFeatureResult> {
    // Implementation
    Ok(MyFeatureResult { test_count: 10 })
}

#[cfg(not(feature = "property-testing"))]
pub fn run() -> crate::Result<MyFeatureResult> {
    Err("Feature 'property-testing' required".into())
}
```

## Step 8: Update Documentation

1. Add to the appropriate how-to guide:

```markdown
## My Example

```bash
cargo run -- core exec --names "my_example"
```

Does [something useful].

**Use when:**
- You need to [solve problem]
- You want to [achieve goal]
```

2. Add to example inventory if comprehensive

3. Update README if major feature

## Best Practices

### Code Quality

- ✅ Follow AAA pattern (Arrange-Act-Assert)
- ✅ Add clear comments
- ✅ Use descriptive names
- ✅ Keep examples focused
- ✅ Handle errors with `?` operator
- ❌ No `.unwrap()` or `.expect()` unless documented
- ❌ No `panic!()` in examples

### Example Structure

```rust
pub fn run() -> crate::Result<MyResult> {
    // Arrange: Set up data

    // Act: Demonstrate feature

    // Assert: Verify behavior

    // Return structured result
    Ok(MyResult {
        // Metrics
    })
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_completes() {
        let result = run().expect("Should complete");
        // Verify results are meaningful
        assert!(result.assertions_passed > 0);
    }
}
```

### Documentation

```rust
/// Brief description of what example demonstrates
///
/// Longer explanation if needed.
///
/// # Example
///
/// ```no_run
/// let result = run()?;
/// assert!(result.assertions_passed > 0);
/// ```
pub fn run() -> crate::Result<MyResult> {
    // ...
}
```

## Checklist

When adding an example:

- ✅ Created file in correct directory
- ✅ Implemented `run()` function
- ✅ Returns `crate::Result<T: Serialize>`
- ✅ Registered in `mod.rs`
- ✅ Added to CLI handler
- ✅ Added unit tests
- ✅ Runs successfully: `cargo run -- category exec --names "my_example"`
- ✅ Listed: `cargo run -- category list`
- ✅ All tests pass: `cargo test`
- ✅ Documentation updated

## Troubleshooting

**Q: "Unknown example"**
A: Verify CLI registration in `src/cli/`.

**Q: "Feature is required"**
A: Use feature gates with `#[cfg(feature = "...")]`.

**Q: "Example doesn't appear in list"**
A: Check `list_examples()` in CLI handler.

**Q: "Tests fail"**
A: Ensure `run()` handles errors properly.

## Example: Complete Custom Example

Here's a complete example you can copy:

```rust
// src/core/my_complete_example.rs

use chicago_tdd_tools::prelude::*;
use serde::{Deserialize, Serialize};

/// Demonstrates combining fixtures, builders, and assertions
#[derive(Serialize, Deserialize)]
pub struct CompleteExampleResult {
    pub fixtures_used: usize,
    pub builders_used: usize,
    pub assertions_made: usize,
    pub duration_ms: u128,
}

pub fn run() -> crate::Result<CompleteExampleResult> {
    let start = std::time::Instant::now();

    let mut fixtures_used = 0;
    let mut builders_used = 0;
    let mut assertions_made = 0;

    // Example 1: Fixture with builder
    fixture_test!(test_combined, fixture, {
        fixtures_used += 1;

        // Builder pattern
        builders_used += 1;
        let counter = 0;

        // Act
        let result = counter + 1;

        // Assertions
        assertions_made += 1;
        assert_eq!(result, 1);
    });

    Ok(CompleteExampleResult {
        fixtures_used,
        builders_used,
        assertions_made,
        duration_ms: start.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_demonstrates_multiple_patterns() {
        let result = run().expect("Example should complete");
        assert!(result.assertions_made > 0);
        assert!(result.fixtures_used > 0);
        assert!(result.builders_used > 0);
    }
}
```

## Next Steps

- **Update documentation** → Update relevant how-to guides
- **Add tests** → Comprehensive test coverage
- **Get feedback** → Review with team
- **Iterate** → Improve based on feedback

---

Extend the playground by following these patterns.
