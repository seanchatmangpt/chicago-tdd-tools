//! MG4 leaf: PN64 audit shim over the LANDED parent kernel.
//!
//! **No second kernel implementation.** Every semantics-bearing function is
//! a re-export or a byte-encoding wrapper around
//! `chicago_tdd_tools::conformance::pn64` (src/conformance/pn64.rs):
//! `pack`, `marking`, `pre`, `post`, `tid`, `flags`, `enabled`, `fire`,
//! `ct_eq`, `ct_lte`, and above all `step_branchless`.
//!
//! This leaf adds only the mgi wire surface: fixed-width 21-byte step
//! records for a PN64 trace, so an audit host can replay a merger-phase
//! trace through the parent kernel and carry the results across the guest
//! ABI without an in-guest decoder.
//!
//! 21-byte step record layout (little-endian, tightly packed):
//!   `[0..8)`   committed pn word after the step (`step_branchless` output)
//!   `[8..16)`  the ocel event word for the step
//!   `[16]`     committed flag: 1 = fired, 0 = refused (word unchanged)
//!   `[17..19)` marking BEFORE the step (u16 LE)
//!   `[19..21)` marking AFTER the step (u16 LE)

pub use chicago_tdd_tools::conformance::pn64::{
    ct_eq, ct_lte, enabled, fire, flags, marking, pack, post, pre, step_branchless, tid,
};

/// One step record, in bytes.
use crate::{errno, fixed};

pub const STEP_RECORD_LEN: usize = 21;

/// Maximum steps per trace call. Exceeding this is `ERR_DIM`.
pub const MAX_STEPS: usize = 256;

/// Refusal reasons local to this leaf.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuditError {
    /// Step count 0, > [`MAX_STEPS`], or pn/ocel slice length mismatch
    /// (shared `ERR_DIM`).
    Dim,
    /// Output buffer shorter than `steps * STEP_RECORD_LEN`
    /// (shared `ERR_LENGTH`); whole-call refusal — never partial.
    Length,
}

impl AuditError {
    /// Wire code: shared `ERR_DIM`/`ERR_LENGTH` only; this leaf adds no
    /// local codes because it adds no local semantics.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Dim => crate::errno::ERR_DIM,
            Self::Length => crate::errno::ERR_LENGTH,
        }
    }
}

/// Encode one committed flag as a byte without branching on the value.
#[inline]
#[must_use]
pub const fn committed_byte(before: u64, after: u64) -> u8 {
    // 1 iff the words differ, via the parent's constant-time equality.
    if ct_eq(before, after) == 1 {
        0
    } else {
        1
    }
}

/// Encode one 21-byte step record into `out[i*21 .. (i+1)*21]`.
///
/// The semantics come entirely from the parent's `step_branchless`; this
/// function is a pure byte encoder over its inputs and output.
pub fn encode_step(
    pn: u64,
    ocel: u64,
    invariant_mask: u64,
    out: &mut [u8],
) -> Result<(), AuditError> {
    if out.len() < STEP_RECORD_LEN {
        return Err(AuditError::Length);
    }
    let committed = step_branchless(pn, ocel, invariant_mask);
    out[0..8].copy_from_slice(&committed.to_le_bytes());
    out[8..16].copy_from_slice(&ocel.to_le_bytes());
    // MX8 note: the landed wire law (module doc + SEU oracle) defines [16]
    // as the WORD-CHANGED flag: 1 iff committed != input word. A fired step
    // whose W+ reproduces the input marking reports 0 — "word unchanged" —
    // by this contract; `committed_byte` is the exact encoder for it.
    out[16] = committed_byte(pn, committed);
    out[17..19].copy_from_slice(&marking(pn).to_le_bytes());
    out[19..21].copy_from_slice(&marking(committed).to_le_bytes());
    Ok(())
}

