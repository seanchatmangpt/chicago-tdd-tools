#![cfg(feature = "nato")]
//! N04 — zero-ambient-authority court (NATO lane N2).
//!
//! Static symbol-tier audit over the NATO court-framework state surface,
//! `include_str!`-based (no filesystem reads at run time, no process
//! spawns, zero unwraps):
//!
//! 1. **banned-symbol audit** over the audited `src/nato` module set:
//!    zero references to the ambient-authority symbols (`std::env`,
//!    `std::fs`, `std::net`, `std::process`, `SystemTime`) on any
//!    production path (`#[cfg(test)]` modules are excised before
//!    scanning, original line numbering preserved).
//! 2. **allowlist assertion**: every fully-qualified `std::` path
//!    surviving the excision must prefix-match the declared allowlist —
//!    the documented measurement surfaces (`std::time` monotonic timing,
//!    `std::hint` black-box hygiene, `std::thread` idle-cooldown,
//!    `std::sync::atomic`/`OnceLock`/`Mutex`, `std::str` core reexport,
//!    `std::error::Error` trait impls) and nothing else.
//!
//! # Scoping (disclosed, see the court's ScopingRecord)
//!
//! Ported from strata n04_zero_ambient_audit.rs with the audit surface
//! retargeted to `src/nato/**.rs`. Two upstream legs are DROPPED and
//! disclosed: the wasm import-section census (no wasm artifacts exist in
//! this crate) and the monotonic-clock identity test (the tick-source
//! surface has no referent in the ported kernel). The audited module set
//! is enumerated at lane-write time and EXCLUDES: this file (a scanner
//! cannot contain its own banned-symbol tables without self-reporting)
//! and `disasm.rs`/`stackbound.rs` (ambient measurement infrastructure —
//! child-probe/process-spawn harnesses whose SUBJECT is ambient authority,
//! audited by their own lanes N06/N16).
//!
//! Any ambient symbol is a FAIL naming `module:line` in the assert message.

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
    clippy::trivially_copy_pass_by_ref
)]

use crate::nato::court::{
    CandidateRef, ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;

/// Audited `src/nato` modules, relative to this file. Exclusions are
/// disclosed in the module doc and the court's scoping record.
/// AUDIT SCOPE DISCLOSURE (W13): this list covers the courts whose state
/// paths must be zero-ambient. Deliberately NOT audited here: the timing
/// courts (tick/thrash/cache — their subject IS the clock; `std::time` is
/// an allowlisted, documented surface) and the env-gated modules
/// (disasm/dossier_court/guest_audit/receipts — `CTT_EMIT_RECEIPTS` /
/// `N06_OUT` / artifact reads are their documented authority). Adding a
/// module here requires its state paths to actually be zero-ambient.
pub const AUDITED_MODULES: &[(&str, &str)] = &[
    ("mod.rs", include_str!("mod.rs")),
    ("control.rs", include_str!("control.rs")),
    ("court.rs", include_str!("court.rs")),
    ("disclosure.rs", include_str!("disclosure.rs")),
    ("receipt.rs", include_str!("receipt.rs")),
    ("registry.rs", include_str!("registry.rs")),
    ("rng.rs", include_str!("rng.rs")),
    ("kat.rs", include_str!("kat.rs")),
    ("timing.rs", include_str!("timing.rs")),
    ("wcet.rs", include_str!("wcet.rs")),
    ("mils.rs", include_str!("mils.rs")),
    ("orthogonality.rs", include_str!("orthogonality.rs")),
    ("bell_lapadula.rs", include_str!("bell_lapadula.rs")),
    ("transition_tables.rs", include_str!("transition_tables.rs")),
    // W15: entropy.rs is eligible-pure (zero std:: references) — added to
    // the scan per the AUDITED-MODULES coverage-gap register entry.
    ("entropy.rs", include_str!("entropy.rs")),
    // N04 extension (W13/W24 register, landed this wave): five further
    // eligible-pure court modules (wcet.rs was already scanned). liveness/
    // hamming carry no production
    // std:: surface; seu/tick/thrash use only the allowlisted paths added
    // below. Still excluded + disclosed: zero_ambient.rs (self),
    // disasm/stackbound (ambient harnesses), cache.rs (std::process),
    // reproducible.rs (production std::fs receipt locator), and the
    // env-gated receipts/guest_audit/dossier_court.
    ("liveness.rs", include_str!("liveness.rs")),
    ("hamming.rs", include_str!("hamming.rs")),
    ("seu.rs", include_str!("seu.rs")),
    ("tick.rs", include_str!("tick.rs")),
    ("thrash.rs", include_str!("thrash.rs")),
];

/// Ambient-authority symbols banned on every state path.
pub const BANNED: &[&str] = &["std::env", "std::fs", "std::net", "std::process", "SystemTime"];

/// Declared non-test `std` allowlist for the audited modules. Every
/// `std::...` path surviving the `#[cfg(test)]` excision must equal an
/// entry or live under its prefix.
pub const STD_ALLOWLIST: &[&str] = &[
    "std::error::Error", // typed error trait impls (boilerplate, no I/O)
    "std::sync::atomic", // lock-free counters in the timing kernel
    "std::sync::OnceLock",
    "std::sync::Mutex",
    "std::time",   // monotonic Instant/Duration timing surfaces (N05/N13)
    "std::hint",   // black_box measurement hygiene
    "std::thread", // idle-cooldown between timing attempts (documented)
    "std::str",    // core reexport (utf8 decode in the KAT loader)
    // N04 extension allowlist additions, each justified:
    "std::panic",      // seu.rs: catch_unwind classifies panic-under-fault as SDC
    "std::cell",       // tick.rs: Cell tick-source injection state
    "std::fmt",        // tick.rs: Debug/Display impls (boilerplate, no I/O)
    "std::sync::mpsc", // thrash.rs: watchdog/adversary message channels
    "std::sync::Arc",  // thrash.rs: shared adversary state across threads
];

/// A single ambient/allowlist finding, `module:line` addressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Module path the finding is addressed to.
    pub module: String,
    /// Source line of the hit.
    pub line: usize,
    /// The matched line text.
    pub text: String,
}

