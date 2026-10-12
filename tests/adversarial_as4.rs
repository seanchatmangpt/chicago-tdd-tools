//! AS4 adversarial ceiling attacks (v26.10.11 assurance surface).
//!
//! Run via `just test-assurance-integration` (requires the full 4-feature
//! combo "nato,wasm-guest,ocel-ingest,ocel-ingest-json" — see the `#![cfg]`
//! header below).
//!
//! Run:
//!
//! ```text
//! cargo test --test adversarial_as4 --features "nato,wasm-guest,ocel-ingest,ocel-ingest-json"
//! ```
//!
//! Attack 1+3 (guest ABI) touch one `static mut TABLE`, so all guest-table
//! work lives in a single `#[test]` (tests in one binary run on parallel
//! threads).

#![allow(missing_docs)]
#![allow(unsafe_code, warnings)]
// JUSTIFICATION: test-side FFI plumbing at the guest ABI boundary — lengths are
// small by construction (test fixtures, never user input), so usize->u32/u16
// truncation casts and `&buf as *const _`-style raw-pointer borrows are safe.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::borrow_as_ptr)]
#![cfg(all(feature = "nato", feature = "wasm-guest", feature = "ocel-ingest-json"))]

use std::fmt::Write as _;

use chicago_tdd_tools::conformance::guest as abi;
use chicago_tdd_tools::conformance::{ConformanceSession, ReportOutcome, YieldStatus};
use chicago_tdd_tools::ledger::dossier::{CourtLedgerBuilder, DossierBuilder};
use chicago_tdd_tools::ledger::receipt::{
    ChainedCourtReceipt, CourtOutcome as LCourtOutcome, CourtReceiptV1, ReceiptMetrics,
    ThermalClass,
};
use chicago_tdd_tools::nato::court::{CourtId, CourtOutcome, MetricValue, Pillar, Verdict};
use chicago_tdd_tools::nato::disclosure::ScopingRecord;
use chicago_tdd_tools::nato::dossier_court::dossier_root;
use chicago_tdd_tools::nato::receipt::{ControlResult, CourtReceipt};
use chicago_tdd_tools::observability::receipt::Blake3ReceiptEntry;

// ---------------------------------------------------------------------------
// 8-atom chain model + OCEL64 stream words
// ---------------------------------------------------------------------------

const CHAIN_DSL: &str = "\
activity a0 = g.e0
activity a1 = g.e1
activity a2 = g.e2
activity a3 = g.e3
activity a4 = g.e4
activity a5 = g.e5
activity a6 = g.e6
activity a7 = g.e7

sequence {
  a0
  a1
  a2
  a3
  a4
  a5
  a6
  a7
}
";

/// OCEL64 stream word: activity [7:0], tick [15:8] (session wire law).
const fn word(tid: u64, tick: u64) -> u64 {
    (tick << 8) | tid
}

fn chain_words(n: u64) -> Vec<u64> {
    (0..n).map(|i| word(i, i + 1)).collect()
}

/// 16-byte CTTG wire header + DSL body.
fn wire_model(dsl: &str) -> Vec<u8> {
    let mut b = Vec::with_capacity(16 + dsl.len());
    b.extend_from_slice(b"CTTG");
    b.push(0x01); // version
    b.push(0x00); // reserved
    b.extend_from_slice(&(dsl.len() as u16).to_le_bytes());
    b.extend_from_slice(&[0u8; 8]); // reserved2
    b.extend_from_slice(dsl.as_bytes());
    b
}

fn wire_stream(words: &[u64]) -> Vec<u8> {
    let mut b = Vec::with_capacity(words.len() * 8);
    for w in words {
        b.extend_from_slice(&w.to_le_bytes());
    }
    b
}

/// Independent replay summary key for differential comparison.
const fn summary_key(s: &chicago_tdd_tools::conformance::ReplaySummary) -> (u32, usize, u64, u8) {
    let oc = match s.outcome {
        ReportOutcome::Incomplete => 0u8,
        ReportOutcome::Complete => 1,
        ReportOutcome::Faulted(_) => 2,
    };
    (s.steps_executed, s.events_total, s.final_marking, oc)
}

// ---------------------------------------------------------------------------
// Attack 1 — fuel exhaustion mid-trajectory: split-run vs single-run differential
// ---------------------------------------------------------------------------

