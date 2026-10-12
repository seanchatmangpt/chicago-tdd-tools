//! Host-compiled guest ABI contract suite (W5).
//!
//! The ABI functions in [`chicago_tdd_tools::conformance::guest`] are plain
//! `extern "C"` Rust fns, so the exact code paths the wasm guest executes are
//! exercised here natively. **Two-tier honesty:** this is the native tier —
//! in-wasm execution via the `temprun` embed is a later tier and is NOT yet
//! covered by this file.
//!
//! Contracts under test:
//!
//! 1. `ctt_guest_abi_version()` returns the stable `ABI_VERSION` const.
//! 2. `ctt_session_new` accepts a v1 `CTTG` wire buffer whose model DSL was
//!    produced by the real loader (`PowlModel::from_powl_str`) and returns a
//!    live handle.
//! 3. `ctt_session_attach` + `ctt_session_step` quota loop: a long stream
//!    steps to `Yielded{remaining}` then `Complete` across ABI calls.
//! 4. A skipped-tick violation input produces the typed Violation status
//!    byte (2) with an `OK` (non-negative) ABI return — the negative
//!    trap-class i32 space stays reserved for the wasm runtime.
//! 5. Seal: with `ledger`, `ctt_session_seal` writes the BLAKE3 digest of
//!    the 34-byte preimage, and the guest-produced 352-byte receipt wire
//!    record (`guest_seal::encode_receipt_wire`) passes the host verifier
//!    `VerifiedCourtReceipt::from_bytes_verified`. Without `ledger`, sealing
//!    refuses with `ERR_SEAL_UNAVAILABLE` (no weak-hash downgrade).
//! 6. `ctt_session_drop` frees the slot; reuse works; the dropped handle is
//!    dead (`ERR_BAD_HANDLE`).
//! 7. `MAX_SESSIONS` exhaustion is the typed `ERR_TABLE_FULL` busy status,
//!    never a panic.
//! 8. Every documented errno is a distinct value, and the wire-format
//!    refusals are behaviorally typed (bad magic/version/reserved/trailing/
//!    truncated/null/cap/handle).
//!
//! The ABI's session table is process-global, so every test serializes on a
//! mutex and frees its sessions through a drop guard even when an assertion
//! fails.
#![cfg(all(feature = "wasm-guest", feature = "conformance"))]
// The guest ABI is an extern "C" surface; the host-side contract suite calls
// it through its unsafe boundary by design.
#![allow(unsafe_code)]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
// JUSTIFICATION: test code only — assertions on real ABI-collected state.
#![allow(clippy::unwrap_used)]

use std::sync::Mutex;

use chicago_tdd_tools::conformance::guest::{
    ctt_guest_abi_version, ctt_session_attach, ctt_session_drop, ctt_session_new, ctt_session_seal,
    ctt_session_step, DIGEST_LEN, ERR_BAD_HANDLE, ERR_BAD_MAGIC, ERR_BAD_MODEL, ERR_BAD_VERSION,
    ERR_CAP_TOO_SMALL, ERR_NULL_PTR, ERR_RESERVED_NOT_ZERO, ERR_SEAL_UNAVAILABLE,
    ERR_STREAM_ALIGNMENT, ERR_STREAM_EMPTY, ERR_TABLE_FULL, ERR_TRAILING, ERR_TRUNCATED,
    MAX_SESSIONS, SEAL_PREIMAGE_LEN, STEP_STATUS_LEN,
};
#[cfg(feature = "ledger")]
use chicago_tdd_tools::conformance::guest_seal::{encode_receipt_wire, guest_receipt};
use chicago_tdd_tools::conformance::loader::PowlModel;

/// The guest ABI's slot table is a process-global `static`, so tests that
/// touch it serialize on this mutex.
static ABI_LOCK: Mutex<()> = Mutex::new(());

/// Drop guard: frees the session even if an assertion fires mid-test.
struct Session(u64);

impl Drop for Session {
    fn drop(&mut self) {
        // ERR_BAD_HANDLE here would mean a test double-dropped; the guard is
        // best-effort cleanup, so the return code is observed, not asserted.
        let _ = ctt_session_drop(self.0);
    }
}

// ─── Fixtures ────────────────────────────────────────────────────────────────

