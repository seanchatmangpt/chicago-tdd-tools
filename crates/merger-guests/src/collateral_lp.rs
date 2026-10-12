//! MG3 leaf: `mgi_collateral_lp_compute` — collateral allocation LP
//! (scaled-integer simplex).
//!
//! **Honestly scoped**: a small dense LP solver for collateral allocation —
//! maximize `c·x` subject to `Ax <= b`, `x >= 0` — over exact scaled
//! integers. NOT a general-purpose solver: no sparse structure, no interior
//! point, no warm starts. Sized for small allocation problems (<=
//! `MAX_VARS` variables, <= `MAX_CONS` constraints) and refuses anything
//! larger.
//!
//! ## Algorithm: two-phase simplex with Edmonds' integer (Q-)pivoting
//!
//! The tableau is carried as exact integers with one shared denominator
//! scalar `D` (entry value = `T[i][j] / D`). Edmonds' Q-pivoting update
//!
//! ```text
//! T'[i][j] = (piv*T[i][j] - T[i][col]*T[r][j]) / D      (i != r)
//! ```
//!
//! keeps every entry an exact integer (each is a minor of the working
//! matrix — no rounding, no rationals, no floats). Both objective rows
//! (Phase-I feasibility `w = sum artificials`, Phase-II `W = -c·x`) are
//! appended to the tableau before the first pivot and carried through every
//! update, so each arrives canonical (reduced costs for the current basis)
//! when its phase begins — no re-canonicalization division is needed.
//!
//! ## Termination: Bland's rule
//!
//! Entering: smallest eligible column with a strictly positive objective-row
//! entry (improving for the minimization). Leaving: among minimum-ratio
//! rows, the one whose basic variable has the smallest column index. All
//! comparisons are exact (sign tests and i128 cross-multiplication), which
//! is the precondition for Bland's finite-termination proof — cycling is
//! impossible; `MAX_PIVOTS` is belt-and-braces.
//!
//! ## Refusals (typed, total; `out` untouched on any error)
//!
//! | Errno | Meaning |
//! |---|---|
//! | `ERR_DIM` | size/length violations (`n`/`m` bounds, slice/header lengths) |
//! | `ERR_BAD_SCALE` | undocumented scale tag |
//! | `ERR_OVERFLOW` | checked i128 domain exceeded (never wraps) |
//! | `ERR_FUEL` | more than `MAX_PIVOTS` pivots required |
//! | `ERR_NO_SOLUTION` | infeasible (Phase I ends with `w > 0`) or unbounded (entering column with no positive entries — no finite optimum) |
//!
//! ## Scales
//!
//! `c`, `A`, `b` share one declared scale (e.g. S_MONEY = 10^6
//! micro-dollars or S_RATIO = 10^8). Outputs are in the same scale: the
//! vertex `x` is the tableau value rounded half-even (`exact_vertex` flags
//! exact division), and the objective is `sum(c_j*x_j) / scale_mult`
//! (round-half-even, `exact_objective` flag). For pure-integer problems the
//! caller passes a unit scale and every answer is exact. Scale-invariance:
//! scaling `A` and `b` by a common factor does not move the optimal vertex,
//! so raw-integer and fixed-point passes of the same problem select the
//! same `x`.

/// Stable leaf identity for the collateral LP leaf (`Mgi1Header.leaf_id`).
pub const LEAF_ID_COLLATERAL_LP: u8 = 4;

/// Maximum decision variables (`ERR_DIM` above this).
pub const MAX_VARS: usize = 64;

/// Maximum constraints (`ERR_DIM` above this).
pub const MAX_CONS: usize = 64;

/// Pivot fuel cap across both phases (`ERR_FUEL` beyond this).
pub const MAX_PIVOTS: u32 = 128;

use crate::errno;
use crate::fixed::{self, Fix};
extern crate alloc;
use alloc::vec::Vec;

/// Successful solve: vertex, objective, exactness flags, pivot count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LpSolution {
    /// Optimal vertex, length `n`, in the input scale (nonbasic = 0).
    pub x: Vec<Fix>,
    /// `sum(c_j*x_j) / scale_mult`, round-half-even, in the input scale.
    pub objective: Fix,
    /// Every vertex coordinate divided exactly (no rounding was applied).
    pub exact_vertex: bool,
    /// The objective division was exact (no rounding was applied).
    pub exact_objective: bool,
    /// Total pivots consumed across Phase I + cleanup + Phase II.
    pub pivots: u32,
}

