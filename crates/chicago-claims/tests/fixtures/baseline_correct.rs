// Fixture: `baseline_correct` — the correct fault-set union law: `a | b`.
//
// This is the CONTROL fixture. It implements the join-semilattice union exactly
// (`faults_out = faults_left UNION faults_right`, empty set = zero), matching the
// real `NumericFaultSet::union` shape in `bcinr-cmca`'s `fixed.rs` at the level of
// the underlying bitwise law (this fixture does not depend on bcinr-cmca at all —
// it is a small, self-contained `u32`-bitset stand-in for the same algebraic law).
//
// Two named consts stand in for two distinct, non-overlapping numeric faults, and
// `EMPTY` stands in for the zero element of the semilattice.

/// Stand-in for one numeric fault flag (distinct bit from `FAULT_B`).
pub const FAULT_A: u32 = 0b0001;
/// Stand-in for a second, distinct numeric fault flag.
pub const FAULT_B: u32 = 0b0010;
/// The zero element of the join-semilattice: the empty fault set.
pub const EMPTY: u32 = 0;

/// The correct union law: `faults_out = faults_left | faults_right`.
pub fn union(a: u32, b: u32) -> u32 {
    a | b
}

#[cfg(all(test, feature = "fixture-self-test"))]
mod tests {
    use super::*;

    /// Intended oracle: union must preserve BOTH operands' distinct faults. This is
    /// the test named in `MutantSpec::intended_oracle_test` for every mutant claim
    /// in this fixture set. It PASSES here (baseline is correct) and is written to
    /// FAIL against every corrupted variant fixture in this directory.
    #[test]
    fn oracle_union_preserves_both_operands_distinct_faults() {
        assert_eq!(union(FAULT_A, FAULT_B), FAULT_A | FAULT_B);
    }

    /// Secondary/unrelated oracle: union of two empty sets is empty. This does NOT
    /// target any specific corruption; it may or may not fail depending on which
    /// corruption is active, which is what lets the reconciliation engine
    /// distinguish `KilledByIntendedOracle` from `KilledBySecondaryOracle` from
    /// `Survived`.
    #[test]
    fn incidental_check_empty_union_is_empty() {
        assert_eq!(union(EMPTY, EMPTY), EMPTY);
    }

    /// Secondary/unrelated oracle: union should be commutative. Included as a second
    /// incidental signal distinct from the empty-union check above (some
    /// corruptions break commutativity, some do not).
    #[test]
    fn incidental_check_union_is_commutative() {
        assert_eq!(union(FAULT_A, FAULT_B), union(FAULT_B, FAULT_A));
    }
}
