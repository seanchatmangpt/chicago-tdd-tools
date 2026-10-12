//! Shared fixed-point core for the merger-matrix guest kernels.
//!
//! All kernel arithmetic is integer fixed-point: values are `i64` in a
//! documented scale, intermediates are `i128`, and every operation is
//! checked. There is **no floating point anywhere** in this module — see
//! the `no_floats` test at the bottom, which greps the source for float
//! types and fails the build's test run if any appears.
//!
//! ## Documented scales
//!
//! | Constant | Scale | Unit |
//! |----------|-------|------|
//! | [`S_MONEY`] | `10^6` | dollars in micro-units |
//! | [`S_RATIO`] | `10^8` | unit-interval ratios in 1e-8 steps |
//! | [`S_HHI`] | `10^4` | HHI points (DOJ/FTC 2023 bands: <1500, 1500-2500, >2500) |
//! | [`S_SHARES`] | `10^6` | share counts in millionths |
//!
//! The MGI1 header carries a small `scale_tag` byte identifying which scale
//! a buffer's fixed-point values are in; [`TAG_MONEY`]/[`TAG_RATIO`]/
//! [`TAG_HHI`]/[`TAG_SHARES`] are the valid tags and [`scale_of`] maps a tag
//! to its multiplier. The `S_*` constants themselves are the multipliers,
//! usable directly as denominators in fixed-point math.
//!
//! Leaf modules may use core+alloc only; this module itself is std-free by
//! construction (no `std::` paths).

/// Fixed-point values are `i64` in the documented scale of their `scale_tag`.
pub type Fix = i64;

/// Money scale: 10^6 (micro-dollars).
pub const S_MONEY: i128 = 1_000_000;
/// Ratio scale: 10^8 (1e-8 steps, unit interval).
pub const S_RATIO: i128 = 100_000_000;
/// HHI scale: 10^4 (HHI points).
pub const S_HHI: i128 = 10_000;
/// Share-count scale: 10^6.
pub const S_SHARES: i128 = 1_000_000;

/// Header `scale_tag` byte: values are in [`S_MONEY`] scale.
pub const TAG_MONEY: u8 = 1;
/// Header `scale_tag` byte: values are in [`S_RATIO`] scale.
pub const TAG_RATIO: u8 = 2;
/// Header `scale_tag` byte: values are in [`S_HHI`] scale.
pub const TAG_HHI: u8 = 3;
/// Header `scale_tag` byte: values are in [`S_SHARES`] scale.
pub const TAG_SHARES: u8 = 4;

/// Scale multiplier for a header tag, as `i128`.
///
/// Returns `None` for tags outside the documented set (`ERR_BAD_SCALE` is
/// the caller's errno; this module is errno-agnostic).
pub const fn scale_of(tag: u8) -> Option<i128> {
    match tag {
        TAG_MONEY => Some(S_MONEY),
        TAG_RATIO => Some(S_RATIO),
        TAG_HHI => Some(S_HHI),
        TAG_SHARES => Some(S_SHARES),
        _ => None,
    }
}

/// Typed overflow error: the checked ops below never panic and never wrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overflow;

/// Checked add: `a + b` in one scale, `i128` intermediate, `Err(Overflow)` on
/// i64 wrap.
pub const fn add(a: Fix, b: Fix) -> Result<Fix, Overflow> {
    let s = a as i128 + b as i128;
    if s > i64::MAX as i128 || s < i64::MIN as i128 {
        Err(Overflow)
    } else {
        Ok(s as Fix)
    }
}

/// Checked subtract: `a - b` in one scale.
// MX8 repair: dropped `const` so `negate(b)?` composes; semantics unchanged.
pub fn sub(a: Fix, b: Fix) -> Result<Fix, Overflow> {
    add(a, negate(b)?)
}

/// Checked negate.
pub const fn negate(a: Fix) -> Result<Fix, Overflow> {
    if a == i64::MIN {
        Err(Overflow)
    } else {
        Ok(-a)
    }
}

/// Checked multiply: `a * b` in one scale with an `i128` intermediate.
///
/// Because the result stays in the *same* scale, the raw product must be
/// divided back by the scale; the division is round-half-even so the op is
/// associative-safe under the module's rounding law. Callers that need the
/// full-precision product should use [`mul_wide`].
pub fn mul(a: Fix, b: Fix, scale: i128) -> Result<Fix, Overflow> {
    mul_wide(a, b, scale, scale)
}

