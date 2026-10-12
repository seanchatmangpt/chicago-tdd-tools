#![cfg(feature = "nato")]
//! Key custody for the ledger signing subsystem (plan P14).
//!
//! **Law**: this is the ONLY module under `ledger::signing` permitted to
//! perform filesystem I/O. Every other signing module (sealer, law, dossier
//! plumbing) operates on in-memory bytes exclusively and receives its key
//! material from [`KeyStore`]. Courts should refuse any sibling module that
//! opens a file.
//!
//! Custody model (generate-on-first-use):
//!
//! 1. Seed file `ml_dsa_65.seed` under the key dir (default
//!    `.ggen/keys/ledger/` at the process working directory — the workspace
//!    root in normal use; override with [`KeyStore::with_dir`] for tests,
//!    which MUST use a tempdir and never the real dir).
//! 2. Seed present and exactly 32 bytes => deterministic reload via
//!    [`SigningKey::from_seed`] (same key id every time).
//! 3. Seed absent => fresh entropy via OS RNG, then atomically persisted
//!    (temp file + rename, 0600 file / 0700 dir on unix; best-effort
//!    permissions elsewhere — see `restrict_file_perms`/`restrict_dir_perms`).
//! 4. Seed present but wrong length or unreadable => [`SealError::KeyCustody`]
//!    refusal. NEVER silently regenerate: a regenerated key would mint seals
//!    the old public key cannot verify, which is indistinguishable from
//!    tampering.
//!
//! **Cargo requirement** (owned by the manifest lane): `ml-dsa = "0.1"`
//! (default features — `from_seed` is core) and `getrandom = "0.3"` must be
//! optional deps enabled by the `nato` feature. `ml-dsa`'s own `rand_core`/
//! `getrandom` features are NOT used: against `crypto-common 0.2.x` they
//! reference features that no longer exist, so entropy is taken directly via
//! `getrandom::fill`.
//!
//! Rotation ([`KeyStore::rotate`]) atomically rewrites the seed with a fresh
//! entropy key. Seals minted under the previous key remain verifiable: the
//! seal record carries the key id and verifier, and old public keys can be
//! resolved from prior ledger records.

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

use std::fs;
use std::path::{Path, PathBuf};

use ml_dsa::{Keypair as _, MlDsa65, Seed, SigningKey};
use sha2::{Digest, Sha256};

use crate::ledger::error::SealError;

/// Default key directory, relative to the process working directory (the
/// workspace root in normal use).
pub const DEFAULT_KEY_DIR: &str = ".ggen/keys/ledger";

/// Seed file name inside the key directory.
pub const SEED_FILE: &str = "ml_dsa_65.seed";

/// ML-DSA-65 seed length in bytes (FIPS 204: 32-byte seed xi).
pub const SEED_LEN: usize = 32;

/// Generate-on-first-use key custody for the ledger signer.
#[derive(Debug, Clone)]
pub struct KeyStore {
    dir: PathBuf,
}

impl KeyStore {
    /// Store at the default location (`.ggen/keys/ledger` under the current
    /// process working directory). NEVER use this constructor in tests — use
    /// [`KeyStore::with_dir`] with a tempdir.
    #[must_use]
    pub fn new() -> Self {
        let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self { dir: base.join(DEFAULT_KEY_DIR) }
    }

    /// Store at an explicit directory. Tests MUST pass a `tempfile::tempdir()`
    /// path; tests never touch the real key dir.
    #[must_use]
    pub fn with_dir<P: Into<PathBuf>>(dir: P) -> Self {
        Self { dir: dir.into() }
    }

    /// The key directory backing this store.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Full path of the seed file.
    #[must_use]
    pub fn seed_path(&self) -> PathBuf {
        self.dir.join(SEED_FILE)
    }

    /// Load the persisted key, creating it on first use.
    ///
    /// Returns the ML-DSA-65 signing key (the key-material path a
    /// `SuperSigner` is constructed from by the sealer module, which owns
    /// that type).
    ///
    /// # Errors
    /// [`SealError::KeyCustody`] when an existing seed is unreadable or not
    /// exactly 32 bytes (never regenerates over it); [`SealError::EntropyRefused`]
    /// when the OS entropy source fails during first-use creation or rotation.
    pub fn load_or_create(&self) -> Result<SigningKey<MlDsa65>, SealError> {
        let raw = self.load_or_create_seed()?;
        if raw.len() != SEED_LEN {
            return Err(SealError::KeyCustody(format!(
                "seed length {len} != {SEED_LEN}",
                len = raw.len()
            )));
        }
        let mut seed_bytes = [0u8; SEED_LEN];
        seed_bytes.copy_from_slice(&raw);
        Ok(SigningKey::<MlDsa65>::from_seed(&Seed::from(seed_bytes)))
        // `SuperSigner` (sealer lane) is constructed from this signing key;
        // keys.rs never names sibling types so both lanes compile in
        // isolation.
    }

