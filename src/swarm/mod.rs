//! Swarm Protocol: Distributed Multi-Sector Coordination
//!
//! Enables multiple agents to coordinate across sectors, share knowledge hooks,
//! and compose operations deterministically. The swarm operates without central
//! control, using gossip and consensus mechanisms.
//!
//! ## Architecture
//!
//! ```
//! Agent 1 (Academic)     Agent 2 (Claims)     Agent 3 (Multi-Sector)
//!    ↓                        ↓                         ↓
//! SwarmMember            SwarmMember             SwarmMember
//!    ↓                        ↓                         ↓
//! ─────────────────────── SwarmCoordinator ──────────────────────
//!    ↓                        ↓                         ↓
//! TaskQueue          KnowledgeComposition          Consensus
//! ```

pub mod composition;
pub mod coordinator;
pub mod member;
pub mod task;
pub mod test_orchestrator;

pub use composition::{ComposedOperation, OperationChain};
pub use coordinator::{SwarmCoordinator, SwarmMembership};
pub use member::SwarmMember;
pub use task::{TaskReceipt, TaskRequest, TaskStatus};
pub use test_orchestrator::{
    QoSClass, ResourceBudget, TestOrchestrator, TestPlan, TestPlanningAPI,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_protocol_available() {
        // Verify swarm protocol modules are available
        let _membership = SwarmMembership::new();
    }
}
