//! W6 lane — host-compiled fault-injection courts against the guest ABI.
//!
//! # Honest two-tier scoping (read this first)
//!
//! These courts run the guest ABI **host-compiled** (the crate's `rlib`
//! target, native codegen): the ABI functions are called as ordinary Rust
//! `extern "C"` functions over host memory. What is proven here is the
//! fail-closed law of the ABI shim itself: every corrupted input is either
//! refused with a typed status from the documented table, or admitted with
//! an outcome that matches the independent kernel oracle — never a wrong-Ok.
//! What is **not** proven here is the in-Wasm trap behavior: `panic = abort`
//! lowering to an unreachable trap, and temprun's observation of that trap,
//! are properties of the `wasm32` cdylib under a real runtime. Those land
//! with the temprun embed and will extend, not change, these courts.
//!
//! # Three corruption tiers
//!
//! 1. **SEU tier** — single-bit flips (LCG, donor Knuth constants, fixed
//!    seed) into the serialized net buffer, the stream buffer, and the
//!    guest-visible handle bytes, before each ABI call. Every flip must land
//!    in exactly one class: `REFUSED` (typed nonzero status) or
//!    `ADMITTED-LAWFUL` (status + step record match the independent kernel
//!    oracle's expectation for the corrupted input). Anything else is SDC
//!    and fails the court.
//! 2. **Torn-input tier** — the net/stream buffers are split mid-write:
//!    truncated, second half zeroed, halves swapped. The ABI must fail
//!    closed with a typed refusal (TraceMalformed-class in the status
//!    space), never partially admit, and never poison the handle table (a
//!    fresh lawful session still succeeds after every torn refusal).
//! 3. **Seal-boundary tier** — single-bit (exhaustive, 2,816 flips),
//!    byte-wide (every byte XOR 0xFF), and LCG random byte corruption across
//!    the 352-byte guest receipt wire. `from_bytes_verified` must reject
//!    every corruption: guest output corruption cannot become an admitted
//!    receipt.
//!
//! # Oracle discipline
//!
//! The oracle never reads guest state and never re-uses the ABI's parse
//! path: it re-derives the lawful outcome from the parent kernel directly
//! (`PowlModel::from_powl_str` + `ConformanceSession` replay) and from the
//! documented wire law (magic/version/reserved/length checks re-derived
//! here, not delegated). The kernel is the ABI's specification (the ABI is
//! documented as a wire shim over the landed kernel), so the court's subject
//! is shim fidelity, not kernel correctness. This mirrors the N09/N10 SEU
//! courts' independent `lawful_image` and the torn court's independently
//! derived checksum.
//!
//! # Tiering (prefix-stream)
//!
//! The SEU sweep runs 10,000 flips fast; the `#[ignore = "nato-tier3"]`
//! full tier runs 1,000,000 flips with the SAME LCG seed, so the full
//! stream's prefix is the fast stream — full subsumes fast. (The `nato`
//! feature is not enabled for this crate, so the donor LCG is reproduced
//! locally from the same constants as the donor courts.)
//!
//! # Positive controls (corrupted-accepting shims)
//!
//! A deliberately corrupted-accepting step shim (`leaky_step_shim`, which
//! reports Complete for any corrupted stream) MUST be caught by the same
//! sweep machinery, and the seal court's contrast control (`leaky_verify`)
//! must accept what the real verifier rejects — proving the courts
//! non-vacuous.

#![allow(clippy::unwrap_used)] // JUSTIFICATION: test code only

use chicago_tdd_tools::conformance::guest_yield::DEFAULT_REDUCTION_QUOTA;
use chicago_tdd_tools::conformance::{ConformanceSession, PowlModel, YieldStatus};

use ctt_wasm_guest::{
    ERR_BAD_HANDLE, ERR_BAD_MAGIC, ERR_BAD_MODEL, ERR_BAD_VERSION, ERR_CAP_TOO_SMALL,
    ERR_RESERVED_NOT_ZERO, ERR_SEAL_UNAVAILABLE, ERR_STREAM_ALIGNMENT, ERR_STREAM_EMPTY,
    ERR_TABLE_FULL, ERR_TRAILING, ERR_TRUNCATED, MAX_SESSIONS, OK, STEP_STATUS_LEN,
};

