//! Conformance loader negative-matrix integration tests (lane C13).
//!
//! Exercises the fail-closed loader/validator surface of
//! `chicago_tdd_tools::conformance` — one test per typed
//! [`ConformanceError`](chicago_tdd_tools::conformance::error::ConformanceError)
//! variant, plus positive golden-DSL and JSON-roundtrip identity cases.
//! Err-assertions use `matches!`/explicit matching against `Display`, never
//! `unwrap` on a `Result::Err`.
//!
//! See Also: `src/conformance/loader.rs`, `src/conformance/error.rs`

//! Run: `cargo test --test conformance_loader_tests --features conformance`
#![cfg(feature = "conformance")]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
// JUSTIFICATION: test code only — expect() on Ok paths of real parses.
#![allow(clippy::unwrap_used)]

use chicago_tdd_tools::conformance::error::ConformanceError;
use chicago_tdd_tools::conformance::loader::{ActivityDecl, PowlModel, PowlNode};
use chicago_tdd_tools::conformance::model::{Arc, NodeRef, PlaceId, Transition};
use chicago_tdd_tools::conformance::net::NetSpec;
use chicago_tdd_tools::conformance::validate;

// ---------------------------------------------------------------------------
// Golden DSL -> expected PowlModel shape (positive)
// ---------------------------------------------------------------------------

const GOLDEN: &str = "\
# golden process: handle then a choice, then a bounded redo loop
activity handle = log.handle
activity fork = robot.fork
activity join = robot.join

sequence {
    handle
    xor {
        fork
        parallel {
            join
            silent
        }
    }
    loop {
        handle
    } redo {
        silent
    } max 7
}
";

#[test]
fn golden_dsl_loads_expected_model_shape() {
    let model = PowlModel::from_powl_str(GOLDEN).expect("golden DSL must parse");

    // Declaration table and order.
    assert_eq!(model.declaration_order, vec!["handle", "fork", "join"]);
    assert_eq!(
        model.activities["handle"],
        ActivityDecl { source: "log".into(), event: "handle".into() }
    );
    assert_eq!(
        model.activities["fork"],
        ActivityDecl { source: "robot".into(), event: "fork".into() }
    );
    assert_eq!(model.activities.len(), 3);

    // Root: sequence of [handle, xor, loop(max 7)].
    let PowlNode::Sequence(children) = &model.root else {
        panic!("root must be a sequence, got {:?}", model.root);
    };
    assert_eq!(children.len(), 3);
    assert_eq!(children[0], PowlNode::Activity("handle".into()));

    let PowlNode::Xor(branch) = &children[1] else {
        panic!("child 1 must be an xor, got {:?}", children[1]);
    };
    assert_eq!(branch.len(), 2);
    assert_eq!(branch[0], PowlNode::Activity("fork".into()));
    let PowlNode::Parallel(concurrent) = &branch[1] else {
        panic!("xor branch 1 must be parallel, got {:?}", branch[1]);
    };
    assert_eq!(concurrent[0], PowlNode::Activity("join".into()));
    assert_eq!(concurrent[1], PowlNode::Silent);

    let PowlNode::Loop { body, redo, max } = &children[2] else {
        panic!("child 2 must be a loop, got {:?}", children[2]);
    };
    assert_eq!(**body, PowlNode::Activity("handle".into()));
    assert_eq!(**redo, PowlNode::Silent);
    assert_eq!(*max, 7);
}

#[test]
fn golden_dsl_via_file_roundtrips_through_json_identity() {
    let model = PowlModel::from_powl_str(GOLDEN).expect("golden DSL must parse");
    let json = serde_json::to_string(&model).expect("serialize golden model");
    let back = PowlModel::from_powl_json(&json).expect("deserialize golden model");
    assert_eq!(model, back, "JSON roundtrip must be identity");

    // Structural identity is exact, including the loop bound.
    assert_eq!(back.declaration_order, vec!["handle", "fork", "join"]);
    let PowlNode::Sequence(children) = &back.root else {
        panic!("roundtripped root must be a sequence");
    };
    assert_eq!(children.len(), 3);
}

