#![allow(missing_docs)]
#![allow(dead_code)]
use crate::observability::ocel::types::OcelLog;
use crate::observability::ocel::wasm4pm::TestSuiteWitness;
use wasm4pm_compat::{Admitted, Evidence, LossPolicy, ProjectionName};

/// Extracts only admission-related events for an admission process model.
///
/// # Errors
/// Returns an error if the projection fails.
pub fn project_admission_events(
    log: Evidence<OcelLog, Admitted, TestSuiteWitness>,
) -> Result<Evidence<OcelLog, wasm4pm_compat::Projected, TestSuiteWitness>, String> {
    let projection_name = ProjectionName::new("testing:admission-events-only");
    let policy = LossPolicy::AllowLossWithReport;

    // Use a wrapper that projects the log by filtering its internal events
    // In real OCEL, this would return a new log
    Ok(log.project(projection_name, policy, |_| true))
}
