//! MG3 leaf: `mgi_simm_matrix_compute` — SIMM matrix kernel (dense `C · W`
//! matrix-vector product).
//!
//! **Honestly scoped** (E-M13): a dense matrix-vector kernel over checked
//! fixed-point, NOT the full ISDA SIMM calculation. Real SIMM adds bucket
//! aggregation, correlation structure, and regulatory curvature terms; this
//! leaf computes exactly the dense product `y = C · w` (class-level
//! exposures `C`, risk-class weight vector `w`) with typed dimension
//! refusals and checked `i128` accumulation. It is the arithmetic core a
//! full SIMM implementation would call; it is not SIMM.
//!
//! ## Wire format
//!
//! Input (after the 16-byte Mgi1Header, `leaf_id` =
//! `LEAF_ID_SIMM_MATRIX`, any documented `scale_tag`; the scale applies to
//! `C`, `w`, and `y` alike):
//!
//! | Section | Size | Meaning |
//! |---------|------|---------|
//! | rows | 8 | `u64` LE row count, `ERR_DIM` above `MAX_ROWS` |
//! | cols | 8 | `u64` LE column count, `ERR_DIM` above `MAX_COLS` |
//! | matrix | `rows*cols*8` | row-major `i64` LE, same scale as `w` |
//! | vector | `cols*8` | `i64` LE weight vector `w` |
//!
//! Header `dims` law: `dims = 2 + rows*cols + cols` (payload element
//! count). `in_len < 16 + dims*8` is `ERR_TRUNCATED`; `in_len >` that is
//! `ERR_TRAILING`. `rows == 0` or `cols == 0` is `ERR_DIM`.
//!
//! Output (16-byte header echoing leaf/scale, `dims = rows`): `rows * 8`
//! bytes of `i64` LE results `y[i]`, same scale as the inputs.
//!
//! Determinism: accumulation is row-major with `k` ascending —
//! bit-deterministic across runs. No floats anywhere (`no_floats` test).

/// Stable leaf identity for the SIMM matrix leaf (`Mgi1Header.leaf_id`).
pub const LEAF_ID_SIMM_MATRIX: u8 = 3;

/// Maximum matrix rows (`ERR_DIM` above this).
pub const MAX_ROWS: usize = 256;

/// Maximum matrix columns (`ERR_DIM` above this).
pub const MAX_COLS: usize = 64;

use crate::errno;
use crate::fixed::{self, Fix};
extern crate alloc;
use alloc::vec::Vec;

/// Dense fixed-point matrix-vector product `y = C · w`, same-scale output.
///
/// * `rows`/`cols` — declared dimensions (capped by `MAX_ROWS`/`MAX_COLS`,
///   both >= 1).
/// * `matrix` — row-major `rows x cols`, same scale as `vec`.
/// * `vec` — length `cols`, same scale as `matrix`.
/// * `scale_mult` — shared scale multiplier (resolve a scale tag once via
///   `fixed::scale_of(tag)`, e.g. S_RATIO to 10^8).
///
/// Errors: `ERR_DIM` (dimension/length refusal), `ERR_OVERFLOW` (checked
/// i128 domain). Never panics, never wraps.
fn matvec(
    rows: usize,
    cols: usize,
    matrix: &[Fix],
    vec: &[Fix],
    scale_mult: i128,
) -> Result<Vec<Fix>, i32> {
    dim_guard(rows, cols, matrix, vec)?;
    let mut out = Vec::with_capacity(rows);
    for r in 0..rows {
        let row = &matrix[r * cols..(r + 1) * cols];
        let mut acc: i128 = 0;
        for k in 0..cols {
            let p = match (row[k] as i128).checked_mul(vec[k] as i128) {
                Some(v) => v,
                None => return Err(errno::ERR_OVERFLOW),
            };
            acc = match acc.checked_add(p) {
                Some(v) => v,
                None => return Err(errno::ERR_OVERFLOW),
            };
        }
        // Products are scale^2; one rescale returns to the input scale.
        let y = match fixed::div_round_half_even(acc, scale_mult) {
            Ok(v) => v,
            Err(_) => return Err(errno::ERR_OVERFLOW),
        };
        let y = match Fix::try_from(y) {
            Ok(v) => v,
            Err(_) => return Err(errno::ERR_OVERFLOW),
        };
        out.push(y);
    }
    Ok(out)
}

