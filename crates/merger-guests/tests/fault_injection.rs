//! MG5 — host-compiled SEU / dimension / fuel fault-injection courts over
//! the merger-guests `mgi_*` leaf ABI (E-M13, single-artifact decision).
//!
//! # Honest two-tier scoping (read this first)
//!
//! These courts drive the leaf ABI **host-compiled** (the crate's `rlib`
//! target, native codegen): each `mgi_<leaf>_compute` is called as an
//! ordinary `extern "C"` function over host memory. What is proven here is
//! the fail-closed law of the leaf kernels and their wire adapters. What is
//! **not** proven here is the in-Wasm behavior of the cdylib under temprun
//! (panic-abort trap lowering, fuel accounting under a real runtime); those
//! land with the temprun bridge and will extend, not change, these courts.
//!
//! # Court law
//!
//! For each of the eight leaves, an LCG-driven single-bit SEU sweep flips
//! one byte of the input buffer per trial and classifies the outcome
//! against an **independent host-side oracle** that re-derives each leaf's
//! documented math (parse law, domain laws, kernel arithmetic, output
//! encoding) — never by calling the leaf's own kernel helpers. Every
//! outcome must land in exactly one class:
//!
//! - `REFUSED` — a typed errno from the documented refusal space (the
//!   shared errno table plus each leaf's documented leaf-local codes);
//! - `ADMITTED-LAWFUL` — status OK and the committed output prefix
//!   byte-equal to the oracle's independently re-derived encoding.
//!
//! Anything else is SDC and fails the court. Census law:
//! `refused + lawful + sdc == flips`, `sdc == 0`.
//!
//! # Tiering (prefix-stream)
//!
//! The SEU sweeps run 10,000 flips each (fast tier); the
//! `#[ignore = "nato-tier3"]` full tier runs 1,000,000 flips **with the
//! same LCG seed**, so the full stream's prefix is the fast stream — full
//! subsumes fast.
//!
//! # Positive control
//!
//! A deliberately corrupted-accepting drive shim (reports OK with a canned
//! output for ANY corrupted input) must be caught by the same sweep
//! machinery with SDC > 0 — proving the sweep non-vacuous.
//!
//! # Fuel / overflow / dimension tier
//!
//! Real ABI calls at the documented caps (and one past them) must produce
//! the documented refusals (`ERR_DIM`/`ERR_OVERFLOW`/`ERR_LENGTH`). The
//! leaves' fuel budgets (`ERR_FUEL`) are, by their own cap arithmetic
//! (asserted below), unreachable-by-construction for in-domain inputs; the
//! fuel tier proves the boundary positively — a max-dimension lawful input
//! still succeeds, never a fuel refusal.
//!
//! No `println!` anywhere: failures speak through the harness.

#![allow(clippy::unwrap_used, clippy::expect_used)] // JUSTIFICATION: test code only

use merger_guests::collateral_lp as lp;
use merger_guests::errno;
use merger_guests::fixed;
use merger_guests::fractional_shares as frac;
use merger_guests::hhi_calc as hhi;
use merger_guests::netting_engine as neng;
use merger_guests::netting_settler as nset;
use merger_guests::pn64_audit as pna;
use merger_guests::simm_matrix as simm;
use merger_guests::vote_reconciler as vote;

use merger_guests::{Mgi1Header, MGI1_MAGIC};

// ---------------------------------------------------------------------------
// LCG — donor Knuth constants, fixed seed (fast tier is a prefix of full)
// ---------------------------------------------------------------------------

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

/// Fixed seed: fast and full tiers share it, so full subsumes fast.
const SEU_SEED: u64 = 0x4D47_3531_0000_0001; // "MG51" — merger SEU court

// ---------------------------------------------------------------------------
// Census + expectation model
// ---------------------------------------------------------------------------

/// Outcome census of one sweep pass (exactly-one-class reconciliation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Census {
    refused: u64,
    lawful: u64,
    sdc: u64,
}

impl Census {
    fn total(self) -> u64 {
        self.refused + self.lawful + self.sdc
    }
}

/// The oracle's prediction for one corrupted input.
enum Expect {
    /// The documented leaf law refuses this input.
    Refuse,
    /// The leaf admits; expected output prefix (header + committed body).
    Admit(Vec<u8>),
}

/// Generous output capacity so `ERR_CAP_TOO_SMALL` never fires in the SEU
/// tier (cap refusals are exercised directly in the cap tier).
const OUT_CAP: usize = 4096;

/// The documented typed-refusal space across the merger-guests ABI: the
/// shared errno table (0 excluded — it is OK) plus every leaf-local code.
const fn is_typed_refusal(st: i32) -> bool {
    matches!(
        st,
        errno::ERR_NULL_PTR
            | errno::ERR_BAD_MAGIC
            | errno::ERR_BAD_VERSION
            | errno::ERR_TRUNCATED
            | errno::ERR_TRAILING
            | errno::ERR_RESERVED_NOT_ZERO
            | errno::ERR_BAD_MODEL
            | errno::ERR_TABLE_FULL
            | errno::ERR_BAD_HANDLE
            | errno::ERR_STREAM_EMPTY
            | errno::ERR_CAP_TOO_SMALL
            | errno::ERR_SEAL_UNAVAILABLE
            | errno::ERR_STREAM_ALIGNMENT
            | errno::ERR_BAD_SCALE
            | errno::ERR_DIM
            | errno::ERR_OVERFLOW
            | errno::ERR_FUEL
            | errno::ERR_NO_SOLUTION
            | errno::ERR_LENGTH
            | -4101 // vote_reconciler: row-domain refusal
            | -4102 // vote_reconciler: threshold-domain refusal
            | -4201 // netting_settler: instruction-domain refusal
            | -4202 // netting_settler: zero-sum refusal
            | -4301 // fractional_shares: holder-domain refusal
            | -4302 // fractional_shares: ratio-domain refusal
            | -4303 // fractional_shares: conservation refusal
    )
}

/// One SEU pass: for each LCG flip, corrupt one byte of the input buffer
/// (subject to the leaf's flip band), drive the real ABI, and classify
/// against the oracle. `drive` is the ABI call — the positive control
/// substitutes a corrupted-accepting shim here.
fn run_seu_sweep<Drive, Law>(
    flips: u64,
    clean: &[u8],
    flip_allowed: impl Fn(usize) -> bool,
    mut drive: Drive,
    law: Law,
) -> Census
where
    Drive: FnMut(&[u8]) -> (i32, Vec<u8>),
    Law: Fn(&[u8]) -> Expect,
{
    let mut rng = Lcg(SEU_SEED);
    let mut c = Census::default();

    while c.total() < flips {
        let idx = rng.below(clean.len() as u64) as usize;
        let bit = rng.below(8);
        if !flip_allowed(idx) {
            // Out-of-band index: burn draws so the stream stays
            // deterministic without crediting a skipped trial.
            let _ = (rng.next(), rng.next(), rng.next());
            continue;
        }
        let mut corrupt = clean.to_vec();
        corrupt[idx] ^= 1 << bit;

        let expect = law(&corrupt);
        let (st, out) = drive(&corrupt);

        if st == errno::OK {
            match expect {
                Expect::Admit(ref expected) => {
                    if out.len() >= expected.len() && out[..expected.len()] == expected[..] {
                        c.lawful += 1;
                    } else {
                        c.sdc += 1;
                    }
                }
                Expect::Refuse => c.sdc += 1, // wrong-Ok on an unlawful input
            }
        } else if is_typed_refusal(st) {
            match expect {
                Expect::Refuse => {
                    // Documented never-partial law: a refusing call must
                    // leave the caller's output buffer bit-exact untouched.
                    if out.iter().any(|&b| b != 0xAA) {
                        c.sdc += 1; // partial write on a refusal path
                    } else {
                        c.refused += 1;
                    }
                }
                Expect::Admit(_) => c.sdc += 1, // wrong-refusal of a lawful input
            }
        } else {
            c.sdc += 1; // status outside the documented refusal space
        }
    }
    c
}

// ---------------------------------------------------------------------------
// Shared wire helpers (re-derived from the documented MGI1 header law)
// ---------------------------------------------------------------------------

/// Independent header parse: magic -> reserved -> scale-tag table
/// (TAG_MONEY=1, TAG_RATIO=2, TAG_HHI=3, TAG_SHARES=4, re-derived here).
fn header_law(b: &[u8], scale_tags: &[u8]) -> Result<(u8, u8, u64), i32> {
    if b.len() < Mgi1Header::SIZE {
        return Err(errno::ERR_TRUNCATED);
    }
    if b[0..4] != MGI1_MAGIC {
        return Err(errno::ERR_BAD_MAGIC);
    }
    if b[6..8] != [0, 0] {
        return Err(errno::ERR_RESERVED_NOT_ZERO);
    }
    let leaf_id = b[4];
    let tag = b[5];
    if !scale_tags.contains(&tag) {
        return Err(errno::ERR_BAD_SCALE);
    }
    let dims = u64::from_le_bytes(b[8..16].try_into().unwrap());
    Ok((leaf_id, tag, dims))
}

fn r_u64(b: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(b[off..off + 8].try_into().unwrap())
}

fn r_i64(b: &[u8], off: usize) -> i64 {
    i64::from_le_bytes(b[off..off + 8].try_into().unwrap())
}

/// Independent round-half-even division (the leaves' rounding law,
/// re-derived): ties round to the even neighbor.
fn round_he(num: i128, den: i128) -> Option<i128> {
    if den <= 0 {
        return None;
    }
    let q = num.div_euclid(den);
    let r = num.rem_euclid(den);
    let r2 = r * 2;
    if r2 > den || (r2 == den && q % 2 != 0) {
        Some(q + 1)
    } else {
        Some(q)
    }
}