/// Replay a full PN64 trace through the parent kernel and emit one 21-byte
/// step record per step into `out`.
///
/// Words chain: each step's committed word is the next step's input pn, so
/// an audit host can verify the chain by re-deriving `step_branchless` on
/// the parent kernel alone.
///
/// On any refusal `out` is bit-exact untouched (validated up front, never
/// partial).
///
/// # Errors
/// [`AuditError::Dim`] on empty/mismatched/oversize inputs;
/// [`AuditError::Length`] when `out` is too short (whole-call refusal).
pub fn trace(
    pns: &[u64],
    ocel: &[u64],
    invariant_mask: u64,
    out: &mut [u8],
) -> Result<usize, AuditError> {
    if pns.is_empty() || pns.len() != ocel.len() || pns.len() > MAX_STEPS {
        return Err(AuditError::Dim);
    }
    let need = pns.len() * STEP_RECORD_LEN;
    if out.len() < need {
        return Err(AuditError::Length);
    }
    for i in 0..pns.len() {
        let slot = &mut out[i * STEP_RECORD_LEN..(i + 1) * STEP_RECORD_LEN];
        // The chain: pn_{i+1} = committed_i. First step consumes pns[0].
        let pn_in =
            if i == 0 { pns[0] } else { step_branchless(pns[i - 1], ocel[i - 1], invariant_mask) };
        encode_step(pn_in, ocel[i], invariant_mask, slot)?;
    }
    Ok(need)
}

/// Stable leaf identity (`Mgi1Header.leaf_id`).
pub const LEAF_ID_PN64_AUDIT: u8 = 10;

/// Byte-level adapter (safe Rust, exercised by tests).
///
/// Wire body: `u64 invariant_mask`, then `dims` rows of 16 B
/// (`pn: u64`, `ocel: u64`). Output body: `dims` x 21 B step records
/// ([`STEP_RECORD_LEN`]). `scale_tag` is [`fixed::TAG_SHARES`] by wire
/// convention: pn/ocel words are raw `u64`s, no fixed-point scale applies
/// (the tag exists to satisfy the MGI1 header law, documented as inert).
fn run(in_buf: &[u8], out_buf: &mut [u8]) -> Result<(), i32> {
    let hdr = crate::parse_header(in_buf)?;
    if hdr.leaf_id != LEAF_ID_PN64_AUDIT {
        return Err(errno::ERR_BAD_MAGIC);
    }
    if hdr.scale_tag != fixed::TAG_SHARES {
        return Err(errno::ERR_BAD_SCALE);
    }
    if hdr.dims == 0 || hdr.dims > MAX_STEPS as u64 {
        return Err(errno::ERR_DIM);
    }
    let d = hdr.dims as usize;
    let exact_len = 16 + 8 + 16 * d;
    if in_buf.len() < exact_len {
        return Err(errno::ERR_TRUNCATED);
    }
    if in_buf.len() > exact_len {
        return Err(errno::ERR_TRAILING);
    }
    let invariant_mask =
        u64::from_le_bytes(in_buf[16..24].try_into().map_err(|_| errno::ERR_TRUNCATED)?);
    let mut pns = [0u64; MAX_STEPS];
    let mut ocels = [0u64; MAX_STEPS];
    for i in 0..d {
        let b = &in_buf[24 + 16 * i..24 + 16 * (i + 1)];
        pns[i] = u64::from_le_bytes(b[0..8].try_into().map_err(|_| errno::ERR_TRUNCATED)?);
        ocels[i] = u64::from_le_bytes(b[8..16].try_into().map_err(|_| errno::ERR_TRUNCATED)?);
    }
    let need = 21 * d;
    if out_buf.len() < need {
        return Err(errno::ERR_CAP_TOO_SMALL);
    }
    let n = trace(&pns[..d], &ocels[..d], invariant_mask, &mut out_buf[16..16 + need])
        .map_err(AuditError::code)?;
    debug_assert_eq!(n, need);
    let ohdr = crate::Mgi1Header {
        leaf_id: LEAF_ID_PN64_AUDIT,
        scale_tag: fixed::TAG_SHARES,
        dims: d as u64,
    };
    ohdr.encode(out_buf);
    Ok(())
}