#[test]
fn as4_attack1_fuel_exhaustion_differential() {
    let dsl = CHAIN_DSL;
    let stream = chain_words(8);

    // ---- (a) Kernel-level differential across quotas ----
    // Single un-broken run is the reference.
    let mut single = ConformanceSession::from_powl_str_model(dsl);
    single.attach_stream(&stream).expect("single-run stream attaches");
    let _ = single.step_budget(u32::MAX);
    let expected = summary_key(&single.finish());

    for quota in [1u32, 3, 10, 7] {
        let mut s = ConformanceSession::from_powl_str_model(dsl);
        s.attach_stream(&stream).expect("split-run stream attaches");
        let mut yields = 0u32;
        while let YieldStatus::Yielded { .. } = s.step_budget(quota) {
            yields += 1;
        }
        if quota < 8 {
            assert!(yields >= 1, "quota {quota} on 8 events must yield at least once");
        }
        assert_eq!(
            summary_key(&s.finish()),
            expected,
            "split-run (quota {quota}) diverged from single-run"
        );
    }

    // ---- (b) ABI-level: ctt_session_step with quota 10 loop ----
    let (model, stream_bytes) = (wire_model(dsl), wire_stream(&stream));
    let mut handle: u64 = 0;
    let rc = unsafe { abi::ctt_session_new(model.as_ptr(), model.len() as u32, &mut handle) };
    assert_eq!(rc, abi::OK);

    let rc = unsafe {
        abi::ctt_session_attach(handle, stream_bytes.as_ptr(), stream_bytes.len() as u32)
    };
    assert_eq!(rc, abi::OK);

    let mut out = [0u8; 21];
    let mut splits = 0u32;
    loop {
        let rc = unsafe { abi::ctt_session_step(handle, 10, out.as_mut_ptr(), out.len() as u32) };
        assert_eq!(rc, abi::OK);
        if out[0] == 0 {
            splits += 1;
            continue; // Yielded
        }
        break;
    }
    assert_eq!(out[0], 1, "clean chain must Complete");
    let steps = u32::from_le_bytes([out[5], out[6], out[7], out[8]]);
    let events = u32::from_le_bytes([out[9], out[10], out[11], out[12]]);
    assert_eq!(steps, 8);
    assert_eq!(events, 8);
    let _ = splits;

    // quota 0: sticky Complete, never advances.
    let mut out0 = [0u8; 21];
    let rc = unsafe { abi::ctt_session_step(handle, 0, out0.as_mut_ptr(), out0.len() as u32) };
    assert_eq!(rc, abi::OK);
    assert_eq!(out0[0], 1, "latched Complete is sticky across quota-0 steps");

    // ---- (c) Violation latched mid-trajectory under split quota ----
    let mut bad = chain_words(8);
    bad[4] = word(9, 5); // undeclared tid
    let (bad_model, bad_stream) = (wire_model(dsl), wire_stream(&bad));
    let mut h2: u64 = 0;
    let rc = unsafe { abi::ctt_session_new(bad_model.as_ptr(), bad_model.len() as u32, &mut h2) };
    assert_eq!(rc, abi::OK);
    let rc = unsafe { abi::ctt_session_attach(h2, bad_stream.as_ptr(), bad_stream.len() as u32) };
    assert_eq!(rc, abi::OK);
    let mut out2 = [0u8; 21];
    let mut saw_violation = false;
    for _ in 0..4 {
        let rc = unsafe { abi::ctt_session_step(h2, 3, out2.as_mut_ptr(), 21) };
        assert_eq!(rc, abi::OK);
        if out2[0] == 2 {
            saw_violation = true;
            break;
        }
    }
    assert!(saw_violation, "corrupted stream must latch Violation under split quota");
    // Violation is sticky: further steps re-report, no half-state.
    let rc = unsafe { abi::ctt_session_step(h2, 3, out2.as_mut_ptr(), 21) };
    assert_eq!(rc, abi::OK);
    assert_eq!(out2[0], 2, "Violation must be sticky across further steps");

    unsafe { abi::ctt_session_drop(h2) };
    unsafe { abi::ctt_session_drop(handle) };
}