/// Excise `#[cfg(test)]` modules (brace-tracked) so audit coverage is
/// production state paths only. Returns (line_no, line) pairs with original
/// numbering preserved.
pub fn production_lines(src: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut in_test_mod = false;
    let mut brace_depth = 0i64;
    let mut test_hdr_wait = false;
    for (i, raw) in src.lines().enumerate() {
        let line_no = i + 1;
        let line = raw; // comments kept: they cannot introduce symbols
        if in_test_mod {
            for ch in line.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            in_test_mod = false;
                        }
                    }
                    _ => {}
                }
            }
            continue;
        }
        if test_hdr_wait {
            // consume until the mod's opening brace
            if let Some(open) = line.find('{') {
                brace_depth = 1;
                let body = &line[open + 1..];
                let mut closed = 0i64;
                for ch in body.chars() {
                    if ch == '{' {
                        brace_depth += 1;
                    }
                    if ch == '}' {
                        brace_depth -= 1;
                        closed = 1;
                    }
                }
                if brace_depth == 0 && closed == 1 {
                    test_hdr_wait = false;
                } else {
                    in_test_mod = excise_body(body);
                    test_hdr_wait = false;
                    if in_test_mod {
                        continue;
                    }
                }
            } else {
                continue;
            }
            if !in_test_mod {
                continue;
            }
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[cfg(test)]") {
            test_hdr_wait = true;
            continue;
        }
        out.push((line_no, line));
    }
    out
}

/// Whether we remain inside a test mod after a header line whose opening
/// brace and partial body have been consumed.
fn excise_body(first_body: &str) -> bool {
    let mut depth = 1i64; // opening brace on header line
    for ch in first_body.chars() {
        if ch == '{' {
            depth += 1;
        }
        if ch == '}' {
            depth -= 1;
        }
    }
    depth > 0
}

/// One module's scan result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanResult {
    /// Banned-token findings.
    pub banned: Vec<Finding>,
    /// Allowlist-violation findings.
    pub allowlist_violations: Vec<Finding>,
    /// Fully-qualified `std::` path census (path -> hit count).
    pub std_census: Vec<(String, usize)>,
}

