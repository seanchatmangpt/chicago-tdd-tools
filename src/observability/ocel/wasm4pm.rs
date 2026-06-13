#![allow(missing_docs)]
#![allow(dead_code)]
use crate::core::governance::RunId;
use crate::observability::ocel::collector::OcelCollector;
use crate::observability::ocel::types::{OcelLog, TestOcelEvent};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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
            if event.timestamp_ns <= *last_ts {
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
    let mut hasher = DefaultHasher::new();
    for (id, ev) in &admitted_log.inner().events {
        id.hash(&mut hasher);
        ev.case_id.hash(&mut hasher);
        ev.timestamp_ns.hash(&mut hasher);
        ev.activity.hash(&mut hasher);
        for (obj_id, obj_type) in &ev.objects {
            obj_id.hash(&mut hasher);
            obj_type.hash(&mut hasher);
        }
    }
    // Derive a 32-byte digest by mixing the 64-bit hash across all four
    // 8-byte lanes so no lane is left as zero bytes.
    let hash_value = hasher.finish();
    let h0 = hash_value.to_le_bytes();
    // Each lane is XOR-rotated to produce independent-looking material.
    let h1 = (hash_value.rotate_left(17) ^ 0x9e37_79b9_7f4a_7c15_u64).to_le_bytes();
    let h2 = (hash_value.rotate_left(31) ^ 0x6c62_272e_07bb_0142_u64).to_le_bytes();
    let h3 = (hash_value.rotate_left(47) ^ 0x94d0_49bb_1331_11eb_u64).to_le_bytes();
    let mut digest = vec![0u8; 32];
    digest[0..8].copy_from_slice(&h0);
    digest[8..16].copy_from_slice(&h1);
    digest[16..24].copy_from_slice(&h2);
    digest[24..32].copy_from_slice(&h3);
    Ok(admitted_log.receipt(digest))
}
