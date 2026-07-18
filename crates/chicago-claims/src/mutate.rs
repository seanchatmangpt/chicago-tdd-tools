//! Mutant activation and oracle attribution for claim-scoped fixture files.
//!
//! # Isolation strategy: in-process direct call, not a `cargo test` subprocess
//!
//! Each fixture in `tests/fixtures/*.rs` is a tiny, self-contained Rust module (no
//! dependency on `bcinr-cmca` or any other crate) exposing a `union(u32, u32) -> u32`
//! function plus its own `#[cfg(test)]` oracle tests. Two ways to determine whether a
//! mutant is killed were considered:
//!
//! 1. Shell out to `cargo test` scoped to the one fixture file, parse its output.
//! 2. Compile the fixture's real function into this process (via `include!`) and
//!    evaluate the same predicate the fixture's own `#[test]` asserts, directly.
//!
//! This module uses (2). Rationale: these fixtures are not wired as their own Cargo
//! test binaries (no `[[test]]` targets, no per-fixture `Cargo.toml`), so "scope
//! `cargo test` to one fixture" would require either giving each fixture its own
//! compilation unit (six new Cargo targets for six tiny functions) or fragile
//! `--test-threads=1 -- fixture_name` filtering against a single combined binary,
//! both of which add process-spawn latency and string-parsing fragility for no
//! semantic gain here: the fixtures have no I/O, no shared state, no concurrency
//! hazards a subprocess boundary would protect against. `include!` pulls the real,
//! unmodified fixture source (function body + consts) into a private submodule of
//! this file at compile time, so the code path actually executed by the intended
//! oracle assertion is byte-identical to what `cargo test --test <fixture>` would
//! run against the standalone file; the mutation harness itself never re-implements
//! or approximates the corrupted logic.
//!
//! One caveat this buys back: because fixtures are pulled in via `include!`, their
//! own `#[cfg(test)] mod tests` blocks compile as part of *this* crate's test
//! binary too (harmlessly — they're additional, directly-runnable tests reachable
//! via `cargo test -p chicago-claims`), not as isolated binaries. That is a
//! disclosed trade of process isolation for evaluation reliability, not a hidden one.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use thiserror::Error;

use crate::claim::MutantSpec;
use crate::reconcile::MutantClassification;

mod fixture_baseline_correct {
    include!("../tests/fixtures/baseline_correct.rs");
}
mod fixture_first_wins {
    include!("../tests/fixtures/first_wins.rs");
}
mod fixture_last_wins {
    include!("../tests/fixtures/last_wins.rs");
}
mod fixture_left_only {
    include!("../tests/fixtures/left_only.rs");
}
mod fixture_right_only {
    include!("../tests/fixtures/right_only.rs");
}
mod fixture_overwrite {
    include!("../tests/fixtures/overwrite.rs");
}
mod fixture_empty_set {
    include!("../tests/fixtures/empty_set.rs");
}

/// Outcome of evaluating one fixture's two oracle predicates in-process: whether the
/// intended oracle assertion holds (`true` = law preserved, matches
/// `oracle_union_preserves_both_operands_distinct_faults` passing), and whether the
/// secondary/incidental oracle (commutativity) holds.
struct OracleOutcome {
    intended_holds: bool,
    secondary_holds: bool,
}

/// Evaluate the two oracle predicates against a known fixture's real `union`
/// function, called in-process (see module doc comment for why). Returns `None` if
/// `fixture_key` does not match any fixture wired into this module.
fn evaluate_fixture(fixture_key: &str) -> Option<OracleOutcome> {
    macro_rules! outcome_for {
        ($m:ident) => {{
            use $m::{union, FAULT_A, FAULT_B};
            OracleOutcome {
                intended_holds: union(FAULT_A, FAULT_B) == (FAULT_A | FAULT_B),
                secondary_holds: union(FAULT_A, FAULT_B) == union(FAULT_B, FAULT_A),
            }
        }};
    }

    let outcome = match fixture_key {
        "baseline_correct" => outcome_for!(fixture_baseline_correct),
        "first_wins" => outcome_for!(fixture_first_wins),
        "last_wins" => outcome_for!(fixture_last_wins),
        "left_only" => outcome_for!(fixture_left_only),
        "right_only" => outcome_for!(fixture_right_only),
        "overwrite" => outcome_for!(fixture_overwrite),
        "empty_set" => outcome_for!(fixture_empty_set),
        _ => return None,
    };
    Some(outcome)
}

/// Witness that a [`MutationProvider`] either could or could not bring a named
/// mutant into a runnable state (fixture key resolved and wired; external crate
/// path/feature/test triple present and pointing at a real `Cargo.toml`). This is
/// deliberately independent of the oracle's pass/fail outcome: activation answers
/// "can this mutant be exercised at all", not "was it caught".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationWitness {
    /// Whether the mutant could be brought into a runnable state.
    pub activated: bool,
    /// Human-readable detail: which fixture/feature was resolved, or why
    /// activation failed (missing field, file not found, unknown fixture key).
    pub detail: String,
}

/// The outcome of actually running a mutant's designated oracle, once activated.
/// Deliberately more than a bare `bool`, and deliberately more than the original
/// three-way `{Killed, Survived, GateFailed}` split: a subprocess-backed,
/// build-from-source provider ([`CargoFeatureProvider`], [`PatchOverlayProvider`])
/// has several distinct ways to fail to reach a kill/survive determination, and
/// collapsing them all into one generic "gate failed" bucket would hide which
/// stage of the pipeline actually broke (a receipt reader cannot tell "the patch
/// pattern didn't match" from "the build broke" from "the process hung" if they
/// all report the same variant). Each variant below is produced by a distinct,
/// disjoint condition — see [`PatchOverlayProvider::run_oracle`] for where each
/// one is actually decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleOutcomeKind {
    /// The oracle's result indicates the mutant was detected/killed.
    Killed,
    /// The oracle's result indicates the mutant was NOT detected (survived).
    Survived,
    /// The oracle could not be run to a real pass/fail determination, for a
    /// harness-level reason not covered by the more specific variants below
    /// (subprocess spawn failure, filesystem I/O failure while setting up the
    /// run). Retained as-is (not renamed/removed) because [`FixtureProvider`]
    /// and [`CargoFeatureProvider`] already produce it and are preserved
    /// byte-identical to their pre-existing behavior.
    GateFailed,
    /// The isolated copy of the target source did not build: the subprocess ran
    /// but never reached a `test result:` summary line at all (as opposed to
    /// reaching one that reports zero matched tests — see
    /// [`Self::OracleInfrastructureFailure`]).
    CompilationFailure,
    /// The mutant's configured search pattern could not be uniquely located in
    /// the target file (zero or more than one occurrence), so no patch was
    /// applied and no build/oracle was attempted.
    MutationActivationFailure,
    /// The isolated copy built successfully and cargo reached a real
    /// `test result:` summary line, but that summary reports zero tests
    /// matched (the name/binary filter selected nothing) — build succeeded,
    /// the oracle itself never ran.
    OracleInfrastructureFailure,
    /// The oracle subprocess exceeded its configured time budget and was killed
    /// before reaching a pass/fail determination.
    Timeout,
}

/// Result of running a mutant's designated oracle, prior to classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleResult {
    /// The coarse outcome kind (see [`OracleOutcomeKind`]).
    pub outcome: OracleOutcomeKind,
    /// Human-readable detail: raw predicate inputs (in-process provider) or a
    /// tail of subprocess stdout/stderr (external-process provider), carried
    /// through for the report/ledger.
    pub detail: String,
    /// A non-cryptographic digest (`std::hash::DefaultHasher`/SipHash, hex
    /// encoded) of the FULL captured stdout+stderr from a subprocess-backed
    /// provider's run, for receipt/ledger correlation ("did this run produce
    /// byte-identical output to a prior run"). `None` for [`FixtureProvider`]
    /// (in-process evaluation captures no subprocess output to hash) and for
    /// any early-return path where no subprocess was actually spawned.
    /// Disclosed explicitly: this is NOT a cryptographic hash and must not be
    /// treated as tamper-evident — it is a cheap collision-resistant-enough
    /// change detector, not a BLAKE3-grade receipt primitive. If a
    /// cryptographic receipt is required downstream, recompute one from the
    /// raw captured bytes.
    pub output_digest: Option<String>,
}

