//! Type-Level Programming for Test State
//!
//! Implements type state pattern for compile-time test lifecycle guarantees.
//! Ensures AAA pattern is enforced at compile time - impossible to call Act before Arrange.

/// Marker type for Arrange phase
pub struct Arrange;

/// Marker type for Act phase
pub struct Act;

/// Marker type for Assert phase
pub struct Assert;

/// Test state with type-level phase tracking
///
/// This type ensures that test phases are followed in the correct order:
/// Arrange -> Act -> Assert
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::state::{TestState, Arrange, Act, Assert};
///
/// // Start with Arrange phase
/// let arrange_state = TestState::<Arrange>::new();
///
/// // Transition to Act phase
/// let act_state = arrange_state.act();
///
/// // Transition to Assert phase
/// let assert_state = act_state.assert();
/// ```
pub struct TestState<Phase> {
    /// Phase marker (zero-sized type)
    _phase: std::marker::PhantomData<Phase>,
    /// Test data (can be extended)
    data: TestData,
}

/// Test data container
#[derive(Default)]
struct TestData {
    /// Arrange data
    arrange_data: Option<Vec<u8>>,
    /// Act result
    act_result: Option<Vec<u8>>,
}

impl TestState<Arrange> {
    /// Create a new test state in Arrange phase
    pub fn new() -> Self {
        Self {
            _phase: std::marker::PhantomData,
            data: TestData::default(),
        }
    }

    /// Add arrange data
    pub fn with_arrange_data(mut self, data: Vec<u8>) -> Self {
        self.data.arrange_data = Some(data);
        self
    }

    /// Transition to Act phase
    ///
    /// This consumes the Arrange state and returns an Act state.
    /// This ensures that Act can only be called after Arrange.
    pub fn act(self) -> TestState<Act> {
        TestState {
            _phase: std::marker::PhantomData,
            data: self.data,
        }
    }
}

impl TestState<Act> {
    /// Execute act operation
    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Option<Vec<u8>>) -> Vec<u8>,
    {
        let result = f(self.data.arrange_data.take());
        self.data.act_result = Some(result);
        self
    }

    /// Transition to Assert phase
    ///
    /// This consumes the Act state and returns an Assert state.
    /// This ensures that Assert can only be called after Act.
    pub fn assert(self) -> TestState<Assert> {
        TestState {
            _phase: std::marker::PhantomData,
            data: self.data,
        }
    }
}

impl TestState<Assert> {
    /// Get act result for assertion
    pub fn act_result(&self) -> Option<&Vec<u8>> {
        self.data.act_result.as_ref()
    }

    /// Get arrange data for assertion
    pub fn arrange_data(&self) -> Option<&Vec<u8>> {
        self.data.arrange_data.as_ref()
    }

    /// Assert with predicate
    pub fn assert_that<F>(&self, predicate: F) -> bool
    where
        F: FnOnce(Option<&Vec<u8>>) -> bool,
    {
        predicate(self.act_result())
    }
}

impl Default for TestState<Arrange> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_lifecycle() {
        // Arrange
        let arrange_state = TestState::<Arrange>::new().with_arrange_data(vec![1, 2, 3]);

        // Act
        let act_state = arrange_state.act();
        let act_state = act_state.execute(|data| {
            let mut result = data.unwrap_or_default();
            result.push(4);
            result
        });

        // Assert
        let assert_state = act_state.assert();
        assert!(
            assert_state.assert_that(|result| { result.map(|r| r.len() == 4).unwrap_or(false) })
        );
    }

    #[test]
    fn test_state_prevents_wrong_order() {
        // This test demonstrates that the type system prevents calling
        // methods in the wrong order. The following would not compile:
        //
        // let state = TestState::<Arrange>::new();
        // state.assert(); // ERROR: assert() not available on Arrange state
        //
        // let state = TestState::<Act>::new(); // ERROR: cannot create Act state directly
        //
        // This is the desired behavior - compile-time safety!
    }
}