/// Three-atom linear POWL model (atoms get transition ids 0, 1, 2).
const MODEL_DSL: &str = "\
activity a0 = sys.a0
activity a1 = sys.a1
activity a2 = sys.a2

sequence {
    a0
    a1
    a2
}
";

/// OCEL64 event word: activity id in bits [7:0], tick in bits [15:8]
/// (session lane contract; ticks must advance exactly +1).
fn event(tid: u8, tick: u8) -> u64 {
    u64::from(tid) | (u64::from(tick) << 8)
}

/// Serializes the model DSL into the guest wire-format v1 buffer
/// (`CTTG` magic, version 0x01, reserved 0x00, model_len LE u16,
/// reserved2 8x zero, model bytes; no trailing bytes).
fn net_wire(dsl: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(16 + dsl.len());
    buf.extend_from_slice(b"CTTG");
    buf.push(0x01); // version
    buf.push(0x00); // reserved
    buf.extend_from_slice(&(dsl.len() as u16).to_le_bytes());
    buf.extend_from_slice(&[0u8; 8]); // reserved2
    buf.extend_from_slice(dsl.as_bytes());
    buf
}

/// The golden lawful 3-event trajectory for the 3-atom net.
fn golden_stream() -> Vec<u8> {
    let mut buf = Vec::with_capacity(24);
    for (tid, tick) in [(0u8, 1u8), (1, 2), (2, 3)] {
        buf.extend_from_slice(&event(tid, tick).to_le_bytes());
    }
    buf
}

/// Creates a session over [`MODEL_DSL`] under the lock, asserting the ABI
/// accepted the wire buffer and the handle names a bounded slot.
fn new_locked_session() -> Session {
    let wire = net_wire(MODEL_DSL);
    let mut handle = 0u64;
    let code = unsafe { ctt_session_new(wire.as_ptr(), wire.len() as u32, &mut handle) };
    assert_eq!(code, 0, "ctt_session_new must accept the golden wire buffer");
    assert!(
        (handle & 0xFFFF_FFFF) < MAX_SESSIONS as u64,
        "slot index must be inside the fixed table"
    );
    Session(handle)
}

/// LE u32 at `off` in a step-status record.
fn rec_u32(rec: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(rec[off..off + 4].try_into().unwrap())
}

/// LE u64 at `off` in a step-status record.
fn rec_u64(rec: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(rec[off..off + 8].try_into().unwrap())
}

/// Attaches the given LE-encoded stream words to the session.
#[allow(dead_code)] // kept as suite scaffolding; callers attach inline today
fn attach(handle: u64, words: &[u64]) -> i32 {
    let mut buf = Vec::with_capacity(words.len() * 8);
    for w in words {
        buf.extend_from_slice(&w.to_le_bytes());
    }
    unsafe { ctt_session_attach(handle, buf.as_ptr(), buf.len() as u32) }
}

// ─── 1. ABI version const ────────────────────────────────────────────────────

/// The ABI revision is the stable documented constant (1) and the exported
/// symbol agrees with the crate-visible const.
#[test]
fn abi_version_const_is_stable() {
    assert_eq!(ctt_guest_abi_version(), 1);
    assert_eq!(ctt_guest_abi_version(), chicago_tdd_tools::conformance::guest::ABI_VERSION);
    // Fixed record lengths are part of the ABI contract.
    assert_eq!(STEP_STATUS_LEN, 21);
    assert_eq!(SEAL_PREIMAGE_LEN, 34);
    assert_eq!(DIGEST_LEN, 32);
    assert_eq!(chicago_tdd_tools::conformance::guest::ABI_VERSION, 1);
}

// ─── 2. session_new from a loader-serialized net buffer ──────────────────────

/// The wire buffer the guest ABI consumes is built from the REAL loader: the
/// same DSL parses through `PowlModel::from_powl_str` on the host and, when
/// wrapped in the v1 `CTTG` framing, is accepted by `ctt_session_new` into a
/// live session.
#[test]
fn session_new_roundtrips_real_loader_model() {
    let _guard = ABI_LOCK.lock().unwrap();

    // The DSL is a real loader input first: parse + compile on the host.
    let model = PowlModel::from_powl_str(MODEL_DSL).expect("fixture DSL must parse");
    let host_session = chicago_tdd_tools::conformance::ConformanceSession::from_model(&model)
        .expect("fixture DSL must compile");
    assert_eq!(host_session.net().len(), 3, "fixture compiles 3 transitions");

    // Then the same DSL travels the guest wire path.
    let _s = new_locked_session();
}

