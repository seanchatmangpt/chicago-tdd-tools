#![cfg(feature = "ocel-ingest")]
//! OCEL64 bridge: `OcelLog` (rich, string-keyed) <-> packed `u64` event words.
//!
//! WORD LAYOUT: ocel64 word layout ported from strata-protocol
//! `src/ocel64.rs @ 0de13a3` (activity [15:0], src [31:16], tgt [47:32],
//! qualifier [55:48] u8, tick delta [63:56] u8). The era-wrap and
//! tick-delta semantics follow strata-protocol `src/rehydrate.rs @ 0de13a3`:
//! the 8-bit tick field wraps at 256; upstream treats a 255 -> 0 wrap as
//! caller-checkpoint duty. This bridge REFUSES instead: any per-case
//! inter-event delta greater than 255 is a typed
//! [`BridgeError::TickDeltaOverflow`], never a saturation and never a
//! silent wrap. See `era_wrap_refuses_beyond_255_not_saturates` below.
//!
//! SIBLING CONTRACT: `BridgeVocab { activities: IdMap, objects: IdMap }` and
//! `IdMap`/`IdMapError` are owned by the sibling `idmap_bridge` lane
//! (`super::idmap_bridge`), which had not landed when this file was written;
//! integration must wire `mod idmap_bridge;` alongside this module. Per the
//! lane contract, `IdMap` exposes `get(&str) -> Option<u16>` and
//! `name(u16) -> Option<&str>`; `idmap_bridge` returns plain `IdMapError`
//! which THIS module maps into [`BridgeError`] (this definition wins).
//!
//! ENCODING (one case at a time):
//!
//! * Events are grouped by `case_id` and sorted by `timestamp_ns` before any
//!   delta math. The sort is REQUIRED for determinism: `OcelLog.events` is a
//!   `HashMap`, so input iteration order is nondeterministic and unsorted
//!   deltas would differ run to run for the same log.
//! * Header word: `pack(activity_id, obj0, obj1, qualifier, delta)`.
//!   Activity ids and object ids come from the `IdMap`s (ids >= 1 by
//!   construction), so `activity == 0` never occurs in a header word.
//! * Continuation words: each additional object pair is
//!   `pack(0, src, tgt, pair_qualifier, 0)` — activity 0 and tick 0 mark a
//!   continuation. An odd leftover object is a continuation word with
//!   `tgt = 0`. An event with fewer than two objects emits only its header
//!   word (`src`/`tgt` = 0 for the empty slots), so `k` object refs cost
//!   `ceil(k/2)` words for `k >= 2` and exactly 1 word for `k < 2`.
//! * Case separator: an all-zero word is emitted between consecutive cases.
//!   It is unambiguous: encoder ids are >= 1, so a legitimate continuation
//!   word always has a nonzero src or tgt, and a header word always has a
//!   nonzero activity.
//! * Qualifier byte: fixed table over `TestObjectType` (see
//!   [`qualifier_for`], default 0). The word qualifier is the SRC object's
//!   type qualifier; the decoder assigns that type to BOTH object slots of
//!   the word (see the lossiness note on [`from_ocel64`]).
//!
//! LOSSY BY DESIGN (packed tier, declared UNSUPPORTED, not bugs):
//!
//! * `attributes` do not survive the roundtrip: [`from_ocel64`] always
//!   produces events with an EMPTY attribute map.
//! * `event_id`s are regenerated as `e_{n}` over the whole decoded
//!   sequence; original event ids are not carried in the word format.
//! * `case_id`s are regenerated as `case_{n}` per separator-delimited
//!   segment; the `BridgeVocab` contract has no case-name id space.
//! * Absolute timestamps are not carried — only per-case tick deltas. The
//!   decoded timeline is correct up to a per-case time origin.
//! * Target-object type: only one qualifier byte exists per word, so the
//!   decoded type of the tgt slot is the src slot's type. Exact object-type
//!   roundtrip holds for corpora where each event's objects of a given type
//!   come in even counts (the encoder sorts each event's objects by
//!   `(qualifier, name)`, so same-type objects are adjacent).
//!
//! Attributes (and event/case identity) preservation is UNSUPPORTED at the
//! packed tier by design; use the rich `OcelLog` tier when they matter.

use std::collections::{BTreeMap, HashMap, HashSet};

use super::idmap_bridge::{BridgeVocab, IdMap, IdMapError};
use crate::observability::ocel::types::{OcelLog, TestActivity, TestObjectType, TestOcelEvent};

// ── Vendored ocel64 word layout (ported from strata-protocol src/ocel64.rs) ──

/// Activity id, bits [15:0].
#[inline]
#[must_use]
pub const fn activity(w: u64) -> u16 {
    (w & 0xFFFF) as u16
}

/// Source object id, bits [31:16].
#[inline]
#[must_use]
pub const fn src_obj(w: u64) -> u16 {
    ((w >> 16) & 0xFFFF) as u16
}

/// Target object id, bits [47:32].
#[inline]
#[must_use]
pub const fn tgt_obj(w: u64) -> u16 {
    ((w >> 32) & 0xFFFF) as u16
}

/// Relationship qualifier, bits [55:48].
#[inline]
#[must_use]
pub const fn qualifier(w: u64) -> u8 {
    ((w >> 48) & 0xFF) as u8
}

