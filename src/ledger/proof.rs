#![cfg(feature = "ledger")]
//! P13 dossier Merkle proof — inclusion proof for a single court head in a
//! [`crate::ledger::dossier::DossierRoot`].
//!
//! Laws (P13 hybrid plan):
//! - leaf  = `BLAKE3(0x00 || "P13-LEAF" || 0x00 || u32le(len(court_id)) || court_id || head)`
//! - node  = `BLAKE3(0x00 || "P13-NODE" || 0x00 || left || right)`
//! - leaves sorted byte-lexicographically by `court_id`
//! - odd node PROMOTED, never duplicated
//! - empty tree root = `BLAKE3("P13-EMPTY")`
//!
//! A [`CourtProof`] is the leaf-to-root sibling path with a side flag per
//! step. Paths longer than [`MAX_PROOF_STEPS`] are refused
//! ([`DossierError::MalformedProof`]).

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

use crate::ledger::dossier::DossierRoot;
use crate::ledger::error::DossierError;

/// Maximum allowed proof-path length. A dossier with more than 2^64 leaves is
/// impossible; anything longer is malformed by construction.
pub const MAX_PROOF_STEPS: usize = 64;

/// One hop in a [`CourtProof`]: the sibling hash and which side it sits on.
///
/// `is_left == true` means the sibling is the LEFT operand of the parent node
/// hash (the running hash is the right operand).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofStep {
    /// The sibling node hash at this level.
    pub sibling: [u8; 32],
    /// `true` if the sibling is the left operand of the parent hash.
    pub is_left: bool,
}

/// Inclusion proof for one `(court_id, head)` leaf in a dossier Merkle root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourtProof {
    /// The court this proof includes.
    pub court_id: String,
    /// The court chain head committed when the proof was minted.
    pub head: [u8; 32],
    /// Sibling path, leaf-first, bottom-up.
    pub path: Vec<ProofStep>,
}

/// Domain-separated BLAKE3 leaf hash for a `(court_id, head)` pair.
#[must_use]
pub fn leaf_hash(court_id: &str, head: &[u8; 32]) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(&[0x00]);
    h.update(b"P13-LEAF");
    h.update(&[0x00]);
    h.update(&u32::to_le_bytes(court_id.len() as u32));
    h.update(court_id.as_bytes());
    h.update(head);
    *h.finalize().as_bytes()
}

/// Domain-separated BLAKE3 internal node hash.
#[must_use]
pub fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(&[0x00]);
    h.update(b"P13-NODE");
    h.update(&[0x00]);
    h.update(left);
    h.update(right);
    *h.finalize().as_bytes()
}

/// Root of the empty dossier: `BLAKE3("P13-EMPTY")`.
#[must_use]
pub fn empty_root() -> [u8; 32] {
    *blake3::hash(b"P13-EMPTY").as_bytes()
}

impl CourtProof {
    /// Verify this proof against a dossier root.
    ///
    /// Replays leaf -> root using the stored sibling path and compares the
    /// final hash to `root`.
    ///
    /// # Errors
    /// - [`DossierError::MalformedProof`] if the path exceeds
    ///   [`MAX_PROOF_STEPS`] steps.
    /// - [`DossierError::VerificationFailed`] if the recomputed root differs.
    pub fn verify(&self, root: &DossierRoot) -> Result<(), DossierError> {
        if self.path.len() > MAX_PROOF_STEPS {
            return Err(DossierError::MalformedProof);
        }

        let mut current = leaf_hash(&self.court_id, &self.head);
        for step in &self.path {
            current = if step.is_left {
                node_hash(&step.sibling, &current)
            } else {
                node_hash(&current, &step.sibling)
            };
        }

        if current != root.0 {
            return Err(DossierError::VerificationFailed(self.court_id.clone()));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "ledger"))]
mod tests {
    use super::*;

    fn head(seed: u8) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(&[seed]);
        *h.finalize().as_bytes()
    }

    /// Independent reference: Merkle root with PROMOTED odd nodes (spec law:
    /// paired nodes first, the promoted odd node appended LAST so it pairs
    /// against the next level the same way seal_with_proofs pairs).
    fn reference_root_promoted(mut leaves: Vec<[u8; 32]>) -> [u8; 32] {
        while leaves.len() > 1 {
            let promoted = (leaves.len() % 2 == 1).then(|| leaves[leaves.len() - 1]);
            let pair_count = leaves.len() / 2;
            let mut next = Vec::with_capacity(pair_count + 1);
            for i in 0..pair_count {
                next.push(node_hash(&leaves[2 * i], &leaves[2 * i + 1]));
            }
            if let Some(p) = promoted {
                next.push(p);
            }
            leaves = next;
        }
        leaves[0]
    }

    #[test]
    fn single_leaf_inclusion_proof_verifies() {
        let root = DossierRoot(reference_root_promoted(vec![leaf_hash("court-a", &head(0))]));
        let proof = CourtProof { court_id: "court-a".into(), head: head(0), path: vec![] };
        assert_eq!(proof.verify(&root), Ok(()));
    }

    #[test]
    fn proof_fails_on_tampered_sibling() {
        let a = leaf_hash("court-a", &head(0));
        let b = leaf_hash("court-b", &head(1));
        let root = DossierRoot(node_hash(&a, &b));
        let good = CourtProof {
            court_id: "court-a".into(),
            head: head(0),
            path: vec![ProofStep { sibling: b, is_left: false }],
        };
        let tampered = CourtProof {
            court_id: "court-a".into(),
            head: head(0),
            path: vec![ProofStep { sibling: node_hash(&b, &head(9)), is_left: false }],
        };
        assert_eq!(good.verify(&root), Ok(()));
        assert!(tampered.verify(&root).is_err());
    }

    #[test]
    fn proof_path_over_limit_is_malformed() {
        let root = DossierRoot([0u8; 32]);
        let path = vec![ProofStep { sibling: [0u8; 32], is_left: false }; MAX_PROOF_STEPS + 1];
        let proof = CourtProof { court_id: "x".into(), head: [0u8; 32], path };
        assert_eq!(proof.verify(&root), Err(DossierError::MalformedProof));
    }
}
