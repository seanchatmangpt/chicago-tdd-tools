//! MG4 leaf: exact i128 fractional-share proration (largest remainder).
//!
//! Each holder `i` with `shares_i` receives, at exchange ratio
//! `ratio_fixed / S_RATIO` (`ratio_fixed` in fixed-point S_RATIO units,
//! `0 < ratio_fixed <= S_RATIO`; see [`ratio_parts`]), a whole-share
//! allocation and a fractional cash residual. All arithmetic is exact
//! `i128`; the only division is the floor split `q / den`, `q % den` of an
//! exact product. Leftover whole shares go out by largest remainder with a
//! documented deterministic tie-break: equal remainders resolve to the
//! ascending holder index.
//!
//! Conservation law (checked, exact i128 equality):
//!   `Σ fractional_cash_i + Σ whole_shares_value_i == Σ consideration_i`
//! where `consideration_i = shares_i * ratio_fixed` (S_RATIO-scaled),
//! `whole_shares_value_i = whole_i * S_RATIO`, and `fractional_cash_i` is
//! the exact residual `consideration_i − whole_shares_value_i`.
//!
//! ## Wire format (MGI1, leaf_id 9, scale_tag TAG_SHARES)
//!
//! Input body: `i64 ratio_fixed` (S_RATIO units), then `dims` rows of 8 B
//! (`shares: i64`). `in_len` must be exactly `16 + 8 + 8*dims`
//! (`ERR_TRUNCATED` / `ERR_TRAILING`). Output body: `i64 leftover`, then
//! `dims` rows of 16 B (`whole_shares: i64`, `fractional_cash: i64`).
//! Short `out_cap` is `ERR_CAP_TOO_SMALL`.

/// Maximum holders per proration call. Exceeding this is `ERR_DIM`.
pub const MAX_HOLDERS: usize = 64;

/// Stable leaf identity (`Mgi1Header.leaf_id`).
use crate::{errno, fixed};

/// Stable leaf identity (`Mgi1Header.leaf_id`).
pub const LEAF_ID_FRACTIONAL_SHARES: u8 = 9;

/// Refusal reasons local to this leaf.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProrateError {
    /// Holder count 0 or > [`MAX_HOLDERS`] (shared `ERR_DIM`).
    Dim,
    /// Caller output slices shorter than the holder count
    /// (shared `ERR_LENGTH`); whole-call refusal — never partial.
    Length,
    /// A holder's shares are negative or exceed `crate::fixed::S_SHARES`,
    /// or an output value would overflow `i128`.
    Holder(usize),
    /// The exchange ratio is malformed (`ratio_fixed <= 0` or
    /// `> S_RATIO`).
    Ratio,
    /// The conservation law failed (defensive check; unreachable by
    /// construction, refusal not silence).
    Conservation,
}

impl ProrateError {
    /// Wire code. Per the ABI law (`lib.rs`), every code returned through
    /// `run()` is a documented shared errno — negatives are reserved for the
    /// host transport and never cross the leaf boundary. An out-of-domain
    /// holder is a payload-framing mismatch (`ERR_LENGTH`); a malformed
    /// exchange ratio is a malformed scale-like ratio (`ERR_BAD_SCALE`); a
    /// conservation failure means no feasible proration exists
    /// (`ERR_NO_SOLUTION`).
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Dim => crate::errno::ERR_DIM,
            Self::Length => crate::errno::ERR_LENGTH,
            Self::Holder(_) => crate::errno::ERR_LENGTH,
            Self::Ratio => crate::errno::ERR_BAD_SCALE,
            Self::Conservation => crate::errno::ERR_NO_SOLUTION,
        }
    }
}

/// One settled holder: whole shares and the exact fractional cash residual.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Split {
    /// Whole shares allocated after largest-remainder top-up.
    pub whole_shares: i128,
    /// Exact residual, S_RATIO-scaled numerator units:
    /// `shares_i * ratio_fixed − whole_shares * S_RATIO`.
    pub fractional_cash: i128,
}

/// The exchange ratio as `(num, den)` in the parent's fixed-point
/// convention: `den` is the [`crate::fixed::S_RATIO`] scale and `num` is
/// the caller's `ratio_fixed` value. This leaf never invents a second
/// encoding — it applies the parent scale and domain-checks.
#[must_use]
pub const fn ratio_parts(ratio_fixed: i128) -> (i128, i128) {
    (ratio_fixed, crate::fixed::S_RATIO)
}