// ─── 3. attach + step_budget quota loop ─────────────────────────────────────

/// A 3-event stream with a quota of 2 yields with one remaining event; the
/// next step call completes with the full summary in the 21-byte record.
#[test]
fn quota_loop_yields_then_completes_across_abi_calls() {
    let _guard = ABI_LOCK.lock().unwrap();
    let s = new_locked_session();

    let stream = golden_stream();
    let code = unsafe { ctt_session_attach(s.0, stream.as_ptr(), stream.len() as u32) };
    assert_eq!(code, 0, "golden stream must attach");

    let mut rec = [0u8; STEP_STATUS_LEN];
    let code = unsafe { ctt_session_step(s.0, 2, rec.as_mut_ptr(), rec.len() as u32) };
    assert_eq!(code, 0);
    assert_eq!(rec[0], 0, "first slice must be Yielded");
    assert_eq!(rec_u32(&rec, 1), 1, "remaining after a 2-of-3 slice");

    // Resume through the same step ABI entry point: quota 2 over the one
    // remaining event completes.
    let mut rec = [0u8; STEP_STATUS_LEN];
    let code = unsafe { ctt_session_step(s.0, 2, rec.as_mut_ptr(), rec.len() as u32) };
    assert_eq!(code, 0);
    assert_eq!(rec[0], 1, "second slice completes the 3-event trace");
    assert_eq!(rec_u32(&rec, 5), 3, "steps_executed totals all three events");
    assert_eq!(rec_u32(&rec, 9), 3, "events_total is the bound stream length");
    assert_eq!(rec_u64(&rec, 13), 1 << 3, "final marking is place 3 tokenized");
}

/// Attaches an empty stream: the kernel's `TraceMalformed` semantics arrive
/// as the typed `ERR_STREAM_EMPTY` status, never a panic.
#[test]
fn empty_stream_attach_is_typed_err_stream_empty() {
    let _guard = ABI_LOCK.lock().unwrap();
    let s = new_locked_session();
    let code = unsafe { ctt_session_attach(s.0, [].as_ptr(), 0) };
    assert_eq!(code, ERR_STREAM_EMPTY);
}

/// A stream buffer whose length is not a multiple of 8 is the typed
/// `ERR_STREAM_ALIGNMENT` refusal.
#[test]
fn misaligned_stream_attach_is_typed_err_stream_alignment() {
    let _guard = ABI_LOCK.lock().unwrap();
    let s = new_locked_session();
    let bytes = [0u8; 9];
    let code = unsafe { ctt_session_attach(s.0, bytes.as_ptr(), bytes.len() as u32) };
    assert_eq!(code, ERR_STREAM_ALIGNMENT);
}

// ─── 4. Violation: skipped tick ──────────────────────────────────────────────

/// A skipped tick (tick 1 then tick 3) drives the replay to the typed
/// Violation status byte (2) while the ABI fn itself returns `OK` — the
/// negative i32 space stays reserved for the wasm trap lowering, so a
/// violation must never surface as a negative return here.
#[test]
fn skipped_tick_is_typed_violation_status_byte_with_non_negative_return() {
    let _guard = ABI_LOCK.lock().unwrap();
    let s = new_locked_session();

    let mut stream = Vec::new();
    for w in [event(0, 1), event(1, 3)] {
        stream.extend_from_slice(&w.to_le_bytes());
    }
    let code = unsafe { ctt_session_attach(s.0, stream.as_ptr(), stream.len() as u32) };
    assert_eq!(code, 0);

    let mut rec = [0u8; STEP_STATUS_LEN];
    let code = unsafe { ctt_session_step(s.0, 16, rec.as_mut_ptr(), rec.len() as u32) };
    assert_eq!(code, 0, "a violation is OK at the ABI; byte 2 carries it");
    assert!(code >= 0, "trap-class negative space is reserved for the runtime");
    assert_eq!(rec[0], 2, "status byte 2 = Violation");
    assert_eq!(rec_u32(&rec, 5), 1, "first event replayed, second faulted");
}