// ---------------------------------------------------------------------------
// MalformedSource — unknown keyword
// ---------------------------------------------------------------------------

#[test]
fn unknown_word_as_expression_is_unknown_activity_at_1_1() {
    // A bare undeclared word in expression position resolves through the
    // activity-reference arm, so it is UnknownActivity (positioned at the
    // word), not MalformedSource.
    let err = match PowlModel::from_powl_str("frobnicate { silent }") {
        Err(e) => e,
        Ok(model) => panic!("unknown word must be refused, got {model:?}"),
    };
    match err {
        ConformanceError::UnknownActivity { name, line, col } => {
            assert_eq!(name, "frobnicate");
            assert_eq!((line, col), (1, 1));
        }
        other => panic!("expected UnknownActivity, got {other}"),
    }
}

#[test]
fn reserved_word_as_activity_name_is_malformed_source() {
    // `max` is a DSL keyword; using it as a declared name is the
    // keyword-position MalformedSource path.
    let err = match PowlModel::from_powl_str("activity max = s.e\nsequence { max }") {
        Err(e) => e,
        Ok(model) => panic!("reserved word must be refused, got {model:?}"),
    };
    match err {
        ConformanceError::MalformedSource { line, col, reason } => {
            assert_eq!((line, col), (1, 10));
            assert!(
                reason.contains("reserved"),
                "reason must name the reserved-word violation, got: {reason}"
            );
        }
        other => panic!("expected MalformedSource, got {other}"),
    }
}

// ---------------------------------------------------------------------------
// MalformedSource — unbalanced braces (both directions)
// ---------------------------------------------------------------------------

#[test]
fn unbalanced_open_brace_is_malformed_source() {
    let err = match PowlModel::from_powl_str("sequence { silent") {
        Err(e) => e,
        Ok(model) => panic!("unclosed block must be refused, got {model:?}"),
    };
    match err {
        ConformanceError::MalformedSource { line, col, reason } => {
            assert_eq!((line, col), (1, 18), "error must sit at EOF (col = len + 1)");
            assert!(reason.contains("'{'"), "reason must name the unbalanced brace, got: {reason}");
        }
        other => panic!("expected MalformedSource, got {other}"),
    }
}

#[test]
fn unbalanced_close_brace_is_malformed_source() {
    let err = match PowlModel::from_powl_str("sequence { silent } }") {
        Err(e) => e,
        Ok(model) => panic!("stray brace must be refused, got {model:?}"),
    };
    assert!(
        matches!(err, ConformanceError::MalformedSource { .. }),
        "expected MalformedSource, got {err}"
    );
}

// ---------------------------------------------------------------------------
// MalformedSource — non-numeric coordinate / bad literals
// ---------------------------------------------------------------------------

#[test]
fn non_numeric_event_coordinate_is_malformed_source() {
    // A third dotted segment is not part of `source.event`.
    let err = match PowlModel::from_powl_str("activity a = s.e.x\nsequence { a }") {
        Err(e) => e,
        Ok(model) => panic!("trailing coordinate segment must be refused, got {model:?}"),
    };
    assert!(
        matches!(err, ConformanceError::MalformedSource { line: 1, .. }),
        "expected MalformedSource on line 1, got {err}"
    );
}

#[test]
fn numeric_event_name_is_malformed_source() {
    let err = match PowlModel::from_powl_str("activity a = s.42\nsequence { a }") {
        Err(e) => e,
        Ok(model) => panic!("numeric event name must be refused, got {model:?}"),
    };
    assert!(
        matches!(err, ConformanceError::MalformedSource { .. }),
        "expected MalformedSource, got {err}"
    );
}