/// Compute [`OracleResult::output_digest`] over the full, untruncated
/// stdout/stderr text. See that field's doc comment for the digest's disclosed
/// scope (non-cryptographic).
fn digest_output(stdout: &str, stderr: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    stdout.hash(&mut hasher);
    stderr.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// A pluggable mechanism for activating a named mutant against claim-scoped code
/// and running its intended oracle, producing a typed [`MutantClassification`].
///
/// Two concrete providers implement this trait: [`FixtureProvider`] (the original
/// MLP's in-process `include!`-based fixture mechanism, unchanged in behavior —
/// see `activate_and_test_mutant`) and [`CargoFeatureProvider`] (a genuinely new
/// capability: drives a real `cargo test -p <crate> --features <feature>
/// <test_name> -- --exact` subprocess against an external crate, e.g.
/// `bcinr-cmca`'s `mutant_1..mutant_11` cfg-feature mutants).
///
/// `classify`'s default implementation is the shared activate-then-run-oracle
/// pipeline both providers use; a provider only needs to implement `activate` and
/// `run_oracle`. It is intentionally still overridable (not `final`/sealed) for a
/// future provider whose activation and oracle steps are not cleanly separable.
pub trait MutationProvider {
    /// Declarative execution requirements this provider needs from its runtime
    /// environment (see [`MutationCapabilities`]). Required (no default body) so
    /// every provider must state its own capabilities explicitly rather than
    /// silently inheriting a guess.
    fn capabilities(&self) -> MutationCapabilities;

    /// Attempt to bring `spec`'s named mutant into a runnable state. Must not run
    /// the oracle itself — only confirm the mutant CAN be exercised.
    fn activate(&self, spec: &MutantSpec) -> ActivationWitness;

    /// Run `spec`'s designated oracle against the (already-activatable) mutant and
    /// report its outcome. Callers should check [`Self::activate`] first;
    /// implementations may assume activation would succeed but are not required to
    /// re-derive it.
    fn run_oracle(&self, spec: &MutantSpec) -> OracleResult;

    /// Activate, then run the oracle, and classify the combined outcome into the
    /// shared [`MutantClassification`] vocabulary (reused from [`crate::reconcile`],
    /// not redefined here). Delegates to [`activate_run_classify`], the single
    /// shared pipeline also used by [`execute_mutant`] — kept as one function so
    /// activation and oracle execution each happen exactly once per call, never
    /// duplicated between this trait method and a caller wanting the richer
    /// [`MutantExecutionReport`].
    #[must_use]
    fn classify(&self, spec: &MutantSpec) -> MutantClassification {
        activate_run_classify(self, spec).classification
    }
}

/// Declarative execution requirements a [`MutationProvider`] needs from its
/// runtime environment. Consulted by [`resolve_provider`] to refuse (via a typed
/// [`MutantResolutionError::CapabilityUnavailable`]) a provider whose declared
/// capability cannot actually be met on this machine, rather than silently
/// attempting the mutation and failing deep inside `run_oracle` with a generic,
/// hard-to-diagnose error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutationCapabilities {
    /// Whether this provider must operate on an isolated copy of the target
    /// source tree (never the caller's real working tree) rather than in place.
    /// [`resolve_provider`] probes this one concretely: it attempts to create
    /// and remove a throwaway directory under the system temp root before
    /// resolving a provider that sets this `true`.
    pub requires_isolated_worktree: bool,
    /// Whether this provider's oracle run may need outbound network access
    /// (e.g. a fresh `cargo build` needing to fetch crates not already
    /// vendored/cached). Declarative only today: no provider probes actual
    /// connectivity, so this documents intent for future resolution logic or
    /// operator policy, not a verified guarantee.
    pub requires_network: bool,
    /// Source languages this provider's activation/oracle mechanism
    /// understands.
    pub supported_languages: &'static [&'static str],
}

/// The single shared activate → run_oracle → classify pipeline used by both
/// [`MutationProvider::classify`]'s default implementation and [`execute_mutant`],
/// so a caller wanting the richer [`MutantExecutionReport`] (activation witness +
/// oracle result, not just the terminal classification) never causes a second,
/// real re-execution of a subprocess-backed provider's build/test cycle.
fn activate_run_classify<P: MutationProvider + ?Sized>(
    provider: &P,
    spec: &MutantSpec,
) -> MutantExecutionReport {
    let activation = provider.activate(spec);
    if !activation.activated {
        return MutantExecutionReport {
            classification: MutantClassification::InfrastructureBlocked,
            activation,
            oracle: None,
        };
    }
    let oracle = provider.run_oracle(spec);
    let classification = match oracle.outcome {
        OracleOutcomeKind::Killed => MutantClassification::KilledByIntendedOracle,
        OracleOutcomeKind::Survived => MutantClassification::Survived,
        OracleOutcomeKind::GateFailed => MutantClassification::MutationGateFailed,
        OracleOutcomeKind::CompilationFailure => MutantClassification::CompilationFailed,
        OracleOutcomeKind::MutationActivationFailure => {
            MutantClassification::MutationActivationFailed
        }
        OracleOutcomeKind::OracleInfrastructureFailure => {
            MutantClassification::OracleInfrastructureFailed
        }
        OracleOutcomeKind::Timeout => MutantClassification::Timeout,
    };
    MutantExecutionReport { classification, activation, oracle: Some(oracle) }
}

/// Full evidence bundle from resolving, activating, and running one mutant's
/// oracle — everything needed for a receipt-grade report line, not just the
/// terminal [`MutantClassification`]. Produced by [`execute_mutant`] and by
/// [`MutationProvider::classify`]'s shared internal pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutantExecutionReport {
    /// The computed typed classification.
    pub classification: MutantClassification,
    /// The activation witness (whether/why the mutant could be brought into a
    /// runnable state).
    pub activation: ActivationWitness,
    /// The oracle result, when the oracle actually ran. `None` only when
    /// activation failed (the oracle never ran, so there is nothing to report).
    pub oracle: Option<OracleResult>,
}

/// Fixture keys wired into [`evaluate_fixture`]. Kept as an explicit list (rather
/// than only relying on `evaluate_fixture`'s `None` fallthrough) so
/// [`FixtureProvider::activate`] can report a witness without evaluating the
/// oracle, matching the trait's activate/run_oracle separation.
const KNOWN_FIXTURE_KEYS: [&str; 7] = [
    "baseline_correct",
    "first_wins",
    "last_wins",
    "left_only",
    "right_only",
    "overwrite",
    "empty_set",
];

/// Wraps the pre-existing, in-process `include!`-based fixture mechanism (see the
/// module doc comment) behind [`MutationProvider`]. This is a pure refactor: the
/// activate/run_oracle split below reproduces exactly the same two checks
/// `activate_and_test_mutant` performed before this trait existed (file-stem
/// resolution, then `evaluate_fixture` lookup), so `activate_and_test_mutant`
/// (now a thin wrapper over `FixtureProvider.classify(..)`) is byte-identical in
/// behavior to its pre-refactor self — see
/// `fixture_provider_refactor_matches_pre_refactor_behavior` below and the
/// unchanged `all_six_named_corruptions_are_killed_by_the_intended_oracle` /
/// `baseline_correct_is_not_flagged_as_a_mutant` /
/// `unknown_fixture_path_is_infrastructure_blocked` tests, which assert the exact
/// same classifications as before this refactor.
pub struct FixtureProvider;

impl FixtureProvider {
    fn fixture_key(spec: &MutantSpec) -> Option<String> {
        let fixture_path = spec.fixture_path.as_deref()?;
        Path::new(fixture_path).file_stem().and_then(|s| s.to_str()).map(str::to_string)
    }
}

impl MutationProvider for FixtureProvider {
    fn capabilities(&self) -> MutationCapabilities {
        MutationCapabilities {
            requires_isolated_worktree: false,
            requires_network: false,
            supported_languages: &["rust"],
        }
    }

    fn activate(&self, spec: &MutantSpec) -> ActivationWitness {
        let Some(fixture_key) = Self::fixture_key(spec) else {
            return ActivationWitness {
                activated: false,
                detail: "no fixture_path set, or it has no usable file stem".to_string(),
            };
        };
        if KNOWN_FIXTURE_KEYS.contains(&fixture_key.as_str()) {
            ActivationWitness {
                activated: true,
                detail: format!("fixture `{fixture_key}` wired into evaluate_fixture"),
            }
        } else {
            ActivationWitness {
                activated: false,
                detail: format!("fixture key `{fixture_key}` not wired into evaluate_fixture"),
            }
        }
    }

    fn run_oracle(&self, spec: &MutantSpec) -> OracleResult {
        let fixture_key = Self::fixture_key(spec).unwrap_or_default();
        match evaluate_fixture(&fixture_key) {
            Some(outcome) => OracleResult {
                // No corruption detected (intended_holds == true) means either this
                // is the control (baseline_correct) or the mutant is invisible to
                // this oracle: either way nothing was killed. intended_holds ==
                // false means the mutant deviates from the correct law exactly as
                // `intended_oracle_test` is written to detect.
                outcome: if outcome.intended_holds {
                    OracleOutcomeKind::Survived
                } else {
                    OracleOutcomeKind::Killed
                },
                detail: format!(
                    "intended_holds={}, secondary_holds={}",
                    outcome.intended_holds, outcome.secondary_holds
                ),
                // In-process evaluation: no subprocess output exists to hash.
                output_digest: None,
            },
            None => OracleResult {
                outcome: OracleOutcomeKind::GateFailed,
                detail: format!("fixture `{fixture_key}` not found by evaluate_fixture"),
                output_digest: None,
            },
        }
    }
}

