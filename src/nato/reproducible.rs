#![cfg(feature = "nato")]
// JUSTIFICATION: test-fixture JSON navigation (cfg(test)-consumed helpers)
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
// Test helpers below use serde_json Value navigation with in-fixture asserts;
// the unwrap/expect sites are unreachable by construction on the emitted shape.
// unwrap_used is allowed in the module allow-block below: schema-court tests
// + test-mutant helpers are test-only.
// Court harness library: probes and full-tier entry points run under
// `#[ignore]`d tests, so non-test builds legitimately see unused items —
// and referenced by the court registry.
#![allow(dead_code)]
//! Reproducible-build contract court (lane N6, plan P11).
//!
//! Contract under audit: `bin/nato_reproducible_build.sh` emits a JSON
//! receipt recording a double clean-target build compared byte-wise
//! (cmp + sha256), with toolchain identity, honest scope, and a MANDATORY
//! `not_run` disclosure naming the 3-host cross-host reproduction commands
//! that this macOS same-host lane does not perform.
//!
//! Delivered scope (honest): the Rust court validates the receipt SCHEMA
//! when the receipt file exists. It does not run the two-build pipeline
//! itself (a 2x full build inside `cargo test` would wreck iteration time
//! and re-enter cargo); the script is the actor, this court is its
//! admission gate. When the receipt is absent the court returns typed SKIP
//! (a passing, explicitly-labeled no-op), never a silent pass.

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
    clippy::trivially_copy_pass_by_ref
)]

use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::nato::court::{
    CandidateRef, ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;

/// Where the script writes its receipt by default (crate-root relative).
const RECEIPT_PATH: &str = "target/nato_reproducible_receipt.json";

/// Typed outcome of the court: Validated with findings, or Skipped.
enum ReproVerdict {
    Validated { byte_identical: bool },
    Skipped { reason: &'static str },
}

fn locate_receipt() -> Option<std::path::PathBuf> {
    // CWD is the crate root when run via cargo test; also tolerate the
    // workspace-root case by probing both spellings.
    for base in [".", ".."] {
        let p = Path::new(base).join(RECEIPT_PATH);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Schema check: required fields present, types correct, disclosure
/// mandatory. Failure is signalled by panic (court assertion); callers
/// wanting a `Result` wrap with [`schema_result`].
fn validate_schema(receipt: &Value) {
    let obj = receipt.as_object().unwrap_or_else(|| panic!("receipt must be a JSON object"));

    for field in [
        "receipt_kind",
        "artifact_a",
        "artifact_b",
        "sha256_a",
        "sha256_b",
        "byte_identical",
        "comparison_method",
        "toolchain",
        "scope",
        "not_run",
    ] {
        assert!(obj.contains_key(field), "receipt missing required field '{field}'");
    }

    let kind = obj["receipt_kind"].as_str().unwrap_or_default();
    assert_eq!(kind, "nato_reproducible_build", "receipt_kind must be nato_reproducible_build");

    for f in ["sha256_a", "sha256_b"] {
        let sha = obj[f].as_str().unwrap_or_default();
        assert_eq!(sha.len(), 64, "{f} must be a 64-hex-char SHA-256 digest");
        assert!(sha.bytes().all(|b| b.is_ascii_hexdigit()), "{f} must be hex");
    }

    assert!(obj["byte_identical"].is_boolean(), "byte_identical must be a JSON boolean");

    let toolchain = obj["toolchain"]
        .as_object()
        .unwrap_or_else(|| panic!("toolchain must be an object"));
    for f in ["rustc", "cargo", "host_os", "host_arch"] {
        assert!(toolchain.contains_key(f), "toolchain missing required field '{f}'");
    }

    // Scope honesty: the delivered scope must be stated and must not claim
    // full spec scale while the 3-host disclosure is not_run.
    let scope = obj["scope"].as_object().unwrap_or_else(|| panic!("scope must be an object"));
    assert!(scope.contains_key("delivered"), "scope must state what was delivered");
    assert!(
        scope.contains_key("is_full_spec_scale"),
        "scope must state whether it is full spec scale"
    );

    // MANDATORY 3-host not_run disclosure: three platforms, each with a
    // non-empty exact-command string.
    let not_run = obj["not_run"]
        .as_object()
        .unwrap_or_else(|| panic!("not_run must be an object"));
    assert!(
        not_run.contains_key("reason")
            && not_run["reason"].as_str().map_or(false, |s| !s.is_empty()),
        "not_run must carry a non-empty reason"
    );
    let hosts = not_run["hosts"]
        .as_array()
        .unwrap_or_else(|| panic!("not_run.hosts must be an array"));
    assert_eq!(
        hosts.len(),
        3,
        "not_run must disclose exactly 3 hosts (linux-x86_64, linux-aarch64, macos-x86_64)"
    );
    for host in hosts {
        let platform = host["platform"].as_str().unwrap_or_default();
        let commands = host["commands"].as_str().unwrap_or_default();
        assert!(!platform.is_empty(), "each not_run host must name its platform");
        assert!(!commands.is_empty(), "each not_run host must carry exact reproduction commands");
    }
    let platforms: Vec<&str> = hosts.iter().filter_map(|h| h["platform"].as_str()).collect();
    for required in ["linux-x86_64", "linux-aarch64", "macos-x86_64"] {
        assert!(platforms.contains(&required), "not_run must disclose the '{required}' host");
    }

    // If scope claims full scale, the not_run disclosure must be absent or
    // empty — consistency check.
    let full_scale = scope["is_full_spec_scale"].as_bool().unwrap_or(false);
    assert!(
        !(full_scale && !hosts.is_empty()),
        "scope cannot claim full spec scale while 3-host reproduction is disclosed not_run"
    );
}

fn run_court() -> ReproVerdict {
    let Some(path) = locate_receipt() else {
        return ReproVerdict::Skipped {
            reason: "no receipt at target/nato_reproducible_receipt.json; run bin/nato_reproducible_build.sh to produce one",
        };
    };
    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) => panic!("receipt at {} unreadable: {e}", path.display()),
    };
    let receipt: Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("receipt at {} is not valid JSON: {e}", path.display()));
    schema_result(&receipt).unwrap_or_else(|e| panic!("receipt schema violation: {e}"));
    ReproVerdict::Validated { byte_identical: receipt["byte_identical"].as_bool().unwrap_or(false) }
}

/// `Result`-returning wrapper over [`validate_schema`]: the panicking court
/// assertion is caught and rendered as `Err`; a schema-valid receipt is
/// `Ok(())`.
fn schema_result(receipt: &Value) -> Result<(), String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| validate_schema(receipt))).map_err(
        |payload| {
            payload.downcast_ref::<&str>().map_or_else(
                || payload.downcast_ref::<String>().cloned().unwrap_or_default(),
                |s| (*s).to_string(),
            )
        },
    )
}

