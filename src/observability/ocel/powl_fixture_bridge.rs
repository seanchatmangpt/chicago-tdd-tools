//! POWL fixture → conformance session words bridge (FR-FIXTURE-02, v26.10.12).
//!
//! Turns a merger-matrix OCEL 2.0 fixture (the
//! `examples/merger_matrix/conformance/*.ocel.json` family) into the PN64
//! session-word stream a [`crate::conformance::session::ConformanceSession`] replays: every fixture
//! event's `ocel:activity` is mapped to its DFS atom index in the POWL model
//! and packed as `tid | tick << 8` with ticks advancing `prev + 1`
//! per the word law in `crate::conformance::session`; traces longer than
//! 255 events are refused ([`FixtureBridgeError::TraceTooLong`]) rather
//! than wrapping the `u8` tick.
//!
//! # Two routes, one word stream
//!
//! * Route A — [`super::import_json::import_ocel_json`]: the standard OCEL
//!   2.0 shape validated and typed into [`OcelLog`] first. When the fixture
//!   carries the in-crate test-lifecycle activity vocabulary this route
//!   succeeds and its events feed the word packer.
//! * Route B — direct `ocel:*` map parse: fixtures whose activities are
//!   arbitrary merger-domain names (`ingest_isda_csa`, `pacs_intake`, …)
//!   cannot deserialize into [`TestActivity`], so [`import_ocel_json`]
//!   refuses them and the bridge parses the `ocel:events` map directly,
//!   keeping the raw activity strings.
//!
//! Both routes sort events by `(timestamp_ns, event_id)` — `OcelLog.events`
//! is a `HashMap`, so the sort is what makes the two routes agree word for
//! word regardless of map iteration order — and then apply the identical
//! name→tid map and tick law. Same fixture, same model, same words.
//!
//! # Fail-closed contract
//!
//! * malformed JSON / missing `ocel:activity` or `ocel:timestamp` /
//!   non-`"2.0"` version → typed [`FixtureBridgeError`] (route B parses the
//!   same required fields the importer enforces)
//! * an activity name that is not a DFS activity leaf of the model →
//!   [`FixtureBridgeError::UnknownActivity`], carrying the name and its
//!   position in the sorted trace; nothing partial is ever emitted.
//! * a trace longer than 255 events → [`FixtureBridgeError::TraceTooLong`]
//!   (the `u8` session tick byte would wrap and alias earlier words)

#![cfg(feature = "ocel-ingest-json")]
#![cfg(feature = "conformance")]

use super::import_json::import_ocel_json;
use super::types::{OcelLog, TestActivity};
use crate::conformance::loader::{PowlModel, PowlNode};

/// Typed refusal for the fixture → session-words bridge. Fail-closed: every
/// variant names the exact evidence, and no partial word stream is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureBridgeError {
    /// Input was not syntactically valid JSON.
    MalformedJson {
        /// `serde_json`'s parse failure description.
        message: String,
    },
    /// A required field was absent or not of the expected shape.
    MissingField {
        /// The `ocel:`-prefixed field name that was absent/malformed.
        field: &'static str,
        /// Where it was missing (document or event id).
        subject: String,
    },
    /// `ocel:version` was present but not `"2.0"`.
    InvalidVersion {
        /// The version string found.
        found: String,
    },
    /// An `ocel:activity` name is not a DFS activity leaf of the model.
    UnknownActivity {
        /// The offending activity name.
        name: String,
        /// Its 0-based position in the timestamp-sorted trace.
        position: usize,
    },
    /// The fixture declared no events; an empty session stream is refused
    /// downstream by [`ConformanceSession::attach_stream`](crate::conformance::session::ConformanceSession::attach_stream), so it is refused
    /// here too rather than handed on vacuously.
    EmptyTrace,
    /// The trace carried more than 255 events. The session tick byte is a
    /// `u8` advancing `prev + 1`; a 256th event would wrap the tick back to
    /// 0 and silently alias earlier words, so the trace is refused before
    /// any word is produced.
    TraceTooLong {
        /// The offending trace length.
        len: usize,
    },
}

impl core::fmt::Display for FixtureBridgeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MalformedJson { message } => write!(f, "malformed fixture JSON: {message}"),
            Self::MissingField { field, subject } => {
                write!(f, "missing required field: {field} (in {subject})")
            }
            Self::InvalidVersion { found } => {
                write!(f, "unsupported ocel:version {found:?} (only \"2.0\" accepted)")
            }
            Self::UnknownActivity { name, position } => {
                write!(f, "unknown activity {name:?} at trace position {position}")
            }
            Self::EmptyTrace => write!(f, "fixture declares no ocel:events"),
            Self::TraceTooLong { len } => write!(
                f,
                "trace length {len} exceeds the 255-event session tick-byte budget"
            ),
        }
    }
}

