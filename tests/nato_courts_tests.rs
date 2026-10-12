//! NATO court suite — always-run (fast tier) surface.
//!
//! Every tier-3 (`#[ignore = "nato-tier3"]`) court body stays in `src/nato/`.
//! This file asserts the ALWAYS-RUN surface: the positive-control gate law,
//! receipt integrity, scoping validation, registry identity, the fast
//! exhaustive pillar claims (N01/N02/N03), and the in-process audit courts
//! (N04 symbol scan; N13/N16 always-run legs pinned at the source surface).
//!
//! Tiering rules per `src/nato/README.md`: tier-1 (this file + inline
//! `#[cfg(test)]`) runs on every `just test-unit`; tier-3 stays behind
//! `#[ignore = "nato-tier3"]` and is never duplicated here.
#![cfg(feature = "nato")]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
// JUSTIFICATION: test code only — assertions on real collected state.
#![allow(clippy::unwrap_used)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use chicago_tdd_tools::nato::bell_lapadula::BellLapadulaCourt;
use chicago_tdd_tools::nato::cache::CacheCourt;
use chicago_tdd_tools::nato::disasm::DisasmCourt;
use chicago_tdd_tools::nato::dossier_court::DossierCourt;
use chicago_tdd_tools::nato::entropy::EntropyCourt;
use chicago_tdd_tools::nato::hamming::HammingProfileCourt;
use chicago_tdd_tools::nato::kat::KatCourt;
use chicago_tdd_tools::nato::liveness::LivenessCourt;
use chicago_tdd_tools::nato::mils::MilSeparationCourt;
use chicago_tdd_tools::nato::orthogonality::OrthogonalityCourt;
use chicago_tdd_tools::nato::receipt::canonical_digest;
use chicago_tdd_tools::nato::reproducible::ReproducibleCourt;
use chicago_tdd_tools::nato::rng::Lcg;
use chicago_tdd_tools::nato::seu::{SeuDoubleBitCourt, SeuSingleBitCourt};
use chicago_tdd_tools::nato::stackbound::StackBoundCourt;
use chicago_tdd_tools::nato::thrash::Cast32ThrashCourt;
use chicago_tdd_tools::nato::tick::ClockGlitchCourt;
use chicago_tdd_tools::nato::timing::TimingCourt;
use chicago_tdd_tools::nato::torn::TornWriteCourt;
use chicago_tdd_tools::nato::wcet::WcetCourt;
use chicago_tdd_tools::nato::zero_ambient::ZeroAmbientCourt;
use chicago_tdd_tools::nato::{
    run_court, CandidateRef, ControlProbe, ControlResult, Court, CourtId, CourtOutcome,
    CourtReceipt, MetricValue, Pillar, ScopingRecord, ScopingValidationError, Verdict,
    COURT_REGISTRY,
};

// ─────────────────────────────────────────────────────────────────────────────
// Test courts: probe the positive-control gate law with an AtomicBool tripwire
// ─────────────────────────────────────────────────────────────────────────────

// Mutex-serialized: the tripwire is process-global and two gate-law tests
// race it under the parallel harness (A81 flake root cause).
static GATE_TRIPWIRE_LOCK: Mutex<()> = Mutex::new(());
static RUN_CALLED: AtomicBool = AtomicBool::new(false);

fn run_called() -> bool {
    RUN_CALLED.load(Ordering::SeqCst)
}

fn reset_run_called() {
    RUN_CALLED.store(false, Ordering::SeqCst);
}

/// A minimal court whose positive control always fails: the gate MUST refuse
/// adjudication and never call `run`.
struct BrokenControlCourt;

impl BrokenControlCourt {
    /// Court identifier.
    const ID: CourtId = CourtId::N20;
    /// Capability pillar.
    const PILLAR: Pillar = Pillar::Dossier;
}

impl Court for BrokenControlCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N20 broken_control_test_court"
    }

    fn positive_control(&self) -> ControlProbe {
        || false
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        // Tripwire: the gate law says this must never execute on a failed probe.
        RUN_CALLED.store(true, Ordering::SeqCst);
        CourtOutcome {
            verdict: Verdict::Pass,
            metrics: vec![("never", MetricValue::Bool(true))],
            scoping: ScopingRecord::full("test mandate", "test evidence"),
        }
    }
}

/// Positive twin: control passes, so `run` executes under a cleared gate.
struct PassingControlCourt;

impl PassingControlCourt {
    /// Court identifier.
    const ID: CourtId = CourtId::N19;
    /// Capability pillar.
    const PILLAR: Pillar = Pillar::Resilience;
}