// ---------------------------------------------------------------------------
// LCG — donor Knuth constants (nato feature is off here; local const seed)
// ---------------------------------------------------------------------------

/// The guest ABI's handle table is a process-global `static mut` shared by
/// every test in this binary. The cargo harness runs tests in parallel
/// threads, so every test that drives the table takes this lock: without it
/// concurrent sessions cross-poison each other's handles (measured: 6/9
/// failures, SDC=6947, all from cross-test interference, none from the ABI).
static ABI_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Locks the ABI table, recovering from a poisoned lock: the parent's table
/// itself recovers poison via `into_inner` (`guest.rs` `with_table`), and a
/// panicking sibling test must cascade a PoisonError into 4 other failures —
/// one root failure is one failure, not six.
fn lock_abi() -> std::sync::MutexGuard<'static, ()> {
    ABI_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        // High bits of an LCG have full-period quality; sample those.
        (self.next() >> 32) % n
    }
}

/// Fixed seed: the fast and full tiers share it, so full subsumes fast.
const SEU_SEED: u64 = 0x5746_3039_0000_0001; // "WF09" — wire-fault court
/// Seal-corruption seed, same prefix discipline.
const SEAL_SEED: u64 = 0x5746_3130_0000_0001; // "WF10" — seal-boundary court

// ---------------------------------------------------------------------------
// Fixture: a lawful model + lawful 2-event stream
// ---------------------------------------------------------------------------

/// Minimal lawful model: `handle` then a choice (`fork` or skip).
const MODEL_DSL: &str = "\
activity handle = log.handle
activity fork = robot.fork

sequence {
    handle
    xor {
        fork
        silent
    }
}
";

/// Net transition ids follow atom order: `handle` = tid 0, `fork` = tid 1.
const TID_HANDLE: u64 = 0;
const TID_FORK: u64 = 1;

/// OCEL64 event word: activity bits [7:0], tick bits [15:8].
fn event_word(activity: u64, tick: u64) -> u64 {
    (tick << 8) | activity
}

/// The lawful 2-event stream for [`MODEL_DSL`]: handle(1), fork(2).
fn lawful_stream() -> [u64; 2] {
    [event_word(TID_HANDLE, 1), event_word(TID_FORK, 2)]
}

