# Swarm Coordination Example

**Category:** How-To Guide  
**Level:** Advanced  
**Prerequisites:** Understanding of distributed systems, task coordination  
**Features Required:** None

---

## Overview

This example demonstrates distributed multi-sector coordination with task receipts. The swarm protocol enables multiple agents to coordinate across sectors, share knowledge hooks, and compose operations deterministically.

**What you'll learn:**
- Creating and managing swarm coordinators
- Registering swarm members with sector capabilities
- Submitting and distributing tasks
- Generating cryptographic task receipts
- Multi-sector coordination patterns
- Consensus checking

---

## Quick Start

```bash
cargo run --example swarm_coordination
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools installed
- Understanding of distributed coordination

---

## Tutorial: Getting Started

### Step 1: Create Coordinator

Create a new swarm coordinator:

```rust
use chicago_tdd_tools::swarm::*;

let coordinator = SwarmCoordinator::new();
```

### Step 2: Register Members

Register swarm members with sector capabilities:

```rust
let mut coordinator = SwarmCoordinator::new();

coordinator.register_member(
    SwarmMember::new("agent-1".to_string(), "Agent 1".to_string())
        .with_sector("Academic".to_string())
        .with_capacity(10),
);
```

### Step 3: Submit Tasks

Submit tasks to the swarm:

```rust
coordinator.submit_task(
    TaskRequest::new(
        "task-001".to_string(),
        "Academic".to_string(),
        "desk-review".to_string(),
        "paper-123".to_string(),
    )
    .with_priority(10),
);
```

### Step 4: Distribute Tasks

Assign tasks to available members:

```rust
let (task_id, member_id) = coordinator.distribute_next_task()?;
```

### Step 5: Generate Receipts

Record task completion with receipts:

```rust
let receipt = TaskReceipt::new(
    task_id,
    member_id,
    vec!["Academic".to_string()],
    TaskStatus::Completed,
    "Result".to_string(),
);

coordinator.record_completion(receipt);
```

---

## How-To: Common Tasks

### Register Multi-Sector Member

```rust
coordinator.register_member(
    SwarmMember::new("agent-multi".to_string(), "Multi-Sector Agent".to_string())
        .with_sector("Academic".to_string())
        .add_sector("Enterprise Claims".to_string())
        .with_capacity(20),
);
```

### Submit Multi-Sector Task

```rust
let task = TaskRequest::new(
    "task-multi".to_string(),
    "Academic".to_string(),
    "operation".to_string(),
    "data".to_string(),
)
.add_sector("Enterprise Claims".to_string());

coordinator.submit_task(task);
```

### Task Priority

Tasks are distributed by priority (higher first):

```rust
coordinator.submit_task(
    TaskRequest::new(/* ... */).with_priority(100) // High priority
);