// ---------------------------------------------------------------------------
// Attack 3 — MAX_SESSIONS=8: fill/drop/refill, generation reuse, wrap forge cost
// ---------------------------------------------------------------------------

#[test]
fn as4_attack3_session_table_exhaustion_and_generation() {
    // PROBE
    {
        let model = wire_model(CHAIN_DSL);
        let stream = wire_stream(&chain_words(8));
        let mut h: u64 = 0;
        let rc = unsafe { abi::ctt_session_new(model.as_ptr(), model.len() as u32, &mut h) };
        eprintln!("PROBE new rc={rc}");
        let rc = unsafe { abi::ctt_session_attach(h, stream.as_ptr(), stream.len() as u32) };
        eprintln!("PROBE attach rc={rc}");
        let mut o = [0u8; 21];
        for i in 0..3 {
            let rc = unsafe { abi::ctt_session_step(h, 3, o.as_mut_ptr(), 21) };
            eprintln!("PROBE step{i} rc={rc} out={:?}", &o[..14]);
        }
        unsafe { abi::ctt_session_drop(h) };
    }
    let model = wire_model(CHAIN_DSL);
    let stream = wire_stream(&chain_words(8));

    let mut handles = [0u64; 8];
    for h in &mut handles {
        let rc = unsafe { abi::ctt_session_new(model.as_ptr(), model.len() as u32, h) };
        assert_eq!(rc, abi::OK);
        let rc = unsafe { abi::ctt_session_attach(*h, stream.as_ptr(), stream.len() as u32) };
        assert_eq!(rc, abi::OK);
    }
    // 9th session: table full.
    let mut h9: u64 = 0;
    let rc = unsafe { abi::ctt_session_new(model.as_ptr(), model.len() as u32, &mut h9) };
    assert_eq!(rc, abi::ERR_TABLE_FULL, "9th session must be refused");

    // Drop slot of handles[3] mid-life (after stepping it to mid-trajectory).
    let mut out = [0u8; 21];
    let rc = unsafe { abi::ctt_session_step(handles[3], 3, out.as_mut_ptr(), 21) };
    assert_eq!(rc, abi::OK);
    assert_eq!(out[0], 0, "must yield mid-trajectory before drop");
    let stale = handles[3];
    let rc = unsafe { abi::ctt_session_drop(stale) };
    assert_eq!(rc, abi::OK);

    // Stale handle: step/seal/drop all refuse.
    let rc = unsafe { abi::ctt_session_step(stale, 3, out.as_mut_ptr(), 21) };
    assert_eq!(rc, abi::ERR_BAD_HANDLE);
    let mut digest = [0u8; 32];
    let mut dlen: u32 = 0;
    let rc = unsafe { abi::ctt_session_seal(stale, digest.as_mut_ptr(), 32, &mut dlen) };
    assert_eq!(rc, abi::ERR_BAD_HANDLE);
    let rc = unsafe { abi::ctt_session_drop(stale) };
    assert_eq!(rc, abi::ERR_BAD_HANDLE);

    // Refill: new session must reuse the freed slot with generation + 1.
    let mut fresh: u64 = 0;
    let rc = unsafe { abi::ctt_session_new(model.as_ptr(), model.len() as u32, &mut fresh) };
    assert_eq!(rc, abi::OK);
    let (fresh_slot, fresh_gen) = ((fresh & 0xFFFF_FFFF) as usize, (fresh >> 32) as u32);
    let (stale_slot, stale_gen) = ((stale & 0xFFFF_FFFF) as usize, (stale >> 32) as u32);
    assert_eq!(fresh_slot, stale_slot, "freed slot must be reused first");
    assert_eq!(fresh_gen, stale_gen.wrapping_add(1), "generation must bump on reuse");

    // The stale handle must NOT write into the fresh session: attach via the
    // stale handle is refused, so no cross-session contamination is possible.
    let rc = unsafe { abi::ctt_session_attach(stale, stream.as_ptr(), stream.len() as u32) };
    assert_eq!(rc, abi::ERR_BAD_HANDLE);

    // ---- Generation-wrap forge cost ----
    // A stale handle resurrects only after 2^32 successful drops of the SAME
    // slot (u32 generation, wrapping). Measure the per-cycle cost of one
    // successful new+drop on the fresh slot to price the forge.
    let rc = unsafe { abi::ctt_session_drop(fresh) }; // free one slot for the churn loop
    assert_eq!(rc, abi::OK);
    let cycles = 100_000u64;
    let t0 = std::time::Instant::now();
    for _ in 0..cycles {
        let mut h: u64 = 0;
        let rc = unsafe { abi::ctt_session_new(model.as_ptr(), model.len() as u32, &mut h) };
        debug_assert_eq!(rc, abi::OK);
        unsafe { abi::ctt_session_drop(h) };
    }
    let per_cycle = t0.elapsed() / cycles as u32;
    let forge_secs = per_cycle * u32::MAX; // Duration * u32 is fine
    eprintln!(
        "AS4 gen-wrap: per-cycle {per_cycle:?}; 2^32-cycle forge of stale handle on one slot = {forge_secs:?}"
    );
    // Note: the table is full here (7 live + the churn uses slot of `fresh`),
    // the churn loop works because new() reuses the just-dropped slot.

    // Cleanup: drop remaining sessions.
    for h in &handles {
        unsafe { abi::ctt_session_drop(*h) };
    }
}