/// Runs a REAL `cargo test -p <crate> --features <feature> [--test <test_binary>]
/// <test_name> -- --exact` subprocess against an external crate on disk (e.g.
/// `bcinr-cmca`'s `mutant_1..mutant_11` cfg-feature mutants, see that crate's
/// `MUTANT_KILL_MATRIX.md`) and classifies the mutant from the REAL, parsed
/// pass/fail of that one named test — not a simulation or a read of a prior
/// report.
///
/// Concretely runs `cargo test --features <feature> [--test <test_binary>]
/// <test_name> -- --exact` with its working directory set to `spec.crate_path`,
/// relying on cargo's own "current package" resolution (running inside a
/// workspace member's directory scopes the test run to that one package,
/// equivalent to `-p <crate>` from the workspace root without needing to
/// separately parse the package name out of `Cargo.toml`).
///
/// `spec.test_binary` (`--test <name>`) is optional but strongly recommended for
/// any crate with more than one test binary — see that field's doc comment in
/// `claim::MutantSpec` for a real failure mode this omission caused during this
/// provider's own development (a name filter with no `--test` scope applies
/// across every test binary in the crate; an unrelated binary's own "0 tests
/// matched" summary line can be misread as the intended test's own result).
///
/// # Polarity note (this is the opposite convention from [`FixtureProvider`])
///
/// `FixtureProvider`'s `intended_oracle_test` names a property that HOLDS for
/// correct code and is VIOLATED by the mutant (holds => survived, violated =>
/// killed). `bcinr-cmca`'s `tests/hostile_mutants.rs` convention (see
/// `MUTANT_KILL_MATRIX.md`) is the opposite shape by construction: each
/// `kill_mutant_N_*` test is itself written to assert the SPECIFIC WRONG value
/// the mutation is expected to produce, and PASSES precisely when that wrong
/// value is observed — i.e. the named test PASSING under the active mutant
/// feature IS the kill signal, and FAILING means the predicted corruption was not
/// observed (survived). This provider therefore maps subprocess test-pass to
/// `Killed`, not `Survived` — documented explicitly here because it is the
/// inverse of `FixtureProvider`'s polarity, each matching its own ecosystem's
/// real, pre-existing test-authoring convention rather than one assumed-universal
/// polarity invented for this trait.
pub struct CargoFeatureProvider;

impl CargoFeatureProvider {
    fn fields(spec: &MutantSpec) -> Option<(&str, &str, &str)> {
        Some((spec.crate_path.as_deref()?, spec.feature.as_deref()?, spec.test_name.as_deref()?))
    }
}

impl MutationProvider for CargoFeatureProvider {
    fn capabilities(&self) -> MutationCapabilities {
        MutationCapabilities {
            // Runs directly against the real crate_path in place — not an
            // isolated copy. Preserved as its existing, pre-established design;
            // PatchOverlayProvider is the isolated alternative for callers that
            // need to never touch the real working tree.
            requires_isolated_worktree: false,
            requires_network: false,
            supported_languages: &["rust"],
        }
    }

    fn activate(&self, spec: &MutantSpec) -> ActivationWitness {
        let Some((crate_path, feature, test_name)) = Self::fields(spec) else {
            return ActivationWitness {
                activated: false,
                detail: "MutantSpec is missing one of crate_path/feature/test_name \
                         required by CargoFeatureProvider"
                    .to_string(),
            };
        };
        let manifest = Path::new(crate_path).join("Cargo.toml");
        if !manifest.is_file() {
            return ActivationWitness {
                activated: false,
                detail: format!(
                    "no Cargo.toml found at {} (crate_path `{crate_path}` does not \
                     look like a real crate directory on this machine)",
                    manifest.display()
                ),
            };
        }
        ActivationWitness {
            activated: true,
            detail: format!(
                "crate `{crate_path}` feature `{feature}` test `{test_name}` ready to run"
            ),
        }
    }

    fn run_oracle(&self, spec: &MutantSpec) -> OracleResult {
        let Some((crate_path, feature, test_name)) = Self::fields(spec) else {
            return OracleResult {
                outcome: OracleOutcomeKind::GateFailed,
                detail: "missing crate_path/feature/test_name at run_oracle time".to_string(),
                output_digest: None,
            };
        };

        let mut args: Vec<&str> = vec!["test", "--features", feature];
        if let Some(test_binary) = spec.test_binary.as_deref() {
            args.push("--test");
            args.push(test_binary);
        }
        args.push(test_name);
        args.push("--");
        args.push("--exact");

        let output = Command::new("cargo").current_dir(crate_path).args(&args).output();

        let output = match output {
            Ok(o) => o,
            Err(e) => {
                return OracleResult {
                    outcome: OracleOutcomeKind::GateFailed,
                    detail: format!("failed to spawn `cargo test` subprocess: {e}"),
                    output_digest: None,
                };
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_tail = tail(&format!("{stdout}\n{stderr}"), 600);
        let output_digest = Some(digest_output(&stdout, &stderr));

        let passed = parse_count_after(&stdout, "passed").unwrap_or(0);
        let failed = parse_count_after(&stdout, "failed").unwrap_or(0);

        if passed == 0 && failed == 0 {
            // No "test result:" line matched any count at all, or it matched with
            // 0/0 (cargo's own behavior when `-- --exact <name>` matches nothing):
            // the harness could not reach a real determination, not "survived".
            return OracleResult {
                outcome: OracleOutcomeKind::GateFailed,
                detail: format!(
                    "cargo test matched zero tests for `{test_name}` under feature \
                     `{feature}` (exit={:?}); output tail: {combined_tail}",
                    output.status.code()
                ),
                output_digest,
            };
        }

        if output.status.success() && failed == 0 && passed > 0 {
            OracleResult {
                outcome: OracleOutcomeKind::Killed,
                detail: format!(
                    "`{test_name}` passed under feature `{feature}` ({passed} passed) — \
                     the predicted corruption was observed"
                ),
                output_digest,
            }
        } else if failed > 0 {
            OracleResult {
                outcome: OracleOutcomeKind::Survived,
                detail: format!(
                    "`{test_name}` failed under feature `{feature}` ({failed} failed) — \
                     the predicted corruption was NOT observed"
                ),
                output_digest,
            }
        } else {
            OracleResult {
                outcome: OracleOutcomeKind::GateFailed,
                detail: format!(
                    "cargo test exited without a parseable pass/fail count \
                     (status={:?}); output tail: {combined_tail}",
                    output.status.code()
                ),
                output_digest,
            }
        }
    }
}

/// Default wall-clock budget (seconds) for [`PatchOverlayProvider`]'s isolated
/// `cargo test` subprocess when a [`MutantSpec`] does not set `timeout_secs`.
pub const DEFAULT_PATCH_OVERLAY_TIMEOUT_SECS: u64 = 180;

/// A directory that is removed (best-effort — a failure to remove is not
/// reported, since there is no further recovery action available and the
/// mutation outcome has already been determined) when it goes out of scope.
/// Used to guarantee [`PatchOverlayProvider`]'s isolated copy is cleaned up on
/// every `run_oracle` return path — including early returns — without
/// duplicating a manual `fs::remove_dir_all` call at each one.
struct IsolatedDirGuard(PathBuf);

impl IsolatedDirGuard {
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for IsolatedDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Create a fresh, uniquely-named directory under the system temp root and
/// return its path. Uniqueness comes from the process ID, a nanosecond
/// timestamp, and a process-local atomic counter — sufficient to avoid
/// collisions between concurrent calls on one machine without pulling in a
/// dependency for cryptographically random names (no adversarial naming
/// pressure exists here: this is a local, single-machine scratch directory, not
/// a security boundary).
fn make_isolated_dir(prefix: &str) -> std::io::Result<PathBuf> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}-{seq}", std::process::id()));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Recursively copy `src` into `dst` (creating `dst` and any needed
/// subdirectories), skipping any entry whose file name matches one in `skip`
/// (used to exclude `.git` and `target` — version-control metadata and build
/// output are neither needed by, nor safe/fast to duplicate for, an isolated
/// mutation run). Symlinks are silently skipped (not expected in a typical
/// crate source tree; copying them naively risks either escaping the isolated
/// copy or an infinite cycle, and this mechanism does not need to support
/// them to satisfy its documented use case).
fn copy_dir_recursive(src: &Path, dst: &Path, skip: &[&str]) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if let Some(name) = file_name.to_str() {
            if skip.contains(&name) {
                continue;
            }
        }
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(&file_name);
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path, skip)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)?;
        }
        // Symlinks (file_type.is_symlink()) fall through and are skipped —
        // see doc comment above.
    }
    Ok(())
}

