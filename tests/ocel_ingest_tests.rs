//! OCEL ingest integration tests (lane C18).
//!
//! Covers the four ingest-side OCEL modules end to end against their real
//! APIs: `ocel64_bridge` (pack/unpack roundtrip + typed refusals),
//! `idmap_bridge` (collision/capacity courts via the bridge entry points),
//! `import_json` (golden + fail-closed version refusal), and `import_xml`
//! (roundtrip-stable import + malformed-shape refusal).
//!
//! All items are gated on the ingest features — the modules are only
//! compiled when those features are active.
#![cfg(all(
    feature = "ocel-ingest",
    feature = "ocel-generation",
    feature = "ocel-ingest-json",
    feature = "ocel-ingest-xml"
))]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
// JUSTIFICATION: test code only — assertions on real imported/packed state.
#![allow(clippy::unwrap_used)]

use chicago_tdd_tools::observability::ocel::idmap_bridge::{
    BridgeError as IdMapBridgeError, BridgeVocab,
};
use chicago_tdd_tools::observability::ocel::import_json::{import_ocel_json, ImportError};
use chicago_tdd_tools::observability::ocel::import_xml::{import_ocel_xml, XmlImportError};
use chicago_tdd_tools::observability::ocel::ocel64_bridge::{
    activity_key, checked_deltas, from_ocel64, to_ocel64, BridgeError,
};
use chicago_tdd_tools::observability::ocel::types::{
    OcelLog, TestActivity, TestObjectType, TestOcelEvent,
};
use std::collections::HashMap;

// ── Corpus construction ─────────────────────────────────────────────────────

/// Build one event with defaults; callers override fields they need.
fn ev(
    case: &str,
    activity: TestActivity,
    ts: u64,
    objects: Vec<(String, TestObjectType)>,
) -> TestOcelEvent {
    TestOcelEvent {
        event_id: String::new(),
        case_id: case.to_string(),
        activity,
        timestamp_ns: ts,
        objects,
        attributes: HashMap::new(),
    }
}

/// Build the clean multi-case corpus: 2 cases x 3 events, distinct unit
/// activities and distinct same-type object pairs per event (same-type pairs
/// are required for exact object-type roundtrip — see the `ocel64_bridge`
/// module docs on the single-qualifier-byte recovery rule).
fn clean_corpus() -> OcelLog {
    let mut log = OcelLog::new();
    let case_a = [
        (TestActivity::TestStarted, 0u64),
        (TestActivity::ReceiptGenerated, 10),
        (TestActivity::DriftZero, 20),
    ];
    for (i, (act, ts)) in case_a.iter().enumerate() {
        log.events.insert(
            format!("a{i}"),
            ev(
                "run-A",
                act.clone(),
                *ts,
                vec![
                    (format!("art-a{i}-1"), TestObjectType::Artifact),
                    (format!("art-a{i}-2"), TestObjectType::Artifact),
                ],
            ),
        );
    }
    let case_b = [
        (TestActivity::FixtureSetup, 0u64),
        (TestActivity::ArtifactAdmitted, 5),
        (TestActivity::OntologyConsistent, 9),
    ];
    for (i, (act, ts)) in case_b.iter().enumerate() {
        log.events.insert(
            format!("b{i}"),
            ev(
                "run-B",
                act.clone(),
                *ts,
                vec![
                    (format!("fix-b{i}-1"), TestObjectType::Fixture),
                    (format!("fix-b{i}-2"), TestObjectType::Fixture),
                ],
            ),
        );
    }
    log
}

// ── Roundtrip identity on a clean multi-case corpus ─────────────────────────

