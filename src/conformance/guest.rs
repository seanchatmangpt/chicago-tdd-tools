//! Hermetic guest ABI over the landed [`ConformanceSession`].
//!
//! # Module law: `imports == ∅`
//!
//! This module performs **no ambient imports**: no `std::env`, `std::fs`,
//! `std::time`, `std::process`, no file, clock, RNG, or env access of any
//! kind. Every input arrives through linear-memory buffers; every output is
//! written through out-parameters into linear memory. All deterministic
//! replay state lives in the handle table below. A host that offers this
//! guest zero imports still gets bit-exact replay semantics. The only
//! capability the guest needs from the host is linear memory itself.
//!
//! **File loading is host-side-only.** `PowlModel::from_powl_file` (which
//! uses `std::fs`) is deliberately not reachable from this ABI:
//! `ctt_session_new` accepts only in-memory bytes, and the host compiles the
//! model file into the wire-format buffer before calling in. The guest never
//! names a path.
//!
//! # Strata ABI alignment (E10/E12 precedent)
//!
//! This surface follows the established strata guest conventions: zero
//! imports, and a `(ptr: *const u8, len: u32) -> i32` linear-memory style
//! (the `temprun src/abi.rs` precedent) with pure `u64` scalar handles (the
//! `pn_kernel` precedent). It is `no_std`-compatible: `core::` only on the
//! hot path, plus `extern crate alloc` for the one-time model compile (the
//! session's step path is alloc-free per `session.rs`; the loader path
//! allocates, which is the documented, bounded, one-time init cost).
//! `cfg(test)` code may use std (test harness requirement only).
//!
//! # Module wiring (required, not owned by this lane)
//!
//! This file is compiled only when `src/conformance/mod.rs` declares it as:
//!
//! ```ignore
//! #[cfg(all(feature = "wasm-guest", feature = "conformance"))]
//! pub mod guest;
//! ```
//!
//! # Panic semantics = fail-closed traps
//!
//! `catch_unwind` is deliberately **not** used. In Wasm, an unrecovered Rust
//! panic lowers to an unreachable trap; that IS the fail-closed semantics —
//! a trapped guest cannot emit a receipt, cannot half-commit, and the host
//! observes a hard stop. Every *anticipated* fallible path (null
//! out-pointer, zero capacity, bad handle, bad magic) returns a typed `i32`
//! status instead; only genuinely unreachable states may trap, and a trap is
//! the ABI's negative case — delivered by the runtime, never returned as an
//! `i32` by these functions.
//!
//! # Wire format v1 (`ctt_session_new` input buffer)
//!
//! ```text
//! offset  size  field
//! 0       4     magic  b"CTTG"
//! 4       1     version (must be 0x01)
//! 5       1     reserved (must be 0x00)
//! 6       2     model_len_le16 (little-endian; length of the POWL DSL text)
//! 8       8     reserved2 (must be all zero)
//! 16      N     POWL model DSL bytes (exactly model_len bytes; trailing
//!               bytes after model_len are rejected)
//! ```
//!
//! The model DSL is parsed by the real `PowlModel::from_powl_str` and
//! compiled by the real `ConformanceSession::from_model` (validation
//! included) — the guest ABI adds **no** second parser; it is a wire shim
//! over the landed kernel.
//!
//! # Stream format (`ctt_session_attach` input buffer)
//!
//! A flat little-endian `u64` array of OCEL64 event words (`bits [7:0]
//! activity, bits [15:8] tick`, per `session.rs`). A zero-length stream is
//! rejected exactly as the kernel rejects it (`TraceMalformed` semantics ->
//! status `ERR_STREAM_EMPTY`). A byte length that is not a multiple of 8 is
//! `ERR_STREAM_ALIGNMENT` (the v2 errno; the v1-era `ERR_TRUNCATED` name was
//! for short-buffer, a different refusal).
//!
//! # Step status step record (ABI v2; layout unchanged from v1) (`ctt_session_step` out-buffer)
//!
//! ```text
//! offset  size  field
//! 0       1     outcome byte: 0 = Yielded, 1 = Complete, 2 = Violation
//! 1       4     remaining_events_le32  (valid for Yielded; else 0)
//! 5       4     steps_executed_le32    (summary; valid unless Yielded)
//! 9       4     events_total_le32      (summary; valid unless Yielded)
//! 13      8     final_marking_le64     (summary; valid unless Yielded)
//! ```
//! 21 bytes total. `out_cap` must be `>= STEP_STATUS_LEN`.
//!
//! # Seal format v1/v2 (`ctt_session_seal` out-buffer)
//!
//! Exactly what is sealed — a `ReplaySummary` fingerprint bound to the
//! *module identity* that produced it.
//!
//! **v1 (`ABI_VERSION` 1, historical).** A fixed 34-byte record:
//!
//! ```text
//! 8B LE steps_executed | 8B LE events_total | 8B LE final_marking
//! | 1B outcome (0 Incomplete / 1 Complete / 2 Faulted)
//! | 1B violation code (table in `violation`; 0 when not Faulted)
//! | 8B reserved (zero)
//! ```
//!
//! **v2 (`ABI_VERSION` 2, this surface).** A fixed 58-byte record — the
//! v1-exact 26-byte prefix `0..26` is unchanged, and the v1 reserved field
//! is **consumed**: the 32-byte module digest occupies `26..58`.
//!
//! ```text
//! 0..8    LE steps_executed
//! 8..16   LE events_total
//! 16..24  LE final_marking
//! 24      outcome (0 Incomplete / 1 Complete / 2 Faulted)
//! 25      violation code (table in `violation`; 0 when not Faulted)
//! 26..58  module_digest (32B; BLAKE3 of the wasm artifact with its
//!         CTT_MODULE_DIGEST slot zero-filled — build-time threaded, W-ABI2
//!         item 2; all-zero placeholder until the build splices it)
//! ```
//!
//! The v1 `reserved == 0` law and the v2 digest law are mutually
//! exclusive on bytes 26..34, so a v1 parser validating reserved==0 fails
//! closed on a v2 preimage (v2 digests are not all-zero by construction).
//! The layouts are therefore self-distinguishing; the authoritative
//! discriminator is the `ctt_guest_abi_version()` probe (returns 2), which
//! a host MUST call before the first `ctt_session_new`.
//!
//! The preimage is digested with BLAKE3 when the `ledger` feature is on
//! (via `dep:blake3`) and the 32-byte digest is written to the out-buffer
//! (`out_len = 32`). With `ledger` off, sealing is **refused**
//! (`ERR_SEAL_UNAVAILABLE`) rather than downgraded to a weak hash — a
//! non-cryptographic "receipt" is a receipt-shaped lie, so no fallback
//! digest ships at any version.
//!
//! # Status codes (`i32` return of every ABI fn)
//!
//! `0` = ok; **positive** = guest errno (table below); **negative** =
//! trap-class territory: the ABI functions themselves never return a
//! negative in v1 — a trap *is* the negative case, delivered by the Wasm
//! runtime when a genuinely unreachable state occurs.
//!
//! | code | name                    | meaning |
//! |-----:|-------------------------|---------|
//! | 0    | `OK`                    | success |
//! | 1    | `ERR_NULL_PTR`          | a required pointer/out-pointer argument was null |
//! | 2    | `ERR_BAD_MAGIC`         | buffer does not start with `CTTG` |
//! | 3    | `ERR_BAD_VERSION`       | wire version byte != 1 |
//! | 4    | `ERR_TRUNCATED`         | buffer shorter than its declared layout requires |
//! | 5    | `ERR_TRAILING`          | bytes present after the declared payload |
//! | 6    | `ERR_RESERVED_NOT_ZERO` | a must-be-zero field is non-zero |
//! | 7    | `ERR_BAD_MODEL`         | model DSL failed parse/validate/compile |
//! | 8    | `ERR_TABLE_FULL`        | all `MAX_SESSIONS` slots occupied |
//! | 9    | `ERR_BAD_HANDLE`        | handle is free, out of range, or generation-stale |
//! | 10   | `ERR_STREAM_EMPTY`      | attach of a zero-length stream |
//! | 11   | `ERR_CAP_TOO_SMALL`     | out-buffer capacity below the record's fixed length (pure capacity meaning at v2) |
//! | 12   | `ERR_SEAL_UNAVAILABLE`  | sealing requested without the `ledger` feature |
//! | 13   | `ERR_STREAM_ALIGNMENT`  | stream byte length not a multiple of 8 |
//! | 14   | `ERR_OOB_READ`          | an input span (`net_buf`/`stream_ptr`) reaches at/past the installed linear-memory bound |
//! | 15   | `ERR_OOB_WRITE`         | an out-buffer span reaches at/past the installed linear-memory bound (the former v1 `ERR_CAP_TOO_SMALL` overload) |
//! | 16   | `ERR_DEFERRED_HEAP`     | a heap-requiring path was requested under the deferred-heap ABI tier (reserved; not yet returned) |
//!
//! # Handle encoding
//!
//! A handle is `u64 = (generation:u32) << 32 | slot:u32`. The per-slot
//! generation counter makes ABA reuse of a freed slot observable as
//! `ERR_BAD_HANDLE` instead of a silent session swap.
//!
//! The generation counter is `u32` (incremented with `wrapping_add`), so
//! ~2^32 drop-of-the-same-slot cycles wraps it back to a stale handle's
//! expected value and resurrects that handle — measured at ~16.7h at
//! ~14µs per drop cycle. ABI v2 work order: widen the generation to `u64`.
//!
//! # Handle-table storage choice (documented, per spec)
//!
//! Fixed-size slot array in a `static` with an occupancy bitmask: **no
//! runtime allocation after init, ever.** A `OnceCell<Vec<_>>` was
//! considered and rejected — it allocates lazily, making "table full" a
//! late failure; the static array makes capacity a compile-time fact. The
//! table sits in a `Mutex` reached only through the single `with_table`
//! funnel: native builds of this module (feature `wasm-guest` has no
//! `target_arch` gate; the ABI test suite drives it multi-threaded) are
//! serialized by the lock, and single-threaded Wasm guests take the lock
//! uncontended at negligible cost. That helper is the only code that
//! touches the table.