/// Run `cmd` to completion, killing it and returning `(output, true)` if it has
/// not exited by `timeout`. Drains stdout/stderr on background threads while
/// waiting so a chatty subprocess (a verbose `cargo test` run) cannot deadlock
/// against a full OS pipe buffer before the poll loop next checks in — `std`
/// provides no built-in subprocess timeout, and this is the minimal
/// dependency-free construction of one.
///
/// # Process-GROUP kill, not just child kill (a real bug this fixes)
///
/// `cmd` here is always `cargo test ...`: cargo itself forks and execs the
/// actual compiled test binary as ITS OWN child, so `cmd`'s direct child is
/// `cargo`, not the test binary. An earlier version of this function called
/// only `child.kill()` on timeout, which sends `SIGKILL` to `cargo` alone — the
/// test binary (cargo's grandchild, our great-grandchild) is left running,
/// orphaned, with no code left waiting on ITS exit. Against an injected
/// `loop {}`-shaped mutant this was directly observed to leave a runaway
/// process spinning indefinitely after this function had already returned; it
/// ALSO deadlocked this function itself, because the orphan process still held
/// the inherited stdout/stderr pipe write-ends open, so `read_to_end` on the
/// reader threads below blocked forever waiting for an EOF that killing only
/// `cargo` would never produce. On `unix`, `cmd` is spawned into its own new
/// process group (`process_group(0)`, stable safe API, no `unsafe`), and a
/// timeout sends `SIGKILL` to the whole group (`-<pid>`) via the system `kill`
/// utility invoked with explicit argv (no shell) before the direct-child kill,
/// so the test binary and everything else in the tree dies with it and every
/// pipe fd actually closes. Non-unix targets fall back to a direct-child-only
/// kill (no process-group primitive available) and inherit the corresponding
/// known orphan-process risk — disclosed here rather than silently assumed
/// away.
fn run_with_timeout(
    mut cmd: Command,
    timeout: Duration,
) -> std::io::Result<(std::process::Output, bool)> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd.spawn()?;
    let child_pid = child.id();

    let stdout_handle = child.stdout.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = pipe.read_to_end(&mut buf);
            buf
        })
    });
    let stderr_handle = child.stderr.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = pipe.read_to_end(&mut buf);
            buf
        })
    });

    let start = Instant::now();
    let mut timed_out = false;
    loop {
        match child.try_wait()? {
            Some(_) => break,
            None => {
                if start.elapsed() >= timeout {
                    timed_out = true;
                    #[cfg(unix)]
                    {
                        let _ =
                            Command::new("kill").arg("-KILL").arg(format!("-{child_pid}")).status();
                    }
                    let _ = child.kill();
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
    let status = child.wait()?;
    let stdout = stdout_handle.and_then(|h| h.join().ok()).unwrap_or_default();
    let stderr = stderr_handle.and_then(|h| h.join().ok()).unwrap_or_default();

    Ok((std::process::Output { status, stdout, stderr }, timed_out))
}

/// Drives the GENERAL mutation-execution mechanism: given a claim specifying a
/// target file, a search pattern (the exact production line to corrupt), and a
/// replacement pattern, this provider copies `repo_root` into an isolated temp
/// directory, applies the patch as a real file edit in that isolated copy ONLY
/// (the real `repo_root` working tree is never opened for writing), builds and
/// runs the specified oracle test against the isolated copy via a real `cargo
/// test` subprocess, parses pass/fail, and always cleans up the isolated copy
/// (via [`IsolatedDirGuard`]) before returning.
///
/// This is what lets a claim attack a REAL production implementation directly
/// — e.g. `NumericFaultSet::union` in `bcinr-cmca/src/fixed.rs`, which has no
/// pre-existing `#[cfg(feature = "mutant_N")]` corruption for
/// [`CargoFeatureProvider`] to toggle — without hand-writing a new fixture file
/// or a new match arm in this crate's source for every such law. The mutant is
/// fully described by [`MutantSpec`]'s patch-overlay fields (`repo_root`,
/// `target_file`, `search_pattern`, `replace_pattern`, `crate_dir`), all
/// TOML-configurable.
///
/// # Isolation strategy: temp-directory copy, not `git worktree`
///
/// A `git worktree` was considered (and is explicitly permitted by this
/// mechanism's design brief as the alternative). It was NOT used, for two
/// concrete reasons specific to this crate's operating constraints rather than
/// a general claim that worktrees are the wrong tool elsewhere:
///
/// 1. `git worktree add` writes worktree administrative metadata into the
///    target repository's own `.git` directory (`.git/worktrees/<name>`). For
///    a target repository this crate does not own (e.g. a sibling checkout
///    like `bcinr`), that write lands outside any region this crate is
///    permitted to write to — a plain recursive file copy into a fresh temp
///    directory touches nothing inside the source repository at all, which is
///    the stronger isolation guarantee of the two options here.
/// 2. The target repository is not guaranteed to be a git repository at all
///    (nothing in `MutantSpec`'s schema requires `repo_root` to have a
///    `.git`); a temp-directory copy has no such precondition.
///
/// The trade-off accepted: a full-tree copy is slower than a worktree's
/// copy-on-write-ish checkout for a large repository, and does not carry
/// history/refs into the isolated copy (irrelevant here — only the working
/// tree contents are needed to build and test). `.git` and `target` are
/// excluded from the copy (see [`copy_dir_recursive`]) to reduce this cost.
///
/// # Polarity: natural convention (opposite of [`CargoFeatureProvider`])
///
/// Unlike [`CargoFeatureProvider`]'s target (`bcinr-cmca`'s
/// `hostile_mutants.rs`, whose `kill_mutant_N_*` tests are deliberately
/// authored to assert the WRONG value and thus PASS under the mutant), a
/// patch-overlay oracle test is an ordinary correctness test — it asserts
/// correct behavior and is expected to PASS against unmodified code. Applying
/// the patch and then observing that test FAIL is the natural mutation-testing
/// kill signal here: a real assertion about correct behavior stopped holding,
/// which is what "the mutant was detected" means for a directly-patched
/// production function with no specially-authored hostile test of its own.
pub struct PatchOverlayProvider;

/// Validated, defaulted view of a [`MutantSpec`]'s patch-overlay fields, held
/// only for the duration of one `activate`/`run_oracle` call.
struct PatchOverlayFields<'a> {
    repo_root: &'a str,
    target_file: &'a str,
    search_pattern: &'a str,
    replace_pattern: &'a str,
    crate_dir: &'a str,
    test_name: Option<&'a str>,
    test_binary: Option<&'a str>,
    timeout: Duration,
}

impl PatchOverlayProvider {
    fn fields(spec: &MutantSpec) -> Option<PatchOverlayFields<'_>> {
        Some(PatchOverlayFields {
            repo_root: spec.repo_root.as_deref()?,
            target_file: spec.target_file.as_deref()?,
            search_pattern: spec.search_pattern.as_deref()?,
            replace_pattern: spec.replace_pattern.as_deref()?,
            crate_dir: spec.crate_dir.as_deref().unwrap_or("."),
            test_name: spec.test_name.as_deref(),
            test_binary: spec.test_binary.as_deref(),
            timeout: Duration::from_secs(
                spec.timeout_secs.unwrap_or(DEFAULT_PATCH_OVERLAY_TIMEOUT_SECS),
            ),
        })
    }
}

impl MutationProvider for PatchOverlayProvider {
    fn capabilities(&self) -> MutationCapabilities {
        MutationCapabilities {
            requires_isolated_worktree: true,
            requires_network: false,
            supported_languages: &["rust"],
        }
    }

    fn activate(&self, spec: &MutantSpec) -> ActivationWitness {
        let Some(f) = Self::fields(spec) else {
            return ActivationWitness {
                activated: false,
                detail: "MutantSpec is missing one of repo_root/target_file/search_pattern/\
                         replace_pattern required by PatchOverlayProvider"
                    .to_string(),
            };
        };
        if !Path::new(f.repo_root).is_dir() {
            return ActivationWitness {
                activated: false,
                detail: format!("repo_root `{}` is not a directory on this machine", f.repo_root),
            };
        }
        let target_path = Path::new(f.repo_root).join(f.target_file);
        if !target_path.is_file() {
            return ActivationWitness {
                activated: false,
                detail: format!(
                    "target_file `{}` not found under repo_root `{}`",
                    f.target_file, f.repo_root
                ),
            };
        }
        ActivationWitness {
            activated: true,
            detail: format!(
                "repo_root `{}` and target_file `{}` present; exact-occurrence check of \
                 search_pattern against the isolated copy is deferred to run_oracle (it \
                 requires the copy to exist first)",
                f.repo_root, f.target_file
            ),
        }
    }

    fn run_oracle(&self, spec: &MutantSpec) -> OracleResult {
        let Some(f) = Self::fields(spec) else {
            return OracleResult {
                outcome: OracleOutcomeKind::GateFailed,
                detail: "missing patch-overlay fields at run_oracle time".to_string(),
                output_digest: None,
            };
        };

        let isolated_root = match make_isolated_dir("chicago-claims-patch-overlay") {
            Ok(p) => IsolatedDirGuard(p),
            Err(e) => {
                return OracleResult {
                    outcome: OracleOutcomeKind::GateFailed,
                    detail: format!("failed to create isolated working directory: {e}"),
                    output_digest: None,
                };
            }
        };

        if let Err(e) =
            copy_dir_recursive(Path::new(f.repo_root), isolated_root.path(), &[".git", "target"])
        {
            return OracleResult {
                outcome: OracleOutcomeKind::GateFailed,
                detail: format!(
                    "failed to copy repo_root `{}` into the isolated directory: {e}",
                    f.repo_root
                ),
                output_digest: None,
            };
        }

        let isolated_target = isolated_root.path().join(f.target_file);
        let original = match fs::read_to_string(&isolated_target) {
            Ok(s) => s,
            Err(e) => {
                return OracleResult {
                    outcome: OracleOutcomeKind::GateFailed,
                    detail: format!(
                        "failed to read isolated copy of target_file `{}`: {e}",
                        f.target_file
                    ),
                    output_digest: None,
                };
            }
        };

        let occurrences = original.matches(f.search_pattern).count();
        if occurrences != 1 {
            return OracleResult {
                outcome: OracleOutcomeKind::MutationActivationFailure,
                detail: format!(
                    "search_pattern matched {occurrences} time(s) in `{}` (expected exactly \
                     1); mutant not applied, no build/oracle attempted",
                    f.target_file
                ),
                output_digest: None,
            };
        }
        let patched = original.replacen(f.search_pattern, f.replace_pattern, 1);
        if let Err(e) = fs::write(&isolated_target, patched) {
            return OracleResult {
                outcome: OracleOutcomeKind::GateFailed,
                detail: format!("failed to write patched target_file into isolated copy: {e}"),
                output_digest: None,
            };
        }

        let mut cmd = Command::new("cargo");
        cmd.current_dir(isolated_root.path().join(f.crate_dir));
        cmd.arg("test");
        if let Some(bin) = f.test_binary {
            cmd.arg("--test").arg(bin);
        }
        if let Some(name) = f.test_name {
            cmd.arg(name);
        }
        cmd.arg("--").arg("--exact");

        let run_result = run_with_timeout(cmd, f.timeout);

        let (output, timed_out) = match run_result {
            Ok(v) => v,
            Err(e) => {
                return OracleResult {
                    outcome: OracleOutcomeKind::GateFailed,
                    detail: format!("failed to spawn isolated `cargo test` subprocess: {e}"),
                    output_digest: None,
                };
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let output_digest = Some(digest_output(&stdout, &stderr));
        let combined_tail = tail(&format!("{stdout}\n{stderr}"), 600);

        if timed_out {
            return OracleResult {
                outcome: OracleOutcomeKind::Timeout,
                detail: format!(
                    "isolated `cargo test` exceeded {:?} and was killed before finishing; \
                     output tail: {combined_tail}",
                    f.timeout
                ),
                output_digest,
            };
        }

        let passed = parse_count_after(&stdout, "passed");
        let failed = parse_count_after(&stdout, "failed");

        match (passed, failed) {
            (None, None) => OracleResult {
                outcome: OracleOutcomeKind::CompilationFailure,
                detail: format!(
                    "isolated copy produced no `test result:` line (build likely failed, \
                     exit={:?}); output tail: {combined_tail}",
                    output.status.code()
                ),
                output_digest,
            },
            (Some(0), Some(0)) => OracleResult {
                outcome: OracleOutcomeKind::OracleInfrastructureFailure,
                detail: format!(
                    "isolated `cargo test` built successfully but matched zero tests; \
                     output tail: {combined_tail}"
                ),
                output_digest,
            },
            (p, Some(failed_n)) if failed_n > 0 => OracleResult {
                outcome: OracleOutcomeKind::Killed,
                detail: format!(
                    "oracle test failed against the patched isolated copy ({failed_n} \
                     failed, {} passed) — the corruption was detected",
                    p.unwrap_or(0)
                ),
                output_digest,
            },
            (Some(passed_n), Some(0)) if passed_n > 0 => OracleResult {
                outcome: OracleOutcomeKind::Survived,
                detail: format!(
                    "oracle test passed against the patched isolated copy ({passed_n} \
                     passed) — the corruption was NOT detected"
                ),
                output_digest,
            },
            _ => OracleResult {
                outcome: OracleOutcomeKind::GateFailed,
                detail: format!(
                    "isolated `cargo test` exited without a parseable pass/fail count \
                     (status={:?}); output tail: {combined_tail}",
                    output.status.code()
                ),
                output_digest,
            },
        }
    }
}

/// Typed refusal for provider resolution: [`resolve_provider`] returns this
/// instead of ever panicking or silently guessing when a [`MutantSpec`] cannot
/// be unambiguously and safely dispatched to one [`MutationProvider`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MutantResolutionError {
    /// The spec populates none of the known provider shapes (`fixture_path`;
    /// `crate_path`+`feature`+`test_name`; `repo_root`+`target_file`+
    /// `search_pattern`+`replace_pattern`).
    #[error(
        "MutantSpec populates none of the known provider shapes (fixture_path; \
         crate_path+feature+test_name; repo_root+target_file+search_pattern+replace_pattern)"
    )]
    NoProviderShapeMatched,
    /// The spec populates more than one provider shape at once; dispatch would
    /// require an arbitrary precedence rule, so it is refused instead.
    #[error("MutantSpec populates more than one provider shape ({0:?}); dispatch is ambiguous")]
    AmbiguousProviderShapes(Vec<&'static str>),
    /// The spec matched exactly one provider shape, but that shape's own
    /// configuration is structurally unsafe or meaningless (e.g. a
    /// `target_file` that could escape the isolated copy via `..`, or a
    /// no-op `search_pattern`/`replace_pattern` pair).
    #[error("provider configuration is unsupported: {0}")]
    UnsupportedConfiguration(String),
    /// The resolved provider declares a [`MutationCapabilities`] requirement
    /// that could not be confirmed available in the current environment (e.g.
    /// an isolated working directory could not be created).
    #[error("required provider capability is unavailable in this environment: {0}")]
    CapabilityUnavailable(String),
}

/// Reject a patch-overlay [`MutantSpec`] configuration that is structurally
/// unsafe or meaningless before any filesystem work is attempted: a
/// `target_file`/`crate_dir` that could escape the isolated copy via an
/// absolute path or a `..` segment, an empty `search_pattern` (matches
/// everywhere / corrupts nothing precisely), or a `replace_pattern` identical
/// to `search_pattern` (a no-op patch is not a mutant).
fn validate_patch_overlay_configuration(spec: &MutantSpec) -> Result<(), MutantResolutionError> {
    let target_file = spec.target_file.as_deref().unwrap_or_default();
    let crate_dir = spec.crate_dir.as_deref().unwrap_or(".");
    let search = spec.search_pattern.as_deref().unwrap_or_default();
    let replace = spec.replace_pattern.as_deref().unwrap_or_default();

    for (label, value) in [("target_file", target_file), ("crate_dir", crate_dir)] {
        if Path::new(value).is_absolute() {
            return Err(MutantResolutionError::UnsupportedConfiguration(format!(
                "{label} `{value}` must be relative to repo_root, not absolute"
            )));
        }
        if Path::new(value)
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(MutantResolutionError::UnsupportedConfiguration(format!(
                "{label} `{value}` must not contain `..` path segments (would escape the \
                 isolated copy)"
            )));
        }
    }
    if search.is_empty() {
        return Err(MutantResolutionError::UnsupportedConfiguration(
            "search_pattern must not be empty".to_string(),
        ));
    }
    if search == replace {
        return Err(MutantResolutionError::UnsupportedConfiguration(
            "search_pattern and replace_pattern are identical; this patch would not change \
             any behavior"
                .to_string(),
        ));
    }
    Ok(())
}