#[test]
fn ocel64_roundtrip_identity_two_cases_three_events() {
    let log = clean_corpus();
    let (words, vocab) = to_ocel64(&log).expect("clean corpus packs");
    let back = from_ocel64(&words, &vocab);

    assert_eq!(back.len(), 6, "2 cases x 3 events decoded");

    // Activities preserved (unit variants, compared by vocab key).
    assert_eq!(activity_key(&back[0].activity), "TestStarted");
    assert_eq!(activity_key(&back[1].activity), "ReceiptGenerated");
    assert_eq!(activity_key(&back[2].activity), "DriftZero");
    assert_eq!(activity_key(&back[3].activity), "FixtureSetup");
    assert_eq!(activity_key(&back[4].activity), "ArtifactAdmitted");
    assert_eq!(activity_key(&back[5].activity), "OntologyConsistent");

    // Case segmentation: 3 events per case, cases in lexicographic order.
    assert_eq!(back[0].case_id, "case_0");
    assert_eq!(back[1].case_id, "case_0");
    assert_eq!(back[2].case_id, "case_0");
    assert_eq!(back[3].case_id, "case_1");
    assert_eq!(back[4].case_id, "case_1");
    assert_eq!(back[5].case_id, "case_1");

    // Per-case relative timelines preserved: run-A 0,10,20; run-B 0,5,9.
    assert_eq!([back[0].timestamp_ns, back[1].timestamp_ns, back[2].timestamp_ns], [0, 10, 20]);
    assert_eq!([back[3].timestamp_ns, back[4].timestamp_ns, back[5].timestamp_ns], [0, 5, 9]);

    // Object names and types preserved for the same-type pairs.
    for (i, event) in back.iter().enumerate() {
        assert_eq!(event.objects.len(), 2, "event {i} keeps both objects");
    }
    assert_eq!(back[0].objects[0].0, "art-a0-1");
    assert_eq!(back[0].objects[1].0, "art-a0-2");
    assert_eq!(back[0].objects[0].1, TestObjectType::Artifact);
    assert_eq!(back[0].objects[1].1, TestObjectType::Artifact);
    assert_eq!(back[3].objects[0].0, "fix-b0-1");
    assert_eq!(back[3].objects[1].0, "fix-b0-2");
    assert_eq!(back[3].objects[0].1, TestObjectType::Fixture);

    // Determinism: the same log packs to byte-identical words.
    let (words2, _) = to_ocel64(&log).expect("second pack");
    assert_eq!(words, words2);
}

// ── Collision refusal (brute-forced 16-bit blake3 sentinel collision) ───────

#[test]
fn ocel64_collision_refused_via_brute_forced_names() {
    // Bound the brute-force loop: 1000 generated names over a 65,536-bucket
    // 16-bit space collide with probability ~99.95% (birthday bound), so the
    // double loop below is at most ~5e5 blake3 truncations and always finds
    // a colliding pair on this corpus. Same technique as the idmap lane's
    // own court in `idmap_bridge.rs`.
    let generated: Vec<String> = (0..1000).map(|i| format!("c{i}")).collect();
    let mut pair = None;
    'outer: for i in 0..generated.len() {
        for j in (i + 1)..generated.len() {
            let sa = blake3::hash(generated[i].as_bytes());
            let sb = blake3::hash(generated[j].as_bytes());
            if sa.as_bytes()[0] == sb.as_bytes()[0] && sa.as_bytes()[1] == sb.as_bytes()[1] {
                pair = Some((generated[i].clone(), generated[j].clone()));
                break 'outer;
            }
        }
    }
    let (a, b) = pair.expect("1000 generated names must contain a 16-bit collision");
    assert_ne!(a, b, "the pair must be two DISTINCT strings");

    // Feed the colliding object names through the real bridge entry point:
    // one event referencing both colliding names as Artifact objects.
    let mut log = OcelLog::new();
    log.events.insert(
        "e0".to_string(),
        ev(
            "run-col",
            TestActivity::TestStarted,
            0,
            vec![(a.clone(), TestObjectType::Artifact), (b.clone(), TestObjectType::Artifact)],
        ),
    );
    let err = to_ocel64(&log).expect_err("colliding object names must refuse the pack");
    match err {
        BridgeError::Collision { field, details } => {
            assert_eq!(field, "objects");
            // Both colliding names surface in the typed details.
            assert!(details.contains(&a), "details must name {a}: {details}");
            assert!(details.contains(&b), "details must name {b}: {details}");
        }
        other => panic!("expected Collision, got {other:?}"),
    }
}

// ── CapacityExceeded at the real bound ──────────────────────────────────────

#[test]
fn capacity_exceeded_refused_at_real_bound_via_bridge_vocab() {
    // `to_ocel64`'s own capacity gate (`ensure_capacity`) is `pub(crate)` and
    // refuses above `u16::MAX` names; hitting it through `to_ocel64` would
    // need a log with >65k distinct object names. The directly constructible
    // small-bound path is `BridgeVocab::build`, whose bound is
    // `u16::MAX - 1` usable ids (1..=65534; id 0 reserved). Bare strings are
    // cheap: build 65,535 distinct activity names and refuse up front.
    let bound = u16::MAX as usize - 1;
    let huge: Vec<String> = (0..=bound).map(|i| format!("x{i:07}")).collect();
    let refs: Vec<&str> = huge.iter().map(String::as_str).collect();
    let err = BridgeVocab::build(&refs, &[]).expect_err("over-bound corpus must refuse");
    match err {
        IdMapBridgeError::CapacityExceeded { field, count, bound: b } => {
            assert_eq!(field, "activities");
            assert_eq!(count, bound + 1);
            assert_eq!(b, bound);
        }
        other => panic!("expected CapacityExceeded, got {other:?}"),
    }
    // A collision-free corpus well under the bound builds fine. (The bound
    // itself cannot be exercised collision-free: 65k distinct names over a
    // 65,536-bucket 16-bit blake3 sentinel space collide with probability
    // ~1 — the idmap module docs' birthday bound — so the capacity gate is
    // what refuses at the bound, not a collision-free build.)
    let ok_corpus: Vec<String> = (0..100).map(|i| format!("y{i:07}")).collect();
    let ok_refs: Vec<&str> = ok_corpus.iter().map(String::as_str).collect();
    assert!(
        BridgeVocab::build(&ok_refs, &[]).is_ok(),
        "a collision-free corpus under the bound must build"
    );
}

