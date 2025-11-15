//! State Examples
//!
//! Demonstrates type-level AAA pattern enforcement with TestState, including advanced patterns.

use chicago_tdd_tools::core::state::{Act, Arrange, Assert, TestState};
use chicago_tdd_tools::prelude::*;

/// Example: Type state pattern
pub fn example_type_state_pattern() {
    // Arrange: Start with Arrange phase (type system enforces order)
    let arrange_state = TestState::<Arrange>::new().with_arrange_data(vec![1, 2, 3]);

    // Act: Transition to Act phase (only possible from Arrange)
    let act_state = arrange_state.act();
    let act_state = act_state.execute(|data| {
        let mut result = data.unwrap_or_default();
        result.push(4);
        result
    });

    // Assert: Transition to Assert phase (only possible from Act)
    let assert_state = act_state.assert();
    assert!(assert_state.assert_that(|result| { result.map(|r| r.len() == 4).unwrap_or(false) }));

    // Type system prevents calling methods in wrong order:
    // - Cannot call `act()` on `TestState<Assert>`
    // - Cannot call `assert()` on `TestState<Arrange>`
    // - Cannot create `TestState<Act>` directly
}

/// Example: Advanced state pattern with multiple transitions
pub fn example_advanced_state_pattern() {
    // Arrange: Start with Arrange phase
    let arrange_state = TestState::<Arrange>::new().with_arrange_data(vec![10, 20, 30]);

    // Act: Transition to Act phase and execute multiple operations
    let act_state = arrange_state.act();

    // First operation: Transform data
    let act_state = act_state.execute(|data| {
        let mut result = data.unwrap_or_default();
        result.iter_mut().for_each(|v| *v *= 2);
        result
    });

    // Second operation: Filter data
    let act_state = act_state
        .execute(|data| data.unwrap_or_default().into_iter().filter(|&v| v > 30).collect());

    // Assert: Transition to Assert phase and verify complex conditions
    let assert_state = act_state.assert();

    // Verify result length
    assert!(assert_state
        .assert_that(|result| { result.as_ref().map(|r| r.len() == 2).unwrap_or(false) }));

    // Verify result values
    assert!(assert_state.assert_that(|result| {
        result.as_ref().map(|r| r.iter().all(|&v| v > 30)).unwrap_or(false)
    }));
}

/// Example: State pattern with no arrange data
pub fn example_state_no_arrange() {
    // Arrange: Start with Arrange phase without data
    let arrange_state = TestState::<Arrange>::new();

    // Act: Transition to Act phase and generate data
    let act_state = arrange_state.act();
    let act_state = act_state.execute(|_data| {
        // Generate data in Act phase
        vec![1, 2, 3, 4, 5]
    });

    // Assert: Transition to Assert phase and verify
    let assert_state = act_state.assert();
    assert!(assert_state
        .assert_that(|result| { result.as_ref().map(|r| r.len() == 5).unwrap_or(false) }));
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_type_state_pattern, {
        // Arrange-Act-Assert: Run example
        example_type_state_pattern();
    });

    test!(test_advanced_state_pattern, {
        // Arrange-Act-Assert: Run example
        example_advanced_state_pattern();
    });

    test!(test_state_no_arrange, {
        // Arrange-Act-Assert: Run example
        example_state_no_arrange();
    });
}