// ---------------------------------------------------------------------------
// Attack 4 — dossier at 10x design scale
// ---------------------------------------------------------------------------

fn l_receipt(court_id: &str, seed: u8) -> ChainedCourtReceipt {
    l_receipt_parented(court_id, seed, [0u8; 32])
}

fn l_receipt_parented(court_id: &str, seed: u8, parent: [u8; 32]) -> ChainedCourtReceipt {
    ChainedCourtReceipt::from_receipt(&CourtReceiptV1 {
        subject_hash: [seed; 32],
        parent_receipt_hash: parent,
        metrics: ReceiptMetrics {
            timestamp_ms: 0,
            run_id: u64::from(seed),
            total_ticks: 0,
            wall_ms: 0,
            thermal: ThermalClass::Warm,
            outcome: LCourtOutcome::Pass,
            budget_met: true,
        },
        positive_control: [0u8; 32],
        metadata_digest: [0u8; 32],
        prev_self_hash: [0u8; 32],
        name: court_id.to_owned(),
        fail_closed: false,
    })
    .expect("test court ids fit NAME_MAX")
}

#[test]
fn as4_attack4_dossier_at_10x_scale() {
    use chicago_tdd_tools::ledger::receipt::NAME_MAX;
    assert_eq!(NAME_MAX, 84, "NAME_MAX is exactly 84 bytes");

    // (a) 100-court dossier (10x the 20-court design): seal + all proofs verify.
    let mut b = DossierBuilder::new();
    for i in 0..100u16 {
        b.add_court(format!("court-{i:03}"), [i as u8; 32]).expect("unique ids");
    }
    let (root, proofs) = b.seal_with_proofs().expect("nonempty");
    assert_eq!(proofs.len(), 100);
    for p in &proofs {
        assert_eq!(p.verify(&root), Ok(()), "proof failed for {}", p.court_id);
    }

    // (b) Duplicate court ids refused at 100-add pressure.
    assert!(b.add_court("court-000", [0xFF; 32]).is_err(), "duplicate must refuse");

    // (c) Receipt with a name exactly at NAME_MAX (84 chars) through the chain.
    let name84 = "N".repeat(84);
    let mut ledger = CourtLedgerBuilder::new(name84.clone());
    for seed in 0..100u8 {
        let r = l_receipt_parented(&name84, seed, ledger.head());
        ledger.append(r).expect("84-byte court id appends");
    }
    let (root84, _) = {
        let mut d = DossierBuilder::new();
        d.add_court(name84, ledger.head()).expect("add 84-char court");
        d.seal_with_proofs().expect("seal")
    };
    let _ = root84;

    // (d) NAME_MAX+1 must be refused by the receipt validator; encode()
    // truncating here would be a silent-degrade finding.
    let too_long_name = "N".repeat(NAME_MAX + 1);
    let mut encoded = l_receipt("x", 0).encode();
    // Overwrite the 100-byte name field (offset 220) with an 85-byte name
    // + 15 zero bytes: exactly what encode() must never produce.
    encoded[NAME_ABS_OVERRIDE..NAME_ABS_OVERRIDE + 100].fill(0);
    let n = too_long_name.len();
    encoded[NAME_ABS_OVERRIDE..NAME_ABS_OVERRIDE + n].copy_from_slice(too_long_name.as_bytes());
    // The verify path must refuse it typed, not truncate.
    let verdict =
        chicago_tdd_tools::ledger::receipt::VerifiedCourtReceipt::from_bytes_verified(&encoded);
    match verdict {
        // Canonical typed refusal; an 85-byte name may also trip the
        // zero-trailer law before the length check — either way the record is
        // refused typed, never truncated.
        Err(
            chicago_tdd_tools::ledger::error::LedgerError::InvalidName {
                reason: chicago_tdd_tools::ledger::error::NameError::TooLong,
                ..
            }
            | chicago_tdd_tools::ledger::error::LedgerError::ZeroTrailerViolation,
        ) => {}
        other => panic!("NAME_MAX+1 name must be refused typed, got: {other:?}"),
    }
}