fn stream_bytes() -> Vec<u8> {
    lawful_stream().iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// CTTG wire buffer for a DSL string:
/// magic[0..4] ver[4] rsv[5] model_len_le16[6..8] rsv2[8..16] dsl[16..].
fn net_wire(dsl: &str) -> Vec<u8> {
    let mut buf = vec![0u8; 16 + dsl.len()];
    buf[0..4].copy_from_slice(b"CTTG");
    buf[4] = 0x01;
    buf[5] = 0x00;
    let len = u16::try_from(dsl.len()).expect("model fits u16");
    buf[6..8].copy_from_slice(&len.to_le_bytes());
    // reserved2 [8..16] stays zero
    buf[16..].copy_from_slice(dsl.as_bytes());
    buf
}

/// Independent kernel oracle: replay `words` through the real kernel and
/// return the terminal (full-quota) outcome.
fn oracle_replay(dsl: &str, words: &[u64]) -> [u8; STEP_STATUS_LEN] {
    // The oracle mirrors the ABI's documented record-encoding law on top of
    // an independent kernel replay (including the Violation case: the
    // summary is the kernel's finish() summary, so a violation on the very
    // first event — steps_executed == 0 — is a LAWFUL record, not SDC).
    let model = PowlModel::from_powl_str(dsl).expect("oracle model parses (lawful checked above)");
    let mut session = ConformanceSession::from_model(&model).expect("oracle model compiles");
    session.attach_stream(words).expect("oracle attach of nonempty stream succeeds");
    let status = session.step_budget(u32::MAX);
    let mut rec = [0u8; STEP_STATUS_LEN];
    match &status {
        YieldStatus::Yielded { remaining_events } => {
            rec[0] = 0;
            rec[1..5].copy_from_slice(&(*remaining_events as u32).to_le_bytes());
        }
        YieldStatus::Complete(summary) => {
            rec[0] = 1;
            rec[5..9].copy_from_slice(&summary.steps_executed.to_le_bytes());
            rec[9..13].copy_from_slice(&(summary.events_total as u32).to_le_bytes());
            rec[13..21].copy_from_slice(&summary.final_marking.to_le_bytes());
        }
        YieldStatus::Violation(err) => {
            // The Violation variant carries the ConformanceError; the
            // record's summary fields mirror the session's finish() summary,
            // which is what the ABI writes — derive it from the error the
            // same way the kernel report does (steps = error step, totals
            // unchanged). Fall back to finish() for exactness.
            rec[0] = 2;
            let summary = session.finish();
            let _ = err;
            rec[5..9].copy_from_slice(&summary.steps_executed.to_le_bytes());
            rec[9..13].copy_from_slice(&(summary.events_total as u32).to_le_bytes());
            rec[13..21].copy_from_slice(&summary.final_marking.to_le_bytes());
        }
    }
    rec
}

// ---------------------------------------------------------------------------
// The ABI calls under the court (unsafe boundary wrapped once, here)
// ---------------------------------------------------------------------------

/// Calls `ctt_session_new` with `net`; returns `(status, handle)`.
///
/// # Panics
///
/// Never; all failure travels through the status.
unsafe fn abi_new(net: &[u8]) -> (i32, u64) {
    let mut handle: u64 = 0;
    let st = ctt_wasm_guest::ctt_session_new(net.as_ptr(), net.len() as u32, &mut handle);
    (st, handle)
}

/// Calls `ctt_session_attach` with `stream`.
unsafe fn abi_attach(handle: u64, stream: &[u8]) -> i32 {
    ctt_wasm_guest::ctt_session_attach(handle, stream.as_ptr(), stream.len() as u32)
}

/// Calls `ctt_session_step`; returns `(status, 21-byte record)`.
unsafe fn abi_step(handle: u64, quota: u32) -> (i32, [u8; STEP_STATUS_LEN]) {
    let mut rec = [0u8; STEP_STATUS_LEN];
    let st =
        ctt_wasm_guest::ctt_session_step(handle, quota, rec.as_mut_ptr(), STEP_STATUS_LEN as u32);
    (st, rec)
}

/// Calls `ctt_session_drop`.
unsafe fn abi_drop(handle: u64) {
    let _ = ctt_wasm_guest::ctt_session_drop(handle);
}

// ---------------------------------------------------------------------------
// Refusal-space classification
// ---------------------------------------------------------------------------

/// Every documented typed-refusal status of the v1 ABI. A corruption sweep
/// hits `REFUSED` iff the call returns one of these. A negative status
/// (trap-class) cannot be returned by the native rlib build; a panic in the
/// native build aborts the test and is thereby caught, not credited as
/// fail-closed.
const fn is_typed_refusal(st: i32) -> bool {
    matches!(
        st,
        ERR_BAD_MAGIC
            | ERR_BAD_VERSION
            | ERR_TRUNCATED
            | ERR_TRAILING
            | ERR_RESERVED_NOT_ZERO
            | ERR_BAD_MODEL
            | ERR_TABLE_FULL
            | ERR_BAD_HANDLE
            | ERR_STREAM_EMPTY
            | ERR_CAP_TOO_SMALL
            | ERR_SEAL_UNAVAILABLE
            | ERR_STREAM_ALIGNMENT
    )
}

/// Wire-law oracle for the CTTG net buffer, re-derived from the documented
/// layout (NOT by calling the ABI). `Err` = the wire law itself refuses.
fn net_wire_law(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() < 16 {
        return Err("truncated header");
    }
    if bytes[0..4] != *b"CTTG" {
        return Err("bad magic");
    }
    if bytes[4] != 0x01 {
        return Err("bad version");
    }
    if bytes[5] != 0 {
        return Err("reserved nonzero");
    }
    let model_len = u16::from_le_bytes([bytes[6], bytes[7]]);
    if bytes[8..16].iter().any(|&b| b != 0) {
        return Err("reserved2 nonzero");
    }
    if bytes.len() < 16 + model_len as usize {
        return Err("truncated model");
    }
    if bytes.len() > 16 + model_len as usize {
        return Err("trailing bytes");
    }
    Ok(())
}

/// `Some(words)` when the stream bytes are a well-formed LE u64 array.
fn stream_words(bytes: &[u8]) -> Option<Vec<u64>> {
    if !bytes.len().is_multiple_of(8) {
        return None;
    }
    Some(
        bytes
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
            .collect(),
    )
}

// ---------------------------------------------------------------------------
// Tier 1 — SEU single-bit sweeps (net / stream / handle surfaces)
// ---------------------------------------------------------------------------

/// Outcome census of one sweep pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Census {
    refused: u64,
    lawful: u64,
    sdc: u64,
}