    /// Load or create the raw 32-byte seed, enforcing the custody law.
    fn load_or_create_seed(&self) -> Result<Vec<u8>, SealError> {
        let path = self.seed_path();

        if path.exists() {
            // Existing seed: read it, refuse on ANY anomaly. Silent
            // regeneration would forge a new identity over old seals.
            let bytes = fs::read(&path).map_err(|e| {
                SealError::KeyCustody(format!(
                    "existing seed at {} is unreadable: {e}",
                    path.display()
                ))
            })?;
            if bytes.len() != SEED_LEN {
                return Err(SealError::KeyCustody(format!(
                    "existing seed at {} is {} bytes, expected exactly {SEED_LEN}; \
                     refusing to regenerate (old seals would become unverifiable)",
                    path.display(),
                    bytes.len()
                )));
            }
            return Ok(bytes);
        }

        // Absent: fresh entropy, then atomic persist. A concurrent creator
        // racing us is fine — whoever's rename lands second is the winner and
        // both sides converge on the persisted bytes on next load.
        let seed = Self::fresh_seed()?;
        self.persist_bytes(&self.seed_path(), &seed)?;
        Ok(seed)
    }

    /// Draw 32 bytes of OS entropy. Failure is a typed
    /// [`SealError::EntropyRefused`] carrying an opaque refusal digest (the
    /// digest of the OS error string), never a panic.
    fn fresh_seed() -> Result<Vec<u8>, SealError> {
        let mut raw = [0u8; SEED_LEN];
        getrandom::fill(&mut raw).map_err(|e| {
            let why = format!("os entropy: {e}");
            let mut digest = [0u8; 32];
            for (i, b) in Sha256::digest(why.as_bytes()).iter().enumerate() {
                digest[i] = *b;
            }
            SealError::EntropyRefused(crate::ledger::error::RefusalDigest(digest))
        })?;
        Ok(raw.to_vec())
    }

    /// Atomically write the seed (temp file + rename) with restrictive
    /// permissions, creating the key dir (0700) as needed.
    fn persist_seed(&self, seed: &[u8]) -> Result<(), SealError> {
        self.persist_bytes(&self.seed_path(), seed)
    }

    /// Atomic write: create dir (0700), write temp sibling, set 0600, rename.
    fn persist_bytes(&self, path: &Path, bytes: &[u8]) -> Result<(), SealError> {
        let dir = path
            .parent()
            .ok_or_else(|| SealError::KeyCustody("seed path has no parent dir".into()))?;
        fs::create_dir_all(dir).map_err(|e| {
            SealError::KeyCustody(format!("cannot create key dir {}: {e}", dir.display()))
        })?;
        restrict_dir_perms(dir);

        let mut tmp = dir.join(SEED_FILE);
        tmp.set_extension("tmp");
        fs::write(&tmp, bytes).map_err(|e| {
            SealError::KeyCustody(format!("cannot write temp seed {}: {e}", tmp.display()))
        })?;
        restrict_file_perms(&tmp);
        fs::rename(&tmp, path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            SealError::KeyCustody(format!(
                "cannot atomically rename seed into {}: {e}",
                path.display()
            ))
        })?;
        Ok(())
    }

    /// Rotate the key: fresh entropy, atomic rewrite of the seed.
    ///
    /// Old seals stay valid — each seal record carries the key id of the key
    /// that minted it, and prior public keys remain resolvable from earlier
    /// ledger records. This is a forward-only identity change; it does not
    /// and must not rewrite history.
    ///
    /// # Errors
    /// [`SealError::EntropyRefused`] when the OS entropy source fails;
    /// [`SealError::KeyCustody`] when the atomic persist fails.
    pub fn rotate(&self) -> Result<[u8; 8], SealError> {
        let seed = Self::fresh_seed()?;
        self.persist_seed(&seed)?;
        let mut seed_bytes = [0u8; SEED_LEN];
        seed_bytes.copy_from_slice(&seed);
        let sk = SigningKey::<MlDsa65>::from_seed(&Seed::from(seed_bytes));
        Ok(key_id(&sk.verifying_key().encode()[..]))
    }
}

impl Default for KeyStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Key identity law (identical recomputation of the sealer's law): the first
/// 8 bytes of BLAKE3 over the encoded ML-DSA-65 public key.
///
/// Unified (E4 reconcile, lane R8): this was SHA-256 and diverged from the
/// sealer's BLAKE3 law; both sides now pin BLAKE3, matching the ledger's
/// single-hash discipline.
#[must_use]
pub fn key_id(public_key: &[u8]) -> [u8; 8] {
    let digest = blake3::hash(public_key);
    let mut id = [0u8; 8];
    id.copy_from_slice(&digest.as_bytes()[..8]);
    id
}

/// Restrict a file to owner-only (0600). Unix: enforced via
/// `PermissionsExt`; non-unix: best-effort no-op (documented limitation —
/// Windows ACLs are out of scope for this module).
#[cfg(unix)]
fn restrict_file_perms(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_file_perms(_path: &Path) {
    // Non-unix: best-effort. The seed lives under the user's own profile;
    // Windows ACL restriction is out of scope here (documented limitation).
}

/// Restrict a directory to owner-only (0700). Same unix/non-unix split as
/// [`restrict_file_perms`].
#[cfg(unix)]
fn restrict_dir_perms(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o700));
}

