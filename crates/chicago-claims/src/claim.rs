//! TOML claim schema: minimal `Claim` structure describing a required Rust structure,
//! its scope (file/symbol), its named mutant variants, and the evidence required to
//! close the claim.

use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

/// A single reconciliation claim: "this file/symbol must have this structure, and
/// these mutant variants must be killed by these oracle tests."
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Claim {
    /// Stable identifier for the claim (e.g. `"cmca-numeric-fault-join-semilattice"`).
    pub id: String,
    /// Where the required structure is expected to live.
    pub scope: Scope,
    /// The structural shape the claim asserts exists.
    pub required: RequiredStructure,
    /// Named mutant variants to activate against the scoped code.
    pub mutants: Vec<MutantSpec>,
    /// Evidence artifact names required before the claim can be marked closed.
    pub evidence_required: Vec<String>,
}

/// Where a claim's required structure is expected to be found.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Scope {
    /// Path to the source file, relative or absolute, as written in the claim.
    pub file: String,
    /// Optional symbol name within the file (e.g. a type or function name).
    pub symbol: Option<String>,
}

/// The structural shape a claim requires: a type with a named field of given
/// visibility, plus a set of required method names.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RequiredStructure {
    /// Name of the required struct/enum type.
    pub type_name: String,
    /// Name of the required field (for a tuple struct, this is a label used only in
    /// the claim; the scanner matches the struct's single tuple field positionally).
    pub field_name: String,
    /// Whether that field must be private (`syn::Visibility::Inherited`).
    pub field_must_be_private: bool,
    /// Method names required to exist in an `impl` block for the type.
    pub required_methods: Vec<String>,
    /// Names whose CONSTRUCTION (a call `Name::something(...)` or a struct literal
    /// `Name { .. }`) must be structurally ABSENT anywhere in the scanned file. This
    /// is a call-site-absence check, distinct in kind from `required_methods`
    /// (presence of a method definition) — see `scan::scan_required_structure`'s doc
    /// comment for why it needed new scanner logic rather than reuse of the existing
    /// field/method checks. Optional; defaults to empty (no such check performed).
    #[serde(default)]
    pub forbidden_constructions: Vec<String>,
}