/// Exact proration across all holders.
///
/// Scratch-validates, allocates whole shares by largest remainder with
/// ascending-index tie-break, verifies the conservation law in exact i128,
/// and only then writes `whole`/`cash`. On any refusal both output slices
/// are bit-exact untouched.
///
/// Returns the number of leftover whole shares distributed (Σ top-ups).
///
/// # Errors
/// See [`ProrateError`]. Every variant is a whole-call refusal.
pub fn prorate(
    shares: &[i128],
    ratio_fixed: i128,
    whole: &mut [i128],
    cash: &mut [i128],
) -> Result<i128, ProrateError> {
    let (num, den) = ratio_parts(ratio_fixed);
    if den <= 0 || num <= 0 || num > den {
        return Err(ProrateError::Ratio);
    }
    if shares.is_empty() || shares.len() > MAX_HOLDERS {
        return Err(ProrateError::Dim);
    }
    // ERR_LENGTH contract: short outputs refuse the WHOLE call up front.
    if whole.len() < shares.len() || cash.len() < shares.len() {
        return Err(ProrateError::Length);
    }

    let cap = crate::fixed::S_SHARES;

    // Scratch phase.
    let mut scratch = [Split::default(); MAX_HOLDERS];
    let mut sum_q: i128 = 0;
    for (i, &s) in shares.iter().enumerate() {
        if s < 0 || s > cap {
            return Err(ProrateError::Holder(i));
        }
        // q_i = shares_i * ratio_fixed — exact quota, S_RATIO-scaled.
        let q: i128 = s.checked_mul(num).ok_or(ProrateError::Holder(i))?;
        let base = q / den; // floor: q >= 0
        let rem = q % den;
        scratch[i] = Split { whole_shares: base, fractional_cash: rem };
        sum_q = sum_q.checked_add(q).ok_or(ProrateError::Holder(i))?;
    }

    // Largest remainder: leftover whole shares = floor(Σq / den) − Σbase.
    // Distribute one whole share each to the largest remainders; equal
    // remainders resolve to the ascending holder index.
    let sum_base: i128 = scratch[..shares.len()]
        .iter()
        .map(|s| s.whole_shares)
        .try_fold(0i128, i128::checked_add)
        .ok_or(ProrateError::Conservation)?;
    let leftover: i128 = (sum_q / den) - sum_base;
    if leftover < 0 {
        return Err(ProrateError::Conservation);
    }
    // JUSTIFICATION: leftover = Σ rem / den < shares.len() <= MAX_HOLDERS
    // by construction; the refusal above makes the conversion total rather
    // than a silent wrap.
    let leftover: usize = usize::try_from(leftover).map_err(|_| ProrateError::Conservation)?;

    // Deterministic top-up: order holders by (remainder desc, index asc)
    // via in-place selection — no sort, no alloc; MAX_HOLDERS bounds cost.
    let mut to_top_up = [false; MAX_HOLDERS];
    let mut order: [usize; MAX_HOLDERS] = core::array::from_fn(|i| i);
    for k in 0..shares.len() {
        let mut best = k;
        let mut best_val = scratch[order[k]].fractional_cash;
        for j in (k + 1)..shares.len() {
            let a = scratch[order[j]].fractional_cash;
            // Strict `>` against the RUNNING BEST (not the slot being
            // filled) keeps equal remainders in ascending index order:
            // a later equal candidate never displaces the earlier one.
            if a > best_val {
                best_val = a;
                best = j;
            }
        }
        order.swap(k, best);
    }
    for &idx in order.iter().take(leftover) {
        to_top_up[idx] = true;
    }

    // Conservation check over the FINAL allocation, exact i128:
    //   Σ cash_i + Σ whole_i * den == Σ q_i
    //
    // RV3 repair: a topped-up holder receives a whole share worth `den`,
    // so their exact residual cash is `rem − den` (the documented law
    // `fractional_cash_i = consideration_i − whole_shares_value_i`, which
    // may be negative). Without this adjustment a top-up over-issues
    // `den` of value and the conservation law is violated for every
    // leftover-distributing call.
    let final_cash = |i: usize| -> Result<i128, ProrateError> {
        if to_top_up[i] {
            scratch[i].fractional_cash.checked_sub(den).ok_or(ProrateError::Conservation)
        } else {
            Ok(scratch[i].fractional_cash)
        }
    };
    let mut sum_cash: i128 = 0;
    let mut sum_whole_value: i128 = 0;
    for i in 0..shares.len() {
        let w = if to_top_up[i] { scratch[i].whole_shares + 1 } else { scratch[i].whole_shares };
        sum_cash = sum_cash.checked_add(final_cash(i)?).ok_or(ProrateError::Conservation)?;
        sum_whole_value = sum_whole_value
            .checked_add(w.checked_mul(den).ok_or(ProrateError::Conservation)?)
            .ok_or(ProrateError::Conservation)?;
    }
    if sum_cash.checked_add(sum_whole_value) != Some(sum_q) {
        return Err(ProrateError::Conservation);
    }

    // Commit phase.
    for i in 0..shares.len() {
        whole[i] = if to_top_up[i] { scratch[i].whole_shares + 1 } else { scratch[i].whole_shares };
        cash[i] = final_cash(i)?;
    }
    Ok(leftover_before_check(leftover))
}