// ── TickDeltaOverflow through the full bridge ───────────────────────────────

#[test]
fn tick_delta_overflow_refused_not_saturated() {
    let mut log = OcelLog::new();
    log.events
        .insert("e0".to_string(), ev("run-X", TestActivity::TestStarted, 0, vec![]));
    log.events.insert(
        "e1".to_string(),
        ev(
            "run-X",
            TestActivity::ReceiptGenerated,
            256, // 256ns after the previous event: one past the 8-bit tick field
            vec![],
        ),
    );
    let err = to_ocel64(&log).expect_err("delta 256 must refuse");
    match err {
        BridgeError::TickDeltaOverflow { case, index, delta_ns } => {
            assert_eq!(case, "run-X");
            assert_eq!(index, 1);
            assert_eq!(delta_ns, 256);
        }
        other => panic!("expected TickDeltaOverflow, got {other:?}"),
    }
}

// ── NonMonotonic refusal via checked_deltas ─────────────────────────────────

#[test]
fn non_monotonic_refused_but_identical_timestamps_accepted() {
    // Decreasing timestamps refuse (unreachable from `to_ocel64`, which
    // sorts first — the check is re-derived in `checked_deltas`).
    let err = checked_deltas("run-M", &[10, 5]).expect_err("decreasing ts must refuse");
    assert_eq!(err, BridgeError::NonMonotonicTimestamp { case: "run-M".to_string(), index: 1 });
    // Identical timestamps are NOT non-monotonic: delta 0, accepted.
    assert_eq!(checked_deltas("run-M", &[7, 7]).expect("identical ts are delta 0"), vec![0, 0]);
    // A full three-point decreasing chain refuses at the first decrease.
    let err = checked_deltas("run-M", &[9, 9, 4]).expect_err("third event decreases");
    assert_eq!(err, BridgeError::NonMonotonicTimestamp { case: "run-M".to_string(), index: 2 });
}

// ── Empty-attributes disclosure (packed tier is lossy by design) ────────────

#[test]
fn from_ocel64_events_have_empty_attributes() {
    let log = clean_corpus();
    let (words, vocab) = to_ocel64(&log).expect("clean corpus packs");
    let back = from_ocel64(&words, &vocab);
    assert_eq!(back.len(), 6);
    assert!(
        back.iter().all(|e| e.attributes.is_empty()),
        "packed-tier decode must always produce EMPTY attribute maps"
    );
}

// ── JSON import: golden + InvalidVersion refusal ────────────────────────────

const JSON_GOLDEN: &str = r#"{
    "ocel:version": "2.0",
    "ocel:events": {
        "e1": {
            "ocel:timestamp": 1700000000000000000,
            "ocel:activity": "TestStarted",
            "ocel:caseId": "run-42",
            "ocel:attributes": {"suite": "ingest"},
            "ocel:objectTypes": [{"id": "f1", "type": "Fixture"}]
        },
        "e2": {
            "ocel:timestamp": 1700000000000000001,
            "ocel:activity": "ReceiptGenerated",
            "ocel:caseId": "run-42"
        }
    },
    "ocel:objects": {
        "f1": {"ocel:type": "Fixture", "ocel:ovmap": {"name": "db"}}
    }
}"#;

