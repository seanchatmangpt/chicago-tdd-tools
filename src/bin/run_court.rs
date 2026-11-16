//! Court-runner CLI: JSON-in/JSON-out entrypoint over the real NATO court
//! registry (`src/nato/court.rs` `run_court`), for the xaas sa2a loop and
//! sa2a-chicago-court-pack templates to target.
//!
//! Usage:
//!
//! ```text
//! run_court <N01..N20 | all> [--candidate <id>] [--subject <sha>]
//!           [--json-out <path>]
//! ```
//!
//! Exit codes: 0 = all receipts `Pass`; 1 = any receipt `Fail`/`Refused`;
//! 2 = typed usage/runtime error (unknown court name, bad flags, IO failure).
#![allow(
    warnings,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
#![allow(missing_docs)] // Binary crate - documentation not required

use std::io::Write;
use std::process::ExitCode;

use chicago_tdd_tools::nato::court::{run_court, CandidateRef};
use chicago_tdd_tools::nato::receipt::CourtReceipt;
use chicago_tdd_tools::nato::registry::COURT_REGISTRY;

/// Exit code: court(s) adjudicated, at least one non-`Pass` verdict.
const EXIT_VERDICT_FAIL: u8 = 1;
/// Exit code: typed error (bad args, unknown court, IO failure).
const EXIT_USAGE: u8 = 2;

/// Parse `<court> [--candidate <id>] [--subject <sha>] [--json-out <path>]`.
///
/// Returns `(court_selector, candidate, subject_sha, json_out_path)`.
fn parse_args(
    args: &[String],
) -> Result<(String, String, String, Option<String>), String> {
    let mut selector: Option<String> = None;
    let mut candidate = String::from("cli-default-candidate");
    let mut subject = String::from("cli-default-subject");
    let mut json_out: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--candidate" => {
                i += 1;
                candidate = args
                    .get(i)
                    .ok_or_else(|| "--candidate requires a value".to_string())?
                    .clone();
            }
            "--subject" => {
                i += 1;
                subject = args
                    .get(i)
                    .ok_or_else(|| "--subject requires a value".to_string())?
                    .clone();
            }
            "--json-out" => {
                i += 1;
                json_out = Some(
                    args.get(i)
                        .ok_or_else(|| "--json-out requires a value".to_string())?
                        .clone(),
                );
            }
            other => {
                if other.starts_with('-') {
                    return Err(format!("unknown flag: {other}"));
                }
                if selector.is_some() {
                    return Err(format!("multiple court selectors given: {other}"));
                }
                selector = Some(other.to_ascii_uppercase());
            }
        }
        i += 1;
    }

    let selector =
        selector.ok_or_else(|| "usage: run_court <N01..N20 | all> [flags]".to_string())?;
    Ok((selector, candidate, subject, json_out))
}

/// Resolve a court selector to the registry entries it names.
///
/// Case-insensitive on the canonical names (`n01` == `N01`); `"all"` selects
/// every registered court in `N01`..`N20` order.
fn resolve_selector(selector: &str) -> Result<Vec<usize>, String> {
    if selector.eq_ignore_ascii_case("all") {
        return Ok((0..COURT_REGISTRY.len()).collect());
    }
    let matches: Vec<usize> = COURT_REGISTRY
        .iter()
        .enumerate()
        .filter(|(_, e)| e.id.canonical().eq_ignore_ascii_case(selector))
        .map(|(i, _)| i)
        .collect();
    match matches.len() {
        1 => Ok(matches),
        0 => Err(format!(
            "REFUSED:unknown-court: {selector} is not a registered court (expected N01..N20 or all)"
        )),
        _ => unreachable!("canonical court names are unique"),
    }
}

/// Run the selected courts for real (positive control + adversarial run) and
/// collect their receipts. On unix, fd 1 is redirected to stderr while the
/// courts execute so court-internal audit prints (e.g. N06's census JSON)
/// cannot contaminate the receipt-JSON stdout channel; fd 1 is restored
/// before any receipt bytes are emitted.
fn run_selected(
    indices: &[usize],
    candidate: &str,
    subject_sha: &str,
) -> Vec<CourtReceipt> {
    let _guard = StdoutRedirect::to_stderr();
    indices
        .iter()
        .map(|&i| {
            let e = &COURT_REGISTRY[i];
            run_court((e.ctor)().as_ref(), &CandidateRef(candidate.to_string()), subject_sha)
        })
        .collect()
}

/// RAII guard: point fd 1 at fd 2 while alive, restore on drop (unix only;
/// no-op elsewhere). Flushes the std stdout buffer before and after so no
/// buffered receipt bytes leak through the redirect window.
#[cfg(unix)]
struct StdoutRedirect {
    saved_fd: i32,
}

#[cfg(unix)]
extern "C" {
    fn dup(fd: i32) -> i32;
    fn dup2(oldfd: i32, newfd: i32) -> i32;
}

#[cfg(unix)]
impl StdoutRedirect {
    fn to_stderr() -> Self {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        let saved_fd = unsafe { dup(1) };
        if saved_fd < 0 {
            // Cannot save fd 1: run without suppression rather than fail.
            return Self { saved_fd: -1 };
        }
        unsafe {
            dup2(2, 1);
        }
        Self { saved_fd }
    }
}

#[cfg(unix)]
impl Drop for StdoutRedirect {
    fn drop(&mut self) {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        if self.saved_fd >= 0 {
            unsafe {
                dup2(self.saved_fd, 1);
                // Do not close saved_fd via close(); the fd was never closed,
                // dup2 aliases it — closing it here would drop the live stdout
                // descriptor. (Leaked fd is process-exit-bounded.)
            }
        }
    }
}

#[cfg(not(unix))]
struct StdoutRedirect;

#[cfg(not(unix))]
impl StdoutRedirect {
    fn to_stderr() -> Self {
        Self
    }
}

/// Serialize receipts to pretty JSON.
fn receipts_to_json(receipts: &[CourtReceipt]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(receipts)
}

/// True when every receipt is a `Pass` verdict.
fn all_pass(receipts: &[CourtReceipt]) -> bool {
    receipts
        .iter()
        .all(|r| matches!(r.verdict, chicago_tdd_tools::nato::court::Verdict::Pass))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (selector, candidate, subject, json_out) = match parse_args(&args) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("run_court: REFUSED:bad-args: {msg}");
            return ExitCode::from(EXIT_USAGE);
        }
    };

    let indices = match resolve_selector(&selector) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("run_court: {msg}");
            return ExitCode::from(EXIT_USAGE);
        }
    };

    let receipts = run_selected(&indices, &candidate, &subject);

    let json = match receipts_to_json(&receipts) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("run_court: REFUSED:serialize: {e}");
            return ExitCode::from(EXIT_USAGE);
    }
    };

    if let Some(path) = &json_out {
        if let Err(e) = std::fs::write(path, format!("{json}\n")) {
            eprintln!("run_court: REFUSED:io: cannot write {path}: {e}");
            return ExitCode::from(EXIT_USAGE);
        }
    } else {
        let mut out = std::io::stdout().lock();
        if let Err(e) = out.write_all(json.as_bytes()).and_then(|()| out.write_all(b"\n")) {
            eprintln!("run_court: REFUSED:io: stdout write failed: {e}");
            return ExitCode::from(EXIT_USAGE);
        }
    }

    if all_pass(&receipts) {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(EXIT_VERDICT_FAIL)
    }
}