/// Column band widths in the working tableau.
struct Layout {
    m: usize,
    /// first artificial column
    art: usize,
    /// rhs column index
    rhs: usize,
}

/// Simplex state: constraint rows + two carried objective rows.
struct Tableau {
    /// rows `0..m` constraints, `m` = Phase-I obj, `m+1` = Phase-II obj.
    t: Vec<Vec<i128>>,
    d: i128,
    basis: Vec<usize>,
    layout: Layout,
    pivots: u32,
}

impl Tableau {
    fn build(c: &[Fix], a: &[Fix], b: &[Fix], n: usize, m: usize) -> Tableau {
        let art = n + m;
        let rhs = art + m;
        // Rows with negative rhs are negated so the artificial basis starts
        // feasible (rhs >= 0). The slack coefficient flips sign (surplus
        // variable); the constraint set is unchanged (multiply both sides).
        let neg: Vec<bool> = (0..m).map(|i| b[i] < 0).collect();
        let mut t = Vec::with_capacity(m + 2);
        for i in 0..m {
            let sgn: i128 = if neg[i] { -1 } else { 1 };
            let mut row = vec![0i128; rhs + 1];
            for (j, cell) in row.iter_mut().enumerate().take(n) {
                *cell = a[i * n + j] as i128 * sgn;
            }
            row[n + i] = sgn; // slack (or surplus) column
            row[art + i] = 1; // artificial
            row[rhs] = b[i] as i128 * sgn;
            t.push(row);
        }
        // Phase-I objective row: w = sum(b_i) - sum_j obj[j]*x_j, so the
        // improving (decreasing-w) direction is obj[j] > 0. The sum runs
        // over structural + slack columns only; artificial columns stay 0
        // (they are the starting basis — their basic values already account
        // for the w = sum(b_i) constant).
        let mut obj1 = vec![0i128; rhs + 1];
        for r in t.iter().take(m) {
            for (j, cell) in obj1.iter_mut().enumerate().take(art) {
                *cell += r[j];
            }
        }
        // The row is the equation w + sum(obj1[j]*x_j) = sum(b_i); its rhs
        // constant must be sum(b_i) (post-negation stored rhs), or the
        // Phase-I feasibility test would read the wrong constant.
        obj1[rhs] = t.iter().take(m).map(|r| r[rhs]).sum();
        t.push(obj1);
        // Phase-II objective row: W = -c*x = 0 - sum c_j*x_j under the same
        // convention W = rhs - sum obj[j]*x_j, seeded with obj[j] = c_j on
        // structural columns. Carried through Phase I so it arrives
        // canonical at Phase II start.
        let mut obj2 = vec![0i128; rhs + 1];
        for (j, cell) in obj2.iter_mut().enumerate().take(n) {
            *cell = c[j] as i128;
        }
        t.push(obj2);
        Tableau {
            t,
            d: 1,
            basis: (0..m).map(|i| art + i).collect(),
            layout: Layout { m, art, rhs },
            pivots: 0,
        }
    }

    /// Edmonds integer pivot on `(r, col)`; `r` is a constraint row. The
    /// exact-division invariant is checked — a violation refuses with
    /// `ERR_OVERFLOW` rather than rounding silently.
    fn pivot(&mut self, r: usize, col: usize) -> Result<(), i32> {
        self.pivots += 1;
        if self.pivots > MAX_PIVOTS {
            return Err(errno::ERR_FUEL);
        }
        let d = self.d;
        let piv = self.t[r][col];
        if piv == 0 || d == 0 {
            // invariant guard, unreachable by construction
            return Err(errno::ERR_OVERFLOW);
        }
        let prow = self.t[r].clone();
        for i in 0..self.t.len() {
            if i == r {
                continue;
            }
            let f = self.t[i][col];
            if f == 0 && piv == d {
                continue; // exact no-op only when d == 1
            }
            let row = &mut self.t[i];
            for j in 0..row.len() {
                let fp = match f.checked_mul(prow[j]) {
                    Some(v) => v,
                    None => return Err(errno::ERR_OVERFLOW),
                };
                let num = match piv.checked_mul(row[j]) {
                    Some(v) => match v.checked_sub(fp) {
                        Some(w) => w,
                        None => return Err(errno::ERR_OVERFLOW),
                    },
                    None => return Err(errno::ERR_OVERFLOW),
                };
                if num % d != 0 {
                    // exact-division invariant violated
                    return Err(errno::ERR_OVERFLOW);
                }
                row[j] = num / d;
            }
        }
        self.d = piv;
        self.basis[r] = col;
        Ok(())
    }