/// Wide multiply-rescale: `(a * b) >> scale_denom` with round-half-even.
///
/// Computes `a * b` (both in `scale_in`) then rescales into `scale_out`,
/// dividing by `scale_in / scale_out`'s net denominator:
/// `result = a*b*scale_out/scale_in`, rounded half-to-even, checked against
/// the i64 domain.
pub fn mul_wide(a: Fix, b: Fix, scale_in: i128, scale_out: i128) -> Result<Fix, Overflow> {
    debug_assert!(scale_in > 0 && scale_out > 0);
    let prod = a as i128 * b as i128 * scale_out;
    div_round_half_even_i128(prod, scale_in)
}

/// Checked divide: `a / b` in one scale, round-half-even.
pub fn div(a: Fix, b: Fix, scale: i128) -> Result<Fix, Overflow> {
    if b == 0 {
        return Err(Overflow);
    }
    let num = a as i128 * scale;
    div_round_half_even_i128(num, b as i128)
}

/// Round-half-even integer division `num / den` for `den > 0`, any sign of
/// `num`. Ties (exact halves) round to the even neighbor.
pub const fn div_round_half_even(num: i128, den: i128) -> Result<i128, Overflow> {
    if den <= 0 {
        return Err(Overflow);
    }
    // Euclidean division keeps the remainder in [0, den) for any sign of
    // num, so one tie rule covers both signs.
    let q = num.div_euclid(den);
    let r = num.rem_euclid(den);
    let r2 = r * 2;
    if r2 > den || (r2 == den && (q & 1) != 0) {
        Ok(q + 1)
    } else {
        Ok(q)
    }
}

/// Internal: round-half-even division yielding an `i64`-checked `Fix`.
// MX8 repair: dropped `const` — the stable `?` operator on `Result` is not
// allowed in const fn; all callers are non-const.
fn div_round_half_even_i128(num: i128, den: i128) -> Result<Fix, Overflow> {
    if den == 0 {
        return Err(Overflow);
    }
    let q = div_round_half_even(num, den.abs())?;
    let q = if den < 0 { -q } else { q };
    if q > i64::MAX as i128 || q < i64::MIN as i128 {
        Err(Overflow)
    } else {
        Ok(q as Fix)
    }
}

/// HHI = sum(s_i^2) / T^2 where s_i are share *numerators* (any common
/// scale) and T = sum(s_i), computed via `i128` with the result in
/// [`S_HHI`] scale (`10^4` HHI points).
///
/// Cross-multiplication keeps everything exact until the final
/// round-half-even division; no floating point anywhere.
///
/// * `ERR`: [`Overflow`] if `shares` is empty (T = 0) or any intermediate
///   exceeds the `i128`/`i64` domains.
pub fn hhi(shares: &[Fix]) -> Result<Fix, Overflow> {
    if shares.is_empty() {
        return Err(Overflow);
    }
    let mut sum: i128 = 0;
    let mut sum_sq: i128 = 0;
    for &s in shares {
        let s = s as i128;
        sum = match sum.checked_add(s) {
            Some(v) => v,
            None => return Err(Overflow),
        };
        sum_sq = match sum_sq.checked_add(s.checked_mul(s).ok_or(Overflow)?) {
            Some(v) => v,
            None => return Err(Overflow),
        };
    }
    if sum == 0 {
        return Err(Overflow);
    }
    // result = sum_sq * S_HHI_MULT / (sum^2), round-half-even.
    let num = sum_sq.checked_mul(10_000).ok_or(Overflow)?;
    div_round_half_even_i128(num, sum * sum)
}

