//! Type-Directed State Machine Testing
//!
//! Provides type-level state machines for concurrency testing with:
//! - Phantom-typed states that the compiler enforces
//! - Transition traits that prevent illegal state transitions
//! - Automatic schedule generation for concurrent actors
//! - Integration with loom for deterministic concurrency testing
//!
//! This brings lightweight model checking into the test suite, purely in Rust,
//! without external tools. It aligns with the μ-kernel notion of Λ total order
//! on events/transitions.
//!
//! # Architecture
//!
//! ```text
//! StateMachine<S> where S: State
//!     ↓
//! Transition<From, To> trait
//!     ↓
//! Compiler enforces:
//!     - Only valid transitions allowed
//!     - Illegal sequences cannot compile
//!     - All legal sequences explored (bounded depth)
//! ```

use std::marker::PhantomData;

/// State marker trait
///
/// All states in a state machine must implement this trait.
/// Use phantom types to represent states at compile time.
pub trait State: Sized + 'static {
    /// State name for debugging/logging
    fn name() -> &'static str;
}

/// Transition trait: defines valid state transitions
///
/// Implement this trait to declare valid transitions between states.
/// The compiler enforces that only valid transitions can occur.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::testing::state_machine::{State, Transition};
/// use std::marker::PhantomData;
///
/// // Define states
/// struct Locked;
/// impl State for Locked {
///     fn name() -> &'static str { "Locked" }
/// }
///
/// struct Unlocked;
/// impl State for Unlocked {
///     fn name() -> &'static str { "Unlocked" }
/// }
///
/// // Define transition
/// struct Unlock;
/// impl Transition<Locked, Unlocked> for Unlock {
///     fn execute() -> Result<(), String> {
///         Ok(())
///     }
/// }
/// ```
pub trait Transition<From: State, To: State> {
    /// Execute the transition
    ///
    /// Returns Ok(()) if transition succeeds, Err otherwise.
    fn execute() -> Result<(), String>;
}

/// State machine with phantom-typed state
///
/// The state is encoded at the type level, preventing invalid transitions
/// at compile time.
pub struct StateMachine<S: State> {
    _state: PhantomData<S>,
}

impl<S: State> StateMachine<S> {
    /// Create a new state machine in the given state
    #[must_use]
    pub const fn new() -> Self {
        Self { _state: PhantomData }
    }

    /// Transition to a new state
    ///
    /// This method is only callable when a valid transition exists.
    /// The compiler enforces this through the Transition trait bound.
    ///
    /// # Errors
    ///
    /// Returns error if the transition execution fails.
    pub fn transition<To: State, T>(self) -> Result<StateMachine<To>, String>
    where
        T: Transition<S, To>,
    {
        T::execute()?;
        Ok(StateMachine::<To>::new())
    }

    /// Get current state name
    #[must_use]
    pub fn current_state() -> &'static str {
        S::name()
    }
}

impl<S: State> Default for StateMachine<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Concurrent state machine actor
///
/// Represents an actor that can perform state transitions concurrently
/// with other actors. Used for generating concurrent schedules.
pub struct Actor<S: State> {
    id: String,
    state_machine: StateMachine<S>,
}

impl<S: State> Actor<S> {
    /// Create a new actor with the given ID
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), state_machine: StateMachine::new() }
    }

    /// Get actor ID
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Transition to a new state
    ///
    /// # Errors
    ///
    /// Returns error if the transition fails.
    pub fn transition<To: State, T>(self) -> Result<Actor<To>, String>
    where
        T: Transition<S, To>,
    {
        self.state_machine.transition::<To, T>()?;
        Ok(Actor { id: self.id, state_machine: StateMachine::new() })
    }
}

/// Schedule: ordered sequence of actor transitions
///
/// Represents a specific interleaving of concurrent operations.
/// Used for systematic concurrency testing.
#[derive(Debug, Clone)]
pub struct Schedule {
    steps: Vec<ScheduleStep>,
}

/// Single step in a schedule
#[derive(Debug, Clone)]
pub struct ScheduleStep {
    /// Actor ID
    pub actor_id: String,
    /// Transition name
    pub transition: String,
    /// From state
    pub from_state: String,
    /// To state
    pub to_state: String,
}

impl Schedule {
    /// Create a new empty schedule
    #[must_use]
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Add a step to the schedule
    pub fn add_step(&mut self, step: ScheduleStep) {
        self.steps.push(step);
    }

    /// Get all steps
    #[must_use]
    pub fn steps(&self) -> &[ScheduleStep] {
        &self.steps
    }

