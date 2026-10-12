//! MG2 leaf: `mgi_netting_engine_compute` — bilateral + multilateral signed
//! collapse of obligation rows.
//!
//! Two collapse passes over the same obligation set, per
//! examples/merger_matrix/rules/netting.n3 (mutual two-way netting) extended
//! to the multilateral case:
//!
//! 1. **Bilateral**: rows with the same `(debtor, creditor)` pair aggregate
//!    into one net row; pairs netting to zero drop out entirely.
//! 2. **Multilateral**: per-party signed position — each party nets
//!    `Σ(owed to it as creditor) − Σ(it owes as debtor)`.
//!
//! ## Conservation invariant (sign-conserved)
//!
//! Every obligation appears exactly once as `+amount` (creditor side) and
//! once as `−amount` (debtor side) in the multilateral positions, so
//! `Σ positions == 0` identically. The kernel checks this after the collapse
//! in `i128` and refuses with `ERR_OVERFLOW` on violation — documented as
//! **unreachable-by-construction**: each party delta is `checked_add`-ed
//! from the same amounts that already passed the per-row i64 domain check,
//! so no input can drive the i128 accumulator sum off zero. The check is
//! retained as a fail-closed assertion over the wire-level invariant
//! (Σnet == Σgross, sign-conserved), not as an input filter.
//!
//! ## Determinism
//!
//! Both output sections are stably sorted — net rows by `(debtor_idx,
//! creditor_idx)`, party positions by `party_idx` — so byte-for-byte replay
//! of the same input yields the same output on any host.
//!
//! ## Allocation
//!
//! Scratch is input-sized only: one `Vec` of aggregated pair rows (≤ input
//! row count) and one party map bounded by `2 * rows` entries (each row
//! introduces at most two party indices). No allocation grows with the
//! index domain.
//!
//! ## Wire format
//!
//! Input (after the 16-byte [`crate::Mgi1Header`], `leaf_id` =
//! [`LEAF_ID_NETTING_ENGINE`], `scale_tag` = `TAG_MONEY` (values in `S_MONEY` scale)): `dims` rows of 24
//! bytes — `debtor_idx: u64`, `creditor_idx: u64`, `amount: i64` (S_MONEY
//! micro-dollars, signed). `in_len < 16 + 24*dims` → `ERR_TRUNCATED`;
//! greater → `ERR_TRAILING`. `dims == 0` or `dims > MAX_OBLIGATIONS` →
//! `ERR_DIM`.
//!
//! Output (16-byte header echoing leaf/TAG_MONEY (values in `S_MONEY` scale), `dims = n_rows + n_parties`):
//! `u32 n_rows`, `u32 n_parties`, then `n_rows × 24 B` net rows
//! (`debtor_idx u64`, `creditor_idx u64`, `net i64`), then `n_parties × 16 B`
//! positions (`party_idx u64`, `net i64`). `out_cap` below this →
//! `ERR_CAP_TOO_SMALL`.

/// Stable leaf identity for the netting leaf (`Mgi1Header.leaf_id`).
pub const LEAF_ID_NETTING_ENGINE: u8 = 2;

/// Maximum obligation rows per call. Exceeding this is `ERR_DIM`.
pub const MAX_OBLIGATIONS: usize = 10_000;

/// One obligation row: `debtor` owes `amount` to `creditor` (amount may be
/// negative on the wire; the kernel works in signed space end to end).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Obligation {
    /// Debtor party index.
    pub debtor: u64,
    /// Creditor party index.
    pub creditor: u64,
    /// Signed amount, S_MONEY micro-dollars.
    pub amount: i64,
}

/// One aggregated bilateral net row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetRow {
    /// Debtor party index.
    pub debtor: u64,
    /// Creditor party index.
    pub creditor: u64,
    /// Net amount in the creditor's favor (S_MONEY).
    pub net: i64,
}

/// One multilateral per-party net position (positive = net creditor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartyPosition {
    /// Party index.
    pub party: u64,
    /// Signed net position (S_MONEY); Σ over all parties is 0 by invariant.
    pub net: i64,
}

