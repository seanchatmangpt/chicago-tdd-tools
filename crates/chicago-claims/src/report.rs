//! Reconciliation report rendering (CLI-facing).
//!
//! Produces a human-readable report in the FAQ's own example shape: claim id,
//! standing, scan evidence (explicitly labeled syntax-only), mutant evidence,
//! computed deltas, and a closing disclosure of what this report does NOT prove.

use crate::claim::Claim;
use crate::reconcile::{Delta, Reconciliation, Standing};

/// Render a [`Reconciliation`] for `claim` as a human-readable report string.
#[must_use]
pub fn render_report(claim: &Claim, r: &Reconciliation) -> String {
    let mut out = String::new();

    out.push_str(&format!("Claim: {}\n", claim.id));
    out.push_str(&format!("Standing: {}\n", format_standing(&r.standing)));
    out.push('\n');

    out.push_str("Scan evidence: (syntax evidence only)\n");
    out.push_str(&format!(
        "  - type `{}` found: {}\n",
        claim.required.type_name, r.scan_result.type_found
    ));
    out.push_str(&format!(
        "  - field `{}` observed private: {:?}\n",
        claim.required.field_name, r.scan_result.field_is_private
    ));
    for (name, found) in &r.scan_result.methods_found {
        out.push_str(&format!("  - method `{name}` found: {found}\n"));
    }
    for (name, absent) in &r.scan_result.forbidden_constructions_absent {
        out.push_str(&format!("  - forbidden construction `{name}` absent: {absent}\n"));
    }
    out.push('\n');

    out.push_str("Mutant evidence:\n");
    if r.mutant_results.is_empty() {
        out.push_str("  (no mutant results recorded)\n");
    } else {
        for (id, classification) in &r.mutant_results {
            out.push_str(&format!("  - {id}: {classification:?}\n"));
        }
    }
    out.push('\n');

    out.push_str("Delta:\n");
    out.push_str(&format!(
        "  - intent -> implementation: {}\n",
        format_delta(&r.intent_to_implementation_delta)
    ));
    out.push_str(&format!(
        "  - implementation -> evidence: {}\n",
        format_delta(&r.implementation_to_evidence_delta)
    ));
    out.push('\n');

    out.push_str(
        "This report does NOT claim: object-code branchlessness of the scanned \
         implementation (source-level AST evidence only, no disassembly performed); \
         universal unforgeability or semantic correctness of any method body (syntax \
         presence only, no proof of behavior); absence of runtime allocation or any \
         other runtime property (no execution trace was collected). See FAQ #4, #13, \
         #14 for the full evidence-scope caveats this report inherits.\n",
    );

    out
}

fn format_standing(standing: &Standing) -> String {
    match standing {
        Standing::Alive => "Alive".to_string(),
        Standing::PartialAlive(msg) => format!("PartialAlive ({msg})"),
        Standing::Blocked(msg) => format!("Blocked ({msg})"),
        Standing::Unknown => "Unknown".to_string(),
    }
}

fn format_delta(delta: &Delta) -> String {
    match delta {
        Delta::None => "none".to_string(),
        Delta::MissingStructure(msg) => format!("MissingStructure({msg})"),
        Delta::PropertyMismatch(msg) => format!("PropertyMismatch({msg})"),
        Delta::EvidenceBelowRequiredLevel(msg) => {
            format!("EvidenceBelowRequiredLevel({msg})")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::{RequiredStructure, Scope};
    use crate::reconcile::{reconcile, MutantClassification};
    use crate::scan::ScanResult;

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
                required_methods: vec!["union".to_string()],
                forbidden_constructions: vec![],
            },
            mutants: vec![],
            evidence_required: vec!["scan_result.json".to_string()],
        }
    }

    #[test]
    fn report_contains_claim_id_standing_and_evidence_scope_caveat() {
        let claim = sample_claim();
        let scan = ScanResult {
            type_found: true,
            field_is_private: Some(true),
            methods_found: vec![("union".to_string(), true)],
            forbidden_constructions_absent: vec![],
        };
        let mutants =
            vec![("first-wins".to_string(), MutantClassification::KilledByIntendedOracle)];
        let r = reconcile(&claim, scan, mutants);

        let report = render_report(&claim, &r);

        assert!(report.contains("Claim: cmca-numeric-fault-join-semilattice"));
        assert!(report.contains("Standing: Alive"));
        assert!(report.contains("(syntax evidence only)"));
        assert!(report.contains("first-wins: KilledByIntendedOracle"));
        assert!(report.contains("does NOT claim"));
        assert!(report.contains("branchlessness"));
    }

    #[test]
    fn report_on_blocked_claim_shows_missing_structure_delta() {
        let claim = sample_claim();
        let scan = ScanResult {
            type_found: false,
            field_is_private: None,
            methods_found: vec![("union".to_string(), false)],
            forbidden_constructions_absent: vec![],
        };
        let r = reconcile(&claim, scan, vec![]);

        let report = render_report(&claim, &r);

        assert!(report.contains("Blocked"));
        assert!(report.contains("MissingStructure"));
    }
}
