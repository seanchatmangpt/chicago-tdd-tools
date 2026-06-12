//! Chapter 3: Knowledge Hooks and YAWL Patterns
//!
//! Tests for Knowledge Hooks and YAWL workflow control patterns:
//! - Theorem 3.1: Knowledge Hook Atomicity
//! - Theorem 3.2: YAWL Pattern 1: Sequence
//! - Theorem 3.3: YAWL Pattern 2: Parallel Split
//! - Theorem 3.4: YAWL Pattern 3: Synchronization
//! - Theorem 3.5: YAWL Pattern 4: Exclusive Choice
//! - Theorem 3.6: YAWL Pattern 5: Simple Merge

use crate::{TestResultType, TheoremMetadata};
use chicago_tdd_tools::operator_registry::global_registry;

/// Get the complete list of theorems for Chapter 3
pub fn theorems() -> Vec<TheoremMetadata> {
    vec![
        TheoremMetadata {
            id: "Thm-3.1".to_string(),
            name: "Knowledge Hook Atomicity".to_string(),
            latex_lines: (401, 450),
            test_path: "chapter03::test_knowledge_hook_atomicity".to_string(),
            expected_result: TestResultType::Pass,
        },
        TheoremMetadata {
            id: "Thm-3.2".to_string(),
            name: "YAWL Pattern 1: Sequence Determinism".to_string(),
            latex_lines: (451, 500),
            test_path: "chapter03::test_yawl_pattern_1_sequence".to_string(),
            expected_result: TestResultType::Pass,
        },
        TheoremMetadata {
            id: "Thm-3.3".to_string(),
            name: "YAWL Pattern 2: Parallel Split Preservation".to_string(),
            latex_lines: (501, 550),
            test_path: "chapter03::test_yawl_pattern_2_parallel_split".to_string(),
            expected_result: TestResultType::Pass,
        },
        TheoremMetadata {
            id: "Thm-3.4".to_string(),
            name: "YAWL Pattern 3: Synchronization Boundedness".to_string(),
            latex_lines: (551, 600),
            test_path: "chapter03::test_yawl_pattern_3_synchronization".to_string(),
            expected_result: TestResultType::Pass,
        },
        TheoremMetadata {
            id: "Thm-3.5".to_string(),
            name: "YAWL Pattern 4: Exclusive Choice Legality".to_string(),
            latex_lines: (601, 650),
            test_path: "chapter03::test_yawl_pattern_4_exclusive_choice".to_string(),
            expected_result: TestResultType::Pass,
        },
        TheoremMetadata {
            id: "Thm-3.6".to_string(),
            name: "YAWL Pattern 5: Simple Merge Non-Determinism".to_string(),
            latex_lines: (651, 700),
            test_path: "chapter03::test_yawl_pattern_5_simple_merge".to_string(),
            expected_result: TestResultType::Pass,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::operator_registry::GuardType;
    use proptest::prelude::*;

    /// Theorem 3.1: Knowledge Hook Atomicity
    ///
    /// Every knowledge hook must be atomic and identifiable.
    #[test]
    fn test_knowledge_hook_atomicity() {
        let registry = global_registry();
        let ops = registry.all_operators();

        assert!(!ops.is_empty(), "Operator registry should not be empty");

        for op in ops {
            assert!(!op.hook_id.is_empty(), "Hook ID must not be empty");
            assert!(op.pattern_number > 0, "Pattern number must be positive");
        }
    }

    /// Theorem 3.2: YAWL Pattern 1: Sequence
    ///
    /// Sequence operator is deterministic and bounded.
    #[test]
    fn test_yawl_pattern_1_sequence() {
        let registry = global_registry();
        let op = registry.get_operator("sequence_op").expect("Sequence operator not found");

        assert!(op.properties.deterministic, "Sequence must be deterministic");
        assert!(op.properties.bounded, "Sequence must be bounded");
        assert!(op.requires_guard(GuardType::Chronology), "Sequence requires Chronology guard");
    }

    /// Theorem 3.3: YAWL Pattern 2: Parallel Split
    ///
    /// Parallel Split is deterministic and preserves types.
    #[test]
    fn test_yawl_pattern_2_parallel_split() {
        let registry = global_registry();
        let op = registry.get_operator("parallel_split_op").expect("Parallel Split not found");

        assert!(op.properties.deterministic, "Parallel Split must be deterministic");
        assert!(op.properties.type_preserving, "Parallel Split must be type preserving");
        assert!(op.requires_guard(GuardType::Legality), "Parallel Split requires Legality guard");
    }

    /// Theorem 3.4: YAWL Pattern 3: Synchronization
    ///
    /// Synchronization is deterministic and bounded.
    #[test]
    fn test_yawl_pattern_3_synchronization() {
        let registry = global_registry();
        let op = registry.get_operator("synchronization_op").expect("Synchronization not found");

        assert!(op.properties.deterministic, "Synchronization must be deterministic");
        assert!(op.properties.bounded, "Synchronization must be bounded");
        assert!(op.requires_guard(GuardType::Causality), "Synchronization requires Causality guard");
    }

    /// Theorem 3.5: YAWL Pattern 4: Exclusive Choice
    ///
    /// Exclusive Choice is deterministic and bounded.
    #[test]
    fn test_yawl_pattern_4_exclusive_choice() {
        let registry = global_registry();
        let op = registry.get_operator("exclusive_choice_op").expect("Exclusive Choice not found");

        assert!(op.properties.deterministic, "Exclusive Choice must be deterministic");
        assert!(op.properties.bounded, "Exclusive Choice must be bounded");
        assert!(op.requires_guard(GuardType::Legality), "Exclusive Choice requires Legality guard");
    }

    /// Theorem 3.6: YAWL Pattern 5: Simple Merge
    ///
    /// Simple Merge can be non-deterministic (first-come-first-served).
    #[test]
    fn test_yawl_pattern_5_simple_merge() {
        let registry = global_registry();
        let op = registry.get_operator("simple_merge_op").expect("Simple Merge not found");

        // Simple merge is often non-deterministic in concurrent systems
        assert!(!op.properties.deterministic, "Simple Merge is non-deterministic");
        assert!(op.properties.bounded, "Simple Merge must be bounded");
    }

    /// Property-based verification of registry properties
    #[test]
    fn test_registry_properties_proptest() {
        proptest!(|(op_idx in 0..10usize)| {
            let registry = global_registry();
            let ops = registry.all_operators();
            if op_idx < ops.len() {
                let op = ops[op_idx];
                // Every operator in our registry should at least be bounded
                // (except specific state-based ones if they existed)
                if op.hook_id != "state_concurrency_op" {
                    prop_assert!(op.properties.bounded, "Operator {} should be bounded", op.hook_id);
                    prop_assert!(op.max_latency_ns > 0, "Bounded operator {} should have max_latency > 0", op.hook_id);
                }
            }
        });
    }
}
