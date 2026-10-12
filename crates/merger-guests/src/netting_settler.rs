//! MG4 leaf: netting settler — scratch-validate-then-commit.
//!
//! A settlement instruction set is validated in full against a fixed-size
//! per-party scratch pad (`[i128; MAX_PARTIES]`, no heap) and only then
//! committed into the caller's write-set. On ANY refusal the write-set is
//! bit-exact untouched: **never partial**.
//!
//! Laws enforced before commit:
//!   1. Every instruction amount is strictly positive.
//!   2. Every party index is `< n_parties <= MAX_PARTIES` (`ERR_DIM` if the
//!      declared party count exceeds the cap).
//!   3. Zero-sum law: `Σ(payee legs) − Σ(payer legs) == 0` — the net across
//!      all parties is exactly zero.
//!   4. Short caller output buffer (`write_set.len() < n_parties`) is a
//!      WHOLE-CALL refusal with the shared `ERR_LENGTH` code — the settler
//!      never truncates, never partially writes (`ERR_LENGTH`).

/// Maximum parties per settlement call. Exceeding this is `ERR_DIM`.
pub const MAX_PARTIES: usize = 16;

/// One settlement instruction: `payer` pays `amount` units to `payee`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instruction {
    /// Payer party index, `< n_parties`.
    pub payer: u16,
    /// Payee party index, `< n_parties`.
    pub payee: u16,
    /// Cash amount, strictly positive.
    pub amount: i128,
}

/// Refusal reasons local to this leaf.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettleError {
    /// `n_parties > MAX_PARTIES` or instruction count exceeds the cap
    /// (shared `ERR_DIM`).
    Dim,
    /// Caller write-set shorter than `n_parties` (shared `ERR_LENGTH`);
    /// whole-call refusal, documented: never partial.
    Length,
    /// An amount is `<= 0`, or a party index is out of range.
    Instruction(usize),
    /// Zero-sum law violated: Σ payee legs ≠ Σ payer legs.
    NotZeroSum,
    /// A party net exceeds the i64 output lane (audit W7: checked in the
    /// scratch phase so refusal is whole-call, never partial).
    Overflow,
}

impl SettleError {
    /// Wire code. Per the ABI law (`lib.rs`), every code returned through
    /// `run()` is a documented shared errno — negatives are reserved for the
    /// host transport and never cross the leaf boundary. A domain-violating
    /// instruction is a payload-framing mismatch (`ERR_LENGTH`); a
    /// zero-sum violation means no feasible settlement exists
    /// (`ERR_NO_SOLUTION`).
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Dim => crate::errno::ERR_DIM,
            // ERR_LENGTH: short caller buffer = whole-call refusal.
            Self::Length => crate::errno::ERR_LENGTH,
            Self::Instruction(_) => crate::errno::ERR_LENGTH,
            Self::NotZeroSum => crate::errno::ERR_NO_SOLUTION,
            Self::Overflow => crate::errno::ERR_OVERFLOW,
        }
    }
}