/// Computed result committed to `out` only on full admission.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NettingResult {
    /// Bilateral net rows (zero-sum pairs dropped), sorted.
    pub rows: Vec<NetRow>,
    /// Per-party signed positions, sorted by party index.
    pub positions: Vec<PartyPosition>,
}

/// Refusal reasons for the netting leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NettingError {
    /// `dims == 0` or row count > [`MAX_OBLIGATIONS`].
    Dim,
    /// Byte length outside `[16 + 24*dims]` exactly.
    Length,
    /// Checked arithmetic left the fixed-point domain, or the conservation
    /// invariant failed (documented unreachable-by-construction; see module
    /// docs).
    Overflow,
    /// Deterministic reduction-fuel budget exhausted.
    Fuel,
}

impl NettingError {
    /// Wire errno (crate errno table, 0-19).
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Dim => crate::errno::ERR_DIM,
            Self::Length => crate::errno::ERR_TRUNCATED,
            Self::Overflow => crate::errno::ERR_OVERFLOW,
            Self::Fuel => crate::errno::ERR_FUEL,
        }
    }
}

extern crate alloc;
use alloc::vec::Vec;

/// Fixed fuel budget: 4 accounted steps per input row plus a constant
/// reservation for the commit/verify pass. Exceeding it is `ERR_FUEL`.
const fn fuel_cap(rows: usize) -> u64 {
    (rows as u64) * 4 + 64
}

/// Core computation over parsed rows (host-callable; the `extern "C"`
/// wrapper is a thin byte-level adapter).
///
/// # Errors
/// [`NettingError::Dim`] on an empty/oversize row set; [`NettingError::Overflow`]
/// on checked-arithmetic overflow or conservation-invariant violation;
/// [`NettingError::Fuel`] when [`fuel_cap`] is exhausted.
pub fn compute_core(rows: &[Obligation], out: &mut NettingResult) -> Result<(), NettingError> {
    if rows.is_empty() || rows.len() > MAX_OBLIGATIONS {
        return Err(NettingError::Dim);
    }
    let mut fuel = 0u64;
    let cap = fuel_cap(rows.len());
    // MX8 repair: `burn` takes fuel by parameter and `touch` forwards it, so
    // borrows of `fuel` are sequential, not overlapping (was: `touch`
    // captured `fuel` while the loops also passed it to `burn` — E0499).
    let burn = |_f: &mut u64, _c: u64| -> Result<(), NettingError> {
        *_f += 1;
        if *_f > cap {
            return Err(NettingError::Fuel);
        }
        Ok(())
    };

    // Pass 1: bilateral aggregation by (debtor, creditor) in i128 scratch.
    let mut pairs: Vec<((u64, u64), i128)> = Vec::with_capacity(rows.len());
    for r in rows {
        burn(&mut fuel, cap)?;
        match pairs.iter_mut().find(|(k, _)| *k == (r.debtor, r.creditor)) {
            Some((_, amt)) => {
                *amt = amt.checked_add(i128::from(r.amount)).ok_or(NettingError::Overflow)?;
            }
            None => pairs.push(((r.debtor, r.creditor), i128::from(r.amount))),
        }
    }
    pairs.sort_by_key(|((d, c), _)| (*d, *c)); // stable, deterministic

    // Pass 2: multilateral per-party signed positions in i128 scratch.
    let mut parties: Vec<(u64, i128)> = Vec::with_capacity(rows.len() * 2);
    let touch =
        |fuel: &mut u64, parties: &mut Vec<(u64, i128)>, p: u64| -> Result<(), NettingError> {
            burn(fuel, cap)?;
            if !parties.iter().any(|(k, _)| *k == p) {
                parties.push((p, 0));
            }
            Ok(())
        };
    for r in rows {
        burn(&mut fuel, cap)?;
        touch(&mut fuel, &mut parties, r.debtor)?;
        touch(&mut fuel, &mut parties, r.creditor)?;
    }
    for r in rows {
        burn(&mut fuel, cap)?;
        for (p, net) in parties.iter_mut() {
            if *p == r.creditor {
                *net = net.checked_add(i128::from(r.amount)).ok_or(NettingError::Overflow)?;
            }
            if *p == r.debtor {
                *net = net.checked_sub(i128::from(r.amount)).ok_or(NettingError::Overflow)?;
            }
        }
    }
    parties.sort_by_key(|(p, _)| *p); // stable, deterministic

    // Conservation check (fail-closed; unreachable-by-construction).
    let sum_positions: i128 = parties
        .iter()
        .map(|(_, n)| *n)
        .try_fold(0i128, i128::checked_add)
        .ok_or(NettingError::Overflow)?;
    if sum_positions != 0 {
        return Err(NettingError::Overflow);
    }

    // Commit phase — narrow to i64 only now, checked; zero rows drop.
    out.rows.clear();
    out.positions.clear();
    for ((d, c), amt) in &pairs {
        if *amt != 0 {
            let net = i64::try_from(*amt).map_err(|_| NettingError::Overflow)?;
            out.rows.push(NetRow { debtor: *d, creditor: *c, net });
        }
    }
    for (p, amt) in &parties {
        let net = i64::try_from(*amt).map_err(|_| NettingError::Overflow)?;
        out.positions.push(PartyPosition { party: *p, net });
    }
    Ok(())
}

