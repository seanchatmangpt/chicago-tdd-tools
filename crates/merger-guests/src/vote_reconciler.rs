//! MG4 leaf: share-weighted vote reconciler.
//!
//! Cross-multiplied fraction tallies in exact `i128` — no division, no
//! floats. A merger vote is admitted only when BOTH thresholds hold:
//!   1. Quorum:         `shares_present * quorum_den >= quorum_num * total_outstanding`
//!   2. Supermajority:  `votes_for      * super_den  >= super_num  * shares_present`
//!
//! `shares_present` = Σ row shares (every outstanding-voted share is either
//! for or against, so present == cast).
//!
//! Refusals are typed and total: on any refusal the output tally is left
//! untouched (scratch-validate-then-write, never partial).
//!
//! Caps: `ERR_DIM` when the row count exceeds [`MAX_ROWS`].
//!
//! ## Wire format (MGI1, leaf_id 7, scale_tag TAG_SHARES)
//!
//! Input body: `dims` rows x 24 B (`class_id: u64`, `shares: i64`,
//! `votes_for: i64`), then 32 B of thresholds (`quorum_num`, `quorum_den`,
//! `super_num`, `super_den`, each `i64`). `in_len` must be exactly
//! `16 + 24*dims + 32` (`ERR_TRUNCATED` / `ERR_TRAILING` otherwise).
//! Output body (40 B): `votes_for: i64`, `against: i64`, `total: i64`,
//! `total_outstanding: i64`, `admitted: u8` + 7 zero pad. Short `out_cap`
//! is `ERR_CAP_TOO_SMALL`. Negative leaf-local tallies never appear on the
//! wire; all i128 tallies fit `i64` under the [`crate::fixed`] scale caps.

/// Maximum vote rows per call. Exceeding this is `ERR_DIM`.
pub const MAX_ROWS: usize = 64;

/// One vote row: share class, shares outstanding in the class, and the
/// shares that voted for the proposal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoteRow {
    /// Share-class identifier (opaque; classes need not be contiguous).
    pub class_id: u64,
    /// Shares outstanding in this class, `0 < shares <= S_SHARES`.
    pub shares: i128,
    /// Shares voted for, `0 <= votes_for <= shares`.
    pub votes_for: i128,
}

/// Quorum + supermajority thresholds as exact fractions `num/den`.
///
/// Both denominators MUST be strictly positive; both fractions MUST lie in
/// `(0, 1]` — enforced by refusal, never clamped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Thresholds {
    /// Quorum numerator (`quorum_num / quorum_den`, e.g. 1/2).
    pub quorum_num: i128,
    /// Quorum denominator.
    pub quorum_den: i128,
    /// Supermajority numerator (e.g. 2/3).
    pub super_num: i128,
    /// Supermajority denominator.
    pub super_den: i128,
}

/// Output tallies, written only on full admission of the call's inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Tally {
    /// Σ votes_for across all classes.
    pub votes_for_shares: i128,
    /// Σ (shares - votes_for) across all classes.
    pub votes_against_shares: i128,
    /// Σ shares across all classes.
    pub total_shares: i128,
    /// True iff quorum AND supermajority both hold.
    pub admitted: bool,
}

/// Refusal reasons local to this leaf (leaf-local namespace; the shared
/// `ERR_LENGTH`/`ERR_DIM` codes come from `crate::errno`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoteError {
    /// Row count > [`MAX_ROWS`] or an empty row set (`ERR_DIM`).
    Dim,
    /// Empty row set (`ERR_DIM`).
    Empty,
    /// A row violates its domain (shares <= 0, votes_for outside
    /// `[0, shares]`, shares > `crate::fixed::S_SHARES`).
    Row,
    /// A threshold fraction is malformed (den <= 0, num <= 0, or num > den).
    Threshold,
}

impl VoteError {
    /// Wire code: `ERR_DIM` for dimension faults, else a leaf-local code.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            // ERR_DIM is the shared dimension cap code.
            Self::Dim | Self::Empty => crate::errno::ERR_DIM,
            Self::Row => -4101,
            Self::Threshold => -4102,
        }
    }
}