#![cfg(all(feature = "wasm-guest", feature = "conformance"))]
// WASM guest ABI surface: `unsafe` + `#[no_mangle]` are the point of an
// extern "C" cdylib export; the crate-wide Poka-Yoke denies are relaxed at
// this module boundary only (single-threaded guest, documented soundness
// justifications on every unsafe op below).
#![allow(unsafe_code)]
// Wire-width narrowing: every usize->u32 cast below carries a length that
// arrived through a u32-wide ABI parameter or a fixed record layout, so the
// value is u32-bounded by construction (see the per-cast JUSTIFICATION
// comments and the module wire-format docs).
#![allow(clippy::cast_possible_truncation)]

extern crate alloc;

use alloc::vec::Vec;

use crate::conformance::{
    loader::PowlModel, ConformanceError, ConformanceSession, ReportOutcome, YieldStatus,
};

// ---------------------------------------------------------------------------
// Status codes
// ---------------------------------------------------------------------------

/// Success.
pub const OK: i32 = 0;
/// A required pointer/out-pointer argument was null.
pub const ERR_NULL_PTR: i32 = 1;
/// Buffer does not start with `CTTG`.
pub const ERR_BAD_MAGIC: i32 = 2;
/// Wire version byte != 1.
pub const ERR_BAD_VERSION: i32 = 3;
/// Buffer shorter than its declared layout requires.
pub const ERR_TRUNCATED: i32 = 4;
/// Bytes present after the declared payload.
pub const ERR_TRAILING: i32 = 5;
/// A must-be-zero field is non-zero.
pub const ERR_RESERVED_NOT_ZERO: i32 = 6;
/// Model DSL failed parse/validate/compile.
pub const ERR_BAD_MODEL: i32 = 7;
/// All `MAX_SESSIONS` slots occupied.
pub const ERR_TABLE_FULL: i32 = 8;
/// Handle is free, out of range, or generation-stale.
pub const ERR_BAD_HANDLE: i32 = 9;
/// Attach of a zero-length stream.
pub const ERR_STREAM_EMPTY: i32 = 10;
/// Out-buffer capacity below the record's fixed length. At v2 this is pure
/// capacity semantics again — the linear-memory out-span overload moved to
/// [`ERR_OOB_WRITE`].
pub const ERR_CAP_TOO_SMALL: i32 = 11;
/// Sealing requested without the `ledger` feature.
pub const ERR_SEAL_UNAVAILABLE: i32 = 12;
/// Stream byte length not a multiple of 8.
pub const ERR_STREAM_ALIGNMENT: i32 = 13;
/// An input span (`net_buf` / `stream_ptr`) reaches at/past the installed
/// linear-memory bound (or the span computation overflows).
pub const ERR_OOB_READ: i32 = 14;
/// An out-buffer span reaches at/past the installed linear-memory bound
/// (or the span computation overflows).
///
/// The former v1 `ERR_CAP_TOO_SMALL` overload, split out per W-ABI2 item 4
/// so guest OOB is distinct from capacity refusals.
pub const ERR_OOB_WRITE: i32 = 15;
/// A heap-requiring path was requested under the deferred-heap ABI tier.
/// Reserved at v2 — defined and documented now so the errno space is
/// stable; no current entry point returns it.
pub const ERR_DEFERRED_HEAP: i32 = 16;