impl std::error::Error for FixtureBridgeError {}

/// One normalized event on the way to the word packer.
struct NamedEvent {
    event_id: String,
    timestamp_ns: u64,
    /// Raw activity name (route B) or the serialized variant tag (route A).
    activity: String,
}

/// Bridge a merger-matrix OCEL 2.0 fixture into PN64 session words against
/// `model` (FR-FIXTURE-02).
///
/// Route A ([`import_ocel_json`]) is attempted first; on its typed refusal
/// route B parses the `ocel:events` map directly. Both routes end at the
/// same words (see the module docs for the ordering argument).
///
/// # Errors
///
/// [`FixtureBridgeError`] on malformed JSON, missing required fields, a
/// non-`"2.0"` version, an activity name absent from the model's DFS atom
/// sequence, or a fixture with zero events.
pub fn merger_fixture_to_session_words(
    model: &PowlModel,
    fixture_json: &str,
) -> Result<Vec<u64>, FixtureBridgeError> {
    let events = match route_a_events(fixture_json) {
        Ok(events) => events,
        Err(()) => route_b_events(fixture_json)?,
    };
    words_from_named_events(model, &events)
}

// ---------------------------------------------------------------------------
// Route A: standard OCEL 2.0 shape through the typed importer
// ---------------------------------------------------------------------------

/// Route A: import through [`import_ocel_json`]. Returns `Err` when the
/// fixture's activity vocabulary is not the in-crate `TestActivity` set —
/// the caller falls back to route B. Never surfaces the importer's error:
/// it is only a route selector, the word-level contract is route B's parse.
fn route_a_events(fixture_json: &str) -> Result<Vec<NamedEvent>, ()> {
    let log: OcelLog = import_ocel_json(fixture_json).map_err(|_| ())?;
    let mut events: Vec<NamedEvent> = log
        .events
        .into_values()
        .map(|event| NamedEvent {
            event_id: event.event_id,
            timestamp_ns: event.timestamp_ns,
            activity: activity_tag(&event.activity),
        })
        .collect();
    events.sort_by(|a, b| (a.timestamp_ns, &a.event_id).cmp(&(b.timestamp_ns, &b.event_id)));
    Ok(events)
}

