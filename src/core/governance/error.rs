//! Typed error for the governance subsystem (DoD §2.3 typed-error wave).
//!
//! Replaces the previous `Result<_, String>` returns across
//! [`crate::core::governance`] with a single enum whose variants cover the
//! actual failure classes: parse, validation, missing verifying key,
//! signature, and channel state. Display messages preserve the original
//! `String`-era text (each variant renders the source message verbatim as a
//! substring, so `contains(...)` assertions keep passing).

/// Typed error for governance parse/validation/signature/channel failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GovernanceError {
    /// A string could not be parsed into a governance code/category/ordinal.
    #[error("governance parse error: {0}")]
    Parse(String),
    /// Structural or consistency validation failed.
    #[error("governance validation error: {0}")]
    Validation(String),
    /// A signature is present but no verifying key was supplied.
    #[error("governance missing key: {0}")]
    MissingKey(String),
    /// Signature verification or signing-key constraints failed.
    #[error("governance signature error: {0}")]
    Signature(String),
    /// The diagnostic channel was in an invalid state for the operation.
    #[error("governance channel state error: {0}")]
    ChannelState(String),
    /// A sink mutex was poisoned (a prior panicked holder).
    #[error("governance lock error: {0}")]
    Lock(String),
    /// OCEL log sealing failed.
    #[error("governance seal error: {0}")]
    Seal(String),
    /// JSON serialization of the sealed log failed.
    #[error("governance serialization error: {0}")]
    Serialization(String),
    /// Filesystem write of the sealed log failed.
    #[error("governance io error: {0}")]
    Io(String),
}

#[cfg(test)]
mod tests {
    use super::GovernanceError;

    #[test]
    fn display_preserves_source_message_as_substring() {
        let e = GovernanceError::Signature("Invalid signature".to_string());
        assert!(e.to_string().contains("Invalid signature"), "got: {e}");
        let e = GovernanceError::Parse("empty diagnostic code string".to_string());
        assert!(e.to_string().contains("empty diagnostic code string"), "got: {e}");
    }
}