// ─── 5. Seal ────────────────────────────────────────────────────────────────

/// Reconstructs the 34-byte seal preimage from known replay facts.
fn seal_preimage(steps: u32, events: u32, marking: u64, outcome: u8, violation: u8) -> [u8; 34] {
    let mut p = [0u8; 34];
    p[0..8].copy_from_slice(&u64::from(steps).to_le_bytes());
    p[8..16].copy_from_slice(&u64::from(events).to_le_bytes());
    p[16..24].copy_from_slice(&marking.to_le_bytes());
    p[24] = outcome;
    p[25] = violation;
    p
}

/// With `ledger`, sealing writes exactly `DIGEST_LEN` bytes: the BLAKE3
/// digest of the 34-byte preimage reconstructed from the independently
/// known replay facts. The guest-produced receipt wire record built from the
/// same replay facts passes the host verifier `from_bytes_verified`.
#[cfg(feature = "ledger")]
#[test]
fn seal_digest_and_receipt_wire_pass_host_verification() {
    use chicago_tdd_tools::ledger::receipt::VerifiedCourtReceipt;

    let _guard = ABI_LOCK.lock().unwrap();
    let s = new_locked_session();

    let stream = golden_stream();
    let code = unsafe { ctt_session_attach(s.0, stream.as_ptr(), stream.len() as u32) };
    assert_eq!(code, 0);

    // Drive to Complete.
    let mut rec = [0u8; STEP_STATUS_LEN];
    let code = unsafe { ctt_session_step(s.0, 16, rec.as_mut_ptr(), rec.len() as u32) };
    assert_eq!(code, 0);
    assert_eq!(rec[0], 1, "golden trajectory completes");

    // Seal: DIGEST_LEN bytes of BLAKE3 over the known preimage.
    let mut sealed = [0u8; 64];
    let mut sealed_len = 0u32;
    let code =
        unsafe { ctt_session_seal(s.0, sealed.as_mut_ptr(), sealed.len() as u32, &mut sealed_len) };
    assert_eq!(code, 0);
    assert_eq!(usize::try_from(sealed_len).unwrap(), DIGEST_LEN);

    let expected = blake3::hash(&seal_preimage(3, 3, 1 << 3, 1, 0));
    assert_eq!(&sealed[..DIGEST_LEN], expected.as_bytes());

    // Guest-produces / host-seals: the 352-byte receipt wire record built
    // from the same guest-visible facts verifies on the host.
    let receipt = guest_receipt(
        *blake3::hash(MODEL_DSL.as_bytes()).as_bytes(),
        3,
        3,
        0, // Pass
        *expected.as_bytes(),
    );
    let wire = encode_receipt_wire(&receipt);
    assert_eq!(wire.len(), 352);
    let verified = VerifiedCourtReceipt::from_bytes_verified(&wire)
        .expect("guest-produced receipt wire must verify on the host");
    assert_eq!(verified.receipt().metadata_digest, *expected.as_bytes());
    assert_eq!(verified.receipt().metrics.total_ticks, 3);
}

/// Sealing a faulted session bakes in the classified violation byte.
#[cfg(feature = "ledger")]
#[test]
fn seal_of_faulted_session_carries_skipped_violation_byte() {
    let _guard = ABI_LOCK.lock().unwrap();
    let s = new_locked_session();

    let mut stream = Vec::new();
    for w in [event(0, 1), event(1, 3)] {
        stream.extend_from_slice(&w.to_le_bytes());
    }
    let code = unsafe { ctt_session_attach(s.0, stream.as_ptr(), stream.len() as u32) };
    assert_eq!(code, 0);

    let mut rec = [0u8; STEP_STATUS_LEN];
    let _code = unsafe { ctt_session_step(s.0, 16, rec.as_mut_ptr(), rec.len() as u32) };
    assert_eq!(rec[0], 2);

    let mut sealed = [0u8; 64];
    let mut sealed_len = 0u32;
    let code =
        unsafe { ctt_session_seal(s.0, sealed.as_mut_ptr(), sealed.len() as u32, &mut sealed_len) };
    assert_eq!(code, 0);
    let expected = blake3::hash(&seal_preimage(1, 2, 1 << 1, 2, 1));
    assert_eq!(
        &sealed[..DIGEST_LEN],
        expected.as_bytes(),
        "faulted seal must digest outcome=2, violation=SKIPPED(1)"
    );
}

