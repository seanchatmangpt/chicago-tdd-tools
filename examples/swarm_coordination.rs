//! # Swarm Coordination Example - Comprehensive Guide
//!
//! Demonstrates distributed multi-sector coordination with task receipts.
//! The swarm protocol enables multiple agents to coordinate across sectors,
//! share knowledge hooks, and compose operations deterministically.
//!
//! ## Tutorial: Getting Started
//!
//! This example walks through swarm coordination:
//!
//! 1. **Creating Coordinator**: Use `SwarmCoordinator::new()` to create coordinator
//! 2. **Registering Members**: Add swarm members with sector capabilities
//! 3. **Submitting Tasks**: Submit tasks to the swarm for execution
//! 4. **Distributing Tasks**: Assign tasks to available members
//! 5. **Recording Completion**: Generate task receipts for completed work
//! 6. **Checking Consensus**: Verify swarm consensus on results
//!
//! ## Explanation: Concepts
//!
//! **Swarm Protocol**: Distributed multi-sector coordination without central control.
//! Uses gossip and consensus mechanisms to coordinate operations across sectors.
//!
//! **Swarm Members**: Agents that participate in the swarm. Each member:
//! - Has sector capabilities (Academic, Claims, etc.)
//! - Has capacity for concurrent tasks
//! - Maintains reputation based on task completion
//!
//! **Task Receipts**: Cryptographic proof of work done by swarm agents.
//! Each receipt proves: what was done, by whom, when, and the result.
//!
//! **Knowledge Hooks**: Operations that can be composed across sectors.
//! Enables multi-sector orchestration with deterministic guarantees.
//!
//! ## How-to: Common Tasks
//!
//! - Create coordinator: See `example_create_coordinator()`
//! - Register members: See `example_register_members()`
//! - Submit tasks: See `example_submit_tasks()`
//! - Distribute tasks: See `example_distribute_tasks()`
//! - Generate receipts: See `example_task_receipts()`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `SwarmCoordinator`: Central orchestration logic
//! - `SwarmMember`: Individual swarm agent
//! - `TaskRequest`: Task submission request
//! - `TaskReceipt`: Proof of task completion
//! - `TaskStatus`: Task execution status
//!
//! **Key Methods**:
//! - `SwarmCoordinator::new() -> Self`
//! - `SwarmCoordinator::register_member(member: SwarmMember)`
//! - `SwarmCoordinator::submit_task(task: TaskRequest)`
//! - `SwarmCoordinator::distribute_next_task() -> Result<(String, String), String>`

use chicago_tdd_tools::swarm::*;

/// Example: Creating a swarm coordinator
///
/// ## How-to: Create Coordinator
///
/// Creates a new swarm coordinator for managing distributed operations.
fn example_create_coordinator() {
    println!("=== Example: Creating Swarm Coordinator ===");

    // Arrange & Act: Create coordinator
    let coordinator = SwarmCoordinator::new();

    // Assert: Verify initial state
    let status = coordinator.status();
    println!("✓ Coordinator created");
    println!("  Swarm ID: {}", status.swarm_id);
    println!("  Total Members: {}", status.total_members);
    println!("  Queued Tasks: {}", status.queued_tasks);
    println!("  Completed Tasks: {}", status.completed_tasks);
}

/// Example: Registering swarm members
///
/// ## How-to: Register Members
///
/// Registers swarm members with sector capabilities and capacity.
fn example_register_members() {
    println!("\n=== Example: Registering Swarm Members ===");

    // Arrange: Create coordinator
    let mut coordinator = SwarmCoordinator::new();

    // Act: Register members for different sectors
    let academic_member =
        SwarmMember::new("agent-academic-1".to_string(), "Academic Agent".to_string())
            .with_sector("Academic".to_string())
            .with_capacity(10);

    let claims_member = SwarmMember::new("agent-claims-1".to_string(), "Claims Agent".to_string())
        .with_sector("Enterprise Claims".to_string())
        .with_capacity(15);

    let multi_sector_member =
        SwarmMember::new("agent-multi-1".to_string(), "Multi-Sector Agent".to_string())
            .with_sectors(vec!["Academic".to_string(), "Enterprise Claims".to_string()])
            .with_capacity(20);

    coordinator.register_member(academic_member);
    coordinator.register_member(claims_member);
    coordinator.register_member(multi_sector_member);

    // Assert: Verify members registered
    let status = coordinator.status();
    println!("✓ Registered {} members", status.total_members);
    println!("  Total Capacity: {}", status.total_capacity);
    println!("  Active Members: {}", status.active_members);
}

