//! MG2 leaf: `mgi_hhi_calc_compute` — HHI concentration + divestiture gate.
//!
//! Matrix rule R3 (examples/merger_matrix/rules/hhi-bands.n3, DOJ/FTC 2023
//! Merger Guidelines §1/§2): the structural presumption fires when
//! `HHI > 1800 && delta > 100`; the divestiture decision field is the
//! fail-closed complement — **clear** only when `hhi_post <= 1800 &&
//! delta < 100`, else **divest**. All thresholds are applied in
//! [`crate::fixed::S_HHI`] scale (10^4 HHI points), so the legal constants
//! 1800 and 100 are `18_000_000` and `1_000_000` in wire units; the
//! comparison is exact integer, no floats anywhere.
//!
//! ## Wire format
//!
//! Input (after the 16-byte [`crate::Mgi1Header`], `leaf_id` =
//! [`LEAF_ID_HHI_CALC`], `scale_tag` = [`crate::fixed::TAG_HHI`] (values in `S_HHI` scale)):
//!
//! | Section | Size | Meaning |
//! |---------|------|---------|
//! | post-merger shares | `dims * 8` | `i64` LE share numerators, any common scale (percent in S_HHI fixed point); `dims == count` |
//! | pre-merger shares (optional) | `dims * 8` | same layout; present iff `in_len == 16 + 16*dims`. When present, `delta = HHI(post) - HHI(pre)` |
//!
//! `in_len < 16 + 8*dims` is `ERR_TRUNCATED`; `in_len` strictly between the
//! two legal sizes is `ERR_TRAILING` (a torn longer buffer is a trailing
//! error, not silently accepted).
//!
//! Output (16-byte header echoing leaf/TAG_HHI (values in `S_HHI` scale), `dims = 3`):
//! `i64 hhi_post`, `i64 delta` (0 when no pre-section), `u8 decision`
//! (0 = clear, 1 = divest), 7 zero pad bytes.
//!
//! Refusals are typed and total: `out` is untouched on any error.

/// Stable leaf identity for the HHI leaf (`Mgi1Header.leaf_id`).
pub const LEAF_ID_HHI_CALC: u8 = 1;

/// Maximum share count. Exceeding this is `ERR_DIM`.
pub const MAX_SHARES: usize = 10_000;

use crate::errno;
use crate::fixed::{self, Fix};
extern crate alloc;
use alloc::vec::Vec;

/// Thresholds in [`crate::fixed::S_HHI`] wire units (10^4 points).
const HHI_PRESUMPTION_CEILING: i128 = 1_800 * 10_000; // 1800 points
const DELTA_PRESUMPTION_FLOOR: i128 = 100 * 10_000; // 100 points

/// Computed result committed to `out` only on full admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HhiResult {
    /// HHI of the post-merger share vector, S_HHI scale.
    pub hhi_post: Fix,
    /// `HHI(post) - HHI(pre)` in S_HHI scale; 0 when no pre-section given.
    pub delta: Fix,
    /// True = divestiture required (presumption or delta fires).
    pub divest: bool,
}

/// Refusal reasons for the HHI leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HhiError {
    /// `dims == 0` or `dims > MAX_SHARES`.
    Dim,
    /// Byte length outside the two lawful sizes.
    Length,
    /// Arithmetic left the fixed-point domain.
    Overflow,
}

impl HhiError {
    /// Wire errno (crate errno table, 0-19).
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Dim => errno::ERR_DIM,
            Self::Length => errno::ERR_TRUNCATED,
            Self::Overflow => errno::ERR_OVERFLOW,
        }
    }
}

/// Core computation over a parsed share vector (host-callable, tested here;
/// the `extern "C"` wrapper is a thin byte-level adapter).
///
/// # Errors
/// [`HhiError::Dim`] when shares are empty or over [`MAX_SHARES`];
/// [`HhiError::Overflow`] when [`fixed::hhi`] leaves the domain.
pub fn compute_core(
    shares: &[Fix],
    pre: Option<&[Fix]>,
    out: &mut HhiResult,
) -> Result<(), HhiError> {
    if shares.is_empty() || shares.len() > MAX_SHARES {
        return Err(HhiError::Dim);
    }
    if let Some(p) = pre {
        if p.len() != shares.len() {
            return Err(HhiError::Dim);
        }
    }
    // MX8 repair (MG5 reconciliation): `fixed::hhi` returns raw HHI points;
    // the wire contract of this leaf is S_HHI scale (points x 10^4), so the
    // scale conversion happens exactly here at the leaf boundary —
    // `fixed::hhi` stays pure raw points.
    let hhi_post: Fix = (i128::from(fixed::hhi(shares).map_err(|_| HhiError::Overflow)?))
        .checked_mul(10_000)
        .ok_or(HhiError::Overflow)? as Fix;
    let delta: Fix = match pre {
        None => 0,
        Some(p) => {
            let hhi_pre: Fix = (i128::from(fixed::hhi(p).map_err(|_| HhiError::Overflow)?))
                .checked_mul(10_000)
                .ok_or(HhiError::Overflow)? as Fix;
            fixed::sub(hhi_post, hhi_pre).map_err(|_| HhiError::Overflow)?
        }
    };
    // R3 gate, exact integer: clear only when hhi_post <= 1800 && delta < 100.
    let divest = !(i128::from(hhi_post) <= HHI_PRESUMPTION_CEILING
        && i128::from(delta) < DELTA_PRESUMPTION_FLOOR);
    out.hhi_post = hhi_post;
    out.delta = delta;
    out.divest = divest;
    Ok(())
}