coordinator.submit_task(
    TaskRequest::new(/* ... */).with_priority(1)   // Low priority
);
```

### Generate Receipt with Metadata

```rust
let receipt = TaskReceipt::new(
    task_id,
    agent_id,
    sectors,
    TaskStatus::Completed,
    result,
)
.with_execution_time(150)
.with_merkle("abc123".to_string())
.add_metadata("key".to_string(), "value".to_string());
```

### Check Swarm Status

```rust
let status = coordinator.status();
println!("Members: {}", status.total_members);
println!("Capacity: {}", status.total_capacity);
println!("Queued: {}", status.queued_tasks);
println!("Completed: {}", status.completed_tasks);
```

### Check Consensus

```rust
let has_consensus = coordinator.check_consensus("Academic");
// Requires 66% of members to be active
```

---

## Explanation: Concepts

### Swarm Protocol

**Distributed Coordination**: Multiple agents coordinate across sectors without central control. Uses gossip and consensus mechanisms.

**No Central Control**: Swarm operates without single point of failure. Members coordinate through shared task queue and receipts.

### Swarm Members

Each member has:
- **ID**: Unique identifier
- **Sectors**: Capabilities (Academic, Claims, etc.)
- **Capacity**: Concurrent task limit
- **Reputation**: Based on task completion success
- **State**: Alive, Dead, etc.

### Task Receipts

Cryptographic proof of work:
- **Task ID**: Which task was executed
- **Agent ID**: Who executed it
- **Status**: Success/Failure
- **Merkle Root**: Cryptographic proof
- **Metadata**: Additional execution data

### Knowledge Hooks

Operations that can be composed across sectors:
- Enables multi-sector orchestration
- Deterministic guarantees
- Composable operations

### Consensus Mechanism

Requires 66% of members to agree:
- Byzantine fault tolerant
- Prevents single point of failure
- Ensures distributed agreement

---

## Reference: Quick Lookup

### SwarmCoordinator

**Creation:**
```rust
pub fn new() -> Self
```

**Methods:**
```rust
pub fn register_member(&mut self, member: SwarmMember)
pub fn submit_task(&mut self, task: TaskRequest)
pub fn distribute_next_task(&mut self) -> Result<(String, String), String>
pub fn record_completion(&mut self, receipt: TaskReceipt) -> Result<(), String>
pub fn check_consensus(&self, sector: &str) -> bool
pub fn status(&self) -> SwarmStatus
```

### SwarmMember

**Creation:**
```rust
pub fn new(id: String, name: String) -> Self
```

**Methods:**
```rust
pub fn with_sector(self, sector: String) -> Self
pub fn add_sector(self, sector: String) -> Self
pub fn with_capacity(self, capacity: u32) -> Self
pub fn can_handle(&self, sector: &str) -> bool
pub fn has_capacity(&self) -> bool
```

### TaskRequest

**Creation:**
```rust
pub fn new(id: String, sector: String, operation: String, input: String) -> Self
```

**Methods:**
```rust
pub fn with_priority(self, priority: u32) -> Self
pub fn add_sector(self, sector: String) -> Self
```

### TaskReceipt

**Creation:**
```rust
pub fn new(
    task_id: String,
    agent_id: String,
    sectors: Vec<String>,
    status: TaskStatus,
    result: String,
) -> Self
```

**Methods:**
```rust
pub fn with_execution_time(self, ms: u64) -> Self
pub fn with_merkle(self, merkle: String) -> Self
pub fn add_metadata(self, key: String, value: String) -> Self
pub fn is_success(&self) -> bool
```

### TaskStatus

```rust
pub enum TaskStatus {
    Queued,
    Executing,
    Completed,
    Failed,
    Cancelled,
}
```

### SwarmStatus

```rust
pub struct SwarmStatus {
    pub swarm_id: String,
    pub total_members: usize,
    pub active_members: usize,
    pub total_capacity: u32,
    pub current_tasks: u32,
    pub queued_tasks: usize,
    pub completed_tasks: usize,
}
```

---

## Common Patterns

### Complete Workflow

```rust
let mut coordinator = SwarmCoordinator::new();

// Register members
coordinator.register_member(/* ... */);

// Submit tasks
coordinator.submit_task(/* ... */);

// Distribute tasks
let (task_id, member_id) = coordinator.distribute_next_task()?;

// Generate receipt
let receipt = TaskReceipt::new(/* ... */);
coordinator.record_completion(receipt);
```

### Multi-Sector Coordination

```rust
// Register multi-sector member
coordinator.register_member(
    SwarmMember::new(/* ... */)
        .with_sector("Academic".to_string())
        .add_sector("Claims".to_string()),
);

// Submit multi-sector task
coordinator.submit_task(
    TaskRequest::new(/* ... */)
        .add_sector("Claims".to_string()),
);
```

### Priority-Based Distribution

```rust
// High priority first
coordinator.submit_task(task.with_priority(100));
coordinator.submit_task(task.with_priority(50));
coordinator.submit_task(task.with_priority(1));
```

---

## Troubleshooting

### No Available Members

**Error**: "No available members for task"

**Solution**: Register members with matching sector capabilities:
```rust
coordinator.register_member(
    SwarmMember::new(/* ... */)
        .with_sector("Academic".to_string())
        .with_capacity(10),
);
```

### Task Not Distributed

**Error**: "No tasks queued"

**Solution**: Submit tasks before distributing:
```rust
coordinator.submit_task(task);
let result = coordinator.distribute_next_task();
```

### Consensus Not Reached

**Error**: `check_consensus()` returns false

**Solution**: Ensure sufficient active members (66% threshold):
```rust
// With 3 members, need at least 2 active
coordinator.register_member(/* ... */);
coordinator.register_member(/* ... */);
coordinator.register_member(/* ... */);
```

---

## Related Documentation

- **Swarm Module**: `src/swarm/`
- **Coordinator**: `src/swarm/coordinator.rs`
- **Task System**: `src/swarm/task.rs`
- **Release Notes**: `docs/releases/RELEASE_NOTES_v1.4.0.md`

---

## See Also

- [Sector Stacks Workflows](sector_stacks_workflows.md) - Production-grade workflows
- [Fail-Fast Verification](fail_fast_verification.md) - 12-phase verification
- [RDF Validation](rdf_validation.md) - RDF-driven validation

---

**Quality is the default. Prevention beats detection.**

*Version 1.4.0 | Updated 2025-01-XX | Team KNHK | License MIT*