/// One SEU pass: for each LCG flip, corrupt one byte of the net buffer, the
/// stream buffer, or the handle bytes, drive the full new->attach->step
/// sequence, and classify against the oracle. `step_shim` is the step call —
/// the positive control substitutes a corrupted-accepting shim here.
fn run_seu_sweep<Step>(flips: u64, mut step_shim: Step) -> Census
where
    Step: FnMut(u64, u32) -> (i32, [u8; STEP_STATUS_LEN]),
{
    let mut rng = Lcg(SEU_SEED);
    let net_clean = net_wire(MODEL_DSL);
    let stream_clean = stream_bytes();
    let mut c = Census::default();

    for _ in 0..flips {
        // Surface choice: 0 = net buffer, 1 = stream buffer, 2 = handle.
        let surface = rng.below(3);
        let net: Vec<u8> = {
            let mut n = net_clean.clone();
            if surface == 0 {
                let idx = rng.below(net_clean.len() as u64) as usize;
                n[idx] ^= 1 << rng.below(8);
            }
            n
        };
        let stream: Vec<u8> = {
            let mut s = stream_clean.clone();
            if surface == 1 {
                let idx = rng.below(stream_clean.len() as u64) as usize;
                s[idx] ^= 1 << rng.below(8);
            }
            s
        };

        // Fresh lawful session so each flip sees an unpolluted table.
        let (st_new, handle0) = unsafe { abi_new(&net) };
        let handle = if surface == 2 {
            let mut hb = handle0.to_le_bytes();
            let byte_idx = rng.below(8) as usize;
            hb[byte_idx] ^= 1 << rng.below(8);
            u64::from_le_bytes(hb)
        } else {
            handle0
        };

        // --- Oracle --------------------------------------------------------
        // Net oracle: documented wire law + real kernel parse.
        // The oracle replays against the ACTUAL net bytes' DSL (a net flip
        // that survives parse law names a different-but-lawful model; the
        // ABI compiles the mutated model, so the oracle must too).
        let net_dsl = core::str::from_utf8(&net[16..]).unwrap_or("\u{7F}");
        let net_lawful = net_wire_law(&net).is_ok() && PowlModel::from_powl_str(net_dsl).is_ok();
        // Stream oracle: word-aligned nonempty streams replay in the kernel;
        // the session law refuses everything else at attach.
        // Deferred: only the kernel oracle runs here, and only for a lawful
        // net — an unlawful net never reaches the replay, and replaying it
        // would panic in the parser (the ABI never sees that panic; it
        // refuses the wire at the law gate first).
        let stream_oracle: Option<[u8; STEP_STATUS_LEN]> = if net_lawful {
            stream_words(&stream)
                .filter(|w| !w.is_empty())
                .map(|w| oracle_replay(net_dsl, &w))
        } else {
            None
        };

        // --- Drive the ABI -------------------------------------------------
        if st_new != OK {
            if !net_lawful && is_typed_refusal(st_new) {
                c.refused += 1;
            } else {
                c.sdc += 1;
            }
            continue;
        }
        if !net_lawful {
            c.sdc += 1; // wrong-Ok: oracle refuses, ABI admitted
            unsafe { abi_drop(handle0) };
            continue;
        }
        let st_attach = unsafe { abi_attach(handle, &stream) };
        if st_attach != OK {
            // Lawful refusal iff the stream is unlawful OR the handle bytes
            // were corrupted (handle law -> ERR_BAD_HANDLE). A refusal of a
            // lawful stream on an uncorrupted handle is a wrong-refusal.
            let input_unlawful = stream_oracle.is_none() || handle != handle0;
            if input_unlawful && is_typed_refusal(st_attach) {
                c.refused += 1;
            } else {
                c.sdc += 1;
            }
            // Drop the TRUE handle (handle0), never the corrupted handle
            // bytes: dropping a corrupted handle is a no-op (ERR_BAD_HANDLE
            // inside ctt_session_drop) and leaks the real slot, filling the
            // 8-slot table and poisoning every later flip with
            // ERR_TABLE_FULL. handle0 is valid whenever st_new == OK, which
            // is the only way control reaches here.
            unsafe { abi_drop(handle0) };
            continue;
        }
        let Some(oracle) = stream_oracle else {
            c.sdc += 1; // wrong-Ok attach of a torn/unlawful stream
            unsafe { abi_drop(handle0) };
            continue;
        };
        let (st_step, rec) = step_shim(handle, DEFAULT_REDUCTION_QUOTA);
        unsafe { abi_drop(handle0) };
        if st_step != OK {
            if is_typed_refusal(st_step) {
                c.refused += 1;
            } else {
                c.sdc += 1;
            }
            continue;
        }
        // Compare the admitted record byte-for-byte against the oracle's
        // independently derived record (full law, incl. Violation summaries).
        let matches = rec == oracle;
        if matches {
            c.lawful += 1;
        } else {
            c.sdc += 1;
        }
    }
    c
}

