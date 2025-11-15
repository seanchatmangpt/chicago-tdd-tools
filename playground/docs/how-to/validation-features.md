# How to Run Validation Feature Examples

**Quick reference** for validation features: coverage analysis, guard constraints, JTBD validation, performance testing.

## Quick Commands

```bash
# Show all validation features
cargo run -- valid stat

# List available validation examples
cargo run -- valid list

# Run specific examples
cargo run -- valid exec --names "cov"
cargo run -- valid exec --names "cov guard jtbd perf"

# Run all validation examples
cargo run -- valid exec --names "cov guard jtbd perf"
```

## Validation Features Overview

| Feature | Purpose | When to Use |
|---------|---------|------------|
| **Coverage** | Test coverage analysis | Track tested code percentage |
| **Guards** | Constraint enforcement | Verify compile-time limits |
| **JTBD** | Jobs To Be Done validation | Validate feature completeness |
| **Performance** | Tick budget validation | Ensure performance constraints |

## Coverage Analysis

```bash
cargo run -- valid exec --names "cov"
```

**What it does:**
Shows which code is tested and identifies gaps.

**Use when:**
- Want to track test coverage percentage
- Identify untested code paths
- Ensure critical code is tested

**Example coverage report:**
```json
{
  "lines_covered": 245,
  "lines_total": 300,
  "coverage_percent": 81.7,
  "uncovered_lines": 55
}
```

**Target:** ≥ 80% coverage is excellent.

**Command:**
```bash
cargo run -- valid exec --names "cov"
```

## Guard Constraints

```bash
cargo run -- valid exec --names "guard"
```

**What it does:**
Enforces compile-time constraints (e.g., MAX_RUN_LENGTH ≤ 8).

**Use when:**
- Need to enforce maximum batch sizes
- Limit recursion depth
- Validate compile-time invariants
- Enforce resource limits

**Key constraints:**
- MAX_RUN_LEN: Maximum execution length
- MAX_BATCH_SIZE: Maximum batch size
- Custom: Define your own

**Example:**
```rust
// Compiler verifies this at compile time
const_assert!(MAX_RUN_LEN <= 8);
const_assert!(MAX_BATCH_SIZE <= 1000);
```

**Command:**
```bash
cargo run -- valid exec --names "guard"
```

## JTBD (Jobs To Be Done) Validation

```bash
cargo run -- valid exec --names "jtbd"
```

**What it does:**
Validates that features complete their intended jobs.

**Use when:**
- Ensuring features fulfill user needs
- Validating feature completeness
- Preventing incomplete features

**JTBD Framework:**
Jobs → Tasks → Features

**Example:**
```
Job: "Be able to handle user authentication"
├── Task: "Accept username/password"
├── Task: "Validate credentials"
└── Task: "Issue session token"
```

**Command:**
```bash
cargo run -- valid exec --names "jtbd"
```

## Performance Testing

```bash
cargo run -- valid exec --names "perf"
```

**What it does:**
Validates operations complete within tick budget.

**Use when:**
- Need performance guarantees
- Testing latency-sensitive code
- Measuring CPU cycles
- Enforcing SLAs

**Example:**
```rust
performance_test!(test_performance, {
    let work = || {
        let mut sum = 0;
        for i in 0..100 {
            sum += i;
        }
        sum
    };

    let result = work();
    assert_within_tick_budget!(result > 0);  // Default: ≤8 ticks
});
```

**Tick budget defaults:**
- Default: ≤ 8 ticks
- Custom: Set as needed

**Command:**
```bash
cargo run -- valid exec --names "perf"
```

## Running Multiple Validation Features

### Essential Validation Suite

```bash
cargo run -- valid exec --names "cov guard"
```

Coverage + constraints validation.

### Complete Validation

```bash
cargo run -- valid exec --names "cov guard jtbd perf"
```

All validation features together.

## Validation Selection Guide

**Track code quality?**
```bash
cargo run -- valid exec --names "cov"
```

**Enforce constraints?**
```bash
cargo run -- valid exec --names "guard"
```

**Validate features?**
```bash
cargo run -- valid exec --names "jtbd"
```

**Check performance?**
```bash
cargo run -- valid exec --names "perf"
```

**Complete validation?**
```bash
cargo run -- valid exec --names "cov guard jtbd perf"
```

## Integration with Testing

Validation works alongside testing:

```
Testing        Validation
┌──────┐       ┌──────────────┐
│ Unit │ ───→  │ Coverage     │
│ Tests│       │ Guards       │
└──────┘       │ JTBD         │
               │ Performance  │
               └──────────────┘
```

**Workflow:**
1. Write tests (core + testing features)
2. Validate with coverage
3. Check constraints with guards
4. Validate completeness with JTBD
5. Verify performance with tick budgets

## Best Practices

1. **Track coverage** - Aim for 80%+
2. **Enforce constraints** - Use guards proactively
3. **Validate features** - Use JTBD framework
4. **Monitor performance** - Set tick budgets
5. **Combine all** - Build complete validation pipeline

## Quality Gates

Recommended validation thresholds:

| Metric | Target | Status |
|--------|--------|--------|
| Coverage | ≥ 80% | Pass/Fail |
| Guard constraints | 100% | Pass/Fail |
| JTBD completion | 100% | Pass/Fail |
| Performance | Within budget | Pass/Fail |

## Troubleshooting

**Q: "Coverage is too low"**
A: Add more tests for uncovered lines:
```bash
cargo run -- valid exec --names "cov"  # See which lines uncovered
```

**Q: "Guard constraint violated"**
A: The compiler caught an issue - fix at compile time (that's the point!).

**Q: "JTBD incomplete"**
A: Complete the feature or update JTBD specification.

**Q: "Performance exceeds budget"**
A: Optimize code or increase tick budget:
```rust
#[performance_test(tick_budget = 16)]  // Increase budget
fn test_expensive_operation() { ... }
```

## Next Steps

- **Copy to your project** → [Copying Examples](../tutorials/copying-examples.md)
- **Testing features** → [Testing Features Guide](testing-features.md)
- **Observability** → [Observability Features](observability-features.md)
- **See all examples** → [Example Inventory](../reference/example-inventory.md)

---

Build validated, high-quality code with comprehensive validation.
