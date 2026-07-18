//! Claim reconciliation: combine scan observations and mutant-kill outcomes into a
//! typed standing (per the project's ALIVE/PARTIAL/BLOCKED/MOCKED/REFUSED/UNSUPPORTED/
//! UNVERIFIED vocabulary, narrowed here to Alive/PartialAlive/Blocked/Unknown for this
//! MLP slice).
//!
//! NOTE: `MutantClassification` is defined here as a minimal compatible stub because
//! `mutate.rs` was still a placeholder stub (no enum) at the time this module was
//! written, per a sibling task working on `mutate.rs` in parallel. If `mutate.rs` has
//! since grown its own `MutantClassification`, the two definitions need to be
//! de-duplicated (re-export one from the other, or move the enum to a shared location)
//! before this crate compiles cleanly with both modules using it.

use crate::claim::Claim;
use crate::scan::ScanResult;

/// Outcome of activating a single named mutant against claim-scoped code and running
/// its intended oracle test (and, where applicable, other tests in the suite).
///
/// STUB/compat: minimal enum mirroring what `mutate.rs` is expected to produce. See
/// the module-level doc comment for de-duplication notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutantClassification {
    /// The mutant was killed by the test named as `intended_oracle_test` in the claim.
    KilledByIntendedOracle,
    /// The mutant was killed, but by some other test in the suite, not the one named
    /// as the intended oracle.
    KilledBySecondaryOracle,
    /// The mutant compiled and ran but no test failed — it survived.
    Survived,
    /// The mutant is behaviorally indistinguishable from the original code (no test
    /// could ever kill it).
    Equivalent,
    /// The mutant could not be meaningfully activated (e.g. does not type-check).
    Unviable,
    /// The mutated code path was never exercised by any test.
    NotReached,
    /// The mutation harness itself failed before a kill/survive determination could
    /// be made, for a reason not covered by the more specific variants below
    /// (subprocess spawn failure, filesystem I/O failure during setup).
    MutationGateFailed,
    /// Infrastructure (filesystem, compiler toolchain, CI runner) blocked the
    /// mutation run entirely.
    InfrastructureBlocked,
    /// The isolated copy of the target source did not build: distinct from
    /// `MutationGateFailed` — this specifically means a build subprocess ran but
    /// never reached a `test result:` summary line at all, as opposed to a
    /// harness-level I/O or spawn failure.
    CompilationFailed,
    /// The mutant's configured search pattern could not be uniquely located in
    /// the target file (zero or more than one occurrence), so no patch was
    /// applied and no oracle was run.
    MutationActivationFailed,
    /// The isolated copy built successfully but the named oracle test could not
    /// be run to a real pass/fail determination for an infrastructure reason
    /// (e.g. the test-name filter matched zero tests).
    OracleInfrastructureFailed,
    /// The oracle subprocess exceeded its configured time budget and was killed
    /// before reaching a pass/fail determination.
    Timeout,
}

impl MutantClassification {
    /// Whether this classification counts as a clean kill for standing purposes:
    /// killed by the intended oracle, or killed by some other oracle in the suite.
    #[must_use]
    pub const fn is_clean_kill(self) -> bool {
        matches!(self, Self::KilledByIntendedOracle | Self::KilledBySecondaryOracle)
    }
}

/// A gap between what a claim asserts (intent), what the source structurally
/// contains (implementation), or what evidence has been produced (evidence) — a
/// deliberately small subset of the full FAQ delta taxonomy for this MLP slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delta {
    /// No gap observed at this stage.
    None,
    /// A required type, field, or method was not found in the scanned source.
    MissingStructure(String),
    /// A structural property (e.g. field visibility) did not match what the claim
    /// requires.
    PropertyMismatch(String),
    /// The evidence produced is below the level the claim requires to close (e.g. a
    /// mutant survived, or a required evidence artifact was never produced).
    EvidenceBelowRequiredLevel(String),
}