/// Fast-tier flip budget (default).
const FAST_N: u64 = 10_000;
/// Full-tier flip budget: donor scale, same seed — full subsumes fast.
const FULL_N: u64 = 1_000_000;

#[test]
fn seu_single_bit_never_wrong_ok_fast() {
    let _table = lock_abi();
    let c = run_seu_sweep(FAST_N, |h, q| unsafe { abi_step(h, q) });
    assert_eq!(c.refused + c.lawful + c.sdc, FAST_N, "every fault must land in exactly one class");
    assert_eq!(
        c.sdc, 0,
        "SDC must be 0.000% — the ABI produced a wrong-Ok or diverged from the oracle"
    );
    assert!(c.refused > 0, "court vacuous if no corruption was ever refused");
    assert!(c.lawful > 0, "court vacuous if no clean-path admission was exercised");
}

#[test]
#[ignore = "nato-tier3"]
fn seu_single_bit_never_wrong_ok_full() {
    let _table = lock_abi();
    let c = run_seu_sweep(FULL_N, |h, q| unsafe { abi_step(h, q) });
    assert_eq!(c.refused + c.lawful + c.sdc, FULL_N);
    assert_eq!(c.sdc, 0, "SDC at donor scale — silent corruption detected");
    assert!(c.refused > 0);
}

// ---------------------------------------------------------------------------
// Handle-table slot-byte tier: every handle-bit corruption must resolve via
// the documented handle law — slot < MAX_SESSIONS AND generation matches the
// generation of an occupied slot — never silently aliasing a freed slot.
// ---------------------------------------------------------------------------

