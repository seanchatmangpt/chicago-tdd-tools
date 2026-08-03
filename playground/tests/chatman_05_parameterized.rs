//! Capability 05 — parameterized testing (`param_test!` over rstest):
//! boundary matrices on the Triple8 table size (0/1/255/256/257) and the
//! dialect x profile-gate routing grid.
//!
//! Note: `param_test!` expands to `#[rstest::rstest(...)]` in the consumer
//! crate, so `rstest` is a direct dev-dependency of the playground.

mod chatman_common;

use chicago_tdd_tools::param_test;
use praxis_graphlaw::chatman::abi::ProfileId;
use praxis_graphlaw::chatman::router::{Dialect, DialectRouter, ProfileGates, QueryShape, Route};
use praxis_graphlaw::chatman::triple8::ProfileSymbolTable;

/// Builds a table with `n` distinct terms and reports whether it was admitted.
fn build_n_terms(n: usize) -> bool {
    let terms: Vec<String> = (0..n).map(|i| format!("<urn:t:{i:04}>")).collect();
    ProfileSymbolTable::build(ProfileId::new("profile:param"), terms).is_ok()
}

param_test! {
    #[case(0, true)]
    #[case(1, true)]
    #[case(255, true)]
    #[case(256, true)]   // exactly the boundary: admitted
    #[case(257, false)]  // one past: Triple8UniverseOverflow
    fn triple8_table_size_boundary_matrix(n: usize, admitted: bool) {
        // Arrange + Act
        let ok = build_n_terms(n);

        // Assert
        assert_eq!(ok, admitted, "table of {n} terms: expected admitted={admitted}");
    }
}

/// A plain SELECT-shaped query with `constraints` triple constraints.
const fn select_shape(constraints: u8) -> QueryShape {
    QueryShape {
        constraint_count: constraints,
        requires_construct: false,
        requires_owl: false,
        requires_n3_builtins: false,
        wants_actuation: false,
    }
}

param_test! {
    // (needs_owl, needs_construct, needs_n3, expected dialect name or refusal)
    #[case(false, false, false, Some("Triple8Pattern"))]
    #[case(false, true,  false, Some("SparqlConstruct"))]
    #[case(true,  false, false, Some("OwlRl"))]
    #[case(true,  true,  false, Some("OwlRl"))]           // OWL RL floor dominates
    #[case(false, false, true,  None)]                     // N3 gated off by default
    fn dialect_routing_grid_under_default_gates(
        owl: bool,
        construct: bool,
        n3: bool,
        expected: Option<&'static str>,
    ) {
        // Arrange: default gates — N3 disabled, actuation empty (read-only).
        let gates = ProfileGates::new(
            ProfileId::new("profile:grid"),
            ProfileGates::DEFAULT_ENABLED_MASK,
            0,
            8,
        )
        .expect("default gates are valid by construction");
        let router = DialectRouter::new(gates);
        let shape = QueryShape {
            constraint_count: 3,
            requires_construct: construct,
            requires_owl: owl,
            requires_n3_builtins: n3,
            wants_actuation: false,
        };

        // Act
        let decision = router.decide(&shape);

        // Assert: least-expressive routing, or refusal when N3 is gated off.
        match expected {
            Some(name) => {
                let d = decision.expect("shape must route under default gates");
                assert_eq!(d.dialect.name(), name);
            }
            None => assert!(decision.is_err(), "N3 must be unavailable by default"),
        }
    }
}

param_test! {
    // Hot-path budget fence: <= max_hot_constraints stays Hot, above spills Warm.
    #[case(0, "Hot")]
    #[case(8, "Hot")]
    #[case(9, "Warm")]
    fn hot_path_constraint_budget_matrix(constraints: u8, route_name: &'static str) {
        // Arrange
        let gates = ProfileGates::new(
            ProfileId::new("profile:budget"),
            ProfileGates::DEFAULT_ENABLED_MASK,
            0,
            8,
        )
        .expect("valid gates");
        let router = DialectRouter::new(gates);

        // Act
        let decision = router.decide(&select_shape(constraints)).expect("routes");

        // Assert
        assert_eq!(decision.route.name(), route_name);
    }
}

param_test! {
    // Route tiers are a law of the dialect order, checked exhaustively.
    #[case(Dialect::Triple8Pattern, Route::Hot)]
    #[case(Dialect::ShaclCore, Route::Warm)]
    #[case(Dialect::SparqlSelect, Route::Warm)]
    #[case(Dialect::SparqlConstruct, Route::Warm)]
    #[case(Dialect::OwlRl, Route::Warm)]
    #[case(Dialect::N3, Route::Cold)]
    fn dialect_route_tier_matrix(dialect: Dialect, route: Route) {
        assert_eq!(dialect.route(), route);
    }
}