/// usize -> i128 widening, refusal-not-silence form.
// JUSTIFICATION: leftover < MAX_HOLDERS (guarded in `prorate`); the
// widening cannot wrap.
#[allow(clippy::cast_possible_wrap)]
const fn leftover_before_check(leftover: usize) -> i128 {
    leftover as i128
}

/// Byte-level adapter (safe Rust, exercised by tests).
fn run(in_buf: &[u8], out_buf: &mut [u8]) -> Result<(), i32> {
    let hdr = crate::parse_header(in_buf)?;
    if hdr.leaf_id != LEAF_ID_FRACTIONAL_SHARES {
        return Err(errno::ERR_BAD_MAGIC);
    }
    if hdr.scale_tag != fixed::TAG_SHARES {
        return Err(errno::ERR_BAD_SCALE);
    }
    // SEU-hardened length computation: a flipped dims bit must refuse typed
    // (ERR_DIM), never panic on usize overflow (debug builds abort).
    let dims = usize::try_from(hdr.dims).map_err(|_| errno::ERR_DIM)?;
    let exact_len = 24usize
        .checked_add(8usize.checked_mul(dims).ok_or(errno::ERR_DIM)?)
        .ok_or(errno::ERR_DIM)?;
    if in_buf.len() < exact_len {
        return Err(errno::ERR_TRUNCATED);
    }
    if in_buf.len() > exact_len {
        return Err(errno::ERR_TRAILING);
    }
    if hdr.dims == 0 || hdr.dims > MAX_HOLDERS as u64 {
        return Err(errno::ERR_DIM);
    }
    let d = hdr.dims as usize;
    let ratio_fixed =
        i64::from_le_bytes(in_buf[16..24].try_into().map_err(|_| errno::ERR_TRUNCATED)?) as i128;
    let mut shares: Vec<i128> = Vec::with_capacity(d);
    for i in 0..d {
        let b = &in_buf[24 + 8 * i..24 + 8 * (i + 1)];
        shares.push(i64::from_le_bytes(b.try_into().map_err(|_| errno::ERR_TRUNCATED)?) as i128);
    }
    let mut whole = vec![0i128; d];
    let mut cash = vec![0i128; d];
    let leftover =
        prorate(&shares, ratio_fixed, &mut whole, &mut cash).map_err(ProrateError::code)?;
    let need = 16 + 8 + 16 * d;
    if out_buf.len() < need {
        return Err(errno::ERR_CAP_TOO_SMALL);
    }
    let ohdr = crate::Mgi1Header {
        leaf_id: LEAF_ID_FRACTIONAL_SHARES,
        scale_tag: fixed::TAG_SHARES,
        dims: d as u64,
    };
    ohdr.encode(out_buf);
    // JUSTIFICATION: leftover < MAX_HOLDERS = 64 (core invariant); the
    // narrowing to i64 cannot truncate.
    #[allow(clippy::cast_possible_truncation)]
    let leftover_out = leftover as i64;
    out_buf[16..24].copy_from_slice(&leftover_out.to_le_bytes());
    for i in 0..d {
        let o = 24 + 16 * i;
        // JUSTIFICATION: whole <= S_SHARES (1e6) and cash <= S_RATIO *
        // S_SHARES = 1e14 by the core domain checks; i64 cannot truncate.
        #[allow(clippy::cast_possible_truncation)]
        let (w_out, c_out) = (whole[i] as i64, cash[i] as i64);
        out_buf[o..o + 8].copy_from_slice(&w_out.to_le_bytes());
        out_buf[o + 8..o + 16].copy_from_slice(&c_out.to_le_bytes());
    }
    Ok(())
}