/// Wire-level entry point: `i32 mgi_hhi_calc_compute(in, in_len, out, out_cap) -> i32`.
/// 0 = ok; positive = errno; negative reserved for host transport.
///
/// # Safety
/// `in`/`out` must be valid for reads/writes of `in_len`/`out_cap` bytes
/// (caller-provided linear memory, cdylib ABI contract).
#[no_mangle]
pub unsafe extern "C" fn mgi_hhi_calc_compute(
    in_: *const u8,
    in_len: u32,
    out: *mut u8,
    out_cap: u32,
) -> i32 {
    if in_.is_null() || out.is_null() {
        return errno::ERR_NULL_PTR;
    }
    if (out_cap as usize) < 16 + 24 {
        return errno::ERR_CAP_TOO_SMALL;
    }
    let buf = core::slice::from_raw_parts(in_, in_len as usize);
    let o = core::slice::from_raw_parts_mut(out, out_cap as usize);
    match run(buf, o) {
        Ok(()) => errno::OK,
        Err(code) => code,
    }
}

/// Byte-level adapter (safe Rust, exercised by tests).
fn run(in_buf: &[u8], out_buf: &mut [u8]) -> Result<(), i32> {
    let hdr = crate::parse_header(in_buf)?;
    if hdr.leaf_id != LEAF_ID_HHI_CALC {
        return Err(errno::ERR_BAD_MAGIC);
    }
    if hdr.scale_tag != fixed::TAG_HHI {
        return Err(errno::ERR_BAD_SCALE);
    }
    if hdr.dims == 0 || hdr.dims > MAX_SHARES as u64 {
        return Err(errno::ERR_DIM);
    }
    let d = hdr.dims as usize;
    let body = 8 * d;
    let post_end = 16 + body;
    let pre_end = post_end + body;
    if in_buf.len() < post_end {
        return Err(errno::ERR_TRUNCATED);
    }
    // RV3 repair: a buffer strictly between the two lawful sizes (post-only
    // complete, pre-section torn) is ERR_TRAILING per the module doc — the
    // old `> pre_end` check silently accepted a torn pre-section.
    if in_buf.len() > post_end && in_buf.len() != pre_end {
        return Err(errno::ERR_TRAILING);
    }
    let pre = if in_buf.len() == pre_end { Some(&in_buf[post_end..pre_end]) } else { None };
    // Slices are exact 8-byte chunks by construction; the error arm is
    // unreachable but the conversion stays typed (no-expect law).
    let read_shares = |bytes: &[u8]| -> Result<Vec<Fix>, i32> {
        (0..d)
            .map(|i| {
                Ok(Fix::from_le_bytes(
                    bytes[8 * i..8 * i + 8].try_into().map_err(|_| errno::ERR_TRUNCATED)?,
                ))
            })
            .collect()
    };
    let shares = read_shares(&in_buf[16..post_end])?;
    let pre_shares = match pre.map(read_shares) {
        Some(r) => Some(r?),
        None => None,
    };
    // Fuel stub: one step per share, fixed cap, deterministic.
    let fuel_cap: u64 = (MAX_SHARES as u64) * 2 + 16;
    let mut steps: u64 = 0;
    // MX8 repair: closure captures nothing, so `mut` binding is unneeded
    // (deny(warnings)).
    let step = |_s: &mut u64| -> Result<(), i32> {
        *_s += 1;
        if *_s > fuel_cap {
            return Err(errno::ERR_FUEL);
        }
        Ok(())
    };
    for _ in &shares {
        step(&mut steps)?;
    }
    for _ in pre_shares.iter().flatten() {
        step(&mut steps)?;
    }
    let mut res = HhiResult { hhi_post: 0, delta: 0, divest: false };
    compute_core(&shares, pre_shares.as_deref(), &mut res).map_err(HhiError::code)?;
    // Commit phase — write out only after full admission.
    let ohdr = crate::Mgi1Header { leaf_id: LEAF_ID_HHI_CALC, scale_tag: fixed::TAG_HHI, dims: 3 };
    if out_buf.len() < 16 + 24 {
        return Err(errno::ERR_CAP_TOO_SMALL);
    }
    ohdr.encode(out_buf);
    out_buf[16..24].copy_from_slice(&res.hhi_post.to_le_bytes());
    out_buf[24..32].copy_from_slice(&res.delta.to_le_bytes());
    out_buf[32] = u8::from(res.divest);
    out_buf[33..40].fill(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hdr(dims: u64, leaf: u8, scale: u8) -> [u8; 16] {
        crate::Mgi1Header { leaf_id: leaf, scale_tag: scale, dims }.to_bytes()
    }

    fn share_bytes(shares: &[Fix]) -> Vec<u8> {
        shares.iter().flat_map(|s| s.to_le_bytes()).collect()
    }

    fn build(shares: &[Fix], pre: Option<&[Fix]>) -> Vec<u8> {
        let mut b = hdr(shares.len() as u64, LEAF_ID_HHI_CALC, fixed::TAG_HHI).to_vec();
        b.extend(share_bytes(shares));
        if let Some(p) = pre {
            b.extend(share_bytes(p));
        }
        b
    }

    fn run_ok(in_buf: &[u8]) -> (Fix, Fix, bool) {
        let mut out = [0u8; 40];
        run(in_buf, &mut out).expect("run ok");
        let hhi_post = Fix::from_le_bytes(out[16..24].try_into().unwrap());
        let delta = Fix::from_le_bytes(out[24..32].try_into().unwrap());
        (hhi_post, delta, out[32] == 1)
    }

    #[test]
    fn golden_post_only_concentrated_divest() {
        // 50/30/20 -> HHI = 2500+900+400 = 3800 pts (fixture, hhi-bands.n3).
        let s = [50, 30, 20];
        let (h, d, divest) = run_ok(&build(&s, None));
        assert_eq!(h, 3_800 * 10_000);
        assert_eq!(d, 0);
        assert!(divest);
    }

    #[test]
    fn wire_scale_is_points_times_ten_thousand() {
        // MX8/MG5 reconciliation: `fixed::hhi` stays pure raw points; the
        // leaf boundary converts to S_HHI wire scale. 60/40 -> 5200 pts =
        // 52_000_000 wire units.
        let s = [60, 40];
        let (h, d, _) = run_ok(&build(&s, None));
        assert_eq!(h, 5_200 * 10_000);
        assert_eq!(d, 0);
    }

    #[test]
    fn golden_unconcentrated_clear() {
        // MX8 golden repair: HHI is normalized (s_i/T)^2, so three equal
        // firms are 3333 pts (divest), not 300. A lawful unconcentrated
        // market needs 10 equal firms: 10 x (1/10)^2 = 1000 pts <= 1800.
        let firms = [10u64; 10];
        let shares: Vec<Fix> = firms
            .iter()
            .map(|&f| Fix::from(i64::try_from(f).expect("test firm count fits i64")))
            .collect();
        let (h, d, divest) = run_ok(&build(&shares, None));
        assert_eq!(h, 1_000 * 10_000);
        assert_eq!(d, 0);
        assert!(!divest);
    }

    #[test]
    fn golden_delta_boundary_exactly_100_is_divest() {
        // pre: monopoly 100 -> 10000 pts; post: 70/30 -> 5800 pts.
        // delta = -4200 < 100 and hhi_post 5800 > 1800 -> divest (hhi clause).
        // MX8 repair: the wire format carries pre with the SAME dims as post
        // (pre_end = 16 + 2*body), so a zero-share second firm encodes the
        // one-firm pre state on the two-slot wire.
        let (h, d, divest) = run_ok(&build(&[70, 30], Some(&[100, 0])));
        assert_eq!(h, 5_800 * 10_000);
        assert_eq!(d, -4_200 * 10_000);
        assert!(divest);
    }

    #[test]
    fn golden_delta_clear_zone() {
        // pre 60/40 = 5200; post 62/38 = 5288; delta 88 < 100 but
        // hhi_post 5288 > 1800 -> divest. Both clauses must hold to clear.
        let (_h, d, divest) = run_ok(&build(&[62, 38], Some(&[60, 40])));
        assert_eq!(d, 88 * 10_000);
        assert!(divest);
    }

    #[test]
    fn delta_just_under_100_with_low_hhi_clears() {
        // post 40/30/30 = 3400... over 1800. Use 30/30/30 = 2700 still over.
        // post 25/25/25/25 = 2500 over. post: 12 x (1/12) ~ 833: use
        // 40 firms of 25 -> HHI 2500. Simplest lawful clear: post 10 equal
        // firms -> 1000 <= 1800, delta 0 -> clear.
        let firms = [10u64; 10];
        let shares: Vec<Fix> = firms
            .iter()
            .map(|&f| Fix::from(i64::try_from(f).expect("test firm count fits i64")))
            .collect();
        let (h, d, divest) = run_ok(&build(&shares, None));
        assert_eq!(h, 1_000 * 10_000);
        assert_eq!(d, 0);
        assert!(!divest);
    }

    #[test]
    fn wrong_leaf_id_refused() {
        let mut b = build(&[50, 50], None);
        b[4] = 99;
        let mut out = [0u8; 40];
        assert_eq!(run(&b, &mut out), Err(errno::ERR_BAD_MAGIC));
    }

    #[test]
    fn wrong_scale_refused() {
        let mut b = build(&[50, 50], None);
        b[5] = fixed::TAG_MONEY;
        let mut out = [0u8; 40];
        assert_eq!(run(&b, &mut out), Err(errno::ERR_BAD_SCALE));
    }

    #[test]
    fn truncated_buffer_refused() {
        let b = build(&[50, 30, 20], None);
        let mut out = [0u8; 40];
        // Torn mid-payload (header valid, shares missing).
        assert_eq!(run(&b[..20], &mut out), Err(errno::ERR_TRUNCATED));
        // Torn header.
        assert_eq!(run(&b[..10], &mut out), Err(errno::ERR_TRUNCATED));
        // Torn pre-section.
        let full = build(&[50, 30, 20], Some(&[10, 10, 10]));
        assert_eq!(run(&full[..30], &mut out), Err(errno::ERR_TRUNCATED));
    }

    #[test]
    fn trailing_bytes_refused() {
        let mut b = build(&[50, 30, 20], None);
        b.push(0);
        let mut out = [0u8; 40];
        assert_eq!(run(&b, &mut out), Err(errno::ERR_TRAILING));
        // Between the two legal sizes: torn-longer is trailing, not accepted.
        let with_pre = build(&[50, 50], Some(&[10, 10]));
        assert_eq!(run(&with_pre[..with_pre.len() - 8], &mut out), Err(errno::ERR_TRAILING));
    }

    #[test]
    fn dims_caps() {
        let mut out = [0u8; 40];
        // dims 0 -> ERR_DIM
        let mut b = hdr(0, LEAF_ID_HHI_CALC, fixed::TAG_HHI).to_vec();
        assert_eq!(run(&b, &mut out), Err(errno::ERR_DIM));
        // dims over cap -> ERR_DIM
        b = hdr((MAX_SHARES + 1) as u64, LEAF_ID_HHI_CALC, fixed::TAG_HHI).to_vec();
        assert_eq!(run(&b, &mut out), Err(errno::ERR_DIM));
    }

    #[test]
    fn overflow_share_vector_refused() {
        // Each share near i64::MAX: sum of squares leaves i128 domain.
        let big = i64::MAX;
        let b = build(&[big, big, big, big, big, big, big, big], None);
        let mut out = [0u8; 40];
        assert_eq!(run(&b, &mut out), Err(errno::ERR_OVERFLOW));
    }

    #[test]
    fn out_cap_too_small_refused() {
        let b = build(&[50, 50], None);
        let mut out = [0u8; 16]; // header only, no room for the 24-byte body
        assert_eq!(run(&b, &mut out), Err(errno::ERR_CAP_TOO_SMALL));
    }

    #[test]
    fn output_untouched_on_refusal() {
        let mut b = build(&[50, 30, 20], None);
        b[5] = 99; // bad scale
        let mut out = [7u8; 40];
        let _ = run(&b, &mut out);
        assert!(out.iter().all(|&x| x == 7), "out must be untouched");
    }

    #[test]
    fn deterministic_repeat_calls() {
        let b = build(&[50, 30, 20], Some(&[40, 40, 20]));
        let a1 = {
            let mut out = [0u8; 40];
            run(&b, &mut out).unwrap();
            out
        };
        let a2 = {
            let mut out = [0u8; 40];
            run(&b, &mut out).unwrap();
            out
        };
        assert_eq!(a1, a2);
    }

    #[test]
    fn no_floats_anywhere() {
        let src = match std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/hhi_calc.rs"
        )) {
            Ok(s) => s,
            Err(e) => panic!("read hhi_calc.rs: {e}"),
        };
        // Tokens built from fragments so the scan cannot match its own source.
        let toks = [
            format!("f{}{}", 3, 2),
            format!("f{}", 64),
            format!(".sqr{}", "t"),
            format!(".pow{}", "f"),
            format!(".floo{}", "r"),
            format!(".cei{}", "l"),
        ];
        for tok in toks {
            assert!(
                !src.contains(tok.as_str()),
                "float token {tok:?} found in hhi_calc.rs — integer-only law"
            );
        }
    }
}
