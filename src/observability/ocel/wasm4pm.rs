#![allow(missing_docs)]
#![allow(dead_code)]
use crate::core::governance::RunId;
use crate::observability::ocel::collector::OcelCollector;
use crate::observability::ocel::types::{OcelLog, TestOcelEvent};
use wasm4pm_compat::{Admit, Admitted, Evidence, Raw, Receipted, Refusal, Witness};

pub struct TestSuiteWitness;
impl Witness for TestSuiteWitness {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestEventRefusal {
    MissingCaseId,
    NonMonotonicTimestamp,
    DanglingObjectReference,
}

impl Admit<TestOcelEvent, TestSuiteWitness> for OcelCollector {
    type Refusal = TestEventRefusal;

    fn admit(
        &self,
        raw: &Evidence<TestOcelEvent, Raw, TestSuiteWitness>,
    ) -> Result<
        Evidence<TestOcelEvent, Admitted, TestSuiteWitness>,
        Refusal<Self::Refusal, TestSuiteWitness>,
    > {
        let event = raw.inner();

        if event.case_id.is_empty() {
            return Err(Refusal::new(TestEventRefusal::MissingCaseId));
        }

        if let Some(last_ts) = self.last_timestamps.get(&event.case_id) {
            if event.timestamp_ns < *last_ts {
                return Err(Refusal::new(TestEventRefusal::NonMonotonicTimestamp));
            }
        }
        self.last_timestamps.insert(event.case_id.clone(), event.timestamp_ns);

        for (obj_id, _) in &event.objects {
            if !self.known_objects.contains(obj_id) {
                return Err(Refusal::new(TestEventRefusal::DanglingObjectReference));
            }
        }

        Ok(raw.admit_unchecked())
    }
}

/// Seals a run and generates Evidence for the OCEL log.
///
/// # Errors
/// Returns an error if the log cannot be sealed.
pub fn seal_run(
    collector: &OcelCollector,
    _run_id: RunId,
) -> Result<Evidence<OcelLog, Receipted, TestSuiteWitness>, String> {
    let events_guard = collector.events.lock().map_err(|e| e.to_string())?;
    let mut log = OcelLog::new();

    for (i, ev) in events_guard.iter().enumerate() {
        let id = format!("evt_{i:03}");
        let _ = log.events.insert(id, ev.inner().clone());
    }

    let admitted_log = Evidence::<OcelLog, Admitted, TestSuiteWitness>::new(log);
    // In real wasm4pm, this would use a proper hasher
    let digest = vec![0u8; 32];
    Ok(admitted_log.receipt(digest))
}