/// Typed standing for a claim after reconciliation, per the project's ALIVE/PARTIAL/
/// BLOCKED/... vocabulary narrowed to this MLP's four-way slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    /// Structure scan is fully clean AND every mutant was cleanly killed (by the
    /// intended or a secondary oracle).
    Alive,
    /// Structure scan is clean, but one or more mutants were not cleanly killed.
    /// Carries a human-readable summary of what's missing.
    PartialAlive(String),
    /// The structure scan itself failed (required type/field/method missing or
    /// mismatched). Carries a human-readable summary of the structural gap.
    Blocked(String),
    /// Standing could not be determined (reserved for future reconciliation paths
    /// not yet reached by this MLP slice's logic).
    Unknown,
}

/// The result of reconciling one [`Claim`] against its [`ScanResult`] and a set of
/// per-mutant classifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reconciliation {
    /// The claim's stable identifier, copied from `claim.id`.
    pub claim_id: String,
    /// The raw structural scan evidence this reconciliation was computed from.
    pub scan_result: ScanResult,
    /// Per-mutant `(mutant_id, classification)` pairs.
    pub mutant_results: Vec<(String, MutantClassification)>,
    /// Gap between the claim's intent and the scanned implementation's structure.
    pub intent_to_implementation_delta: Delta,
    /// Gap between the implementation's structure and the evidence actually
    /// produced (mutant-kill outcomes).
    pub implementation_to_evidence_delta: Delta,
    /// The computed typed standing.
    pub standing: Standing,
}

/// Reconcile a [`Claim`] against a completed structural [`ScanResult`] and a set of
/// mutant classification outcomes, producing a [`Reconciliation`] with a typed
/// [`Standing`].
///
/// Structural cleanliness requires: `scan.type_found` is `true`, and — when the
/// claim requires the field to be private — `scan.field_is_private == Some(true)`,
/// and every entry in `scan.methods_found` is `true`.
#[must_use]
pub fn reconcile(
    claim: &Claim,
    scan: ScanResult,
    mutants: Vec<(String, MutantClassification)>,
) -> Reconciliation {
    let structure_gap = structural_gap(claim, &scan);

    if let Some(gap) = structure_gap {
        return Reconciliation {
            claim_id: claim.id.clone(),
            scan_result: scan,
            mutant_results: mutants,
            intent_to_implementation_delta: Delta::MissingStructure(gap.clone()),
            implementation_to_evidence_delta: Delta::None,
            standing: Standing::Blocked(gap),
        };
    }

    let unclean: Vec<String> = mutants
        .iter()
        .filter(|(_, classification)| !classification.is_clean_kill())
        .map(|(id, classification)| format!("{id} => {classification:?}"))
        .collect();

    let (implementation_to_evidence_delta, standing) = if unclean.is_empty() {
        (Delta::None, Standing::Alive)
    } else {
        let summary = unclean.join(", ");
        (Delta::EvidenceBelowRequiredLevel(summary.clone()), Standing::PartialAlive(summary))
    };

    Reconciliation {
        claim_id: claim.id.clone(),
        scan_result: scan,
        mutant_results: mutants,
        intent_to_implementation_delta: Delta::None,
        implementation_to_evidence_delta,
        standing,
    }
}