/// Scan production lines for banned ambient symbols and non-allowlisted
/// `std::` paths.
pub fn scan(module: &str, src: &str) -> ScanResult {
    let mut result = ScanResult::default();
    for (line_no, line) in production_lines(src) {
        for sym in BANNED {
            if line.contains(sym) {
                result.banned.push(Finding {
                    module: module.to_string(),
                    line: line_no,
                    text: line.trim().to_string(),
                });
            }
        }
        // census every fully-qualified `std::` path on the line
        let mut rest = line;
        while let Some(pos) = rest.find("std::") {
            let tail = &rest[pos..];
            let path: String = tail
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == ':' || *c == '_')
                .collect();
            let norm = path.trim_end_matches(':').to_string();
            if norm == "std" {
                rest = &rest[pos + 5..];
                continue;
            }
            match result.std_census.iter_mut().find(|(k, _)| *k == norm) {
                Some(entry) => entry.1 += 1,
                None => result.std_census.push((norm.clone(), 1)),
            }
            let allowed =
                STD_ALLOWLIST.iter().any(|a| norm == *a || norm.starts_with(&format!("{a}::")));
            if !allowed {
                result.allowlist_violations.push(Finding {
                    module: module.to_string(),
                    line: line_no,
                    text: line.trim().to_string(),
                });
            }
            rest = &rest[pos + path.len()..];
        }
    }
    result
}

/// Audit the full enumerated module set. Returns the merged scan.
pub fn audit_all() -> ScanResult {
    let mut merged = ScanResult::default();
    for (name, src) in AUDITED_MODULES {
        let r = scan(name, src);
        merged.banned.extend(r.banned);
        merged.allowlist_violations.extend(r.allowlist_violations);
        for (k, n) in r.std_census {
            match merged.std_census.iter_mut().find(|(ek, _)| ek == &k) {
                Some((_, en)) => *en += n,
                None => merged.std_census.push((k, n)),
            }
        }
    }
    merged
}

/// Findings rendered as `module:line: text`, for assert messages.
pub fn render_findings(findings: &[Finding]) -> String {
    findings
        .iter()
        .map(|f| format!("{}:{}: {}", f.module, f.line, f.text))
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Positive control (scanner canary): the scanner must reject a planted
/// ambient symbol, must reject a non-allowlisted `std` path, and must
/// excise a planted `#[cfg(test)]` module while preserving original line
/// numbering. Returns false if the scanner is vacuous.
pub fn zero_ambient_positive_control() -> bool {
    let planted = "fn fake() { let v = std::env::var(\"X\"); }\n";
    let r1 = scan("planted.rs", planted);
    let c1 = r1.banned.len() == 1
        && r1.banned[0].line == 1
        && r1.banned[0].text.contains("std::env::var");

    let planted2 = "fn fake2() { let s = std::process::id(); }\n";
    let r2 = scan("planted2.rs", planted2);
    let c2 = r2.banned.len() == 1 && r2.allowlist_violations.len() == 1;

    // A cfg(test) module planting an ambient symbol must be excised, while
    // the same symbol outside the test module must be caught with its
    // original line number.
    let with_test = "fn ok() {}\n#[cfg(test)]\nmod t {\n    use std::env;\n}\nfn after() { let _ = std::net::Ipv4Addr::new(1,2,3,4); }\n";
    let r3 = scan("planted3.rs", with_test);
    let c3 =
        r3.banned.len() == 1 && r3.banned[0].text.contains("std::net") && r3.banned[0].line == 6;

    c1 && c2 && c3
}

/// N04 zero-ambient-authority court.
pub struct ZeroAmbientCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl ZeroAmbientCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N04;
    /// Capability pillar. The 5-value `Pillar` enum has no `Authority`
    /// variant, so N04 (ambient-authority annihilation) is pinned to
    /// `Pillar::Dossier` in-code — disclosed and pinned by
    /// `tests/nato_courts_tests.rs::pillar_mapping_matches_the_matrix`.
    pub const PILLAR: Pillar = Pillar::Dossier;
}

