//! Negative space: the Act-phase operation cannot be executed from Assert.
//!
//! `execute()` exists only on `TestState<Act>`; once the state has advanced to
//! `Assert`, executing work is a type error, not a runtime discipline.

use chicago_tdd_tools::state::{Arrange, TestState};

fn main() {
    let state = TestState::<Arrange>::new().act().assert();
    // ERROR: no method `execute` found for `TestState<Assert>` (E0599)
    let _ = state.execute(|data| data.unwrap_or_default());
}