#[test]
fn handle_bit_flips_obey_handle_law() {
    let _table = lock_abi();
    let mut rng = Lcg(SEU_SEED ^ 0x0000_0000_0000_BEEF);
    let net = net_wire(MODEL_DSL);
    // Two occupied slots (whatever generations the shared table currently
    // holds — earlier tests' drops bump per-slot generations, so this court
    // must be reset-agnostic, never assume gen 0 / slots 0..=1).
    let (st0, h0) = unsafe { abi_new(&net) };
    let (st1, h1) = unsafe { abi_new(&net) };
    assert_eq!(st0, OK);
    assert_eq!(st1, OK);
    assert_ne!(h0, h1, "distinct sessions must have distinct handles");

    for _ in 0..2_000 {
        let target = if rng.below(2) == 0 { h0 } else { h1 };
        let mut hb = target.to_le_bytes();
        let byte_idx = rng.below(8) as usize;
        hb[byte_idx] ^= 1 << rng.below(8);
        let corrupted = u64::from_le_bytes(hb);

        // Oracle: documented handle law, re-derived here, RELATIVE to the
        // handles this court actually holds. A single-bit flip of `target`
        // either (a) lands on the exact bit pattern of the other live
        // handle (h0 or h1 — the only occupied slot+generation pairs in the
        // table, and unoccupied slots refuse regardless of generation), or
        // (b) names a stale/foreign slot+generation -> ERR_BAD_HANDLE.
        let slot = (corrupted & 0xFFFF_FFFF) as usize;
        let generation = (corrupted >> 32) as u32;
        let expected_ok = corrupted == h0 || corrupted == h1;
        let _ = (slot, generation); // kept for the diagnostic in the asserts below

        let st = unsafe { abi_attach(corrupted, &stream_bytes()) };
        if expected_ok {
            assert_eq!(
                st, OK,
                "lawful handle corruption (slot {slot}, gen {generation}) must not be refused"
            );
        } else {
            assert_eq!(
                st, ERR_BAD_HANDLE,
                "unlawful handle (slot {slot}, gen {generation}) must refuse ERR_BAD_HANDLE"
            );
        }
    }
    unsafe { abi_drop(h0) };
    unsafe { abi_drop(h1) };
}

// ---------------------------------------------------------------------------
// Tier 2 — torn-input courts: mid-write splits, zeroed halves, swapped halves
// ---------------------------------------------------------------------------

/// A torn input must refuse typed (documented refusal space), never admit.
fn assert_torn_refused(tag: &'static str, st: i32) {
    assert_ne!(st, OK, "torn input `{tag}` was ADMITTED (wrong-Ok)");
    assert!(is_typed_refusal(st), "torn input `{tag}` refused with untyped status {st}");
}

#[test]
fn torn_net_input_fails_closed() {
    let _table = lock_abi();
    let clean = net_wire(MODEL_DSL);
    let mid = clean.len() / 2;

    // (a) Truncation at every prefix class around the header boundary and
    //     the full-body boundary.
    for len in [0usize, 1, 4, 8, 15, 16, mid, clean.len() - 1] {
        let (st, _h) = unsafe { abi_new(&clean[..len]) };
        assert_torn_refused("truncated net", st);
    }

    // (b) Second half zeroed: the model bytes blank out; the length field
    //     still claims a model that no longer parses.
    let mut zeroed = clean.clone();
    zeroed[mid..].fill(0);
    let (st, _h) = unsafe { abi_new(&zeroed) };
    assert_torn_refused("zeroed second half of net", st);

    // (c) Swapped halves: header displaced — magic/version must fire.
    // Concatenate instead of two copy_from_slice calls: MODEL_DSL length is
    // odd, so clean.len()/2 floors and the half lengths differ by one.
    let mut swapped = clean[mid..].to_vec();
    swapped.extend_from_slice(&clean[..mid]);
    let (st, _h) = unsafe { abi_new(&swapped) };
    assert_torn_refused("swapped net halves", st);

    // No partial admission / no table poisoning: a lawful session still
    // succeeds after every torn refusal above.
    let (st, h) = unsafe { abi_new(&clean) };
    assert_eq!(st, OK, "handle table poisoned by torn refusals");
    unsafe { abi_drop(h) };
}