#[cfg(not(unix))]
fn restrict_dir_perms(_path: &Path) {
    // Non-unix: best-effort no-op (see restrict_file_perms).
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tempdir-based store. The real default dir is never touched by tests.
    fn temp_store() -> (tempfile::TempDir, KeyStore) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store = KeyStore::with_dir(tmp.path().join("keys"));
        (tmp, store)
    }

    #[test]
    fn test_first_use_creates_seed_with_owner_only_perms() {
        let (tmp, store) = temp_store();
        let seed_path = store.seed_path();

        assert!(!seed_path.exists());
        store.load_or_create().expect("first use should create the key");
        assert!(seed_path.exists(), "seed file must be created on first use");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&seed_path).expect("seed metadata").permissions().mode();
            assert_eq!(mode & 0o777, 0o600, "seed file must be 0600, got {mode:o}");
            let dir_mode =
                fs::metadata(store.dir()).expect("key dir metadata").permissions().mode();
            assert_eq!(dir_mode & 0o777, 0o700, "key dir must be 0700");
        }

        assert_eq!(
            fs::read(seed_path).expect("seed bytes").len(),
            SEED_LEN,
            "seed must be exactly 32 bytes"
        );
        drop(tmp);
    }

    #[test]
    fn test_second_load_reproduces_same_key_id() {
        let (tmp, store) = temp_store();
        let first = store.load_or_create().expect("first load");
        let id_first = key_id(&first.verifying_key().encode()[..]);

        let second = store.load_or_create().expect("second load");
        let id_second = key_id(&second.verifying_key().encode()[..]);

        assert_eq!(
            id_first, id_second,
            "reload from persisted seed must reproduce the same identity"
        );
        drop(tmp);
    }

    #[test]
    fn test_corrupted_seed_is_refused_not_regenerated() {
        let (tmp, store) = temp_store();
        store.load_or_create().expect("create initial key");

        // Truncate to 31 bytes — wrong-length seed.
        fs::write(store.seed_path(), [0u8; 31]).expect("corrupt seed");
        let err = store.load_or_create().expect_err("wrong-length seed must be refused");
        assert!(matches!(err, SealError::KeyCustody(_)), "expected KeyCustody, got {err:?}");
        assert!(
            err.to_string().contains("refusing to regenerate"),
            "refusal must name the no-silent-regen law: {err}"
        );

        // Still 31 bytes on disk: refusal must NOT have rewritten it.
        assert_eq!(
            fs::read(store.seed_path()).expect("seed still on disk").len(),
            31,
            "refused custody must leave the corrupt seed untouched"
        );

        // Unreadable seed refuses too (dir perms make it unreadable on unix).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::write(store.seed_path(), [0u8; 32]).expect("rewrite seed");
            fs::set_permissions(store.seed_path(), fs::Permissions::from_mode(0o000))
                .expect("strip perms");
            // Restore before tempdir cleanup so drop doesn't complain.
            let result = store.load_or_create();
            fs::set_permissions(store.seed_path(), fs::Permissions::from_mode(0o600))
                .expect("restore perms");
            assert!(
                matches!(result, Err(SealError::KeyCustody(_))),
                "unreadable existing seed must be refused, got {result:?}"
            );
        }
        drop(tmp);
    }

    #[test]
    fn test_rotation_changes_key_id_and_still_reloads() {
        let (tmp, store) = temp_store();
        let original = store.load_or_create().expect("create");
        let id_before = key_id(&original.verifying_key().encode()[..]);

        let id_after = store.rotate().expect("rotate");
        assert_ne!(id_before, id_after, "rotation must produce a new identity");

        // The rotated key persists: next load reproduces the NEW id.
        let reloaded = store.load_or_create().expect("reload after rotate");
        assert_eq!(
            key_id(&reloaded.verifying_key().encode()[..]),
            id_after,
            "rotated seed must be the one persisted"
        );
        drop(tmp);
    }

    #[test]
    fn test_key_id_law_is_first_8_blake3_bytes() {
        // Law pin: key_id == BLAKE3(pubkey)[..8], stable across calls,
        // independent of KeyStore, and identical to the sealer's envelope
        // key_id law (sealer.rs).
        let pk = [0xab_u8; 1952];
        let mut expected = [0u8; 8];
        expected.copy_from_slice(&&blake3::hash(&pk).as_bytes()[..8]);
        assert_eq!(key_id(&pk), expected);
        assert_ne!(key_id(&pk), key_id(&[0xab_u8; 1951]));
    }

    #[test]
    fn test_temp_overrides_real_dir() {
        // Guard: with_dir must fully own the location — nothing is written to
        // the default path during a with_dir-backed load.
        let (tmp, store) = temp_store();
        store.load_or_create().expect("load in tempdir");
        assert!(store.seed_path().starts_with(tmp.path()));
        drop(tmp);
    }
}
