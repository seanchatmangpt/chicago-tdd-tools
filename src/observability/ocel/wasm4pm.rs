#![allow(missing_docs)]
#![allow(dead_code)]
use crate::core::governance::RunId;
use crate::observability::ocel::collector::OcelCollector;
use crate::observability::ocel::types::{OcelLog, TestOcelEvent};
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

    let mut hasher = blake3::Hasher::new();
    for (id, ev) in &log.events {
        hasher.update(id.as_bytes());
        hasher.update(ev.case_id.as_bytes());
        hasher.update(&ev.timestamp_ns.to_le_bytes());
        hasher.update(format!("{:?}", ev.activity).as_bytes());
        for (obj_id, obj_type) in &ev.objects {
            hasher.update(obj_id.as_bytes());
            hasher.update(format!("{obj_type:?}").as_bytes());
        }
    }

    let digest_bytes: [u8; 32] = *hasher.finalize().as_bytes();
    let digest_hex = digest_bytes.iter().fold(String::with_capacity(64), |mut acc, b| {
        use std::fmt::Write as _;
        let _ = write!(acc, "{b:02x}");
        acc
    });

    let admitted = Admission::<_, TestSuiteWitness>::new(log).into_evidence();
    let receipted = admitted.into_receipted();
    Ok((receipted, digest_hex))
}
