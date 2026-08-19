//! Manufactured external contracts. Do not edit.
use super::types::{Authority, ExternalContract};

pub const EXTERNAL_CONTRACTS: &[ExternalContract] = &[
    ExternalContract { order: 1, id: "contract.otlp", protocol: "otlp", source_realization: "obs.otel", authority: Authority::Observer, receipt_required: true, replay_required: true, private_dependency_leak: false, standing: "ADMITTED" },
    ExternalContract { order: 2, id: "contract.ocel", protocol: "ocel-2.0", source_realization: "obs.ocel", authority: Authority::Observer, receipt_required: true, replay_required: true, private_dependency_leak: false, standing: "ADMITTED" },
    ExternalContract { order: 3, id: "contract.cli", protocol: "stdio", source_realization: "verify.cli", authority: Authority::Observer, receipt_required: true, replay_required: true, private_dependency_leak: false, standing: "ADMITTED" },
    ExternalContract { order: 4, id: "contract.docker", protocol: "docker-api", source_realization: "verify.integration", authority: Authority::Observer, receipt_required: true, replay_required: true, private_dependency_leak: false, standing: "ADMITTED" },
    ExternalContract { order: 5, id: "contract.container-broker", protocol: "docker-api", source_realization: "actuate.testcontainers", authority: Authority::Brokered, receipt_required: true, replay_required: true, private_dependency_leak: false, standing: "ADMITTED" },
    ExternalContract { order: 6, id: "contract.sector-broker", protocol: "rust-api", source_realization: "actuate.sector", authority: Authority::Brokered, receipt_required: true, replay_required: true, private_dependency_leak: false, standing: "ADMITTED" },
    ExternalContract { order: 7, id: "contract.git-hook", protocol: "git-hook-intent", source_realization: "actuate.git_hook", authority: Authority::Brokered, receipt_required: true, replay_required: true, private_dependency_leak: false, standing: "ADMITTED" },
    ExternalContract { order: 8, id: "contract.ocel-receipt", protocol: "ocel-2.0", source_realization: "receipt.ocel", authority: Authority::Observer, receipt_required: true, replay_required: true, private_dependency_leak: false, standing: "ADMITTED" },
    ExternalContract { order: 9, id: "contract.ocel-replay", protocol: "ocel-2.0", source_realization: "replay.ocel", authority: Authority::Observer, receipt_required: true, replay_required: true, private_dependency_leak: false, standing: "ADMITTED" },
    ExternalContract { order: 10, id: "contract.verifier-report", protocol: "verifier-report-v1", source_realization: "replay.verifier_report", authority: Authority::Observer, receipt_required: true, replay_required: true, private_dependency_leak: false, standing: "ADMITTED" },
];