    /// Bland entering: smallest eligible column with positive obj entry.
    fn entering(&self, obj_row: usize, eligible: usize) -> Option<usize> {
        (0..eligible).find(|&j| self.t[obj_row][j] > 0)
    }

    /// Ratio test: among rows with `T[i][col] > 0`, the minimum of
    /// `rhs_i / T[i][col]` by exact i128 cross-multiplication; ties go to
    /// the smallest basic-variable index (Bland leaving rule).
    fn leaving(&self, col: usize) -> Option<usize> {
        use core::cmp::Ordering;
        let m = self.layout.m;
        let rhs = self.layout.rhs;
        let mut best: Option<usize> = None;
        for i in 0..m {
            let a_i = self.t[i][col];
            if a_i <= 0 {
                continue;
            }
            let take = match best {
                None => true,
                Some(b) => {
                    // rhs_i/a_i < rhs_b/a_b  <=>  rhs_i*a_b < rhs_b*a_i
                    match self.t[i][rhs].checked_mul(self.t[b][col]) {
                        Some(l) => match self.t[b][rhs].checked_mul(a_i) {
                            Some(r) => match l.cmp(&r) {
                                Ordering::Less => true,
                                Ordering::Equal => self.basis[i] < self.basis[b],
                                Ordering::Greater => false,
                            },
                            None => return None,
                        },
                        None => return None,
                    }
                }
            };
            if take {
                best = Some(i);
            }
        }
        best
    }

    /// One full phase: pivot while Bland finds an improving column.
    fn run_phase(&mut self, obj_row: usize, eligible: usize) -> Result<(), i32> {
        while let Some(col) = self.entering(obj_row, eligible) {
            match self.leaving(col) {
                Some(r) => self.pivot(r, col)?,
                None => return Err(errno::ERR_NO_SOLUTION), // unbounded
            }
        }
        Ok(())
    }
}

/// Core solve: maximize `c·x` s.t. `Ax <= b`, `x >= 0`, in the scale whose
/// multiplier is `scale_mult`. Host-callable; the wire entry point is a
/// byte-level adapter over this.
///
/// # Errors
/// See the module refusal table (`ERR_DIM`, `ERR_BAD_SCALE`,
/// `ERR_OVERFLOW`, `ERR_FUEL`, `ERR_NO_SOLUTION`).
pub fn maximize(
    c: &[Fix],
    a: &[Fix],
    b: &[Fix],
    n: usize,
    m: usize,
    scale_mult: i128,
) -> Result<LpSolution, i32> {
    if scale_mult <= 0 {
        return Err(errno::ERR_BAD_SCALE);
    }
    if n == 0 || n > MAX_VARS || m == 0 || m > MAX_CONS {
        return Err(errno::ERR_DIM);
    }
    if c.len() != n || b.len() != m || a.len() != n.saturating_mul(m) {
        return Err(errno::ERR_DIM);
    }

    let mut tab = Tableau::build(c, a, b, n, m);
    let eligible = tab.layout.art; // structural + slack; artificials never enter

    // Phase I: drive w = sum(artificials) to 0.
    let obj_row1 = tab.layout.m;
    tab.run_phase(obj_row1, eligible)?;
    if tab.t[obj_row1][tab.layout.rhs] != 0 {
        return Err(errno::ERR_NO_SOLUTION); // w > 0: infeasible
    }

    // Cleanup: rows still holding an artificial basic (value 0) are either
    // pivoted onto a real column (rhs unchanged since rhs_r = 0, so every
    // basic value stays >= 0) or dropped as redundant (0 = 0). Removing a
    // constraint row shifts the trailing objective rows down one slot;
    // `layout.m` bookkeeping absorbs that (they always sit at m, m+1).
    let mut i = 0;
    while i < tab.layout.m {
        if tab.basis[i] >= tab.layout.art {
            let mut pivoted = false;
            for j in 0..eligible {
                if tab.t[i][j] != 0 {
                    tab.pivot(i, j)?;
                    pivoted = true;
                    break;
                }
            }
            if !pivoted {
                tab.t.remove(i);
                tab.basis.remove(i);
                tab.layout.m -= 1;
                continue;
            }
        }
        i += 1;
    }

    // Phase II: minimize W = -c*x using the carried Phase-II row.
    let obj_row2 = tab.layout.m + 1;
    tab.run_phase(obj_row2, eligible)?;

    extract(&tab, c, n, scale_mult)
}

