//! Conformance Test Macros
//!
//! `macro_rules!` surface for conformance testing over a `ConformanceSession`
//! and a receipt value. Composition over `src/core/receipt.rs` (`TestReceipt`):
//! the macros assert two boolean-ish results and panic with the deviation list
//! on failure; they own no session or receipt logic themselves.
//!
//! **Contract (as of this file's writing)**: there is no `session.rs` in
//! `src/conformance/` yet — `ConformanceSession` is an expected type from the
//! conformance lane, not a defined one. The macros therefore assert on a
//! `conformance_report()` accessor, which is the documented seam:
//!
//! - `$session.conformance_report()` must yield a value with
//!   `is_conforming() -> bool` and `deviations() -> &[S]` (or anything whose
//!   `is_empty()` reflects the deviations list; the panic message renders
//!   `Debug` of the report).
//! - `$receipt.is_algebraically_closed() -> bool` must exist on the receipt
//!   expression. The current `TestReceipt` (`src/core/receipt.rs`) does not
//!   yet expose `is_algebraically_closed()`; wiring that method into
//!   `TestReceipt` (or a wrapper) is the responsibility of the lane that owns
//!   the receipt integration, not of this macro.

/// Conformance test wrapper.
///
/// Expands the body into a `#[test]` function. The body must bind a session
/// value and end with an assertion against that session's report (typically
/// via `assert_conforms!`).
///
/// # Usage
///
/// ```rust,ignore
/// conformance_test!(test_order_process_conforms, {
///     // Arrange
///     let session = ConformanceSession::from(model, log);
///
///     // Act
///     session.replay();
///
///     // Assert
///     assert_conforms!(session, session.finish_receipt());
/// });
/// ```
///
/// The macro never panics outside the assertion path; it only wraps the body
/// in a `#[test]` function (no timeout, no async, no fixture machinery —
/// compose with [`test!`](crate::test) for that).
#[macro_export]
macro_rules! conformance_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() $body
    };
}

/// Dual conformance assertion: session report + receipt closure.
///
/// Asserts, in order:
///
/// 1. `$session.conformance_report().is_conforming()` is `true`, and
/// 2. `$receipt.is_algebraically_closed()` is `true`.
///
/// On either failure the test panics with the session's report `Debug` output
/// (which carries the deviations list under the documented contract above).
///
/// # Usage
///
/// ```rust,ignore
/// assert_conforms!(session, receipt);
/// ```
#[macro_export]
macro_rules! assert_conforms {
    ($session:expr, $receipt:expr) => {{
        let __report = ($session).conformance_report();
        let __conforming = __report.is_conforming();
        let __closed = ($receipt).is_algebraically_closed();
        if !__conforming || !__closed {
            panic!(
                "conformance failed: session conforms = {}, receipt algebraically closed = {}, report = {:#?}",
                __conforming, __closed, __report
            );
        }
    }};
}

#[cfg(test)]
mod tests {
    // The `macro_rules!` macros defined above are in textual scope here; no
    // `use` is needed (and importing the `#[macro_export]` re-exports would
    // make the names ambiguous).

    /// Trivial in-crate session stand-in: the macros are textual, so any type
    /// exposing the documented `conformance_report()` / `is_conforming()` /
    /// `is_algebraically_closed()` seams satisfies them. This is a real
    /// collaborator (hand-written real interface, not a mock of an owned
    /// collaborator — there is no other implementation to fake).
    struct TrivialSession {
        conforming: bool,
        closed: bool,
    }

    #[derive(Debug)]
    struct Report {
        conforming: bool,
    }

    impl Report {
        fn is_conforming(&self) -> bool {
            self.conforming
        }
    }

    impl TrivialSession {
        fn conformance_report(&self) -> Report {
            Report { conforming: self.conforming }
        }
    }

    struct TrivialReceipt(bool);

    impl TrivialReceipt {
        fn is_algebraically_closed(&self) -> bool {
            self.0
        }
    }

    conformance_test!(conformance_test_macro_passes_on_conforming_session, {
        // Arrange
        let session = TrivialSession { conforming: true, closed: true };
        let receipt = TrivialReceipt(true);

        // Act: none — the assertion is the observation of constructed state.

        // Assert
        assert_conforms!(session, receipt);
    });

    #[test]
    fn assert_conforms_macro_panics_on_nonconforming_session() {
        // Arrange
        let session = TrivialSession { conforming: false, closed: true };
        let receipt = TrivialReceipt(true);

        // Act + Assert
        let result = std::panic::catch_unwind(|| assert_conforms!(session, receipt));
        assert!(result.is_err(), "must panic when the session does not conform");
    }

    #[test]
    fn assert_conforms_macro_panics_on_unclosed_receipt() {
        // Arrange
        let session = TrivialSession { conforming: true, closed: true };
        let receipt = TrivialReceipt(false);

        // Act + Assert
        let result = std::panic::catch_unwind(|| assert_conforms!(session, receipt));
        assert!(result.is_err(), "must panic when the receipt is not algebraically closed");
    }
}
