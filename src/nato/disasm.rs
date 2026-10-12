#![cfg(feature = "nato")]
//! N06 — branchless disassembly court (NATO lane).
//!
//! Ported from strata `tests/nato/n06_branchless_disasm.rs` +
//! `scripts/n06_disasm_audit.sh` (Apache/MIT per strata LICENSE).
//!
//! Asserts, over real build artifacts, that the PN64 state-execution path
//! carries zero conditional branches. The real collaborator is
//! `scripts/n06_disasm_audit.sh`, which performs the census over the
//! compiler-emitted assembly of the release lib build (the `#[inline(never)]`
//! [`kernel_step`] wrapper — whose body is the fully inlined
//! [`crate::conformance::pn64::step_branchless`] — is the audited symbol)
//! and emits a structured JSON receipt. The court asserts on the audit's
//! real output and accepts ONLY a [`ReceiptStatus::Pass`] or a typed
//! [`ReceiptStatus::Skipped`]; any `Fail` or malformed receipt fails.
//!
//! As a Chicago-style behavioral sanity check, [`kernel_step`] is also
//! executed against the branched reference semantics over a xorshift64*
//! corpus (a mis-audited wrapper — e.g. the census hit a different symbol —
//! would not match the reference).

// Lint-scope note: NATO court-harness and ledger wire-format modules use
// domain literals, metric casts, and court-assertion panics by design; the
// style-class lints below are allowed at module scope with this
// justification rather than 300+ inline permits.
#![allow(
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::use_self,
    clippy::missing_const_for_fn,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_long_first_doc_paragraph,
    clippy::doc_overindented_list_items,
    clippy::doc_lazy_continuation,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::explicit_iter_loop,
    clippy::unusual_byte_groupings,
    clippy::unreadable_literal,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::unused_self,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    clippy::needless_range_loop,
    clippy::needless_continue,
    clippy::unnecessary_wraps,
    clippy::double_must_use,
    clippy::op_ref,
    clippy::match_same_arms,
    clippy::type_complexity,
    clippy::or_fun_call,
    clippy::nonminimal_bool,
    clippy::unnecessary_map_or,
    clippy::manual_is_multiple_of,
    clippy::manual_midpoint,
    clippy::identity_op,
    clippy::format_collect,
    clippy::trivially_copy_pass_by_ref,
    dead_code // hoisted court probes/controls (W-NTW §3): referenced by the
              // #[cfg(test)] wrappers and referenced by the court registry
)]

use serde::{Deserialize, Serialize};

use crate::nato::court::{
    CandidateRef, Court, CourtId, CourtOutcome, ControlProbe, MetricValue, Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;

// --- Typed N06 receipt model ---------------------------------------------

/// Terminal status of an N06 audit receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReceiptStatus {
    /// All executed tiers passed their census bounds.
    Pass,
    /// The audit failed: at least one executed tier's census bound was
    /// exceeded (conditional branches found).
    Fail,
    /// The audit could not run a tier for an environmental reason (missing
    /// toolchain or artifact). Typed, never silent.
    Skipped,
}

/// Typed reason a tier (or the whole audit) was skipped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkipReason {
    /// A required audit tool is not installed on this host.
    ToolchainUnavailable {
        /// Name of the missing tool (e.g. `wasm-objdump`).
        tool: String,
    },
    /// A required artifact does not exist (e.g. no wasm32 target installed).
    ArtifactMissing {
        /// Path or name of the missing artifact.
        path: String,
    },
}

/// Per-tier census result as recorded in the receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierReceipt {
    /// Census count: 0 required for `Pass`.
    pub count: u64,
    /// Required bound (always 0 for N06).
    pub bound: u64,
    /// Typed skip reason when the tier did not execute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<SkipReason>,
}

/// Structured N06 receipt (JSON, emitted by `scripts/n06_disasm_audit.sh`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct N06Receipt {
    /// Lane identifier (always "N06").
    pub lane: String,
    /// Terminal status.
    pub status: ReceiptStatus,
    /// Typed skip reason when `status == Skipped`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<SkipReason>,
    /// Human-readable failure reason when `status == Fail`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Wasm-tier census (present when that tier executed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm: Option<TierReceipt>,
    /// Native-tier census (must be present and zero for `Pass`).
    #[serde(default)]
    pub native: Option<TierReceipt>,
}