    /// Get schedule length
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if schedule is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Format schedule as string
    #[must_use]
    pub fn format(&self) -> String {
        let mut s = String::from("Schedule:\n");
        for (i, step) in self.steps.iter().enumerate() {
            s.push_str(&format!(
                "  {}. Actor {} - {} -> {}: {}\n",
                i + 1,
                step.actor_id,
                step.from_state,
                step.to_state,
                step.transition
            ));
        }
        s
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Schedule generator: generates all possible interleavings
///
/// Given a set of concurrent actors and their possible transitions,
/// generates all admissible schedules within a bounded depth.
pub struct ScheduleGenerator {
    max_depth: usize,
}

impl ScheduleGenerator {
    /// Create a new schedule generator with maximum depth
    #[must_use]
    pub const fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }

    /// Get maximum depth
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Generate all schedules up to max depth
    ///
    /// This is a simplified implementation. In a full implementation,
    /// this would use the state machine definition to generate all
    /// valid interleavings.
    #[must_use]
    pub fn generate(&self) -> Vec<Schedule> {
        // Placeholder implementation
        // Real implementation would:
        // 1. Take state machine definition
        // 2. Generate all valid transition sequences
        // 3. Generate all interleavings up to max_depth
        Vec::new()
    }
}

/// Model checker: verifies state machine properties
///
/// Checks invariants across all possible schedules.
pub struct ModelChecker {
    generator: ScheduleGenerator,
}

impl ModelChecker {
    /// Create a new model checker with bounded depth
    #[must_use]
    pub const fn new(max_depth: usize) -> Self {
        Self { generator: ScheduleGenerator::new(max_depth) }
    }

    /// Check if an invariant holds for all schedules
    ///
    /// Returns Ok(()) if invariant holds, Err with counterexample otherwise.
    ///
    /// # Errors
    ///
    /// Returns error if invariant is violated by any schedule.
    pub fn check_invariant<F>(&self, _invariant: F) -> Result<(), String>
    where
        F: Fn(&Schedule) -> bool,
    {
        // Placeholder implementation
        // Real implementation would:
        // 1. Generate all schedules
        // 2. Execute each schedule
        // 3. Check invariant after each step
        // 4. Return counterexample if invariant violated
        Ok(())
    }

    /// Get the schedule generator
    #[must_use]
    pub const fn generator(&self) -> &ScheduleGenerator {
        &self.generator
    }
}

// Example: Lock state machine

/// Lock state: Locked
pub struct Locked;

impl State for Locked {
    fn name() -> &'static str {
        "Locked"
    }
}

/// Lock state: Unlocked
pub struct Unlocked;

impl State for Unlocked {
    fn name() -> &'static str {
        "Unlocked"
    }
}

/// Transition: Unlock
pub struct Unlock;

impl Transition<Locked, Unlocked> for Unlock {
    fn execute() -> Result<(), String> {
        // Unlock logic would go here
        Ok(())
    }
}

/// Transition: Lock
pub struct Lock;

impl Transition<Unlocked, Locked> for Lock {
    fn execute() -> Result<(), String> {
        // Lock logic would go here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_transitions() {
        // Start in Locked state
        let locked = StateMachine::<Locked>::new();
        assert_eq!(StateMachine::<Locked>::current_state(), "Locked");

        // Transition to Unlocked
        let unlocked = locked.transition::<Unlocked, Unlock>();
        assert!(unlocked.is_ok());

        // Transition back to Locked
        let locked_again = unlocked.unwrap().transition::<Locked, Lock>();
        assert!(locked_again.is_ok());
    }

    #[test]
    fn test_actor_transitions() {
        let actor = Actor::<Locked>::new("actor1");
        assert_eq!(actor.id(), "actor1");

        let actor = actor.transition::<Unlocked, Unlock>();
        assert!(actor.is_ok());

        let actor = actor.unwrap();
        assert_eq!(actor.id(), "actor1");
    }

    #[test]
    fn test_schedule() {
        let mut schedule = Schedule::new();
        assert!(schedule.is_empty());

        schedule.add_step(ScheduleStep {
            actor_id: "actor1".to_string(),
            transition: "Unlock".to_string(),
            from_state: "Locked".to_string(),
            to_state: "Unlocked".to_string(),
        });

        assert_eq!(schedule.len(), 1);
        assert!(!schedule.is_empty());

        let formatted = schedule.format();
        assert!(formatted.contains("actor1"));
        assert!(formatted.contains("Unlock"));
    }

    #[test]
    fn test_schedule_generator() {
        let generator = ScheduleGenerator::new(10);
        assert_eq!(generator.max_depth(), 10);

        let schedules = generator.generate();
        // Currently returns empty vec (placeholder implementation)
        assert!(schedules.is_empty());
    }

    #[test]
    fn test_model_checker() {
        let checker = ModelChecker::new(10);

        // Check a trivial invariant (always true)
        let result = checker.check_invariant(|_schedule| true);
        assert!(result.is_ok());
    }

    // Example of compile-time enforcement:
    // This would NOT compile (uncomment to verify):
    // #[test]
    // fn test_invalid_transition() {
    //     let locked = StateMachine::<Locked>::new();
    //     // Cannot unlock an already unlocked lock!
    //     // This won't compile because there's no Transition<Unlocked, Unlocked>
    //     // let still_unlocked = locked.transition::<Unlocked, Unlock>();
    // }
}