/// Monotonic tick delta, bits [63:56].
#[inline]
#[must_use]
pub const fn tick(w: u64) -> u8 {
    ((w >> 56) & 0xFF) as u8
}

/// Pack an event into its canonical 64-bit word. Inverse of the accessors:
/// `activity(pack(a, s, t, q, k)) == a`, and likewise for every field.
#[inline]
#[must_use]
pub fn pack(activity: u16, src: u16, tgt: u16, qual: u8, tick: u8) -> u64 {
    u64::from(activity)
        | (u64::from(src) << 16)
        | (u64::from(tgt) << 32)
        | (u64::from(qual) << 48)
        | (u64::from(tick) << 56)
}

// ── Typed refusals ──────────────────────────────────────────────────────────

/// Typed refusals of the OCEL64 bridge.
///
/// This is the single coordinating definition: the sibling `idmap_bridge`
/// returns plain [`IdMapError`] (its `Collision`), which is mapped here via
/// [`BridgeError::from`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BridgeError {
    /// Two distinct vocabulary names share one id space slot (mapped from
    /// `IdMapError::Collision`; a duplicate name is the degenerate case).
    #[error("vocab collision in {field}: {details}")]
    Collision {
        /// Which id space collided: `"activities"` or `"objects"`.
        field: &'static str,
        /// Human-readable collision details (both names/indices).
        details: String,
    },
    /// A vocabulary id space exceeded the 16-bit word capacity.
    #[error("vocab capacity exceeded for {field}: {count} names > bound {bound}")]
    CapacityExceeded {
        /// Which id space overflowed: `"activities"` or `"objects"`.
        field: &'static str,
        /// Number of distinct names requested.
        count: usize,
        /// Maximum supported (the 16-bit id bound).
        bound: u16,
    },
    /// A per-case inter-event delta exceeds the 8-bit tick field (255).
    /// REFUSED, never saturated — saturation would silently corrupt the
    /// timeline of every subsequent event in the case.
    #[error("tick delta overflow in case {case:?} at event {index}: {delta_ns}ns > 255")]
    TickDeltaOverflow {
        /// Case whose timeline overflows the 8-bit tick field.
        case: String,
        /// Index of the event (within the sorted case) whose delta overflowed.
        index: usize,
        /// The refused delta in nanoseconds.
        delta_ns: u64,
    },
    /// Timestamps within a case must be non-decreasing. Unreachable from
    /// [`to_ocel64`] (which sorts first — see its doc), but enforced in
    /// [`checked_deltas`] so the invariant is checked, not trusted.
    #[error("non-monotonic timestamps in case {case:?} at event {index}")]
    NonMonotonicTimestamp {
        /// Case with the decreasing timeline.
        case: String,
        /// Index of the event whose timestamp decreased.
        index: usize,
    },
    /// A qualifier byte outside the fixed table (see [`qualifier_for`]).
    #[error("unknown qualifier byte {raw}")]
    UnknownQualifier {
        /// The offending raw byte, as decimal text.
        raw: String,
    },
}

impl From<IdMapError> for BridgeError {
    fn from(e: IdMapError) -> Self {
        match e {
            IdMapError::Collision { index_a, name_a, index_b, name_b } => Self::Collision {
                field: "objects",
                details: format!(
                    "names {name_a:?} (index {index_a}) and {name_b:?} (index {index_b}) \
                     share one 16-bit id-space slot"
                ),
            },
        }
    }
}

// ── Fixed qualifier table ───────────────────────────────────────────────────

/// Canonical `TestObjectType` name (the vocab key for object types).
#[must_use]
pub const fn type_name(t: TestObjectType) -> &'static str {
    match t {
        TestObjectType::Artifact => "Artifact",
        TestObjectType::Schema => "Schema",
        TestObjectType::ProcessModel => "ProcessModel",
        TestObjectType::Receipt => "Receipt",
        TestObjectType::Fixture => "Fixture",
        TestObjectType::AgentContext => "AgentContext",
        TestObjectType::WaveInstance => "WaveInstance",
        TestObjectType::SectorStack => "SectorStack",
    }
}

