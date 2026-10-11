//! Negative space: the Arrange -> Act transition is one-shot.
//!
//! `act()` consumes the Arrange state and yields `TestState<Act>`, which has
//! no `act()` method — the state machine cannot be run backwards or repeated.

use chicago_tdd_tools::state::{Arrange, TestState};

fn main() {
    let state = TestState::<Arrange>::new();
    // ERROR: no method `act` found for `TestState<Act>` (E0599)
    let _ = state.act().act();
}