// ---------------------------------------------------------------------------
// MalformedSource — line:col correctness at a known offset
// ---------------------------------------------------------------------------

#[test]
fn malformed_source_reports_exact_line_and_column() {
    // Offset of '@' is 21: 17 chars of line 1 + its '\n', line 2's '\n',
    // two leading spaces, so line 3 column 3.
    let src = "activity a = s.e\n\n  @\n";
    let err = match PowlModel::from_powl_str(src) {
        Err(e) => e,
        Ok(model) => panic!("illegal character must be refused, got {model:?}"),
    };
    match err {
        ConformanceError::MalformedSource { line, col, reason } => {
            assert_eq!((line, col), (3, 3), "position must match the '@' at byte offset 21");
            assert!(
                reason.contains("illegal character"),
                "reason must name the illegal character, got: {reason}"
            );
        }
        other => panic!("expected MalformedSource, got {other}"),
    }
}

// ---------------------------------------------------------------------------
// UnknownActivity
// ---------------------------------------------------------------------------

#[test]
fn undeclared_activity_reference_is_unknown_activity() {
    let err = match PowlModel::from_powl_str("sequence { ghost }") {
        Err(e) => e,
        Ok(model) => panic!("undeclared name must be refused, got {model:?}"),
    };
    match err {
        ConformanceError::UnknownActivity { name, line, col } => {
            assert_eq!(name, "ghost");
            assert_eq!((line, col), (1, 12));
        }
        other => panic!("expected UnknownActivity, got {other}"),
    }
}

// ---------------------------------------------------------------------------
// DuplicateActivity
// ---------------------------------------------------------------------------

#[test]
fn duplicate_activity_declaration_is_typed() {
    let src = "activity a = s.e\nactivity a = s.f\nsequence { a }";
    let err = match PowlModel::from_powl_str(src) {
        Err(e) => e,
        Ok(model) => panic!("duplicate declaration must be refused, got {model:?}"),
    };
    match err {
        ConformanceError::DuplicateActivity { name, line, col } => {
            assert_eq!(name, "a");
            assert_eq!((line, col), (2, 10), "position must be the duplicate name on line 2");
        }
        other => panic!("expected DuplicateActivity, got {other}"),
    }
}

#[test]
fn duplicate_place_in_net_text_is_duplicate_activity() {
    let err = match chicago_tdd_tools::conformance::loader::ParsedNet::from_net_str(
        "place p1\nplace p1\n",
    ) {
        Err(e) => e,
        Ok(net) => panic!("duplicate place must be refused, got {net:?}"),
    };
    match err {
        ConformanceError::DuplicateActivity { name, line, .. } => {
            assert_eq!(name, "p1");
            assert_eq!(line, 2);
        }
        other => panic!("expected DuplicateActivity, got {other}"),
    }
}

// ---------------------------------------------------------------------------
// InvalidJson
// ---------------------------------------------------------------------------

#[test]
fn malformed_json_is_invalid_json() {
    let err = match PowlModel::from_powl_json("{ definitely not json") {
        Err(e) => e,
        Ok(model) => panic!("malformed JSON must be refused, got {model:?}"),
    };
    match err {
        ConformanceError::InvalidJson { message } => {
            assert!(!message.is_empty(), "parser diagnostic must be captured");
        }
        other => panic!("expected InvalidJson, got {other}"),
    }
}

#[test]
fn json_with_wrong_shape_is_invalid_json() {
    // Well-formed JSON that is not a PowlModel.
    let err = match PowlModel::from_powl_json("[1, 2, 3]") {
        Err(e) => e,
        Ok(model) => panic!("non-model JSON must be refused, got {model:?}"),
    };
    assert!(matches!(err, ConformanceError::InvalidJson { .. }), "expected InvalidJson, got {err}");
}

// ---------------------------------------------------------------------------
// Io — from_powl_file on a nonexistent path
// ---------------------------------------------------------------------------