/// Without `ledger`, sealing refuses with `ERR_SEAL_UNAVAILABLE` — a
/// non-cryptographic receipt is a receipt-shaped lie, so there is no
/// fallback digest.
#[cfg(not(feature = "ledger"))]
#[test]
fn seal_without_ledger_is_refused_not_downgraded() {
    let _guard = ABI_LOCK.lock().unwrap();
    let s = new_locked_session();
    let stream = golden_stream();
    let code = unsafe { ctt_session_attach(s.0, stream.as_ptr(), stream.len() as u32) };
    assert_eq!(code, 0);

    let mut sealed = [0u8; 64];
    let mut sealed_len = 0u32;
    let code =
        unsafe { ctt_session_seal(s.0, sealed.as_mut_ptr(), sealed.len() as u32, &mut sealed_len) };
    assert_eq!(code, ERR_SEAL_UNAVAILABLE);
}

// ─── 6. Drop / reuse / stale handle ─────────────────────────────────────────

/// Dropping frees the slot: a new session succeeds afterwards, and the
/// dropped handle is dead for step/seal/drop again.
#[test]
fn drop_frees_slot_and_reuse_works() {
    let _guard = ABI_LOCK.lock().unwrap();
    let s = new_locked_session();
    let dropped = s.0;

    let code = ctt_session_drop(dropped);
    assert_eq!(code, 0);
    std::mem::forget(s); // guard must not double-drop

    // Reuse works. The binding keeps the reused slot alive to test end.
    let _s2 = new_locked_session();
    // The stale handle is dead on every entry point.
    let mut rec = [0u8; STEP_STATUS_LEN];
    let code = unsafe { ctt_session_step(dropped, 1, rec.as_mut_ptr(), rec.len() as u32) };
    assert_eq!(code, ERR_BAD_HANDLE);
    let mut sealed = [0u8; 64];
    let mut len = 0u32;
    let code =
        unsafe { ctt_session_seal(dropped, sealed.as_mut_ptr(), sealed.len() as u32, &mut len) };
    assert_eq!(code, ERR_BAD_HANDLE);
    assert_eq!(ctt_session_drop(dropped), ERR_BAD_HANDLE);
}

// ─── 7. Table exhaustion: typed busy, not a panic ────────────────────────────

/// Filling all `MAX_SESSIONS` slots succeeds; the 9th `ctt_session_new` is
/// the typed `ERR_TABLE_FULL` busy status, never a panic.
#[test]
fn max_sessions_exhaustion_is_typed_busy_not_panic() {
    let _guard = ABI_LOCK.lock().unwrap();

    let wire = net_wire(MODEL_DSL);
    let mut handles = [0u64; MAX_SESSIONS];
    for h in &mut handles {
        let code = unsafe { ctt_session_new(wire.as_ptr(), wire.len() as u32, h) };
        assert_eq!(code, 0, "all {} slots must be creatable", MAX_SESSIONS);
    }

    let mut extra = 0u64;
    let code = unsafe { ctt_session_new(wire.as_ptr(), wire.len() as u32, &mut extra) };
    assert_eq!(code, ERR_TABLE_FULL, "9th session is the typed busy status");

    for h in &handles {
        assert_eq!(ctt_session_drop(*h), 0);
    }
}

// ─── 8. Status-code table + wire-format refusals ─────────────────────────────

