// Fixture: `last_wins` — corrupted union that drops the left operand's faults
// whenever the right operand is non-empty.
//
// Corruption: `union(a, b) = if b != EMPTY { b } else { a }`.

pub const FAULT_A: u32 = 0b0001;
pub const FAULT_B: u32 = 0b0010;
pub const EMPTY: u32 = 0;

/// Corrupted union: keeps only `b` if `b` is non-empty, silently dropping `a`.
pub fn union(a: u32, b: u32) -> u32 {
    if b != EMPTY {
        b
    } else {
        a
    }
}

#[cfg(all(test, feature = "fixture-self-test"))]
mod tests {
    use super::*;

    /// Intended oracle: must FAIL, since `union(FAULT_A, FAULT_B)` collapses to
    /// `FAULT_B` alone, dropping `FAULT_A`.
    #[test]
    fn oracle_union_preserves_both_operands_distinct_faults() {
        assert_eq!(union(FAULT_A, FAULT_B), FAULT_A | FAULT_B);
    }

    /// Secondary/unrelated oracle: unaffected by this corruption, PASSES.
    #[test]
    fn incidental_check_empty_union_is_empty() {
        assert_eq!(union(EMPTY, EMPTY), EMPTY);
    }

    /// Secondary/unrelated oracle: this corruption also breaks commutativity
    /// (`union(A, B) = B` but `union(B, A) = A`), so this FAILS too.
    #[test]
    fn incidental_check_union_is_commutative() {
        assert_eq!(union(FAULT_A, FAULT_B), union(FAULT_B, FAULT_A));
    }
}