#[test]
fn torn_stream_input_fails_closed() {
    let _table = lock_abi();
    let clean = stream_bytes();
    let (st_new, h) = unsafe { abi_new(&net_wire(MODEL_DSL)) };
    assert_eq!(st_new, OK);

    // (a) Truncation mid-write: every non-multiple-of-8 length around the
    //     word boundaries is a torn stream.
    for len in [1usize, 4, 7, 9, 12, 15] {
        let st = unsafe { abi_attach(h, &clean[..len]) };
        assert_torn_refused("truncated stream", st);
    }

    // (b) Second half zeroed: word 2 becomes activity 0 / tick 0. The ABI
    //     must agree with the kernel oracle — never a clean Complete on a
    //     stream the oracle does not Complete. FRESH session per sub-case:
    //     the kernel law latches Phase::Faulted on violation and attach does
    //     not reset it, so a shared session would re-report the previous
    //     sub-case's latched fault (measured divergence, kernel-confirmed).
    let mut zeroed = clean.clone();
    zeroed[8..].fill(0);
    let words = stream_words(&zeroed).unwrap();
    let oracle = oracle_replay(MODEL_DSL, &words);
    let (st_b, h_b) = unsafe { abi_new(&net_wire(MODEL_DSL)) };
    assert_eq!(st_b, OK, "fresh session for (b) refused");
    let st = unsafe { abi_attach(h_b, &zeroed) };
    if st == OK {
        let (st_step, rec) = unsafe { abi_step(h_b, DEFAULT_REDUCTION_QUOTA) };
        assert_eq!(st_step, OK, "step after attach of zeroed half refused");
        assert_eq!(rec[0], oracle[0], "zeroed-half stream diverges from the kernel oracle");
        assert_eq!(rec, oracle, "record diverges from oracle");
    } else {
        assert!(is_typed_refusal(st), "zeroed half refused untyped {st}");
    }
    unsafe { abi_drop(h_b) };

    // (c) Swapped halves: word order reversed — the kernel oracle replays
    //     the swapped words; the ABI must agree, or refuse typed. FRESH
    //     session, same latching law as (b).
    let mut swapped = vec![0u8; clean.len()];
    swapped[..8].copy_from_slice(&clean[8..]);
    swapped[8..].copy_from_slice(&clean[..8]);
    let words = stream_words(&swapped).unwrap();
    let oracle = oracle_replay(MODEL_DSL, &words);
    let (st_c, h_c) = unsafe { abi_new(&net_wire(MODEL_DSL)) };
    assert_eq!(st_c, OK, "fresh session for (c) refused");
    let st = unsafe { abi_attach(h_c, &swapped) };
    if st == OK {
        let (st_step, rec) = unsafe { abi_step(h_c, DEFAULT_REDUCTION_QUOTA) };
        assert_eq!(st_step, OK);
        assert_eq!(rec[0], oracle[0], "swapped stream diverges from the kernel oracle");
        assert_eq!(rec, oracle, "record diverges from oracle");
    } else {
        assert!(is_typed_refusal(st), "swapped stream refused untyped {st}");
    }

    unsafe { abi_drop(h_c) };
    unsafe { abi_drop(h) };
}

#[test]
fn torn_refusals_never_partial_admit() {
    let _table = lock_abi();
    // After every torn refusal above, the table must be intact: MAX_SESSIONS
    // lawful sessions still succeed, proving no leaked or poisoned slots.
    let net = net_wire(MODEL_DSL);
    for _ in 0..MAX_SESSIONS {
        let (st, h) = unsafe { abi_new(&net) };
        assert_eq!(st, OK, "slot leak after torn refusals: table full");
        unsafe { abi_drop(h) };
    }
}

// ---------------------------------------------------------------------------
// Tier 3 — torn-write across the seal boundary (the N11 analogue)
// ---------------------------------------------------------------------------

/// Builds a clean 352-byte guest receipt wire.
fn clean_wire() -> [u8; 352] {
    use chicago_tdd_tools::conformance::guest_seal::{encode_receipt_wire, guest_receipt};
    encode_receipt_wire(&guest_receipt([0xA5; 32], 12, 12, 0, [0x5A; 32]))
}

#[test]
fn seal_boundary_bitflip_never_admitted() {
    use chicago_tdd_tools::ledger::receipt::VerifiedCourtReceipt;

    let clean = clean_wire();
    // Baseline: the uncorrupted wire MUST verify.
    assert!(
        VerifiedCourtReceipt::from_bytes_verified(&clean).is_ok(),
        "uncorrupted guest wire must be host-verifiable"
    );

    // (a) Exhaustive single-bit: all 352 * 8 = 2,816 flips must reject.
    let mut rejected = 0u64;
    for byte_idx in 0..clean.len() {
        for bit in 0..8u32 {
            let mut corrupt = clean;
            corrupt[byte_idx] ^= 1 << bit;
            assert!(
                VerifiedCourtReceipt::from_bytes_verified(&corrupt).is_err(),
                "single-bit flip (byte {byte_idx}, bit {bit}) was ADMITTED as a verified receipt"
            );
            rejected += 1;
        }
    }
    assert_eq!(rejected, 2_816, "exhaustive flip census mismatch");

    // (b) Exhaustive byte-wide: every byte fully flipped must reject.
    for byte_idx in 0..clean.len() {
        let mut corrupt = clean;
        corrupt[byte_idx] ^= 0xFF;
        assert!(
            VerifiedCourtReceipt::from_bytes_verified(&corrupt).is_err(),
            "byte-wide corruption (byte {byte_idx}) was ADMITTED as a verified receipt"
        );
    }
}

