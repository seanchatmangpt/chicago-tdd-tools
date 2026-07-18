// Fixture: `left_only` — corrupted union that always returns the left operand,
// ignoring the right operand entirely (unconditionally, regardless of emptiness).
//
// Corruption: `union(a, b) = a`.

pub const FAULT_A: u32 = 0b0001;
pub const FAULT_B: u32 = 0b0010;
pub const EMPTY: u32 = 0;

/// Corrupted union: always returns `a`, `b` is never consulted.
pub fn union(a: u32, _b: u32) -> u32 {
    a
}

#[cfg(all(test, feature = "fixture-self-test"))]
mod tests {
    use super::*;

    /// Intended oracle: must FAIL, since `b` is never observed.
    #[test]
    fn oracle_union_preserves_both_operands_distinct_faults() {
        assert_eq!(union(FAULT_A, FAULT_B), FAULT_A | FAULT_B);
    }

    /// Secondary/unrelated oracle: unaffected by this corruption, PASSES.
    #[test]
    fn incidental_check_empty_union_is_empty() {
        assert_eq!(union(EMPTY, EMPTY), EMPTY);
    }

    /// Secondary/unrelated oracle: `union(A, B) = A` but `union(B, A) = B`, so this
    /// FAILS too.
    #[test]
    fn incidental_check_union_is_commutative() {
        assert_eq!(union(FAULT_A, FAULT_B), union(FAULT_B, FAULT_A));
    }
}
