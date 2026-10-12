//! Wave Orchestration (CA-8.1)
//!
//! Implements N-phase sequential waves with M parallel tasks.
//! Provides wave state observability and failure classification.

use crate::core::governance::Severity;
use crate::swarm::task::{TaskReceipt, TaskRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of a wave execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveStatus {
    /// Wave is queued and waiting for execution.
    Queued,
    /// Wave is currently executing phases.
    Executing,
    /// Wave completed all phases successfully.
    Completed,
    /// Wave failed during execution of a phase.
    Failed,
}

/// A classification of a failure that occurred during a wave.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualClass {
    /// Unique identifier for the failure class.
    pub code: String,
    /// Detailed description of the failure.
    pub description: String,
    /// Severity of the failure.
    pub severity: Severity,
}

/// A phase within a wave, consisting of parallel tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WavePhase {
    /// Name of the phase.
    pub name: String,
    /// Tasks to be executed in parallel during this phase.
    pub tasks: Vec<TaskRequest>,
}

/// A wave is a sequence of phases executed sequentially.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wave {
    /// Unique wave identifier.
    pub id: String,
    /// Phases to be executed sequentially.
    pub phases: Vec<WavePhase>,
    /// Current execution status.
    pub status: WaveStatus,
    /// Metadata for the wave.
    pub metadata: HashMap<String, String>,
}

/// Proof of wave completion, composed of task receipts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveReceipt {
    /// Wave identifier.
    pub wave_id: String,
    /// Receipts for each phase.
    pub phase_receipts: Vec<PhaseReceipt>,
    /// Final status of the wave.
    pub status: WaveStatus,
    /// Failure classifications if any.
    pub residual_classes: Vec<ResidualClass>,
    /// Total execution time in milliseconds.
    pub total_execution_time_ms: u64,
}

/// Proof of phase completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseReceipt {
    /// Phase name.
    pub phase_name: String,
    /// Receipts for tasks executed in this phase.
    pub task_receipts: Vec<TaskReceipt>,
}

impl Wave {
    /// Create a new wave.
    #[must_use]
    pub fn new(id: String) -> Self {
        Self { id, phases: Vec::new(), status: WaveStatus::Queued, metadata: HashMap::new() }
    }

    /// Add a phase to the wave.
    pub fn add_phase(&mut self, name: String, tasks: Vec<TaskRequest>) {
        self.phases.push(WavePhase { name, tasks });
    }

    /// Execute the wave (mock implementation for orchestrator).
    /// In a real system, this would be handled by the `SwarmCoordinator`.
    pub async fn execute<F, Fut>(&mut self, mut executor: F) -> WaveReceipt
    where
        F: FnMut(TaskRequest) -> Fut,
        Fut: std::future::Future<Output = TaskReceipt>,
    {
        self.status = WaveStatus::Executing;
        let mut phase_receipts = Vec::new();
        let mut total_time = 0;
        let mut failed = false;
        let mut residual_classes = Vec::new();

        for phase in &self.phases {
            let mut task_receipts = Vec::new();

            // Execute tasks in parallel (M parallel tasks)
            let mut futures = Vec::new();
            for task in phase.tasks.clone() {
                futures.push(executor(task));
            }

            let results = futures::future::join_all(futures).await;

            for receipt in results {
                total_time += receipt.execution_time_ms;
                if !receipt.is_success() {
                    failed = true;
                    residual_classes.push(ResidualClass {
                        code: format!("TASK_FAIL_{}", receipt.task_id),
                        description: receipt.result.clone(),
                        severity: Severity::Andon,
                    });
                }
                task_receipts.push(receipt);
            }

            phase_receipts.push(PhaseReceipt { phase_name: phase.name.clone(), task_receipts });

            if failed {
                self.status = WaveStatus::Failed;
                break;
            }
        }

        if !failed {
            self.status = WaveStatus::Completed;
        }

        WaveReceipt {
            wave_id: self.id.clone(),
            phase_receipts,
            status: self.status,
            residual_classes,
            total_execution_time_ms: total_time,
        }
    }
}