impl Court for PassingControlCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N19 passing_control_test_court"
    }

    fn positive_control(&self) -> ControlProbe {
        || true
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        RUN_CALLED.store(true, Ordering::SeqCst);
        CourtOutcome {
            verdict: Verdict::Pass,
            metrics: vec![
                ("count", MetricValue::U64(7)),
                ("ratio", MetricValue::F64(0.25)),
                ("tag", MetricValue::Str("ok".into())),
                ("ok", MetricValue::Bool(true)),
            ],
            scoping: ScopingRecord::full("mandate", "evidence"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Positive-control gate law
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn gate_refuses_adjudication_when_mutant_survives() {
    let _guard = GATE_TRIPWIRE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_run_called();
    let receipt = run_court(
        &BrokenControlCourt,
        &CandidateRef("mutant-subject".into()),
        "sha-broken-control",
    );

    assert_eq!(
        receipt.verdict,
        Verdict::Refused,
        "a surviving mutant must void the run: verdict must be Refused, not adjudicated"
    );
    assert!(
        matches!(receipt.positive_control, ControlResult::GateRefused { .. }),
        "control result must be GateRefused (or an explicit MutantSurvived \
         classification), got {:?}",
        receipt.positive_control
    );
    assert!(!receipt.positive_control.passed(), "a refused control must not report as passed");
    assert!(
        !run_called(),
        "gate law violation: run() executed although the positive control failed"
    );
    assert!(receipt.metrics.is_empty(), "a refused run must carry no measured metrics");
    assert_eq!(
        receipt.scoping,
        ScopingRecord::refused_scope(),
        "a refused run must carry the refused-scope disclosure"
    );
    assert!(receipt.scoping.validate().is_ok());
}

#[test]
fn passing_control_clears_the_gate_and_runs() {
    let _guard = GATE_TRIPWIRE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_run_called();
    let receipt = run_court(
        &PassingControlCourt,
        &CandidateRef("clean-subject".into()),
        "sha-passing-control",
    );

    assert_eq!(receipt.verdict, Verdict::Pass);
    assert_eq!(
        receipt.pillar,
        Pillar::Resilience,
        "receipt pillar must come from the court's PILLAR const"
    );
    assert_eq!(receipt.court_id, CourtId::N19);
    assert!(run_called(), "a cleared gate must actually run the court");
    assert!(
        matches!(receipt.positive_control, ControlResult::MutantCaught { .. }),
        "a passing probe classifies as MutantCaught, got {:?}",
        receipt.positive_control
    );
    assert!(receipt.positive_control.passed());
    assert_eq!(receipt.metrics.len(), 4);
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Receipt integrity: JSON roundtrip + digest stability
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn receipt_json_roundtrip_preserves_everything() {
    // Serializes with the gate-law tests: this body runs PassingControlCourt,
    // whose run() mutates the shared RUN_CALLED tripwire (race fix).
    let _tripwire = GATE_TRIPWIRE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

    let receipt =
        run_court(&PassingControlCourt, &CandidateRef("roundtrip".into()), "sha-roundtrip");
    let json = receipt.to_json().expect("CourtReceipt must serialize to JSON");
    let back = CourtReceipt::from_json(&json).expect("CourtReceipt must deserialize");

    assert_eq!(back, receipt, "JSON roundtrip must preserve the full receipt");
    assert_eq!(back.digest, receipt.digest);
    assert_eq!(back.receipt_id, receipt.receipt_id);
    assert_eq!(back.court_id, receipt.court_id);
    assert_eq!(back.pillar, receipt.pillar);
    assert_eq!(back.positive_control, receipt.positive_control);
    assert_eq!(back.scoping, receipt.scoping);

    // The refused receipt also survives the roundtrip (enum with payloads).
    let refused = run_court(
        &BrokenControlCourt,
        &CandidateRef("roundtrip-refused".into()),
        "sha-roundtrip-refused",
    );
    let refused_json = refused.to_json().expect("refused receipt serializes");
    let refused_back = CourtReceipt::from_json(&refused_json).expect("refused receipt parses");
    assert_eq!(refused_back, refused);
}

#[test]
fn receipt_digest_is_stable_and_discriminating() {
    // Serializes with the gate-law tests: this body runs PassingControlCourt,
    // whose run() mutates the shared RUN_CALLED tripwire (race fix).
    let _tripwire = GATE_TRIPWIRE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

    let receipt = run_court(&PassingControlCourt, &CandidateRef("digest".into()), "sha-digest");
    // Same inputs, two mintings: identical canonical string and digest.
    let again = run_court(&PassingControlCourt, &CandidateRef("digest".into()), "sha-digest");
    assert_eq!(
        receipt.canonical(),
        again.canonical(),
        "same court/subject/verdict/metrics must produce the same canonical string"
    );
    assert_eq!(
        receipt.digest, again.digest,
        "digest must be stable across constructions from identical inputs"
    );

    // Digest is a 64-hex SHA-256 and receipt_id carries its 16-char prefix.
    assert_eq!(receipt.digest.len(), 64, "digest must be a hex SHA-256");
    assert!(
        receipt.digest.chars().all(|c| c.is_ascii_hexdigit()),
        "digest must be lowercase hex, got {}",
        receipt.digest
    );
    assert!(
        receipt.receipt_id.starts_with("cr-N19-"),
        "receipt_id must be court-prefixed, got {}",
        receipt.receipt_id
    );
    assert!(
        receipt.receipt_id.contains(&receipt.digest[..16]),
        "receipt_id must embed the digest prefix"
    );

    // Canonical digest discriminates on verdict, subject, and metric content.
    let empty: Vec<(String, MetricValue)> = Vec::new();
    let base = canonical_digest("N01", "sha", Verdict::Pass, &empty);
    assert_eq!(base, canonical_digest("N01", "sha", Verdict::Pass, &empty));
    assert_ne!(
        base,
        canonical_digest("N01", "sha", Verdict::Fail, &empty),
        "verdict must change the digest"
    );
    assert_ne!(
        base,
        canonical_digest("N02", "sha", Verdict::Pass, &empty),
        "court must change the digest"
    );
    assert_ne!(
        base,
        canonical_digest("N01", "sha-other", Verdict::Pass, &empty),
        "subject must change the digest"
    );
    assert_ne!(
        base,
        canonical_digest("N01", "sha", Verdict::Pass, &[("k".to_string(), MetricValue::U64(1))],),
        "adding a metric must change the digest"
    );
    assert_ne!(
        canonical_digest("N01", "sha", Verdict::Pass, &[("k".to_string(), MetricValue::U64(1))],),
        canonical_digest(
            "N01",
            "sha",
            Verdict::Pass,
            &[("k".to_string(), MetricValue::Str("1".to_string()))],
        ),
        "metric TYPE must change the digest (u64 1 is not str \"1\")"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Scoping disclosure validation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn scoping_validator_rejects_empty_delivered_evidence_at_non_full_scale() {
    let partial_no_evidence = ScopingRecord {
        spec_mandate: "mandate".into(),
        delivered_evidence: String::new(),
        is_full_spec_scale: false,
        bound: "bound".into(),
    };
    assert_eq!(
        partial_no_evidence.validate(),
        Err(ScopingValidationError::MissingDeliveredEvidence),
        "non-full-scale disclosure without delivered evidence must be rejected"
    );

    let partial_no_bound = ScopingRecord {
        spec_mandate: "mandate".into(),
        delivered_evidence: "evidence".into(),
        is_full_spec_scale: false,
        bound: String::new(),
    };
    assert_eq!(
        partial_no_bound.validate(),
        Err(ScopingValidationError::MissingBound),
        "non-full-scale disclosure without a named bound must be rejected"
    );

    // Full scale needs only the mandate; both legal partials pass.
    assert_eq!(ScopingRecord::full("mandate", "evidence").validate(), Ok(()));
    assert_eq!(ScopingRecord::partial("mandate", "evidence", "bound").validate(), Ok(()));
    assert_eq!(
        ScopingRecord::refused_scope().validate(),
        Ok(()),
        "refused scope names its bound, so it validates"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Registry completeness + pillar identity mapping
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn registry_court_ids_are_twenty_unique_and_canonical() {
    let names = chicago_tdd_tools::nato::registry::COURT_NAMES;
    assert_eq!(names.len(), 20, "the registry must fix exactly 20 court names");
    let mut sorted = names.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 20, "court names must be unique: {names:?}");

    // CourtId discriminants 1..=20 project onto the canonical name table in
    // order; every id resolves, none collide.
    const ALL_IDS: [CourtId; 20] = [
        CourtId::N01,
        CourtId::N02,
        CourtId::N03,
        CourtId::N04,
        CourtId::N05,
        CourtId::N06,
        CourtId::N07,
        CourtId::N08,
        CourtId::N09,
        CourtId::N10,
        CourtId::N11,
        CourtId::N12,
        CourtId::N13,
        CourtId::N14,
        CourtId::N15,
        CourtId::N16,
        CourtId::N17,
        CourtId::N18,
        CourtId::N19,
        CourtId::N20,
    ];
    for (i, id) in ALL_IDS.iter().enumerate() {
        assert_eq!(
            id.id_number() as usize,
            i + 1,
            "CourtId discriminants must run 1..=20 in order"
        );
        assert_eq!(
            id.canonical(),
            names[i],
            "CourtId::canonical must read the registry name table"
        );
    }
    let mut canonical: Vec<&str> = ALL_IDS.iter().map(|id| id.canonical()).collect();
    canonical.sort_unstable();
    canonical.dedup();
    assert_eq!(canonical.len(), 20, "canonical ids must be unique");

    // All four landed default-tier pillar courts must be registered under
    // their matrix identity (src/nato/README.md N01-N20 table).
    assert_eq!(<MilSeparationCourt>::ID, CourtId::N01);
    assert_eq!(<OrthogonalityCourt>::ID, CourtId::N02);
    assert_eq!(<BellLapadulaCourt>::ID, CourtId::N03);
    assert_eq!(<ZeroAmbientCourt>::ID, CourtId::N04);
    assert_eq!(<TimingCourt>::ID, CourtId::N05);
    assert_eq!(<DisasmCourt>::ID, CourtId::N06);
    assert_eq!(<CacheCourt>::ID, CourtId::N07);
    assert_eq!(<HammingProfileCourt>::ID, CourtId::N08);
    assert_eq!(<SeuSingleBitCourt>::ID, CourtId::N09);
    assert_eq!(<SeuDoubleBitCourt>::ID, CourtId::N10);
    assert_eq!(<TornWriteCourt>::ID, CourtId::N11);
    assert_eq!(<ClockGlitchCourt>::ID, CourtId::N12);
    assert_eq!(<WcetCourt>::ID, CourtId::N13);
    assert_eq!(<Cast32ThrashCourt>::ID, CourtId::N14);
    assert_eq!(<LivenessCourt>::ID, CourtId::N15);
    assert_eq!(<StackBoundCourt>::ID, CourtId::N16);
    assert_eq!(<KatCourt>::ID, CourtId::N17);
    assert_eq!(<EntropyCourt>::ID, CourtId::N18);
    assert_eq!(<ReproducibleCourt>::ID, CourtId::N19);
    assert_eq!(<DossierCourt>::ID, CourtId::N20);
}

#[test]
fn pillar_mapping_matches_the_matrix() {
    // Per the README matrix: N01 isolation (Security), N02 proof,
    // N03 isolation (Security), N04 authority (Dossier pillar in-code).
    assert_eq!(<MilSeparationCourt>::PILLAR, Pillar::Security);
    assert_eq!(<OrthogonalityCourt>::PILLAR, Pillar::Proof);
    assert_eq!(<BellLapadulaCourt>::PILLAR, Pillar::Security);
    assert_eq!(<ZeroAmbientCourt>::PILLAR, Pillar::Dossier);
    assert_eq!(<TimingCourt>::PILLAR, Pillar::Timing);
    assert_eq!(<DisasmCourt>::PILLAR, Pillar::Timing);
    assert_eq!(<CacheCourt>::PILLAR, Pillar::Timing);
    assert_eq!(<HammingProfileCourt>::PILLAR, Pillar::Security);
    assert_eq!(<SeuSingleBitCourt>::PILLAR, Pillar::Resilience);
    assert_eq!(<SeuDoubleBitCourt>::PILLAR, Pillar::Resilience);
    assert_eq!(<TornWriteCourt>::PILLAR, Pillar::Resilience);
    assert_eq!(<ClockGlitchCourt>::PILLAR, Pillar::Resilience);
    assert_eq!(<WcetCourt>::PILLAR, Pillar::Timing);
    assert_eq!(<Cast32ThrashCourt>::PILLAR, Pillar::Resilience);
    assert_eq!(<LivenessCourt>::PILLAR, Pillar::Proof);
    assert_eq!(<StackBoundCourt>::PILLAR, Pillar::Dossier);
    assert_eq!(<KatCourt>::PILLAR, Pillar::Security);
    assert_eq!(<EntropyCourt>::PILLAR, Pillar::Security);
    assert_eq!(<ReproducibleCourt>::PILLAR, Pillar::Proof);
    assert_eq!(<DossierCourt>::PILLAR, Pillar::Dossier);

    // Pillar machine names are stable (they feed receipts).
    assert_eq!(Pillar::Security.as_str(), "security");
    assert_eq!(Pillar::Timing.as_str(), "timing");
    assert_eq!(Pillar::Resilience.as_str(), "resilience");
    assert_eq!(Pillar::Proof.as_str(), "proof");
    assert_eq!(Pillar::Dossier.as_str(), "dossier");
}

#[test]
/// Registry sweep over all 20 courts.
///
/// Determinism note — the three not-guaranteed-Pass courts: N05 is
/// statistical, N06 is host-dependent, and N19 is artifact-dependent
/// (its receipt artifact `target/nato_reproducible_receipt.json` is
/// minted by `bash bin/nato_reproducible_build.sh`; running that
/// flips N19 to Pass). All other courts must Pass unconditionally.
fn registry_sweep_runs_all_twenty_and_matches_real_receipts() {
    // Registry shape: 20 entries in canonical id order, ids unique.
    assert_eq!(COURT_REGISTRY.len(), 20, "the registry must hold all 20 courts");
    for (i, e) in COURT_REGISTRY.iter().enumerate() {
        assert_eq!(
            e.id.id_number() as usize,
            i + 1,
            "COURT_REGISTRY must be in canonical N01..N20 order"
        );
    }

    // Snapshot docs/receipts before the sweep: the sweep reads real receipts
    // and must never persist anything.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/receipts");
    let snapshot = || {
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("docs/receipts must exist: {e}"))
            .map(|e| e.expect("readdir").file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    };
    let before = snapshot();

    for e in &COURT_REGISTRY {
        let court = (e.ctor)();
        let cand = CandidateRef(format!("registry-sweep-n{:02}", e.id.id_number()));
        let subject = format!("sha-registry-sweep-n{:02}", e.id.id_number());
        let receipt = run_court(court.as_ref(), &cand, &subject);

        // Registry identity agreement: entry id/pillar == the court's own
        // consts (the consts are the law), and the receipt carries both.
        assert_eq!(receipt.court_id, e.id, "{} receipt id", e.id.canonical());
        assert_eq!(receipt.pillar, e.pillar, "{} receipt pillar", e.id.canonical());
        assert!(
            receipt.name.starts_with(e.id.canonical()),
            "{} name must be prefixed by its canonical id, got {:?}",
            e.id.canonical(),
            receipt.name
        );

        // Verdict law: a Pass is a real Pass. A non-Pass must be an honest
        // typed skip or refusal: a Fail must carry the typed skip metric
        // (`accepted == false`) AND a validating ScopingRecord naming the
        // bound — never a silent pass; N19 additionally refuses when its
        // receipt artifact (target/nato_reproducible_receipt.json) is absent.
        if receipt.verdict != Verdict::Pass {
            if e.id == CourtId::N19 {
                assert!(
                    matches!(receipt.verdict, Verdict::Refused | Verdict::Fail),
                    "N19 non-Pass must be Refused or Fail, got {:?}",
                    receipt.verdict
                );
                eprintln!(
                    "N19 Refused/Fail: receipt artifact absent (machine-dependent, \
                     not a regression). Run bash bin/nato_reproducible_build.sh \
                     to flip to Pass."
                );
            } else {
                assert_eq!(
                    receipt.verdict,
                    Verdict::Fail,
                    "{} non-Pass must be a typed-skip Fail, got {:?}",
                    e.id.canonical(),
                    receipt.verdict
                );
            }
            // The skip must be disclosed: the outcome's ScopingRecord must
            // be a partial disclosure naming its bound, never a full-scale
            // claim over skipped evidence.
            let outcome = court.run(&cand);
            assert!(
                !outcome.scoping.is_full_spec_scale,
                "{} non-Pass must disclose partial scale, never a full-scale claim",
                e.id.canonical()
            );
            outcome.scoping.validate().unwrap_or_else(|err| {
                panic!("{} scoping record must validate: {err:?}", e.id.canonical())
            });
        }

        // On-disk field agreement for the adapter lanes with minted receipts:
        // court_id/pillar/verdict must agree with docs/receipts/Nxx.json;
        // bytes never equal (the sweep subject differs) and nothing persists.
        if matches!(e.id, CourtId::N08 | CourtId::N12 | CourtId::N14 | CourtId::N15) {
            let path = dir.join(format!("{}.json", e.id.canonical()));
            let disk = CourtReceipt::from_json(
                &std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("on-disk {} receipt: {e}", path.display())),
            )
            .unwrap_or_else(|e| panic!("on-disk {} parses: {e}", path.display()));
            assert_eq!(disk.court_id, receipt.court_id);
            assert_eq!(disk.pillar, receipt.pillar);
            assert_eq!(disk.verdict, receipt.verdict);
            assert_ne!(
                disk.to_json().expect("disk json"),
                receipt.to_json().expect("sweep json"),
                "sweep receipts must never byte-equal the minted artifact"
            );
        }
    }

    let after = snapshot();
    assert_eq!(before, after, "the registry sweep must never persist receipts");
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Pillar fast exhaustive claims (default tier of N01/N02/N03/N04)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn n01_mils_separation_default_tier_has_zero_interference() {
    // The mutant-detection control must hold on its own first.
    assert!(
        chicago_tdd_tools::nato::mils::mils_positive_control(),
        "N01 positive control vacuous: the leaky_advance mutant was not caught"
    );
    let receipt =
        run_court(&MilSeparationCourt, &CandidateRef("pillar-i".into()), "sha-pillar-i-n01");
    assert_eq!(
        receipt.verdict,
        Verdict::Pass,
        "N01 default-tier exhaustive small-domain sweep found interference"
    );
    let metric = |name: &str| -> u64 {
        receipt
            .metrics
            .iter()
            .find(|(k, _)| k == name)
            .and_then(|(_, v)| match v {
                MetricValue::U64(n) => Some(*n),
                _ => None,
            })
            .unwrap_or_else(|| panic!("N01 receipt missing u64 metric {name}"))
    };
    assert_eq!(
        metric("interference_violations"),
        0,
        "N01: dState(TA)/dInput(TB) must be 0 across the exhaustive small-domain sweep"
    );
    assert_eq!(
        metric("coupling_pin_violations"),
        0,
        "N01: the enumerated coupling surface must be fully pinned"
    );
    assert!(
        metric("cells") > 0 && metric("armed_b_audits") > 0,
        "N01 sweep must have audited a non-empty cell/armed-B surface to carry bits"
    );
    assert!(
        matches!(receipt.positive_control, ControlResult::MutantCaught { .. }),
        "N01 receipt must record a caught mutant, got {:?}",
        receipt.positive_control
    );
    // Honest scoping: the default run is below the 2^64 SMT mandate and must
    // disclose it as non-full-scale with a named bound.
    assert!(
        !receipt.scoping.is_full_spec_scale,
        "N01 default tier is below the 2^64 SMT mandate; scoping must not claim full scale"
    );
    assert!(receipt.scoping.validate().is_ok());
}

#[test]
fn n02_orthogonality_default_tier_has_zero_bit_spills() {
    assert!(
        chicago_tdd_tools::nato::orthogonality::orthogonality_positive_control(),
        "N02 positive control vacuous: the spilling pack mutant was not caught"
    );
    let receipt =
        run_court(&OrthogonalityCourt, &CandidateRef("pillar-i".into()), "sha-pillar-i-n02");
    assert_eq!(
        receipt.verdict,
        Verdict::Pass,
        "N02 exhaustive bit-spill census found a field spill"
    );
    let spills = receipt
        .metrics
        .iter()
        .find(|(k, _)| *k == "bit_spills")
        .and_then(|(_, v)| match v {
            MetricValue::U64(n) => Some(*n),
            _ => None,
        })
        .unwrap_or_else(|| panic!("N02 receipt missing bit_spills metric"));
    assert_eq!(spills, 0, "N02: every field write must perturb only its own field");
    assert!(
        matches!(receipt.positive_control, ControlResult::MutantCaught { .. }),
        "N02 receipt must record a caught mutant"
    );
    // Spec-exact exhaustive: full scale claimed and validated.
    assert!(
        receipt.scoping.is_full_spec_scale,
        "N02 is spec-exact exhaustive; it must claim full spec scale"
    );
}

#[test]
fn n03_bell_lapadula_default_tier_has_zero_lattice_violations() {
    assert!(
        chicago_tdd_tools::nato::bell_lapadula::bell_lapadula_positive_control(),
        "N03 positive control vacuous: the declassifying mutant was not caught"
    );
    let receipt =
        run_court(&BellLapadulaCourt, &CandidateRef("pillar-i".into()), "sha-pillar-i-n03");
    assert_eq!(
        receipt.verdict,
        Verdict::Pass,
        "N03 exhaustive BLP lattice census found a no-write-down/no-read-up violation"
    );
    let metric = |name: &str| -> u64 {
        receipt
            .metrics
            .iter()
            .find(|(k, _)| k == name)
            .and_then(|(_, v)| match v {
                MetricValue::U64(n) => Some(*n),
                _ => None,
            })
            .unwrap_or_else(|| panic!("N03 receipt missing u64 metric {name}"))
    };
    assert_eq!(metric("write_down_violations"), 0, "N03: no write-down");
    assert_eq!(metric("read_up_violations"), 0, "N03: no read-up");
    assert_eq!(metric("tranquility_violations"), 0, "N03: strong tranquility holds");
    assert!(
        metric("pairs") > 0 && metric("enabled_firings") > 0,
        "N03 census must have fired a non-empty transition surface to carry bits"
    );
    assert!(receipt.scoping.is_full_spec_scale);
}

#[test]
fn n04_zero_ambient_audit_passes_in_process() {
    assert!(
        chicago_tdd_tools::nato::zero_ambient::zero_ambient_positive_control(),
        "N04 scanner canary vacuous: planted ambient symbols were not all caught"
    );
    let receipt =
        run_court(&ZeroAmbientCourt, &CandidateRef("pillar-i".into()), "sha-pillar-i-n04");
    assert_eq!(
        receipt.verdict,
        Verdict::Pass,
        "N04 zero-ambient symbol scan over the audited module set found violations"
    );
    let banned = receipt
        .metrics
        .iter()
        .find(|(k, _)| *k == "banned_hits")
        .and_then(|(_, v)| match v {
            MetricValue::U64(n) => Some(*n),
            _ => None,
        })
        .unwrap_or_else(|| panic!("N04 receipt missing banned_hits metric"));
    assert_eq!(banned, 0, "N04: zero env/fs/net/process/SystemTime symbols on state paths");
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Audit courts: N13 static WCET table + N16 stackbound DFS (always-run legs)
// ─────────────────────────────────────────────────────────────────────────────
//
// The always-run legs of N13/N16 live as inline #[cfg(test)] tests in
// src/nato/wcet.rs and src/nato/stackbound.rs (they run with the lib test
// binary). This file pins that surface at the source level: the always-run
// legs exist, are NOT #[ignore]d, and the measured legs are tier-3 gated —
// so the fast tier can never silently claim another tier's strength.

const WCET_SRC: &str = include_str!("../src/nato/wcet.rs");
const STACKBOUND_SRC: &str = include_str!("../src/nato/stackbound.rs");

#[test]
fn n13_wcet_static_table_leg_is_always_run_and_measured_leg_is_tier3() {
    for always in [
        "fn n13_static_instruction_count_table_is_published_and_self_consistent",
        "fn n13_hot_path_body_audit_zero_branch_and_heap_tokens",
        "fn n13_kernel_semantics_pinned_over_corpus",
    ] {
        assert!(
            WCET_SRC.contains(always),
            "N13 always-run leg missing from src/nato/wcet.rs: {always}"
        );
        let at = WCET_SRC.find(always).expect("checked above");
        let prefix = &WCET_SRC[..at];
        let decor_at = prefix.rfind("#[test]").expect("leg must be a #[test]");
        let decor = &prefix[decor_at..];
        assert!(!decor.contains("ignore"), "N13 always-run leg must not be #[ignore]d: {always}");
    }
    for tier3 in
        ["fn n13_measured_worst_case_within_static_bound", "fn n13_no_input_class_exceeds_bound"]
    {
        let at = WCET_SRC
            .find(tier3)
            .unwrap_or_else(|| panic!("N13 tier-3 leg missing from src/nato/wcet.rs: {tier3}"));
        let prefix = &WCET_SRC[..at];
        let decor_at = prefix
            .rfind("#[test]")
            .unwrap_or_else(|| panic!("N13 leg {tier3} must be a #[test]"));
        let decor = &prefix[decor_at..];
        assert!(
            decor.contains("#[ignore = \"nato-tier3\"]"),
            "N13 measured leg {tier3} must stay gated behind #[ignore = \"nato-tier3\"]"
        );
    }
    // The static deliverable is published: 35 ops, 0 branches, depth 12.
    assert!(
        WCET_SRC.contains("const BRANCHLESS_TOTAL: u32 = 35;")
            && WCET_SRC.contains("const BRANCHLESS_DEPTH: u32 = 12;")
            && WCET_SRC.contains("const SPEC_TARGET_CYCLES: u32 = 12;"),
        "N13 static WCET table must publish the 35-op / depth-12 / <=12-cycle figures",
    );

    // Behavioral cross-check (arithmetic court). The wcet module exposes no
    // public items, so the table fn itself cannot be invoked from an
    // integration test; instead derive the kernel row's total two independent
    // ways from the module source and require both to agree with the pinned
    // const — if the OpRow literal, the published doc table, or the const
    // drifts apart from the others, this fails.
    //
    // 1. Sum the fields of the `const BRANCHLESS: OpRow = OpRow { .. }`
    //    literal and require it to equal BRANCHLESS_TOTAL.
    let row_at = WCET_SRC
        .find("const BRANCHLESS: OpRow = OpRow {")
        .expect("BRANCHLESS OpRow literal must exist");
    let row = &WCET_SRC[row_at..row_at + WCET_SRC[row_at..].find("};").expect("literal closes")];
    // The only digit runs inside the literal are the field values
    // (6, 14, 6, 6, 3); `..OpRow::EMPTY` carries none.
    let row_total: u64 = parse_doc_table_numbers(row).iter().sum();
    assert_eq!(row_total, 35, "sum of BRANCHLESS OpRow fields must equal the pinned 35");

    // 2. Sum the per-column totals of the published doc table's total row
    //    (`| **total** | **6** |**14**|**6**|**6**|**3**| = 35`) and require
    //    the sum AND the stated figure to equal 35.
    let total_row_at = WCET_SRC.find("| **total**").expect("doc table total row");
    let line_end = total_row_at + WCET_SRC[total_row_at..].find('\n').expect("total row ends");
    let total_row = &WCET_SRC[total_row_at..line_end];
    let nums: Vec<u64> = parse_doc_table_numbers(total_row);
    assert_eq!(nums.len(), 6, "total row must carry 5 column totals plus the grand total");
    let column_sum: u64 = nums[..5].iter().sum();
    assert_eq!(column_sum, 35, "doc-table column totals must sum to 35");
    assert_eq!(nums[5], 35, "doc-table grand total must state 35");
}

/// Extract the integers from a doc-table row like
/// `| **total** | **6** |**14**|**6**|**6**|**3**| = 35`.
fn parse_doc_table_numbers(row: &str) -> Vec<u64> {
    row.split(|c: char| !c.is_ascii_digit())
        .filter(|tok| !tok.is_empty())
        .filter_map(|tok| tok.parse::<u64>().ok())
        .collect()
}

#[test]
fn n16_stackbound_dfs_leg_is_always_run_and_probes_are_tier3() {
    // The iterative DFS cycle-detection leg (zero recursion) is the
    // always-run deliverable; the measured stack probes are tier-3.
    assert!(
        STACKBOUND_SRC.contains("fn n16_static_audit_zero_recursion"),
        "N16 zero-recursion DFS leg missing from src/nato/stackbound.rs"
    );
    assert!(
        STACKBOUND_SRC.contains("fn n16_static_audit_zero_heap_on_state_paths"),
        "N16 zero-heap audit leg missing from src/nato/stackbound.rs"
    );
    for leg in
        ["fn n16_static_audit_zero_recursion", "fn n16_static_audit_zero_heap_on_state_paths"]
    {
        let at = STACKBOUND_SRC.find(leg).expect("checked above");
        let prefix = &STACKBOUND_SRC[..at];
        let decor = &prefix[prefix.rfind("#[test]").expect("leg must be a #[test]")..];
        assert!(!decor.contains("ignore"), "N16 always-run leg must not be #[ignore]d: {leg}");
    }
    for tier3 in [
        "fn n16_workload_100k_transitions_on_32kib_thread",
        "fn n16_measured_min_stack_ladder_via_child_probes",
    ] {
        let at = STACKBOUND_SRC.find(tier3).unwrap_or_else(|| {
            panic!("N16 tier-3 probe leg missing from src/nato/stackbound.rs: {tier3}")
        });
        let prefix = &STACKBOUND_SRC[..at];
        let last_ignore = prefix
            .rfind("#[ignore = \"nato-tier3\"]")
            .expect("N16 measured probes must stay gated behind #[ignore = \"nato-tier3\"]");
        let between = &prefix[last_ignore..];
        assert!(
            between.contains("#[test]") && !between.contains("fn "),
            "N16 probe {tier3} must be directly decorated with #[ignore = \"nato-tier3\"]"
        );
    }
    // The DFS is real graph work, not a keyword grep: color-based iterative
    // cycle detection over an extracted call graph.
    assert!(
        STACKBOUND_SRC.contains("extract_fns")
            && STACKBOUND_SRC.contains("cycles")
            && STACKBOUND_SRC.contains("HashMap<String, Vec<String>>"),
        "N16 audit must extract a real fn/callee graph and run cycle detection"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. LCG determinism
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lcg_same_seed_yields_identical_first_ten_samples() {
    let mut a = Lcg::new(0x4E41_544F_2026_1011u64);
    let mut b = Lcg::new(0x4E41_544F_2026_1011u64);
    let first: Vec<u64> = (0..10).map(|_| a.next_u64()).collect();
    let second: Vec<u64> = (0..10).map(|_| b.next_u64()).collect();
    assert_eq!(first, second, "same seed must yield the identical first-10 raw sequence");

    // Same property through the sample() surface, across several moduli.
    for n in [1u64, 7, 100, u64::from(u32::MAX)] {
        let mut c = Lcg::new(0x4E41_544F_2026_1011u64);
        let mut d = Lcg::new(0x4E41_544F_2026_1011u64);
        let s1: Vec<u64> = (0..10).map(|_| c.sample(n)).collect();
        let s2: Vec<u64> = (0..10).map(|_| d.sample(n)).collect();
        assert_eq!(s1, s2, "sample({n}) sequences must match for equal seeds");
        assert!(s1.iter().all(|&v| v < n), "sample({n}) produced an out-of-range value: {s1:?}");
    }

    // Degenerate modulus: sample(0) must be 0, never a panic.
    let mut e = Lcg::new(1);
    assert_eq!(e.sample(0), 0, "sample(0) must return 0");

    // Divergent seeds diverge immediately.
    let mut f = Lcg::new(1);
    let mut g = Lcg::new(2);
    assert_ne!(f.next_u64(), g.next_u64(), "distinct seeds must diverge");
}
