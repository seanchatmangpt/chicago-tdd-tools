//! Cooperative-yield protocol between the WASM guest and the BEAM scheduler (PRD §2).
//!
//! # The temprun contract
//!
//! The guest **never blocks the VM**. Every entry into guest code runs for at most
//! one bounded reduction slice (`DEFAULT_REDUCTION_QUOTA`), then returns an honest
//! status code to the host. Exceeding thermal or tick ceilings is the **supervisor's
//! termination call** — that decision lives on the OTP side, outside this module.
//! The guest's entire duty under this contract is:
//!
//! 1. bounded slices — never run longer than the quota handed to it, and
//! 2. honest status — return the accurate errno from the table below rather than
//!    masking a problem behind `STATUS_OK`.
//!
//! Anything beyond that (preemption, kill, thermal backoff) is host policy, not
//! guest behavior.
//!
//! # Build surface (v1 decisions, per coordinator E12 probe)
//!
//! - **Feature deps**: guest builds depend on `wasm-guest` + `conformance` +
//!   `ledger` only — NEVER `nato`. Verified: `src/ledger/mod.rs` gates
//!   `pub mod signing` under `#[cfg(feature = "nato")]`, and `signing/keys.rs`
//!   is the sole wasm32 compile blocker (`std::os::unix::PermissionsExt`).
//!   Ledger without nato does not compile signing, so it is wasm32-clean.
//! - **Default features**: try `default-features = false` first; if the parent
//!   crate's logging feature (tracing) or anything else breaks the wasm32
//!   build, keeping default features is acceptable — std compiles on
//!   wasm32-unknown-unknown, only fs/process calls fail at runtime. Pick
//!   whichever builds and document the choice.
//! - **Linear-memory ABI**: unlike the strata `pn_kernel` precedent
//!   (`no_std` + memory stripped post-build via `strip_memory.py`), v1 KEEPS
//!   guest linear memory — deliberate, following the temprun `src/abi.rs`
//!   precedent.
//! - **Artifact convention**: `ctt_wasm_guest.wasm` under
//!   `target/wasm32-unknown-unknown/release/`, per strata's on-disk discovery.
//!
//! # Canonical source of the errno table
//!
//! The canonical ABI errno table lives in `guest.rs` (`ERR_*` 0..=13, doc
//! table in its module header). This module owns ONLY the slice-scheduling
//! protocol. [`STATUS_OK`] is an alias of the canonical `guest::OK` so the
//! two surfaces share one zero. The remaining `STATUS_*` consts are
//! **yield-scheduling statuses, NOT ABI errnos**: their numeric values
//! intentionally do NOT match the canonical table's occupants at the same
//! numbers, and they must never cross the wire as ABI errnos. If a yield
//! status ever needs to appear in an ABI errno position, extend `guest.rs`'s
//! table there — never renumber these to coincide with `ERR_*`.
#![cfg(feature = "wasm-guest")]

/// Default reduction quota per guest slice (PRD step-slice): ~2000 reductions,
/// matching the BEAM's classical per-process reduction budget.
pub const DEFAULT_REDUCTION_QUOTA: u32 = 2000;

/// Hard ceiling on any quota the host may hand to a guest slice.
pub const MAX_REDUCTION_QUOTA: u32 = 1_000_000;

/// Guest slice completed within quota; result valid.
///
/// Alias of the canonical ABI success code `guest::OK` (0) — one zero for
/// both surfaces.
pub use super::guest::OK as STATUS_OK;

/// Guest slice exhausted its reduction quota before completing (host should reschedule).
///
/// Yield-scheduling status, NOT an ABI errno (canonical `ERR_*` 1 is
/// `ERR_NULL_PTR`; no semantic equivalent exists in the canonical table).
pub const STATUS_E_INTERRUPTED: i32 = 1;

/// Guest rejected a malformed or out-of-contract request from the host.
///
/// Yield-scheduling status, NOT an ABI errno (canonical `ERR_*` 2 is
/// `ERR_BAD_MAGIC`; no semantic equivalent exists in the canonical table).
pub const STATUS_E_INVALID: i32 = 2;

/// Guest asked for a resource (memory, table, fd) the host has not granted.
///
/// Yield-scheduling status, NOT an ABI errno. Distinct from the canonical
/// `ERR_CAP_TOO_SMALL` (11), which is the ABI form of "out-buffer too small".
pub const STATUS_E_NOMEM: i32 = 3;

/// Trap/panic inside guest code; guest state is poisoned for this slice.
///
/// Yield-scheduling status, NOT an ABI errno (no equivalent in the
/// canonical table).
pub const STATUS_E_TRAP: i32 = 4;

/// Guest detected a thermal or tick ceiling breach and yielded early (advisory;
/// the supervisor still owns the termination decision).
///
/// Yield-scheduling status, NOT an ABI errno (no equivalent in the
/// canonical table).
pub const STATUS_E_THERMAL: i32 = 5;