/// Share-weighted tally with cross-multiplied threshold admission.
///
/// Scratch-validates every row and both threshold fractions BEFORE writing
/// `out`; on any refusal `out` is bit-exact untouched.
///
/// `total_outstanding` is the total shares outstanding across ALL classes
/// (including classes not present in `rows`); quorum is measured against it.
///
/// # Errors
/// [`VoteError::Dim`]/[`VoteError::Empty`] when the row set exceeds
/// [`MAX_ROWS`] or is empty; [`VoteError::Row`] on any out-of-domain row
/// (or `total_outstanding < Σ shares`); [`VoteError::Threshold`] on a
/// malformed fraction.
pub fn tally(
    rows: &[VoteRow],
    total_outstanding: i128,
    th: &Thresholds,
    out: &mut Tally,
) -> Result<(), VoteError> {
    if rows.is_empty() {
        return Err(VoteError::Empty);
    }
    if rows.len() > MAX_ROWS {
        return Err(VoteError::Dim);
    }
    // Fraction domain: 0 < num <= den, den > 0. No clamping.
    if th.quorum_den <= 0 || th.quorum_num <= 0 || th.quorum_num > th.quorum_den {
        return Err(VoteError::Threshold);
    }
    if th.super_den <= 0 || th.super_num <= 0 || th.super_num > th.super_den {
        return Err(VoteError::Threshold);
    }

    // Scratch accumulators — committed to `out` only after every row passes.
    let mut total: i128 = 0;
    let mut for_shares: i128 = 0;
    for r in rows {
        if r.shares <= 0 || r.shares > crate::fixed::S_SHARES {
            return Err(VoteError::Row);
        }
        if r.votes_for < 0 || r.votes_for > r.shares {
            return Err(VoteError::Row);
        }
        total = total.checked_add(r.shares).ok_or(VoteError::Row)?;
        for_shares = for_shares.checked_add(r.votes_for).ok_or(VoteError::Row)?;
    }
    let against: i128 = total - for_shares;
    if total_outstanding < total {
        return Err(VoteError::Row);
    }

    // Cross-multiplied fractions — integer only, no division anywhere.
    //   quorum:        shares_present * qd >= qn * total_outstanding
    //   supermajority: votes_for      * sd >= sn * shares_present
    let quorum_lhs: i128 = total.checked_mul(th.quorum_den).ok_or(VoteError::Row)?;
    let quorum_rhs: i128 = total_outstanding.checked_mul(th.quorum_num).ok_or(VoteError::Row)?;
    let super_lhs: i128 = for_shares.checked_mul(th.super_den).ok_or(VoteError::Row)?;
    let super_rhs: i128 = total.checked_mul(th.super_num).ok_or(VoteError::Row)?;
    let quorum_ok: bool = quorum_lhs >= quorum_rhs;
    let super_ok: bool = super_lhs >= super_rhs;

    // Commit phase — every input validated, tallies known exact.
    out.votes_for_shares = for_shares;
    out.votes_against_shares = against;
    out.total_shares = total;
    out.admitted = quorum_ok && super_ok;
    Ok(())
}

/// Stable leaf identity (`Mgi1Header.leaf_id`).
use crate::{errno, fixed};

/// Stable leaf identity (`Mgi1Header.leaf_id`).
pub const LEAF_ID_VOTE_RECONCILER: u8 = 7;

/// Output body size: 5 x i64 tallies + admitted byte + 7 zero pad.
const VOTE_OUT_LEN: usize = 48;