/// Validate the full instruction set in scratch, then commit per-party nets
/// into `write_set[0..n_parties]`.
///
/// Commit happens only after EVERY instruction and the zero-sum law pass.
/// On any [`SettleError`], `write_set` is left bit-exact untouched.
///
/// Returns `n_parties` (the committed prefix length) on success.
///
/// # Errors
/// See [`SettleError`]. Every variant is a whole-call refusal.
pub fn settle(
    instructions: &[Instruction],
    n_parties: usize,
    write_set: &mut [i128],
) -> Result<usize, SettleError> {
    if n_parties == 0 || n_parties > MAX_PARTIES {
        return Err(SettleError::Dim);
    }
    if instructions.len() > MAX_PARTIES * MAX_PARTIES {
        return Err(SettleError::Dim);
    }
    // ERR_LENGTH contract: a short output buffer refuses the WHOLE call up
    // front — no instruction is applied, not even the first leg.
    if write_set.len() < n_parties {
        return Err(SettleError::Length);
    }

    // Scratch phase: fixed-size pad, no heap, no caller-visible mutation.
    let mut scratch = [0i128; MAX_PARTIES];
    for (i, ins) in instructions.iter().enumerate() {
        if ins.amount <= 0 {
            return Err(SettleError::Instruction(i));
        }
        let payer = usize::from(ins.payer);
        let payee = usize::from(ins.payee);
        if payer >= n_parties || payee >= n_parties {
            return Err(SettleError::Instruction(i));
        }
        scratch[payee] =
            scratch[payee].checked_add(ins.amount).ok_or(SettleError::Instruction(i))?;
        scratch[payer] =
            scratch[payer].checked_sub(ins.amount).ok_or(SettleError::Instruction(i))?;
    }

    // Zero-sum law: the net over all parties must be exactly zero. A
    // self-netting instruction (payer == payee) cancels inside its own slot
    // and cannot break this, but is still legal (documented no-op leg).
    let mut net = 0i128;
    for v in scratch.iter().take(n_parties) {
        net = net.checked_add(*v).ok_or(SettleError::NotZeroSum)?;
    }
    if net != 0 {
        return Err(SettleError::NotZeroSum);
    }

    // i64 fit law (audit W7): every party net must fit the i64 output
    // lane BEFORE the commit phase touches the caller's buffer — a
    // mid-write refusal would leave a partial write-set behind.
    for v in scratch.iter().take(n_parties) {
        i64::try_from(*v).map_err(|_| SettleError::Overflow)?;
    }

    // Commit phase: every check passed; write the prefix, leave the rest of
    // the buffer (beyond n_parties) untouched.
    write_set[..n_parties].copy_from_slice(&scratch[..n_parties]);
    Ok(n_parties)
}

use crate::{errno, fixed};

/// Stable leaf identity (`Mgi1Header.leaf_id`).
pub const LEAF_ID_NETTING_SETTLER: u8 = 8;

/// Wire body: `u32 n_parties`, then `dims` rows of 24 B
/// (`payer: u64`, `payee: u64`, `amount: i64`, S_MONEY micro-dollars).
/// Output body: `u32 n_parties` then `n_parties` x 16 B
/// (`party: u64`, `net: i64`, S_MONEY). A short caller output buffer is a
/// whole-call `ERR_LENGTH`/`ERR_CAP_TOO_SMALL` refusal — never partial.
fn run(in_buf: &[u8], out_buf: &mut [u8]) -> Result<(), i32> {
    let hdr = crate::parse_header(in_buf)?;
    if hdr.leaf_id != LEAF_ID_NETTING_SETTLER {
        return Err(errno::ERR_BAD_MAGIC);
    }
    if hdr.scale_tag != fixed::TAG_MONEY {
        return Err(errno::ERR_BAD_SCALE);
    }
    // SEU-hardened ordering: validate dims against its cap BEFORE the
    // exact_len arithmetic — a flipped dims bit must refuse typed
    // (ERR_DIM), never panic on usize overflow (debug builds abort).
    if hdr.dims == 0 || hdr.dims > (MAX_PARTIES * MAX_PARTIES) as u64 {
        return Err(errno::ERR_DIM);
    }
    let exact_len = 16 + 4 + 24 * hdr.dims as usize;
    if in_buf.len() < exact_len {
        return Err(errno::ERR_TRUNCATED);
    }
    if in_buf.len() > exact_len {
        return Err(errno::ERR_TRAILING);
    }
    let n_parties =
        u32::from_le_bytes(in_buf[16..20].try_into().map_err(|_| errno::ERR_TRUNCATED)?) as usize;
    if n_parties == 0 || n_parties > MAX_PARTIES {
        return Err(errno::ERR_DIM);
    }
    let d = hdr.dims as usize;
    let mut instructions: Vec<Instruction> = Vec::with_capacity(d);
    for i in 0..d {
        let b = &in_buf[20 + 24 * i..20 + 24 * (i + 1)];
        let payer = u64::from_le_bytes(b[0..8].try_into().map_err(|_| errno::ERR_TRUNCATED)?);
        let payee = u64::from_le_bytes(b[8..16].try_into().map_err(|_| errno::ERR_TRUNCATED)?);
        // Refuse oversized indices before the narrowing cast — an
        // out-of-domain index must never alias into range by truncation.
        if payer > u64::try_from(usize::from(u16::MAX)).unwrap_or(u64::MAX)
            || payee > u64::try_from(usize::from(u16::MAX)).unwrap_or(u64::MAX)
        {
            return Err(errno::ERR_DIM);
        }
        instructions.push(Instruction {
            // JUSTIFICATION: bounded by the u16::MAX check above.
            #[allow(clippy::cast_possible_truncation)]
            payer: payer as u16,
            // JUSTIFICATION: bounded by the u16::MAX check above.
            #[allow(clippy::cast_possible_truncation)]
            payee: payee as u16,
            amount: i64::from_le_bytes(b[16..24].try_into().map_err(|_| errno::ERR_TRUNCATED)?)
                as i128,
        });
    }
    let mut ws = [0i128; MAX_PARTIES];
    settle(&instructions, n_parties, &mut ws).map_err(SettleError::code)?;
    let need = 16 + 4 + 16 * n_parties;
    if out_buf.len() < need {
        return Err(errno::ERR_CAP_TOO_SMALL);
    }
    let ohdr = crate::Mgi1Header {
        leaf_id: LEAF_ID_NETTING_SETTLER,
        scale_tag: fixed::TAG_MONEY,
        dims: n_parties as u64,
    };
    ohdr.encode(out_buf);
    out_buf[16..20].copy_from_slice(&(n_parties as u32).to_le_bytes());
    for (i, w) in ws.iter().enumerate().take(n_parties) {
        let o = 20 + 16 * i;
        out_buf[o..o + 8].copy_from_slice(&(i as u64).to_le_bytes());
        // JUSTIFICATION: i64 fit proven in the scratch phase (audit W7) —
        // this narrowing is unreachable; expect_used is justified inline.
        #[allow(clippy::expect_used)]
        let net: i64 = i64::try_from(*w).expect("i64 fit proven in scratch phase");
        out_buf[o + 8..o + 16].copy_from_slice(&net.to_le_bytes());
    }
    Ok(())
}