/// Probe whether a capability a resolved provider declares it needs is
/// actually available right now, so [`resolve_provider`] can refuse with a
/// typed error instead of letting `run_oracle` discover the same problem deep
/// inside a subprocess. Only `requires_isolated_worktree` has a real probe
/// today (attempt to create and remove a throwaway directory under the system
/// temp root) — `requires_network` is declarative only, see
/// [`MutationCapabilities::requires_network`].
fn probe_capabilities(caps: MutationCapabilities) -> Result<(), MutantResolutionError> {
    if caps.requires_isolated_worktree {
        match make_isolated_dir("chicago-claims-capability-probe") {
            Ok(dir) => {
                let _ = fs::remove_dir_all(&dir);
            }
            Err(e) => {
                return Err(MutantResolutionError::CapabilityUnavailable(format!(
                    "provider requires an isolated worktree/temp directory, but one could not \
                     be created: {e}"
                )));
            }
        }
    }
    Ok(())
}

/// Resolve `spec` to the [`MutationProvider`] its populated fields
/// unambiguously indicate, generically — this is the ONE place shape
/// detection happens; no claim-specific or fixture-name-specific branching
/// exists anywhere else in this crate. Refuses (typed [`MutantResolutionError`],
/// never a panic, never a silent fallback) a spec that matches zero or more
/// than one shape, a matched patch-overlay spec whose configuration is unsafe
/// or meaningless, or a resolved provider whose declared capability cannot be
/// confirmed available in the current environment.
pub fn resolve_provider(
    spec: &MutantSpec,
) -> Result<Box<dyn MutationProvider>, MutantResolutionError> {
    let has_fixture = spec.fixture_path.is_some();
    let has_cargo_feature =
        spec.crate_path.is_some() && spec.feature.is_some() && spec.test_name.is_some();
    let has_patch_overlay = spec.repo_root.is_some()
        && spec.target_file.is_some()
        && spec.search_pattern.is_some()
        && spec.replace_pattern.is_some();

    let matched: Vec<&'static str> = [
        has_fixture.then_some("fixture"),
        has_cargo_feature.then_some("cargo-feature"),
        has_patch_overlay.then_some("patch-overlay"),
    ]
    .into_iter()
    .flatten()
    .collect();

    let shape = match matched.as_slice() {
        [] => return Err(MutantResolutionError::NoProviderShapeMatched),
        [one] => *one,
        _ => return Err(MutantResolutionError::AmbiguousProviderShapes(matched)),
    };

    let provider: Box<dyn MutationProvider> = match shape {
        "fixture" => Box::new(FixtureProvider),
        "cargo-feature" => Box::new(CargoFeatureProvider),
        "patch-overlay" => {
            validate_patch_overlay_configuration(spec)?;
            Box::new(PatchOverlayProvider)
        }
        // `matched` is built exclusively from the three `then_some` calls
        // above, so `shape` can only ever be one of those three literals —
        // this is an internal invariant of this function, not a
        // user-reachable state.
        other => unreachable!("unexpected provider shape label `{other}`"),
    };

    probe_capabilities(provider.capabilities())?;
    Ok(provider)
}

