// Fixture: `overwrite` — corrupted union with the SAME observable shape as
// `right_only` (`union(a, b) = b`, `a` unconditionally discarded), but kept as a
// DISTINCT named fixture rather than merged into `right_only.rs`.
//
// Merge decision (explicit, per task instructions): NOT merged. The FAQ's mutant
// catalogue names `right-only` and `overwrite` as two separate claim entries
// (distinct `MutantSpec::id`s pointing at distinct `fixture_path`s), because they
// model two different real-world bug shapes even though, for this minimal
// bit-union stand-in, their corrupted logic happens to coincide:
//   - `right-only`: a caller mistakenly threads only the right-hand fault set
//     through a merge function (a *selection* bug — the wrong operand is chosen).
//   - `overwrite`: an in-place accumulator is assigned (`acc = b`) instead of
//     joined (`acc = acc | b`) — a *mutation* bug (assignment instead of union),
//     which happens to reduce to the same `b`-only output when the accumulator
//     starts as `a`.
// Keeping them as separate claims/fixtures preserves the 1:1 mapping between the
// FAQ's six named mutant categories and this MLP's evidence artifacts, so a future
// reader auditing the mutant ledger sees six distinct classifications rather than
// a merged five, and the reconciliation report's mutant table stays legible against
// the FAQ text it is grounding. If the two fixtures' logic ever needs to diverge
// (e.g. `overwrite` gains a partial-overwrite variant), they are already separate
// files and no further split is needed.

pub const FAULT_A: u32 = 0b0001;
pub const FAULT_B: u32 = 0b0010;
pub const EMPTY: u32 = 0;

/// Corrupted union: models an in-place accumulator overwrite (`acc = b` instead of
/// `acc = acc | b`), which for a two-argument `union(a, b)` reduces to returning
/// `b` unconditionally.
pub fn union(_a: u32, b: u32) -> u32 {
    b
}

#[cfg(all(test, feature = "fixture-self-test"))]
mod tests {
    use super::*;

    /// Intended oracle: must FAIL, since `a` is never observed.
    #[test]
    fn oracle_union_preserves_both_operands_distinct_faults() {
        assert_eq!(union(FAULT_A, FAULT_B), FAULT_A | FAULT_B);
    }

    /// Secondary/unrelated oracle: unaffected by this corruption, PASSES.
    #[test]
    fn incidental_check_empty_union_is_empty() {
        assert_eq!(union(EMPTY, EMPTY), EMPTY);
    }

    /// Secondary/unrelated oracle: `union(A, B) = B` but `union(B, A) = A`, so this
    /// FAILS too.
    #[test]
    fn incidental_check_union_is_commutative() {
        assert_eq!(union(FAULT_A, FAULT_B), union(FAULT_B, FAULT_A));
    }
}