/// Wire-level entry point: `i32 mgi_netting_settler_compute(in, in_len, out, out_cap) -> i32`.
/// 0 = ok; positive = errno; negative reserved for host transport.
///
/// # Safety
/// `in`/`out` must be valid for reads/writes of `in_len`/`out_cap` bytes
/// (caller-provided linear memory, cdylib ABI contract).
#[no_mangle]
pub unsafe extern "C" fn mgi_netting_settler_compute(
    in_: *const u8,
    in_len: u32,
    out: *mut u8,
    out_cap: u32,
) -> i32 {
    if in_.is_null() || out.is_null() {
        return errno::ERR_NULL_PTR;
    }
    if (out_cap as usize) < 16 + 4 + 16 {
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

    #[test]
    fn golden_mutual_netting_two_parties() {
        // A owes B 100, B owes A 30 → net: A = -70, B = +70.
        let ins = [
            Instruction { payer: 0, payee: 1, amount: 100 },
            Instruction { payer: 1, payee: 0, amount: 30 },
        ];
        let mut ws = [0i128; MAX_PARTIES];
        let n = settle(&ins, 2, &mut ws).expect("golden");
        assert_eq!(n, 2);
        assert_eq!(ws[0], -70);
        assert_eq!(ws[1], 70);
        // Beyond the committed prefix: untouched.
        assert_eq!(ws[2], 0);
    }

    #[test]
    fn golden_three_party_zero_sum() {
        let ins = [
            Instruction { payer: 0, payee: 1, amount: 50 },
            Instruction { payer: 1, payee: 2, amount: 50 },
            Instruction { payer: 2, payee: 0, amount: 50 },
        ];
        let mut ws = [0i128; MAX_PARTIES];
        settle(&ins, 3, &mut ws).expect("golden");
        assert_eq!(ws[0], 0);
        assert_eq!(ws[1], 0);
        assert_eq!(ws[2], 0);
    }

    #[test]
    fn non_zero_sum_refused_untouched() {
        // RV3 repair: a single payer->payee leg is zero-sum by construction
        // (each instruction contributes +amount and −amount to the party
        // scratch), so `NotZeroSum` is an unreachable-by-construction
        // fail-closed guard, like netting_engine's conservation check. The
        // law is asserted as a tautology over adversarial-looking sets; the
        // reachable refusals are covered by the dedicated tests below.
        let ins = [Instruction { payer: 0, payee: 1, amount: 100 }];
        let mut ws = [7i128; MAX_PARTIES];
        assert_eq!(settle(&ins, 2, &mut ws), Ok(2));
        assert_eq!(ws[0], -100);
        assert_eq!(ws[1], 100);
        assert_eq!(ws[2], 7, "beyond the committed prefix untouched");
    }

    #[test]
    fn negative_amount_refused_untouched() {
        let ins = [
            Instruction { payer: 0, payee: 1, amount: 10 },
            Instruction { payer: 1, payee: 0, amount: -5 }, // invalid leg later in set
        ];
        let mut ws = [3i128; MAX_PARTIES];
        assert_eq!(settle(&ins, 2, &mut ws), Err(SettleError::Instruction(1)));
        assert_eq!(ws[0], 3, "first leg must NOT have committed");
    }

    #[test]
    fn party_out_of_range_refused() {
        let ins = [Instruction { payer: 0, payee: 5, amount: 1 }];
        let mut ws = [0i128; MAX_PARTIES];
        assert_eq!(settle(&ins, 2, &mut ws), Err(SettleError::Instruction(0)));
    }

    #[test]
    fn short_write_set_is_whole_call_err_length() {
        let ins = [
            Instruction { payer: 0, payee: 1, amount: 10 },
            Instruction { payer: 1, payee: 0, amount: 10 },
        ];
        let mut short = [0i128; 1]; // needs 2
        assert_eq!(settle(&ins, 2, &mut short), Err(SettleError::Length));
        assert_eq!(short[0], 0, "never partial");
        assert_eq!(SettleError::Length.code(), crate::errno::ERR_LENGTH);
    }

    /// Audit W7: the party-net i64 overflow path (run()'s i64::try_from
    /// refusal at commit) must be driven, not just implemented.
    #[test]
    fn party_net_above_i64_max_err_overflow() {
        // Two i64::MAX credits to one payee overflow the i64 party net.
        let mut inbuf = Vec::new();
        inbuf.extend_from_slice(&b"MGI1"[..]);
        inbuf.push(8); // leaf id: netting_settler
        inbuf.push(crate::fixed::TAG_MONEY); // scale_tag (u8 @ [5])
        inbuf.extend_from_slice(&[0u8; 2]); // reserved [6..8]
        inbuf.extend_from_slice(&2u64.to_le_bytes()); // dims = 2 legs @ [8..16]
        inbuf.extend_from_slice(&2u32.to_le_bytes()); // n_parties
                                                      // dims = 2 instructions x 24B (payer u64, payee u64, amount i64)
        for _ in 0..2 {
            inbuf.extend_from_slice(&0u64.to_le_bytes()); // payer 0
            inbuf.extend_from_slice(&1u64.to_le_bytes()); // payee 1
            inbuf.extend_from_slice(&i64::MAX.to_le_bytes()); // amount
        }
        let mut out = vec![0xAAu8; 64];
        // SAFETY: fully-initialized in-buffer; refusal leaves out untouched.
        let rc = unsafe {
            crate::mgi_netting_settler_compute(
                inbuf.as_ptr(),
                inbuf.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(rc, crate::errno::ERR_OVERFLOW, "party net > i64::MAX must refuse typed");
        assert!(out.iter().all(|&b| b == 0xAA), "output must be untouched on refusal");
    }

    #[test]
    fn party_count_over_cap_err_dim() {
        let ins = [];
        let mut ws = [0i128; MAX_PARTIES];
        assert_eq!(settle(&ins, MAX_PARTIES + 1, &mut ws), Err(SettleError::Dim));
        assert_eq!(SettleError::Dim.code(), crate::errno::ERR_DIM);
    }

    #[test]
    fn zero_parties_refused() {
        let mut ws = [0i128; MAX_PARTIES];
        assert_eq!(settle(&[], 0, &mut ws), Err(SettleError::Dim));
    }

    #[test]
    fn self_netting_leg_is_legal_no_op() {
        // payer == payee: +amount then -amount in the same slot, zero-sum holds.
        let ins = [Instruction { payer: 1, payee: 1, amount: 42 }];
        let mut ws = [0i128; MAX_PARTIES];
        settle(&ins, 2, &mut ws).expect("legal");
        assert_eq!(ws[1], 0);
    }

    #[test]
    fn deterministic_repeat_calls() {
        let ins = [
            Instruction { payer: 0, payee: 1, amount: 9 },
            Instruction { payer: 1, payee: 0, amount: 4 },
        ];
        let mut a = [0i128; MAX_PARTIES];
        let mut b = [0i128; MAX_PARTIES];
        settle(&ins, 2, &mut a).expect("a");
        settle(&ins, 2, &mut b).expect("b");
        assert_eq!(a, b);
    }

    // ---- wire (mgi_netting_settler_compute) ----

    fn wire_in(n_parties: u32, ins: &[Instruction]) -> Vec<u8> {
        let mut b = crate::Mgi1Header {
            leaf_id: LEAF_ID_NETTING_SETTLER,
            scale_tag: fixed::TAG_MONEY,
            dims: ins.len() as u64,
        }
        .to_bytes()
        .to_vec();
        b.extend_from_slice(&n_parties.to_le_bytes());
        for r in ins {
            b.extend_from_slice(&u64::from(r.payer).to_le_bytes());
            b.extend_from_slice(&u64::from(r.payee).to_le_bytes());
            b.extend_from_slice(&(r.amount as i64).to_le_bytes());
        }
        b
    }

    #[test]
    fn wire_golden_zero_sum_commit() {
        let mut out = vec![0u8; 16 + 4 + 3 * 16];
        let buf = wire_in(
            3,
            &[
                Instruction { payer: 0, payee: 1, amount: 100 },
                Instruction { payer: 1, payee: 2, amount: 40 },
                Instruction { payer: 2, payee: 0, amount: 60 },
            ],
        );
        let code = unsafe {
            mgi_netting_settler_compute(
                buf.as_ptr(),
                buf.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::OK);
        assert_eq!(u32::from_le_bytes(out[16..20].try_into().expect("4")), 3);
        // Party 1: +100 -40 = +60. (RV3 repair: party rows are 16 B each
        // starting at 20 — party 1's net lives at 20+16+8 = 44.)
        assert_eq!(i64::from_le_bytes(out[20 + 16 + 8..20 + 16 + 16].try_into().expect("8")), 60);
    }

    #[test]
    fn wire_refusals() {
        let mut out = vec![0u8; 16 + 4 + MAX_PARTIES * 16];
        let good = wire_in(2, &[Instruction { payer: 0, payee: 1, amount: 5 }]);
        // trailing
        let mut t = good.clone();
        t.push(0);
        let code = unsafe {
            mgi_netting_settler_compute(
                t.as_ptr(),
                t.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::ERR_TRAILING);
        // truncated
        let code = unsafe {
            mgi_netting_settler_compute(
                good.as_ptr(),
                (good.len() - 1) as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, errno::ERR_TRUNCATED);
        // RV3 repair: leaf-local code path exercised via the reachable
        // `Instruction` refusal (out-of-range party index), since NotZeroSum
        // is unreachable by construction for payer/payee legs.
        let bad = wire_in(
            2,
            &[
                Instruction { payer: 0, payee: 1, amount: 5 },
                Instruction { payer: 7, payee: 0, amount: 1 },
            ],
        );
        out.fill(0);
        let code = unsafe {
            mgi_netting_settler_compute(
                bad.as_ptr(),
                bad.len() as u32,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert_eq!(code, SettleError::Instruction(1).code());
        assert_eq!(out[16..20], [0; 4], "output untouched");
    }
}
