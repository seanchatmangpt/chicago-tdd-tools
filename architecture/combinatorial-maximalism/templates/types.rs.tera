//! Pure combinatorial-maximalism types. Manufactured; do not edit.

/// Realization visibility at the internal/external boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Private implementation detail within the admitted repository boundary.
    Internal,
    /// Published interoperable surface with an explicit external contract.
    External,
}

/// Authority carried by a realization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Authority {
    /// Pure graph-domain construction with no side-effect authority.
    Pure,
    /// Read-only observation of an external collaborator.
    Observer,
    /// Side effects permitted only through an explicit broker grant.
    Brokered,
}

/// Law-state standing. Lifecycle and standing remain separate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Candidate composition manufactured from admitted facts.
    Candidate,
    /// Candidate whose evidence obligations were observed.
    Verified,
    /// Verified plan carrying an explicit bounded execution grant.
    Authorized,
    /// Consequence observed after broker execution.
    Actuated,
    /// Consequence bound to a causal receipt.
    Receipted,
}

/// One constitutional lifecycle facet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Facet {
    pub order: u16,
    pub id: &'static str,
    pub title: &'static str,
    pub phase: &'static str,
    pub description: &'static str,
}

/// One admitted implementation choice within a facet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Realization {
    pub order: u16,
    pub id: &'static str,
    pub facet: &'static str,
    pub module: &'static str,
    pub visibility: Visibility,
    pub authority: Authority,
    pub effect: &'static str,
    pub protocol: &'static str,
    pub dependency: &'static str,
    pub standing: &'static str,
    pub receipt_required: bool,
}

/// Named bounded combination of realizations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    pub order: u16,
    pub id: &'static str,
    pub title: &'static str,
    pub external_allowed: bool,
    pub include_all: bool,
    pub selections: &'static str,
    pub standing: &'static str,
    pub description: &'static str,
}


/// Published external interoperability contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalContract {
    pub order: u16,
    pub id: &'static str,
    pub protocol: &'static str,
    pub source_realization: &'static str,
    pub authority: Authority,
    pub receipt_required: bool,
    pub replay_required: bool,
    pub private_dependency_leak: bool,
    pub standing: &'static str,
}

/// One orthogonal projection dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectionAxis {
    pub order: u16,
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

/// One proof obligation required for verified standing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceObligation {
    pub order: u16,
    pub id: &'static str,
    pub title: &'static str,
    pub required: bool,
}

/// One stable refusal definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefusalDefinition {
    pub order: u16,
    pub code: &'static str,
    pub name: &'static str,
    pub boundary: &'static str,
}

/// Request to manufacture a candidate plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompositionRequest<'a> {
    pub profile: &'a str,
    pub include_external: bool,
}

/// One realization projected onto one architecture axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Projection {
    pub realization_id: &'static str,
    pub axis_id: &'static str,
}

/// Pure candidate plan. This type carries no actuation authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidatePlan {
    pub profile: &'static Profile,
    pub realizations: Vec<&'static Realization>,
    pub projections: Vec<Projection>,
    pub external_contracts: Vec<&'static ExternalContract>,
    pub fingerprint: u64,
    pub standing: Standing,
}

/// Bitset of independently observed proof obligations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceSet {
    bits: u64,
}

impl EvidenceSet {
    #[must_use]
    pub const fn none() -> Self { Self { bits: 0 } }

    #[must_use]
    pub const fn from_bits(bits: u64) -> Self { Self { bits } }

    #[must_use]
    pub const fn bits(self) -> u64 { self.bits }
}

/// Verified plan. Verification is not authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPlan {
    pub candidate: CandidatePlan,
    pub standing: Standing,
}

/// Explicit bounded authority grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionGrant {
    pub exact_head: String,
    pub allow_external: bool,
    pub allow_brokered_actuation: bool,
    pub allow_network: bool,
    pub allow_storage: bool,
    pub allow_process: bool,
    pub allow_filesystem: bool,
    pub receipt_required: bool,
}

/// Authorized plan. Still no side effect has occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedPlan {
    pub verified: VerifiedPlan,
    pub grant: ExecutionGrant,
    pub standing: Standing,
}

/// Intent emitted to the sole actuation broker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerIntent {
    pub profile_id: &'static str,
    pub plan_fingerprint: u64,
    pub exact_head: String,
    pub realization_ids: Vec<&'static str>,
}

/// Observed consequence returned by the broker boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedConsequence {
    pub plan_fingerprint: u64,
    pub exact_head: String,
    pub artifact_digest_blake3: String,
    pub consequence_ok: bool,
    pub receiver_verified: bool,
}

/// Causal consequence receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsequenceReceipt {
    pub profile_id: &'static str,
    pub plan_fingerprint: u64,
    pub exact_head: String,
    pub artifact_digest_blake3: String,
    pub standing: Standing,
}

/// Stable pure-kernel refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    UnknownProfile,
    UnknownRealization,
    DuplicateRealization,
    MissingFacet,
    DependencyNotClosed,
    ExternalBoundaryNotAdmitted,
    ExternalContractMissing,
    AuthorityLeak,
    ProjectionOverflow,
    EvidenceIncomplete,
    ExactHeadInvalid,
    ReceiptRequired,
    ConsequenceMismatch,
    ReceiptDigestInvalid,
}

impl Refusal {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnknownProfile | Self::UnknownRealization | Self::DuplicateRealization => "CMD-REA-001",
            Self::MissingFacet => "CMD-PRF-001",
            Self::DependencyNotClosed => "CMD-DEP-001",
            Self::ExternalBoundaryNotAdmitted | Self::ExternalContractMissing => "CMD-EXT-001",
            Self::AuthorityLeak => "CMD-AUT-001",
            Self::ProjectionOverflow => "CMD-PRJ-001",
            Self::EvidenceIncomplete => "CMD-EVD-001",
            Self::ExactHeadInvalid => "CMD-RCP-002",
            Self::ReceiptRequired | Self::ReceiptDigestInvalid | Self::ConsequenceMismatch => "CMD-RCP-001",
        }
    }
}