#[test]
fn json_import_golden_maps_events_and_objects() {
    let log = import_ocel_json(JSON_GOLDEN).expect("golden must import");
    assert_eq!(log.events.len(), 2);
    assert_eq!(log.objects.len(), 1);

    let e1 = log.events.get("e1").expect("e1");
    assert_eq!(e1.event_id, "e1");
    assert_eq!(e1.case_id, "run-42");
    assert_eq!(e1.timestamp_ns, 1_700_000_000_000_000_000);
    assert_eq!(
        serde_json::to_string(&e1.activity).expect("serialize activity"),
        r#""TestStarted""#
    );
    assert_eq!(e1.objects, vec![("f1".to_string(), TestObjectType::Fixture)]);
    assert_eq!(e1.attributes.get("suite").and_then(serde_json::Value::as_str), Some("ingest"));

    let obj = log.objects.get("f1").expect("f1");
    assert_eq!(obj.object_type, TestObjectType::Fixture);
    assert_eq!(obj.attributes.get("name").and_then(serde_json::Value::as_str), Some("db"));

    // e2 has no ocel:attributes: default empty map, case falls back to id.
    let e2 = log.events.get("e2").expect("e2");
    assert!(e2.attributes.is_empty());
    assert_eq!(e2.case_id, "run-42");
}

#[test]
fn json_import_refuses_invalid_version() {
    let json = r#"{"ocel:version": "1.0", "ocel:events": {}}"#;
    let err = import_ocel_json(json).expect_err("version 1.0 must refuse");
    match err {
        ImportError::InvalidVersion { found } => assert_eq!(found, "1.0"),
        other => panic!("expected InvalidVersion, got {other:?}"),
    }
}

// ── XML import: roundtrip-stable + MalformedXml refusal ─────────────────────

/// Export-shaped fixture: the exact envelope an ocel-xml exporter emits for
/// a two-event log (same shape as the import module's documented example).
fn xml_fixture() -> String {
    "<ocel:events version=\"2.0\">\n\
     <event id=\"e_0\" timestamp=\"9\" activity=\"TestStarted\">\n  \
     <object id=\"o_1\" type=\"Artifact\"/>\n  \
     <object id=\"o_2\" type=\"Receipt\"/>\n\
     </event>\n\
     <event id=\"e_1\" timestamp=\"10\" activity=\"DriftZero\" caseId=\"run_7\"/>\n\
     </ocel:events>"
        .to_string()
}

#[test]
fn xml_import_roundtrip_is_stable_and_field_exact() {
    let log = import_ocel_xml(&xml_fixture()).expect("fixture must import");
    assert_eq!(log.events.len(), 2);

    let e0 = log.events.get("e_0").expect("e_0");
    assert_eq!(e0.event_id, "e_0");
    assert_eq!(e0.timestamp_ns, 9);
    assert_eq!(e0.case_id, "e_0");
    assert!(matches!(e0.activity, TestActivity::TestStarted));
    assert_eq!(
        e0.objects,
        vec![
            ("o_1".to_string(), TestObjectType::Artifact),
            ("o_2".to_string(), TestObjectType::Receipt)
        ]
    );

    let e1 = log.events.get("e_1").expect("e_1");
    assert_eq!(e1.case_id, "run_7");
    assert!(matches!(e1.activity, TestActivity::DriftZero));
    assert!(e1.objects.is_empty());

    // Roundtrip stability: byte-identical input yields byte-identical decoded
    // state (deterministic reader) — the reader half of the export/import
    // roundtrip.
    let again = import_ocel_xml(&xml_fixture()).expect("second import");
    assert_eq!(again.events.len(), log.events.len());
    for (id, event) in &log.events {
        let other = again.events.get(id).expect("same id set");
        assert_eq!(event.event_id, other.event_id);
        assert_eq!(event.timestamp_ns, other.timestamp_ns);
        assert_eq!(event.case_id, other.case_id);
        assert_eq!(event.objects, other.objects);
        assert_eq!(
            serde_json::to_string(&event.activity).expect("activity json"),
            serde_json::to_string(&other.activity).expect("activity json")
        );
    }
}

#[test]
fn xml_import_refuses_malformed_xml() {
    // Unknown entity: a hard refusal, never a silent passthrough.
    let xml = "<ocel:events version=\"2.0\"><event id=\"&#65;\" timestamp=\"0\" \
               activity=\"DriftZero\"/></ocel:events>";
    let err = import_ocel_xml(xml).expect_err("numeric entity must refuse");
    assert!(matches!(err, XmlImportError::MalformedXml { .. }));
    assert!(err.to_string().contains("unknown entity"));

    // Unclosed root tag: also a MalformedXml-family refusal (UnclosedTag).
    let err =
        import_ocel_xml("<ocel:events version=\"2.0\">").expect_err("unclosed root must refuse");
    assert!(matches!(err, XmlImportError::UnclosedTag { tag, .. } if tag == "ocel:events"));
}

#[test]
fn xml_import_refuses_invalid_version() {
    let err =
        import_ocel_xml("<ocel:events version=\"1.0\"/>").expect_err("version 1.0 must refuse");
    assert!(matches!(
        err,
        XmlImportError::InvalidVersion { found } if found == "1.0"
    ));
}
