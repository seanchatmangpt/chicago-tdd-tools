use super::types::{Part, Task, TaskState};

/// Assert that a task has reached the `Completed` state.
///
/// # Panics
///
/// Panics if the task state is not [`TaskState::Completed`].
pub fn assert_task_completed(task: &Task) {
    assert_eq!(
        task.state,
        TaskState::Completed,
        "expected task {} to be Completed, got {:?}",
        task.id,
        task.state
    );
}

/// Assert that a task is in a specific state.
///
/// # Panics
///
/// Panics if the task state does not equal `expected`.
pub fn assert_task_state(task: &Task, expected: TaskState) {
    assert_eq!(
        task.state, expected,
        "task {} state mismatch: expected {:?}, got {:?}",
        task.id, expected, task.state
    );
}

/// Assert that at least one message part in the task contains `substring`.
///
/// # Panics
///
/// Panics if no text part in any message contains the substring.
pub fn assert_task_text_content(task: &Task, substring: &str) {
    let found = task
        .messages
        .iter()
        .any(|m| m.parts.iter().any(|p| matches!(p, Part::Text(t) if t.text.contains(substring))));
    assert!(
        found,
        "expected task {} to contain text {substring:?}, messages: {:?}",
        task.id, task.messages
    );
}

/// Assert that a task has failed (state is [`TaskState::Failed`]).
///
/// # Panics
///
/// Panics if the task state is not `Failed`.
pub fn assert_task_failed(task: &Task) {
    assert_eq!(
        task.state,
        TaskState::Failed,
        "expected task {} to be Failed, got {:?}",
        task.id,
        task.state
    );
}
