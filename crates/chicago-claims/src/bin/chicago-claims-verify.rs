//! CLI: load a claim TOML, scan real source, activate all named mutants, reconcile,
//! and print the report. Exits nonzero if the reconciled standing is `Blocked`.
//!
//! Usage: `chicago-claims-verify <path-to-claim.toml>`

use std::path::Path;
use std::process::ExitCode;

use chicago_claims::{
    execute_mutant, reconcile, render_report, scan_required_structure, Claim, MutantClassification,
    Standing,
};

fn main() -> ExitCode {
    let mut args = std::env::args();
    let _program = args.next();
    let Some(claim_path) = args.next() else {
        eprintln!("usage: chicago-claims-verify <path-to-claim.toml>");
        return ExitCode::FAILURE;
    };

    let claim = match Claim::load_from_toml(Path::new(&claim_path)) {
        Ok(claim) => claim,
        Err(err) => {
            eprintln!("failed to load claim from {claim_path}: {err}");
            return ExitCode::FAILURE;
        }
    };

    let scan_result = match scan_required_structure(Path::new(&claim.scope.file), &claim.required) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("failed to scan {}: {err}", claim.scope.file);
            return ExitCode::FAILURE;
        }
    };

    // Resolution/activation/oracle dispatch is entirely config-driven — see
    // chicago_claims::resolve_provider, the ONE place a MutantSpec's populated
    // fields are inspected to pick a provider. No claim ID or fixture name is
    // matched against here or anywhere else in this binary.
    let mutant_results: Vec<_> = claim
        .mutants
        .iter()
        .map(|m| match execute_mutant(m) {
            Ok(report) => {
                let oracle_line = report
                    .oracle
                    .as_ref()
                    .map(|o| {
                        let digest = o
                            .output_digest
                            .as_deref()
                            .map(|d| format!(" [digest {d}]"))
                            .unwrap_or_default();
                        format!("; oracle: {}{digest}", o.detail)
                    })
                    .unwrap_or_default();
                eprintln!(
                    "  mutant `{}`: {:?} (activation: {}{oracle_line})",
                    m.id, report.classification, report.activation.detail
                );
                (m.id.clone(), report.classification)
            }
            Err(err) => {
                eprintln!("  mutant `{}`: provider resolution refused: {err}", m.id);
                (m.id.clone(), MutantClassification::InfrastructureBlocked)
            }
        })
        .collect();

    let reconciliation = reconcile(&claim, scan_result, mutant_results);
    let report = render_report(&claim, &reconciliation);
    println!("{report}");

    if matches!(reconciliation.standing, Standing::Blocked(_)) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