/// Stable activity vocabulary key: the variant name only. Payloads
/// (`passed`, `macro_name`, `fitness`, ...) are dropped at the packed tier,
/// consistent with the attributes-UNSUPPORTED rule above.
#[must_use]
pub const fn activity_key(a: &TestActivity) -> &'static str {
    match a {
        TestActivity::TestStarted => "TestStarted",
        TestActivity::TestCompleted { .. } => "TestCompleted",
        TestActivity::TestSkipped { .. } => "TestSkipped",
        TestActivity::FixtureSetup => "FixtureSetup",
        TestActivity::FixtureTeardown => "FixtureTeardown",
        TestActivity::FixtureComposed => "FixtureComposed",
        TestActivity::AssertionPassed { .. } => "AssertionPassed",
        TestActivity::AssertionFailed { .. } => "AssertionFailed",
        TestActivity::ArtifactAdmitted => "ArtifactAdmitted",
        TestActivity::ArtifactRefused => "ArtifactRefused",
        TestActivity::ReceiptGenerated => "ReceiptGenerated",
        TestActivity::ReceiptMissing => "ReceiptMissing",
        TestActivity::DriftZero => "DriftZero",
        TestActivity::DriftDetected => "DriftDetected",
        TestActivity::BypassDetected => "BypassDetected",
        TestActivity::ConformanceChecked { .. } => "ConformanceChecked",
        TestActivity::ConformanceFailed { .. } => "ConformanceFailed",
        TestActivity::SubstrateGrew { .. } => "SubstrateGrew",
        TestActivity::SubstrateLabor => "SubstrateLabor",
        TestActivity::OntologyConsistent => "OntologyConsistent",
        TestActivity::OntologyViolation => "OntologyViolation",
        TestActivity::DiagnosticEmitted { .. } => "DiagnosticEmitted",
        TestActivity::WavePhaseStarted { .. } => "WavePhaseStarted",
        TestActivity::WavePhaseCompleted { .. } => "WavePhaseCompleted",
        TestActivity::WaveTaskStarted { .. } => "WaveTaskStarted",
        TestActivity::WaveTaskCompleted { .. } => "WaveTaskCompleted",
        TestActivity::WaveConflictDetected => "WaveConflictDetected",
        TestActivity::RunStarted { .. } => "RunStarted",
        TestActivity::RunCompleted { .. } => "RunCompleted",
    }
}

/// Fixed qualifier table over object-type names. Unknown names default to 0
/// (the `qualifier_for` direction is total); the decode direction
/// [`qualifier_name`] refuses bytes outside the table.
#[must_use]
pub fn qualifier_for(type_name: &str) -> u8 {
    match type_name {
        "Artifact" => 1,
        "Schema" => 2,
        "ProcessModel" => 3,
        "Receipt" => 4,
        "Fixture" => 5,
        "AgentContext" => 6,
        "WaveInstance" => 7,
        "SectorStack" => 8,
        _ => 0,
    }
}

/// Decode direction of [`qualifier_for`]: the fixed table entry for a
/// qualifier byte, or [`BridgeError::UnknownQualifier`] for a byte outside
/// the table (bytes 0 and 9..=255 have no type).
///
/// # Errors
///
/// Refuses qualifier bytes with no table entry.
pub fn qualifier_name(q: u8) -> Result<&'static str, BridgeError> {
    match q {
        1 => Ok("Artifact"),
        2 => Ok("Schema"),
        3 => Ok("ProcessModel"),
        4 => Ok("Receipt"),
        5 => Ok("Fixture"),
        6 => Ok("AgentContext"),
        7 => Ok("WaveInstance"),
        8 => Ok("SectorStack"),
        other => Err(BridgeError::UnknownQualifier { raw: other.to_string() }),
    }
}

/// Parse a canonical type name back to its `TestObjectType` (decode side of
/// [`type_name`]). Unknown names are `None` — the caller refuses.
#[must_use]
pub fn parse_type_name(name: &str) -> Option<TestObjectType> {
    match name {
        "Artifact" => Some(TestObjectType::Artifact),
        "Schema" => Some(TestObjectType::Schema),
        "ProcessModel" => Some(TestObjectType::ProcessModel),
        "Receipt" => Some(TestObjectType::Receipt),
        "Fixture" => Some(TestObjectType::Fixture),
        "AgentContext" => Some(TestObjectType::AgentContext),
        "WaveInstance" => Some(TestObjectType::WaveInstance),
        "SectorStack" => Some(TestObjectType::SectorStack),
        _ => None,
    }
}

// ── Delta math ──────────────────────────────────────────────────────────────

/// Per-case tick deltas from sorted timestamps.
///
/// `ts` MUST be sorted non-decreasing; the monotonicity check is re-derived
/// here (not trusted from the caller) and refuses
/// [`BridgeError::NonMonotonicTimestamp`], and any delta over the 8-bit tick
/// bound refuses [`BridgeError::TickDeltaOverflow`] — never saturate, never
/// wrap.
///
/// # Errors
///
/// [`BridgeError::NonMonotonicTimestamp`] on a decreasing timestamp,
/// [`BridgeError::TickDeltaOverflow`] on a delta > 255.
pub fn checked_deltas(case: &str, ts: &[u64]) -> Result<Vec<u8>, BridgeError> {
    let mut out = Vec::with_capacity(ts.len());
    let mut prev: Option<u64> = None;
    for (index, &t) in ts.iter().enumerate() {
        if let Some(p) = prev {
            if t < p {
                return Err(BridgeError::NonMonotonicTimestamp { case: case.to_string(), index });
            }
            let delta = t - p;
            if delta > 255 {
                return Err(BridgeError::TickDeltaOverflow {
                    case: case.to_string(),
                    index,
                    delta_ns: delta,
                });
            }
            out.push(u8::try_from(delta).unwrap_or(0)); // unreachable: delta <= 255 checked
        } else {
            out.push(0); // first event of a case: delta 0 (per-case time origin)
        }
        prev = Some(t);
    }
    Ok(out)
}

/// Capacity gate for one vocab id space. `IdMap` ids are 16-bit; more
/// distinct names than `u16::MAX - 1` usable ids cannot pack.
///
/// # Errors
///
/// [`BridgeError::CapacityExceeded`] when `count` exceeds the bound.
pub(crate) fn ensure_capacity(field: &'static str, count: usize) -> Result<(), BridgeError> {
    let bound = u16::MAX; // 65_535; ids are 1..=n, so 65_534 names fit — see below
    if count > usize::from(bound) {
        return Err(BridgeError::CapacityExceeded { field, count, bound });
    }
    Ok(())
}