impl N06Receipt {
    /// The court's acceptance rule: ONLY `Pass`, or `Skipped` with a typed
    /// reason. `Fail` and malformed receipts are rejected by the caller.
    #[must_use]
    pub fn accepted(&self) -> bool {
        match self.status {
            ReceiptStatus::Pass => self.native.as_ref().is_some_and(|n| n.count <= n.bound),
            ReceiptStatus::Skipped => self.skip_reason.is_some(),
            ReceiptStatus::Fail => false,
        }
    }
}

// --- Audited symbol --------------------------------------------------------

/// The audited wrapper: a separate emitted symbol so the disassembly census
/// has a stable, nameable function body that contains the inlined kernel.
/// The test harness around it may branch freely; only this symbol is
/// audited.
#[inline(never)]
pub fn kernel_step(pn: u64, ocel: u64, invariant_mask: u64) -> u64 {
    crate::conformance::pn64::step_branchless(pn, ocel, invariant_mask)
}

// ---------------------------------------------------------------------------
// Court wiring (W-NTW §3/§4): hoisted probe + positive control + trait impl
// ---------------------------------------------------------------------------

/// Run the real audit (census over the emitted `kernel_step` symbol via
/// `scripts/n06_disasm_audit.sh`) and parse its receipt as typed JSON.
///
/// Hoisted (W-NTW §3) from the former test-local `run_audit` + census body so
/// the court adapter and any emitter share ONE probe: metrics cannot diverge.
fn run_audit() -> N06Receipt {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = root.join("scripts/n06_disasm_audit.sh");
    let out = std::env::temp_dir().join(format!("n06_receipt_{}.json", std::process::id()));
    let status = std::process::Command::new("bash")
        .arg(&script)
        .env("N06_OUT", &out)
        .status()
        .expect("failed to spawn scripts/n06_disasm_audit.sh");
    let text = std::fs::read_to_string(&out)
        .expect("N06 receipt unreadable (audit script did not write it)");
    let _ = std::fs::remove_file(&out);
    assert!(status.success(), "audit script exited nonzero; receipt: {text}");
    serde_json::from_str(&text).expect("N06 receipt is not valid typed JSON")
}

/// The N06 court probe (W-NTW §4): runs the real disassembly census and
/// returns the typed receipt. Verdict mapping lives in
/// [`DisasmCourt::run`]: `accepted()` -> Pass; typed skip -> Fail with an
/// honest ScopingRecord disclosure (never a silent pass).
pub(crate) fn n06_run_court_probe() -> N06Receipt {
    run_audit()
}

/// N06 positive control (hoisted from the mutant legs of
/// `n06_receipt_model_acceptance_rules`): the acceptance rule MUST accept a
/// clean Pass receipt and a typed skip, and MUST reject a reason-less skip
/// and a census `Fail`. A violation on any leg means the court logic is
/// vacuous and the control gate refuses the run.
pub(crate) fn n06_positive_control() -> bool {
    let pass = N06Receipt {
        lane: "N06".into(),
        status: ReceiptStatus::Pass,
        skip_reason: None,
        reason: None,
        wasm: None,
        native: Some(TierReceipt { count: 0, bound: 0, skip_reason: None }),
    };
    let skipped = N06Receipt {
        lane: "N06".into(),
        status: ReceiptStatus::Skipped,
        skip_reason: Some(SkipReason::ToolchainUnavailable { tool: "wasm-objdump".into() }),
        reason: None,
        wasm: Some(TierReceipt {
            count: 0,
            bound: 0,
            skip_reason: Some(SkipReason::ToolchainUnavailable { tool: "wasm-objdump".into() }),
        }),
        native: Some(TierReceipt { count: 0, bound: 0, skip_reason: None }),
    };
    let skip_no_reason = N06Receipt { skip_reason: None, ..skipped.clone() };
    let fail = N06Receipt {
        lane: "N06".into(),
        status: ReceiptStatus::Fail,
        skip_reason: None,
        reason: Some("native census nonzero".into()),
        wasm: None,
        native: Some(TierReceipt { count: 3, bound: 0, skip_reason: None }),
    };
    // Also: the wire format the script emits must round-trip.
    let json = serde_json::to_string(&pass).expect("serialize");
    let back: N06Receipt = serde_json::from_str(&json).expect("deserialize");

    pass.accepted()
        && skipped.accepted()
        && !skip_no_reason.accepted()
        && !fail.accepted()
        && back == pass
        && back.status == ReceiptStatus::Pass
}

