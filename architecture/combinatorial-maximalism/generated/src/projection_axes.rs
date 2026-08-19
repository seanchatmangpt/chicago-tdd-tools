//! Manufactured projection axes. Do not edit.
use super::types::ProjectionAxis;

pub const PROJECTION_AXES: &[ProjectionAxis] = &[
    ProjectionAxis { order: 1, id: "axis.strategy", title: "Strategy", description: "Capability and value projection." },
    ProjectionAxis { order: 2, id: "axis.process", title: "Process", description: "Workflow and lifecycle projection." },
    ProjectionAxis { order: 3, id: "axis.role", title: "Role", description: "Ownership and responsibility projection." },
    ProjectionAxis { order: 4, id: "axis.data", title: "Data", description: "Semantic and storage projection." },
    ProjectionAxis { order: 5, id: "axis.application", title: "Application", description: "API and module projection." },
    ProjectionAxis { order: 6, id: "axis.technology", title: "Technology", description: "Runtime and infrastructure projection." },
    ProjectionAxis { order: 7, id: "axis.control", title: "Control", description: "Policy and authority projection." },
    ProjectionAxis { order: 8, id: "axis.evidence", title: "Evidence", description: "Test, receipt, and verifier projection." },
    ProjectionAxis { order: 9, id: "axis.deployment", title: "Deployment", description: "Environment and rollout projection." },
    ProjectionAxis { order: 10, id: "axis.consumer", title: "Consumer", description: "External interoperability projection." },
];