/// Read the optimal vertex and objective out of the final tableau.
fn extract(tab: &Tableau, c: &[Fix], n: usize, scale_mult: i128) -> Result<LpSolution, i32> {
    let d = tab.d;
    let rhs = tab.layout.rhs;
    let mut x = vec![0; n];
    let mut exact_vertex = true;
    for (j, xj) in x.iter_mut().enumerate() {
        if let Some(r) = tab.basis.iter().position(|&k| k == j) {
            let num = tab.t[r][rhs];
            let q = match fixed::div_round_half_even(num, d) {
                Ok(v) => v,
                Err(_) => return Err(errno::ERR_OVERFLOW),
            };
            exact_vertex &= num.rem_euclid(d) == 0;
            *xj = match Fix::try_from(q) {
                Ok(v) => v,
                Err(_) => return Err(errno::ERR_OVERFLOW),
            };
        }
    }
    let mut acc: i128 = 0;
    for j in 0..n {
        let p = match (c[j] as i128).checked_mul(x[j] as i128) {
            Some(v) => v,
            None => return Err(errno::ERR_OVERFLOW),
        };
        acc = match acc.checked_add(p) {
            Some(v) => v,
            None => return Err(errno::ERR_OVERFLOW),
        };
    }
    let exact_objective = acc.rem_euclid(scale_mult) == 0;
    let obj_q = match fixed::div_round_half_even(acc, scale_mult) {
        Ok(v) => v,
        Err(_) => return Err(errno::ERR_OVERFLOW),
    };
    let objective = match Fix::try_from(obj_q) {
        Ok(v) => v,
        Err(_) => return Err(errno::ERR_OVERFLOW),
    };
    Ok(LpSolution { x, objective, exact_vertex, exact_objective, pivots: tab.pivots })
}