/// N06 — branchless disassembly court (W-NTW §2 adapter, mils template).
pub struct DisasmCourt;

/// Court identity constants.
impl DisasmCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N06;
    /// Capability pillar.
    pub const PILLAR: Pillar = Pillar::Timing;
}

impl Court for DisasmCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N06 branchless_disasm"
    }

    fn positive_control(&self) -> ControlProbe {
        n06_positive_control
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        let receipt = n06_run_court_probe();
        // Verdict law: ONLY a real Pass is a Pass. A typed skip is an honest
        // Fail carrying the skip disclosure — never a silent pass.
        let verdict = if receipt.status == ReceiptStatus::Pass && receipt.accepted() {
            Verdict::Pass
        } else {
            Verdict::Fail
        };
        let native_count = receipt.native.as_ref().map_or(0, |n| n.count);
        CourtOutcome {
            verdict,
            metrics: vec![
                ("native_branch_count", MetricValue::U64(native_count)),
                ("accepted", MetricValue::Bool(receipt.accepted())),
            ],
            scoping: ScopingRecord::partial(
                "zero conditional branches in the emitted PN64 step symbol across all tiers",
                "objdump census over the #[inline(never)] kernel_step symbol in the \
                 release wasm + native builds via scripts/n06_disasm_audit.sh, plus a \
                 xorshift64* semantic equivalence corpus against the branched reference",
                "census is build-artifact-scoped (this host/toolchain); a tier that \
                 cannot run is a typed skip surfaced as Fail + this disclosure, never \
                 a silent pass; formal all-inputs branch-freedom proof out of scope",
            ),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)] // JUSTIFICATION: test code only
mod tests {
    use super::*;

    #[test]
    fn n06_branchless_disasm_census() {
        let receipt = n06_run_court_probe();
        assert!(
            receipt.accepted(),
            "N06 receipt not accepted: status = {:?}, skip_reason = {:?}, \
             reason = {:?}, native = {:?}",
            receipt.status,
            receipt.skip_reason,
            receipt.reason,
            receipt.native
        );
        if receipt.status == ReceiptStatus::Pass {
            let n = receipt.native.as_ref().expect("Pass requires a native tier census");
            assert_eq!(n.count, 0, "native tier: nonzero conditional-branch census in kernel_step");
        }
    }

    /// The receipt model round-trips and the acceptance rule rejects Fail
    /// and malformed shapes (court logic tested directly, no subprocess).
    /// Wrapper over the hoisted positive control (W-NTW §3).
    #[test]
    fn n06_receipt_model_acceptance_rules() {
        assert!(
            n06_positive_control(),
            "N06 positive control failed: acceptance rule rejected a clean pass/typed skip, \
             accepted a reason-less skip or census Fail, or the wire format drifted"
        );
    }

    #[test]
    fn kernel_step_matches_reference_semantics() {
        // Chicago: the audited symbol is the real executable collaborator. A
        // xorshift64* corpus must reproduce the branched reference semantics
        // exactly — a branchless wrapper that was mis-audited (census hit a
        // different symbol) would not.
        struct Rng(u64);
        impl Rng {
            fn next(&mut self) -> u64 {
                let mut x = self.0;
                x ^= x >> 12;
                x ^= x << 25;
                x ^= x >> 27;
                self.0 = x;
                x.wrapping_mul(0x2545_F491_4F6C_DD1D)
            }
        }
        let mut r = Rng(0x060C_0C0C_0C0C_0C01);
        for _ in 0..50_000 {
            let pn = r.next();
            let ocel = r.next();
            let inv = r.next();
            // Branched reference, restated inline (semantics per pn64 doc).
            let m = pn & 0xFFFF;
            let w_pre = (pn >> 16) & 0xFFFF;
            let w_post = (pn >> 32) & 0xFFFF;
            let tid = (pn >> 48) & 0xFF;
            let activity = ocel & 0xFF;
            let next_m = ((m ^ w_pre) | w_post) & 0xFFFF;
            let valid =
                (activity == tid) && ((m & w_pre) == w_pre) && (next_m & (inv & 0xFFFF) == 0);
            let want = if valid { (pn & !0xFFFF) | next_m } else { pn };

            assert_eq!(kernel_step(pn, ocel, inv), want, "pn={pn:#x} ocel={ocel:#x} inv={inv:#x}");
        }
    }
}