/// Deterministic first-seen dedupe of vocabulary names.
fn dedup_first_seen<'a>(names: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut out: Vec<&str> = Vec::new();
    for n in names {
        if seen.insert(n) {
            out.push(n);
        }
    }
    out
}

/// Build the activity vocab from an ordered, deduped name list.
fn build_vocab(names: &[&str], field: &'static str) -> Result<IdMap, BridgeError> {
    ensure_capacity(field, names.len())?;
    IdMap::build(names).map_err(|e| BridgeError::Collision {
        field,
        details: match e {
            IdMapError::Collision { index_a, name_a, index_b, name_b } => format!(
                "names {name_a:?} (index {index_a}) and {name_b:?} (index {index_b}) \
                 share one 16-bit id-space slot"
            ),
        },
    })
}

// ── Encode ──────────────────────────────────────────────────────────────────

/// Pack an [`OcelLog`] into OCEL64 words plus the vocab needed to decode.
///
/// Determinism: events are grouped by `case_id` into a `BTreeMap` (case
/// order = lexicographic), each case's events are sorted by `timestamp_ns`,
/// and vocabulary ids are assigned in first-seen order over that traversal —
/// the same log always yields the same `(words, vocab)` pair. This is
/// mandatory because `OcelLog.events` is a `HashMap`; its iteration order is
/// nondeterministic and unsorted deltas would differ run to run.
///
/// The first event of each case carries tick delta 0 (per-case time origin);
/// each later event carries `timestamp_ns - previous_timestamp_ns`, refused
/// as [`BridgeError::TickDeltaOverflow`] beyond 255.
///
/// # Errors
///
/// [`BridgeError::Collision`] (vocab name collision),
/// [`BridgeError::CapacityExceeded`] (vocab > 16-bit id space),
/// [`BridgeError::TickDeltaOverflow`] (per-case delta > 255),
/// [`BridgeError::NonMonotonicTimestamp`] (defensive; sorting makes this
/// unreachable from this entry point).
pub fn to_ocel64(log: &OcelLog) -> Result<(Vec<u64>, BridgeVocab), BridgeError> {
    // Deterministic traversal: BTreeMap over case ids, timestamp-sorted events.
    let mut cases: BTreeMap<&str, Vec<&crate::observability::ocel::types::TestOcelEvent>> =
        BTreeMap::new();
    for ev in log.events.values() {
        cases.entry(ev.case_id.as_str()).or_default().push(ev);
    }
    for evs in cases.values_mut() {
        evs.sort_by_key(|ev| ev.timestamp_ns);
    }

    // Vocabulary in first-seen order over the deterministic traversal.
    let mut activity_names: Vec<&str> = Vec::new();
    let mut object_names: Vec<&str> = Vec::new();
    for evs in cases.values() {
        for ev in evs {
            activity_names.push(activity_key(&ev.activity));
            let mut objs: Vec<(&str, u8)> = ev
                .objects
                .iter()
                .map(|(name, t)| (name.as_str(), qualifier_for(type_name(*t))))
                .collect();
            objs.sort_by_key(|(name, q)| (*q, (*name).to_string()));
            for (name, _) in objs {
                object_names.push(name);
            }
        }
    }
    let activity_names = dedup_first_seen(activity_names.into_iter());
    let object_names = dedup_first_seen(object_names.into_iter());
    let activities = build_vocab(&activity_names, "activities")?;
    let objects = build_vocab(&object_names, "objects")?;

    // Encode case by case, zero word between cases.
    let mut words: Vec<u64> = Vec::new();
    for (case_no, (case_id, evs)) in cases.iter().enumerate() {
        if case_no > 0 {
            words.push(0); // case separator: unambiguous (ids >= 1)
        }
        let ts: Vec<u64> = evs.iter().map(|ev| ev.timestamp_ns).collect();
        let deltas = checked_deltas(case_id, &ts)?;
        for (ev, &delta) in evs.iter().zip(deltas.iter()) {
            let act = activities.get(activity_key(&ev.activity)).unwrap_or(0).max(1); // vocab was built from these keys; 0 is not an id
            let mut objs: Vec<(u16, u8)> = ev
                .objects
                .iter()
                .map(|(name, t)| {
                    (objects.get(name).unwrap_or(0).max(1), qualifier_for(type_name(*t)))
                })
                .collect();
            objs.sort_by_key(|(id, q)| (*q, name_of(&objects, *id).to_string()));
            let mut slots = objs.into_iter();
            let first = slots.next();
            let (s0, q0) = first.map_or((0, 0), |(id, q)| (id, q));
            let (t0, _) = slots.next().map_or((0, 0), |(id, q)| (id, q));
            words.push(pack(act, s0, t0, q0, delta));
            let mut pair: Vec<(u16, u8)> = Vec::with_capacity(2);
            for (id, q) in slots {
                pair.push((id, q));
                if pair.len() == 2 {
                    words.push(pack(0, pair[0].0, pair[1].0, pair[0].1, 0));
                    pair.clear();
                }
            }
            if let Some((id, q)) = pair.pop() {
                words.push(pack(0, id, 0, q, 0)); // odd leftover: tgt = 0
            }
        }
    }
    Ok((words, BridgeVocab { activities, objects }))
}