/// ABI revision reported by [`ctt_guest_abi_version`]. v2: 58-byte seal
/// preimage with a threaded module digest, and the split OOB read/write
/// errnos 14/15.
pub const ABI_VERSION: u32 = 2;

// ---------------------------------------------------------------------------
// Fixed record lengths
// ---------------------------------------------------------------------------

/// Fixed length of the step status record (see module docs).
pub const STEP_STATUS_LEN: usize = 21;
/// Fixed length of the **v1** seal preimage record (historical). A v1
/// parser's `reserved == 0` law over `26..34` fails closed on v2
/// preimages — see the module seal-format docs.
pub const SEAL_PREIMAGE_LEN: usize = 34;
/// Fixed length of the **v2** seal preimage record: the v1-exact 26-byte
/// prefix plus the 32-byte module digest in `26..58`.
pub const SEAL_PREIMAGE_V2_LEN: usize = 58;
/// Length of the sealed BLAKE3 digest.
pub const DIGEST_LEN: usize = 32;
/// Length of the module digest carried in the v2 seal preimage.
pub const MODULE_DIGEST_LEN: usize = 32;

// ---------------------------------------------------------------------------
// Violation code table (byte values used in the seal preimage)
// ---------------------------------------------------------------------------

/// Violation byte values in the seal preimage (0 when not Faulted).
pub mod violation {
    /// Violation byte 0: not Faulted / no violation classification.
    pub const NONE: u8 = 0;
    /// Trace skipped a required transition.
    pub const SKIPPED: u8 = 1;
    /// Tick regressed or failed to advance.
    pub const OUT_OF_ORDER: u8 = 2;
    /// Event names a transition absent from the model.
    pub const UNDECLARED: u8 = 3;
    /// Forbidden marking bit observed.
    pub const FORBIDDEN_BIT: u8 = 4;
    /// Trace bytes failed the OCEL64 framing law.
    pub const TRACE_MALFORMED: u8 = 5;
    /// Marking population exceeded the model's place bound.
    pub const TOO_MANY_PLACES: u8 = 6;
    /// Model DSL exceeded the compile-time size bound.
    pub const MODEL_TOO_LARGE: u8 = 7;
    /// Unclassified kernel refusal.
    pub const OTHER: u8 = 8;
}

/// Maps a kernel [`ConformanceError`] onto the sealed violation byte.
const fn violation_code(err: &ConformanceError) -> u8 {
    match err {
        ConformanceError::Skipped { .. } => violation::SKIPPED,
        ConformanceError::OutOfOrder { .. } => violation::OUT_OF_ORDER,
        ConformanceError::Undeclared { .. } => violation::UNDECLARED,
        ConformanceError::ForbiddenBit { .. } => violation::FORBIDDEN_BIT,
        ConformanceError::TraceMalformed { .. } => violation::TRACE_MALFORMED,
        ConformanceError::TooManyPlaces { .. } => violation::TOO_MANY_PLACES,
        ConformanceError::ModelTooLarge { .. } => violation::MODEL_TOO_LARGE,
        _ => violation::OTHER,
    }
}