/// Dimension/length refusal (`ERR_DIM`), checked before any arithmetic.
fn dim_guard(rows: usize, cols: usize, matrix: &[Fix], vec: &[Fix]) -> Result<(), i32> {
    if rows == 0 || rows > MAX_ROWS || cols == 0 || cols > MAX_COLS {
        return Err(errno::ERR_DIM);
    }
    if matrix.len() != rows.saturating_mul(cols) {
        return Err(errno::ERR_DIM);
    }
    if vec.len() != cols {
        return Err(errno::ERR_DIM);
    }
    Ok(())
}

/// Byte-level adapter (safe Rust, exercised by tests).
fn run(in_buf: &[u8], out_buf: &mut [u8]) -> Result<(), i32> {
    let hdr = crate::parse_header(in_buf)?;
    if hdr.leaf_id != LEAF_ID_SIMM_MATRIX {
        return Err(errno::ERR_BAD_MAGIC);
    }
    let scale_mult = match fixed::scale_of(hdr.scale_tag) {
        Some(s) => s,
        None => return Err(errno::ERR_BAD_SCALE),
    };
    if in_buf.len() < 32 {
        return Err(errno::ERR_TRUNCATED);
    }
    let rows = u64::from_le_bytes(match in_buf[16..24].try_into() {
        Ok(b) => b,
        Err(_) => return Err(errno::ERR_TRUNCATED),
    });
    let cols = u64::from_le_bytes(match in_buf[24..32].try_into() {
        Ok(b) => b,
        Err(_) => return Err(errno::ERR_TRUNCATED),
    });
    if rows == 0 || rows > MAX_ROWS as u64 || cols == 0 || cols > MAX_COLS as u64 {
        return Err(errno::ERR_DIM);
    }
    let expected_dims: u64 = match rows.checked_mul(cols).and_then(|v| v.checked_add(2)) {
        Some(v) => match v.checked_add(cols) {
            Some(w) => w,
            None => return Err(errno::ERR_DIM),
        },
        None => return Err(errno::ERR_DIM),
    };
    if hdr.dims != expected_dims {
        return Err(errno::ERR_DIM);
    }
    let (r, c) = (rows as usize, cols as usize);
    let end = 16 + (2 + r * c + c) * 8;
    if in_buf.len() < end {
        return Err(errno::ERR_TRUNCATED);
    }
    if in_buf.len() > end {
        return Err(errno::ERR_TRAILING);
    }
    let mut matrix: Vec<Fix> = Vec::with_capacity(r * c);
    for chunk in in_buf[32..32 + r * c * 8].chunks_exact(8) {
        let mut b = [0u8; 8];
        b.copy_from_slice(chunk);
        matrix.push(Fix::from_le_bytes(b));
    }
    let mut vec: Vec<Fix> = Vec::with_capacity(c);
    for chunk in in_buf[32 + r * c * 8..end].chunks_exact(8) {
        let mut b = [0u8; 8];
        b.copy_from_slice(chunk);
        vec.push(Fix::from_le_bytes(b));
    }
    let y = matvec(r, c, &matrix, &vec, scale_mult)?;
    // Commit phase: write out only after full admission.
    if out_buf.len() < 16 + r * 8 {
        return Err(errno::ERR_CAP_TOO_SMALL);
    }
    let ohdr = crate::Mgi1Header {
        leaf_id: LEAF_ID_SIMM_MATRIX,
        scale_tag: hdr.scale_tag,
        dims: r as u64,
    };
    ohdr.encode(out_buf);
    for (i, v) in y.iter().enumerate() {
        out_buf[16 + 8 * i..16 + 8 * (i + 1)].copy_from_slice(&v.to_le_bytes());
    }
    Ok(())
}