impl WaveReceipt {
    /// Check if the wave was successful.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status == WaveStatus::Completed
    }

    /// Returns the number of successful tasks.
    #[must_use]
    pub fn success_count(&self) -> usize {
        self.phase_receipts
            .iter()
            .flat_map(|p| &p.task_receipts)
            .filter(|t| t.is_success())
            .count()
    }

    /// Returns the total number of tasks.
    #[must_use]
    pub fn total_tasks(&self) -> usize {
        self.phase_receipts.iter().flat_map(|p| &p.task_receipts).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swarm::task::TaskStatus;

    fn request(id: &str) -> TaskRequest {
        TaskRequest::new(id.to_string(), "Academic".to_string(), "op".to_string(), "in".to_string())
    }

    fn receipt(task_id: &str, status: TaskStatus) -> TaskReceipt {
        TaskReceipt::new(
            task_id.to_string(),
            "agent-1".to_string(),
            vec!["Academic".to_string()],
            status,
            "result".to_string(),
        )
        .with_execution_time(7)
    }

    /// Deterministic async runtime: block_on is sufficient here because the
    /// executor futures are immediately-resolved closures (no sleeps, no I/O).
    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        futures::executor::block_on(fut)
    }

    #[test]
    fn test_execute_happy_path_all_tasks_pass() {
        let mut wave = Wave::new("wave-1".to_string());
        wave.add_phase("phase-a".to_string(), vec![request("t1"), request("t2")]);
        wave.add_phase("phase-b".to_string(), vec![request("t3")]);

        let r = block_on(wave.execute(|task| async move {
            receipt(&task.id, TaskStatus::Completed)
        }));

        assert_eq!(wave.status, WaveStatus::Completed);
        assert!(r.is_success());
        assert_eq!(r.wave_id, "wave-1");
        assert_eq!(r.phase_receipts.len(), 2);
        assert_eq!(r.phase_receipts[0].phase_name, "phase-a");
        assert_eq!(r.residual_classes.len(), 0);
        assert_eq!(r.success_count(), 3);
        assert_eq!(r.total_tasks(), 3);
        assert_eq!(r.total_execution_time_ms, 21);
    }

    #[test]
    fn test_execute_fail_fast_stops_subsequent_phases() {
        let mut wave = Wave::new("wave-2".to_string());
        wave.add_phase("phase-a".to_string(), vec![request("ok-1"), request("bad-1")]);
        wave.add_phase("phase-b".to_string(), vec![request("never-run")]);

        let r = block_on(wave.execute(|task| async move {
            if task.id == "bad-1" {
                receipt("bad-1", TaskStatus::Failed)
            } else {
                receipt(&task.id, TaskStatus::Completed)
            }
        }));

        assert_eq!(wave.status, WaveStatus::Failed);
        assert!(!r.is_success());
        assert_eq!(r.status, WaveStatus::Failed);
        // Second phase never executed.
        assert_eq!(r.phase_receipts.len(), 1);
        // Both tasks of the failing phase recorded; failing one classified.
        assert_eq!(r.total_tasks(), 2);
        assert_eq!(r.success_count(), 1);
        assert_eq!(r.residual_classes.len(), 1);
        let residual = &r.residual_classes[0];
        assert_eq!(residual.code, "TASK_FAIL_bad-1");
        assert_eq!(residual.severity, Severity::Andon);
        assert!(!residual.description.is_empty());
    }

    #[test]
    fn test_accessors_and_status_lifecycle() {
        let mut wave = Wave::new("wave-3".to_string());
        assert_eq!(wave.status, WaveStatus::Queued);
        assert_eq!(wave.id, "wave-3");

        wave.add_phase("only".to_string(), vec![request("t1")]);
        assert_eq!(wave.phases.len(), 1);

        let r = block_on(wave.execute(|task| async move {
            receipt(&task.id, TaskStatus::Completed)
        }));

        assert_eq!(wave.status, WaveStatus::Completed);
        assert_eq!(r.success_count(), r.total_tasks());
        assert_eq!(r.total_tasks(), 1);
    }
}