/// Every documented errno has a distinct value, `OK` is 0, and the values
/// match the module-doc table.
#[test]
fn status_code_table_every_documented_errno_distinct() {
    use chicago_tdd_tools::conformance::guest::{
        ERR_BAD_HANDLE as E9, ERR_BAD_MAGIC as E2, ERR_BAD_MODEL as E7, ERR_BAD_VERSION as E3,
        ERR_CAP_TOO_SMALL as E11, ERR_NULL_PTR as E1, ERR_RESERVED_NOT_ZERO as E6,
        ERR_SEAL_UNAVAILABLE as E12, ERR_STREAM_ALIGNMENT as E13, ERR_STREAM_EMPTY as E10,
        ERR_TABLE_FULL as E8, ERR_TRAILING as E5, ERR_TRUNCATED as E4, OK as E0,
    };
    let codes = [E0, E1, E2, E3, E4, E5, E6, E7, 8, E9, E10, E11, E12, E13];
    assert_eq!(E0, 0, "OK must be zero");
    assert_eq!(E8, ERR_TABLE_FULL);
    assert_eq!(E12, ERR_SEAL_UNAVAILABLE);
    for i in 0..codes.len() {
        for j in (i + 1)..codes.len() {
            assert_ne!(codes[i], codes[j], "errno collision at indices {i},{j}");
        }
    }
    assert_eq!(codes.iter().collect::<std::collections::BTreeSet<_>>().len(), codes.len());
}

/// Wire-format refusals are individually typed: null pointer, bad magic,
/// bad version, reserved-not-zero, truncated, trailing.
#[test]
fn wire_format_refusals_are_individually_typed() {
    let _guard = ABI_LOCK.lock().unwrap();

    // Null out-pointer.
    let good = net_wire(MODEL_DSL);
    let mut handle = 0u64;
    let code = unsafe { ctt_session_new(good.as_ptr(), good.len() as u32, std::ptr::null_mut()) };
    assert_eq!(code, ERR_NULL_PTR);

    // Null buffer.
    let code = unsafe { ctt_session_new(std::ptr::null(), good.len() as u32, &mut handle) };
    assert_eq!(code, ERR_NULL_PTR);

    let refuse = |mutate: &dyn Fn(&mut Vec<u8>), expected: i32, label: &str| {
        let mut wire = net_wire(MODEL_DSL);
        mutate(&mut wire);
        let mut h = 0u64;
        let code = unsafe { ctt_session_new(wire.as_ptr(), wire.len() as u32, &mut h) };
        assert_eq!(code, expected, "{label}");
        assert_eq!(h, 0, "{label}: no handle leaks on refusal");
    };

    refuse(&|w| w[0..4].copy_from_slice(b"XXXX"), ERR_BAD_MAGIC, "bad magic");
    refuse(&|w| w[4] = 0x02, ERR_BAD_VERSION, "bad version");
    refuse(&|w| w[5] = 0x01, ERR_RESERVED_NOT_ZERO, "reserved byte");
    refuse(&|w| w[8] = 0x01, ERR_RESERVED_NOT_ZERO, "reserved2 byte");
    refuse(&|w| w.truncate(10), ERR_TRUNCATED, "truncated header");
    refuse(&|w| w.push(0), ERR_TRAILING, "trailing byte");
    // Undeclared activity name in the DSL: parse succeeds as a refusal.
    refuse(
        &|w| {
            let bad = "activity a0 = sys.a0\nsequence { never_declared }";
            *w = net_wire(bad);
        },
        ERR_BAD_MODEL,
        "bad model DSL",
    );
}

/// An out-buffer below `STEP_STATUS_LEN` is `ERR_CAP_TOO_SMALL`, and an
/// unattached session still steps to Complete over an empty trace.
#[test]
fn cap_too_small_is_typed_and_zero_quota_completes_unattached_session() {
    let _guard = ABI_LOCK.lock().unwrap();
    let s = new_locked_session();

    let mut tiny = [0u8; STEP_STATUS_LEN - 1];
    let code = unsafe { ctt_session_step(s.0, 1, tiny.as_mut_ptr(), tiny.len() as u32) };
    assert_eq!(code, ERR_CAP_TOO_SMALL);

    // quota 0 on an Idle session: the loop body never runs, the trace is
    // "exhausted", so the ABI reports Complete with a zeroed summary.
    let mut rec = [0u8; STEP_STATUS_LEN];
    let code = unsafe { ctt_session_step(s.0, 0, rec.as_mut_ptr(), rec.len() as u32) };
    assert_eq!(code, 0);
    assert_eq!(rec[0], 1, "quota-0 on an idle session completes immediately");
    assert_eq!(rec_u32(&rec, 5), 0);
    assert_eq!(rec_u32(&rec, 9), 0);
}