/// Compute the structural gap description for a scan against a claim's required
/// structure, or `None` if the scan is fully clean.
fn structural_gap(claim: &Claim, scan: &ScanResult) -> Option<String> {
    if !scan.type_found {
        return Some(format!("type `{}` not found in scanned source", claim.required.type_name));
    }

    if claim.required.field_must_be_private && scan.field_is_private != Some(true) {
        return Some(format!(
            "field `{}` is not observed as private (found: {:?})",
            claim.required.field_name, scan.field_is_private
        ));
    }

    let missing_methods: Vec<&str> = scan
        .methods_found
        .iter()
        .filter(|(_, found)| !found)
        .map(|(name, _)| name.as_str())
        .collect();

    if !missing_methods.is_empty() {
        return Some(format!("required method(s) not found: {}", missing_methods.join(", ")));
    }

    let present_forbidden: Vec<&str> = scan
        .forbidden_constructions_absent
        .iter()
        .filter(|(_, absent)| !absent)
        .map(|(name, _)| name.as_str())
        .collect();

    if !present_forbidden.is_empty() {
        return Some(format!("forbidden construction(s) found: {}", present_forbidden.join(", ")));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::{RequiredStructure, Scope};

    fn sample_claim() -> Claim {
        Claim {
            id: "cmca-numeric-fault-join-semilattice".to_string(),
            scope: Scope {
                file: "crates/bcinr-cmca/src/fixed.rs".to_string(),
                symbol: Some("NumericFaultSet".to_string()),
            },
            required: RequiredStructure {
                type_name: "NumericFaultSet".to_string(),
                field_name: "0".to_string(),
                field_must_be_private: true,
                required_methods: vec!["union".to_string(), "is_empty".to_string()],
                forbidden_constructions: vec![],
            },
            mutants: vec![],
            evidence_required: vec!["scan_result.json".to_string()],
        }
    }

    fn clean_scan() -> ScanResult {
        ScanResult {
            type_found: true,
            field_is_private: Some(true),
            methods_found: vec![("union".to_string(), true), ("is_empty".to_string(), true)],
            forbidden_constructions_absent: vec![],
        }
    }

    #[test]
    fn clean_scan_and_all_mutants_killed_yields_alive() {
        let claim = sample_claim();
        let scan = clean_scan();
        let mutants = vec![
            ("first-wins".to_string(), MutantClassification::KilledByIntendedOracle),
            ("last-wins".to_string(), MutantClassification::KilledBySecondaryOracle),
            ("left-only".to_string(), MutantClassification::KilledByIntendedOracle),
            ("right-only".to_string(), MutantClassification::KilledByIntendedOracle),
            ("empty-set".to_string(), MutantClassification::KilledByIntendedOracle),
            ("overwrite".to_string(), MutantClassification::KilledByIntendedOracle),
        ];

        let result = reconcile(&claim, scan, mutants);

        assert_eq!(result.standing, Standing::Alive);
        assert_eq!(result.intent_to_implementation_delta, Delta::None);
        assert_eq!(result.implementation_to_evidence_delta, Delta::None);
        assert_eq!(result.claim_id, "cmca-numeric-fault-join-semilattice");
    }

    #[test]
    fn missing_type_yields_blocked() {
        let claim = sample_claim();
        let scan = ScanResult {
            type_found: false,
            field_is_private: None,
            methods_found: vec![("union".to_string(), false), ("is_empty".to_string(), false)],
            forbidden_constructions_absent: vec![],
        };

        let result = reconcile(&claim, scan, vec![]);

        match result.standing {
            Standing::Blocked(ref msg) => {
                assert!(msg.contains("NumericFaultSet"));
            }
            other => panic!("expected Standing::Blocked, got {other:?}"),
        }
        match result.intent_to_implementation_delta {
            Delta::MissingStructure(ref msg) => assert!(msg.contains("NumericFaultSet")),
            other => panic!("expected Delta::MissingStructure, got {other:?}"),
        }
    }

    #[test]
    fn clean_scan_with_one_survived_mutant_yields_partial_alive() {
        let claim = sample_claim();
        let scan = clean_scan();
        let mutants = vec![
            ("first-wins".to_string(), MutantClassification::KilledByIntendedOracle),
            ("last-wins".to_string(), MutantClassification::Survived),
        ];

        let result = reconcile(&claim, scan, mutants);

        match result.standing {
            Standing::PartialAlive(ref msg) => assert!(msg.contains("last-wins")),
            other => panic!("expected Standing::PartialAlive, got {other:?}"),
        }
        assert!(matches!(
            result.implementation_to_evidence_delta,
            Delta::EvidenceBelowRequiredLevel(_)
        ));
    }

    #[test]
    fn private_field_mismatch_yields_blocked() {
        let claim = sample_claim();
        let scan = ScanResult {
            type_found: true,
            field_is_private: Some(false),
            methods_found: vec![("union".to_string(), true), ("is_empty".to_string(), true)],
            forbidden_constructions_absent: vec![],
        };

        let result = reconcile(&claim, scan, vec![]);

        assert!(matches!(result.standing, Standing::Blocked(_)));
    }
}