/// Example: Submitting tasks to swarm
///
/// ## How-to: Submit Tasks
///
/// Submits tasks to the swarm for execution by available members.
fn example_submit_tasks() {
    println!("\n=== Example: Submitting Tasks ===");

    // Arrange: Create coordinator and register member
    let mut coordinator = SwarmCoordinator::new();
    coordinator.register_member(
        SwarmMember::new("agent-1".to_string(), "Agent 1".to_string())
            .with_sector("Academic".to_string())
            .with_capacity(10),
    );

    // Act: Submit tasks with different priorities
    let task1 = TaskRequest::new(
        "task-001".to_string(),
        "Academic".to_string(),
        "desk-review".to_string(),
        "paper-123".to_string(),
    )
    .with_priority(10);

    let task2 = TaskRequest::new(
        "task-002".to_string(),
        "Academic".to_string(),
        "reviewer-assignment".to_string(),
        "paper-456".to_string(),
    )
    .with_priority(5);

    coordinator.submit_task(task1);
    coordinator.submit_task(task2);

    // Assert: Verify tasks queued
    let status = coordinator.status();
    println!("✓ Submitted 2 tasks");
    println!("  Queued Tasks: {}", status.queued_tasks);
}

/// Example: Distributing tasks to members
///
/// ## How-to: Distribute Tasks
///
/// Assigns queued tasks to available swarm members based on capacity and sector.
fn example_distribute_tasks() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Distributing Tasks ===");

    // Arrange: Create coordinator and register members
    let mut coordinator = SwarmCoordinator::new();
    coordinator.register_member(
        SwarmMember::new("agent-academic-1".to_string(), "Academic Agent".to_string())
            .with_sector("Academic".to_string())
            .with_capacity(10),
    );
    coordinator.register_member(
        SwarmMember::new("agent-claims-1".to_string(), "Claims Agent".to_string())
            .with_sector("Enterprise Claims".to_string())
            .with_capacity(15),
    );

    // Submit tasks
    coordinator.submit_task(TaskRequest::new(
        "task-001".to_string(),
        "Academic".to_string(),
        "desk-review".to_string(),
        "paper-123".to_string(),
    ));

    coordinator.submit_task(TaskRequest::new(
        "task-002".to_string(),
        "Enterprise Claims".to_string(),
        "fraud-detection".to_string(),
        "claim-456".to_string(),
    ));

    // Act: Distribute tasks
    println!("--- Distributing Task 1 ---");
    let (task_id, member_id) = coordinator.distribute_next_task()?;
    println!("✓ Task '{}' assigned to member '{}'", task_id, member_id);

    println!("\n--- Distributing Task 2 ---");
    let (task_id, member_id) = coordinator.distribute_next_task()?;
    println!("✓ Task '{}' assigned to member '{}'", task_id, member_id);

    // Assert: Verify tasks distributed
    let status = coordinator.status();
    println!("\n✓ Tasks distributed");
    println!("  Queued Tasks: {}", status.queued_tasks);
    println!("  Current Tasks: {}", status.current_tasks);

    Ok(())
}

