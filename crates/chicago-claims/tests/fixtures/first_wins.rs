// Fixture: `first_wins` — corrupted union that drops the right operand's faults
// whenever the left operand is non-empty.
//
// Corruption: `union(a, b) = if a != EMPTY { a } else { b }`. This unconditionally
// discards `b`'s faults whenever `a` already carries any fault, breaking the
// join-semilattice law.

pub const FAULT_A: u32 = 0b0001;
pub const FAULT_B: u32 = 0b0010;
pub const EMPTY: u32 = 0;

/// Corrupted union: keeps only `a` if `a` is non-empty, silently dropping `b`.
pub fn union(a: u32, b: u32) -> u32 {
    if a != EMPTY {
        a
    } else {
        b
    }
}

#[cfg(all(test, feature = "fixture-self-test"))]
mod tests {
    use super::*;

    /// Intended oracle (same name/assertion as `baseline_correct.rs`): must FAIL
    /// here, since `union(FAULT_A, FAULT_B)` collapses to `FAULT_A` alone, dropping
    /// `FAULT_B`.
    #[test]
    fn oracle_union_preserves_both_operands_distinct_faults() {
        assert_eq!(union(FAULT_A, FAULT_B), FAULT_A | FAULT_B);
    }

    /// Secondary/unrelated oracle: unaffected by this corruption (empty|empty still
    /// falls through to `b == EMPTY`), so this PASSES even though the mutant is
    /// detected by the intended oracle above.
    #[test]
    fn incidental_check_empty_union_is_empty() {
        assert_eq!(union(EMPTY, EMPTY), EMPTY);
    }

    /// Secondary/unrelated oracle: this corruption also breaks commutativity
    /// (`union(A, B) = A` but `union(B, A) = B`), so this FAILS too.
    #[test]
    fn incidental_check_union_is_commutative() {
        assert_eq!(union(FAULT_A, FAULT_B), union(FAULT_B, FAULT_A));
    }
}