#[test]
fn missing_file_is_io_error_naming_the_path() {
    let path = "/definitely/not/a/real/path.powl";
    let err = match PowlModel::from_powl_file(path) {
        Err(e) => e,
        Ok(model) => panic!("missing file must be refused, got {model:?}"),
    };
    match err {
        ConformanceError::Io { path: reported, message } => {
            assert!(
                reported.contains(path),
                "Io variant must carry the requested path, got: {reported}"
            );
            assert!(!message.is_empty(), "OS diagnostic must be captured");
        }
        other => panic!("expected Io, got {other}"),
    }
}

// ---------------------------------------------------------------------------
// ArityBelowMinimum — empty sequence through the validator
// ---------------------------------------------------------------------------

#[test]
fn empty_sequence_is_arity_below_minimum() {
    use std::collections::BTreeMap;

    // Loader-literal PowlModel (loader.rs types): no declared activities, an
    // empty sequence root. Size-legal, structurally illegal.
    let model = PowlModel {
        activities: BTreeMap::new(),
        declaration_order: Vec::new(),
        root: PowlNode::Sequence(Vec::new()),
    };
    let err = match validate(&model) {
        Err(e) => e,
        Ok(()) => panic!("empty sequence must fail the arity layer"),
    };
    match err {
        ConformanceError::ArityBelowMinimum { constructor, minimum, found } => {
            assert_eq!(constructor, "sequence");
            assert_eq!((minimum, found), (1, 0));
        }
        other => panic!("expected ArityBelowMinimum, got {other}"),
    }
}

// ---------------------------------------------------------------------------
// ModelTooLarge — > 64 nodes
// ---------------------------------------------------------------------------

#[test]
fn model_over_sixty_four_nodes_is_too_large() {
    // 64 activities under one sequence root = 65 nodes > MAX_POWL_NODES (64);
    // the loader itself enforces the cap at parse time (loader.rs
    // `from_powl_str`).
    let mut src = String::new();
    for i in 0..64 {
        src.push_str(&format!("activity big{i} = sys.big{i}\n"));
    }
    src.push_str("\nsequence {\n");
    for i in 0..64 {
        src.push_str(&format!("    big{i}\n"));
    }
    src.push_str("}\n");
    let err = match PowlModel::from_powl_str(&src) {
        Err(e) => e,
        Ok(model) => panic!("65-node model must be refused, got {model:?}"),
    };
    match err {
        ConformanceError::ModelTooLarge { found, limit } => {
            assert_eq!((found, limit), (65, 64));
        }
        other => panic!("expected ModelTooLarge, got {other}"),
    }
}

#[test]
fn net_spec_over_transition_budget_is_too_large() {
    let mut builder = NetSpec::builder().place("p0");
    for i in 0..257 {
        builder = builder.transition(format!("t{i}"), "act");
    }
    let err = match builder.build().compile() {
        Err(e) => e,
        Ok(net) => panic!("257-transition net must be refused, got {net:?}"),
    };
    match err {
        ConformanceError::ModelTooLarge { found, limit } => {
            assert_eq!((found, limit), (257, 256));
        }
        other => panic!("expected ModelTooLarge, got {other}"),
    }
}

// ---------------------------------------------------------------------------
// UnboundedLoop — see note at bottom of file (not expressible under this
// feature set; documented rather than faked).
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// WeightedArc / TooManyPlaces / UnknownNodeRef — Petri-net constructors
// ---------------------------------------------------------------------------

