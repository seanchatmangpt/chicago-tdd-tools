//! Negative space of the AAA state machine: assertions cannot run before Act.
//!
//! There is no `assert()` method on `TestState<Arrange>` — only the lawful
//! Arrange -> Act -> Assert path exists as methods.

use chicago_tdd_tools::state::{Arrange, TestState};

fn main() {
    let state = TestState::<Arrange>::new();
    // ERROR: no method `assert` found for `TestState<Arrange>` (E0599)
    let _ = state.assert();
}
