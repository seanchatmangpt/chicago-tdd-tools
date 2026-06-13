#![allow(missing_docs)]
#![allow(dead_code)]
use crate::core::governance::RunId;
use crate::observability::ocel::collector::OcelCollector;
use crate::observability::ocel::types::{OcelLog, TestOcelEvent};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use wasm4pm_compat::admission::Admission;
use wasm4pm_compat::{Admitted, Evidence, Raw, Receipted, Witness, WitnessFamily};

pub struct TestSuiteWitness;

impl Witness for TestSuiteWitness {
    const KEY: &'static str = "test-suite";
    const FAMILY: WitnessFamily = WitnessFamily::Standard;
    const TITLE: &'static str = "Chicago TDD Test Suite";
    const YEAR: Option<u16> = None;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestEventRefusal {
    MissingCaseId,
    NonMonotonicTimestamp,
    DanglingObjectReference,
}

impl OcelCollector {
    pub(crate) fn admit_event(
        &self,
        raw: &Evidence<TestOcelEvent, Raw, TestSuiteWitness>,
    ) -> Result<Evidence<TestOcelEvent, Admitted, TestSuiteWitness>, TestEventRefusal> {
        let event = raw.inner();

        if event.case_id.is_empty() {
            return Err(TestEventRefusal::MissingCaseId);
        }

        if let Some(last_ts) = self.last_timestamps.get(&event.case_id) {
            if event.timestamp_ns <= *last_ts {
                return Err(TestEventRefusal::NonMonotonicTimestamp);
            }
        }
        self.last_timestamps.insert(event.case_id.clone(), event.timestamp_ns);

        for (obj_id, _) in &event.objects {
            if !self.known_objects.contains(obj_id) {
                return Err(TestEventRefusal::DanglingObjectReference);
            }
        }

        Ok(Admission::<_, TestSuiteWitness>::new(event.clone()).into_evidence())
    }
}

/// Seals a run and generates receipted Evidence for the OCEL log, along with
/// a hex-encoded digest string.
///
/// # Errors
/// Returns an error if the log cannot be sealed.
pub fn seal_run(
    collector: &OcelCollector,
    _run_id: RunId,
) -> Result<(Evidence<OcelLog, Receipted, TestSuiteWitness>, String), String> {
    let mut log = OcelLog::new();
    {
        let events_guard = collector.events.lock().map_err(|e| e.to_string())?;
        for (i, ev) in events_guard.iter().enumerate() {
            let id = format!("evt_{i:03}");
            let _ = log.events.insert(id, ev.inner().clone());
        }
    }

    let mut hasher = DefaultHasher::new();
    for (id, ev) in &log.events {
        id.hash(&mut hasher);
        ev.case_id.hash(&mut hasher);
        ev.timestamp_ns.hash(&mut hasher);
        format!("{:?}", ev.activity).hash(&mut hasher);
        for (obj_id, obj_type) in &ev.objects {
            obj_id.hash(&mut hasher);
            obj_type.hash(&mut hasher);
        }
    }

    let hash_value = hasher.finish();
    let h0 = hash_value.to_le_bytes();
    let h1 = (hash_value.rotate_left(17) ^ 0x9e37_79b9_7f4a_7c15_u64).to_le_bytes();
    let h2 = (hash_value.rotate_left(31) ^ 0x6c62_272e_07bb_0142_u64).to_le_bytes();
    let h3 = (hash_value.rotate_left(47) ^ 0x94d0_49bb_1331_11eb_u64).to_le_bytes();
    let mut digest_bytes = [0u8; 32];
    digest_bytes[0..8].copy_from_slice(&h0);
    digest_bytes[8..16].copy_from_slice(&h1);
    digest_bytes[16..24].copy_from_slice(&h2);
    digest_bytes[24..32].copy_from_slice(&h3);
    let digest_hex = digest_bytes.iter().fold(String::with_capacity(64), |mut acc, b| {
        use std::fmt::Write as _;
        let _ = write!(acc, "{b:02x}");
        acc
    });

    let admitted = Admission::<_, TestSuiteWitness>::new(log).into_evidence();
    let receipted = admitted.into_receipted();
    Ok((receipted, digest_hex))
}