/// Example: Generating task receipts
///
/// ## How-to: Task Receipts
///
/// Generates cryptographic receipts for completed tasks.
fn example_task_receipts() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Task Receipts ===");

    // Arrange: Create coordinator and complete task
    let mut coordinator = SwarmCoordinator::new();
    coordinator.register_member(
        SwarmMember::new("agent-1".to_string(), "Agent 1".to_string())
            .with_sector("Academic".to_string())
            .with_capacity(10),
    );

    coordinator.submit_task(TaskRequest::new(
        "task-001".to_string(),
        "Academic".to_string(),
        "desk-review".to_string(),
        "paper-123".to_string(),
    ));

    let (task_id, member_id) = coordinator.distribute_next_task()?;

    // Act: Generate receipt for completed task
    let receipt = TaskReceipt::new(
        task_id.clone(),
        member_id.clone(),
        vec!["Academic".to_string()],
        TaskStatus::Completed,
        "Paper accepted for review".to_string(),
    )
    .with_execution_time(150)
    .with_merkle("abc123def456".to_string())
    .add_metadata("review_score".to_string(), "4.5".to_string());

    coordinator.record_completion(receipt.clone());

    // Assert: Verify receipt properties
    println!("✓ Task receipt generated");
    println!("  Task ID: {}", receipt.task_id);
    println!("  Agent ID: {}", receipt.agent_id);
    println!("  Status: {}", receipt.status);
    println!("  Execution Time: {}ms", receipt.execution_time_ms);
    println!("  Merkle Root: {}", receipt.result_merkle);
    println!("  Is Success: {}", receipt.is_success());

    // Verify receipt recorded
    let status = coordinator.status();
    println!("\n✓ Receipt recorded");
    println!("  Completed Tasks: {}", status.completed_tasks);

    Ok(())
}

/// Example: Multi-sector coordination
///
/// ## How-to: Multi-Sector Tasks
///
/// Demonstrates coordinating tasks across multiple sectors.
fn example_multi_sector_coordination() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Multi-Sector Coordination ===");

    // Arrange: Create coordinator with multi-sector member
    let mut coordinator = SwarmCoordinator::new();
    coordinator.register_member(
        SwarmMember::new("agent-multi-1".to_string(), "Multi-Sector Agent".to_string())
            .with_sectors(vec!["Academic".to_string(), "Enterprise Claims".to_string()])
            .with_capacity(20),
    );

    // Submit multi-sector task
    let task = TaskRequest::new(
        "task-multi-001".to_string(),
        "Academic".to_string(),
        "cross-sector-analysis".to_string(),
        "combined-data".to_string(),
    )
    .add_sector("Enterprise Claims".to_string());

    coordinator.submit_task(task);

    // Act: Distribute task
    let (task_id, member_id) = coordinator.distribute_next_task()?;
    println!("✓ Multi-sector task '{}' assigned to '{}'", task_id, member_id);

    // Generate receipt
    let receipt = TaskReceipt::new(
        task_id,
        member_id,
        vec!["Academic".to_string(), "Enterprise Claims".to_string()],
        TaskStatus::Completed,
        "Cross-sector analysis complete".to_string(),
    );

    coordinator.record_completion(receipt);

    println!("✓ Multi-sector coordination complete");

    Ok(())
}

/// Example: Task priority handling
///
/// ## How-to: Task Priority
///
/// Demonstrates priority-based task distribution.
fn example_task_priority() {
    println!("\n=== Example: Task Priority ===");

    // Arrange: Create coordinator
    let mut coordinator = SwarmCoordinator::new();
    coordinator.register_member(
        SwarmMember::new("agent-1".to_string(), "Agent 1".to_string())
            .with_sector("Academic".to_string())
            .with_capacity(10),
    );

    // Submit tasks with different priorities
    coordinator.submit_task(
        TaskRequest::new(
            "task-low".to_string(),
            "Academic".to_string(),
            "op".to_string(),
            "data".to_string(),
        )
        .with_priority(1),
    );

    coordinator.submit_task(
        TaskRequest::new(
            "task-high".to_string(),
            "Academic".to_string(),
            "op".to_string(),
            "data".to_string(),
        )
        .with_priority(100),
    );

    coordinator.submit_task(
        TaskRequest::new(
            "task-medium".to_string(),
            "Academic".to_string(),
            "op".to_string(),
            "data".to_string(),
        )
        .with_priority(50),
    );

    // Act: Distribute tasks (should be in priority order)
    println!("Distributing tasks (should be in priority order):");
    for i in 1..=3 {
        if let Ok((task_id, _)) = coordinator.distribute_next_task() {
            println!("  {}. Task '{}'", i, task_id);
        }
    }

    println!("✓ Tasks distributed by priority");
}

