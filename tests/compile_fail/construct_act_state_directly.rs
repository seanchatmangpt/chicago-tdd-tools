//! Negative space: intermediate states cannot be forged.
//!
//! `TestState`'s fields are private, so a caller cannot bypass the type state
//! machine by struct-literal-constructing a `TestState<Act>` (E0451).

use chicago_tdd_tools::state::{Act, TestState};

fn main() {
    let _forged: TestState<Act> = TestState {
        _phase: std::marker::PhantomData,
        data: (),
    };
}