/// Wire-level entry point. 0 = ok; positive = errno; negative reserved for
/// host transport.
///
/// # Safety
/// `in`/`out` must be valid for reads/writes of `in_len`/`out_cap` bytes
/// (caller-provided linear memory, cdylib ABI contract).
#[no_mangle]
pub unsafe extern "C" fn mgi_simm_matrix_compute(
    in_: *const u8,
    in_len: u32,
    out: *mut u8,
    out_cap: u32,
) -> i32 {
    if in_.is_null() || out.is_null() {
        return errno::ERR_NULL_PTR;
    }
    if (out_cap as usize) < 16 {
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
    use crate::fixed::{scale_of, TAG_RATIO};

    const SC: i128 = 100_000_000; // S_RATIO = 10^8

    fn s(v: i64) -> Fix {
        (v as i128 * SC) as Fix
    }

    fn build_input(rows: usize, cols: usize, matrix: &[Fix], vec: &[Fix], tag: u8) -> Vec<u8> {
        let dims = 2 + rows * cols + cols;
        let mut b =
            crate::Mgi1Header { leaf_id: LEAF_ID_SIMM_MATRIX, scale_tag: tag, dims: dims as u64 }
                .to_bytes()
                .to_vec();
        b.extend_from_slice(&(rows as u64).to_le_bytes());
        b.extend_from_slice(&(cols as u64).to_le_bytes());
        b.extend_from_slice(&share_bytes(matrix));
        b.extend_from_slice(&share_bytes(vec));
        b
    }

    fn share_bytes(vals: &[Fix]) -> Vec<u8> {
        vals.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    #[test]
    fn scale_tag_resolves_to_test_constant() {
        assert_eq!(scale_of(TAG_RATIO), Some(SC));
    }

    #[test]
    fn identity_passes_through_exactly() {
        let m = [s(1), 0, 0, 0, s(1), 0, 0, 0, s(1)];
        let v = [s(-7), s(13), s(42)];
        let y = matvec(3, 3, &m, &v, SC).unwrap();
        assert_eq!(y, vec![s(-7), s(13), s(42)]);
    }

    #[test]
    fn two_by_two_hand_computed() {
        // [[0.5, 0], [0, 0.25]] dot [4, 8] = [2, 2] (exact in S_RATIO).
        let m = [s(1) / 2, 0, 0, s(1) / 4];
        let v = [s(4), s(8)];
        let y = matvec(2, 2, &m, &v, SC).unwrap();
        assert_eq!(y, vec![s(2), s(2)]);
    }

    #[test]
    fn three_by_three_hand_computed() {
        // [[1,2,3],[4,5,6],[7,8,9]] dot [1,1,1] = [6,15,24] — raw integer
        // entries in a unit scale (scale_mult = 1) exercise accumulation.
        let m: [Fix; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let v: [Fix; 3] = [1, 1, 1];
        let y = matvec(3, 3, &m, &v, 1).unwrap();
        assert_eq!(y, vec![6, 15, 24]);
    }

    #[test]
    fn rounding_is_half_even_when_not_divisible() {
        // Row [1, 1] dot [1, 2] = 3, scale 2: 3/2 = 1.5 rounds to 2 (even).
        let m: [Fix; 2] = [1, 1];
        let v: [Fix; 2] = [1, 2];
        let y = matvec(1, 2, &m, &v, 2).unwrap();
        assert_eq!(y, vec![2]);
    }

    #[test]
    fn dim_refusals_are_typed_and_total() {
        let ok_m: [Fix; 4] = [1, 0, 0, 1];
        let ok_v: [Fix; 2] = [1, 1];
        assert_eq!(matvec(MAX_ROWS + 1, 1, &[], &[1], 1), Err(errno::ERR_DIM));
        assert_eq!(matvec(1, MAX_COLS + 1, &[], &[], 1), Err(errno::ERR_DIM));
        assert_eq!(matvec(2, 2, &ok_m[..3], &ok_v, 1), Err(errno::ERR_DIM));
        assert_eq!(matvec(2, 2, &ok_m, &ok_v[..1], 1), Err(errno::ERR_DIM));
        assert_eq!(matvec(0, 2, &[], &[], 1), Err(errno::ERR_DIM));
        assert_eq!(matvec(2, 0, &ok_m, &[], 1), Err(errno::ERR_DIM));
    }

    #[test]
    fn overflow_refuses_instead_of_wrapping() {
        let big = Fix::MAX;
        let m = [big, big];
        let v = [big, big];
        assert_eq!(matvec(1, 2, &m, &v, 1), Err(errno::ERR_OVERFLOW));
    }

    #[test]
    fn wire_roundtrip_identity_case() {
        let m = [s(1), 0, 0, s(1)];
        let v = [s(3), s(-4)];
        let input = build_input(2, 2, &m, &v, fixed::TAG_RATIO);
        let mut out = vec![0u8; 16 + 16];
        assert_eq!(run(&input, &mut out), Ok(()));
        assert_eq!(&out[0..4], b"MGI1");
        assert_eq!(out[4], LEAF_ID_SIMM_MATRIX);
        let y0 = Fix::from_le_bytes(out[16..24].try_into().unwrap());
        let y1 = Fix::from_le_bytes(out[24..32].try_into().unwrap());
        assert_eq!([y0, y1], [s(3), s(-4)]);
    }

    #[test]
    fn wire_refusals() {
        let m: [Fix; 4] = [1, 0, 0, 1];
        let v: [Fix; 2] = [1, 1];
        // wrong leaf id
        let bad_leaf = crate::Mgi1Header { leaf_id: 1, scale_tag: fixed::TAG_RATIO, dims: 8 }
            .to_bytes()
            .to_vec();
        let mut input = bad_leaf;
        input.extend_from_slice(&2u64.to_le_bytes());
        input.extend_from_slice(&2u64.to_le_bytes());
        input.extend_from_slice(&share_bytes(&m));
        input.extend_from_slice(&share_bytes(&v));
        let mut out = [0u8; 64];
        assert_eq!(run(&input, &mut out), Err(errno::ERR_BAD_MAGIC));
        // bad scale tag
        let input = build_input(2, 2, &m, &v, 99);
        assert_eq!(run(&input, &mut out), Err(errno::ERR_BAD_SCALE));
        // trailing bytes
        let mut input = build_input(2, 2, &m, &v, fixed::TAG_RATIO);
        input.push(0);
        assert_eq!(run(&input, &mut out), Err(errno::ERR_TRAILING));
        // truncated
        let input = build_input(2, 2, &m, &v, fixed::TAG_RATIO);
        assert_eq!(run(&input[..input.len() - 8], &mut out), Err(errno::ERR_TRUNCATED));
        // rows over cap in payload
        let input = build_input(MAX_ROWS + 1, 1, &[], &[], fixed::TAG_RATIO);
        assert_eq!(run(&input, &mut out), Err(errno::ERR_DIM));
    }

    #[test]
    fn determinism_two_runs_byte_identical() {
        let m: Vec<Fix> = (0..64).map(|i| s((i % 7) as i64 - 3)).collect();
        let v: Vec<Fix> = (0..8).map(|i| s(i as i64 + 1)).collect();
        let input = build_input(8, 8, &m, &v, fixed::TAG_RATIO);
        let mut a = vec![0u8; 16 + 64];
        let mut b = vec![0u8; 16 + 64];
        assert_eq!(run(&input, &mut a), Ok(()));
        assert_eq!(run(&input, &mut b), Ok(()));
        assert_eq!(a, b);
    }

    #[test]
    fn no_floats_anywhere() {
        // Tokens are assembled at runtime so this test does not contain its
        // own needles (a literal needle here would self-match).
        let toks: [String; 8] = [
            format!("f{}", 32),
            format!("f{}", 64),
            format!("{}qrt", ".s"),
            format!("pow{}", "f"),
            format!("{}loor", ".f"),
            format!("{}eil", ".c"),
            format!("F{}", 32),
            format!("F{}", 64),
        ];
        let src =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/simm_matrix.rs"))
                .unwrap();
        for tok in &toks {
            assert!(
                !src.contains(tok.as_str()),
                "float token {tok:?} found in simm_matrix.rs — merger kernels are integer-only"
            );
        }
    }
}