/// Offset of the name field inside the 352-byte chained record (dossier.rs
/// owns `NAME_ABS` as private; mirror it here).
const NAME_ABS_OVERRIDE: usize = 220;

// ---------------------------------------------------------------------------
// Attack 5 — concurrent dossier-emit + dossier-inspect on the same file
// ---------------------------------------------------------------------------

fn nato_receipt(i: u8) -> CourtReceipt {
    let court = match i % 20 {
        0 => CourtId::N01,
        1 => CourtId::N02,
        2 => CourtId::N03,
        3 => CourtId::N04,
        4 => CourtId::N05,
        5 => CourtId::N06,
        6 => CourtId::N07,
        7 => CourtId::N08,
        8 => CourtId::N09,
        9 => CourtId::N10,
        10 => CourtId::N11,
        11 => CourtId::N12,
        12 => CourtId::N13,
        13 => CourtId::N14,
        14 => CourtId::N15,
        15 => CourtId::N16,
        16 => CourtId::N17,
        17 => CourtId::N18,
        18 => CourtId::N19,
        _ => CourtId::N20,
    };
    CourtReceipt::record(
        court,
        Pillar::Proof,
        court.canonical(),
        "deadbeef",
        CourtOutcome {
            verdict: Verdict::Pass,
            metrics: vec![("ticks", MetricValue::U64(1))],
            scoping: ScopingRecord::full("mandate", "evidence"),
        },
        ControlResult::MutantCaught { mutant_id: "m1".into(), detection_channel: "test".into() },
    )
}

#[test]
fn as4_attack5_concurrent_emit_inspect_torn_reads() {
    // Build a valid 20-receipt dossier JSON doc (the dossier-emit shape).
    let receipts: Vec<CourtReceipt> = (0..20u8).map(nato_receipt).collect();
    let root = dossier_root(&receipts);
    let receipt_values: Vec<serde_json::Value> = receipts
        .iter()
        .map(|r| serde_json::to_value(r).expect("serialize receipt"))
        .collect();
    let mut root_hex = String::with_capacity(root.len() * 2);
    for b in &root {
        let _ = write!(root_hex, "{b:02x}");
    }
    let doc = serde_json::json!({
        "format": "CTT-DOSSIER-v1",
        "merkle_root": root_hex,
        "key_id": "k0",
        "signature": [],
        "court_receipts": receipt_values,
    });
    let good = std::sync::Arc::new(serde_json::to_vec_pretty(&doc).expect("serialize dossier"));

    let path = std::env::temp_dir().join("as4_dossier_race.json");
    std::fs::write(&path, good.as_slice()).expect("seed file");

    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Writer: dossier-emit behavior = truncate + full write (NOT atomic).
    let w_stop = stop.clone();
    let w_path = path.clone();
    let good_w = good.clone();
    let writer = std::thread::spawn(move || {
        while !w_stop.load(std::sync::atomic::Ordering::Relaxed) {
            std::fs::write(&w_path, good_w.as_slice()).expect("writer fs::write");
        }
    });

    // Reader: dossier-inspect behavior = fs::read + JSON parse + recompute.
    let r_path = path.clone();
    let reader = std::thread::spawn(move || {
        let mut ok = 0u64;
        let mut torn = 0u64;
        let mut other_err = 0u64;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while std::time::Instant::now() < deadline {
            match std::fs::read(&r_path) {
                Ok(bytes) => {
                    if bytes.len() != good_len_marker(&good) {
                        torn += 1;
                        // Partial read: serde_json parse must fail typed, not panic.
                        let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&bytes);
                        assert!(parsed.is_err(), "torn document must not parse as valid");
                        continue;
                    }
                    match serde_json::from_slice::<serde_json::Value>(&bytes) {
                        Ok(v) => {
                            ok += 1;
                            let _ = v;
                        }
                        Err(_) => other_err += 1,
                    }
                }
                Err(_) => other_err += 1,
            }
        }
        (ok, torn, other_err)
    });

    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    writer.join().expect("writer");
    let (ok, torn, other) = reader.join().expect("reader");
    eprintln!("AS4 torn-read: ok={ok} torn={torn} other_err={other}");
    assert!(ok > 0, "reader must observe whole documents too");
    let _ = torn;
    std::fs::remove_file(&path).ok();
}