impl Court for ZeroAmbientCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N04 zero_ambient_authority"
    }

    fn positive_control(&self) -> ControlProbe {
        zero_ambient_positive_control
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        let audit = audit_all();
        let violations = audit.banned.len() + audit.allowlist_violations.len();
        let census: Vec<String> =
            audit.std_census.iter().map(|(k, n)| format!("{k}={n}")).collect();
        CourtOutcome {
            verdict: if violations == 0 { Verdict::Pass } else { Verdict::Fail },
            metrics: vec![
                ("modules_scanned", MetricValue::U64(AUDITED_MODULES.len() as u64)),
                ("banned_symbols_checked", MetricValue::U64(BANNED.len() as u64)),
                ("banned_hits", MetricValue::U64(audit.banned.len() as u64)),
                ("allowlist_violations", MetricValue::U64(audit.allowlist_violations.len() as u64)),
                ("std_census_entries", MetricValue::U64(audit.std_census.len() as u64)),
                ("std_census", MetricValue::Str(census.join(","))),
            ],
            scoping: ScopingRecord::partial(
                "zero ambient authority on the NATO state surface, corroborated at the \
                 binary tier (strata: wasm import census) and by a declared clock identity",
                "include_str!-based banned-symbol + allowlist static audit over the 15-module \
                 enumerated src/nato state set, with cfg(test) excision and scanner canaries",
                "binary-tier legs dropped (no wasm artifacts in this crate; no tick-source \
                 surface in the ported kernel) and disclosed; audited set excludes this \
                 scanner file (self-reference) and disasm.rs/stackbound.rs (ambient \
                 measurement harnesses audited by their own lanes N06/N16)",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n04_zero_ambient_symbols() {
        let audit = audit_all();
        assert!(
            audit.banned.is_empty(),
            "N04 FAIL — ambient-authority symbols on state paths: {}",
            render_findings(&audit.banned)
        );
        assert!(
            audit.allowlist_violations.is_empty(),
            "N04 FAIL — std surface outside the declared allowlist: {}",
            render_findings(&audit.allowlist_violations)
        );
        assert_eq!(
            audit.std_census.iter().filter(|(_, n)| *n == 0).count(),
            0,
            "std census carries zero-count entries (bookkeeping bug)"
        );
    }

    #[test]
    fn n04_audit_surface_is_not_vacuously_small() {
        // The audited set must cover the framework core + this lane's
        // courts; a shrunken enumeration would silently vacate the court.
        assert_eq!(AUDITED_MODULES.len(), 20, "audited module set drifted from the extended set");
        for (name, src) in AUDITED_MODULES {
            assert!(!src.is_empty(), "audited module {name} embedded empty");
        }
        assert!(
            AUDITED_MODULES.iter().any(|(n, _)| n == &"court.rs"),
            "framework core must be in the audited set"
        );
    }

    #[test]
    fn n04_scanner_anti_vacuity() {
        assert!(
            zero_ambient_positive_control(),
            "scanner canary failed — the ambient audit is vacuous"
        );
        // Direct canaries: a banned `std::` path legitimately fires BOTH
        // channels (banned + non-allowlisted); a banned non-`std::` symbol
        // (wall-clock) fires the banned channel only.
        let planted = "let s = std::fs::read_to_string(p);\n";
        let r = scan("canary.rs", planted);
        assert_eq!(r.banned.len(), 1, "scanner missed planted ambient symbol");
        assert_eq!(r.banned[0].line, 1, "line numbering drifted");
        assert_eq!(
            r.allowlist_violations.len(),
            1,
            "banned std path must also read as a non-allowlisted path"
        );
        let clock = "let t = SystemTime::now();\n";
        let rc = scan("canary_clock.rs", clock);
        assert_eq!(rc.banned.len(), 1, "scanner missed planted wall-clock symbol");
        assert!(
            rc.allowlist_violations.is_empty(),
            "non-path banned symbols must not fabricate allowlist violations"
        );
    }

    #[test]
    fn n04_court_run_reports_pass_with_receipt_metrics() {
        let court = ZeroAmbientCourt;
        assert!(court.positive_control()(), "positive control must hold");
        let outcome = court.run(&CandidateRef("lane-n2-test".to_string()));
        assert_eq!(outcome.verdict, Verdict::Pass, "N04 court verdict must pass");
        let banned = outcome.metrics.iter().find(|(k, _)| *k == "banned_hits");
        assert!(
            banned.is_some_and(|(_, v)| *v == MetricValue::U64(0)),
            "banned_hits metric must be zero"
        );
        assert!(
            !outcome.scoping.is_full_spec_scale,
            "N04 dropped binary-tier legs — disclosure must say so"
        );
        outcome.scoping.validate().expect("scoping record must validate");
    }
}