/// Oracle-side 16-byte MGI1 header writer.
fn push_header(out: &mut Vec<u8>, leaf_id: u8, tag: u8, dims: u64) {
    let mut h = [0u8; 16];
    h[0..4].copy_from_slice(&MGI1_MAGIC);
    h[4] = leaf_id;
    h[5] = tag;
    h[8..16].copy_from_slice(&dims.to_le_bytes());
    out.extend_from_slice(&h);
}

// ---------------------------------------------------------------------------
// Leaf fixtures (lawful inputs) + flip bands + drive wrappers
// ---------------------------------------------------------------------------

/// HHI fixture: post-merger shares 60/40, no pre-merger section.
fn hhi_clean() -> Vec<u8> {
    let shares: [i64; 2] = [6, 4];
    let mut b = Mgi1Header {
        leaf_id: hhi::LEAF_ID_HHI_CALC,
        scale_tag: fixed::TAG_HHI,
        dims: shares.len() as u64,
    }
    .to_bytes()
    .to_vec();
    for s in shares {
        b.extend_from_slice(&s.to_le_bytes());
    }
    b
}

fn hhi_drive(buf: &[u8]) -> (i32, Vec<u8>) {
    let mut out = vec![0xAA_u8; OUT_CAP];
    let st = unsafe {
        hhi::mgi_hhi_calc_compute(
            buf.as_ptr(),
            buf.len() as u32,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    };
    (st, out)
}

/// Netting-engine fixture: three obligations over three parties.
fn neng_clean() -> Vec<u8> {
    let rows: [(u64, u64, i64); 3] = [(1, 2, 100), (2, 1, 30), (1, 3, 50)];
    let mut b = Mgi1Header {
        leaf_id: neng::LEAF_ID_NETTING_ENGINE,
        scale_tag: fixed::TAG_MONEY,
        dims: rows.len() as u64,
    }
    .to_bytes()
    .to_vec();
    for (d, c, a) in rows {
        b.extend_from_slice(&d.to_le_bytes());
        b.extend_from_slice(&c.to_le_bytes());
        b.extend_from_slice(&a.to_le_bytes());
    }
    b
}

fn neng_drive(buf: &[u8]) -> (i32, Vec<u8>) {
    let mut out = vec![0xAA_u8; OUT_CAP];
    let st = unsafe {
        neng::mgi_netting_engine_compute(
            buf.as_ptr(),
            buf.len() as u32,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    };
    (st, out)
}

/// SIMM fixture: 2x2 diagonal [[0.5, 0], [0, 0.25]] times w = [4, 8].
fn simm_clean() -> Vec<u8> {
    let (rows, cols) = (2usize, 2usize);
    let matrix: [i64; 4] = [(fixed::S_RATIO / 2) as i64, 0, 0, (fixed::S_RATIO / 4) as i64];
    let vec: [i64; 2] = [(4 * fixed::S_RATIO) as i64, (8 * fixed::S_RATIO) as i64];
    let dims = (2 + rows * cols + cols) as u64;
    let mut b =
        Mgi1Header { leaf_id: simm::LEAF_ID_SIMM_MATRIX, scale_tag: fixed::TAG_RATIO, dims }
            .to_bytes()
            .to_vec();
    b.extend_from_slice(&(rows as u64).to_le_bytes());
    b.extend_from_slice(&(cols as u64).to_le_bytes());
    for v in matrix {
        b.extend_from_slice(&v.to_le_bytes());
    }
    for v in vec {
        b.extend_from_slice(&v.to_le_bytes());
    }
    b
}

fn simm_drive(buf: &[u8]) -> (i32, Vec<u8>) {
    let mut out = vec![0xAA_u8; OUT_CAP];
    let st = unsafe {
        simm::mgi_simm_matrix_compute(
            buf.as_ptr(),
            buf.len() as u32,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    };
    (st, out)
}

/// LP fixture (bounded, generic, integral optimum):
/// max 3,000,001 x + 2,000,002 y  s.t.  x + y <= 4,000,003,
/// x + 3y <= 6,000,005, x,y >= 0. Unique optimum (4,000,003, 0); vertex
/// objective gaps ~1e6 dwarf the low-byte flip perturbations (<= 255).
fn lp_clean() -> Vec<u8> {
    let n = 2usize;
    let m = 2usize;
    let c: [i64; 2] = [3_000_001, 2_000_002];
    let a: [i64; 4] = [1_000_001, 1_000_002, 1_000_003, 3_000_004];
    let b_rhs: [i64; 2] = [4_000_003, 6_000_005];
    let dims = (2 + n + n * m + m) as u64;
    let mut b =
        Mgi1Header { leaf_id: lp::LEAF_ID_COLLATERAL_LP, scale_tag: fixed::TAG_MONEY, dims }
            .to_bytes()
            .to_vec();
    b.extend_from_slice(&(n as u64).to_le_bytes());
    b.extend_from_slice(&(m as u64).to_le_bytes());
    for v in c {
        b.extend_from_slice(&v.to_le_bytes());
    }
    for v in a {
        b.extend_from_slice(&v.to_le_bytes());
    }
    for v in b_rhs {
        b.extend_from_slice(&v.to_le_bytes());
    }
    b
}

fn lp_drive(buf: &[u8]) -> (i32, Vec<u8>) {
    let mut out = vec![0xAA_u8; OUT_CAP];
    let st = unsafe {
        lp::mgi_collateral_lp_compute(
            buf.as_ptr(),
            buf.len() as u32,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    };
    (st, out)
}

/// Vote fixture: 600/500 + 400/300 against quorum 1/2, supermajority 2/3.
fn vote_clean() -> Vec<u8> {
    let rows: [(u64, i64, i64); 2] = [(1, 600, 500), (2, 400, 300)];
    let mut b = Mgi1Header {
        leaf_id: vote::LEAF_ID_VOTE_RECONCILER,
        scale_tag: fixed::TAG_SHARES,
        dims: rows.len() as u64,
    }
    .to_bytes()
    .to_vec();
    for (cls, sh, vf) in rows {
        b.extend_from_slice(&cls.to_le_bytes());
        b.extend_from_slice(&sh.to_le_bytes());
        b.extend_from_slice(&vf.to_le_bytes());
    }
    b.extend_from_slice(&1_i64.to_le_bytes()); // quorum_num
    b.extend_from_slice(&2_i64.to_le_bytes()); // quorum_den
    b.extend_from_slice(&2_i64.to_le_bytes()); // super_num
    b.extend_from_slice(&3_i64.to_le_bytes()); // super_den
    b
}

fn vote_drive(buf: &[u8]) -> (i32, Vec<u8>) {
    let mut out = vec![0xAA_u8; OUT_CAP];
    let st = unsafe {
        vote::mgi_vote_reconciler_compute(
            buf.as_ptr(),
            buf.len() as u32,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    };
    (st, out)
}

/// Settler fixture: 2 parties, mutual netting 100 / 30 (zero-sum).
fn nset_clean() -> Vec<u8> {
    let ins: [(u64, u64, i64); 2] = [(0, 1, 100), (1, 0, 30)];
    let mut b = Mgi1Header {
        leaf_id: nset::LEAF_ID_NETTING_SETTLER,
        scale_tag: fixed::TAG_MONEY,
        dims: ins.len() as u64,
    }
    .to_bytes()
    .to_vec();
    b.extend_from_slice(&2_u32.to_le_bytes()); // n_parties
    for (p, q, a) in ins {
        b.extend_from_slice(&p.to_le_bytes());
        b.extend_from_slice(&q.to_le_bytes());
        b.extend_from_slice(&a.to_le_bytes());
    }
    b
}

fn nset_drive(buf: &[u8]) -> (i32, Vec<u8>) {
    let mut out = vec![0xAA_u8; OUT_CAP];
    let st = unsafe {
        nset::mgi_netting_settler_compute(
            buf.as_ptr(),
            buf.len() as u32,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    };
    (st, out)
}

/// Fractional fixture: 3 holders at exchange ratio 1/2 (0.5 in S_RATIO).
fn frac_clean() -> Vec<u8> {
    let holders: [i64; 3] = [1000, 501, 3];
    let mut b = Mgi1Header {
        leaf_id: frac::LEAF_ID_FRACTIONAL_SHARES,
        scale_tag: fixed::TAG_SHARES,
        dims: holders.len() as u64,
    }
    .to_bytes()
    .to_vec();
    b.extend_from_slice(&((fixed::S_RATIO / 2) as i64).to_le_bytes()); // ratio 0.5
    for s in holders {
        b.extend_from_slice(&s.to_le_bytes());
    }
    b
}

fn frac_drive(buf: &[u8]) -> (i32, Vec<u8>) {
    let mut out = vec![0xAA_u8; OUT_CAP];
    let st = unsafe {
        frac::mgi_fractional_shares_compute(
            buf.as_ptr(),
            buf.len() as u32,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    };
    (st, out)
}

/// PN64 fixture: two chained steps over a sample word, invariant mask 0.
fn pna_clean() -> Vec<u8> {
    use chicago_tdd_tools::conformance::pn64 as parent;
    // Landed chaining fixture (trace_chains_words_through_parent_kernel):
    // W+ = 0b1010 re-produces the W- place 0b0010, so the chained step 1
    // stays enabled and fires again.
    let pn0 = parent::pack(0b0110, 0b0010, 0b1010, 3, 0);
    let steps: [(u64, u64); 2] = [(pn0, 0x0107), (0, 0x0104)];
    let mut b = Mgi1Header {
        leaf_id: pna::LEAF_ID_PN64_AUDIT,
        scale_tag: fixed::TAG_SHARES,
        dims: steps.len() as u64,
    }
    .to_bytes()
    .to_vec();
    b.extend_from_slice(&0_u64.to_le_bytes()); // invariant_mask
    for (pn, ocel) in steps {
        b.extend_from_slice(&pn.to_le_bytes());
        b.extend_from_slice(&ocel.to_le_bytes());
    }
    b
}

fn pna_drive(buf: &[u8]) -> (i32, Vec<u8>) {
    let mut out = vec![0xAA_u8; OUT_CAP];
    let st = unsafe {
        pna::mgi_pn64_audit_compute(
            buf.as_ptr(),
            buf.len() as u32,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    };
    (st, out)
}

/// LP numeric payload band: header + n/m fields flip freely; body words
/// flip their LOW byte only (keeps values in the oracle's exact band,
/// keeps the LP feasible/bounded with a unique optimum — see fixture doc).
fn lp_flip_allowed(idx: usize) -> bool {
    idx < 32 || idx % 8 == 0
}

/// Every byte of every leaf is in band except the LP body restriction.
fn all_flips_allowed(_idx: usize) -> bool {
    true
}

// ---------------------------------------------------------------------------
// Oracles — independent host-side re-derivations, one per leaf
//
// Pattern: each oracle has an `Option`-valued inner body. `None` means "the
// documented law refuses this input" (domain fault, length fault, or
// checked-arithmetic overflow — all collapse into the same REFUSED class);
// `Some(Expect::Admit(bytes))` is the independently re-derived output
// prefix the ABI must commit byte-for-byte.
// ---------------------------------------------------------------------------

/// HHI oracle. Wire law: header(leaf=HHI, TAG_HHI, dims 1..=10_000), len ==
/// 16+8d (post only) or 16+16d (post+pre); anything between is a torn
/// pre-section -> ERR_TRAILING refusal. Kernel law (re-derived from the
/// landed leaf): `fixed::hhi` is round_he(sum_sq * S_HHI / sum^2) in raw
/// 10^4-point units, and the leaf boundary scales it again x10^4, so
/// hhi_wire = round_he(sum_sq * 1e4 / sum^2) * 1e4 (checked); delta =
/// post - pre (checked sub); divest = !(hhi_post <= 18_000_000 && delta <
/// 1_000_000) — the 1800/100-point thresholds in S_HHI wire units.
fn hhi_law(b: &[u8]) -> Expect {
    hhi_inner(b).unwrap_or(Expect::Refuse)
}

fn hhi_inner(b: &[u8]) -> Option<Expect> {
    const CEILING: i128 = 18_000_000; // 1800 pts in S_HHI wire units
    const DELTA_FLOOR: i128 = 1_000_000; // 100 pts in S_HHI wire units
    const MAX_SHARES: u64 = 10_000;
    let (leaf, _tag, dims) = header_law(b, &[fixed::TAG_HHI]).ok()?;
    if leaf != hhi::LEAF_ID_HHI_CALC {
        return None;
    }
    if dims == 0 || dims > MAX_SHARES {
        return None;
    }
    let d = dims as usize;
    let post_end = 16 + 8 * d;
    let pre_end = post_end + 8 * d;
    // Landed length law: torn pre-section (strictly between the two lawful
    // sizes) is ERR_TRAILING, not a silent post-only acceptance.
    if b.len() < post_end || (b.len() > post_end && b.len() != pre_end) {
        return None;
    }
    // Independent HHI, re-derived from the landed leaf: fixed::hhi is
    // round_he(sum_sq * S_HHI(1e4) / sum^2) in raw 10^4-point units, and the
    // leaf boundary multiplies by 10^4 again -> wire units = pts * 1e4.
    let hhi_of = |words: &[i64]| -> Option<i64> {
        let mut sum: i128 = 0;
        let mut sum_sq: i128 = 0;
        for &s in words {
            let s = i128::from(s);
            sum = sum.checked_add(s)?;
            sum_sq = sum_sq.checked_add(s.checked_mul(s)?)?;
        }
        if sum == 0 {
            return None;
        }
        let num = sum_sq.checked_mul(10_000)?;
        let v = round_he(num, sum.checked_mul(sum)?)?;
        // Leaf-boundary rescale x10^4 (checked, i64 domain).
        i64::try_from(v.checked_mul(10_000)?).ok()
    };
    let shares: Vec<i64> = (0..d).map(|i| r_i64(b, 16 + 8 * i)).collect();
    let pre_words: Option<Vec<i64>> = if b.len() == pre_end {
        Some((0..d).map(|i| r_i64(b, post_end + 8 * i)).collect())
    } else {
        None
    };
    let hhi_post = hhi_of(&shares)?;
    let delta: i64 = match &pre_words {
        None => 0,
        Some(p) => {
            let pre = hhi_of(p)?;
            hhi_post.checked_sub(pre)?
        }
    };
    let divest = !(i128::from(hhi_post) <= CEILING && i128::from(delta) < DELTA_FLOOR);
    let mut out = Vec::new();
    push_header(&mut out, hhi::LEAF_ID_HHI_CALC, fixed::TAG_HHI, 3);
    out.extend_from_slice(&hhi_post.to_le_bytes());
    out.extend_from_slice(&delta.to_le_bytes());
    out.push(u8::from(divest));
    out.extend_from_slice(&[0u8; 7]);
    Some(Expect::Admit(out))
}

/// Netting-engine oracle. Wire law: header(leaf=NETTING_ENGINE, TAG_MONEY,
/// dims 1..=10_000), len == 16+24d exact. Kernel law: bilateral pair
/// aggregation in checked i128 (zero pairs drop), multilateral signed
/// positions (creditor += amount, debtor -= amount), conservation sum == 0,
/// then checked i64 narrowing on commit — every failure is a refusal.
fn neng_law(b: &[u8]) -> Expect {
    neng_inner(b).unwrap_or(Expect::Refuse)
}

fn neng_inner(b: &[u8]) -> Option<Expect> {
    let (leaf, tag, dims) = header_law(b, &[fixed::TAG_MONEY]).ok()?;
    if leaf != neng::LEAF_ID_NETTING_ENGINE {
        return None;
    }
    if tag != fixed::TAG_MONEY || dims == 0 || dims > 10_000 {
        return None;
    }
    let d = dims as usize;
    if b.len() != 16 + 24 * d {
        return None;
    }
    let mut pairs: Vec<((u64, u64), i128)> = Vec::new();
    // Landed party law (mirrors the leaf's touch pass): EVERY row's debtor
    // AND creditor are registered as parties (at 0) before any deltas are
    // applied — the old first-touch split dropped a party whenever only its
    // counterpart was already registered.
    let mut parties: Vec<(u64, i128)> = Vec::new();
    let touch = |parties: &mut Vec<(u64, i128)>, p: u64| {
        if !parties.iter().any(|(k, _)| *k == p) {
            parties.push((p, 0));
        }
    };
    for i in 0..d {
        let dbt = r_u64(b, 16 + 24 * i);
        let crd = r_u64(b, 24 + 24 * i);
        let amt = i128::from(r_i64(b, 32 + 24 * i));
        match pairs.iter_mut().find(|(k, _)| *k == (dbt, crd)) {
            Some((_, v)) => *v = v.checked_add(amt)?,
            None => pairs.push(((dbt, crd), amt)),
        }
        touch(&mut parties, dbt);
        touch(&mut parties, crd);
    }
    for i in 0..d {
        let dbt = r_u64(b, 16 + 24 * i);
        let crd = r_u64(b, 24 + 24 * i);
        let amt = i128::from(r_i64(b, 32 + 24 * i));
        for (p, v) in parties.iter_mut() {
            if *p == crd {
                *v = v.checked_add(amt)?;
            }
            if *p == dbt {
                *v = v.checked_sub(amt)?;
            }
        }
    }
    parties.sort_by_key(|(p, _)| *p);
    let merged = parties;
    let total: i128 = merged.iter().try_fold(0_i128, |a, (_, v)| a.checked_add(*v))?;
    let _ = total; // conservation is sign-conserved by construction (leaf law)
    for (_, v) in &merged {
        let _ = i64::try_from(*v).ok()?; // commit-phase narrowing refusal
    }
    pairs.sort_by_key(|((dd, cc), _)| (*dd, *cc));
    let mut out = Vec::new();
    let mut n_rows = 0u32;
    for ((dd, cc), v) in &pairs {
        if *v != 0 {
            out.extend_from_slice(&dd.to_le_bytes());
            out.extend_from_slice(&cc.to_le_bytes());
            out.extend_from_slice(&i64::try_from(*v).ok()?.to_le_bytes());
            n_rows += 1;
        }
    }
    let n_parties = merged.len() as u32;
    for (p, v) in &merged {
        out.extend_from_slice(&p.to_le_bytes());
        out.extend_from_slice(&i64::try_from(*v).ok()?.to_le_bytes());
    }
    // Re-emit with the header in front (the count is needed up front).
    let mut full = Vec::new();
    push_header(
        &mut full,
        neng::LEAF_ID_NETTING_ENGINE,
        fixed::TAG_MONEY,
        u64::from(n_rows) + u64::from(n_parties),
    );
    full.extend_from_slice(&n_rows.to_le_bytes());
    full.extend_from_slice(&n_parties.to_le_bytes());
    full.extend_from_slice(&out);
    Some(Expect::Admit(full))
}

/// SIMM oracle. Wire law: header(leaf=SIMM, any documented tag, dims ==
/// 2 + rows*cols + cols), rows 1..=256, cols 1..=64, len exact 16+dims*8.
/// Kernel law: y[i] = round_he(sum_k C[i][k]*w[k], scale), checked i128,
/// i64 narrowing refusal.
fn simm_law(b: &[u8]) -> Expect {
    simm_inner(b).unwrap_or(Expect::Refuse)
}

fn simm_inner(b: &[u8]) -> Option<Expect> {
    const MAX_ROWS: u64 = 256;
    const MAX_COLS: u64 = 64;
    let tags = [fixed::TAG_MONEY, fixed::TAG_RATIO, fixed::TAG_HHI, fixed::TAG_SHARES];
    let (leaf, tag, dims) = header_law(b, &tags).ok()?;
    if leaf != simm::LEAF_ID_SIMM_MATRIX {
        return None;
    }
    let scale: i128 = match tag {
        fixed::TAG_MONEY => 1_000_000,
        fixed::TAG_RATIO => 100_000_000,
        fixed::TAG_HHI => 10_000,
        _ => 1_000_000, // TAG_SHARES
    };
    if b.len() < 32 {
        return None;
    }
    let (rows, cols) = (r_u64(b, 16), r_u64(b, 24));
    if rows == 0 || rows > MAX_ROWS || cols == 0 || cols > MAX_COLS {
        return None;
    }
    if dims != 2u64.checked_add(rows.checked_mul(cols)?)?.checked_add(cols)? {
        return None;
    }
    let (r, c) = (rows as usize, cols as usize);
    if b.len() != 16 + (2 + r * c + c) * 8 {
        return None;
    }
    let matrix: Vec<i64> = (0..r * c).map(|i| r_i64(b, 32 + 8 * i)).collect();
    let vec: Vec<i64> = (0..c).map(|i| r_i64(b, 32 + 8 * r * c + 8 * i)).collect();
    let mut ys: Vec<i64> = Vec::new();
    for i in 0..r {
        let mut acc: i128 = 0;
        for k in 0..c {
            acc =
                acc.checked_add(i128::from(matrix[i * c + k]).checked_mul(i128::from(vec[k]))?)?;
        }
        let y = round_he(acc, scale)?;
        ys.push(i64::try_from(y).ok()?);
    }
    let mut out = Vec::new();
    push_header(&mut out, simm::LEAF_ID_SIMM_MATRIX, tag, r as u64);
    for y in &ys {
        out.extend_from_slice(&y.to_le_bytes());
    }
    Some(Expect::Admit(out))
}

/// Collateral LP oracle — genuinely independent of the leaf's simplex: 2D
/// vertex enumeration in exact i128 rationals. Given the fixture's bounded
/// flip band (low payload byte only), the corrupted LP is always feasible
/// (b stays positive, so the origin stays feasible) and bounded (all A
/// entries stay positive, so no nonzero recession ray exists), with a
/// generically unique optimum (objective gaps ~1e6 dwarf the <= 255
/// perturbations) — so the leaf MUST return the enumerated optimum, its
/// half-even rounding, and its exactness flags.
fn lp_law(b: &[u8]) -> Expect {
    lp_inner(b).unwrap_or(Expect::Refuse)
}

fn lp_inner(b: &[u8]) -> Option<Expect> {
    const MAX_VARS: u64 = 64;
    const MAX_CONS: u64 = 64;
    let tags = [fixed::TAG_MONEY, fixed::TAG_RATIO, fixed::TAG_HHI, fixed::TAG_SHARES];
    let (leaf, tag, dims) = header_law(b, &tags).ok()?;
    if leaf != lp::LEAF_ID_COLLATERAL_LP {
        return None;
    }
    let scale: i128 = match tag {
        fixed::TAG_MONEY => 1_000_000,
        fixed::TAG_RATIO => 100_000_000,
        fixed::TAG_HHI => 10_000,
        _ => 1_000_000, // TAG_SHARES
    };
    if b.len() < 32 {
        return None;
    }
    let (n, m) = (r_u64(b, 16), r_u64(b, 24));
    if n == 0 || n > MAX_VARS || m == 0 || m > MAX_CONS {
        return None;
    }
    if dims != 2u64.checked_add(n)?.checked_add(n.checked_mul(m)?)?.checked_add(m)? {
        return None;
    }
    let (nv, mv) = (n as usize, m as usize);
    if b.len() != 16 + (2 + nv + nv * mv + mv) * 8 {
        return None;
    }
    let c: Vec<i64> = (0..nv).map(|i| r_i64(b, 32 + 8 * i)).collect();
    let a: Vec<i64> = (0..nv * mv).map(|i| r_i64(b, 32 + 8 * nv + 8 * i)).collect();
    let rhs: Vec<i64> = (0..mv).map(|i| r_i64(b, 32 + 8 * (nv + nv * mv) + 8 * i)).collect();

    // The enumeration oracle is scoped to the 2x2 fixture family; other
    // declared arities are outside the sweep's flip band (n/m flips break
    // the dims law -> refusal, already handled above).
    if nv != 2 || mv != 2 {
        return None;
    }

    // Lines x1 = 0, x2 = 0, a_r . x = b_r, as (n1, n2, h).
    let lines: [(i128, i128, i128); 4] = [
        (1, 0, 0),
        (0, 1, 0),
        (i128::from(a[0]), i128::from(a[1]), i128::from(rhs[0])),
        (i128::from(a[2]), i128::from(a[3]), i128::from(rhs[1])),
    ];
    // Half-space membership for the rational point (px/d, py/d), d > 0:
    // x >= 0, y >= 0, and a_r0*px + a_r1*py <= b_r*d.
    let feasible = |px: i128, py: i128, d: i128| -> bool {
        if px < 0 || py < 0 {
            return false;
        }
        for r in 0..2 {
            let lhs = match i128::from(a[2 * r])
                .checked_mul(px)
                .and_then(|v| v.checked_add(i128::from(a[2 * r + 1]).checked_mul(py)?))
            {
                Some(v) => v,
                None => return false, // out-of-band: treat as refusal
            };
            match i128::from(rhs[r]).checked_mul(d) {
                Some(rv) if lhs <= rv => {}
                _ => return false,
            }
        }
        true
    };
    // Enumerate candidate vertices: pairwise line intersections (Cramer,
    // denominator normalized positive), keep the feasible maximizer.
    let mut best: Option<(i128, i128, i128, i128, i128)> = None; // (px, py, d, objnum, objden)
    for i in 0..4 {
        for j in (i + 1)..4 {
            let raw_det = match lines[i]
                .0
                .checked_mul(lines[j].1)
                .and_then(|v| v.checked_sub(lines[i].1.checked_mul(lines[j].0)?))
            {
                Some(v) => v,
                None => return None,
            };
            if raw_det == 0 {
                continue;
            }
            let (det, sgn) = if raw_det < 0 { (-raw_det, -1_i128) } else { (raw_det, 1_i128) };
            let x1 = match lines[i]
                .2
                .checked_mul(lines[j].1)
                .and_then(|v| v.checked_sub(lines[j].2.checked_mul(lines[i].1)?))
                .and_then(|v| v.checked_mul(sgn))
            {
                Some(v) => v,
                None => return None,
            };
            let x2 = match lines[i]
                .0
                .checked_mul(lines[j].2)
                .and_then(|v| v.checked_sub(lines[j].0.checked_mul(lines[i].2)?))
                .and_then(|v| v.checked_mul(sgn))
            {
                Some(v) => v,
                None => return None,
            };
            if !feasible(x1, x2, det) {
                continue;
            }
            // Objective: (c1*x1 + c2*x2) / (det * scale).
            let onum = match i128::from(c[0])
                .checked_mul(x1)
                .and_then(|v| v.checked_add(i128::from(c[1]).checked_mul(x2)?))
            {
                Some(v) => v,
                None => return None,
            };
            let oden = det.checked_mul(scale)?;
            let better = match best {
                None => true,
                Some((_, _, _, bn, bd)) => match onum.checked_mul(bd).zip(bn.checked_mul(oden)) {
                    Some((l, r)) => l > r,
                    None => return None,
                },
            };
            if better {
                best = Some((x1, x2, det, onum, oden));
            }
        }
    }
    let (px, py, d, _onum, _oden) = best?; // infeasible -> refusal (ERR_NO_SOLUTION)
                                           // Landed extract law (src/collateral_lp.rs `extract`): the vertex is
                                           // rounded half-even FIRST, and the objective is computed from the
                                           // ROUNDED vertex as round_he(sum(c_j * x_j), scale) — the old oracle
                                           // carried the objective as an exact rational over the unrounded vertex.
    let x0 = round_he(px, d)?;
    let x1 = round_he(py, d)?;
    let acc = match i128::from(c[0])
        .checked_mul(x0)
        .and_then(|v| v.checked_add(i128::from(c[1]).checked_mul(x1)?))
    {
        Some(v) => v,
        None => return None,
    };
    let objective = round_he(acc, scale)?;
    if x0 < i64::MIN as i128
        || x0 > i64::MAX as i128
        || x1 < i64::MIN as i128
        || x1 > i64::MAX as i128
        || objective < i64::MIN as i128
        || objective > i64::MAX as i128
    {
        return None;
    }
    let exact_vertex = px % d == 0 && py % d == 0;
    let exact_objective = acc.rem_euclid(scale) == 0;
    let mut out = Vec::new();
    push_header(&mut out, lp::LEAF_ID_COLLATERAL_LP, tag, (nv + 2) as u64);
    out.extend_from_slice(&(objective as i64).to_le_bytes());
    let mut flags = 0u8;
    if exact_vertex {
        flags |= 1;
    }
    if exact_objective {
        flags |= 2;
    }
    out.push(flags);
    out.extend_from_slice(&[0u8; 7]);
    out.extend_from_slice(&(x0 as i64).to_le_bytes());
    out.extend_from_slice(&(x1 as i64).to_le_bytes());
    Some(Expect::Admit(out))
}

/// Vote-reconciler oracle. Wire law: header(leaf=VOTE, TAG_SHARES,
/// dims 1..=64), len == 16 + 24d + 32 exact; rows (class u64, shares i64,
/// votes_for i64); thresholds 4 x i64, each fraction 0 < num <= den.
/// Kernel law: row domains (0 < shares <= 1e6, 0 <= votes <= shares,
/// checked sums); quorum total*qd >= qn*total_outstanding and supermajority
/// for*sd >= sn*total, with total_outstanding == total on this wire.
fn vote_law(b: &[u8]) -> Expect {
    vote_inner(b).unwrap_or(Expect::Refuse)
}

fn vote_inner(b: &[u8]) -> Option<Expect> {
    const MAX_ROWS: u64 = 64;
    let (leaf, tag, dims) = header_law(b, &[fixed::TAG_SHARES]).ok()?;
    if leaf != vote::LEAF_ID_VOTE_RECONCILER {
        return None;
    }
    if tag != fixed::TAG_SHARES || dims == 0 || dims > MAX_ROWS {
        return None;
    }
    let d = dims as usize;
    if b.len() != 16 + 24 * d + 32 {
        return None;
    }
    let (qn, qd, sn, sd) = (
        i128::from(r_i64(b, 16 + 24 * d)),
        i128::from(r_i64(b, 24 + 24 * d)),
        i128::from(r_i64(b, 32 + 24 * d)),
        i128::from(r_i64(b, 40 + 24 * d)),
    );
    let domain = |num: i128, den: i128| den > 0 && num > 0 && num <= den;
    if !domain(qn, qd) || !domain(sn, sd) {
        return None;
    }
    let mut total: i128 = 0;
    let mut for_shares: i128 = 0;
    for i in 0..d {
        let shares = i128::from(r_i64(b, 24 + 24 * i));
        let vf = i128::from(r_i64(b, 32 + 24 * i));
        if shares <= 0 || shares > 1_000_000 || vf < 0 || vf > shares {
            return None;
        }
        total = total.checked_add(shares)?;
        for_shares = for_shares.checked_add(vf)?;
    }
    let quorum_ok = total.checked_mul(qd)?.checked_sub(total.checked_mul(qn)?)?;
    let super_ok = for_shares.checked_mul(sd)?.checked_sub(total.checked_mul(sn)?)?;
    let admitted = quorum_ok >= 0 && super_ok >= 0;
    let mut out = Vec::new();
    push_header(&mut out, vote::LEAF_ID_VOTE_RECONCILER, fixed::TAG_SHARES, 4);
    out.extend_from_slice(&(for_shares as i64).to_le_bytes());
    out.extend_from_slice(&((total - for_shares) as i64).to_le_bytes());
    out.extend_from_slice(&(total as i64).to_le_bytes());
    out.extend_from_slice(&(total as i64).to_le_bytes()); // outstanding == total on this wire
    out.push(u8::from(admitted));
    out.extend_from_slice(&[0u8; 15]);
    Some(Expect::Admit(out))
}

/// Netting-settler oracle. Wire law: header(leaf=SETTLER, TAG_MONEY,
/// dims 1..=256), len == 16 + 4 + 24d exact; u32 n_parties 1..=16; rows
/// (payer u64, payee u64, amount i64); payer/payee <= u16::MAX and
/// < n_parties, amount > 0; zero-sum law in checked i128; per-party i64
/// narrowing refusal.
fn nset_law(b: &[u8]) -> Expect {
    nset_inner(b).unwrap_or(Expect::Refuse)
}

fn nset_inner(b: &[u8]) -> Option<Expect> {
    const MAX_PARTIES: usize = 16;
    let (leaf, tag, dims) = header_law(b, &[fixed::TAG_MONEY]).ok()?;
    if leaf != nset::LEAF_ID_NETTING_SETTLER {
        return None;
    }
    if tag != fixed::TAG_MONEY {
        return None;
    }
    if dims == 0 || dims > 256 || b.len() != 20 + 24 * dims as usize {
        return None;
    }
    let n_parties = u32::from_le_bytes(b[16..20].try_into().ok()?) as usize;
    if n_parties == 0 || n_parties > MAX_PARTIES {
        return None;
    }
    let mut scratch = vec![0_i128; n_parties];
    for i in 0..dims as usize {
        let payer = r_u64(b, 20 + 24 * i);
        let payee = r_u64(b, 28 + 24 * i);
        let amount = i128::from(r_i64(b, 36 + 24 * i));
        if payer > 65_535 || payee > 65_535 || amount <= 0 {
            return None;
        }
        if payer as usize >= n_parties || payee as usize >= n_parties {
            return None;
        }
        scratch[payee as usize] = scratch[payee as usize].checked_add(amount)?;
        scratch[payer as usize] = scratch[payer as usize].checked_sub(amount)?;
    }
    let mut net: i128 = 0;
    for v in &scratch {
        net = net.checked_add(*v)?;
    }
    if net != 0 {
        return None;
    }
    let mut out = Vec::new();
    push_header(&mut out, nset::LEAF_ID_NETTING_SETTLER, fixed::TAG_MONEY, n_parties as u64);
    out.extend_from_slice(&(n_parties as u32).to_le_bytes());
    for (i, v) in scratch.iter().enumerate() {
        out.extend_from_slice(&(i as u64).to_le_bytes());
        out.extend_from_slice(&i64::try_from(*v).ok()?.to_le_bytes());
    }
    Some(Expect::Admit(out))
}

/// Fractional-shares oracle. Wire law: header(leaf=FRAC, TAG_SHARES,
/// dims 1..=64), len == 16 + 8 + 8d exact; ratio i64 with 0 < ratio <=
/// 1e8 (num = ratio, den = S_RATIO); holders 0 <= s <= 1e6. Kernel law:
/// base = q/den floor, rem = q%den, leftover = sum_q/den - sum_base
/// (>= 0), top-ups by (remainder desc, index asc); conservation exact.
fn frac_law(b: &[u8]) -> Expect {
    frac_inner(b).unwrap_or(Expect::Refuse)
}

fn frac_inner(b: &[u8]) -> Option<Expect> {
    const MAX_HOLDERS: usize = 64;
    const S_RATIO: i128 = 100_000_000;
    const S_SHARES: i128 = 1_000_000;
    let (leaf, tag, dims) = header_law(b, &[fixed::TAG_SHARES]).ok()?;
    if leaf != frac::LEAF_ID_FRACTIONAL_SHARES {
        return None;
    }
    if tag != fixed::TAG_SHARES || dims == 0 || dims as usize > MAX_HOLDERS {
        return None;
    }
    let d = dims as usize;
    if b.len() != 24 + 8 * d {
        return None;
    }
    let ratio = i128::from(r_i64(b, 16));
    if ratio <= 0 || ratio > S_RATIO {
        return None;
    }
    let mut base: Vec<i128> = Vec::new();
    let mut rem: Vec<i128> = Vec::new();
    let mut sum_q: i128 = 0;
    for i in 0..d {
        let s = i128::from(r_i64(b, 24 + 8 * i));
        if s < 0 || s > S_SHARES {
            return None;
        }
        let q = s.checked_mul(ratio)?;
        base.push(q / S_RATIO);
        rem.push(q % S_RATIO);
        sum_q = sum_q.checked_add(q)?;
    }
    let sum_base: i128 = base.iter().try_fold(0_i128, |a, v| a.checked_add(*v))?;
    let leftover = match (sum_q / S_RATIO).checked_sub(sum_base) {
        Some(v) if v >= 0 => v,
        _ => return None,
    };
    // Deterministic top-up: order by (remainder desc, index asc).
    let mut order: Vec<usize> = (0..d).collect();
    order.sort_by(|&x, &y| rem[y].cmp(&rem[x]));
    let leftover = usize::try_from(leftover).ok()?;
    let mut whole = base.clone();
    for &idx in order.iter().take(leftover) {
        whole[idx] += 1;
    }
    // Conservation: sum(cash) + sum(whole * den) == sum_q (exact i128).
    // Landed law (RV3, src fractional_shares.rs:171-199): a topped-up holder
    // receives a whole share worth `den`, paid out of their cash — the
    // documented `fractional_cash_i = consideration_i - whole_shares_value_i`
    // (may be negative). An unadjusted `rem` would make chk = sum_q +
    // leftover*den and refuse every leftover>0 input, diverging from the
    // implementation.
    let mut is_top: Vec<bool> = vec![false; d];
    for &idx in order.iter().take(leftover) {
        is_top[idx] = true;
    }
    let mut chk: i128 = 0;
    let mut cash: Vec<i128> = Vec::with_capacity(d);
    for i in 0..d {
        let c_i = if is_top[i] { rem[i].checked_sub(S_RATIO)? } else { rem[i] };
        cash.push(c_i);
        chk = chk.checked_add(c_i)?.checked_add(whole[i].checked_mul(S_RATIO)?)?;
    }
    if chk != sum_q {
        return None;
    }
    let mut out = Vec::new();
    push_header(&mut out, frac::LEAF_ID_FRACTIONAL_SHARES, fixed::TAG_SHARES, d as u64);
    out.extend_from_slice(&(leftover as i64).to_le_bytes());
    for i in 0..d {
        out.extend_from_slice(&(*whole.get(i)? as i64).to_le_bytes());
        out.extend_from_slice(&(*cash.get(i)? as i64).to_le_bytes());
    }
    Some(Expect::Admit(out))
}

/// PN64-audit oracle. Wire law: header(leaf=PN64, TAG_SHARES, dims 1..=256),
/// len == 16 + 8 + 16d exact; u64 invariant_mask then d rows (pn, ocel).
/// Kernel law: the landed parent kernel is the leaf's specification —
/// pn_in(i) = pns[0] for i == 0 else step_branchless(pn_{i-1}, ocel_{i-1});
/// record i = committed(pn_in(i), ocel_i) || ocel_i || committed-flag ||
/// marking(pn_in(i)) || marking(committed).
fn pna_law(b: &[u8]) -> Expect {
    pna_inner(b).unwrap_or(Expect::Refuse)
}

fn pna_inner(b: &[u8]) -> Option<Expect> {
    use chicago_tdd_tools::conformance::pn64 as parent;
    const MAX_STEPS: u64 = 256;
    let (leaf, tag, dims) = header_law(b, &[fixed::TAG_SHARES]).ok()?;
    if leaf != pna::LEAF_ID_PN64_AUDIT {
        return None;
    }
    if tag != fixed::TAG_SHARES || dims == 0 || dims > MAX_STEPS {
        return None;
    }
    let d = dims as usize;
    if b.len() != 24 + 16 * d {
        return None;
    }
    let mask = r_u64(b, 16);
    let pns: Vec<u64> = (0..d).map(|i| r_u64(b, 24 + 16 * i)).collect();
    let ocels: Vec<u64> = (0..d).map(|i| r_u64(b, 32 + 16 * i)).collect();
    let mut out = Vec::new();
    push_header(&mut out, pna::LEAF_ID_PN64_AUDIT, fixed::TAG_SHARES, d as u64);
    for i in 0..d {
        let pn_in =
            if i == 0 { pns[0] } else { parent::step_branchless(pns[i - 1], ocels[i - 1], mask) };
        let committed = parent::step_branchless(pn_in, ocels[i], mask);
        out.extend_from_slice(&committed.to_le_bytes());
        out.extend_from_slice(&ocels[i].to_le_bytes());
        out.push(u8::from(parent::ct_eq(pn_in, committed) != 1));
        out.extend_from_slice(&parent::marking(pn_in).to_le_bytes());
        out.extend_from_slice(&parent::marking(committed).to_le_bytes());
    }
    Some(Expect::Admit(out))
}

// ---------------------------------------------------------------------------
// SEU courts — fast tier (10,000 flips per leaf) and full tier (1,000,000,
// #[ignore = "nato-tier3"], same seed: full subsumes fast)
// ---------------------------------------------------------------------------

const FAST_N: u64 = 10_000;
const FULL_N: u64 = 1_000_000;

/// Census law shared by every leaf court: exact class reconciliation,
/// SDC == 0, and both classes exercised (the court is vacuous if the sweep
/// never refused anything or never exercised the lawful path).
fn assert_court(flips: u64, c: Census) {
    assert_eq!(c.total(), flips, "every fault must land in exactly one class");
    assert_eq!(c.sdc, 0, "SDC must be 0 — a wrong-Ok, wrong-refusal, or oracle divergence");
    assert!(c.refused > 0, "court vacuous if no corruption was ever refused");
    assert!(c.lawful > 0, "court vacuous if no clean-path admission was exercised");
}

#[test]
fn seu_hhi_calc_never_wrong_ok_fast() {
    let c = run_seu_sweep(FAST_N, &hhi_clean(), all_flips_allowed, hhi_drive, hhi_law);
    assert_court(FAST_N, c);
}

#[test]
fn seu_netting_engine_never_wrong_ok_fast() {
    let c = run_seu_sweep(FAST_N, &neng_clean(), all_flips_allowed, neng_drive, neng_law);
    assert_court(FAST_N, c);
}

#[test]
fn seu_simm_matrix_never_wrong_ok_fast() {
    let c = run_seu_sweep(FAST_N, &simm_clean(), all_flips_allowed, simm_drive, simm_law);
    assert_court(FAST_N, c);
}

#[test]
fn seu_collateral_lp_never_wrong_ok_fast() {
    let c = run_seu_sweep(FAST_N, &lp_clean(), lp_flip_allowed, lp_drive, lp_law);
    // LP band is admit-heavy (bounded fixture): refused may be 0 here, so
    // only the reconciliation + SDC laws are asserted; the refusal space is
    // exercised by the cap tier and the header-band sweep below.
    assert_eq!(c.total(), FAST_N, "exactly-one-class reconciliation");
    assert_eq!(c.sdc, 0, "SDC must be 0 (LP bounded band)");
    assert!(c.lawful > 0, "court vacuous if no admission was exercised");
}

#[test]
fn seu_collateral_lp_header_band_refuses() {
    // Header band only (first 32 bytes, tag byte excepted): every flip
    // breaks the MGI1 law (magic / leaf / reserved / dims-vs-n-m), so the
    // outcome must be a refusal and NEVER a wrong-Ok. Byte 5 (scale tag) is
    // excluded because flipping TAG_MONEY to another DOCUMENTED tag is a
    // lawful re-scale the leaf admits by law (scale_from_tag table); that
    // admission is exercised (and must byte-match the oracle) in the main
    // sweep instead.
    let c = run_seu_sweep(2_000, &lp_clean(), |idx| idx < 32 && idx != 5, lp_drive, lp_law);
    assert_eq!(c.total(), 2_000);
    assert_eq!(c.sdc, 0, "header-band flip admitted");
    assert_eq!(c.lawful, 0, "header-band flip produced a lawful admission");
    assert_eq!(c.refused, 2_000, "header-band flips must all refuse");
}

#[test]
fn seu_vote_reconciler_never_wrong_ok_fast() {
    let c = run_seu_sweep(FAST_N, &vote_clean(), all_flips_allowed, vote_drive, vote_law);
    assert_court(FAST_N, c);
}

#[test]
fn seu_netting_settler_never_wrong_ok_fast() {
    let c = run_seu_sweep(FAST_N, &nset_clean(), all_flips_allowed, nset_drive, nset_law);
    assert_court(FAST_N, c);
}

#[test]
fn seu_fractional_shares_never_wrong_ok_fast() {
    let c = run_seu_sweep(FAST_N, &frac_clean(), all_flips_allowed, frac_drive, frac_law);
    assert_court(FAST_N, c);
}

#[test]
fn seu_pn64_audit_never_wrong_ok_fast() {
    let c = run_seu_sweep(FAST_N, &pna_clean(), all_flips_allowed, pna_drive, pna_law);
    assert_court(FAST_N, c);
}

/// Full tier: every leaf, donor scale, SAME seed — the fast streams above
/// are byte-for-byte prefixes of these.
#[test]
#[ignore = "nato-tier3"]
fn seu_all_leaves_never_wrong_ok_full() {
    let c = run_seu_sweep(FULL_N, &hhi_clean(), all_flips_allowed, hhi_drive, hhi_law);
    assert_eq!(c.sdc, 0, "hhi_calc: SDC at donor scale");
    let c = run_seu_sweep(FULL_N, &neng_clean(), all_flips_allowed, neng_drive, neng_law);
    assert_eq!(c.sdc, 0, "netting_engine: SDC at donor scale");
    let c = run_seu_sweep(FULL_N, &simm_clean(), all_flips_allowed, simm_drive, simm_law);
    assert_eq!(c.sdc, 0, "simm_matrix: SDC at donor scale");
    let c = run_seu_sweep(FULL_N, &lp_clean(), lp_flip_allowed, lp_drive, lp_law);
    assert_eq!(c.sdc, 0, "collateral_lp: SDC at donor scale");
    let c = run_seu_sweep(FULL_N, &vote_clean(), all_flips_allowed, vote_drive, vote_law);
    assert_eq!(c.sdc, 0, "vote_reconciler: SDC at donor scale");
    let c = run_seu_sweep(FULL_N, &nset_clean(), all_flips_allowed, nset_drive, nset_law);
    assert_eq!(c.sdc, 0, "netting_settler: SDC at donor scale");
    let c = run_seu_sweep(FULL_N, &frac_clean(), all_flips_allowed, frac_drive, frac_law);
    assert_eq!(c.sdc, 0, "fractional_shares: SDC at donor scale");
    let c = run_seu_sweep(FULL_N, &pna_clean(), all_flips_allowed, pna_drive, pna_law);
    assert_eq!(c.sdc, 0, "pn64_audit: SDC at donor scale");
}

// ---------------------------------------------------------------------------
// Positive control: a corrupted-accepting shim MUST be caught
// ---------------------------------------------------------------------------

/// MUTANT: a drive shim that admits ANY corrupted input with OK and the
/// clean fixture's canned output. The sweep machinery must classify its
/// behavior as SDC — proving the courts non-vacuous.
fn leaky_drive(_buf: &[u8]) -> (i32, Vec<u8>) {
    let mut out = vec![0xAA_u8; OUT_CAP];
    let clean = vote_clean();
    let n = clean.len().min(out.len());
    out[..n].copy_from_slice(&clean[..n]);
    (errno::OK, out)
}

#[test]
fn positive_control_leaky_drive_is_caught() {
    let c = run_seu_sweep(2_000, &vote_clean(), all_flips_allowed, leaky_drive, vote_law);
    assert!(c.sdc > 0, "corrupted-accepting shim was NOT caught — sweep machinery vacuous");
}

// ---------------------------------------------------------------------------
// Dimension / overflow / fuel tier (real ABI calls at the caps)
// ---------------------------------------------------------------------------

/// Re-derives the MGI1 header and payload prefix for the cap-tier inputs.
fn cap_probe(drive: impl Fn(&[u8]) -> (i32, Vec<u8>), buf: &[u8], want_ok: bool) -> i32 {
    let (st, _) = drive(buf);
    if want_ok {
        assert_eq!(st, errno::OK, "in-domain cap input refused");
    } else {
        assert_ne!(st, errno::OK, "over-cap input admitted");
        assert!(is_typed_refusal(st), "over-cap input refused untyped ({st})");
    }
    st
}

#[test]
fn dim_caps_refuse_typed() {
    // HHI: dims 0 and MAX_SHARES + 1 are both ERR_DIM.
    let zero = Mgi1Header { leaf_id: hhi::LEAF_ID_HHI_CALC, scale_tag: fixed::TAG_HHI, dims: 0 }
        .to_bytes();
    cap_probe(hhi_drive, &zero, false);
    let over =
        Mgi1Header { leaf_id: hhi::LEAF_ID_HHI_CALC, scale_tag: fixed::TAG_HHI, dims: 10_001 }
            .to_bytes();
    cap_probe(hhi_drive, &over, false);

    // Netting engine: dims 0 (with a full row body the dims law still fires
    // first — zero dims is the dimension refusal).
    let mut nb =
        Mgi1Header { leaf_id: neng::LEAF_ID_NETTING_ENGINE, scale_tag: fixed::TAG_MONEY, dims: 0 }
            .to_bytes()
            .to_vec();
    nb.extend_from_slice(&0_u64.to_le_bytes());
    nb.extend_from_slice(&0_u64.to_le_bytes());
    nb.extend_from_slice(&0_i64.to_le_bytes());
    cap_probe(neng_drive, &nb, false);
}

#[test]
fn fuel_and_dimension_boundary_holds_at_max_domain() {
    // HHI leaf at the full documented domain (10_000 shares, 80 KB body):
    // must succeed — the fuel stub cap (2*MAX_SHARES + 16) dominates the
    // 10_000-step worst case, so ERR_FUEL is unreachable-by-construction.
    let d = 10_000usize;
    let mut b =
        Mgi1Header { leaf_id: hhi::LEAF_ID_HHI_CALC, scale_tag: fixed::TAG_HHI, dims: d as u64 }
            .to_bytes()
            .to_vec();
    for i in 0..d {
        b.extend_from_slice(&((i % 97 + 1) as i64).to_le_bytes());
    }
    cap_probe(hhi_drive, &b, true);
}

#[test]
fn dimension_caps_refuse_across_leaves() {
    // Vote: 65 rows > MAX_ROWS = 64.
    let mut v = Mgi1Header {
        leaf_id: vote::LEAF_ID_VOTE_RECONCILER,
        scale_tag: fixed::TAG_SHARES,
        dims: 65,
    }
    .to_bytes()
    .to_vec();
    for i in 0..65 {
        v.extend_from_slice(&(i as u64).to_le_bytes());
        v.extend_from_slice(&100_i64.to_le_bytes());
        v.extend_from_slice(&50_i64.to_le_bytes());
    }
    v.extend_from_slice(&1_i64.to_le_bytes());
    v.extend_from_slice(&2_i64.to_le_bytes());
    v.extend_from_slice(&2_i64.to_le_bytes());
    v.extend_from_slice(&3_i64.to_le_bytes());
    cap_probe(vote_drive, &v, false);

    // Settler: n_parties 17 > MAX_PARTIES = 16 (ERR_DIM).
    let mut s =
        Mgi1Header { leaf_id: nset::LEAF_ID_NETTING_SETTLER, scale_tag: fixed::TAG_MONEY, dims: 1 }
            .to_bytes()
            .to_vec();
    s.extend_from_slice(&17_u32.to_le_bytes());
    s.extend_from_slice(&0_u64.to_le_bytes());
    s.extend_from_slice(&1_u64.to_le_bytes());
    s.extend_from_slice(&5_i64.to_le_bytes());
    cap_probe(nset_drive, &s, false);

    // Fractional: 65 holders > MAX_HOLDERS = 64.
    let mut f = Mgi1Header {
        leaf_id: frac::LEAF_ID_FRACTIONAL_SHARES,
        scale_tag: fixed::TAG_SHARES,
        dims: 65,
    }
    .to_bytes()
    .to_vec();
    f.extend_from_slice(&((fixed::S_RATIO / 2) as i64).to_le_bytes());
    for _ in 0..65 {
        f.extend_from_slice(&1_i64.to_le_bytes());
    }
    cap_probe(frac_drive, &f, false);

    // PN64: 257 steps > MAX_STEPS = 256.
    let mut p =
        Mgi1Header { leaf_id: pna::LEAF_ID_PN64_AUDIT, scale_tag: fixed::TAG_SHARES, dims: 257 }
            .to_bytes()
            .to_vec();
    p.extend_from_slice(&0_u64.to_le_bytes());
    for _ in 0..257 {
        p.extend_from_slice(&0_u64.to_le_bytes());
        p.extend_from_slice(&0_u64.to_le_bytes());
    }
    cap_probe(pna_drive, &p, false);

    // SIMM: rows 257 > MAX_ROWS = 256 (dims law consistent, cap refusal).
    let mut sm = Mgi1Header {
        leaf_id: simm::LEAF_ID_SIMM_MATRIX,
        scale_tag: fixed::TAG_RATIO,
        dims: 2 + 257 * 2 + 2,
    }
    .to_bytes()
    .to_vec();
    sm.extend_from_slice(&257_u64.to_le_bytes());
    sm.extend_from_slice(&2_u64.to_le_bytes());
    for _ in 0..(257 * 2 + 2) {
        sm.extend_from_slice(&0_i64.to_le_bytes());
    }
    cap_probe(simm_drive, &sm, false);

    // LP: n = 65 > MAX_VARS = 64 (dims law consistent, cap refusal).
    let mut l = Mgi1Header {
        leaf_id: lp::LEAF_ID_COLLATERAL_LP,
        scale_tag: fixed::TAG_MONEY,
        dims: 2 + 65 + 65 + 2,
    }
    .to_bytes()
    .to_vec();
    l.extend_from_slice(&65_u64.to_le_bytes());
    l.extend_from_slice(&2_u64.to_le_bytes());
    for _ in 0..(65 + 65 * 2 + 2) {
        l.extend_from_slice(&0_i64.to_le_bytes());
    }
    cap_probe(lp_drive, &l, false);
}

#[test]
fn overflow_refuses_typed() {
    // Netting engine: two identical pairs whose i128 scratch accumulation
    // overflows (i64::MAX + i64::MAX) -> ERR_OVERFLOW, never a wrap.
    let mut b =
        Mgi1Header { leaf_id: neng::LEAF_ID_NETTING_ENGINE, scale_tag: fixed::TAG_MONEY, dims: 2 }
            .to_bytes()
            .to_vec();
    for _ in 0..2 {
        b.extend_from_slice(&7_u64.to_le_bytes());
        b.extend_from_slice(&9_u64.to_le_bytes());
        b.extend_from_slice(&i64::MAX.to_le_bytes());
    }
    let st = cap_probe(neng_drive, &b, false);
    assert_eq!(st, errno::ERR_OVERFLOW, "pair-sum overflow must be ERR_OVERFLOW");
}

// ---------------------------------------------------------------------------
// Torn-input tier (audit wave: per-leaf torn framing courts)
//
// Three tearing modes over each leaf's maximal lawful input:
//   truncate-to-half  — the caller presents half the documented length;
//   zero-second-half  — full length presented, second half zeroed;
//   swap-halves       — full length presented, the two halves exchanged.
//
// Mode classification is by the per-leaf INDEPENDENT oracle (never by the
// leaf's own law): truncate and swap always break the framing law (exact
// length / magic-at-offset-0), so the leaf MUST refuse typed and leave the
// caller's buffer bit-exact 0xAA. The zero-half mode preserves length, so
// whether it is torn is decided by the leaf's domain law: where the oracle
// still admits the zeroed body (simm / netting-engine / fractional / pn64
// accept all-zero rows by law), the court demands a byte-exact oracle
// admission; where the oracle refuses (hhi / vote / settler / lp), it
// demands a typed refusal with the 0xAA law. Asserting a refusal for a
// lawfully-admissible zeroed body would be asserting a bug — so the court
// classifies against the oracle on BOTH branches and additionally proves
// non-vacuity: every leaf is driven, both branches are exercised, and no
// status ever lands outside the documented refusal space.
// ---------------------------------------------------------------------------

/// Torn-law classification: what the leaf must do on a mutated input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TornLaw {
    /// Documented law refuses: typed errno + bit-exact 0xAA output buffer.
    Refuse,
    /// Documented law admits: OK + byte-exact oracle output prefix.
    Admit,
}

/// Torn law per (leaf, mode), fixed by the oracles: modes 0 (truncate) and
/// 2 (swap) refuse for every leaf; mode 1 (zero-half) refuses exactly for
/// the leaves whose domain law rejects all-zero bodies (hhi sum==0, vote
/// threshold/row domains, settler amount>0 domain, lp unbounded on A=0).
const fn torn_mode_law(leaf: usize, mode: u8) -> TornLaw {
    if mode != 1 {
        return TornLaw::Refuse;
    }
    match leaf {
        0 | 4 | 5 | 3 => TornLaw::Refuse, // hhi, vote, nset, lp
        _ => TornLaw::Admit,              // neng, simm, frac, pna
    }
}

/// One leaf descriptor: (name, clean builder, drive, zero-half law index).
type LeafCase = (&'static str, fn() -> Vec<u8>, fn(&[u8]) -> (i32, Vec<u8>), usize);

#[test]
fn torn_inputs_refuse_across_leaves() {
    let leaves: [LeafCase; 8] = [
        ("hhi_calc", hhi_clean, hhi_drive, 0),
        ("netting_engine", neng_clean, neng_drive, 1),
        ("simm_matrix", simm_clean, simm_drive, 2),
        ("collateral_lp", lp_clean, lp_drive, 3),
        ("vote_reconciler", vote_clean, vote_drive, 4),
        ("netting_settler", nset_clean, nset_drive, 5),
        ("fractional_shares", frac_clean, frac_drive, 6),
        ("pn64_audit", pna_clean, pna_drive, 7),
    ];
    let mut refused = 0u32;
    let mut admitted = 0u32;
    for (name, clean_fn, drive, leaf) in leaves {
        let clean = clean_fn();
        for mode in 0..=2u8 {
            // For the leaves whose oracle is IN SCOPE on the zeroed body
            // (simm / neng / frac / pna — the all-zero rows are within the
            // fixture families the oracles enumerate), the admit law must
            // agree with the oracle. The lp oracle is scoped to the bounded
            // fixture family and cannot see unboundedness, so lp's zeroed
            // body (A=0 -> unbounded) is classified by the leaf's own
            // documented cap law (ERR_NO_SOLUTION), verified directly by
            // the refusal half-trial below.
            if matches!(leaf, 1 | 2 | 6 | 7) {
                let mutated_zero = {
                    let mut b = clean.clone();
                    let half = b.len() / 2;
                    b[half..].fill(0);
                    b
                };
                let oracle_zero = matches!(law_of(leaf)(&mutated_zero), Expect::Admit(_));
                assert_eq!(
                    torn_mode_law(leaf, 1) == TornLaw::Admit,
                    oracle_zero,
                    "{name}: zero-half law table disagrees with the oracle"
                );
            }

            match torn_mode_law(leaf, mode) {
                TornLaw::Refuse => {
                    torn_trial_refuse(clean.as_slice(), mode, drive);
                    refused += 1;
                }
                TornLaw::Admit => {
                    torn_trial_admit(&clean, mode, drive, law_of(leaf));
                    admitted += 1;
                }
            }
        }
    }
    assert_eq!(refused + admitted, 24, "8 leaves x 3 modes = 24 torn trials");
    assert!(refused >= 16, "truncate+swap must refuse on all 8 leaves");
    assert!(admitted >= 4, "zero-half must exercise the oracle-admit branch");
}

/// Drive helper: law table -> the leaf's oracle function.
fn law_of(leaf: usize) -> fn(&[u8]) -> Expect {
    match leaf {
        0 => hhi_law,
        1 => neng_law,
        2 => simm_law,
        3 => lp_law,
        4 => vote_law,
        5 => nset_law,
        6 => frac_law,
        _ => pna_law,
    }
}

/// Refusal half-trial: the leaf must refuse typed with a 0xAA buffer.
fn torn_trial_refuse(clean: &[u8], mode: u8, drive: impl Fn(&[u8]) -> (i32, Vec<u8>)) {
    let mutated: Vec<u8> = match mode {
        0 => clean[..clean.len() / 2].to_vec(),
        1 => {
            let mut b = clean.to_vec();
            let half = b.len() / 2;
            b[half..].fill(0);
            b
        }
        _ => {
            let half = clean.len() / 2;
            let mut b = clean.to_vec();
            b[..half].copy_from_slice(&clean[half..]);
            b[half..].copy_from_slice(&clean[..half]);
            b
        }
    };
    let (st, out) = drive(&mutated);
    assert_ne!(st, errno::OK, "torn input admitted (mode {mode})");
    assert!(is_typed_refusal(st), "torn input refused untyped (st={st}, mode {mode})");
    assert!(
        out.iter().all(|&b| b == 0xAA),
        "refusing call wrote to the caller buffer (mode {mode})"
    );
}

/// Admission half-trial: the leaf must admit with a byte-exact oracle match.
fn torn_trial_admit(
    clean: &[u8],
    mode: u8,
    drive: impl Fn(&[u8]) -> (i32, Vec<u8>),
    law: impl Fn(&[u8]) -> Expect,
) {
    let mut mutated = clean.to_vec();
    let half = mutated.len() / 2;
    mutated[half..].fill(0);
    match law(&mutated) {
        Expect::Admit(expected) => {
            let (st, out) = drive(&mutated);
            assert_eq!(st, errno::OK, "lawful zeroed body refused (mode {mode})");
            assert!(
                out.len() >= expected.len() && out[..expected.len()] == expected[..],
                "admitted zero-half input diverged from the oracle"
            );
        }
        Expect::Refuse => panic!("admit-classified trial with a refusing oracle"),
    }
}

#[test]
fn err_no_solution_driven_on_lp_and_settler() {
    // LP: an over-constrained row (x + y <= -1 with x, y >= 0) makes the
    // system infeasible — Phase I ends with w > 0: exact ERR_NO_SOLUTION.
    let mut l = lp_clean();
    let rhs0_off = 16 + 8 * (2 + 2 + 2 * 2); // header + n + m + c[2] + a[4]
    l[rhs0_off..rhs0_off + 8].copy_from_slice(&(-1_i64).to_le_bytes());
    let st = cap_probe(lp_drive, &l, false);
    assert_eq!(st, errno::ERR_NO_SOLUTION, "infeasible LP must be ERR_NO_SOLUTION");

    // Settler — HONEST SCOPE (audit correction): the audit asked for a
    // "party short by any amount" -> ERR_NO_SOLUTION, but the settler's
    // wire law conserves by construction (each instruction credits the
    // payee and debits the payer the same amount, so the 16-slot net is
    // identically zero; `NotZeroSum` is unreachable from the wire — the
    // crate's own comment at src/netting_settler.rs says the same). The
    // short-party input is therefore LAWFUL: the court asserts exact
    // conservation (payer -100 / payee +100) instead of a false refusal.
    let mut s =
        Mgi1Header { leaf_id: nset::LEAF_ID_NETTING_SETTLER, scale_tag: fixed::TAG_MONEY, dims: 1 }
            .to_bytes()
            .to_vec();
    s.extend_from_slice(&2_u32.to_le_bytes()); // n_parties
    s.extend_from_slice(&0_u64.to_le_bytes()); // payer
    s.extend_from_slice(&1_u64.to_le_bytes()); // payee
    s.extend_from_slice(&100_i64.to_le_bytes()); // amount
    let mut want = Vec::new();
    push_header(&mut want, nset::LEAF_ID_NETTING_SETTLER, fixed::TAG_MONEY, 2);
    want.extend_from_slice(&2_u32.to_le_bytes());
    want.extend_from_slice(&0_u64.to_le_bytes());
    want.extend_from_slice(&(-100_i64).to_le_bytes());
    want.extend_from_slice(&1_u64.to_le_bytes());
    want.extend_from_slice(&100_i64.to_le_bytes());
    let (st, out) = nset_drive(&s);
    assert_eq!(st, errno::OK, "conserving short-party input must be admitted");
    assert!(
        out.len() >= want.len() && out[..want.len()] == want[..],
        "settler net must conserve exactly (payer -100 / payee +100)"
    );
}

#[test]
fn err_overflow_driven_on_simm_and_hhi() {
    // HHI: two shares at i64::MAX — the i128 sum^2 term
    // (2*i64::MAX)^2 overflows checked i128: exact ERR_OVERFLOW.
    let mut h = Mgi1Header { leaf_id: hhi::LEAF_ID_HHI_CALC, scale_tag: fixed::TAG_HHI, dims: 2 }
        .to_bytes()
        .to_vec();
    for _ in 0..2 {
        h.extend_from_slice(&i64::MAX.to_le_bytes());
    }
    let st = cap_probe(hhi_drive, &h, false);
    assert_eq!(st, errno::ERR_OVERFLOW, "sum^2 i128 overflow must be ERR_OVERFLOW");

    // SIMM: 1x3 matvec with every entry i64::MAX — the i128 dot-product
    // accumulation 3 * i64::MAX^2 overflows checked i128: exact ERR_OVERFLOW.
    let (rows, cols) = (1usize, 3usize);
    let dims = (2 + rows * cols + cols) as u64;
    let mut b =
        Mgi1Header { leaf_id: simm::LEAF_ID_SIMM_MATRIX, scale_tag: fixed::TAG_RATIO, dims }
            .to_bytes()
            .to_vec();
    b.extend_from_slice(&(rows as u64).to_le_bytes());
    b.extend_from_slice(&(cols as u64).to_le_bytes());
    for _ in 0..(rows * cols + cols) {
        b.extend_from_slice(&i64::MAX.to_le_bytes());
    }
    let st = cap_probe(simm_drive, &b, false);
    assert_eq!(st, errno::ERR_OVERFLOW, "dot-product i128 overflow must be ERR_OVERFLOW");
}