/// The court: validate the receipt schema when present; typed SKIP when
/// absent.
#[test]
fn reproducible_build_receipt_schema_court() {
    match run_court() {
        ReproVerdict::Validated { byte_identical } => {
            println!("[nato-repro-court] receipt validated (byte_identical={byte_identical})");
            // The court admits the receipt's schema; the byte verdict is
            // reported honestly, and a false verdict is a FAIL, not a skip:
            assert!(
                byte_identical,
                "receipt present and schema-valid but byte_identical=false: the two clean builds differ"
            );
        }
        ReproVerdict::Skipped { reason } => {
            // JUSTIFICATION: court-harness skip notice; stdout is the court's
            // disclosure channel (no log subscriber in court runs).
            #[allow(clippy::print_stdout)]
            {
                println!("[nato-repro-court] TYPED SKIP: {reason}");
            }
        }
    }
}

/// Schema failures are caught, not skipped: a tampered receipt (missing
/// disclosure, wrong digest length) must be refused by the same validator
/// the real receipt goes through.
/// Hoisted body of [`tampered_receipts_are_refused`] (also the positive
/// control of the N19 court adapter): the honest baseline receipt validates,
/// and every injected mutation is refused by the same validator the real
/// receipt goes through. Returns false on the first non-refusal.
fn tampered_mutations_refused() -> bool {
    let base = honest_base_receipt();
    if schema_result(&base).is_err() {
        return false; // honest baseline must validate
    }
    let mutations: Vec<(Value, &str)> = tamper_mutations(base);
    for (m, label) in &mutations {
        if schema_result(m).is_ok() {
            // JUSTIFICATION: court-harness diagnostic; no log subscriber in runs.
            #[allow(clippy::print_stderr)]
            {
                eprintln!("[nato-repro-court] control: mutation NOT refused: {label}");
            }
            return false;
        }
    }
    true
}

/// The honest baseline receipt used by the tamper control.
fn honest_base_receipt() -> Value {
    serde_json::json!({
        "receipt_kind": "nato_reproducible_build",
        "artifact_a": "libchicago_tdd_tools.rlib",
        "artifact_b": "libchicago_tdd_tools.rlib",
        "sha256_a": &"a".repeat(64),
        "sha256_b": &"b".repeat(64),
        "byte_identical": true,
        "comparison_method": "cmp + sha256",
        "toolchain": {"rustc": "r", "cargo": "c", "host_os": "Darwin", "host_arch": "arm64"},
        "scope": {"delivered": "macOS same-host", "is_full_spec_scale": false},
        "not_run": {
            "reason": "cross-host reproduction not performed",
            "hosts": [
                {"platform": "linux-x86_64", "commands": "docker run ..."},
                {"platform": "linux-aarch64", "commands": "docker run --platform linux/arm64 ..."},
                {"platform": "macos-x86_64", "commands": "rustup target add x86_64-apple-darwin && ..."}
            ]
        }
    })
}