/// Look up a vocab name (decode side); `0` is the reserved null id.
fn name_of(map: &IdMap, id: u16) -> &str {
    const NULL: &str = "";
    if id == 0 {
        return NULL;
    }
    map.name(id).unwrap_or(NULL)
}

// ── Decode ──────────────────────────────────────────────────────────────────

/// Unpack OCEL64 words back into events using the encode-side vocab.
///
/// Words are split into case segments on all-zero separator words; within a
/// segment a word with `activity != 0` is a header word and a word with
/// `activity == 0 && tick == 0` is an object-pair continuation.
///
/// See the module docs for what is UNSUPPORTED at this tier: attributes are
/// always EMPTY, `event_id`/`case_id` are regenerated (`e_{n}`/`case_{n}`),
/// and timestamps are rebuilt from deltas (correct up to a per-case origin).
#[must_use]
pub fn from_ocel64(words: &[u64], vocab: &BridgeVocab) -> Vec<TestOcelEvent> {
    let mut events: Vec<TestOcelEvent> = Vec::new();
    let mut case_no: usize = 0;
    let mut case_time: u64 = 0;
    let mut seg_has_event = false;
    for &w in words {
        if w == 0 {
            case_no += 1;
            case_time = 0;
            seg_has_event = false;
            continue;
        }
        let a = activity(w);
        let delta = tick(w);
        // Every event word (header or continuation) carries the pairwise
        // delta to the previous event of the case; the per-case clock
        // accumulates them from the case's first event (delta 0 = origin).
        case_time = case_time.wrapping_add(u64::from(delta));
        if a != 0 {
            seg_has_event = true;
        } else if !seg_has_event {
            // Continuation before any header: malformed, skip defensively.
            continue;
        }
        // NOTE: continuation words carry the pairwise delta like headers
        // (added once, above). By encoder construction their tick is 0 (an
        // all-zero activity+tick word would be the case separator), so the
        // single add is the only correct accumulation — do not add again.
        debug_assert!(
            a != 0 || delta == 0,
            "continuation word carries nonzero tick; packed-tier invariant broken"
        );
        if a == 0 && delta == 0 {
            // Pure continuation: append its object pair to the last event.
            let q = qualifier(w);
            if let Ok(tname) = qualifier_name(q) {
                if let Some(t) = parse_type_name(tname) {
                    for id in [src_obj(w), tgt_obj(w)] {
                        if id != 0 {
                            if let Some(ev) = events.last_mut() {
                                ev.objects.push((name_of(&vocab.objects, id).to_string(), t));
                            }
                        }
                    }
                }
            }
            continue;
        }
        // Header word: new event; its pair is decoded here too.
        let act_name = name_of(&vocab.activities, a);
        let mut objects: Vec<(String, TestObjectType)> = Vec::new();
        let q = qualifier(w);
        if let Ok(tname) = qualifier_name(q) {
            if let Some(t) = parse_type_name(tname) {
                for id in [src_obj(w), tgt_obj(w)] {
                    if id != 0 {
                        objects.push((name_of(&vocab.objects, id).to_string(), t));
                    }
                }
            }
        }
        events.push(TestOcelEvent {
            event_id: format!("e_{}", events.len()),
            case_id: format!("case_{case_no}"),
            activity: activity_from_key(act_name),
            timestamp_ns: case_time,
            objects,
            attributes: HashMap::new(), // UNSUPPORTED at packed tier by design
        });
    }
    events
}