/// Closed-vote wire law: the host encodes no separate outstanding figure,
/// so quorum is measured against Σ row shares (a class not in `rows` did
/// not vote and is not outstanding for quorum purposes).
fn run(in_buf: &[u8], out_buf: &mut [u8]) -> Result<(), i32> {
    let hdr = crate::parse_header(in_buf)?;
    if hdr.leaf_id != LEAF_ID_VOTE_RECONCILER {
        return Err(errno::ERR_BAD_MAGIC);
    }
    if hdr.scale_tag != fixed::TAG_SHARES {
        return Err(errno::ERR_BAD_SCALE);
    }
    if hdr.dims == 0 || hdr.dims > MAX_ROWS as u64 {
        return Err(errno::ERR_DIM);
    }
    let d = hdr.dims as usize;
    let rows_end = 16 + 24 * d;
    let exact_len = rows_end + 32;
    if in_buf.len() < exact_len {
        return Err(errno::ERR_TRUNCATED);
    }
    if in_buf.len() > exact_len {
        return Err(errno::ERR_TRAILING);
    }
    let mut rows: Vec<VoteRow> = Vec::with_capacity(d);
    for i in 0..d {
        let b = &in_buf[16 + 24 * i..16 + 24 * (i + 1)];
        rows.push(VoteRow {
            class_id: u64::from_le_bytes(b[0..8].try_into().map_err(|_| errno::ERR_TRUNCATED)?),
            shares: i64::from_le_bytes(b[8..16].try_into().map_err(|_| errno::ERR_TRUNCATED)?)
                as i128,
            votes_for: i64::from_le_bytes(b[16..24].try_into().map_err(|_| errno::ERR_TRUNCATED)?)
                as i128,
        });
    }
    let tb = &in_buf[rows_end..exact_len];
    let rd = |o: usize| -> Result<i128, i32> {
        Ok(i64::from_le_bytes(tb[o..o + 8].try_into().map_err(|_| errno::ERR_TRUNCATED)?) as i128)
    };
    let th = Thresholds {
        quorum_num: rd(0)?,
        quorum_den: rd(8)?,
        super_num: rd(16)?,
        super_den: rd(24)?,
    };
    let total_outstanding: i128 = rows.iter().map(|r| r.shares).sum();

    let mut t = Tally::default();
    tally(&rows, total_outstanding, &th, &mut t).map_err(VoteError::code)?;
    if out_buf.len() < MGI_OUT_MIN {
        return Err(errno::ERR_CAP_TOO_SMALL);
    }
    let ohdr = crate::Mgi1Header {
        leaf_id: LEAF_ID_VOTE_RECONCILER,
        scale_tag: fixed::TAG_SHARES,
        dims: 4,
    };
    ohdr.encode(out_buf);
    // JUSTIFICATION: every tally is bounded by MAX_ROWS * S_SHARES =
    // 64 * 1e6 = 6.4e7; the i64 narrowing cannot truncate.
    #[allow(clippy::cast_possible_truncation)]
    let tallies = (
        t.votes_for_shares as i64,
        t.votes_against_shares as i64,
        t.total_shares as i64,
        total_outstanding as i64,
    );
    out_buf[16..24].copy_from_slice(&tallies.0.to_le_bytes());
    out_buf[24..32].copy_from_slice(&tallies.1.to_le_bytes());
    out_buf[32..40].copy_from_slice(&tallies.2.to_le_bytes());
    out_buf[40..48].copy_from_slice(&tallies.3.to_le_bytes());
    out_buf[48] = u8::from(t.admitted);
    out_buf[49..MGI_OUT_MIN].fill(0);
    Ok(())
}

const MGI_OUT_MIN: usize = 16 + VOTE_OUT_LEN;