/// Resolve, activate, and run `spec`'s configured mutant, returning the full
/// [`MutantExecutionReport`] (classification + activation witness + oracle
/// result with output digest) rather than collapsing everything down to a bare
/// [`MutantClassification`] the way [`classify_mutant`] does. This is the
/// entry point a receipt-producing caller (a CLI, a report renderer) should
/// use when it wants the underlying evidence, not just the verdict.
pub fn execute_mutant(spec: &MutantSpec) -> Result<MutantExecutionReport, MutantResolutionError> {
    let provider = resolve_provider(spec)?;
    Ok(activate_run_classify(provider.as_ref(), spec))
}

/// Parse the integer immediately preceding `label` out of cargo's own
/// `test result: ok. N passed; M failed; ...` summary line. Deliberately a tiny
/// hand-written scanner (no regex dependency) over a single well-known cargo
/// output format; returns `None` if no `test result:` line, or no clause matching
/// `label`, is found.
fn parse_count_after(stdout: &str, label: &str) -> Option<u32> {
    for line in stdout.lines() {
        let Some(rest) = line.trim().strip_prefix("test result:") else {
            // Not the summary line (e.g. "running 1 test", a per-test "... ok"
            // line): skip it and keep scanning, do NOT bail out of the whole
            // function — a `?` here would incorrectly return `None` on the very
            // first non-matching line instead of continuing to the real summary.
            continue;
        };
        for clause in rest.split(';') {
            let clause = clause.trim();
            let Some(digit_start) = clause.find(|c: char| c.is_ascii_digit()) else {
                continue;
            };
            if clause[digit_start..].contains(label) {
                let digits: String =
                    clause[digit_start..].chars().take_while(char::is_ascii_digit).collect();
                if let Ok(n) = digits.parse::<u32>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Last `n` characters of `s`, char-boundary-safe (never panics on a multi-byte
/// UTF-8 boundary), for bounding how much raw subprocess output is carried into a
/// classification's `detail` string.
fn tail(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= n {
        s.to_string()
    } else {
        chars[chars.len() - n..].iter().collect()
    }
}

/// Dispatch `mutant` to the provider [`resolve_provider`] resolves it to, and
/// classify it, collapsing any typed [`MutantResolutionError`] (no shape
/// matched, an ambiguous match, an unsupported configuration, or an
/// unavailable capability) to [`MutantClassification::InfrastructureBlocked`]
/// for callers that only want a bare classification. Preserved as the
/// pre-existing, back-compatible entry point; callers that want the refusal
/// reason itself (or the richer per-mutant evidence) should call
/// [`resolve_provider`] or [`execute_mutant`] directly instead.
#[must_use]
pub fn classify_mutant(mutant: &MutantSpec) -> MutantClassification {
    match resolve_provider(mutant) {
        Ok(provider) => provider.classify(mutant),
        Err(_) => MutantClassification::InfrastructureBlocked,
    }
}

/// Activate the mutant named by `mutant.fixture_path`, run its oracle predicates
/// in-process against the real (unmodified) fixture code, and classify the
/// outcome. Preserved as a named, directly-callable compatibility entry point —
/// now a thin wrapper over `FixtureProvider.classify(..)` (see that impl for the
/// activate/run_oracle logic, which reproduces this function's pre-refactor
/// behavior exactly).
#[must_use]
pub fn activate_and_test_mutant(mutant: &MutantSpec) -> MutantClassification {
    FixtureProvider.classify(mutant)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(id: &str, fixture_file: &str) -> MutantSpec {
        MutantSpec {
            id: id.to_string(),
            description: format!("mutant fixture {fixture_file}"),
            intended_oracle_test: "oracle_union_preserves_both_operands_distinct_faults"
                .to_string(),
            fixture_path: Some(format!("fixtures/{fixture_file}")),
            crate_path: None,
            feature: None,
            test_name: None,
            test_binary: None,
            ..Default::default()
        }
    }

    fn cargo_feature_spec(
        id: &str,
        crate_path: &str,
        feature: &str,
        test_name: &str,
    ) -> MutantSpec {
        cargo_feature_spec_with_binary(id, crate_path, feature, test_name, None)
    }

    fn cargo_feature_spec_with_binary(
        id: &str,
        crate_path: &str,
        feature: &str,
        test_name: &str,
        test_binary: Option<&str>,
    ) -> MutantSpec {
        MutantSpec {
            id: id.to_string(),
            description: format!("cargo-feature mutant {feature}"),
            intended_oracle_test: test_name.to_string(),
            fixture_path: None,
            crate_path: Some(crate_path.to_string()),
            feature: Some(feature.to_string()),
            test_name: Some(test_name.to_string()),
            test_binary: test_binary.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn baseline_correct_is_not_flagged_as_a_mutant() {
        let m = spec("baseline-correct", "baseline_correct.rs");

        let classification = activate_and_test_mutant(&m);

        assert_eq!(
            classification,
            MutantClassification::Survived,
            "baseline_correct must not be reported as killed by its own intended \
             oracle: the intended oracle should PASS against correct code"
        );
    }

    #[test]
    fn all_six_named_corruptions_are_killed_by_the_intended_oracle() {
        let mutants = [
            spec("first-wins", "first_wins.rs"),
            spec("last-wins", "last_wins.rs"),
            spec("left-only", "left_only.rs"),
            spec("right-only", "right_only.rs"),
            spec("empty-set", "empty_set.rs"),
            spec("overwrite", "overwrite.rs"),
        ];

        for m in &mutants {
            let classification = activate_and_test_mutant(m);
            assert_eq!(
                classification,
                MutantClassification::KilledByIntendedOracle,
                "mutant `{}` (fixture `{:?}`) was not classified as \
                 KilledByIntendedOracle, got {classification:?}",
                m.id,
                m.fixture_path,
            );
        }
    }

    #[test]
    fn unknown_fixture_path_is_infrastructure_blocked() {
        let m = spec("nonexistent", "does_not_exist.rs");

        let classification = activate_and_test_mutant(&m);

        assert_eq!(classification, MutantClassification::InfrastructureBlocked);
    }

    #[test]
    fn empty_set_mutant_is_blind_to_the_secondary_oracle_but_caught_by_intended() {
        // Direct evidence for why intended_oracle_test must be checked, not just
        // "did any test fail": the secondary (commutativity) oracle passes against
        // empty_set (EMPTY == EMPTY regardless of argument order), yet the mutant is
        // still correctly killed by the intended oracle.
        let outcome = evaluate_fixture("empty_set").expect("empty_set fixture wired");
        assert!(
            outcome.secondary_holds,
            "secondary oracle should NOT detect the empty_set corruption"
        );
        assert!(!outcome.intended_holds, "intended oracle SHOULD detect the empty_set corruption");

        let m = spec("empty-set", "empty_set.rs");
        assert_eq!(activate_and_test_mutant(&m), MutantClassification::KilledByIntendedOracle);
    }

    /// Direct proof for the FixtureProvider refactor's own requirement: every one
    /// of the 7 pilot fixture keys (control + 6 named corruptions) classifies
    /// identically whether reached through the pre-existing
    /// `activate_and_test_mutant` name or the new `FixtureProvider.classify(..)`
    /// trait call — and both match the exact pre-refactor expected values
    /// (`Survived` for the control, `KilledByIntendedOracle` for every named
    /// corruption), not merely "equal to each other regardless of what they are".
    #[test]
    fn fixture_provider_refactor_matches_pre_refactor_behavior() {
        let cases: [(&str, &str, MutantClassification); 7] = [
            ("baseline-correct", "baseline_correct.rs", MutantClassification::Survived),
            ("first-wins", "first_wins.rs", MutantClassification::KilledByIntendedOracle),
            ("last-wins", "last_wins.rs", MutantClassification::KilledByIntendedOracle),
            ("left-only", "left_only.rs", MutantClassification::KilledByIntendedOracle),
            ("right-only", "right_only.rs", MutantClassification::KilledByIntendedOracle),
            ("empty-set", "empty_set.rs", MutantClassification::KilledByIntendedOracle),
            ("overwrite", "overwrite.rs", MutantClassification::KilledByIntendedOracle),
        ];

        for (id, fixture_file, expected) in cases {
            let m = spec(id, fixture_file);
            let via_compat_wrapper = activate_and_test_mutant(&m);
            let via_trait_directly = FixtureProvider.classify(&m);
            let via_dispatcher = classify_mutant(&m);

            assert_eq!(
                via_compat_wrapper, expected,
                "`{id}`: activate_and_test_mutant regressed from its pre-refactor value"
            );
            assert_eq!(
                via_trait_directly, expected,
                "`{id}`: FixtureProvider.classify disagrees with the expected pre-refactor value"
            );
            assert_eq!(
                via_dispatcher, expected,
                "`{id}`: classify_mutant dispatcher disagrees with the expected pre-refactor value"
            );
            assert_eq!(
                via_compat_wrapper, via_trait_directly,
                "`{id}`: compatibility wrapper and direct trait call disagree — refactor is \
                 not behavior-preserving"
            );
        }
    }

    #[test]
    fn classify_mutant_reports_infrastructure_blocked_when_no_provider_shape_populated() {
        let m = MutantSpec {
            id: "empty-spec".to_string(),
            description: "neither fixture nor cargo-feature fields set".to_string(),
            intended_oracle_test: "n/a".to_string(),
            fixture_path: None,
            crate_path: None,
            feature: None,
            test_name: None,
            test_binary: None,
            ..Default::default()
        };

        assert_eq!(classify_mutant(&m), MutantClassification::InfrastructureBlocked);
        // resolve_provider returns Box<dyn MutationProvider> on success, which is
        // not PartialEq — match on the Err variant directly instead of
        // assert_eq!-ing the whole Result.
        match resolve_provider(&m) {
            Err(MutantResolutionError::NoProviderShapeMatched) => {}
            Err(other) => panic!("expected NoProviderShapeMatched, got Err({other:?})"),
            Ok(_) => panic!("expected NoProviderShapeMatched, got Ok(_)"),
        }
    }

    #[test]
    fn classify_mutant_reports_infrastructure_blocked_when_both_provider_shapes_populated() {
        // Ambiguous: a spec that could dispatch to either provider must not be
        // silently resolved by an arbitrary precedence rule.
        let m = MutantSpec {
            id: "ambiguous-spec".to_string(),
            description: "both fixture_path and the cargo-feature triple set".to_string(),
            intended_oracle_test: "n/a".to_string(),
            fixture_path: Some("fixtures/first_wins.rs".to_string()),
            crate_path: Some("/does/not/matter".to_string()),
            feature: Some("mutant_1".to_string()),
            test_name: Some("some_test".to_string()),
            test_binary: None,
            ..Default::default()
        };

        assert_eq!(classify_mutant(&m), MutantClassification::InfrastructureBlocked);
        match resolve_provider(&m) {
            Err(MutantResolutionError::AmbiguousProviderShapes(shapes)) => {
                assert_eq!(shapes, vec!["fixture", "cargo-feature"]);
            }
            Err(other) => panic!("expected AmbiguousProviderShapes, got Err({other:?})"),
            Ok(_) => panic!("expected AmbiguousProviderShapes, got Ok(_)"),
        }
    }

    #[test]
    fn cargo_feature_provider_activate_fails_fast_on_missing_fields() {
        let m = cargo_feature_spec("partial", "/Users/sac/bcinr/crates/bcinr-cmca", "mutant_1", "");
        let m = MutantSpec { test_name: None, ..m };

        let witness = CargoFeatureProvider.activate(&m);

        assert!(!witness.activated);
        assert!(witness.detail.contains("missing"));
    }

    #[test]
    fn cargo_feature_provider_activate_fails_when_crate_path_does_not_exist() {
        let m = cargo_feature_spec(
            "nonexistent-crate",
            "/nonexistent/chicago-claims/no-such-crate",
            "mutant_1",
            "kill_mutant_1_single_measure_collapse",
        );

        let witness = CargoFeatureProvider.activate(&m);

        assert!(!witness.activated);
        assert!(witness.detail.contains("Cargo.toml"));
    }

    #[test]
    fn cargo_feature_provider_infrastructure_blocked_end_to_end_on_missing_crate() {
        let m = cargo_feature_spec(
            "nonexistent-crate",
            "/nonexistent/chicago-claims/no-such-crate",
            "mutant_1",
            "kill_mutant_1_single_measure_collapse",
        );

        assert_eq!(CargoFeatureProvider.classify(&m), MutantClassification::InfrastructureBlocked);
        assert_eq!(classify_mutant(&m), MutantClassification::InfrastructureBlocked);
    }

    #[test]
    fn parse_count_after_reads_real_cargo_test_result_lines() {
        let one_passed = "running 1 test\ntest kill_mutant_1_single_measure_collapse ... ok\n\n\
             test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; \
             finished in 0.00s\n";
        assert_eq!(parse_count_after(one_passed, "passed"), Some(1));
        assert_eq!(parse_count_after(one_passed, "failed"), Some(0));

        let one_failed = "running 1 test\ntest some_test ... FAILED\n\n\
             test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; \
             finished in 0.00s\n";
        assert_eq!(parse_count_after(one_failed, "passed"), Some(0));
        assert_eq!(parse_count_after(one_failed, "failed"), Some(1));

        let zero_matched = "running 0 tests\n\n\
             test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; \
             finished in 0.00s\n";
        assert_eq!(parse_count_after(zero_matched, "passed"), Some(0));
        assert_eq!(parse_count_after(zero_matched, "failed"), Some(0));

        let no_result_line = "error[E0432]: unresolved import\n";
        assert_eq!(parse_count_after(no_result_line, "passed"), None);
    }

    #[test]
    fn tail_is_char_boundary_safe_and_bounds_length() {
        let short = "hello";
        assert_eq!(tail(short, 100), "hello");

        let long: String = "a".repeat(1000);
        assert_eq!(tail(&long, 10).len(), 10);

        // Multi-byte UTF-8 near the truncation boundary must not panic.
        let multibyte = "日本語テキストは複数バイト文字を含みます".repeat(5);
        let truncated = tail(&multibyte, 7);
        assert_eq!(truncated.chars().count(), 7);
    }

    /// Genuine end-to-end proof that [`CargoFeatureProvider`] drives a REAL
    /// external `cargo test` subprocess and classifies from its REAL parsed
    /// pass/fail — not a fixture, not a simulation. Uses `bcinr-cmca`'s real
    /// `mutant_1` cfg-feature mutant and its real dedicated oracle test
    /// `kill_mutant_1_single_measure_collapse` (see the sibling checkout's
    /// `MUTANT_KILL_MATRIX.md`: `mutant_1` is `KILLED_BY_INTENDED_ORACLE` with no
    /// collateral damage, the cleanest of the 11 for an isolated demonstration).
    ///
    /// This mutant is intentionally NOT wired into either
    /// `cmca-observatory-proposal-only.toml` or `cmca-rejection-invariance.toml`
    /// — neither of those two claims' own laws is what `mutant_1` corrupts (see
    /// those TOML files' own comments for the honest non-applicability finding).
    /// This test instead proves the provider mechanism itself is real and working
    /// against a real, unrelated, genuinely-applicable mutant, independent of
    /// which specific claims it ends up wired into.
    ///
    /// `#[ignore]`d by default (same pattern as `scan::tests::
    /// real_bcinr_fixed_rs_has_numeric_fault_set_correctly_structured`): requires
    /// the `bcinr` checkout on disk and spawns a real, several-second `cargo test`
    /// subprocess. Run explicitly with `cargo test -p chicago-claims -- --ignored`.
    #[test]
    #[ignore = "requires the bcinr checkout on disk and spawns a real cargo test subprocess"]
    fn cargo_feature_provider_kills_real_bcinr_cmca_mutant_1_via_real_subprocess() {
        const BCINR_CMCA: &str = "/Users/sac/bcinr/crates/bcinr-cmca";
        assert!(
            Path::new(BCINR_CMCA).join("Cargo.toml").is_file(),
            "bcinr checkout not found at {BCINR_CMCA}; this grounding test requires it"
        );

        let m = cargo_feature_spec_with_binary(
            "mutant-1-demo",
            BCINR_CMCA,
            "mutant_1",
            "kill_mutant_1_single_measure_collapse",
            Some("hostile_mutants"),
        );

        let witness = CargoFeatureProvider.activate(&m);
        assert!(witness.activated, "activation failed: {}", witness.detail);

        let oracle = CargoFeatureProvider.run_oracle(&m);
        assert_eq!(
            oracle.outcome,
            OracleOutcomeKind::Killed,
            "expected mutant_1's dedicated oracle to pass (=> Killed) under the real \
             subprocess; got {:?} ({})",
            oracle.outcome,
            oracle.detail
        );

        assert_eq!(CargoFeatureProvider.classify(&m), MutantClassification::KilledByIntendedOracle);
        assert_eq!(classify_mutant(&m), MutantClassification::KilledByIntendedOracle);
    }

    // ---------------------------------------------------------------------
    // MutationCapabilities: typed, provider-specific, not one shared default.
    // ---------------------------------------------------------------------

    #[test]
    fn each_provider_declares_its_own_distinct_capabilities() {
        let fixture = FixtureProvider.capabilities();
        let cargo_feature = CargoFeatureProvider.capabilities();
        let patch_overlay = PatchOverlayProvider.capabilities();

        assert!(!fixture.requires_isolated_worktree);
        assert!(!cargo_feature.requires_isolated_worktree);
        assert!(
            patch_overlay.requires_isolated_worktree,
            "PatchOverlayProvider is the one provider that must operate on an \
             isolated copy, not the real working tree"
        );

        assert_eq!(fixture.supported_languages, &["rust"]);
        assert_eq!(cargo_feature.supported_languages, &["rust"]);
        assert_eq!(patch_overlay.supported_languages, &["rust"]);
    }

    // ---------------------------------------------------------------------
    // resolve_provider: typed refusals for unsupported patch-overlay configs.
    // ---------------------------------------------------------------------

    fn patch_overlay_spec(
        repo_root: &Path,
        target_file: &str,
        search: &str,
        replace: &str,
        timeout_secs: Option<u64>,
    ) -> MutantSpec {
        MutantSpec {
            id: "patch-overlay-smoke".to_string(),
            description: "PatchOverlayProvider smoke test".to_string(),
            // The oracle test lives in `mod tests`, so its exact `--exact`-matched
            // path is `tests::add_is_correct`, not the bare function name — the
            // same full-path requirement CargoFeatureProvider's own `test_binary`
            // doc comment already documents as a real failure mode.
            intended_oracle_test: "tests::add_is_correct".to_string(),
            repo_root: Some(repo_root.display().to_string()),
            target_file: Some(target_file.to_string()),
            search_pattern: Some(search.to_string()),
            replace_pattern: Some(replace.to_string()),
            test_name: Some("tests::add_is_correct".to_string()),
            timeout_secs,
            ..Default::default()
        }
    }

    #[test]
    fn resolve_provider_refuses_absolute_target_file() {
        let m = patch_overlay_spec(Path::new("/tmp/whatever"), "/etc/passwd", "a", "b", None);

        match resolve_provider(&m) {
            Err(MutantResolutionError::UnsupportedConfiguration(msg)) => {
                assert!(msg.contains("target_file"));
                assert!(msg.contains("absolute"));
            }
            Err(other) => panic!("expected UnsupportedConfiguration, got Err({other:?})"),
            Ok(_) => panic!("expected UnsupportedConfiguration, got Ok(_)"),
        }
    }

    #[test]
    fn resolve_provider_refuses_parent_dir_traversal_in_target_file() {
        let m = patch_overlay_spec(Path::new("/tmp/whatever"), "../../etc/passwd", "a", "b", None);

        match resolve_provider(&m) {
            Err(MutantResolutionError::UnsupportedConfiguration(msg)) => {
                assert!(msg.contains("target_file"));
                assert!(msg.contains(".."));
            }
            Err(other) => panic!("expected UnsupportedConfiguration, got Err({other:?})"),
            Ok(_) => panic!("expected UnsupportedConfiguration, got Ok(_)"),
        }
    }

    #[test]
    fn resolve_provider_refuses_empty_search_pattern() {
        let m = patch_overlay_spec(Path::new("/tmp/whatever"), "src/lib.rs", "", "b", None);

        match resolve_provider(&m) {
            Err(MutantResolutionError::UnsupportedConfiguration(msg)) => {
                assert!(msg.contains("search_pattern"));
            }
            Err(other) => panic!("expected UnsupportedConfiguration, got Err({other:?})"),
            Ok(_) => panic!("expected UnsupportedConfiguration, got Ok(_)"),
        }
    }

    #[test]
    fn resolve_provider_refuses_no_op_patch() {
        let m = patch_overlay_spec(
            Path::new("/tmp/whatever"),
            "src/lib.rs",
            "identical",
            "identical",
            None,
        );

        match resolve_provider(&m) {
            Err(MutantResolutionError::UnsupportedConfiguration(msg)) => {
                assert!(msg.contains("identical"));
            }
            Err(other) => panic!("expected UnsupportedConfiguration, got Err({other:?})"),
            Ok(_) => panic!("expected UnsupportedConfiguration, got Ok(_)"),
        }
    }

    // ---------------------------------------------------------------------
    // PatchOverlayProvider smoke tests: REAL isolated-copy + patch + build +
    // run + cleanup, against a tiny throwaway target crate materialized fresh
    // for each test — never bcinr-cmca or any other production target (per
    // this track's explicit instruction to prove the mechanism on a safe,
    // disposable target before Track B points it at real production code).
    // ---------------------------------------------------------------------

    /// Materialize a tiny, self-contained, zero-dependency Rust crate into a
    /// fresh temp directory: `pub fn add(a, b) -> i32 { a + b }`, an unused
    /// marker constant (a corruption target with no oracle coverage, for
    /// proving `Survived`), and one oracle test asserting `add`'s correct
    /// behavior. An empty `[workspace]` table marks it as its own workspace
    /// root so it is fully self-contained wherever it is copied.
    fn materialize_smoke_target_crate() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("create smoke target temp dir");
        fs::create_dir_all(dir.path().join("src")).expect("create src dir");
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\n\
             name = \"chicago-claims-patch-overlay-smoke-target\"\n\
             version = \"0.1.0\"\n\
             edition = \"2021\"\n\
             \n\
             [workspace]\n",
        )
        .expect("write Cargo.toml");
        fs::write(
            dir.path().join("src").join("lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 {\n\
             \x20\x20\x20\x20a + b\n\
             }\n\
             \n\
             pub const UNUSED_MARKER: i32 = 1;\n\
             \n\
             #[cfg(test)]\n\
             mod tests {\n\
             \x20\x20\x20\x20use super::*;\n\
             \n\
             \x20\x20\x20\x20#[test]\n\
             \x20\x20\x20\x20fn add_is_correct() {\n\
             \x20\x20\x20\x20\x20\x20\x20\x20assert_eq!(add(2, 3), 5);\n\
             \x20\x20\x20\x20}\n\
             }\n",
        )
        .expect("write src/lib.rs");
        dir
    }

    #[test]
    fn patch_overlay_provider_smoke_kills_a_real_corrupted_arithmetic_mutant() {
        let target = materialize_smoke_target_crate();
        let m = patch_overlay_spec(target.path(), "src/lib.rs", "a + b", "a - b", None);

        let report = execute_mutant(&m).expect("valid patch-overlay spec should resolve");

        let oracle = report.oracle.clone();
        assert_eq!(
            report.classification,
            MutantClassification::KilledByIntendedOracle,
            "activation={:?} oracle={oracle:?}",
            report.activation
        );
        assert!(
            oracle.expect("oracle should have run").output_digest.is_some(),
            "a real subprocess ran; a digest must be captured"
        );
    }

    #[test]
    fn patch_overlay_provider_smoke_survives_an_undetected_corruption() {
        let target = materialize_smoke_target_crate();
        let m = patch_overlay_spec(
            target.path(),
            "src/lib.rs",
            "UNUSED_MARKER: i32 = 1",
            "UNUSED_MARKER: i32 = 2",
            None,
        );

        let report = execute_mutant(&m).expect("valid patch-overlay spec should resolve");

        assert_eq!(
            report.classification,
            MutantClassification::Survived,
            "activation={:?} oracle={:?}",
            report.activation,
            report.oracle
        );
    }

    #[test]
    fn patch_overlay_provider_smoke_reports_activation_failure_for_absent_pattern() {
        let target = materialize_smoke_target_crate();
        let m = patch_overlay_spec(
            target.path(),
            "src/lib.rs",
            "totally_bogus_pattern_xyz_not_present",
            "irrelevant",
            None,
        );

        let report = execute_mutant(&m).expect("valid patch-overlay spec should resolve");

        assert_eq!(
            report.classification,
            MutantClassification::MutationActivationFailed,
            "activation={:?} oracle={:?}",
            report.activation,
            report.oracle
        );
        assert_eq!(
            report
                .oracle
                .expect("oracle branch should still run to report the mismatch")
                .outcome,
            OracleOutcomeKind::MutationActivationFailure
        );
    }

    #[test]
    fn patch_overlay_provider_smoke_reports_compilation_failure_for_a_syntax_breaking_patch() {
        let target = materialize_smoke_target_crate();
        let m = patch_overlay_spec(target.path(), "src/lib.rs", "a + b", "a +", None);

        let report = execute_mutant(&m).expect("valid patch-overlay spec should resolve");

        assert_eq!(
            report.classification,
            MutantClassification::CompilationFailed,
            "activation={:?} oracle={:?}",
            report.activation,
            report.oracle
        );
    }

    #[test]
    fn patch_overlay_provider_smoke_reports_timeout_for_a_hanging_mutant() {
        let target = materialize_smoke_target_crate();
        let m = patch_overlay_spec(target.path(), "src/lib.rs", "a + b", "loop {}", Some(3));

        let report = execute_mutant(&m).expect("valid patch-overlay spec should resolve");

        assert_eq!(
            report.classification,
            MutantClassification::Timeout,
            "activation={:?} oracle={:?}",
            report.activation,
            report.oracle
        );
    }

    #[test]
    fn patch_overlay_provider_smoke_reports_infrastructure_blocked_for_missing_repo_root() {
        let m = patch_overlay_spec(
            Path::new("/nonexistent/chicago-claims/no-such-repo"),
            "src/lib.rs",
            "a + b",
            "a - b",
            None,
        );

        let report = execute_mutant(&m).expect("valid shape should resolve to a provider");

        assert_eq!(report.classification, MutantClassification::InfrastructureBlocked);
        assert!(!report.activation.activated);
        assert!(report.oracle.is_none(), "oracle must not run when activation failed");
    }

    #[test]
    fn patch_overlay_provider_isolated_copy_never_touches_the_real_repo_root() {
        // The real target crate's own file must be byte-identical after the
        // provider runs — proving the patch landed only in the isolated copy,
        // never in repo_root itself.
        let target = materialize_smoke_target_crate();
        let original_contents =
            fs::read_to_string(target.path().join("src/lib.rs")).expect("read original");

        let m = patch_overlay_spec(target.path(), "src/lib.rs", "a + b", "a - b", None);
        let _ = execute_mutant(&m).expect("valid patch-overlay spec should resolve");

        let contents_after =
            fs::read_to_string(target.path().join("src/lib.rs")).expect("read after run");
        assert_eq!(
            original_contents, contents_after,
            "PatchOverlayProvider must never write into the real repo_root working tree"
        );
    }
}
