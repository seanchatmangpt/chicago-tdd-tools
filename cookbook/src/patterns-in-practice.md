# Patterns in Practice

> 📘 Integration Guide

This guide bridges the gap between the theoretical patterns in this cookbook and the practical tutorials in the **Application Guide**. 

## Pattern → Feature Lookup

When you are building a specific feature, which patterns should you apply?

| Feature / Task | Relevant Patterns | Application Guide Tutorial |
|----------------|-------------------|----------------------------|
| **Building a CLI App** | [Pattern 6: Generic Base](architecture-patterns/generic-base.md), [Pattern 11: Zero-Cost Abstractions](design-patterns/zero-cost-abstractions.md) | [CLI Application Tutorial](../../application-guide/src/tutorials/cli-app-tutorial.md) |
| **Building a Web Service** | [Pattern 9: Single Source of Truth](architecture-patterns/single-source-of-truth.md), [Pattern 15: Type State Enforcement](design-patterns/type-state-pattern.md) | [REST Web Service Tutorial](../../application-guide/src/tutorials/web-service-tutorial.md) |
| **Handling Complex Test Data** | [Pattern 17: Builder-Driven Test Data](design-patterns/builder-test-data.md) | [Fixtures Deep Dive](../../application-guide/src/tutorials/fixtures-tutorial.md) |
| **Structuring Tests** | [Pattern 1: AAA Pattern](testing-patterns/aaa-pattern.md), [Pattern 4: Resource Cleanup](testing-patterns/resource-cleanup.md) | [Getting Started](../../application-guide/src/tutorials/getting-started.md) |

## Cross-References

### Application Guide Tutorials to Patterns

- **Getting Started**: Focuses on the basics. Applies [Pattern 1: AAA Pattern](testing-patterns/aaa-pattern.md) and [Pattern 2: Error Path Testing](testing-patterns/error-path-testing.md).
- **Fixtures Deep Dive**: Teaches resource management. Heavily relies on [Pattern 4: Resource Cleanup](testing-patterns/resource-cleanup.md) and [Pattern 16: Fixture Lifecycle Management](design-patterns/fixture-lifecycle.md).
- **CLI Application**: Explores architecture. Implements [Pattern 6: Generic Base Layer](architecture-patterns/generic-base.md) and [Pattern 8: Composition Over Duplication](architecture-patterns/composition-over-duplication.md).
- **REST Web Service**: Handles state and validation. Uses [Pattern 14: Compile-Time Validation](design-patterns/compile-time-validation.md) and [Pattern 15: Type State Enforcement](design-patterns/type-state-pattern.md).

For more detailed guides on implementing these features, visit the [Application Guide](../../application-guide/src/README.md).
