//! Capability 04 — property-based testing (`ProptestStrategy`) with pinned
//! deterministic seeds derived via BLAKE3 (through the ABI's own
//! `envelope_hash`, so no second hashing scheme exists).
//!
//! Properties from the Fable Operating Constitution:
//! - distinct envelopes -> distinct hashes (identity injectivity),
//! - the 256-vs-257 Triple8 term fence,
//! - mask-arithmetic truth vs `AdmissionTable8` (P1/P5/P6: the precomputed
//!   table must agree with the defining mask predicate on all 256 states).

mod chatman_common;

use chatman_common::{envelope, seed_from};
use chicago_tdd_tools::test;
use chicago_tdd_tools::testing::property::ProptestStrategy;
use praxis_graphlaw::chatman::abi::{InvocationId, ProfileId};
use praxis_graphlaw::chatman::admission8::{AdmissionTable8, ConstraintMask};
use praxis_graphlaw::chatman::triple8::ProfileSymbolTable;

test!(property_distinct_envelopes_have_distinct_hashes, {
    // Arrange: pinned seed — the same value sequence on every run.
    let strategy = ProptestStrategy::new()
        .with_cases(64)
        .with_seed(seed_from("distinct-envelopes"));

    // Act + Assert: differing invocation ids never collide; equal inputs
    // always agree (determinism is the other face of injectivity).
    strategy.test_default::<(String, String), _>(|(a, b)| {
        let mut ea = envelope("prop");
        let mut eb = envelope("prop");
        ea.invocation_id = InvocationId::new(a.clone());
        eb.invocation_id = InvocationId::new(b.clone());
        if a == b {
            ea.envelope_hash() == eb.envelope_hash()
        } else {
            ea.envelope_hash() != eb.envelope_hash()
        }
    });
});

test!(property_triple8_universe_fence_at_256_terms, {
    // Arrange: pinned seed; term counts straddle the fence via modulo.
    let strategy = ProptestStrategy::new()
        .with_cases(48)
        .with_seed(seed_from("term-fence"));

    // Act + Assert: exactly 256 is the boundary, 257 refuses — for all n.
    strategy.test_default::<u16, _>(|raw| {
        let n = usize::from(raw % 320); // covers 0..=319, straddling 256/257
        let terms: Vec<String> = (0..n).map(|i| format!("<urn:t:{i:04}>")).collect();
        let built = ProfileSymbolTable::build(ProfileId::new("profile:fence"), terms);
        (n <= 256) == built.is_ok()
    });
});

test!(property_admission_table_agrees_with_mask_arithmetic, {
    // Arrange: pinned seed; random (required, forbidden) mask pairs.
    let strategy = ProptestStrategy::new()
        .with_cases(32)
        .with_seed(seed_from("mask-truth"));

    // Act + Assert: for every mask pair, the 256-entry precomputed table
    // must equal the defining predicate on ALL states (P1/P5/P6).
    strategy.test_default::<(u8, u8), _>(|(required, forbidden)| {
        let table = match AdmissionTable8::from_masks(
            vec!["c0".to_string()],
            ConstraintMask(required),
            ConstraintMask(forbidden),
            ConstraintMask(0),
            ConstraintMask(0),
        ) {
            Ok(t) => t,
            Err(_) => return false, // 1 name never exceeds the Need9 fence
        };
        (0u16..=255).all(|s| {
            let s = s as u8;
            let truth = (s & required) == required && (s & forbidden) == 0;
            table.lookup(s).admit == truth
        })
    });
});

test!(property_admission_successor_is_or_then_and, {
    // Arrange: pinned seed; random set/clear masks with a fixed gate.
    let strategy = ProptestStrategy::new()
        .with_cases(32)
        .with_seed(seed_from("successor-law"));

    // Act + Assert: admitted successor == (state | set) & !clear, always.
    strategy.test_default::<(u8, u8, u8), _>(|(state, set, clear)| {
        let table = match AdmissionTable8::from_masks(
            vec![],
            ConstraintMask(0), // nothing required
            ConstraintMask(0), // nothing forbidden -> every state admits
            ConstraintMask(set),
            ConstraintMask(clear),
        ) {
            Ok(t) => t,
            Err(_) => return false,
        };
        table.admit(state) == Ok((state | set) & !clear)
    });
});