#[test]
fn non_unit_arc_weight_is_weighted_arc() {
    use chicago_tdd_tools::conformance::model::TransitionId;

    let net = chicago_tdd_tools::conformance::model::PetriNet::try_new(
        vec!["p0".into(), "p1".into()],
        vec![Transition { name: "t0".into(), label: Some("act".into()) }],
        vec![Arc {
            from: NodeRef::Place(PlaceId(0)),
            to: NodeRef::Transition(TransitionId(0)),
            weight: 3,
        }],
        std::collections::BTreeSet::from([PlaceId(0)]),
    );
    let err = match net {
        Err(e) => e,
        Ok(net) => panic!("weight-3 arc must be refused, got {net:?}"),
    };
    match err {
        ConformanceError::WeightedArc { weight } => assert_eq!(weight, 3),
        other => panic!("expected WeightedArc, got {other}"),
    }
}

#[test]
fn net_over_place_budget_is_too_many_places() {
    use chicago_tdd_tools::conformance::model::{PetriNet as ModelPetriNet, MAX_PETRI_PLACES};

    let places: Vec<String> = (0..=MAX_PETRI_PLACES).map(|i| format!("p{i}")).collect();
    let err = match ModelPetriNet::try_new(places, Vec::new(), Vec::new(), Default::default()) {
        Err(e) => e,
        Ok(net) => panic!("over-budget net must be refused, got {net:?}"),
    };
    match err {
        ConformanceError::TooManyPlaces { limit } => {
            assert_eq!(limit, usize::from(MAX_PETRI_PLACES));
        }
        other => panic!("expected TooManyPlaces, got {other}"),
    }
}

#[test]
fn net_spec_place_overflow_is_too_many_places() {
    let mut builder = NetSpec::builder();
    for i in 0..65 {
        builder = builder.place(format!("p{i}"));
    }
    let err = match builder.build().compile() {
        Err(e) => e,
        Ok(net) => panic!("65-place net must be refused, got {net:?}"),
    };
    match err {
        ConformanceError::TooManyPlaces { limit } => assert_eq!(limit, 64),
        other => panic!("expected TooManyPlaces, got {other}"),
    }
}

#[test]
fn unknown_arc_place_reference_is_unknown_node_ref() {
    let spec = NetSpec::builder()
        .place("p0")
        .transition("t0", "act")
        .add_pre("p0")
        .add_post("ghost")
        .build();
    let err = match spec.compile() {
        Err(e) => e,
        Ok(net) => panic!("undeclared place reference must be refused, got {net:?}"),
    };
    match err {
        ConformanceError::UnknownNodeRef { side, id } => {
            assert_eq!(side, "place");
            assert!(id.contains("ghost"), "diagnostic must name the place, got: {id}");
        }
        other => panic!("expected UnknownNodeRef, got {other}"),
    }
}

// ---------------------------------------------------------------------------
// DependencyCycle — not expressible, by construction
// ---------------------------------------------------------------------------
// Block-structured POWL constructors (Sequence/Xor/Parallel/Loop) cannot
// express a dependency cycle (see src/conformance/validate.rs module docs:
// acyclicity is a debug_assert over a Kahn ordering, and only
// PartialOrder-style constructors expose real precedence edges). The loader
// DSL cannot produce one, so no loader-path test exists — a vacuous test
// constructing the variant by hand would verify nothing.

// ---------------------------------------------------------------------------
// UnboundedLoop note
// ---------------------------------------------------------------------------
// `UnboundedLoop` is raised only by the BCINR bridge
// (src/conformance/bcinr.rs) for a Loop with `max_iters == 0`, behind the
// `powl-bcinr` feature + external `bcinr-powl` dependency. The loader DSL
// itself defaults an omitted `max` to 255 and never raises this variant, so
// it is unreachable under `feature = "conformance"` alone and is not faked
// here. The loader DOES admit `max 0` verbatim; that value is carried, not
// rejected, so the rejection correctly lives at the bridge boundary.
#[test]
fn loop_with_zero_max_is_carried_not_rejected_by_loader() {
    let model = PowlModel::from_powl_str("loop { silent } max 0").expect("max 0 must parse");
    let PowlNode::Loop { max, .. } = model.root else {
        panic!("root must be a loop");
    };
    assert_eq!(max, 0);
}