/// The externally-tagged serde tag of a [`TestActivity`] variant — the
/// route-A analogue of a raw activity name.
fn activity_tag(activity: &TestActivity) -> String {
    match serde_json::to_value(activity) {
        Ok(serde_json::Value::String(tag)) => tag,
        Ok(serde_json::Value::Object(fields)) => fields.keys().next().cloned().unwrap_or_default(),
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Route B: direct ocel:* map parse (merger-domain activity names)
// ---------------------------------------------------------------------------

/// Route B: parse the `ocel:events` map directly, keeping raw activity
/// strings. Enforces the same required-field and version law as
/// [`import_ocel_json`], fail-closed.
fn route_b_events(fixture_json: &str) -> Result<Vec<NamedEvent>, FixtureBridgeError> {
    let doc: serde_json::Value = serde_json::from_str(fixture_json)
        .map_err(|e| FixtureBridgeError::MalformedJson { message: e.to_string() })?;

    let version = doc.get("ocel:version").ok_or_else(|| FixtureBridgeError::MissingField {
        field: "ocel:version",
        subject: "<document>".to_string(),
    })?;
    let version = version
        .as_str()
        .ok_or_else(|| FixtureBridgeError::InvalidVersion { found: version.to_string() })?;
    if version != "2.0" {
        return Err(FixtureBridgeError::InvalidVersion { found: version.to_string() });
    }

    let events = doc.get("ocel:events").ok_or_else(|| FixtureBridgeError::MissingField {
        field: "ocel:events",
        subject: "<document>".to_string(),
    })?;
    let events = events.as_object().ok_or_else(|| FixtureBridgeError::MissingField {
        field: "ocel:events",
        subject: "<document>".to_string(),
    })?;

    let mut parsed: Vec<NamedEvent> = Vec::with_capacity(events.len());
    for (event_id, event) in events {
        let activity =
            event.get("ocel:activity").and_then(serde_json::Value::as_str).ok_or_else(|| {
                FixtureBridgeError::MissingField {
                    field: "ocel:activity",
                    subject: event_id.clone(),
                }
            })?;
        let timestamp_ns = event
            .get("ocel:timestamp")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| FixtureBridgeError::MissingField {
                field: "ocel:timestamp",
                subject: event_id.clone(),
            })?;
        parsed.push(NamedEvent {
            event_id: event_id.clone(),
            timestamp_ns,
            activity: activity.to_string(),
        });
    }
    parsed.sort_by(|a, b| (a.timestamp_ns, &a.event_id).cmp(&(b.timestamp_ns, &b.event_id)));
    Ok(parsed)
}

/// Strict E-M14 session-word law over explicit names against a phase model's
/// DFS atom sequence.
///
/// `session_word[i] = tid_i | (i+1) << 8`. A name outside the atom sequence
/// is a typed [`FixtureBridgeError::UnknownActivity`]. A stream longer than
/// 255 events is a typed [`FixtureBridgeError::TraceTooLong`] refusal — the
/// tick byte is a `u8` and a 256th event would wrap it back to 0, silently
/// aliasing earlier words.
///
/// This is the entry point for streams whose names were extracted upstream
/// (e.g. the contract wire form's `attributes["powl"]` field, which lives in
/// the rich OCEL log tier and never reaches the packed tier).
///
/// # Errors
///
/// [`FixtureBridgeError::UnknownActivity`] for a name not in the model's
/// DFS atoms; [`FixtureBridgeError::TraceTooLong`] for a stream longer than
/// 255 events.
pub fn session_words_for_atoms(
    names: &[String],
    atoms: &[String],
) -> Result<Vec<u64>, FixtureBridgeError> {
    if names.len() > 255 {
        return Err(FixtureBridgeError::TraceTooLong { len: names.len() });
    }
    let mut tick: u8 = 0;
    let mut words = Vec::with_capacity(names.len());
    for (position, name) in names.iter().enumerate() {
        let tid_index = atoms
            .iter()
            .position(|a| a == name)
            .ok_or_else(|| FixtureBridgeError::UnknownActivity { name: name.clone(), position })?;
        // JUSTIFICATION: tid_index < atoms.len() <= 256 enforced by the
        // session's MAX_ATOMS gate; narrowing is total.
        #[allow(clippy::cast_possible_truncation)]
        let tid = tid_index as u64;
        tick = tick.wrapping_add(1);
        // JUSTIFICATION: tick is u8; widening is total.
        #[allow(clippy::cast_possible_truncation)]
        let word = tid | (u64::from(tick) << 8);
        words.push(word);
    }
    Ok(words)
}

/// Relaxed E-M14 variant: a name outside the atom sequence encodes the
/// non-declared activity byte `0xFF`.
///
/// The net itself reports the divergence as `Undeclared` — the vocabulary
/// divergence stays loud at the NET, not at the mapper. Used by the phase-7
/// divergence court.
///
/// # Errors
///
/// Currently infallible for streams of at most 255 events (the session tick
/// byte law); longer streams wrap ticks at the u8 boundary exactly as the
/// session's own mod-256 tick law does.
#[must_use]
pub fn session_words_for_atoms_relaxed(names: &[String], atoms: &[String]) -> Vec<u64> {
    let mut tick: u8 = 0;
    names
        .iter()
        .map(|name| {
            // JUSTIFICATION: tid < 256 (atom sequence) or literal 0xFF;
            // narrowing is total.
            #[allow(clippy::cast_possible_truncation)]
            let tid = atoms.iter().position(|a| a == name).map_or(0xFF_u64, |tid_index| {
                #[allow(clippy::cast_possible_truncation)]
                let tid = tid_index as u64;
                tid
            });
            tick = tick.wrapping_add(1);
            // JUSTIFICATION: tick is u8; widening is total.
            #[allow(clippy::cast_possible_truncation)]
            let word = tid | (u64::from(tick) << 8);
            word
        })
        .collect()
}

/// Deterministic activity-name stream of a landed fixture.
///
/// The same `(timestamp_ns, event_id)` sort as
/// [`merger_fixture_to_session_words`], but the raw `ocel:activity` names
/// instead of words. The phase-7 vocabulary-divergence court needs the names
/// themselves (its words go through the relaxed mapper); every other court
/// asserts the names before mapping.
///
/// # Errors
///
/// [`FixtureBridgeError`] per `route_b_events`' fail-closed law.
pub fn fixture_activity_names(fixture_json: &str) -> Result<Vec<String>, FixtureBridgeError> {
    Ok(route_b_events(fixture_json)?.into_iter().map(|event| event.activity).collect())
}

// ---------------------------------------------------------------------------
// Word packing: DFS atom index + tick law
// ---------------------------------------------------------------------------

/// Pack the sorted named events into session words: `tid | tick << 8` with
/// `tid` = the activity's first DFS atom index and ticks advancing
/// `prev + 1` starting from 1 (the session admits any first tick;
/// 1 is the canonical start). Traces longer than 255 events are refused
/// with [`FixtureBridgeError::TraceTooLong`] before any word is produced —
/// the tick byte is a `u8` and a 256th event would wrap it back to 0,
/// silently aliasing earlier words; there is no mod-256 wrap on this path.
fn words_from_named_events(
    model: &PowlModel,
    events: &[NamedEvent],
) -> Result<Vec<u64>, FixtureBridgeError> {
    if events.is_empty() {
        return Err(FixtureBridgeError::EmptyTrace);
    }
    if events.len() > 255 {
        return Err(FixtureBridgeError::TraceTooLong { len: events.len() });
    }
    let atom_names = dfs_atom_names(&model.root);
    let mut words = Vec::with_capacity(events.len());
    let mut tick: u8 = 0;
    for (position, event) in events.iter().enumerate() {
        let tid_index =
            atom_names.iter().position(|name| name == &event.activity).ok_or_else(|| {
                FixtureBridgeError::UnknownActivity { name: event.activity.clone(), position }
            })?;
        // JUSTIFICATION: tid_index < atom count <= u8::MAX + 1 enforced by
        // the session's MAX_ATOMS gate; the usize -> u8 narrowing is total.
        #[allow(clippy::cast_possible_truncation)]
        let tid = tid_index as u64;
        tick = tick.wrapping_add(1);
        // JUSTIFICATION: tick is a u8 already; widening is total.
        #[allow(clippy::cast_possible_truncation)]
        let word = tid | (u64::from(tick) << 8);
        words.push(word);
    }
    Ok(words)
}

/// Depth-first, left-to-right activity-leaf names of a loader-lane model —
/// the same traversal [`crate::conformance::session`] uses to assign
/// `tid = DFS index`, including its `Silent`-contributes-nothing law and
/// its `Loop { body, redo }` ordering.
fn dfs_atom_names(node: &PowlNode) -> Vec<String> {
    match node {
        PowlNode::Activity(name) => vec![name.clone()],
        PowlNode::Silent => Vec::new(),
        PowlNode::Sequence(children) | PowlNode::Xor(children) | PowlNode::Parallel(children) => {
            children.iter().flat_map(dfs_atom_names).collect()
        }
        PowlNode::Loop { body, redo, .. } => {
            dfs_atom_names(body).into_iter().chain(dfs_atom_names(redo)).collect()
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Absolute repo-root-relative fixture path via CARGO_MANIFEST_DIR so
    /// the tests run the real on-disk merger-matrix capital (Chicago-style
    /// real collaborators, no fixtures synthesized in-test for the golden
    /// paths).
    fn example_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/merger_matrix")
    }

    fn load_model(phase: &str) -> PowlModel {
        let path = example_dir().join(format!("models/{phase}.powl"));
        PowlModel::from_powl_file(&path).unwrap_or_else(|e| panic!("load {phase}.powl: {e}"))
    }

    fn load_fixture(phase: &str) -> String {
        std::fs::read_to_string(example_dir().join(format!("conformance/{phase}.ocel.json")))
            .unwrap_or_else(|e| panic!("read {phase}.ocel.json: {e}"))
    }

    /// Word accessor helpers matching the session word law.
    fn word_tid(word: u64) -> u64 {
        word & 0xFF
    }

    fn word_tick(word: u64) -> u8 {
        ((word >> 8) & 0xFF) as u8
    }

    /// FR-FIXTURE-02 golden: the phase4 novation fixture maps to 5 session
    /// words against its POWL model, ticks 1..=5, tids within the atom
    /// budget — and the stream replays to `Complete` in a real session.
    #[test]
    fn phase4_fixture_maps_and_replays_complete() {
        let model = load_model("phase4_novation");
        let fixture = load_fixture("phase4_novation");
        let words = merger_fixture_to_session_words(&model, &fixture)
            .unwrap_or_else(|e| panic!("bridge: {e}"));

        assert_eq!(words.len(), 5, "one word per fixture event");
        for (i, word) in words.iter().enumerate() {
            assert_eq!(word_tick(*word), (i + 1) as u8, "tick law at {i}");
        }
        assert_eq!(word_tid(words[0]), 0, "ingest_isda_csa is the first DFS atom");

        // Real end-to-end: the same words attach and complete a session.
        let mut session = crate::conformance::session::ConformanceSession::from_model(&model)
            .unwrap_or_else(|e| panic!("compile: {e}"));
        session.attach_stream(&words).unwrap_or_else(|e| panic!("attach: {e}"));
        match session.step_budget(16) {
            crate::conformance::session::YieldStatus::Complete(_) => {}
            other => panic!("expected Complete replay, got {other:?}"),
        }
    }

    /// FR-FIXTURE-02 golden: the phase7 settlement fixture (11 events, with
    /// repeated loop-body activities) maps against its model; ticks advance
    /// 1..=11 and every tid names a declared atom.
    #[test]
    fn phase7_fixture_maps_with_repeated_activities() {
        let model = load_model("phase7_settlement");
        let fixture = load_fixture("phase7_settlement");
        let words = merger_fixture_to_session_words(&model, &fixture)
            .unwrap_or_else(|e| panic!("bridge: {e}"));

        assert_eq!(words.len(), 11);
        for (i, word) in words.iter().enumerate() {
            assert_eq!(word_tick(*word), ((i + 1) % 256) as u8, "tick law at {i}");
        }
        let atom_count = dfs_atom_names(&model.root).len();
        for word in &words {
            assert!((word_tid(*word) as usize) < atom_count, "tid in atoms");
        }
    }

    /// Fail-closed: an activity name absent from the model is refused with
    /// the typed variant carrying the name and trace position.
    #[test]
    fn unknown_activity_name_is_refused() {
        let model = load_model("phase4_novation");
        let fixture =
            load_fixture("phase4_novation").replace("ingest_isda_csa", "totally_unknown_activity");
        let err = merger_fixture_to_session_words(&model, &fixture)
            .expect_err("unknown name must be refused");
        match err {
            FixtureBridgeError::UnknownActivity { name, position } => {
                assert_eq!(name, "totally_unknown_activity");
                assert_eq!(position, 0);
            }
            other => panic!("expected UnknownActivity, got {other:?}"),
        }
    }

    /// Route A/B agreement: a fixture whose activities ARE in the
    /// `TestActivity` vocabulary imports through route A and yields the
    /// same words the direct parse yields for the same document shape.
    #[test]
    fn route_a_and_route_b_agree_on_test_activity_vocabulary() {
        let doc = r#"{
            "ocel:version": "2.0",
            "ocel:events": {
                "e2": {"ocel:activity": "ReceiptGenerated", "ocel:timestamp": 2},
                "e1": {"ocel:activity": "TestStarted", "ocel:timestamp": 1}
            }
        }"#;
        // Model whose single atom is named for the first tag encountered.
        let model = crate::conformance::loader::PowlModel {
            activities: std::collections::BTreeMap::new(),
            declaration_order: Vec::new(),
            root: PowlNode::Sequence(vec![
                PowlNode::Activity("TestStarted".to_string()),
                PowlNode::Activity("ReceiptGenerated".to_string()),
            ]),
        };
        let words =
            merger_fixture_to_session_words(&model, doc).unwrap_or_else(|e| panic!("bridge: {e}"));
        assert_eq!(words.len(), 2);
        assert_eq!(word_tid(words[0]), 0, "TestStarted first");
        assert_eq!(word_tid(words[1]), 1);
        assert_eq!(word_tick(words[0]), 1);
        assert_eq!(word_tick(words[1]), 2);
    }

    /// Determinism: parsing the same fixture twice yields byte-identical
    /// words despite `HashMap` iteration order in route A's OcelLog.
    #[test]
    fn repeated_parse_is_deterministic() {
        let model = load_model("phase4_novation");
        let fixture = load_fixture("phase4_novation");
        let a = merger_fixture_to_session_words(&model, &fixture).unwrap();
        let b = merger_fixture_to_session_words(&model, &fixture).unwrap();
        assert_eq!(a, b);
    }

    /// Fail-closed on malformed JSON and on a missing required field.
    #[test]
    fn malformed_and_missing_field_refused() {
        let model = load_model("phase4_novation");
        assert!(matches!(
            merger_fixture_to_session_words(&model, "not json"),
            Err(FixtureBridgeError::MalformedJson { .. })
        ));
        let no_ts = r#"{
            "ocel:version": "2.0",
            "ocel:events": {"e1": {"ocel:activity": "ingest_isda_csa"}}
        }"#;
        assert!(matches!(
            merger_fixture_to_session_words(&model, no_ts),
            Err(FixtureBridgeError::MissingField { field: "ocel:timestamp", .. })
        ));
    }

    /// An empty `ocel:events` map is refused, never handed on as a vacuous
    /// empty stream.
    #[test]
    fn empty_trace_refused() {
        let model = load_model("phase4_novation");
        let empty = r#"{"ocel:version": "2.0", "ocel:events": {}}"#;
        assert!(matches!(
            merger_fixture_to_session_words(&model, empty),
            Err(FixtureBridgeError::EmptyTrace)
        ));
    }

    /// Fail-closed: a 256-event trace would wrap the u8 session tick byte
    /// back to 0, silently aliasing earlier words — refused with the typed
    /// length. 255 events (the exact budget) still map.
    #[test]
    fn trace_longer_than_255_is_refused() {
        let atoms = vec!["a".to_string(), "b".to_string()];

        // Strict mapper: exactly at budget succeeds with ticks 1..=255.
        let ok_names: Vec<String> =
            (0..255).map(|i| atoms[i % 2].to_string()).collect();
        let words = session_words_for_atoms(&ok_names, &atoms)
            .unwrap_or_else(|e| panic!("255 events must map: {e}"));
        assert_eq!(words.len(), 255);
        assert_eq!(word_tick(words[254]), 255, "last tick is the u8 ceiling");

        // One more event wraps the tick — refused before any word is made.
        let too_long: Vec<String> = (0..256).map(|i| atoms[i % 2].to_string()).collect();
        assert_eq!(
            session_words_for_atoms(&too_long, &atoms),
            Err(FixtureBridgeError::TraceTooLong { len: 256 })
        );

        // The bridge's internal packer enforces the same law: a synthesized
        // 256-event fixture document through route B is refused.
        let events: String = (0..256)
            .map(|i| {
                format!(
                    r#""e{i:03}": {{"ocel:activity": "ingest_isda_csa", "ocel:timestamp": {i}}}"#
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let doc = format!(r#"{{"ocel:version": "2.0", "ocel:events": {{{events}}}}}"#);
        let model = load_model("phase4_novation");
        assert_eq!(
            merger_fixture_to_session_words(&model, &doc),
            Err(FixtureBridgeError::TraceTooLong { len: 256 })
        );
    }

    /// `session_words_for_atoms` refusal law: a name absent from the atom
    /// sequence is the typed variant carrying the offending name and its
    /// 0-based position in the input stream.
    #[test]
    fn session_words_for_atoms_unknown_activity_is_typed() {
        let atoms = vec!["a".to_string(), "b".to_string()];
        let names: Vec<String> = ["a", "b", "not_an_atom", "b"].iter().map(|s| s.to_string()).collect();
        assert_eq!(
            session_words_for_atoms(&names, &atoms),
            Err(FixtureBridgeError::UnknownActivity {
                name: "not_an_atom".to_string(),
                position: 2,
            })
        );
    }

    /// Relaxed mapper contract: a name outside the atom sequence encodes
    /// the non-declared activity byte `0xFF` at its position (the NET
    /// reports the divergence) while declared names keep their DFS tid and
    /// the tick law advances through unknowns unchanged.
    #[test]
    fn relaxed_mapper_encodes_unknown_names_as_0xff() {
        let atoms = vec!["a".to_string(), "b".to_string()];
        let names: Vec<String> =
            ["a", "not_an_atom", "b", "also_missing"].iter().map(|s| s.to_string()).collect();
        let words = session_words_for_atoms_relaxed(&names, &atoms);
        assert_eq!(words.len(), 4, "one word per name, nothing partial dropped");
        assert_eq!(word_tid(words[0]), 0, "declared name keeps its DFS tid");
        assert_eq!(word_tid(words[1]), 0xFF, "undeclared name encodes 0xFF");
        assert_eq!(word_tid(words[2]), 1);
        assert_eq!(word_tid(words[3]), 0xFF);
        for (i, word) in words.iter().enumerate() {
            assert_eq!(word_tick(*word), (i + 1) as u8, "tick law at {i}");
        }
    }
}