/// Guest does not implement the requested protocol version/capability.
///
/// Yield-scheduling status, NOT an ABI errno (no equivalent in the
/// canonical table).
pub const STATUS_E_UNSUPPORTED: i32 = 6;

/// Guest refused: caller lacks authority for the requested operation.
///
/// Yield-scheduling status, NOT an ABI errno (no equivalent in the
/// canonical table).
pub const STATUS_E_UNAUTHORIZED: i32 = 7;

/// Deadline for the slice passed before completion.
///
/// Yield-scheduling status, NOT an ABI errno (no equivalent in the
/// canonical table).
pub const STATUS_E_TIMEDOUT: i32 = 8;

/// Internal guest error not covered by any more specific status.
///
/// Yield-scheduling status, NOT an ABI errno (no equivalent in the
/// canonical table).
pub const STATUS_E_INTERNAL: i32 = 9;

/// A host-provided yield directive: how many reductions this slice may burn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YieldDirective {
    /// Reduction budget for the slice. Law: `0 < quota <= MAX_REDUCTION_QUOTA`.
    pub quota: u32,
}

/// Typed refusal for a quota outside the contract `0 < quota <= MAX_REDUCTION_QUOTA`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaRefusal {
    /// `quota == 0` — a slice that may run zero reductions can never make progress.
    ZeroQuota,
    /// `quota > MAX_REDUCTION_QUOTA` — exceeds the host's hard ceiling.
    OverCeiling,
}

impl YieldDirective {
    /// Construct a directive, refusing zero or over-ceiling quotas.
    ///
    /// # Errors
    ///
    /// [`QuotaRefusal::ZeroQuota`] when `quota == 0`; [`QuotaRefusal::OverCeiling`]
    /// when `quota > MAX_REDUCTION_QUOTA`.
    pub const fn new(quota: u32) -> Result<Self, QuotaRefusal> {
        match quota {
            0 => Err(QuotaRefusal::ZeroQuota),
            q if q > MAX_REDUCTION_QUOTA => Err(QuotaRefusal::OverCeiling),
            q => Ok(Self { quota: q }),
        }
    }
}

/// Adaptive quota policy placeholder. **The BEAM supervisor owns the policy**;
/// Adaptive quota policy placeholder.
///
/// The BEAM supervisor owns the policy; this function exists only so
/// guest-side code has a named seam to call. The host is expected to override
/// it — v1 returns the constant [`DEFAULT_REDUCTION_QUOTA`] unconditionally,
/// ignoring `previous`.
#[must_use]
pub const fn next_quota(_previous: u32) -> u32 {
    DEFAULT_REDUCTION_QUOTA
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accepts_default_quota() {
        let d = YieldDirective::new(DEFAULT_REDUCTION_QUOTA).unwrap_or_else(|e| {
            panic!("default quota must be valid, got {:?}", e);
        });
        assert_eq!(d.quota, DEFAULT_REDUCTION_QUOTA);
    }

    #[test]
    fn test_accepts_max_quota_boundary() {
        let d = YieldDirective::new(MAX_REDUCTION_QUOTA)
            .unwrap_or_else(|e| panic!("max quota is inclusive boundary, got {:?}", e));
        assert_eq!(d.quota, MAX_REDUCTION_QUOTA);
    }

    #[test]
    fn test_accepts_one_reduction() {
        let d = YieldDirective::new(1)
            .unwrap_or_else(|e| panic!("quota 1 is the inclusive lower bound, got {:?}", e));
        assert_eq!(d.quota, 1);
    }

    #[test]
    fn test_refuses_zero_quota() {
        assert_eq!(YieldDirective::new(0), Err(QuotaRefusal::ZeroQuota));
    }

    #[test]
    fn test_refuses_over_ceiling() {
        assert_eq!(YieldDirective::new(MAX_REDUCTION_QUOTA + 1), Err(QuotaRefusal::OverCeiling));
        assert_eq!(YieldDirective::new(u32::MAX), Err(QuotaRefusal::OverCeiling));
    }

    #[test]
    fn test_status_codes_distinct_and_ok_is_zero() {
        let codes = [
            STATUS_OK,
            STATUS_E_INTERRUPTED,
            STATUS_E_INVALID,
            STATUS_E_NOMEM,
            STATUS_E_TRAP,
            STATUS_E_THERMAL,
            STATUS_E_UNSUPPORTED,
            STATUS_E_UNAUTHORIZED,
            STATUS_E_TIMEDOUT,
            STATUS_E_INTERNAL,
        ];
        assert_eq!(codes[0], 0);
        let mut sorted = codes.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), codes.len(), "status codes must be distinct");
    }

    #[test]
    fn test_next_quota_v1_is_constant() {
        assert_eq!(next_quota(0), DEFAULT_REDUCTION_QUOTA);
        assert_eq!(next_quota(DEFAULT_REDUCTION_QUOTA), DEFAULT_REDUCTION_QUOTA);
        assert_eq!(next_quota(MAX_REDUCTION_QUOTA), DEFAULT_REDUCTION_QUOTA);
    }
}
