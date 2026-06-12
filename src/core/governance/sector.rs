#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::must_use_candidate, clippy::needless_lifetimes, clippy::return_self_not_must_use)]

use serde_json::Value;

pub trait SectorStack: Send + Sync {
    fn name(&self) -> &str;
    fn validate_artifact(&self, artifact: &Value) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub enum MergeStrategy {
    Strict,
    Lenient,
    Precedence(Vec<String>),
}

pub struct ProcessIntelligenceSector;
impl SectorStack for ProcessIntelligenceSector {
    fn name(&self) -> &str {
        "process_intelligence"
    }
    fn validate_artifact(&self, _artifact: &Value) -> Result<(), String> {
        Ok(())
    }
}