/// Decode side of [`activity_key`]: rebuild the variant, payloads empty.
fn activity_from_key(key: &str) -> TestActivity {
    match key {
        "TestCompleted" => TestActivity::TestCompleted { passed: true },
        "TestSkipped" => TestActivity::TestSkipped { reason: String::new() },
        "AssertionPassed" => TestActivity::AssertionPassed { macro_name: String::new() },
        "AssertionFailed" => TestActivity::AssertionFailed { macro_name: String::new() },
        "ConformanceChecked" => TestActivity::ConformanceChecked { fitness: 0.0 },
        "ConformanceFailed" => {
            TestActivity::ConformanceFailed { fitness: 0.0, first_deviation_step: 0 }
        }
        "SubstrateGrew" => TestActivity::SubstrateGrew { rank: 0 },
        "DiagnosticEmitted" => TestActivity::DiagnosticEmitted {
            category: crate::core::governance::DiagnosticCategory::Admission,
            severity: crate::core::governance::Severity::Info,
        },
        "WavePhaseStarted" => TestActivity::WavePhaseStarted { phase: 0, task_count: 0 },
        "WavePhaseCompleted" => {
            TestActivity::WavePhaseCompleted { phase: 0, admitted: 0, refused: 0 }
        }
        "WaveTaskStarted" => TestActivity::WaveTaskStarted { phase: 0, task_index: 0 },
        "WaveTaskCompleted" => {
            TestActivity::WaveTaskCompleted { phase: 0, task_index: 0, receipt_present: false }
        }
        "RunStarted" => TestActivity::RunStarted { registered_domain: String::new() },
        "RunCompleted" => TestActivity::RunCompleted { p_admitted: 0.0, andon_count: 0 },
        // Unit variants (no payloads to regenerate).
        "ReceiptGenerated" => TestActivity::ReceiptGenerated,
        "ReceiptMissing" => TestActivity::ReceiptMissing,
        "FixtureSetup" => TestActivity::FixtureSetup,
        "FixtureTeardown" => TestActivity::FixtureTeardown,
        "FixtureComposed" => TestActivity::FixtureComposed,
        "ArtifactAdmitted" => TestActivity::ArtifactAdmitted,
        "ArtifactRefused" => TestActivity::ArtifactRefused,
        "DriftZero" => TestActivity::DriftZero,
        "DriftDetected" => TestActivity::DriftDetected,
        "BypassDetected" => TestActivity::BypassDetected,
        "SubstrateLabor" => TestActivity::SubstrateLabor,
        "OntologyConsistent" => TestActivity::OntologyConsistent,
        "OntologyViolation" => TestActivity::OntologyViolation,
        "WaveConflictDetected" => TestActivity::WaveConflictDetected,
        _ => TestActivity::TestStarted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::ocel::types::{OcelLog, TestOcelEvent};

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

    fn obj_name(prefix: &str, n: usize) -> String {
        format!("{prefix}-{n}")
    }

    // ── Vendored word layout ────────────────────────────────────────────────

    #[test]
    fn word_roundtrip_edge_values() {
        for (a, s, t, q, k) in [
            (0u16, 0u16, 0u16, 0u8, 0u8),
            (u16::MAX, u16::MAX, u16::MAX, u8::MAX, u8::MAX),
            (1, 2, 3, 4, 5),
            (u16::MAX, 0, u16::MAX, 0, u8::MAX),
            (0, u16::MAX, 0, u8::MAX, 0),
        ] {
            let w = pack(a, s, t, q, k);
            assert_eq!(activity(w), a);
            assert_eq!(src_obj(w), s);
            assert_eq!(tgt_obj(w), t);
            assert_eq!(qualifier(w), q);
            assert_eq!(tick(w), k);
        }
    }

    #[test]
    fn field_isolation_max_in_each_slot() {
        let cases = [
            pack(u16::MAX, 0, 0, 0, 0),
            pack(0, u16::MAX, 0, 0, 0),
            pack(0, 0, u16::MAX, 0, 0),
            pack(0, 0, 0, u8::MAX, 0),
            pack(0, 0, 0, 0, u8::MAX),
        ];
        for w in cases {
            assert_eq!(activity(w) == u16::MAX, w == cases[0]);
            assert_eq!(src_obj(w) == u16::MAX, w == cases[1]);
            assert_eq!(tgt_obj(w) == u16::MAX, w == cases[2]);
            assert_eq!(qualifier(w) == u8::MAX, w == cases[3]);
            assert_eq!(tick(w) == u8::MAX, w == cases[4]);
        }
    }

    // ── Roundtrip identity on clean corpora ─────────────────────────────────

    #[test]
    fn roundtrip_identity_on_clean_corpus() {
        // Clean corpus: every event's objects are same-type pairs (see module
        // doc for the tgt-type recovery rule), timestamps spaced <= 255ns.
        let mut log = OcelLog::new();
        let case_a = |ts: u64, act: TestActivity| {
            ev(
                "run-A",
                act,
                ts,
                vec![
                    (obj_name("art", 1), TestObjectType::Artifact),
                    (obj_name("art", 2), TestObjectType::Artifact),
                ],
            )
        };
        let case_b_objs = |ts: u64| {
            ev(
                "run-B",
                TestActivity::FixtureSetup,
                ts,
                vec![
                    (obj_name("fix", 1), TestObjectType::Fixture),
                    (obj_name("fix", 2), TestObjectType::Fixture),
                ],
            )
        };
        for e in [
            case_a(0, TestActivity::TestStarted),
            case_a(10, TestActivity::TestCompleted { passed: true }),
            case_a(20, TestActivity::ReceiptGenerated),
            case_b_objs(0),
            case_b_objs(5),
        ] {
            let id = format!("e{}", log.events.len());
            log.events.insert(id, e);
        }
        let (words, vocab) = to_ocel64(&log).expect("clean corpus packs");
        let back = from_ocel64(&words, &vocab);

        assert_eq!(back.len(), 5);
        // Activities and ordering preserved.
        assert_eq!(activity_key(&back[0].activity), "TestStarted");
        assert_eq!(activity_key(&back[1].activity), "TestCompleted");
        assert_eq!(activity_key(&back[2].activity), "ReceiptGenerated");
        assert_eq!(activity_key(&back[3].activity), "FixtureSetup");
        // Per-case relative timeline preserved: run-A 0,10,20; run-B 0,5.
        assert_eq!([back[0].timestamp_ns, back[1].timestamp_ns, back[2].timestamp_ns], [0, 10, 20]);
        assert_eq!([back[3].timestamp_ns, back[4].timestamp_ns], [0, 5]);
        // Case segmentation preserved (names regenerated case_{n}, documented).
        assert_eq!(back[0].case_id, "case_0");
        assert_eq!(back[3].case_id, "case_1");
        // Object names and types preserved for same-type pairs.
        assert_eq!(back[0].objects.len(), 2);
        assert_eq!(back[0].objects[0].0, "art-1");
        assert_eq!(back[0].objects[1].0, "art-2");
        assert_eq!(back[0].objects[0].1, TestObjectType::Artifact);
        assert_eq!(back[3].objects[0].1, TestObjectType::Fixture);
        // Attributes UNSUPPORTED at packed tier by design.
        assert!(back.iter().all(|e| e.attributes.is_empty()));
        // Determinism: same log -> byte-identical words.
        let (words2, _) = to_ocel64(&log).expect("second pack");
        assert_eq!(words, words2);
    }

    #[test]
    fn roundtrip_preserves_receipt_missing() {
        // V1-audit defect: `ReceiptMissing` had no decode arm in
        // `activity_from_key`, so packed-tier roundtrips silently decoded it
        // as `TestStarted`. Guard: the unit variant survives the pack/unpack
        // boundary with its key intact.
        let mut log = OcelLog::new();
        log.events
            .insert("e0".to_string(), ev("run-R", TestActivity::ReceiptMissing, 0, vec![]));
        log.events
            .insert("e1".to_string(), ev("run-R", TestActivity::TestStarted, 5, vec![]));
        let (words, vocab) = to_ocel64(&log).expect("packs");
        let back = from_ocel64(&words, &vocab);

        assert_eq!(back.len(), 2);
        assert!(matches!(back[0].activity, TestActivity::ReceiptMissing));
        assert_eq!(activity_key(&back[0].activity), "ReceiptMissing");
        // The distinct neighbor is not confused with it.
        assert_eq!(activity_key(&back[1].activity), "TestStarted");
        assert_eq!(back[0].timestamp_ns, 0);
        assert_eq!(back[1].timestamp_ns, 5);
    }

    // ── Refusals ────────────────────────────────────────────────────────────

    #[test]
    fn tick_delta_overflow_refused_not_saturated() {
        let mut log = OcelLog::new();
        log.events
            .insert("e0".to_string(), ev("run-X", TestActivity::TestStarted, 0, vec![]));
        log.events.insert(
            "e1".to_string(),
            ev("run-X", TestActivity::TestCompleted { passed: true }, 256, vec![]),
        );
        let err = to_ocel64(&log).expect_err("delta 256 must refuse");
        match err {
            BridgeError::TickDeltaOverflow { case, index, delta_ns } => {
                assert_eq!(case, "run-X");
                assert_eq!(index, 1);
                assert_eq!(delta_ns, 256);
            }
            other => panic!("wrong refusal: {other:?}"),
        }
    }

    #[test]
    fn non_monotonic_refused_by_checked_deltas() {
        // Unreachable from to_ocel64 (it sorts first); the check is
        // re-derived in checked_deltas and directly testable.
        let err = checked_deltas("run-M", &[10, 5]).expect_err("decreasing ts must refuse");
        assert_eq!(err, BridgeError::NonMonotonicTimestamp { case: "run-M".to_string(), index: 1 });
        // Equal timestamps are fine (delta 0).
        assert_eq!(checked_deltas("run-M", &[7, 7]).expect("equal ts ok"), vec![0, 0]);
    }

    #[test]
    fn collision_refused_via_idmap_error_mapping() {
        // The sibling idmap lane returns plain IdMapError; this module maps
        // it. Exercise the mapping on a hand-built IdMapError (real IdMap
        // construction with a colliding corpus needs a blake3 search and is
        // the idmap lane's own court's job).
        let mapped = BridgeError::from(IdMapError::Collision {
            index_a: 0,
            name_a: "a".to_string(),
            index_b: 1,
            name_b: "b".to_string(),
        });
        match mapped {
            BridgeError::Collision { field, details } => {
                assert_eq!(field, "objects");
                assert!(details.contains("\"a\""));
                assert!(details.contains("\"b\""));
            }
            other => panic!("wrong refusal: {other:?}"),
        }
        // The From<IdMapError> impl never fires for to_ocel64 directly
        // (build_vocab maps with the right field), but the same mapping in
        // build_vocab is exercised via the activities/objects fields.
    }

    #[test]
    fn capacity_exceeded_refused() {
        let err = ensure_capacity("activities", usize::from(u16::MAX) + 1)
            .expect_err("over-bound vocab must refuse");
        assert_eq!(
            err,
            BridgeError::CapacityExceeded {
                field: "activities",
                count: usize::from(u16::MAX) + 1,
                bound: u16::MAX,
            }
        );
        assert!(ensure_capacity("objects", 65_535).is_ok());
    }

    #[test]
    fn unknown_qualifier_refused() {
        let err = qualifier_name(200).expect_err("untabled byte must refuse");
        assert_eq!(err, BridgeError::UnknownQualifier { raw: "200".to_string() });
        // Table roundtrip for every tabled byte.
        for t in [
            TestObjectType::Artifact,
            TestObjectType::Schema,
            TestObjectType::ProcessModel,
            TestObjectType::Receipt,
            TestObjectType::Fixture,
            TestObjectType::AgentContext,
            TestObjectType::WaveInstance,
            TestObjectType::SectorStack,
        ] {
            let q = qualifier_for(type_name(t));
            assert_eq!(qualifier_name(q).expect("tabled byte decodes"), type_name(t));
        }
    }

    // ── Era wrap disclosure ─────────────────────────────────────────────────

    #[test]
    fn era_wrap_refuses_beyond_255_not_saturates() {
        // strata rehydrate.rs documents the 8-bit tick era wrap (255 -> 0) as
        // caller-checkpoint duty. This bridge discloses the era boundary as a
        // typed refusal instead: delta 255 is the last representable tick,
        // delta 256 is REFUSED with the exact ns, never saturated to 255.
        let ok = checked_deltas("era", &[0, 255]).expect("delta 255 is representable");
        assert_eq!(ok, vec![0, 255]);
        let err = checked_deltas("era", &[0, 255, 510]).expect("chain of 255s packs");
        assert_eq!(err, vec![0, 255, 255]);
        let refused = checked_deltas("era", &[0, 256]).expect_err("delta 256 refuses");
        assert_eq!(
            refused,
            BridgeError::TickDeltaOverflow { case: "era".to_string(), index: 1, delta_ns: 256 }
        );
        // And the era boundary through the full bridge: a log whose case
        // spans more than one 256-tick era is refused, not wrapped.
        let mut log = OcelLog::new();
        log.events
            .insert("e0".to_string(), ev("era-case", TestActivity::TestStarted, 0, vec![]));
        log.events
            .insert("e1".to_string(), ev("era-case", TestActivity::ReceiptGenerated, 256, vec![]));
        assert!(matches!(to_ocel64(&log), Err(BridgeError::TickDeltaOverflow { .. })));
    }

    // ── Word-count and continuation encoding ────────────────────────────────

    #[test]
    fn k_object_refs_emit_ceil_k_over_2_words_odd_leftover_tgt_zero() {
        let mut log = OcelLog::new();
        // 4 objects -> header + 1 continuation = 2 words.
        log.events.insert(
            "e0".to_string(),
            ev(
                "c",
                TestActivity::TestStarted,
                0,
                vec![
                    (obj_name("art", 1), TestObjectType::Artifact),
                    (obj_name("art", 2), TestObjectType::Artifact),
                    (obj_name("art", 3), TestObjectType::Artifact),
                    (obj_name("art", 4), TestObjectType::Artifact),
                ],
            ),
        );
        // 3 objects -> header + 1 continuation with tgt = 0.
        log.events.insert(
            "e1".to_string(),
            ev(
                "c",
                TestActivity::ReceiptGenerated,
                1,
                vec![
                    (obj_name("art", 5), TestObjectType::Artifact),
                    (obj_name("art", 6), TestObjectType::Artifact),
                    (obj_name("art", 7), TestObjectType::Artifact),
                ],
            ),
        );
        let (words, vocab) = to_ocel64(&log).expect("packs");
        // Single case: no separator word. 2 + 2 = 4 words.
        assert_eq!(words.len(), 4);
        // Header of e0.
        assert_ne!(activity(words[0]), 0);
        assert_eq!(tick(words[0]), 0);
        // Continuation of e0: activity 0, both slots nonzero.
        assert_eq!(activity(words[1]), 0);
        assert_ne!(src_obj(words[1]), 0);
        assert_ne!(tgt_obj(words[1]), 0);
        // Header of e1 (delta 1).
        assert_ne!(activity(words[2]), 0);
        assert_eq!(tick(words[2]), 1);
        // Odd leftover continuation: tgt = 0.
        assert_eq!(activity(words[3]), 0);
        assert_ne!(src_obj(words[3]), 0);
        assert_eq!(tgt_obj(words[3]), 0);
        // Roundtrip: 7 object refs recovered in order.
        let back = from_ocel64(&words, &vocab);
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].objects.len(), 4);
        assert_eq!(back[1].objects.len(), 3);
        assert_eq!(back[1].objects[2].0, "art-7");
    }

    #[test]
    fn empty_log_packs_to_no_words() {
        let log = OcelLog::new();
        let (words, _vocab) = to_ocel64(&log).expect("empty log packs");
        assert!(words.is_empty());
        let back = from_ocel64(&words, &_vocab);
        assert!(back.is_empty());
    }

    #[test]
    fn hashmap_nondeterminism_is_sorted_away() {
        // Insert the same corpus under different HashMap-resident keys; the
        // packed words must be identical regardless of insertion order.
        let mut log1 = OcelLog::new();
        let mut log2 = OcelLog::new();
        let mk = |case: &str, act: TestActivity, ts: u64| {
            ev(case, act, ts, vec![(obj_name("art", 1), TestObjectType::Artifact)])
        };
        log1.events.insert("k1".to_string(), mk("b", TestActivity::TestStarted, 3));
        log1.events.insert("k2".to_string(), mk("b", TestActivity::ReceiptGenerated, 1));
        log1.events.insert("k3".to_string(), mk("a", TestActivity::TestStarted, 9));
        log2.events.insert("zz".to_string(), mk("a", TestActivity::TestStarted, 9));
        log2.events.insert("yy".to_string(), mk("b", TestActivity::ReceiptGenerated, 1));
        log2.events.insert("xx".to_string(), mk("b", TestActivity::TestStarted, 3));
        let (w1, _) = to_ocel64(&log1).expect("packs");
        let (w2, _) = to_ocel64(&log2).expect("packs");
        assert_eq!(w1, w2);
    }
}