/// Fast-tier budget for the seal random-corruption sweep.
const SEAL_FAST_N: u64 = 10_000;
/// Full-tier budget (donor scale, same seed prefix discipline).
const SEAL_FULL_N: u64 = 1_000_000;

/// Shared LCG random-corruption seal sweep. Same seed at both tiers, so the
/// full stream's prefix is the fast stream — full subsumes fast.
fn run_seal_sweep(n: u64) {
    use chicago_tdd_tools::ledger::receipt::VerifiedCourtReceipt;

    let clean = clean_wire();
    let mut rng = Lcg(SEAL_SEED);
    for _ in 0..n {
        let mut corrupt = clean;
        let n_bytes = 1 + rng.below(4) as usize;
        for _ in 0..n_bytes {
            let idx = rng.below(clean.len() as u64) as usize;
            corrupt[idx] = (rng.next() & 0xFF) as u8;
        }
        // An LCG byte can land on the original value: that is identity, not
        // corruption, and the verifier rightly admits the clean wire. Skip
        // non-corruptions instead of crediting the verifier with a reject.
        if corrupt == clean {
            continue;
        }
        assert!(
            VerifiedCourtReceipt::from_bytes_verified(&corrupt).is_err(),
            "random corruption was ADMITTED as a verified receipt"
        );
    }
}

#[test]
fn seal_boundary_random_corruption_fast() {
    run_seal_sweep(SEAL_FAST_N);
}

#[test]
#[ignore = "nato-tier3"]
fn seal_boundary_random_corruption_full() {
    run_seal_sweep(SEAL_FULL_N);
}

// ---------------------------------------------------------------------------
// Positive controls: corrupted-accepting shims MUST be caught
// ---------------------------------------------------------------------------

/// MUTANT: a step shim that admits ANY corrupted stream with Ok + a canned
/// Complete record (the corrupted-accepting guest). The SEU sweep machinery
/// must classify its behavior as SDC.
fn leaky_step_shim(_handle: u64, _quota: u32) -> (i32, [u8; STEP_STATUS_LEN]) {
    let mut rec = [0u8; STEP_STATUS_LEN];
    rec[0] = 1; // Complete, regardless of corruption
    (OK, rec)
}

#[test]
fn positive_control_leaky_step_shim_is_caught() {
    let _table = lock_abi();
    // The sweep machinery, run against the leaky shim on the clean fixture:
    // the shim reports Complete even where the oracle says refusal/violation
    // or a different summary — SDC must fire.
    let c = run_seu_sweep(256, leaky_step_shim);
    assert!(c.sdc > 0, "corrupted-accepting shim was NOT caught — sweep machinery vacuous");
}

/// MUTANT: a verify shim that accepts any 352-byte record.
fn leaky_verify(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() == 352 {
        Ok(())
    } else {
        Err("length")
    }
}

#[test]
fn positive_control_leaky_verify_is_caught() {
    let clean = clean_wire();
    let mut corrupt = clean;
    corrupt[0] ^= 0xFF; // smash the magic
                        // The leaky shim accepts the corruption (control setup is honest).
    assert!(
        leaky_verify(&corrupt).is_ok(),
        "control setup: leaky verify must accept the corruption"
    );
    // The REAL verifier rejects the same corruption — the contrast proves
    // the court's verifier is load-bearing and the seal court non-vacuous.
    assert!(
        chicago_tdd_tools::ledger::receipt::VerifiedCourtReceipt::from_bytes_verified(&corrupt)
            .is_err(),
        "real verifier admitted magic corruption — seal court vacuous"
    );
}