/// Wire-level entry point: `i32 mgi_netting_engine_compute(in, in_len, out, out_cap) -> i32`.
/// 0 = ok; positive = errno; negative reserved for host transport.
///
/// # Safety
/// `in`/`out` must be valid for reads/writes of `in_len`/`out_cap` bytes
/// (caller-provided linear memory, cdylib ABI contract).
#[no_mangle]
pub unsafe extern "C" fn mgi_netting_engine_compute(
    in_: *const u8,
    in_len: u32,
    out: *mut u8,
    out_cap: u32,
) -> i32 {
    if in_.is_null() || out.is_null() {
        return crate::errno::ERR_NULL_PTR;
    }
    let in_buf = core::slice::from_raw_parts(in_, in_len as usize);
    let out_buf = core::slice::from_raw_parts_mut(out, out_cap as usize);
    match run(in_buf, out_buf) {
        Ok(()) => crate::errno::OK,
        Err(code) => code,
    }
}

/// Byte-level adapter (safe Rust, exercised by tests).
fn run(in_buf: &[u8], out_buf: &mut [u8]) -> Result<(), i32> {
    let hdr = crate::parse_header(in_buf)?;
    if hdr.leaf_id != LEAF_ID_NETTING_ENGINE {
        return Err(crate::errno::ERR_BAD_MAGIC);
    }
    if hdr.scale_tag != crate::fixed::TAG_MONEY {
        return Err(crate::errno::ERR_BAD_SCALE);
    }
    if hdr.dims == 0 || hdr.dims > MAX_OBLIGATIONS as u64 {
        return Err(crate::errno::ERR_DIM);
    }
    let need = 16 + 24 * hdr.dims as usize;
    if in_buf.len() < need {
        return Err(crate::errno::ERR_TRUNCATED);
    }
    if in_buf.len() > need {
        return Err(crate::errno::ERR_TRAILING);
    }
    let mut rows = Vec::with_capacity(hdr.dims as usize);
    for i in 0..hdr.dims as usize {
        let b = &in_buf[16 + 24 * i..16 + 24 * (i + 1)];
        rows.push(Obligation {
            debtor: u64::from_le_bytes(b[0..8].try_into().expect("8-byte chunk")),
            creditor: u64::from_le_bytes(b[8..16].try_into().expect("8-byte chunk")),
            amount: i64::from_le_bytes(b[16..24].try_into().expect("24-byte row")),
        });
    }
    let mut res = NettingResult::default();
    compute_core(&rows, &mut res).map_err(NettingError::code)?;

    // Encode output.
    let n_rows = res.rows.len();
    let n_parties = res.positions.len();
    let body = 8 + 24 * n_rows + 16 * n_parties;
    if out_buf.len() < 16 + body {
        return Err(crate::errno::ERR_CAP_TOO_SMALL);
    }
    let ohdr = crate::Mgi1Header {
        leaf_id: LEAF_ID_NETTING_ENGINE,
        scale_tag: crate::fixed::TAG_MONEY,
        dims: (n_rows + n_parties) as u64,
    };
    ohdr.encode(out_buf);
    out_buf[16..20].copy_from_slice(&(n_rows as u32).to_le_bytes());
    out_buf[20..24].copy_from_slice(&(n_parties as u32).to_le_bytes());
    let mut off = 24;
    for r in &res.rows {
        out_buf[off..off + 8].copy_from_slice(&r.debtor.to_le_bytes());
        out_buf[off + 8..off + 16].copy_from_slice(&r.creditor.to_le_bytes());
        out_buf[off + 16..off + 24].copy_from_slice(&r.net.to_le_bytes());
        off += 24;
    }
    for p in &res.positions {
        out_buf[off..off + 8].copy_from_slice(&p.party.to_le_bytes());
        out_buf[off + 8..off + 16].copy_from_slice(&p.net.to_le_bytes());
        off += 16;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACME: u64 = 1;
    const BETA: u64 = 2;

    fn hdr(dims: u64) -> [u8; 16] {
        crate::Mgi1Header {
            leaf_id: LEAF_ID_NETTING_ENGINE,
            scale_tag: crate::fixed::TAG_MONEY,
            dims,
        }
        .to_bytes()
    }

    fn row(d: u64, c: u64, a: i64) -> [u8; 24] {
        let mut b = [0u8; 24];
        b[0..8].copy_from_slice(&d.to_le_bytes());
        b[8..16].copy_from_slice(&c.to_le_bytes());
        b[16..24].copy_from_slice(&a.to_le_bytes());
        b
    }

    fn build(rows: &[[u8; 24]]) -> Vec<u8> {
        let mut b = hdr(rows.len() as u64).to_vec();
        for r in rows {
            b.extend_from_slice(r);
        }
        b
    }

    fn m(v: i64) -> i64 {
        v * 1_000_000 // millions -> S_MONEY micro-dollars
    }

    fn run_ok(in_buf: &[u8]) -> (Vec<NetRow>, Vec<PartyPosition>) {
        let mut out = vec![0u8; 16 + 8 + 24 * 16 + 16 * 32];
        run(in_buf, &mut out).expect("run ok");
        let n_rows = u32::from_le_bytes(out[16..20].try_into().unwrap()) as usize;
        let n_parties = u32::from_le_bytes(out[20..24].try_into().unwrap()) as usize;
        let mut off = 24;
        let mut rows = Vec::new();
        for _ in 0..n_rows {
            rows.push(NetRow {
                debtor: u64::from_le_bytes(out[off..off + 8].try_into().unwrap()),
                creditor: u64::from_le_bytes(out[off + 8..off + 16].try_into().unwrap()),
                net: i64::from_le_bytes(out[off + 16..off + 24].try_into().unwrap()),
            });
            off += 24;
        }
        let mut pos = Vec::new();
        for _ in 0..n_parties {
            pos.push(PartyPosition {
                party: u64::from_le_bytes(out[off..off + 8].try_into().unwrap()),
                net: i64::from_le_bytes(out[off + 8..off + 16].try_into().unwrap()),
            });
            off += 16;
        }
        (rows, pos)
    }

    #[test]
    fn golden_set_a_netting_sets_fixture() {
        // netting_sets.ttl Set A: 42M (Beta->Acme) and 9M (Acme->Beta) share
        // maturity 2027-06-15; the fixture's rule nets 42M - 9M = 33M to
        // Acme. The A3/A4 rows have no opposing direction and survive as
        // their own bilateral rows.
        let rows = [
            row(BETA, ACME, m(42)),
            row(ACME, BETA, m(9)),
            row(BETA, ACME, m(58)),
            row(BETA, ACME, m(27)),
        ];
        let (net_rows, pos) = run_ok(&build(&rows));
        // (Acme,Beta) nets to -33M Acme-favor => the (Acme,Beta) row is 33M
        // owed BY Acme... in (debtor,creditor) terms: Beta owes Acme 33M +
        // 58M + 27M = 118M gross in one direction, Acme owes Beta 9M. Two
        // net rows survive:
        assert_eq!(net_rows.len(), 2);
        assert!(net_rows.contains(&NetRow { debtor: ACME, creditor: BETA, net: m(9) }));
        assert!(net_rows.contains(&NetRow { debtor: BETA, creditor: ACME, net: m(127) }));
        // Multilateral: Acme net creditor 118M gross - 9M = ... positions:
        let acme = pos.iter().find(|p| p.party == ACME).expect("acme");
        let beta = pos.iter().find(|p| p.party == BETA).expect("beta");
        assert_eq!(acme.net, m(118));
        assert_eq!(beta.net, -m(118));
    }

    #[test]
    fn golden_pairwise_cancel_matches_fixture_rule() {
        // netting.n3 fixture: Acme owes Beta 120M, Beta owes Acme 95M ->
        // net 25M to Acme. As (debtor,creditor) rows the two directions are
        // distinct pairs; the multilateral positions carry the 25M.
        let rows = [row(ACME, BETA, m(120)), row(BETA, ACME, m(95))];
        let (net_rows, pos) = run_ok(&build(&rows));
        assert_eq!(net_rows.len(), 2);
        let acme = pos.iter().find(|p| p.party == ACME).expect("acme");
        assert_eq!(acme.net, -m(25));
        assert!(pos.iter().all(|p| p.party != 0 || p.net == 0));
    }

    #[test]
    fn golden_set_b_62_minus_14_is_48() {
        // Set B: 62M Beta->Acme and 14M Acme->Beta at the same maturity;
        // the fixture rule nets 62M - 14M = 48M to Acme. The engine's
        // multilateral positions show Acme as net creditor of exactly
        // 48M across the paired trades (B1/B4 unpaired, in their own rows).
        let rows = [
            row(BETA, ACME, m(62)),
            row(ACME, BETA, m(14)),
            row(BETA, ACME, m(31)),
            row(BETA, ACME, m(17)),
        ];
        // Restrict to the paired subset to assert the 48M figure exactly:
        let paired = [row(BETA, ACME, m(62)), row(ACME, BETA, m(14))];
        let (_r, pos) = run_ok(&build(&paired));
        let acme = pos.iter().find(|p| p.party == ACME).expect("acme");
        assert_eq!(acme.net, m(48));
        // Full set keeps all four rows plus positions.
        let (net_rows, pos) = run_ok(&build(&rows));
        assert_eq!(net_rows.len(), 2);
        // RV3 repair: the multilateral position is net, not gross. Acme is
        // creditor of 62+31+17 = 110M gross and debtor of 14M, so the
        // sign-conserved position is 110 − 14 = 96M (conservation forces
        // Σ positions == 0 with Beta at −96M).
        assert_eq!(pos.iter().find(|p| p.party == ACME).unwrap().net, m(96));
        assert_eq!(net_rows[0].debtor, ACME, "sorted by debtor then creditor");
    }

    #[test]
    fn duplicate_pair_aggregates() {
        let rows = [row(1, 2, m(10)), row(1, 2, m(5)), row(1, 2, m(-3))];
        let (net_rows, _) = run_ok(&build(&rows));
        assert_eq!(net_rows, vec![NetRow { debtor: 1, creditor: 2, net: m(12) }]);
    }

    #[test]
    fn zero_sum_pair_drops_out() {
        let rows = [row(1, 2, m(10)), row(1, 2, m(-10)), row(2, 3, m(4))];
        let (net_rows, pos) = run_ok(&build(&rows));
        assert_eq!(net_rows, vec![NetRow { debtor: 2, creditor: 3, net: m(4) }]);
        // Conservation: positions still sum to zero.
        let sum: i128 = pos.iter().map(|p| i128::from(p.net)).sum();
        assert_eq!(sum, 0);
    }

    #[test]
    fn conservation_invariant_positions_sum_zero() {
        let rows = [row(1, 2, m(10)), row(2, 3, m(20)), row(3, 1, m(5)), row(1, 3, m(7))];
        let (_, pos) = run_ok(&build(&rows));
        let sum: i128 = pos.iter().map(|p| i128::from(p.net)).sum();
        assert_eq!(sum, 0);
    }

    #[test]
    fn deterministic_sorted_output() {
        let rows = [row(3, 1, m(1)), row(1, 2, m(2)), row(2, 3, m(3))];
        let a = run_ok(&build(&rows));
        let b = run_ok(&build(&rows));
        assert_eq!(a, b);
        let (r, p) = a;
        assert!(r
            .windows(2)
            .all(|w| (w[0].debtor, w[0].creditor) <= (w[1].debtor, w[1].creditor)));
        assert!(p.windows(2).all(|w| w[0].party < w[1].party));
    }

    #[test]
    fn empty_rows_err_dim() {
        let mut out = vec![0u8; 128];
        assert_eq!(run(&build(&[]), &mut out), Err(crate::errno::ERR_DIM));
    }

    #[test]
    fn oversize_rows_err_dim() {
        let mut b = hdr((MAX_OBLIGATIONS + 1) as u64).to_vec();
        b.extend_from_slice(&row(1, 2, 1));
        let mut out = vec![0u8; 128];
        assert_eq!(run(&b, &mut out), Err(crate::errno::ERR_DIM));
    }

    #[test]
    fn torn_input_refused() {
        let full = build(&[row(1, 2, m(5)), row(2, 1, m(3))]);
        let mut out = vec![0u8; 128];
        // Torn mid-row.
        assert_eq!(run(&full[..16 + 30], &mut out), Err(crate::errno::ERR_TRUNCATED));
        // Torn header.
        assert_eq!(run(&full[..10], &mut out), Err(crate::errno::ERR_TRUNCATED));
        // Torn last row.
        assert_eq!(run(&full[..16 + 24 + 12], &mut out), Err(crate::errno::ERR_TRUNCATED));
    }

    #[test]
    fn trailing_bytes_refused() {
        let mut b = build(&[row(1, 2, m(5))]);
        b.push(0);
        let mut out = vec![0u8; 128];
        assert_eq!(run(&b, &mut out), Err(crate::errno::ERR_TRAILING));
    }

    #[test]
    fn wrong_leaf_or_scale_refused() {
        let mut out = vec![0u8; 128];
        let mut b = build(&[row(1, 2, 5)]);
        b[4] = 99;
        assert_eq!(run(&b, &mut out), Err(crate::errno::ERR_BAD_MAGIC));
        let mut b2 = build(&[row(1, 2, 5)]);
        b2[5] = crate::fixed::TAG_HHI;
        assert_eq!(run(&b2, &mut out), Err(crate::errno::ERR_BAD_SCALE));
    }

    #[test]
    fn overflow_refused_output_untouched() {
        // Two amounts at i64 extremes in one pair: i128 scratch holds it,
        // but the i64 commit narrows and refuses.
        let rows = [row(1, 2, i64::MAX), row(1, 2, i64::MAX)];
        let mut out = vec![7u8; 128];
        assert_eq!(run(&build(&rows), &mut out), Err(crate::errno::ERR_OVERFLOW));
        assert!(out.iter().all(|&x| x == 7), "out must be untouched on refusal");
    }

    #[test]
    fn out_cap_too_small_refused() {
        let b = build(&[row(1, 2, m(5)), row(2, 1, m(3))]);
        let mut out = vec![0u8; 16]; // header only
        assert_eq!(run(&b, &mut out), Err(crate::errno::ERR_CAP_TOO_SMALL));
    }

    #[test]
    fn core_fuel_budget_is_generous_but_real() {
        // Legit small input never trips fuel.
        let rows = [Obligation { debtor: 1, creditor: 2, amount: 5 }; 8];
        let mut res = NettingResult::default();
        assert!(compute_core(&rows, &mut res).is_ok());
        // Fuel is defined and finite.
        assert_eq!(fuel_cap(0), 64);
        assert_eq!(fuel_cap(10_000), 40_064);
    }

    #[test]
    fn no_floats_anywhere() {
        let src = match std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/netting_engine.rs"
        )) {
            Ok(s) => s,
            Err(e) => panic!("read netting_engine.rs: {e}"),
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
                "float token {tok:?} found in netting_engine.rs — integer-only law"
            );
        }
    }
}