/// Wire-level entry point: `i32 mgi_vote_reconciler_compute(in, in_len, out, out_cap) -> i32`.
/// 0 = ok; positive = errno; negative reserved for host transport.
///
/// # Safety
/// `in`/`out` must be valid for reads/writes of `in_len`/`out_cap` bytes
/// (caller-provided linear memory, cdylib ABI contract).
#[no_mangle]
pub unsafe extern "C" fn mgi_vote_reconciler_compute(
    in_: *const u8,
    in_len: u32,
    out: *mut u8,
    out_cap: u32,
) -> i32 {
    if in_.is_null() || out.is_null() {
        return errno::ERR_NULL_PTR;
    }
    if (out_cap as usize) < MGI_OUT_MIN {
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

    const HALF: Thresholds =
        Thresholds { quorum_num: 1, quorum_den: 2, super_num: 2, super_den: 3 };

    #[test]
    fn golden_admitted_majority_of_majority() {
        let rows = [
            VoteRow { class_id: 1, shares: 600, votes_for: 500 },
            VoteRow { class_id: 2, shares: 400, votes_for: 300 },
        ];
        let mut t = Tally::default();
        tally(&rows, 1_000, &HALF, &mut t).expect("golden");
        assert_eq!(t.total_shares, 1_000);
        assert_eq!(t.votes_for_shares, 800);
        assert_eq!(t.votes_against_shares, 200);
        assert!(t.admitted);
    }

    #[test]
    fn golden_supermajority_refused() {
        // 666/1000 = 66.6% < 2/3 = 66.67%: cross-multiplication refuses.
        let rows = [VoteRow { class_id: 1, shares: 1_000, votes_for: 666 }];
        let mut t = Tally::default();
        tally(&rows, 1_000, &HALF, &mut t).expect("valid input");
        assert!(!t.admitted);
        assert_eq!(t.votes_for_shares, 666);
    }

    #[test]
    fn boundary_exact_two_thirds_admitted() {
        // 667/1000 > 2/3 (667*3=2001 >= 2*1000) — admitted at the boundary.
        let rows = [VoteRow { class_id: 1, shares: 1_000, votes_for: 667 }];
        let mut t = Tally::default();
        tally(&rows, 1_000, &HALF, &mut t).expect("valid input");
        assert!(t.admitted);
    }

    #[test]
    fn quorum_fail_admission_refused_but_tallies_committed() {
        // Quorum is an admission predicate, not an input fault: the call
        // succeeds and reports admitted=false.
        let rows = [VoteRow { class_id: 1, shares: 1_000, votes_for: 600 }];
        let th = Thresholds { quorum_num: 9, quorum_den: 10, super_num: 1, super_den: 2 };
        let mut t = Tally::default();
        // RV3 repair: quorum is attendance-based (present vs outstanding).
        // 1000 of 1000 outstanding always meets 9/10; use 2_000 outstanding
        // so 1000*10 = 10_000 < 9*2_000 = 18_000 → quorum FAILS while the
        // supermajority (600 ≥ 500 of present) holds → admitted=false with
        // tallies committed.
        tally(&rows, 2_000, &th, &mut t).expect("valid input");
        assert!(!t.admitted);
    }

    #[test]
    fn quorum_measured_against_outstanding_not_present() {
        // 900 present of 2_000 outstanding: 900*2=1800 < 1*2000 → quorum fails.
        let rows = [VoteRow { class_id: 1, shares: 900, votes_for: 900 }];
        let th = Thresholds { quorum_num: 1, quorum_den: 2, super_num: 1, super_den: 2 };
        let mut t = Tally::default();
        tally(&rows, 2_000, &th, &mut t).expect("valid input");
        assert!(!t.admitted);
    }

    #[test]
    fn empty_rows_refused_output_untouched() {
        let mut t = Tally { votes_for_shares: -1, ..Tally::default() };
        assert_eq!(tally(&[], 100, &HALF, &mut t), Err(VoteError::Empty));
        assert_eq!(t.votes_for_shares, -1, "output must be untouched");
    }

    #[test]
    fn oversize_rows_err_dim() {
        let rows = [VoteRow { class_id: 1, shares: 1, votes_for: 0 }; MAX_ROWS + 1];
        let mut t = Tally::default();
        assert_eq!(tally(&rows, 100, &HALF, &mut t), Err(VoteError::Dim));
    }

    #[test]
    fn bad_row_refused_output_untouched() {
        let rows = [
            VoteRow { class_id: 1, shares: 500, votes_for: 100 },
            VoteRow { class_id: 2, shares: 100, votes_for: 200 }, // votes > shares
        ];
        let mut t = Tally::default();
        assert_eq!(tally(&rows, 1_000, &HALF, &mut t), Err(VoteError::Row));
        assert_eq!(t.total_shares, 0, "no partial commit");
    }

    #[test]
    fn present_exceeds_outstanding_refused() {
        let rows = [VoteRow { class_id: 1, shares: 500, votes_for: 0 }];
        let mut t = Tally::default();
        assert_eq!(tally(&rows, 400, &HALF, &mut t), Err(VoteError::Row));
    }

    #[test]
    fn shares_over_s_cap_refused() {
        let rows = [VoteRow { class_id: 1, shares: crate::fixed::S_SHARES + 1, votes_for: 0 }];
        let mut t = Tally::default();
        assert_eq!(tally(&rows, 100, &HALF, &mut t), Err(VoteError::Row));
    }

    #[test]
    fn malformed_fractions_refused() {
        let rows = [VoteRow { class_id: 1, shares: 10, votes_for: 5 }];
        let mut t = Tally::default();
        for th in [
            Thresholds { quorum_num: 0, quorum_den: 2, super_num: 1, super_den: 2 },
            Thresholds { quorum_num: 3, quorum_den: 2, super_num: 1, super_den: 2 },
            Thresholds { quorum_num: 1, quorum_den: 0, super_num: 1, super_den: 2 },
            Thresholds { quorum_num: 1, quorum_den: 2, super_num: 0, super_den: 2 },
            Thresholds { quorum_num: 1, quorum_den: 2, super_num: 5, super_den: 4 },
        ] {
            assert_eq!(tally(&rows, 10, &th, &mut t), Err(VoteError::Threshold));
        }
    }

    #[test]
    fn deterministic_repeat_calls() {
        let rows = [VoteRow { class_id: 7, shares: 999, votes_for: 501 }];
        let mut a = Tally::default();
        let mut b = Tally::default();
        tally(&rows, 999, &HALF, &mut a).expect("a");
        tally(&rows, 999, &HALF, &mut b).expect("b");
        assert_eq!(a, b);
    }

    // ---- wire (mgi_vote_reconciler_compute) ----

    fn wire_in(rows: &[VoteRow], th: &Thresholds) -> Vec<u8> {
        let mut b = crate::Mgi1Header {
            leaf_id: LEAF_ID_VOTE_RECONCILER,
            scale_tag: fixed::TAG_SHARES,
            dims: rows.len() as u64,
        }
        .to_bytes()
        .to_vec();
        for r in rows {
            b.extend_from_slice(&r.class_id.to_le_bytes());
            b.extend_from_slice(&(r.shares as i64).to_le_bytes());
            b.extend_from_slice(&(r.votes_for as i64).to_le_bytes());
        }
        for v in [th.quorum_num, th.quorum_den, th.super_num, th.super_den] {
            b.extend_from_slice(&(v as i64).to_le_bytes());
        }
        b
    }

    #[test]
    fn wire_golden_admitted() {
        let mut out = vec![0u8; MGI_OUT_MIN];
        let code = unsafe {
            mgi_vote_reconciler_compute(
                wire_in(
                    &[
                        VoteRow { class_id: 1, shares: 600, votes_for: 500 },
                        VoteRow { class_id: 2, shares: 400, votes_for: 300 },
                    ],
                    &HALF,
                )
                .as_ptr(),
                0,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        // in_len = 0 must truncate-refuse; then the real call.
        assert_eq!(code, errno::ERR_TRUNCATED);
        let buf = wire_in(
            &[
                VoteRow { class_id: 1, shares: 600, votes_for: 500 },
                VoteRow { class_id: 2, shares: 400, votes_for: 300 },
            ],
            &HALF,
        );
        let code = unsafe {
            mgi_vote_reconciler_compute(
                buf.as_ptr(),
                buf.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::OK);
        assert_eq!(crate::parse_header(&out).expect("hdr").leaf_id, LEAF_ID_VOTE_RECONCILER);
        assert_eq!(out[48], 1, "admitted");
        assert_eq!(i64::from_le_bytes(out[32..40].try_into().expect("8")), 1_000, "total shares");
    }

    #[test]
    fn wire_refusals() {
        let mut out = vec![0u8; MGI_OUT_MIN];
        let good = wire_in(&[VoteRow { class_id: 1, shares: 10, votes_for: 5 }], &HALF);
        // trailing
        let mut t = good.clone();
        t.push(0);
        let code = unsafe {
            mgi_vote_reconciler_compute(
                t.as_ptr(),
                t.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::ERR_TRAILING);
        // wrong leaf id
        let mut b = crate::Mgi1Header { leaf_id: 99, scale_tag: fixed::TAG_SHARES, dims: 1 }
            .to_bytes()
            .to_vec();
        b.extend_from_slice(&good[16..]);
        let code = unsafe {
            mgi_vote_reconciler_compute(
                b.as_ptr(),
                b.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::ERR_BAD_MAGIC);
        // small cap
        let code = unsafe {
            mgi_vote_reconciler_compute(good.as_ptr(), good.len() as u32, out.as_mut_ptr(), 4)
        };
        assert_eq!(code, errno::ERR_CAP_TOO_SMALL);
    }
}