/// A single named mutant variant to activate against the claim's scope.
///
/// Exactly one of three shapes must be populated, dispatched by
/// `mutate::classify_mutant` / `mutate::resolve_provider` to the matching
/// `mutate::MutationProvider`:
///
/// - **Fixture shape** (`fixture_path` set; all other shape-selecting fields
///   left unset): the original MLP mechanism — a standalone Rust file under
///   `tests/fixtures/*.rs`, evaluated in-process via `include!` by
///   `mutate::FixtureProvider`.
/// - **Cargo-feature shape** (`crate_path`, `feature`, and `test_name` all set;
///   `fixture_path` and the patch-overlay fields left unset): drives a real
///   `cargo test -p <crate> --features <feature> <test_name> -- --exact`
///   subprocess against an external crate on disk, via
///   `mutate::CargoFeatureProvider`.
/// - **Patch-overlay shape** (`repo_root`, `target_file`, `search_pattern`, and
///   `replace_pattern` all set; `fixture_path` left unset): copies `repo_root`
///   into an isolated temp directory, applies a literal search/replace patch to
///   `target_file` inside that copy ONLY, then runs `cargo test` against the
///   isolated copy and classifies from the real parsed pass/fail, via
///   `mutate::PatchOverlayProvider`. This is the general mechanism for attacking
///   a real production implementation directly (no pre-existing cfg-feature
///   mutant or fixture required). `crate_path`/`feature` are NOT used by this
///   shape (no cargo feature is toggled — the source is patched directly);
///   `test_name`/`test_binary` are reused from the cargo-feature shape's fields
///   with the same meaning (exact test name / `--test` binary scope).
///
/// All three shapes are plain optional fields (matching the existing
/// `RequiredStructure::forbidden_constructions` `#[serde(default)]` precedent)
/// rather than a tagged enum, so existing TOML claims using the bare
/// `fixture_path = "..."` shape continue to parse unchanged. A spec matching
/// none of the three shapes, or more than one simultaneously, is not rejected at
/// parse time (this schema does not encode the "exactly one" constraint
/// structurally) — it is instead reported as a typed
/// `mutate::MutantResolutionError` (`NoProviderShapeMatched` /
/// `AmbiguousProviderShapes`) by `mutate::resolve_provider`, which
/// `mutate::classify_mutant` collapses to `MutantClassification::
/// InfrastructureBlocked` for backward compatibility with callers that only
/// want a classification.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct MutantSpec {
    /// Mutant identifier (e.g. `"first-wins"`, `"last-wins"`, `"left-only"`,
    /// `"right-only"`, `"empty-set"`, `"overwrite"`).
    pub id: String,
    /// Human-readable description of what the mutant corrupts.
    pub description: String,
    /// Name of the oracle test intended to kill this mutant. Purely informational
    /// metadata for the fixture shape (dispatch is by `fixture_path` alone); for
    /// the cargo-feature shape this typically duplicates `test_name`, which is
    /// the field actually used to drive the subprocess.
    pub intended_oracle_test: String,
    /// Fixture shape: path to a standalone Rust file implementing the corrupted
    /// variant. `#[serde(default)]` so cargo-feature-shaped entries may omit it.
    #[serde(default)]
    pub fixture_path: Option<String>,
    /// Cargo-feature shape: filesystem path to the external crate directory to
    /// run `cargo test` against (a directory containing that crate's own
    /// `Cargo.toml`), e.g. `"/Users/sac/bcinr/crates/bcinr-cmca"`.
    #[serde(default)]
    pub crate_path: Option<String>,
    /// Cargo-feature shape: cargo feature name to enable for the mutant, e.g.
    /// `"mutant_1"`.
    #[serde(default)]
    pub feature: Option<String>,
    /// Cargo-feature shape: exact test name to run with `-- --exact`, e.g.
    /// `"kill_mutant_1_single_measure_collapse"`.
    #[serde(default)]
    pub test_name: Option<String>,
    /// Cargo-feature shape, optional: the integration-test binary (as named by a
    /// `[[test]]` target, or a file under `tests/*.rs`) to scope the run to via
    /// `--test <name>`, e.g. `"hostile_mutants"`. Strongly recommended whenever
    /// the target crate has more than one test binary: omitting it makes `cargo
    /// test <test_name> -- --exact` apply the name filter across EVERY test
    /// binary in the crate, which is slow and can produce a misleading
    /// "0 tests matched" result from an unrelated binary that happens to report
    /// its summary line first — a real failure mode this provider's own
    /// hostile-subprocess test caught (see `mutate::tests::
    /// cargo_feature_provider_kills_real_bcinr_cmca_mutant_1_via_real_subprocess`'s
    /// history). `bcinr-cmca`'s own documented working command
    /// (`MUTANT_KILL_MATRIX.md`) always scopes with `--test hostile_mutants`.
    #[serde(default)]
    pub test_binary: Option<String>,
    /// Patch-overlay shape: filesystem path to the repository root to copy into
    /// an isolated temp directory before patching, e.g. `"/Users/sac/bcinr"`.
    /// May be relative (resolved against the current process's working
    /// directory) or absolute; real usage is expected to be absolute, matching
    /// `crate_path`'s convention.
    #[serde(default)]
    pub repo_root: Option<String>,
    /// Patch-overlay shape: path to the file to patch, relative to `repo_root`,
    /// e.g. `"crates/bcinr-cmca/src/fixed.rs"`. Must not be absolute and must
    /// not contain `..` path segments — `mutate::resolve_provider` refuses such
    /// a spec with a typed `MutantResolutionError::UnsupportedConfiguration`
    /// rather than attempting a patch that could escape the isolated copy.
    #[serde(default)]
    pub target_file: Option<String>,
    /// Patch-overlay shape: the exact literal text of the one production line
    /// (or expression) to corrupt. Must appear in `target_file` (inside the
    /// isolated copy) EXACTLY ONCE; zero or multiple occurrences is reported as
    /// `MutantClassification::MutationActivationFailed` rather than corrupting
    /// an ambiguous or wrong location.
    #[serde(default)]
    pub search_pattern: Option<String>,
    /// Patch-overlay shape: the literal replacement text substituted for the
    /// single `search_pattern` occurrence. Must differ from `search_pattern` —
    /// an identical value is refused at resolution time (a no-op patch is not a
    /// mutant).
    #[serde(default)]
    pub replace_pattern: Option<String>,
    /// Patch-overlay shape, optional: directory containing the crate's own
    /// `Cargo.toml` to run `cargo test` from, relative to `repo_root` (e.g.
    /// `"crates/bcinr-cmca"` when `repo_root` is a whole workspace checkout).
    /// Defaults to `"."` (repo_root itself is the crate root) when unset. Must
    /// not be absolute and must not contain `..` path segments, same
    /// constraint and same typed refusal as `target_file`.
    #[serde(default)]
    pub crate_dir: Option<String>,
    /// Patch-overlay shape, optional: wall-clock budget in seconds for the
    /// isolated `cargo test` subprocess before it is killed and classified as
    /// `MutantClassification::Timeout`. Defaults to a provider-level constant
    /// (`mutate::DEFAULT_PATCH_OVERLAY_TIMEOUT_SECS`) when unset.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Typed errors for claim loading. Never panics on malformed input.
