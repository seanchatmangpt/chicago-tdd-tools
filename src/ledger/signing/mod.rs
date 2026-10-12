#![cfg(feature = "nato")]
//! Dossier signing and sealing: pure-Rust ML-DSA-65 sealing per the P14 design.
//!
//! This module is the ONLY subtree in `crate::ledger` permitted to touch
//! `std::fs` — that authority lives solely in [`keys`] (`KeyStore`), which
//! persists and rotates the signing seed on disk. Everything else here is
//! pure computation over in-memory byte buffers: domain-separated subjects,
//! ML-DSA-65 signatures, and the wire-format envelope for sealed dossiers.
//!
//! Submodules:
//! - [`signer`]: ML-DSA-65 key generation, signing, verification (`Signer`)
//! - [`subject`]: domain-separated 32-byte dossier subject derivation
//! - [`sealer`]: `SealedDossier` wire format, `seal`/`verify_sealed`
//! - [`keys`]: on-disk key persistence and rotation (`KeyStore`, the lone
//!   `std::fs` user in the ledger)

pub mod keys;
pub mod sealer;
pub mod signer;
pub mod subject;

pub use keys::{key_id, KeyStore};
pub use sealer::{
    alg_marker_ml_dsa_65, seal, verify_sealed, SealEnvelope, SealVerdict, SealedDossier,
    FORMAT_VERSION,
};
pub use signer::{
    assert_wire_sizes, Signer, PUBLIC_KEY_LEN, SEED_LEN, SIGNATURE_LEN, SIGN_CONTEXT,
};
pub use subject::{dossier_subject, SUBJECT_DOMAIN};
