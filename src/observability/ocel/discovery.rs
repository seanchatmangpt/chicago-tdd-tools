//! OCEL process-model store and graduation helpers.
// Mutex poison recovery via `unwrap_or_else(|e| e.into_inner())` is intentional.
#![allow(clippy::unwrap_used)]
use crate::observability::ocel::types::{OcelLog, TestSuiteWitness};
use std::collections::HashMap;
use std::sync::Mutex;
use wasm4pm_compat::{Evidence, GraduationCandidate, GraduationReason, Receipted};

pub struct ProcessModelStore {
    models: Mutex<HashMap<String, Vec<u8>>>,
}

impl ProcessModelStore {
    #[must_use]
    pub fn new() -> Self {
        Self { models: Mutex::new(HashMap::new()) }
    }

    pub fn store(&self, name: &str, model: Vec<u8>) {
        let mut models = self.models.lock().unwrap_or_else(|e| e.into_inner());
        let _ = models.insert(name.to_string(), model);
    }
}

impl Default for ProcessModelStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn graduate_for_discovery(
    receipted_log: Evidence<OcelLog, Receipted, TestSuiteWitness>,
) -> GraduationCandidate {
    receipted_log.graduate(GraduationReason::NeedsDiscovery, "test-execution-process-model")
}