/// Wire-level entry point: `i32 mgi_fractional_shares_compute(in, in_len, out, out_cap) -> i32`.
/// 0 = ok; positive = errno; negative reserved for host transport.
///
/// # Safety
/// `in`/`out` must be valid for reads/writes of `in_len`/`out_cap` bytes
/// (caller-provided linear memory, cdylib ABI contract).
#[no_mangle]
pub unsafe extern "C" fn mgi_fractional_shares_compute(
    in_: *const u8,
    in_len: u32,
    out: *mut u8,
    out_cap: u32,
) -> i32 {
    if in_.is_null() || out.is_null() {
        return errno::ERR_NULL_PTR;
    }
    if (out_cap as usize) < 16 + 8 + 16 {
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

    // 1:2 exchange ratio, S_RATIO fixed-point; 1:1 is the identity ratio.
    const RATIO_1_2: i128 = fixed::S_RATIO / 2;
    const RATIO_1_1: i128 = fixed::S_RATIO;

    #[test]
    fn ratio_parts_golden() {
        assert_eq!(ratio_parts(RATIO_1_2), (fixed::S_RATIO / 2, fixed::S_RATIO));
        assert_eq!(ratio_parts(RATIO_1_1), (fixed::S_RATIO, fixed::S_RATIO));
    }

    #[test]
    fn golden_two_holders_one_of_two() {
        // 1 share at 1:2 → q = S_RATIO/2, base = 0, rem = S_RATIO/2.
        // leftover = Σq/den − Σbase = 1 − 0 = 1: the largest remainder
        // (tie) top-ups holder 0 → whole [1, 0]. The topped-up holder's
        // exact residual cash is rem − den = −S_RATIO/2 (they received a
        // whole share worth `den` against an entitlement of S_RATIO/2);
        // conservation Σcash + Σwhole·den == Σq stays exact.
        let shares = [1, 1];
        let mut w = [0i128; 2];
        let mut c = [0i128; 2];
        let tops = prorate(&shares, RATIO_1_2, &mut w, &mut c).expect("golden");
        assert_eq!(tops, 1);
        assert_eq!(w, [1, 0]);
        assert_eq!(c, [-fixed::S_RATIO / 2, fixed::S_RATIO / 2]);
    }

    #[test]
    fn golden_largest_remainder_top_up_with_index_tiebreak() {
        // 3 holders, 1:2: q = 1.5, 1.5, 1.0 (× S_RATIO) → base 1, 1, 1
        // (rem S_RATIO/2, S_RATIO/2, 0); Σq = 4·S_RATIO → leftover =
        // 4 − 3 = 1 → tie between holders 0 and 1 → ascending index:
        // holder 0 gets the extra share.
        let shares = [3, 3, 2];
        let mut w = [0i128; 3];
        let mut c = [0i128; 3];
        let tops = prorate(&shares, RATIO_1_2, &mut w, &mut c).expect("golden");
        assert_eq!(tops, 1);
        assert_eq!(w, [2, 1, 1]);
        // Topped-up holder 0: exact residual = rem − den = −S_RATIO/2.
        assert_eq!(c, [-fixed::S_RATIO / 2, fixed::S_RATIO / 2, 0]);
    }

    #[test]
    fn conservation_law_holds_exactly() {
        let shares = [7, 13, 1, 40, 9];
        let mut w = [0i128; 5];
        let mut c = [0i128; 5];
        prorate(&shares, RATIO_1_2, &mut w, &mut c).expect("run");
        let (num, den) = ratio_parts(RATIO_1_2);
        let consideration: i128 = shares.iter().map(|s| s * num).sum();
        let whole_value: i128 = w.iter().map(|x| x * den).sum();
        let cash: i128 = c.iter().sum();
        assert_eq!(cash + whole_value, consideration);
        for (&s, &x) in shares.iter().zip(w.iter()) {
            assert!(x <= s);
        }
    }

    #[test]
    fn one_to_one_ratio_no_remainder() {
        let shares = [5, 10];
        let mut w = [0i128; 2];
        let mut c = [0i128; 2];
        let tops = prorate(&shares, RATIO_1_1, &mut w, &mut c).expect("run");
        assert_eq!(tops, 0);
        assert_eq!(w, [5, 10]);
        assert_eq!(c, [0, 0]);
    }

    #[test]
    fn malformed_ratio_refused() {
        let shares = [1];
        let mut w = [0i128; 1];
        let mut c = [0i128; 1];
        for r in [0i128, -1, fixed::S_RATIO + 1] {
            assert_eq!(prorate(&shares, r, &mut w, &mut c), Err(ProrateError::Ratio));
        }
    }

    #[test]
    fn empty_and_oversize_err_dim() {
        let mut w = [0i128; MAX_HOLDERS];
        let mut c = [0i128; MAX_HOLDERS];
        assert_eq!(prorate(&[], RATIO_1_2, &mut w, &mut c), Err(ProrateError::Dim));
        let big = [1i128; MAX_HOLDERS + 1];
        let mut wb = [0i128; MAX_HOLDERS + 1];
        let mut cb = [0i128; MAX_HOLDERS + 1];
        assert_eq!(prorate(&big, RATIO_1_2, &mut wb, &mut cb), Err(ProrateError::Dim));
    }

    #[test]
    fn short_output_slices_whole_call_err_length() {
        let shares = [3, 3];
        let mut w = [0i128; 1]; // short
        let mut c = [0i128; 2];
        assert_eq!(prorate(&shares, RATIO_1_2, &mut w, &mut c), Err(ProrateError::Length));
        assert_eq!(w[0], 0, "never partial");
        assert_eq!(ProrateError::Length.code(), crate::errno::ERR_LENGTH);
    }

    #[test]
    fn negative_and_over_cap_holders_refused() {
        let mut w = [0i128; 2];
        let mut c = [0i128; 2];
        let neg = [-1, 5];
        assert_eq!(prorate(&neg, RATIO_1_2, &mut w, &mut c), Err(ProrateError::Holder(0)));
        let over = [1, crate::fixed::S_SHARES + 1];
        assert_eq!(prorate(&over, RATIO_1_2, &mut w, &mut c), Err(ProrateError::Holder(1)));
    }

    #[test]
    fn deterministic_repeat_calls() {
        let shares = [9, 4, 17];
        let mut a_w = [0i128; 3];
        let mut a_c = [0i128; 3];
        let mut b_w = [0i128; 3];
        let mut b_c = [0i128; 3];
        prorate(&shares, RATIO_1_2, &mut a_w, &mut a_c).expect("a");
        prorate(&shares, RATIO_1_2, &mut b_w, &mut b_c).expect("b");
        assert_eq!(a_w, b_w);
        assert_eq!(a_c, b_c);
    }

    // ---- wire (mgi_fractional_shares_compute) ----

    fn wire_in(ratio_fixed: i128, shares: &[i128]) -> Vec<u8> {
        let mut b = crate::Mgi1Header {
            leaf_id: LEAF_ID_FRACTIONAL_SHARES,
            scale_tag: fixed::TAG_SHARES,
            dims: shares.len() as u64,
        }
        .to_bytes()
        .to_vec();
        b.extend_from_slice(&(ratio_fixed as i64).to_le_bytes());
        for s in shares {
            b.extend_from_slice(&(*s as i64).to_le_bytes());
        }
        b
    }

    #[test]
    fn wire_golden_proration() {
        // 3 holders, 1:2 → whole [2, 1, 1], leftover 1 (index tiebreak).
        let mut out = vec![0u8; 16 + 8 + 3 * 16];
        let buf = wire_in(RATIO_1_2, &[3, 3, 2]);
        let code = unsafe {
            mgi_fractional_shares_compute(
                buf.as_ptr(),
                buf.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::OK);
        assert_eq!(i64::from_le_bytes(out[16..24].try_into().expect("8")), 1, "leftover");
        assert_eq!(i64::from_le_bytes(out[24..32].try_into().expect("8")), 2);
        assert_eq!(i64::from_le_bytes(out[40..48].try_into().expect("8")), 1);
    }

    #[test]
    fn wire_refusals() {
        let mut out = vec![0u8; 16 + 8 + 8 * 16];
        let good = wire_in(RATIO_1_2, &[3, 3]);
        // trailing
        let mut t = good.clone();
        t.push(0);
        let code = unsafe {
            mgi_fractional_shares_compute(
                t.as_ptr(),
                t.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::ERR_TRAILING);
        // dims 0
        let zero = wire_in(RATIO_1_2, &[]);
        let code = unsafe {
            mgi_fractional_shares_compute(
                zero.as_ptr(),
                zero.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::ERR_DIM);
        // bad ratio reaches the core refusal
        let bad = wire_in(0, &[3]);
        let code = unsafe {
            mgi_fractional_shares_compute(
                bad.as_ptr(),
                bad.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, ProrateError::Ratio.code());
    }
}
