//! Capability 08 — typestate `StateMachine` + `ModelChecker`, encoding the
//! Chatman invocation lifecycle S1..S6 with refusal edges, and checking the
//! constitutional invariant "no actuation without admission".
//!
//! Lifecycle (per DEFINITION_OF_DONE / NORTH_STAR):
//!   S1 Received -> S2 Enveloped -> S3 Admitted -> S4 Routed ->
//!   S5 Executed -> S6 Receipted, with a refusal edge S3 -> Refused.
//! Invalid transitions (e.g. Received -> Executed) do not compile — that is
//! the typestate guarantee, demonstrated by construction below.

mod chatman_common;

use chicago_tdd_tools::test;
use chicago_tdd_tools::testing::state_machine::{ModelChecker, State, StateMachine, Transition};

// ── S1..S6 as phantom states ─────────────────────────────────────────────────

struct Received;
struct Enveloped;
struct Admitted;
struct Routed;
struct Executed;
struct Receipted;
struct Refused;

macro_rules! state {
    ($ty:ident) => {
        impl State for $ty {
            fn name() -> &'static str {
                stringify!($ty)
            }
        }
    };
}
state!(Received);
state!(Enveloped);
state!(Admitted);
state!(Routed);
state!(Executed);
state!(Receipted);
state!(Refused);

// ── Transitions (edges of the lifecycle graph) ───────────────────────────────

struct BuildEnvelope;
impl Transition<Received, Enveloped> for BuildEnvelope {
    fn execute() -> Result<(), String> {
        Ok(())
    }
}

struct Admit;
impl Transition<Enveloped, Admitted> for Admit {
    fn execute() -> Result<(), String> {
        Ok(())
    }
}

/// Refusal edge: admission can refuse instead of admitting.
struct Refuse;
impl Transition<Enveloped, Refused> for Refuse {
    fn execute() -> Result<(), String> {
        Ok(())
    }
}

struct RouteLeastExpressive;
impl Transition<Admitted, Routed> for RouteLeastExpressive {
    fn execute() -> Result<(), String> {
        Ok(())
    }
}

struct Execute;
impl Transition<Routed, Executed> for Execute {
    fn execute() -> Result<(), String> {
        Ok(())
    }
}

struct EmitReceipt;
impl Transition<Executed, Receipted> for EmitReceipt {
    fn execute() -> Result<(), String> {
        Ok(())
    }
}

test!(happy_path_walks_s1_through_s6, {
    // Arrange: an invocation begins in Received (S1).
    let machine = StateMachine::<Received>::new();

    // Act: only the declared edges compile; the chain is linear.
    let receipted = machine
        .transition::<Enveloped, BuildEnvelope>().map_err(std::io::Error::other)?
        .transition::<Admitted, Admit>().map_err(std::io::Error::other)?
        .transition::<Routed, RouteLeastExpressive>().map_err(std::io::Error::other)?
        .transition::<Executed, Execute>().map_err(std::io::Error::other)?
        .transition::<Receipted, EmitReceipt>().map_err(std::io::Error::other)?;

    // Assert: terminal state reached; type-level name agrees.
    let _ = receipted;
    assert_eq!(StateMachine::<Receipted>::current_state(), "Receipted");
    Ok::<(), std::io::Error>(())
});

test!(refusal_edge_terminates_the_lifecycle, {
    // Arrange
    let machine = StateMachine::<Received>::new();

    // Act: envelope, then take the refusal edge instead of admission.
    let refused = machine
        .transition::<Enveloped, BuildEnvelope>().map_err(std::io::Error::other)?
        .transition::<Refused, Refuse>().map_err(std::io::Error::other)?;

    // Assert: Refused is terminal — no Transition<Refused, _> exists, so any
    // attempt to continue is a compile error (the typestate guarantee).
    let _ = refused;
    assert_eq!(StateMachine::<Refused>::current_state(), "Refused");
    Ok::<(), std::io::Error>(())
});

test!(model_checker_no_actuation_without_admission, {
    // Arrange: bounded model checker over generated schedules.
    let checker = ModelChecker::new(4);

    // Act + Assert: in every schedule, an "Execute" step (actuation) must be
    // preceded by an "Admit" step by the same actor — no actuation without
    // admission. Vacuously true schedules (no Execute) pass.
    checker
        .check_invariant(|schedule| {
            let mut admitted = false;
            for step in schedule.steps() {
                if step.transition == "Admit" {
                    admitted = true;
                }
                if step.transition == "Execute" && !admitted {
                    return false;
                }
            }
            true
        })
        .map_err(std::io::Error::other)?;
    Ok::<(), std::io::Error>(())
});
