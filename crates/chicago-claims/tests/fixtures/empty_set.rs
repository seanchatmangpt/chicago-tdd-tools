// Fixture: `empty_set` — corrupted union that always returns the zero element,
// discarding both operands entirely.
//
// Corruption: `union(a, b) = EMPTY`.

pub const FAULT_A: u32 = 0b0001;
pub const FAULT_B: u32 = 0b0010;
pub const EMPTY: u32 = 0;

/// Corrupted union: always returns the empty set, regardless of inputs.
pub fn union(_a: u32, _b: u32) -> u32 {
    EMPTY
}

#[cfg(all(test, feature = "fixture-self-test"))]
mod tests {
    use super::*;

    /// Intended oracle: must FAIL, since the result is always `EMPTY`.
    #[test]
    fn oracle_union_preserves_both_operands_distinct_faults() {
        assert_eq!(union(FAULT_A, FAULT_B), FAULT_A | FAULT_B);
    }

    /// Secondary/unrelated oracle: PASSES — trivially true since the corrupted
    /// function always returns `EMPTY`, including for `union(EMPTY, EMPTY)`. This
    /// is the interesting case in the fixture set: the secondary oracle is BLIND to
    /// this corruption, and only the intended oracle detects it — concrete
    /// demonstration of why `intended_oracle_test` must be checked first, and why a
    /// broad "some test failed" signal is not sufficient attribution.
    #[test]
    fn incidental_check_empty_union_is_empty() {
        assert_eq!(union(EMPTY, EMPTY), EMPTY);
    }

    /// Secondary/unrelated oracle: also PASSES — `EMPTY == EMPTY` regardless of
    /// argument order, so commutativity is (trivially, vacuously) preserved by this
    /// corruption even though the union law itself is completely broken.
    #[test]
    fn incidental_check_union_is_commutative() {
        assert_eq!(union(FAULT_A, FAULT_B), union(FAULT_B, FAULT_A));
    }
}