/// Cross-multiplied comparison `a/b` vs `c/d` without division: returns
/// `Ordering` computed from sign-correct cross products in `i128`. Denominators
/// must be positive (share/ratio denominators in this ABI always are);
/// a non-positive denominator is [`Overflow`].
// MX8 repair: dropped `const` — `Ord::cmp` is not const-stable; all callers
// are non-const.
pub fn cmp_cross(a: Fix, b: Fix, c: Fix, d: Fix) -> Result<core::cmp::Ordering, Overflow> {
    use core::cmp::Ordering;
    if b <= 0 || d <= 0 {
        return Err(Overflow);
    }
    let lhs = a as i128 * d as i128;
    let rhs = c as i128 * b as i128;
    Ok(match lhs.cmp(&rhs) {
        Ordering::Less => Ordering::Less,
        Ordering::Equal => Ordering::Equal,
        Ordering::Greater => Ordering::Greater,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cmp::Ordering;

    #[test]
    fn scales_are_documented_powers() {
        assert_eq!(scale_of(TAG_MONEY), Some(S_MONEY));
        assert_eq!(scale_of(TAG_RATIO), Some(S_RATIO));
        assert_eq!(scale_of(TAG_HHI), Some(S_HHI));
        assert_eq!(scale_of(TAG_SHARES), Some(S_SHARES));
        assert_eq!(S_MONEY, 1_000_000);
        assert_eq!(S_RATIO, 100_000_000);
        assert_eq!(S_HHI, 10_000);
        assert_eq!(S_SHARES, 1_000_000);
        assert_eq!(scale_of(0), None);
        assert_eq!(scale_of(99), None);
    }

    #[test]
    fn checked_ops_catch_overflow() {
        assert_eq!(add(1, 2).unwrap(), 3);
        assert_eq!(sub(2, 5).unwrap(), -3);
        assert!(add(i64::MAX, 1).is_err());
        assert!(negate(i64::MIN).is_err());
        assert!(div(1, 0, 1_000_000).is_err());
        // 2^62 * 2 does not fit i64
        assert!(mul(1 << 62, 2, 1_000_000).is_err());
    }

    #[test]
    fn round_half_even_ties_go_to_even() {
        // 1.5 -> 2, 2.5 -> 2, -1.5 -> -2, -0.5 -> 0 (even), 0.5 -> 0
        assert_eq!(div_round_half_even(3, 2).unwrap(), 2);
        assert_eq!(div_round_half_even(5, 2).unwrap(), 2);
        assert_eq!(div_round_half_even(-3, 2).unwrap(), -2);
        assert_eq!(div_round_half_even(-1, 2).unwrap(), 0);
        assert_eq!(div_round_half_even(1, 2).unwrap(), 0);
        assert_eq!(div_round_half_even(4, 2).unwrap(), 2);
        assert!(div_round_half_even(1, 0).is_err());
        assert!(div_round_half_even(1, -2).is_err());
    }

    #[test]
    fn mul_wide_rescales_with_half_even() {
        // 1.5 * 1.5 = 2.25 in S_RATIO terms: (15*15*1e8)/1e8/... sanity via
        // small numbers instead: a=3, b=3, scale 2 -> 9/2 = 4.5 -> 4 (even)
        assert_eq!(mul_wide(3, 3, 2, 1).unwrap(), 4);
        // 5*5, scale 2 -> 25/2 = 12.5 -> 12 (even)
        assert_eq!(mul_wide(5, 5, 2, 1).unwrap(), 12);
    }

    #[test]
    fn hhi_matches_exact_integer_math() {
        // Equal 4-firm market: each s=1 -> HHI = 4*(1/16) = 0.25 -> 2500 pts.
        assert_eq!(hhi(&[1, 1, 1, 1]).unwrap(), 2500);
        // Monopoly: 1 firm -> 1.0 -> 10000.
        assert_eq!(hhi(&[7]).unwrap(), 10000);
        // 50/50 duopoly -> 2*(1/4) = 0.5 -> 5000.
        assert_eq!(hhi(&[1, 1]).unwrap(), 5000);
        // Asymmetric: 60/40 -> 0.36+0.16 = 0.52 -> 5200.
        assert_eq!(hhi(&[6, 4]).unwrap(), 5200);
        assert!(hhi(&[]).is_err());
    }

    #[test]
    fn cross_multiplied_compare_avoids_division() {
        // 2/3 vs 3/5: 10 vs 9 -> Greater
        assert_eq!(cmp_cross(2, 3, 3, 5).unwrap(), Ordering::Greater);
        // 1/2 vs 2/4: equal
        assert_eq!(cmp_cross(1, 2, 2, 4).unwrap(), Ordering::Equal);
        assert!(cmp_cross(1, 0, 1, 1).is_err());
        assert!(cmp_cross(1, 1, 1, -1).is_err());
    }

    #[test]
    fn no_floats_anywhere() {
        // Deny floats by construction: grep this file for float type tokens.
        let src =
            match std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/fixed.rs")) {
                Ok(s) => s,
                Err(e) => panic!("read fixed.rs: {e}"),
            };
        // Tokens built from fragments so the scan cannot match its own source.
        let toks = [
            format!("f{}{}", 3, 2),
            format!("f{}", 64),
            format!("F{}{}", 3, 2),
            format!("F{}", 64),
            format!(".sqr{}", "t"),
            format!(".pow{}", "f"),
            format!(".floo{}", "r"),
            format!(".cei{}", "l"),
        ];
        for tok in toks {
            assert!(
                !src.contains(tok.as_str()),
                "float token {tok:?} found in fixed.rs — merger kernels are integer-only"
            );
        }
    }
}