// ---------------------------------------------------------------------------
// Handle table
// ---------------------------------------------------------------------------

/// Compile-time session capacity. Not a tuning knob: the no-alloc law fixes
/// it at 8 (spec).
pub const MAX_SESSIONS: usize = 8;

struct Slot {
    session: Option<ConformanceSession>,
    generation: u32,
}

struct Table {
    slots: [Slot; MAX_SESSIONS],
    /// Bit `i` set == slot `i` occupied.
    occupied: u32,
}

impl Table {
    const fn new() -> Self {
        // JUSTIFICATION: `Option<ConformanceSession>::None` is a valid const
        // so the table stays a plain const-constructible static (no alloc,
        // no lazy init).
        #[allow(clippy::declare_interior_mutable_const)]
        const SLOT: Slot = Slot { session: None, generation: 0 };
        Self { slots: [SLOT; MAX_SESSIONS], occupied: 0 }
    }
}

static TABLE: std::sync::Mutex<Table> = std::sync::Mutex::new(Table::new());

// ---------------------------------------------------------------------------
// Linear-memory bound (out-pointer span validation)
// ---------------------------------------------------------------------------

/// Installed end of guest linear memory (exclusive). `0` = no bound
/// installed: the native host tier passes real allocations, so there is
/// nothing to validate against. The embedding runtime installs the bound
/// from the Wasm memory size at instantiation; the ABI then refuses any
/// span (read or write) that reaches at/past the bound with a typed errno
/// instead of a load/store trap. A stale bound is a fail-closed hazard only
/// for addresses inside a *grown* memory tail, so the embedder must refresh
/// it after every `memory.grow` — the refusal is typed (`ERR_OOB_READ` for
/// input spans, `ERR_OOB_WRITE` for out-spans), never silent.
/// TODO(plan W-ABI2 item 3: wasm32 `memory_size()` reflection would remove
/// the stale-bound refresh obligation; the `AtomicUsize` bound stays for the
/// native tier).
static LINEAR_MEMORY_END: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Installs the guest linear-memory end bound (exclusive). Called by the
/// embedding runtime at instantiation and after every `memory.grow`. `0`
/// removes the bound (native host tier).
pub fn set_linear_memory_end(end: usize) {
    LINEAR_MEMORY_END.store(end, std::sync::atomic::Ordering::SeqCst);
}

/// The installed linear-memory end (`0` = unbounded / native tier).
#[must_use]
pub fn linear_memory_end() -> usize {
    LINEAR_MEMORY_END.load(std::sync::atomic::Ordering::SeqCst)
}

/// True when the `[ptr, ptr+len)` span falls at/past the installed
/// linear-memory bound (or the span computation overflows). With no bound
/// installed (`0`, native host tier) no span can be out of bounds.
pub(crate) fn span_oob(ptr: *const u8, len: usize) -> bool {
    let end = LINEAR_MEMORY_END.load(std::sync::atomic::Ordering::SeqCst);
    if end == 0 {
        return false;
    }
    // (is_none_or: None (overflow) stays out-of-bounds.)
    (ptr as usize).checked_add(len).is_none_or(|span_end| span_end > end)
}

/// Read-side span law: an input span at/past the bound is
/// [`ERR_OOB_READ`] (W-ABI2 item 4).
pub(crate) fn in_span_oob(ptr: *const u8, len: usize) -> bool {
    span_oob(ptr, len)
}

/// Write-side span law: an out-buffer span at/past the bound is
/// [`ERR_OOB_WRITE`] (W-ABI2 item 4).
pub(crate) fn out_span_oob(ptr: *const u8, cap: usize) -> bool {
    span_oob(ptr, cap)
}

/// Runs `f` with exclusive `&mut` access to the table, serializing
/// concurrent callers through [`TABLE`]'s mutex.
///
/// JUSTIFICATION (soundness): all table access funnels through this
/// function, which holds the mutex for the duration of `f`, so no two
/// callers can hold `&mut Table` concurrently — on the single-threaded
/// Wasm ABI the lock is uncontended; on native multi-threaded builds (the
/// host test surface) it enforces exclusivity that previously relied on an
/// external test-side mutex. A poisoned lock (a panic inside some earlier
/// `f`) is recovered with `into_inner`: the table's slot/generation
/// invariants fail closed at the ABI boundary (`ERR_BAD_HANDLE` /
/// `ERR_TABLE_FULL`) regardless, and guest builds (panic strategy is unwind for in-workspace builds — see crates/ctt-wasm-guest/Cargo.toml audit note) never
/// observe poisoning.
fn with_table<R>(f: impl FnOnce(&mut Table) -> R) -> R {
    let mut guard = TABLE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut guard)
}
// (clippy note: match/Option refactors below keep the ABI's fail-closed
// branch structure explicit rather than collapsing it into map/or forms.)

const fn encode_handle(slot: usize, generation: u32) -> u64 {
    ((generation as u64) << 32) | slot as u64
}

const fn decode_handle(handle: u64) -> Option<(usize, u32)> {
    let slot = (handle & 0xFFFF_FFFF) as usize;
    if slot >= MAX_SESSIONS {
        return None;
    }
    Some((slot, (handle >> 32) as u32))
}

