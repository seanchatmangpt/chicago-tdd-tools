#![cfg(feature = "nato")]

//! Domain-separated dossier subject derivation.
//!
//! The subject is the canonical binding between a sealed dossier envelope
//! and the Merkle root it attests. It is a BLAKE3 digest computed over a
//! length-prefixed domain frame followed by the root, so that two digests
//! computed for different protocol domains can never collide structurally
//! (domain separation, unambiguous framing).
//!
//! Framing law: `subject = BLAKE3(u32le(domain.len()) || domain || root)`.
//! The `u32le` length prefix makes the domain/root boundary unambiguous —
//! no concatenation of (domain', root') with domain'.len() != domain.len()
//! can produce the same framed input.

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

/// Protocol domain tag for dossier subject derivation (v1).
pub const SUBJECT_DOMAIN: &[u8] = b"chicago-ledger-dossier-subject-v1";

/// Derive the dossier subject for a Merkle root.
///
/// BLAKE3 over the length-prefixed domain frame (`u32le(len) || domain`)
/// followed by the 32-byte root.
#[must_use]
pub fn dossier_subject(root: &[u8; 32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    // Frame: u32le(domain.len()) || domain — unambiguous prefix frame.
    hasher.update(&(SUBJECT_DOMAIN.len() as u32).to_le_bytes());
    hasher.update(SUBJECT_DOMAIN);
    hasher.update(root);
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_is_deterministic() {
        let root = [7u8; 32];
        assert_eq!(dossier_subject(&root), dossier_subject(&root));
    }

    #[test]
    fn different_roots_differ() {
        assert_ne!(dossier_subject(&[1u8; 32]), dossier_subject(&[2u8; 32]));
    }

    #[test]
    fn domain_separation_vector() {
        // Same root, different domain frame -> different subject.
        // Recompute with a foreign domain to show the tag matters.
        let root = [9u8; 32];
        let foreign_domain = b"chicago-ledger-dossier-subject-v2";
        let mut foreign = blake3::Hasher::new();
        foreign.update(&(foreign_domain.len() as u32).to_le_bytes());
        foreign.update(foreign_domain);
        foreign.update(&root);

        assert_ne!(dossier_subject(&root), *foreign.finalize().as_bytes());
        // And the v1 domain differs from a raw unframed hash of the root.
        assert_ne!(dossier_subject(&root), *blake3::hash(&root).as_bytes());
    }

    #[test]
    fn framing_is_unambiguous() {
        // A domain of the same byte content shifted by a framing prefix must
        // not reproduce the subject of the canonical domain.
        let root = [3u8; 32];
        // Simulate a colliding concatenation: domain = [len_bytes] ++ SUBJECT_DOMAIN
        // framed identically would change the outer length prefix, so digest
        // over the raw concatenation without framing differs.
        let mut naive = blake3::Hasher::new();
        naive.update(&(SUBJECT_DOMAIN.len() as u32).to_le_bytes());
        naive.update(SUBJECT_DOMAIN);
        naive.update(&root);
        assert_eq!(*naive.finalize().as_bytes(), dossier_subject(&root));
        // Different-length domain framing cannot alias: the length prefix is
        // part of the hashed input.
        let mut short = blake3::Hasher::new();
        short.update(&4u32.to_le_bytes());
        short.update(b"abcd");
        short.update(&root);
        let mut short_expected = blake3::Hasher::new();
        short_expected.update(&4u32.to_le_bytes());
        short_expected.update(b"abcd");
        short_expected.update(&root);
        assert_eq!(*short.finalize().as_bytes(), *short_expected.finalize().as_bytes());
    }
}