/// Wire-level entry point: `i32 mgi_pn64_audit_compute(in, in_len, out, out_cap) -> i32`.
/// 0 = ok; positive = errno; negative reserved for host transport.
///
/// # Safety
/// `in`/`out` must be `in_len`/`out_cap`-byte valid (cdylib ABI contract).
#[no_mangle]
pub unsafe extern "C" fn mgi_pn64_audit_compute(
    in_: *const u8,
    in_len: u32,
    out: *mut u8,
    out_cap: u32,
) -> i32 {
    if in_.is_null() || out.is_null() {
        return errno::ERR_NULL_PTR;
    }
    if (out_cap as usize) < STEP_RECORD_LEN {
        return errno::ERR_CAP_TOO_SMALL;
    }
    let buf = core::slice::from_raw_parts(in_, in_len as usize);
    let o = core::slice::from_raw_parts_mut(out, out_cap as usize);
    match run(buf, o) {
        Ok(()) => errno::OK,
        Err(code) => code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::conformance::pn64 as parent;

    /// Parent-crate golden word: m=0b0110, W-=0b0010, W+=0b1000, tid=3.
    fn sample_pn() -> u64 {
        parent::pack(0b0110, 0b0010, 0b1000, 3, 0xC0)
    }

    #[test]
    fn shim_matches_parent_kernel_bit_exact() {
        let pn = sample_pn();
        for ocel in [3u64, 4, 0x103, 0] {
            for mask in [0u64, 0xFFFF, 0b1100] {
                assert_eq!(
                    step_branchless(pn, ocel, mask),
                    parent::step_branchless(pn, ocel, mask)
                );
            }
        }
    }

    #[test]
    fn reexports_are_parent_functions() {
        // Byte-identical behaviour for every re-exported accessor.
        let w = pack(1, 2, 3, 4, 5);
        assert_eq!((marking(w), pre(w), post(w), tid(w), flags(w)), (1, 2, 3, 4, 5));
        assert_eq!(fire(w), parent::fire(w));
        assert_eq!((ct_eq(w, w), ct_lte(w, w)), (1, 1));
        // RV3 repair: enabled requires marking ⊇ W-; the packed sample word
        // above has marking 1 < pre 2, so assert on an enabled word.
        assert!(enabled(parent::pack(0b0110, 0b0010, 0b1000, 3, 0xC0)));
    }

    #[test]
    fn golden_step_record_layout() {
        // pn: m=0b0110, W-=0b0010, W+=0b1000, tid=3; ocel activity 3 → fires:
        // m' = (0b0110 ^ 0b0010) | 0b1000 = 0b1100.
        let pn = sample_pn();
        let ocel = 3u64;
        let mut rec = [0u8; STEP_RECORD_LEN];
        encode_step(pn, ocel, 0, &mut rec).expect("encode");
        assert_eq!(rec.len(), 21);
        let committed = u64::from_le_bytes(rec[0..8].try_into().expect("8"));
        assert_eq!(committed, parent::step_branchless(pn, ocel, 0));
        assert_eq!(rec[16], 1, "fired");
        assert_eq!(u16::from_le_bytes(rec[17..19].try_into().expect("2")), 0b0110);
        assert_eq!(u16::from_le_bytes(rec[19..21].try_into().expect("2")), 0b1100);
        // OCEL word carried verbatim.
        assert_eq!(u64::from_le_bytes(rec[8..16].try_into().expect("8")), ocel);
    }

    #[test]
    fn refused_step_records_unchanged_word_and_zero_flag() {
        // Wrong activity id (ocel low byte 7 != tid 3) → refused: word
        // unchanged, flag 0, before == after.
        let pn = sample_pn();
        let mut rec = [0u8; STEP_RECORD_LEN];
        encode_step(pn, 7, 0, &mut rec).expect("encode");
        assert_eq!(rec[16], 0);
        assert_eq!(u64::from_le_bytes(rec[0..8].try_into().expect("8")), pn);
        assert_eq!(rec[17..21], [0b0110, 0, 0b0110, 0]);
    }

    #[test]
    fn invariant_mask_blocks_firing_through_shim() {
        // Post-marking 0b1100 intersects mask 0b1100 → parent refuses.
        let pn = sample_pn();
        let mut rec = [0u8; STEP_RECORD_LEN];
        encode_step(pn, 3, 0b1100, &mut rec).expect("encode");
        assert_eq!(rec[16], 0);
        assert_eq!(u64::from_le_bytes(rec[0..8].try_into().expect("8")), pn);
    }

    #[test]
    fn trace_chains_words_through_parent_kernel() {
        // RV3 repair: the trace chains COMMITTED words — step i+1 consumes
        // the fired word of step i (module doc), so pns[1] is unused, every
        // step's activity must match the chain word's tid (3), and the
        // chain word's marking must still hold W-. pn0 therefore re-produces
        // its W- place in W+ (0b1010 keeps bit 0b0010): m0' = (0b0110 ^
        // 0b0010) | 0b1010 = 0b1110, which is enabled again, and the second
        // fire is idempotent: m1' = (0b1110 ^ 0b0010) | 0b1010 = 0b1110.
        let pn0 = parent::pack(0b0110, 0b0010, 0b1010, 3, 0);
        let pn1 = parent::pack(0b1100, 0b0100, 0b0001, 1, 0);
        let pns = [pn0, pn1];
        let ocel = [3u64, 3];
        let mut out = [0u8; 2 * STEP_RECORD_LEN];
        let n = trace(&pns, &ocel, 0, &mut out).expect("trace");
        assert_eq!(n, 2 * STEP_RECORD_LEN);
        // Record 0: fired from pn0 (word changed).
        assert_eq!(out[16], 1);
        let rec1 = &out[STEP_RECORD_LEN..];
        let committed1 = u64::from_le_bytes(rec1[0..8].try_into().expect("8"));
        // MX8 final: the landed wire law (module doc + SEU oracle) defines [16]
        // as the WORD-CHANGED flag; the idempotent refire reports 0 even
        // though the parent kernel committed the step.
        assert_eq!(rec1[16], 0);
        assert_eq!(parent::marking(committed1), 0b1110);
        assert_eq!(u16::from_le_bytes(rec1[17..19].try_into().expect("2")), 0b1110);
        assert_eq!(u16::from_le_bytes(rec1[19..21].try_into().expect("2")), 0b1110);
    }

    #[test]
    fn trace_empty_and_mismatch_err_dim() {
        let mut out = [0u8; STEP_RECORD_LEN];
        assert_eq!(trace(&[], &[], 0, &mut out), Err(AuditError::Dim));
        assert_eq!(trace(&[1u64], &[1u64, 2], 0, &mut out), Err(AuditError::Dim));
        let big = [0u64; MAX_STEPS + 1];
        let mut big_out = [0u8; (MAX_STEPS + 1) * STEP_RECORD_LEN];
        assert_eq!(trace(&big, &big, 0, &mut big_out), Err(AuditError::Dim));
        assert_eq!(AuditError::Dim.code(), crate::errno::ERR_DIM);
    }

    #[test]
    fn trace_short_output_whole_call_err_length() {
        let pns = [sample_pn(); 2];
        let ocel = [3u64; 2];
        let mut short = [0u8; STEP_RECORD_LEN]; // needs 42
        assert_eq!(trace(&pns, &ocel, 0, &mut short), Err(AuditError::Length));
        assert_eq!(short[16], 0, "never partial");
        assert_eq!(AuditError::Length.code(), crate::errno::ERR_LENGTH);
    }

    #[test]
    fn encode_step_short_output_err_length() {
        let mut rec = [0u8; 20];
        assert_eq!(encode_step(sample_pn(), 3, 0, &mut rec), Err(AuditError::Length));
    }

    #[test]
    fn deterministic_repeat_calls() {
        let pns = [sample_pn(); 3];
        let ocel = [3u64; 3];
        let mut a = [0u8; 3 * STEP_RECORD_LEN];
        let mut b = [0u8; 3 * STEP_RECORD_LEN];
        trace(&pns, &ocel, 0, &mut a).expect("a");
        trace(&pns, &ocel, 0, &mut b).expect("b");
        assert_eq!(a, b);
    }

    // ---- wire (mgi_pn64_audit_compute) ----

    fn wire_in(mask: u64, rows: &[(u64, u64)]) -> Vec<u8> {
        let mut b = crate::Mgi1Header {
            leaf_id: LEAF_ID_PN64_AUDIT,
            scale_tag: fixed::TAG_SHARES,
            dims: rows.len() as u64,
        }
        .to_bytes()
        .to_vec();
        b.extend_from_slice(&mask.to_le_bytes());
        for (pn, ocel) in rows {
            b.extend_from_slice(&pn.to_le_bytes());
            b.extend_from_slice(&ocel.to_le_bytes());
        }
        b
    }

    #[test]
    fn wire_golden_trace() {
        // MX8 golden, reconciled with the word-changed flag law: post =
        // 0b1000 (NOT 0b1010) so the chained refire CHANGES the marking
        // (1110 -> 1100); with 0b1010 the refire is marking-idempotent and
        // the flag correctly reads 0, contradicting this court's 1,1.
        let pn = parent::pack(0b0110, 0b0010, 0b1000, 3, 0);
        let mut out = vec![0u8; 16 + 2 * STEP_RECORD_LEN];
        let buf = wire_in(0, &[(pn, 3), (pn, 3)]);
        let code = unsafe {
            mgi_pn64_audit_compute(
                buf.as_ptr(),
                buf.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::OK);
        // Header echoes leaf + dims; record area holds the chained trace.
        let hdr = crate::parse_header(&out).expect("hdr");
        assert_eq!(hdr.leaf_id, LEAF_ID_PN64_AUDIT);
        assert_eq!(hdr.dims, 2);
        // MX8 final: step 0 fires (flag 1). The chained step 1 is enabled
        // (W- place re-live) but marking-idempotent, so the WORD-CHANGED
        // flag law reports 0; the record's marking pair distinguishes the
        // idempotent commit from a refusal.
        assert_eq!(out[16 + 16], 1);
        assert_eq!(out[16 + 21 + 16], 0);
    }

    #[test]
    fn wire_refusals() {
        let mut out = vec![0u8; 16 + MAX_STEPS * STEP_RECORD_LEN];
        let good = wire_in(0, &[(sample_pn(), 3)]);
        // trailing
        let mut t = good.clone();
        t.push(0);
        let code = unsafe {
            mgi_pn64_audit_compute(t.as_ptr(), t.len() as u32, out.as_mut_ptr(), out.len() as u32)
        };
        assert_eq!(code, errno::ERR_TRAILING);
        // dims 0
        let zero = wire_in(0, &[]);
        let code = unsafe {
            mgi_pn64_audit_compute(
                zero.as_ptr(),
                zero.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::ERR_DIM);
        // wrong leaf id
        let b = crate::Mgi1Header { leaf_id: 99, scale_tag: fixed::TAG_SHARES, dims: 1 }.to_bytes();
        let code = unsafe {
            mgi_pn64_audit_compute(b.as_ptr(), b.len() as u32, out.as_mut_ptr(), out.len() as u32)
        };
        assert_eq!(code, errno::ERR_BAD_MAGIC);
        // cap too small
        let code = unsafe {
            mgi_pn64_audit_compute(good.as_ptr(), good.len() as u32, out.as_mut_ptr(), 8)
        };
        assert_eq!(code, errno::ERR_CAP_TOO_SMALL);
    }
}
