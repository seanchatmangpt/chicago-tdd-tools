//! Negative space: the phase-marker trait is sealed (and unnameable).
//!
//! The `Sealed` trait lives in a private module behind `core::state`, so the
//! import fails with E0432 (unresolved import), not E0599 (trait not
//! implemented). The failing import also makes `TestState` unnameable in the
//! same file, so rustc reports a secondary E0433 for `TestState::<Arrange>`.

use chicago_tdd_tools::state::{Arrange, Sealed};

struct MyPhase;

impl Sealed for MyPhase {}

fn main() {
    let _ = TestState::<Arrange>::new();
}