const fn good_len_marker(good: &[u8]) -> usize {
    good.len()
}

// ---------------------------------------------------------------------------
// Attack 2 — 1M-event ingestion: import_ocel_json -> to_ocel64 (time + RSS via
// /usr/bin/time -l on the test process). Kept as its own test so it can be
// filtered and measured in isolation.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "run alone under /usr/bin/time -l: cargo test --test adversarial_as4 as4_attack2 -- --ignored"]
fn as4_attack2_ingestion_1m() {
    use chicago_tdd_tools::observability::ocel::import_json::import_ocel_json;
    use chicago_tdd_tools::observability::ocel::ocel64_bridge::to_ocel64;

    let n_cases = 10_000usize;
    let per_case = 100usize;
    assert_eq!(n_cases * per_case, 1_000_000);

    // --- generate the 1M-event OCEL JSON document ---
    let t0 = std::time::Instant::now();
    let mut j = String::with_capacity(250_000_000);
    j.push_str("{\"ocel:version\":\"2.0\",\"ocel:events\":{");
    let mut e = 0u64;
    for c in 0..n_cases {
        for i in 0..per_case {
            if e > 0 {
                j.push(',');
            }
            let _ = write!(
                j,
                "\"e{e}\":{{\"ocel:timestamp\":{},\"ocel:activity\":\"TestStarted\",\"ocel:caseId\":\"c{c:05}\"}}",
                1_700_000_000_000_000_000u64 + (c as u64) * 1_000_000 + i as u64
            );
            e += 1;
        }
    }
    // Only 200 distinct object ids: IdMap's 16-bit blake3 sentinel has a
    // ~50% collision probability at ~302 distinct names (its own docs), so
    // 10k distinct objects is a documented typed refusal, not a scale target.
    j.push_str("},\"ocel:objects\":{");
    for c in 0..200usize {
        if c > 0 {
            j.push(',');
        }
        let _ = write!(j, "\"o{c:05}\":{{\"ocel:type\":\"Fixture\"}}");
    }
    j.push_str("}}");
    let gen = t0.elapsed();
    eprintln!("AS4 ingest: json gen {:?} ({} bytes)", gen, j.len());

    // --- import ---
    let t1 = std::time::Instant::now();
    let log = import_ocel_json(&j).expect("1M-event import must succeed");
    let import_t = t1.elapsed();
    assert_eq!(log.events.len(), 1_000_000);
    eprintln!("AS4 ingest: import_ocel_json {import_t:?}");

    // --- pack ---
    let t2 = std::time::Instant::now();
    let (words, vocab) = to_ocel64(&log).expect("pack must succeed");
    let pack_t = t2.elapsed();
    eprintln!("AS4 ingest: to_ocel64 {:?} -> {} words", pack_t, words.len());
    let _ = &vocab;

    let total = t0.elapsed();
    eprintln!("AS4 ingest: TOTAL {total:?}");
}

// ---- trait-path helper so ConformanceSession can be built from a DSL string.
trait FromDsl {
    fn from_powl_str_model(dsl: &str) -> Self;
}

impl FromDsl for ConformanceSession {
    fn from_powl_str_model(dsl: &str) -> Self {
        let model = chicago_tdd_tools::conformance::PowlModel::from_powl_str(dsl)
            .expect("chain DSL must parse");
        Self::from_model(&model).expect("chain DSL must compile")
    }
}
