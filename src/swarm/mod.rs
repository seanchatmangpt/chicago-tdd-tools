//! Swarm Protocol: In-Process Multi-Sector Coordination
//!
//! Coordinates multiple logical agents within a single process: members
//! register with a coordinator, tasks are queued and distributed by
//! reputation/capacity, and completions produce receipts. This is plain
//! in-process coordination (a membership `HashMap` and a task `Vec` queue) —
//! there is no network transport, gossip protocol, or distributed consensus.
//!
//! The `check_consensus` method is a simple idle-fraction heuristic: it
//! reports whether at least `consensus_threshold` of a sector's members are
//! currently idle (`current_tasks == 0`). It is not a voting or agreement
//! protocol.
//!
//! ## Architecture
//!
//! ```text
//! Agent 1 (Academic)     Agent 2 (Claims)     Agent 3 (Multi-Sector)
//!    ↓                        ↓                         ↓
//! SwarmMember            SwarmMember             SwarmMember
//!    ↓                        ↓                         ↓
//! ─────────────────────── SwarmCoordinator ──────────────────────
//!    ↓                        ↓                         ↓
//! TaskQueue          KnowledgeComposition     IdleFractionHeuristic
//! ```

pub mod composition;
pub mod coordinator;
pub mod member;
pub mod task;
pub mod test_orchestrator;
pub mod wave;

pub use composition::{ComposedOperation, OperationChain};
pub use coordinator::{SwarmCoordinator, SwarmMembership};
pub use member::{SwarmError, SwarmMember};
pub use task::{TaskReceipt, TaskRequest, TaskStatus};
pub use test_orchestrator::{
    QoSClass, ResourceBudget, TestOrchestrator, TestPlan, TestPlanningAPI,
};
pub use wave::{ResidualClass, Wave, WavePhase, WaveReceipt, WaveStatus};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_protocol_available() {
        // Verify swarm protocol modules are available
        let _membership = SwarmMembership::new();
    }
}