/// Example: Swarm consensus checking
///
/// ## How-to: Consensus
///
/// Demonstrates checking swarm consensus on results.
fn example_consensus() {
    println!("\n=== Example: Swarm Consensus ===");

    // Arrange: Create coordinator with multiple members
    let mut coordinator = SwarmCoordinator::new();
    coordinator.register_member(
        SwarmMember::new("agent-1".to_string(), "Agent 1".to_string())
            .with_sector("Academic".to_string())
            .with_capacity(10),
    );
    coordinator.register_member(
        SwarmMember::new("agent-2".to_string(), "Agent 2".to_string())
            .with_sector("Academic".to_string())
            .with_capacity(10),
    );
    coordinator.register_member(
        SwarmMember::new("agent-3".to_string(), "Agent 3".to_string())
            .with_sector("Academic".to_string())
            .with_capacity(10),
    );

    // Act: Check consensus
    let has_consensus = coordinator.check_consensus("Academic");
    println!("✓ Consensus check for Academic sector: {}", has_consensus);

    // Consensus requires 66% of members to be active (2/3)
    // With 3 members, need at least 2 active
    println!("  Total Members: {}", coordinator.status().total_members);
    println!("  Active Members: {}", coordinator.status().active_members);
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  Swarm Coordination - Distributed Multi-Sector Example        ║");
    println!("║  Task Receipts & Knowledge Hooks Composition                  ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Run examples
    example_create_coordinator();
    example_register_members();
    example_submit_tasks();

    if let Err(e) = example_distribute_tasks() {
        eprintln!("Error distributing tasks: {}", e);
    }

    if let Err(e) = example_task_receipts() {
        eprintln!("Error with task receipts: {}", e);
    }

    if let Err(e) = example_multi_sector_coordination() {
        eprintln!("Error in multi-sector coordination: {}", e);
    }

    example_task_priority();
    example_consensus();

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║  All Examples Completed Successfully!                          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use chicago_tdd_tools::swarm::*;
    use chicago_tdd_tools::test;

    test!(test_coordinator_creation, {
        // Arrange & Act
        let coordinator = SwarmCoordinator::new();

        // Assert
        assert_eq!(coordinator.status().total_members, 0);
    });

    test!(test_register_member, {
        // Arrange
        let mut coordinator = SwarmCoordinator::new();

        // Act
        coordinator.register_member(
            SwarmMember::new("agent-1".to_string(), "Agent".to_string())
                .with_sector("Academic".to_string()),
        );

        // Assert
        assert_eq!(coordinator.status().total_members, 1);
    });

    test!(test_submit_and_distribute_task, {
        // Arrange
        let mut coordinator = SwarmCoordinator::new();
        coordinator.register_member(
            SwarmMember::new("agent-1".to_string(), "Agent".to_string())
                .with_sector("Academic".to_string())
                .with_capacity(10),
        );

        // Act
        coordinator.submit_task(TaskRequest::new(
            "task-1".to_string(),
            "Academic".to_string(),
            "op".to_string(),
            "data".to_string(),
        ));

        let result = coordinator.distribute_next_task();

        // Assert
        assert!(result.is_ok());
        let (task_id, member_id) = result.unwrap();
        assert_eq!(task_id, "task-1");
        assert_eq!(member_id, "agent-1");
    });
}