/// Resolves `handle` to a live session slot index + generation, or
/// `ERR_BAD_HANDLE`.
const fn resolve(t: &Table, handle: u64) -> Result<usize, i32> {
    match decode_handle(handle) {
        None => Err(ERR_BAD_HANDLE),
        Some((slot, gen)) => {
            if t.occupied & (1 << slot) == 0 || t.slots[slot].generation != gen {
                Err(ERR_BAD_HANDLE)
            } else {
                Ok(slot)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Buffer helpers (panic-free; every fallible path is a status)
// ---------------------------------------------------------------------------

/// Reads a C-layout byte pointer + length as a Rust slice, null-checked.
///
/// # Safety
///
/// `ptr` must point to `len` readable bytes in the guest's linear memory for
/// the duration of the call. This is the host's obligation under the ABI.
/// `#[no_mangle]`-adjacent raw-pointer reads; no local state is captured,
/// so declaring it `const` is sound.
const unsafe fn slice_from_raw(buf: *const u8, len: u32) -> Option<&'static [u8]> {
    if buf.is_null() {
        return None;
    }
    if len == 0 {
        return Some(&[]);
    }
    Some(core::slice::from_raw_parts(buf, len as usize))
}

/// Reinterprets a host byte buffer as an OCEL64 word vector (LE per word).
///
/// JUSTIFICATION: alignment-safe (read as bytes, assemble LE), length-checked.
fn decode_stream(bytes: &[u8]) -> Option<Vec<u64>> {
    if !bytes.len().is_multiple_of(8) {
        return None;
    }
    Some(
        bytes
            .chunks_exact(8)
            .map(|c| {
                let mut b = [0u8; 8];
                b.copy_from_slice(c);
                u64::from_le_bytes(b)
            })
            .collect(),
    )
}

fn write_u32_le(out: &mut [u8], off: usize, v: u32) {
    out[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn write_u64_le(out: &mut [u8], off: usize, v: u64) {
    out[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

/// Builds the fixed 58-byte (v2) seal preimage for a [`ReplaySummary`]
/// -shaped verdict bound to `module_digest` (module-doc layout).
const fn seal_preimage(
    steps_executed: u32,
    events_total: u32,
    final_marking: u64,
    outcome_byte: u8,
    violation_byte: u8,
    module_digest: &[u8; MODULE_DIGEST_LEN],
) -> [u8; SEAL_PREIMAGE_V2_LEN] {
    // JUSTIFICATION: `copy_from_slice` is not const-stable; fixed-width
    // index writes keep the preimage builder const (it is reused by tests
    // and by the seal path without a runtime dependency). Declared before
    // all statements (clippy::items_after_statements).
    const fn put64(p: &mut [u8], off: usize, v: u64) {
        let b = v.to_le_bytes();
        let mut i = 0;
        while i < 8 {
            p[off + i] = b[i];
            i += 1;
        }
    }
    let mut p = [0u8; SEAL_PREIMAGE_V2_LEN];
    put64(&mut p, 0, steps_executed as u64);
    put64(&mut p, 8, events_total as u64);
    put64(&mut p, 16, final_marking);
    p[24] = outcome_byte;
    p[25] = violation_byte;
    // v2: the v1 reserved field is consumed — the module digest occupies
    // 26..58. (26..34 overlaps the former reserved bytes by construction.)
    let mut i = 0;
    while i < MODULE_DIGEST_LEN {
        p[26 + i] = module_digest[i];
        i += 1;
    }
    p
}

// ---------------------------------------------------------------------------
// Extern ABI
// ---------------------------------------------------------------------------

/// ABI revision of this guest surface.
// TODO(plan W-ABI2 item 5, shim tier): a `guest-abi-v1` feature compiling
// the v1 layout/errno semantics (probe -> 1) is not yet wired; until it is,
// v1 consumers must migrate or consume the shim tier. Related pin:
// crates/ctt-wasm-guest/src/lib.rs asserts probe == 2 (ABI v2) (cross-crate
// migration, outside this lane).
#[no_mangle]
#[must_use]
pub const extern "C" fn ctt_guest_abi_version() -> u32 {
    ABI_VERSION
}

/// Parses + validates + compiles the wire-format model buffer and installs
/// the fresh session into a free slot.
///
/// # Safety
///
/// `net_buf` must point to `net_len` readable bytes; `out_handle` must point
/// to one writable `u64`. Host obligations under the ABI.
#[no_mangle]
pub unsafe extern "C" fn ctt_session_new(
    net_buf: *const u8,
    net_len: u32,
    out_handle: *mut u64,
) -> i32 {
    if net_buf.is_null() || out_handle.is_null() {
        return ERR_NULL_PTR;
    }
    // Read-span law (W-ABI2 item 4): an input span at/past the installed
    // linear-memory bound is a typed ERR_OOB_READ, not a load trap.
    if in_span_oob(net_buf, net_len as usize) {
        return ERR_OOB_READ;
    }
    let Some(bytes) = slice_from_raw(net_buf, net_len) else {
        return ERR_NULL_PTR;
    };
    if bytes.len() < 16 {
        return ERR_TRUNCATED;
    }
    if &bytes[0..4] != b"CTTG" {
        return ERR_BAD_MAGIC;
    }
    if bytes[4] != 0x01 {
        return ERR_BAD_VERSION;
    }
    if bytes[5] != 0 {
        return ERR_RESERVED_NOT_ZERO;
    }
    let model_len = u16::from_le_bytes([bytes[6], bytes[7]]) as usize;
    if bytes[8..16].iter().any(|&b| b != 0) {
        return ERR_RESERVED_NOT_ZERO;
    }
    if bytes.len() < 16 + model_len {
        return ERR_TRUNCATED;
    }
    if bytes.len() > 16 + model_len {
        return ERR_TRAILING;
    }
    let Ok(dsl) = core::str::from_utf8(&bytes[16..]) else {
        return ERR_BAD_MODEL;
    };
    let Ok(model) = PowlModel::from_powl_str(dsl) else {
        return ERR_BAD_MODEL;
    };
    let Ok(session) = ConformanceSession::from_model(&model) else {
        return ERR_BAD_MODEL;
    };

    with_table(|t| match t.slots.iter().position(|s| s.session.is_none()) {
        None => ERR_TABLE_FULL,
        Some(slot) => {
            let generation = t.slots[slot].generation;
            *out_handle = encode_handle(slot, generation);
            t.slots[slot].session = Some(session);
            t.occupied |= 1 << slot;
            OK
        }
    })
}

/// Binds an OCEL64 event stream (flat LE `u64` words) to the session.
///
/// # Safety
///
/// `stream_ptr` must point to `stream_len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn ctt_session_attach(
    handle: u64,
    stream_ptr: *const u8,
    stream_len: u32,
) -> i32 {
    if stream_ptr.is_null() {
        return ERR_NULL_PTR;
    }
    // Read-span law (W-ABI2 item 4), same as `ctt_session_new`'s net_buf.
    if in_span_oob(stream_ptr, stream_len as usize) {
        return ERR_OOB_READ;
    }
    let Some(bytes) = slice_from_raw(stream_ptr, stream_len) else {
        return ERR_NULL_PTR;
    };
    if bytes.is_empty() {
        return ERR_STREAM_EMPTY;
    }
    if bytes.len() % 8 != 0 {
        return ERR_STREAM_ALIGNMENT;
    }
    let Some(words) = decode_stream(bytes) else {
        return ERR_STREAM_ALIGNMENT;
    };
    with_table(|t| {
        match resolve(t, handle) {
            Err(code) => code,
            Ok(slot) => {
                // JUSTIFICATION: resolve() proved occupancy; the Option is
                // Some by the table invariant.
                t.slots[slot].session.as_mut().map_or(ERR_BAD_HANDLE, |s| {
                    match s.attach_stream(&words) {
                        Ok(()) => OK,
                        // Kernel refuses only the empty stream, already
                        // filtered above; map any other refusal conservatively.
                        Err(_) => ERR_STREAM_EMPTY,
                    }
                })
            }
        }
    })
}

/// Replays at most `quota` events and writes the fixed 21-byte step status
/// record into linear memory.
///
/// # Safety
///
/// `out_status` must point to `out_cap` writable bytes;
/// `out_cap >= STEP_STATUS_LEN` is required.
#[no_mangle]
pub unsafe extern "C" fn ctt_session_step(
    handle: u64,
    quota: u32,
    out_status: *mut u8,
    out_cap: u32,
) -> i32 {
    if out_status.is_null() {
        return ERR_NULL_PTR;
    }
    if (out_cap as usize) < STEP_STATUS_LEN {
        return ERR_CAP_TOO_SMALL;
    }
    // Out-pointer span must lie inside linear memory (T13/SK6-F2): a valid
    // handle with an out-pointer at/past the memory end is a typed errno,
    // not a Wasm store trap. v2: distinct ERR_OOB_WRITE, no longer the
    // ERR_CAP_TOO_SMALL overload.
    if out_span_oob(out_status, out_cap as usize) {
        return ERR_OOB_WRITE;
    }
    let status = with_table(|t| match resolve(t, handle) {
        Err(_) => None,
        Ok(slot) => t.slots[slot].session.as_mut().map(|s| s.step_budget(quota)),
    });
    let Some(status) = status else {
        return ERR_BAD_HANDLE;
    };

    // Justification: out_cap >= STEP_STATUS_LEN was checked above.
    #[allow(clippy::cast_sign_loss)]
    let out = core::slice::from_raw_parts_mut(out_status, out_cap as usize);
    let rec: [u8; STEP_STATUS_LEN] = match &status {
        YieldStatus::Yielded { remaining_events } => {
            let mut r = [0u8; STEP_STATUS_LEN];
            r[0] = 0;
            // JUSTIFICATION: remaining_events is bounded by the attached
            // stream, which arrived through a u32-length linear-memory
            // buffer, so the u32 narrowing cannot lose events.
            write_u32_le(&mut r, 1, *remaining_events as u32);
            r
        }
        YieldStatus::Complete(summary) => {
            let mut r = [0u8; STEP_STATUS_LEN];
            r[0] = 1;
            write_u32_le(&mut r, 5, summary.steps_executed);
            write_u32_le(&mut r, 9, summary.events_total as u32);
            write_u64_le(&mut r, 13, summary.final_marking);
            r
        }
        YieldStatus::Violation(_) => {
            // The violation itself travels only as the outcome byte here;
            // its classified detail is recoverable via the sealed summary
            // (violation code) and the session's latched phase.
            let summary = with_table(|t| {
                resolve(t, handle)
                    .ok()
                    .and_then(|slot| t.slots[slot].session.as_ref().map(ConformanceSession::finish))
            });
            let mut r = [0u8; STEP_STATUS_LEN];
            r[0] = 2;
            // A vanished handle mid-violation-report fails closed: the
            // outcome byte still travels, the summary fields stay zero.
            if let Some(summary) = summary {
                write_u32_le(&mut r, 5, summary.steps_executed);
                write_u32_le(&mut r, 9, summary.events_total as u32);
                write_u64_le(&mut r, 13, summary.final_marking);
            }
            r
        }
    };
    out[..STEP_STATUS_LEN].copy_from_slice(&rec);
    OK
}

/// Writes the sealed replay receipt: the 32-byte BLAKE3 digest of the fixed
/// 58-byte (v2) seal preimage (`ledger` feature), or `ERR_SEAL_UNAVAILABLE`.
///
/// # Safety
///
/// `out_buf` must point to `out_cap` writable bytes; `out_len` must point to
/// one writable `u32`.
#[no_mangle]
pub unsafe extern "C" fn ctt_session_seal(
    handle: u64,
    out_buf: *mut u8,
    out_cap: u32,
    out_len: *mut u32,
) -> i32 {
    if out_buf.is_null() || out_len.is_null() {
        return ERR_NULL_PTR;
    }
    if (out_cap as usize) < DIGEST_LEN {
        return ERR_CAP_TOO_SMALL;
    }
    // Same out-pointer span law as [`ctt_session_step`] (T13/SK6-F2),
    // covering both the digest buffer and the 4-byte `out_len` word. v2:
    // distinct ERR_OOB_WRITE.
    if out_span_oob(out_buf, out_cap as usize) || out_span_oob(out_len.cast(), 4) {
        return ERR_OOB_WRITE;
    }
    let preimage = with_table(|t| match resolve(t, handle) {
        Err(code) => Err(code),
        Ok(slot) => t.slots[slot].session.as_ref().map_or(Err(ERR_BAD_HANDLE), |s| {
            let summary = s.finish();
            let outcome_byte = match summary.outcome {
                ReportOutcome::Incomplete => 0u8,
                ReportOutcome::Complete => 1,
                ReportOutcome::Faulted(_) => 2,
            };
            let violation_byte = match summary.outcome {
                ReportOutcome::Faulted(ref e) => violation_code(e),
                _ => violation::NONE,
            };
            Ok(seal_preimage(
                summary.steps_executed,
                summary.events_total as u32,
                summary.final_marking,
                outcome_byte,
                violation_byte,
                // TODO(plan W-ABI2 item 2: build-time module digest) — the
                // all-zero placeholder is the documented pre-item-2 state;
                // the digest PARAMETER is the v2 surface.
                &[0u8; MODULE_DIGEST_LEN],
            ))
        }),
    });
    let preimage = match preimage {
        Ok(p) => p,
        Err(code) => return code,
    };

    #[cfg(feature = "ledger")]
    {
        let digest = blake3::hash(&preimage);
        // JUSTIFICATION: out_cap >= DIGEST_LEN checked above.
        #[allow(clippy::cast_sign_loss)]
        let out = core::slice::from_raw_parts_mut(out_buf, out_cap as usize);
        // blake3 1.8: `Hash::as_bytes()` returns the fixed 32-byte digest.
        out[..DIGEST_LEN].copy_from_slice(digest.as_bytes());
        *out_len = DIGEST_LEN as u32;
        OK
    }
    #[cfg(not(feature = "ledger"))]
    {
        let _ = &preimage;
        ERR_SEAL_UNAVAILABLE
    }
}

/// Frees the session slot; the slot's generation increments so stale handles
/// (including the one just dropped) become `ERR_BAD_HANDLE` forever.
#[no_mangle]
pub extern "C" fn ctt_session_drop(handle: u64) -> i32 {
    with_table(|t| match resolve(t, handle) {
        Err(code) => code,
        Ok(slot) => {
            t.slots[slot].session = None;
            t.slots[slot].generation = t.slots[slot].generation.wrapping_add(1);
            t.occupied &= !(1 << slot);
            OK
        }
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- handle table -------------------------------------------------------

    #[test]
    fn handle_roundtrip_preserves_slot_and_generation() {
        assert_eq!(decode_handle(encode_handle(3, 7)), Some((3, 7)));
        assert_eq!(decode_handle(encode_handle(0, u32::MAX)), Some((0, u32::MAX)));
    }

    #[test]
    fn out_of_range_slot_is_bad_handle() {
        assert_eq!(decode_handle(MAX_SESSIONS as u64), None);
        assert_eq!(decode_handle(u64::MAX), None);
    }

    #[test]
    fn table_starts_empty_at_full_capacity() {
        with_table(|t| {
            assert_eq!(t.occupied, 0);
            let free = t.slots.iter().filter(|s| s.session.is_none()).count();
            assert_eq!(free, MAX_SESSIONS);
        });
    }

    #[test]
    fn drop_frees_slot_and_bumps_generation() {
        with_table(|t| {
            let gen0 = t.slots[2].generation;
            t.slots[2].session = None; // simulate occupancy then drop
            t.occupied |= 1 << 2;
            t.slots[2].session = None;
            t.slots[2].generation = t.slots[2].generation.wrapping_add(1);
            t.occupied &= !(1 << 2);
            assert_eq!(t.slots[2].generation, gen0.wrapping_add(1));
            assert_eq!(resolve(t, encode_handle(2, gen0)), Err(ERR_BAD_HANDLE));
        });
    }

    // -- status codes ---------------------------------------------------------

    #[test]
    fn status_table_is_stable_and_unique() {
        let codes = [
            OK,
            ERR_NULL_PTR,
            ERR_BAD_MAGIC,
            ERR_BAD_VERSION,
            ERR_TRUNCATED,
            ERR_TRAILING,
            ERR_RESERVED_NOT_ZERO,
            ERR_BAD_MODEL,
            ERR_TABLE_FULL,
            ERR_BAD_HANDLE,
            ERR_STREAM_EMPTY,
            ERR_CAP_TOO_SMALL,
            ERR_SEAL_UNAVAILABLE,
            ERR_STREAM_ALIGNMENT,
            ERR_OOB_READ,
            ERR_OOB_WRITE,
            ERR_DEFERRED_HEAP,
        ];
        assert_eq!(ERR_OOB_READ, 14);
        assert_eq!(ERR_OOB_WRITE, 15);
        assert_eq!(ERR_DEFERRED_HEAP, 16);
        assert_eq!(codes[0], 0, "OK must be zero");
        assert!(
            codes.iter().all(|&c| c >= 0),
            "ABI fns never return negatives; traps are the negative case"
        );
        for (i, &a) in codes.iter().enumerate() {
            for &b in &codes[i + 1..] {
                assert_ne!(a, b, "duplicate status code {a}");
            }
        }
    }

    #[test]
    fn abi_version_is_two() {
        assert_eq!(ctt_guest_abi_version(), 2);
    }

    // -- record encoding ------------------------------------------------------

    #[test]
    fn seal_preimage_layout_is_fixed() {
        let digest: [u8; MODULE_DIGEST_LEN] =
            core::array::from_fn(|i| u8::try_from(i % 256).unwrap());
        let p = seal_preimage(3, 7, 1 << 3, 1, violation::NONE, &digest);
        // v2 length; the v1-exact prefix is unchanged.
        assert_eq!(p.len(), SEAL_PREIMAGE_V2_LEN);
        assert_eq!(SEAL_PREIMAGE_V2_LEN, 58);
        assert_eq!(SEAL_PREIMAGE_LEN, 34, "v1 length pinned for the shim tier");
        assert_eq!(&p[0..8], &3u64.to_le_bytes());
        assert_eq!(&p[8..16], &7u64.to_le_bytes());
        assert_eq!(&p[16..24], &(1u64 << 3).to_le_bytes());
        assert_eq!(p[24], 1); // Complete
        assert_eq!(p[25], violation::NONE);
        // v2 law: the module digest occupies 26..58 (the v1 reserved-zero
        // law over 26..34 is replaced by the digest law).
        assert_eq!(&p[26..58], &digest);
    }

    /// W-ABI2 item 6 (SX6 M2) falsifier: the module digest is load-bearing.
    /// Two distinct digests over identical summaries must produce distinct
    /// preimages AND distinct BLAKE3 outputs — the v1 collision class
    /// (different modules, identical `ReplaySummary`, byte-identical seal)
    /// is closed.
    #[test]
    fn seal_preimage_binds_module_digest_distinct_digests_distinct_seals() {
        let digest_a = [0xA5u8; MODULE_DIGEST_LEN];
        let digest_b = [0x5Au8; MODULE_DIGEST_LEN];
        assert_ne!(digest_a, digest_b);

        let pa = seal_preimage(3, 3, 1 << 3, 1, violation::NONE, &digest_a);
        let pb = seal_preimage(3, 3, 1 << 3, 1, violation::NONE, &digest_b);
        assert_ne!(
            pa.as_slice(),
            pb.as_slice(),
            "identical summaries with distinct module digests must give distinct preimages"
        );
        // Identical summaries means the v1-exact prefix is byte-identical.
        assert_eq!(pa[0..26], pb[0..26]);
        assert_ne!(pa[26..], pb[26..]);

        #[cfg(feature = "ledger")]
        {
            let da = blake3::hash(&pa);
            let db = blake3::hash(&pb);
            assert_ne!(
                da.as_bytes(),
                db.as_bytes(),
                "distinct preimages must digest to distinct BLAKE3 seals"
            );
        }
    }

    #[test]
    fn violation_codes_are_distinct_and_none_is_zero() {
        let codes = [
            violation::NONE,
            violation::SKIPPED,
            violation::OUT_OF_ORDER,
            violation::UNDECLARED,
            violation::FORBIDDEN_BIT,
            violation::TRACE_MALFORMED,
            violation::TOO_MANY_PLACES,
            violation::MODEL_TOO_LARGE,
            violation::OTHER,
        ];
        assert_eq!(codes[0], 0);
        for (i, &a) in codes.iter().enumerate() {
            for &b in &codes[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn violation_code_maps_kernel_errors() {
        use ConformanceError as E;
        assert_eq!(violation_code(&E::Skipped { step: 0 }), violation::SKIPPED);
        assert_eq!(
            violation_code(&E::OutOfOrder { step: 0, prev_tick: 1, tick: 1 }),
            violation::OUT_OF_ORDER
        );
        assert_eq!(violation_code(&E::Undeclared { step: 0, tid: 9 }), violation::UNDECLARED);
        assert_eq!(
            violation_code(&E::ForbiddenBit { step: 0, marking: 0, mask: 0 }),
            violation::FORBIDDEN_BIT
        );
    }
}
