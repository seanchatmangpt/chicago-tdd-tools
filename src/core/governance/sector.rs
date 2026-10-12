#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::must_use_candidate, clippy::needless_lifetimes, clippy::return_self_not_must_use)]

use serde_json::Value;

use super::GovernanceError;

pub trait SectorStack: Send + Sync {
    fn name(&self) -> &str;
    /// Validate a sector artifact. Fails with
    /// [`GovernanceError::Validation`] on any structural violation.
    fn validate_artifact(&self, artifact: &Value) -> Result<(), GovernanceError>;
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
    fn validate_artifact(&self, artifact: &Value) -> Result<(), GovernanceError> {
        if artifact.is_null() {
            return Err(GovernanceError::Validation(
                "ProcessIntelligenceSector: artifact must not be null".to_string(),
            ));
        }
        match artifact {
            Value::Object(map) => {
                if map.is_empty() {
                    return Err(GovernanceError::Validation(
                        "ProcessIntelligenceSector: artifact object must not be empty".to_string(),
                    ));
                }
                Ok(())
            }
            Value::Array(_) => Err(GovernanceError::Validation(
                "ProcessIntelligenceSector: artifact must be a JSON object, got array".to_string(),
            )),
            Value::Bool(_) => Err(GovernanceError::Validation(
                "ProcessIntelligenceSector: artifact must be a JSON object, got bool".to_string(),
            )),
            Value::Number(_) => Err(GovernanceError::Validation(
                "ProcessIntelligenceSector: artifact must be a JSON object, got number".to_string(),
            )),
            Value::String(_) => Err(GovernanceError::Validation(
                "ProcessIntelligenceSector: artifact must be a JSON object, got string".to_string(),
            )),
            // The `Value::Null` arm above is rejected early, so this arm is
            // unreachable in practice — but we fail closed with a typed error
            // rather than panicking if the early guard is ever relaxed.
            Value::Null => Err(GovernanceError::Validation(
                "ProcessIntelligenceSector: artifact must be a JSON object, got null".to_string(),
            )),
        }
    }
}