/// The six tamper mutations (each `(mutated_receipt, label)`), derived from
/// [`tampered_receipts_are_refused`].
fn tamper_mutations(base: Value) -> Vec<(Value, &'static str)> {
    // Mutation 1: drop the not_run disclosure entirely.
    let mut m1 = base.clone();
    m1.as_object_mut().unwrap().remove("not_run");
    // Mutation 2: truncate the disclosure to 2 hosts.
    let mut m2 = base.clone();
    m2["not_run"]["hosts"].as_array_mut().unwrap().truncate(2);
    // Mutation 3: empty command strings (disclosure theater).
    let mut m3 = base.clone();
    for h in m3["not_run"]["hosts"].as_array_mut().unwrap() {
        h["commands"] = Value::String(String::new());
    }
    // Mutation 4: short sha digest.
    let mut m4 = base.clone();
    m4["sha256_a"] = Value::String("abcd".into());
    // Mutation 5: wrong receipt kind.
    let mut m5 = base.clone();
    m5["receipt_kind"] = Value::String("other".into());
    // Mutation 6: claim full spec scale while 3-host is not_run.
    let mut m6 = base;
    m6["scope"]["is_full_spec_scale"] = Value::Bool(true);
    vec![
        (m1, "missing not_run"),
        (m2, "2-host disclosure"),
        (m3, "empty commands"),
        (m4, "short digest"),
        (m5, "wrong kind"),
        (m6, "full-scale claim alongside not_run"),
    ]
}

#[test]
fn tampered_receipts_are_refused() {
    // The honest baseline passes.
    schema_result(&honest_base_receipt()).expect("honest baseline receipt must validate");
    for (m, label) in tamper_mutations(honest_base_receipt()) {
        assert!(schema_result(&m).is_err(), "{label} must be refused");
    }
}

// ---------------------------------------------------------------------------
// N19 court adapter (W-NTW): trait wiring per the mils.rs N01 template
// ---------------------------------------------------------------------------

/// N19 reproducible-build contract court. The adapter maps the existing
/// `run_court` verdict: `Validated { true }` -> Pass, `Validated { false }`
/// -> Fail, `Skipped { reason }` -> `Verdict::Refused` with the reason
/// carried in the run's [`ScopingRecord`] (a skip is NEVER fabricated into
/// a Pass).
pub struct ReproducibleCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl ReproducibleCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N19;
    /// Capability pillar.
    pub const PILLAR: Pillar = Pillar::Proof;
}

/// Positive control: the tamper-mutation court — the honest baseline
/// validates and all six injected tamper mutations are refused by the same
/// validator the real receipt goes through. Returns false if vacuous.
pub(crate) fn reproducible_positive_control() -> bool {
    tampered_mutations_refused()
}

/// Hoisted court run: the existing `run_court` packaged as a
/// [`CourtOutcome`]. A panic inside the court's own validation (unreadable
/// or JSON-invalid receipt) is caught and mapped to `Verdict::Fail` — a
/// broken subject fails, it never passes by accident.
pub(crate) fn reproducible_run_court_probe() -> CourtOutcome {
    let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(run_court));
    let (verdict, skip_reason): (Verdict, Option<&'static str>) = match ran {
        Ok(ReproVerdict::Validated { byte_identical: true }) => (Verdict::Pass, None),
        Ok(ReproVerdict::Validated { byte_identical: false }) => (Verdict::Fail, None),
        Ok(ReproVerdict::Skipped { reason }) => (Verdict::Refused, Some(reason)),
        // A panic inside the court's own validation (unreadable or
        // JSON-invalid receipt) is a FAIL, never a pass by accident.
        Err(_) => (Verdict::Fail, None),
    };
    let (delivered, bound): (&'static str, &'static str) = if skip_reason.is_some() {
        (
            "typed skip: no receipt present — the court did not evaluate the candidate",
            "receipt absent at target/nato_reproducible_receipt.json; run \
             bin/nato_reproducible_build.sh to produce one",
        )
    } else {
        (
            "receipt schema validation via the shared run_court validator; \
             tamper-mutation positive control (6 mutations)",
            "macOS same-host only; the 3-host cross-host reproduction is the \
             script's mandatory not_run disclosure, not run by this court",
        )
    };
    // A Skipped reason must survive verbatim into the disclosure record.
    let bound = skip_reason.map_or(bound.to_string(), |reason| {
        format!("{bound} [typed skip reason: {reason}]")
    });
    let byte_identical = matches!(verdict, Verdict::Pass);
    CourtOutcome {
        verdict,
        metrics: vec![
            ("receipt_present", MetricValue::Bool(!matches!(verdict, Verdict::Refused))),
            ("byte_identical", MetricValue::Bool(byte_identical)),
        ],
        scoping: ScopingRecord {
            spec_mandate: "reproducible-build contract: double clean-target build compared \
                           byte-wise with toolchain identity and a mandatory 3-host not_run \
                           disclosure, admitted against the receipt schema"
                .into(),
            delivered_evidence: delivered.into(),
            is_full_spec_scale: false,
            bound,
        },
    }
}

impl Court for ReproducibleCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N19 reproducible_build"
    }

    fn positive_control(&self) -> ControlProbe {
        reproducible_positive_control
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        reproducible_run_court_probe()
    }
}