#[derive(Debug, Error)]
pub enum ClaimError {
    /// The claim file could not be read from disk.
    #[error("failed to read claim file {path}: {source}")]
    Io {
        /// Path that failed to read.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The claim file's contents did not parse as valid TOML matching the schema.
    #[error("failed to parse claim TOML at {path}: {source}")]
    Parse {
        /// Path that failed to parse.
        path: String,
        /// Underlying TOML deserialization error.
        #[source]
        source: toml::de::Error,
    },
}

impl Claim {
    /// Load and parse a [`Claim`] from a TOML file at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`ClaimError::Io`] if the file cannot be read, or
    /// [`ClaimError::Parse`] if the contents do not deserialize into a valid `Claim`.
    /// Never panics on malformed input.
    pub fn load_from_toml(path: &Path) -> Result<Self, ClaimError> {
        let text = fs::read_to_string(path)
            .map_err(|source| ClaimError::Io { path: path.display().to_string(), source })?;
        toml::from_str(&text)
            .map_err(|source| ClaimError::Parse { path: path.display().to_string(), source })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_toml(contents: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().expect("create temp file");
        f.write_all(contents.as_bytes()).expect("write temp file");
        f
    }

    #[test]
    fn parses_valid_minimal_claim() {
        let toml_text = r#"
            id = "cmca-numeric-fault-join-semilattice"
            evidence_required = ["scan_result.json", "mutant_ledger.json"]

            [scope]
            file = "crates/bcinr-cmca/src/fixed.rs"
            symbol = "NumericFaultSet"

            [required]
            type_name = "NumericFaultSet"
            field_name = "0"
            field_must_be_private = true
            required_methods = ["union", "is_empty", "bits"]

            [[mutants]]
            id = "first-wins"
            description = "union collapses to keeping only the first operand's faults"
            intended_oracle_test = "test_union_is_not_first_wins"
            fixture_path = "fixtures/first_wins.rs"

            [[mutants]]
            id = "last-wins"
            description = "union collapses to keeping only the second operand's faults"
            intended_oracle_test = "test_union_is_not_last_wins"
            fixture_path = "fixtures/last_wins.rs"
        "#;
        let file = write_temp_toml(toml_text);

        let claim = Claim::load_from_toml(file.path()).expect("valid claim should parse");

        assert_eq!(claim.id, "cmca-numeric-fault-join-semilattice");
        assert_eq!(claim.scope.file, "crates/bcinr-cmca/src/fixed.rs");
        assert_eq!(claim.scope.symbol.as_deref(), Some("NumericFaultSet"));
        assert_eq!(claim.required.type_name, "NumericFaultSet");
        assert!(claim.required.field_must_be_private);
        assert_eq!(claim.required.required_methods.len(), 3);
        assert_eq!(claim.mutants.len(), 2);
        assert_eq!(claim.mutants[0].id, "first-wins");
        assert_eq!(claim.evidence_required.len(), 2);
    }

    #[test]
    fn malformed_toml_yields_typed_error_not_panic() {
        // Missing required fields and invalid TOML syntax (unterminated table).
        let toml_text = r#"
            id = "broken-claim"
            [scope
            file = "somewhere.rs"
        "#;
        let file = write_temp_toml(toml_text);

        let result = Claim::load_from_toml(file.path());

        match result {
            Err(ClaimError::Parse { path, .. }) => {
                assert!(path.contains(
                    file.path().file_name().and_then(|n| n.to_str()).unwrap_or_default()
                ));
            }
            other => panic!("expected ClaimError::Parse, got {other:?}"),
        }
    }

    #[test]
    fn missing_file_yields_typed_io_error_not_panic() {
        let missing = Path::new("/nonexistent/chicago-claims/does-not-exist.toml");

        let result = Claim::load_from_toml(missing);

        match result {
            Err(ClaimError::Io { .. }) => {}
            other => panic!("expected ClaimError::Io, got {other:?}"),
        }
    }

    /// Config-driven proof for the third (patch-overlay) shape: a claim TOML
    /// naming a target file, search pattern, and replacement pattern parses into
    /// exactly the fields `mutate::PatchOverlayProvider` reads — no source-code
    /// match statement or hardcoded fixture name is required to describe this
    /// mutant, only TOML data.
    #[test]
    fn parses_patch_overlay_shaped_mutant_from_toml() {
        let toml_text = r#"
            id = "cmca-numeric-fault-union-direct-patch"
            evidence_required = ["scan_result.json"]

            [scope]
            file = "crates/bcinr-cmca/src/fixed.rs"
            symbol = "NumericFaultSet"

            [required]
            type_name = "NumericFaultSet"
            field_name = "0"
            field_must_be_private = true
            required_methods = ["union"]

            [[mutants]]
            id = "union-overwrite-direct"
            description = "union overwrites rather than bitwise-ORs (direct production patch)"
            intended_oracle_test = "union_accumulates_both_operands_distinct_faults"
            repo_root = "/Users/sac/bcinr"
            target_file = "crates/bcinr-cmca/src/fixed.rs"
            search_pattern = "Self(self.0 | other.0)"
            replace_pattern = "Self(other.0)"
            crate_dir = "crates/bcinr-cmca"
            test_name = "union_accumulates_both_operands_distinct_faults"
            test_binary = "fixed"
            timeout_secs = 120
        "#;
        let file = write_temp_toml(toml_text);

        let claim = Claim::load_from_toml(file.path()).expect("valid claim should parse");

        assert_eq!(claim.mutants.len(), 1);
        let m = &claim.mutants[0];
        assert_eq!(m.repo_root.as_deref(), Some("/Users/sac/bcinr"));
        assert_eq!(m.target_file.as_deref(), Some("crates/bcinr-cmca/src/fixed.rs"));
        assert_eq!(m.search_pattern.as_deref(), Some("Self(self.0 | other.0)"));
        assert_eq!(m.replace_pattern.as_deref(), Some("Self(other.0)"));
        assert_eq!(m.crate_dir.as_deref(), Some("crates/bcinr-cmca"));
        assert_eq!(m.timeout_secs, Some(120));
        // Cargo-feature-shape fields must be absent for this shape.
        assert_eq!(m.fixture_path, None);
        assert_eq!(m.feature, None);
    }
}