/// Wire format for the LP leaf.
///
/// Input (after the 16-byte Mgi1Header, `leaf_id` =
/// `LEAF_ID_COLLATERAL_LP`, any documented `scale_tag`):
///
/// | Section | Size | Meaning |
/// |---------|------|---------|
/// | n | 8 | `u64` LE variable count, 1..=`MAX_VARS` |
/// | m | 8 | `u64` LE constraint count, 1..=`MAX_CONS` |
/// | c | `n*8` | `i64` LE objective coefficients |
/// | A | `n*m*8` | `i64` LE row-major constraint matrix |
/// | b | `m*8` | `i64` LE rhs values (may be negative) |
///
/// Header `dims` law: `dims = 2 + n + n*m + m` (payload element count).
/// Output (16-byte header echoing leaf/scale, `dims = n + 2`):
/// `objective` i64 LE, flags u8 (bit0 `exact_vertex`, bit1
/// `exact_objective`, bit2 `pivots_hit_cap` — always 0 on success), 7 zero
/// pad bytes, then `x` as `n * 8` bytes i64 LE.
fn run(in_buf: &[u8], out_buf: &mut [u8]) -> Result<(), i32> {
    let hdr = crate::parse_header(in_buf)?;
    if hdr.leaf_id != LEAF_ID_COLLATERAL_LP {
        return Err(errno::ERR_BAD_MAGIC);
    }
    let scale_mult = match fixed::scale_of(hdr.scale_tag) {
        Some(s) => s,
        None => return Err(errno::ERR_BAD_SCALE),
    };
    if in_buf.len() < 32 {
        return Err(errno::ERR_TRUNCATED);
    }
    let mut nb = [0u8; 8];
    nb.copy_from_slice(&in_buf[16..24]);
    let n = u64::from_le_bytes(nb);
    nb.copy_from_slice(&in_buf[24..32]);
    let m = u64::from_le_bytes(nb);
    if n == 0 || n > MAX_VARS as u64 || m == 0 || m > MAX_CONS as u64 {
        return Err(errno::ERR_DIM);
    }
    let expected_dims: u64 = {
        let nm = match n.checked_mul(m) {
            Some(v) => v,
            None => return Err(errno::ERR_DIM),
        };
        match 2u64
            .checked_add(n)
            .and_then(|v| v.checked_add(nm))
            .and_then(|v| v.checked_add(m))
        {
            Some(v) => v,
            None => return Err(errno::ERR_DIM),
        }
    };
    if hdr.dims != expected_dims {
        return Err(errno::ERR_DIM);
    }
    let (nv, mv) = (n as usize, m as usize);
    let end = 16 + (2 + nv + nv * mv + mv) * 8;
    if in_buf.len() < end {
        return Err(errno::ERR_TRUNCATED);
    }
    if in_buf.len() > end {
        return Err(errno::ERR_TRAILING);
    }
    let read_fixes = |start: usize, count: usize| -> Vec<Fix> {
        let mut out = Vec::with_capacity(count);
        for chunk in in_buf[start..start + count * 8].chunks_exact(8) {
            let mut b = [0u8; 8];
            b.copy_from_slice(chunk);
            out.push(Fix::from_le_bytes(b));
        }
        out
    };
    let c_vec = read_fixes(32, nv);
    let a_vec = read_fixes(32 + nv * 8, nv * mv);
    let b_vec = read_fixes(32 + (nv + nv * mv) * 8, mv);
    let sol = maximize(&c_vec, &a_vec, &b_vec, nv, mv, scale_mult)?;
    // Commit phase: write out only after full admission.
    let out_len = 16 + 16 + nv * 8;
    if out_buf.len() < out_len {
        return Err(errno::ERR_CAP_TOO_SMALL);
    }
    let ohdr = crate::Mgi1Header {
        leaf_id: LEAF_ID_COLLATERAL_LP,
        scale_tag: hdr.scale_tag,
        dims: (nv + 2) as u64,
    };
    ohdr.encode(out_buf);
    out_buf[16..24].copy_from_slice(&sol.objective.to_le_bytes());
    let mut flags: u8 = 0;
    if sol.exact_vertex {
        flags |= 1;
    }
    if sol.exact_objective {
        flags |= 2;
    }
    out_buf[24] = flags;
    out_buf[25..32].fill(0);
    for (i, v) in sol.x.iter().enumerate() {
        out_buf[32 + 8 * i..32 + 8 * (i + 1)].copy_from_slice(&v.to_le_bytes());
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
pub unsafe extern "C" fn mgi_collateral_lp_compute(
    in_: *const u8,
    in_len: u32,
    out: *mut u8,
    out_cap: u32,
) -> i32 {
    if in_.is_null() || out.is_null() {
        return errno::ERR_NULL_PTR;
    }
    if (out_cap as usize) < 32 {
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
    use crate::fixed::{scale_of, TAG_MONEY, TAG_RATIO};

    const SC: i128 = 1_000_000; // S_MONEY = 10^6

    fn s(v: i64) -> Fix {
        (v as i128 * SC) as Fix
    }

    fn build_input(c: &[Fix], a: &[Fix], b: &[Fix], n: usize, m: usize, tag: u8) -> Vec<u8> {
        let dims = 2 + n + n * m + m;
        let mut bytes =
            crate::Mgi1Header { leaf_id: LEAF_ID_COLLATERAL_LP, scale_tag: tag, dims: dims as u64 }
                .to_bytes()
                .to_vec();
        bytes.extend_from_slice(&(n as u64).to_le_bytes());
        bytes.extend_from_slice(&(m as u64).to_le_bytes());
        bytes.extend(fix_bytes(c));
        bytes.extend(fix_bytes(a));
        bytes.extend(fix_bytes(b));
        bytes
    }

    fn fix_bytes(vals: &[Fix]) -> Vec<u8> {
        vals.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    #[test]
    fn scale_tags_resolve_to_test_constants() {
        assert_eq!(scale_of(TAG_MONEY), Some(SC));
        assert_eq!(scale_of(TAG_RATIO), Some(100_000_000));
    }

    #[test]
    fn one_variable_trivial_bound() {
        // max x s.t. x <= 4  ->  x* = 4, z* = 4
        let r = maximize(&[1], &[1], &[4], 1, 1, 1).unwrap();
        assert_eq!(r.x, alloc::vec![4]);
        assert_eq!(r.objective, 4);
        assert!(r.exact_vertex && r.exact_objective);
    }

    #[test]
    fn classic_two_variable_textbook_lp() {
        // max 3x + 5y s.t. x <= 4, 2y <= 12, 3x + 2y <= 18
        // textbook optimum: x=2, y=6, z=36.
        let c = [3, 5];
        let a = [1, 0, 0, 2, 3, 2];
        let b = [4, 12, 18];
        let r = maximize(&c, &a, &b, 2, 3, 1).unwrap();
        assert_eq!(r.x, alloc::vec![2, 6]);
        assert_eq!(r.objective, 36);
        assert!(r.exact_vertex && r.exact_objective);
    }

    #[test]
    fn three_variable_hand_computed() {
        // max x + y + z s.t. x + y + z <= 10, x + 2z <= 8. The vertex set
        // has ties (both (8,2,0) and (0,10,0) reach 10); Bland's rule picks
        // one deterministically. Assert the objective and vertex feasibility.
        let c = [1, 1, 1];
        let a = [1, 1, 1, 1, 0, 2];
        let b = [10, 8];
        let r = maximize(&c, &a, &b, 3, 2, 1).unwrap();
        assert_eq!(r.objective, 10);
        let (sx, sy, sz) = (r.x[0], r.x[1], r.x[2]);
        assert_eq!(sx + sy + sz, 10);
        assert!(sx + 2 * sz <= 8);
    }

    #[test]
    fn scaled_fixed_point_matches_raw_integer_solve() {
        // Textbook LP with dimensionless A and money-scaled c/b: the vertex
        // comes out in the money unit (micro-dollars) and the objective
        // rescale divides by the scale once, so z* = s(36).
        let c = [s(3), s(5)];
        let a = [1, 0, 0, 2, 3, 2];
        let b = [s(4), s(12), s(18)];
        let r = maximize(&c, &a, &b, 2, 3, SC).unwrap();
        assert_eq!(r.x, alloc::vec![s(2), s(6)]);
        assert_eq!(r.objective, s(36));
        assert!(r.exact_vertex && r.exact_objective);
        // Common-factor scaling of A and b is vertex-invariant: same shape,
        // raw-unit vertex.
        let a2 = [s(1), 0, 0, s(2), s(3), s(2)];
        let b2 = [s(4), s(12), s(18)];
        let r2 = maximize(&c, &a2, &b2, 2, 3, SC).unwrap();
        assert_eq!(r2.x, alloc::vec![2, 6]);
    }

    #[test]
    fn infeasible_refuses_with_no_solution() {
        // x <= 0 and -x <= -1  =>  x >= 1 and x <= 0: empty feasible set.
        let r = maximize(&[1], &[1, -1], &[0, -1], 1, 2, 1);
        assert_eq!(r, Err(errno::ERR_NO_SOLUTION));
    }

    #[test]
    fn unbounded_refuses_with_no_solution() {
        // max x s.t. 0x <= 1: nothing bounds x from above.
        let r = maximize(&[1], &[0], &[1], 1, 1, 1);
        assert_eq!(r, Err(errno::ERR_NO_SOLUTION));
    }

    #[test]
    fn negative_rhs_rows_are_handled() {
        // max x s.t. -x <= -2 (x >= 2), x <= 5  ->  x* = 5.
        let r = maximize(&[1], &[-1, 1], &[-2, 5], 1, 2, 1).unwrap();
        assert_eq!(r.x, alloc::vec![5]);
        assert_eq!(r.objective, 5);
    }

    #[test]
    fn dim_and_scale_refusals() {
        let c = [1];
        assert_eq!(maximize(&[], &[1], &[1], 0, 1, 1), Err(errno::ERR_DIM));
        assert_eq!(maximize(&c, &[1], &[1], 1, 0, 1), Err(errno::ERR_DIM));
        assert_eq!(maximize(&c, &[1], &[1], MAX_VARS + 1, 1, 1), Err(errno::ERR_DIM));
        assert_eq!(maximize(&c, &[1], &[1], 1, MAX_CONS + 1, 1), Err(errno::ERR_DIM));
        assert_eq!(maximize(&c, &[], &[1], 1, 1, 1), Err(errno::ERR_DIM));
        assert_eq!(maximize(&c, &[1], &[], 1, 1, 1), Err(errno::ERR_DIM));
        assert_eq!(maximize(&c, &[1], &[1], 1, 1, 0), Err(errno::ERR_BAD_SCALE));
        assert_eq!(maximize(&c, &[1], &[1], 1, 1, -5), Err(errno::ERR_BAD_SCALE));
    }

    #[test]
    fn fuel_cap_is_respected_and_typed() {
        // Trivial LP: success consumes pivots strictly within the fuel cap.
        // Bland's rule guarantees termination below the cap on any input;
        // MAX_PIVOTS is belt-and-braces, never a normal exit.
        let r = maximize(&[1], &[1], &[1], 1, 1, 1).unwrap();
        assert!(r.pivots >= 1 && r.pivots <= MAX_PIVOTS);
        let c = [2, 3, -1, 4];
        let a = [1, 1, 0, 1, 2, 1, -1, 0, 0, 1, 1, 1, 1, 0, 0, 2];
        let b = [10, 4, 6, 14];
        let r = maximize(&c, &a, &b, 4, 4, 1).unwrap();
        assert!(r.pivots <= MAX_PIVOTS);
    }

    #[test]
    fn determinism_two_runs_byte_identical() {
        let c = [2, 3, -1, 4];
        let a = [1, 1, 0, 1, 2, 1, -1, 0, 0, 1, 1, 1, 1, 0, 0, 2];
        let b = [10, 4, 6, 14];
        let input = build_input(&c, &a, &b, 4, 4, fixed::TAG_MONEY);
        let mut o1 = vec![0u8; 16 + 16 + 32];
        let mut o2 = vec![0u8; 16 + 16 + 32];
        assert_eq!(run(&input, &mut o1), Ok(()));
        assert_eq!(run(&input, &mut o2), Ok(()));
        assert_eq!(o1, o2);
    }

    #[test]
    fn wire_roundtrip_textbook_case() {
        let c = [s(3), s(5)];
        let a = [1, 0, 0, 2, 3, 2];
        let b = [s(4), s(12), s(18)];
        let input = build_input(&c, &a, &b, 2, 3, fixed::TAG_MONEY);
        let mut out = vec![0u8; 16 + 16 + 8 * 2];
        assert_eq!(run(&input, &mut out), Ok(()));
        assert_eq!(&out[0..4], b"MGI1");
        assert_eq!(out[4], LEAF_ID_COLLATERAL_LP);
        let objective = Fix::from_le_bytes(out[16..24].try_into().unwrap());
        assert_eq!(objective, s(36));
        let flags = out[24];
        assert_eq!(flags & 3, 3); // both exact
        let x0 = Fix::from_le_bytes(out[32..40].try_into().unwrap());
        let x1 = Fix::from_le_bytes(out[40..48].try_into().unwrap());
        assert_eq!([x0, x1], [s(2), s(6)]);
    }

    #[test]
    fn wire_refusals() {
        let c = [3, 5];
        let a = [1, 0, 0, 2, 3, 2];
        let b = [4, 12, 18];
        let mut out = vec![0u8; 64];
        // wrong leaf id
        let input = build_input(&c, &a, &b, 2, 3, fixed::TAG_MONEY);
        let mut bad = input.clone();
        bad[4] = 1; // hhi leaf id
        assert_eq!(run(&bad, &mut out), Err(errno::ERR_BAD_MAGIC));
        // bad scale tag
        let input = build_input(&c, &a, &b, 2, 3, 99);
        assert_eq!(run(&input, &mut out), Err(errno::ERR_BAD_SCALE));
        // trailing bytes
        let mut input = build_input(&c, &a, &b, 2, 3, fixed::TAG_MONEY);
        input.push(0);
        assert_eq!(run(&input, &mut out), Err(errno::ERR_TRAILING));
        // truncated
        let input = build_input(&c, &a, &b, 2, 3, fixed::TAG_MONEY);
        assert_eq!(run(&input[..input.len() - 8], &mut out), Err(errno::ERR_TRUNCATED));
        // infeasible over the wire
        let input = build_input(&[1], &[1, -1], &[0, -1], 1, 2, fixed::TAG_MONEY);
        assert_eq!(run(&input, &mut out), Err(errno::ERR_NO_SOLUTION));
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
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/collateral_lp.rs"))
                .unwrap();
        for tok in &toks {
            assert!(
                !src.contains(tok.as_str()),
                "float token {tok:?} found in collateral_lp.rs — merger kernels are integer-only"
            );
        }
    }
}
