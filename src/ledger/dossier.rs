#![cfg(feature = "ledger")]
//! P13 hybrid dossier: per-court BLAKE3 chains + a Merkle dossier root over
//! all court heads.
//!
//! Two builders:
//!
//! - [`CourtLedgerBuilder`] maintains one court's BLAKE3 chain. The per-entry
//!   hash law matches [`Blake3ReceiptEntry`] (BLAKE3 over
//!   `prev_hash || content`), but the LINK rule is a builder fold, distinct
//!   from the validator's `prev = prior stored_hash` law:
//!   `head(0) = [0u8; 32]` (zeroed genesis) and
//!   `head(n+1) = BLAKE3(head(n) || receipt.stored_hash())`. Builder chains
//!   are consumed via `head()` into the Merkle dossier, never replayed
//!   through `validate_chain`. Appending a receipt belonging to a different
//!   court is refused ([`LedgerError::CrossCourtAppend`]).
//! - [`DossierBuilder`] collects `(court_id, head)` leaves, sorts them
//!   byte-lexicographically by `court_id`, and seals a Merkle root with the
//!   domain-separated leaf/node hashes documented in
//!   [`crate::ledger::proof`]. Odd nodes are PROMOTED, never duplicated.
//!   Duplicate `court_id`s and empty dossiers are refused.

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

use std::collections::HashSet;

use crate::ledger::error::{DossierError, LedgerError};
#[cfg(test)]
use crate::ledger::proof::empty_root;
use crate::ledger::proof::{leaf_hash, node_hash, CourtProof, ProofStep};
use crate::ledger::receipt::ChainedCourtReceipt;
use crate::observability::receipt::Blake3ReceiptEntry;

/// Absolute offset of the `name` field inside the 352-byte chained record
/// (16-byte header + body-relative offset 204).
const NAME_ABS: usize = 220;

/// The court identity of a receipt: its validated `name` field, decoded from
/// the encoded record (NUL-stripped UTF-8).
#[must_use]
pub fn court_id_of(receipt: &ChainedCourtReceipt) -> String {
    let record = receipt.encode();
    let field = &record[NAME_ABS..NAME_ABS + crate::ledger::receipt::NAME_MAX];
    let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

/// Maintains one court's BLAKE3 receipt chain and its running head.
///
/// Zeroed genesis: a fresh builder's `head()` is `[0u8; 32]`; every append
/// advances `head <- BLAKE3(head || receipt.stored_hash())` — a builder fold
/// law distinct from the [`Blake3ReceiptEntry`] validator's link rule (the
/// receipt's chain hash is the content contribution, never the threaded
/// prev-hash for n >= 1).
///
/// Two append gates beyond the court check:
///
/// - **Replay refusal**: a receipt whose `stored_hash` was already appended
///   to this builder is refused
///   ([`LedgerError::ChainDiscontinuity`]).
/// - **Parent threading**: the receipt's `parent_receipt_hash` must equal
///   the builder's current head; genesis (all-zero parent) is lawful only as
///   the first append, where it matches the zeroed genesis head. A mis-threaded
///   receipt is refused with [`LedgerError::ChainDiscontinuity`].
#[derive(Debug)]
pub struct CourtLedgerBuilder {
    court_id: String,
    head: [u8; 32],
    receipts: Vec<ChainedCourtReceipt>,
    stored_hashes: Vec<[u8; 32]>,
    /// Membership index over `stored_hashes`; keeps replay refusal O(1) per
    /// append instead of a linear Vec scan (O(N) per append, O(N²) per chain).
    stored_hash_set: HashSet<[u8; 32]>,
}

impl CourtLedgerBuilder {
    /// Create a builder for `court_id` with a zeroed genesis head.
    #[must_use]
    pub fn new(court_id: impl Into<String>) -> Self {
        Self {
            court_id: court_id.into(),
            head: [0u8; 32],
            receipts: Vec::new(),
            stored_hashes: Vec::new(),
            stored_hash_set: HashSet::new(),
        }
    }

    /// The court this builder owns.
    #[must_use]
    pub fn court_id(&self) -> &str {
        &self.court_id
    }

    /// Current chain head (zeroed until the first append).
    #[must_use]
    pub const fn head(&self) -> [u8; 32] {
        self.head
    }

    /// Append a receipt belonging to this builder's court.
    ///
    /// # Errors
    /// - [`LedgerError::CrossCourtAppend`] if the receipt's court (its
    ///   validated `name` field) differs from this builder's court.
    /// - [`LedgerError::ChainDiscontinuity`] if the receipt is a
    ///   byte-identical replay of one already appended, or if its
    ///   `parent_receipt_hash` does not equal the current head (genesis
    ///   all-zero parent is lawful only at index 0).
    pub fn append(&mut self, receipt: ChainedCourtReceipt) -> Result<(), LedgerError> {
        let receipt_court = court_id_of(&receipt);
        if receipt_court != self.court_id {
            return Err(LedgerError::CrossCourtAppend {
                expected: self.court_id.clone(),
                actual: receipt_court,
            });
        }
        let stored = receipt.stored_hash();
        let actual_parent = receipt.prev_hash();
        if self.stored_hash_set.contains(&stored) {
            return Err(LedgerError::ChainDiscontinuity {
                expected_parent: self.head,
                actual: actual_parent,
            });
        }
        if actual_parent != self.head {
            return Err(LedgerError::ChainDiscontinuity {
                expected_parent: self.head,
                actual: actual_parent,
            });
        }
        let mut h = blake3::Hasher::new();
        h.update(&self.head);
        h.update(&stored);
        self.head = *h.finalize().as_bytes();
        self.stored_hashes.push(stored);
        self.stored_hash_set.insert(stored);
        self.receipts.push(receipt);
        Ok(())
    }

    /// Consume the builder, returning the accumulated receipts.
    ///
    /// # Errors
    /// - [`LedgerError::EmptyChain`] if no receipts were appended.
    pub fn build(self) -> Result<Vec<ChainedCourtReceipt>, LedgerError> {
        if self.receipts.is_empty() {
            return Err(LedgerError::EmptyChain { court_id: self.court_id });
        }
        Ok(self.receipts)
    }
}

/// The sealed Merkle root over all court heads in a dossier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DossierRoot(pub [u8; 32]);

/// Collects `(court_id, head)` leaves and seals a [`DossierRoot`].
#[derive(Debug, Clone, Default)]
pub struct DossierBuilder {
    courts: Vec<(String, [u8; 32])>,
}

impl DossierBuilder {
    /// Create an empty dossier builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a court leaf.
    ///
    /// # Errors
    /// - [`DossierError::DuplicateCourt`] if this `court_id` was already added.
    pub fn add_court(
        &mut self,
        court_id: impl Into<String>,
        head: [u8; 32],
    ) -> Result<(), DossierError> {
        let court_id = court_id.into();
        if self.courts.iter().any(|(id, _)| id == &court_id) {
            return Err(DossierError::DuplicateCourt(court_id));
        }
        self.courts.push((court_id, head));
        Ok(())
    }

    /// Sorted (byte-lexicographic by `court_id`) leaf view.
    fn sorted_leaves(&self) -> Vec<(String, [u8; 32])> {
        let mut leaves = self.courts.clone();
        leaves.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        leaves
    }

    /// Seal the Merkle root. Leaf order is independent of insertion order.
    ///
    /// # Errors
    /// - [`DossierError::EmptyDossier`] if no courts were added.
    pub fn seal(&self) -> Result<DossierRoot, DossierError> {
        let (root, _) = self.seal_with_proofs()?;
        Ok(root)
    }

    /// Seal the Merkle root and mint an inclusion proof for every court.
    ///
    /// At N leaves the deepest path is `ceil(lg2(N))` steps (a promoted node
    /// shortens some paths), so ~5 steps at 20 leaves.
    ///
    /// # Errors
    /// - [`DossierError::EmptyDossier`] if no courts were added.
    pub fn seal_with_proofs(&self) -> Result<(DossierRoot, Vec<CourtProof>), DossierError> {
        if self.courts.is_empty() {
            return Err(DossierError::EmptyDossier);
        }
        let leaves = self.sorted_leaves();
        let mut level: Vec<[u8; 32]> =
            leaves.iter().map(|(id, head)| leaf_hash(id, head)).collect();

        // Per-LEAF proof paths, built bottom-up. `node_of_leaf[i]` is leaf
        // i's node index at the current level, so the index compaction caused
        // by promoting odd nodes never desynchronizes a leaf from its path.
        // A promoted (odd) node carries its path to the next level unchanged:
        // nothing to append, it simply advances.
        let n = level.len();
        let mut paths: Vec<Vec<ProofStep>> = vec![Vec::new(); n];
        let mut node_of_leaf: Vec<usize> = (0..n).collect();

        while level.len() > 1 {
            let promoted = (level.len() % 2 == 1).then(|| level[level.len() - 1]);
            let pair_count = level.len() / 2;
            let mut next = Vec::with_capacity(pair_count + 1);
            for i in 0..pair_count {
                let (l, r) = (level[2 * i], level[2 * i + 1]);
                next.push(node_hash(&l, &r));
                for leaf in (0..n).filter(|lf| node_of_leaf[*lf] == 2 * i) {
                    paths[leaf].push(ProofStep { sibling: r, is_left: false });
                }
                for leaf in (0..n).filter(|lf| node_of_leaf[*lf] == 2 * i + 1) {
                    paths[leaf].push(ProofStep { sibling: l, is_left: true });
                }
            }
            if let Some(p) = promoted {
                next.push(p);
            }
            // Remap: a node at index j of `next` now hosts the path(s) of the
            // leaf indices whose node advanced there. Pair children move to
            // index i; the promoted child keeps its place as the last node.
            for leaf in 0..n {
                let node = node_of_leaf[leaf];
                node_of_leaf[leaf] = if node < 2 * pair_count { node / 2 } else { pair_count };
            }
            level = next;
        }

        let root = DossierRoot(level[0]);
        let proofs = leaves
            .iter()
            .enumerate()
            .map(|(i, (id, head))| CourtProof {
                court_id: id.clone(),
                head: *head,
                path: std::mem::take(&mut paths[i]),
            })
            .collect();

        Ok((root, proofs))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "ledger"))]
mod tests {
    use super::*;
    use crate::ledger::receipt::{CourtOutcome, CourtReceiptV1, ReceiptMetrics, ThermalClass};

    /// Deterministic head for a seed byte.
    fn head(seed: u8) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(&[seed]);
        *h.finalize().as_bytes()
    }

    fn leaf(id: &str, seed: u8) -> [u8; 32] {
        leaf_hash(id, &head(seed))
    }

    /// Independent reference: Merkle root with PROMOTED odd nodes appended
    /// LAST at each level (the spec law).
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

    /// Independent reference: root with DUPLICATED odd nodes (the forbidden
    /// variant).
    fn reference_root_duplicated(mut leaves: Vec<[u8; 32]>) -> [u8; 32] {
        while leaves.len() > 1 {
            if leaves.len() % 2 == 1 {
                leaves.push(*leaves.last().expect("nonempty"));
            }
            let mut next = Vec::new();
            for pair in leaves.chunks(2) {
                next.push(node_hash(&pair[0], &pair[1]));
            }
            leaves = next;
        }
        leaves[0]
    }

    /// A real ChainedCourtReceipt whose name (court id) is `court_id`,
    /// threading `parent` as its parent hash.
    fn threaded(court_id: &str, parent: [u8; 32], seed: u8) -> ChainedCourtReceipt {
        ChainedCourtReceipt::from_receipt(&CourtReceiptV1 {
            subject_hash: head(seed),
            parent_receipt_hash: parent,
            metrics: ReceiptMetrics {
                timestamp_ms: 0,
                run_id: u64::from(seed),
                total_ticks: 0,
                wall_ms: 0,
                thermal: ThermalClass::Warm,
                outcome: CourtOutcome::Pass,
                budget_met: true,
            },
            positive_control: [0u8; 32],
            metadata_digest: [0u8; 32],
            prev_self_hash: [0u8; 32],
            name: court_id.to_owned(),
            fail_closed: false,
        })
        .expect("test court ids are within NAME_MAX")
    }

    /// A genesis (zero-parent) receipt for `court_id`.
    fn receipt(court_id: &str, seed: u8) -> ChainedCourtReceipt {
        threaded(court_id, [0u8; 32], seed)
    }

    #[test]
    fn twenty_court_dossier_all_proofs_verify() {
        let mut b = DossierBuilder::new();
        for i in 0..20u8 {
            b.add_court(format!("court-{i:02}"), head(i)).expect("unique ids");
        }
        let (root, proofs) = b.seal_with_proofs().expect("nonempty");
        assert_eq!(proofs.len(), 20);
        for p in &proofs {
            assert_eq!(p.verify(&root), Ok(()), "proof for {} failed", p.court_id);
        }
        // Level sizes 20 -> 10 -> 5 -> 3 -> 2 -> 1: five levels, so the
        // deepest path is 5 steps.
        let max_steps = proofs.iter().map(|p| p.path.len()).max().expect("nonempty");
        assert_eq!(max_steps, 5, "deepest path should be 5 steps at 20 leaves");
    }

    #[test]
    fn tamper_matrix_every_mutated_input_fails_some_proof() {
        let mut b = DossierBuilder::new();
        for i in 0..8u8 {
            b.add_court(format!("court-{i}"), head(i)).expect("unique ids");
        }
        let (root, proofs) = b.seal_with_proofs().expect("nonempty");

        // (a) tamper a head: only that court's proof fails against the new root.
        let mut b2 = DossierBuilder::new();
        for i in 0..8u8 {
            let h = if i == 3 { head(200) } else { head(i) };
            b2.add_court(format!("court-{i}"), h).expect("unique ids");
        }
        let (root2, proofs2) = b2.seal_with_proofs().expect("nonempty");
        assert_ne!(root, root2, "tampered head must change the root");
        // Merkle locality: a proof binds every sibling on its path, so ANY
        // original proof fails against the tampered root (each path crosses
        // the tampered leaf's subtree at 8 leaves), and the tampered court's
        // proof fails against the original root.
        for p in &proofs {
            assert!(
                p.verify(&root2).is_err(),
                "original proof must fail against the tampered root ({})",
                p.court_id
            );
            if p.court_id == "court-3" {
                continue;
            }
            assert_eq!(p.verify(&root), Ok(()), "untampered courts verify their own root");
        }
        // Fresh proofs minted over the tampered dossier verify against it
        // (the failure above is locality, not a proof-minting defect), and
        // every one of them fails against the ORIGINAL root: the tampered
        // court's head is bound into its leaf, the others by locality.
        for p in &proofs2 {
            assert_eq!(p.verify(&root2), Ok(()), "tampered-dossier proof verifies");
            assert!(p.verify(&root).is_err(), "tampered-dossier proof must fail the original root");
        }

        // (b) tamper the root: every proof fails.
        let mut bad_root = root;
        bad_root.0[0] ^= 0x01;
        for p in &proofs {
            assert!(p.verify(&bad_root).is_err(), "tampered root fails all proofs");
        }

        // (c) tamper a sibling hash: proof fails.
        let mut tp = proofs[0].clone();
        tp.path[0].sibling[0] ^= 0x01;
        assert!(tp.verify(&root).is_err(), "tampered sibling must fail");

        // (d) flip an is_left flag: asymmetric node hash must fail.
        let mut fp = proofs[0].clone();
        let last = fp.path.len() - 1;
        fp.path[last].is_left = !fp.path[last].is_left;
        assert!(fp.verify(&root).is_err(), "flipped side flag must fail");
    }

    #[test]
    fn permutation_invariance_shuffled_insertion_same_root() {
        let make = || {
            let mut b = DossierBuilder::new();
            for i in 0..7u8 {
                b.add_court(format!("court-{i}"), head(i + 10)).expect("unique ids");
            }
            b
        };
        let (r1, _) = make().seal_with_proofs().expect("nonempty");

        let mut b2 = DossierBuilder::new();
        for i in (0..7u8).rev() {
            b2.add_court(format!("court-{i}"), head(i + 10)).expect("unique ids");
        }
        let (r2, _) = b2.seal_with_proofs().expect("nonempty");
        assert_eq!(r1, r2, "insertion order must not affect the root");

        let mut ids: Vec<(String, u8)> = (0..7u8).map(|i| (format!("court-{i}"), i + 10)).collect();
        ids.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let leaves: Vec<[u8; 32]> = ids.iter().map(|(id, s)| leaf(id, *s)).collect();
        assert_eq!(r1, DossierRoot(reference_root_promoted(leaves)));
    }

    #[test]
    fn odd_leaf_promotion_at_3_and_5() {
        for n in [3usize, 5] {
            let mut b = DossierBuilder::new();
            for i in 0..n {
                b.add_court(format!("c{i}"), head(i as u8)).expect("unique ids");
            }
            let leaves: Vec<[u8; 32]> = (0..n).map(|i| leaf(&format!("c{i}"), i as u8)).collect();
            let promoted = reference_root_promoted(leaves.clone());
            let duplicated = reference_root_duplicated(leaves);

            let (root, proofs) = b.seal_with_proofs().expect("nonempty");
            assert_eq!(root, DossierRoot(promoted), "n={n}: root must be PROMOTED variant");
            assert_ne!(
                root,
                DossierRoot(duplicated),
                "n={n}: root must NOT be the duplicated variant"
            );
            for p in &proofs {
                assert_eq!(p.verify(&root), Ok(()), "n={n}: proof for {} failed", p.court_id);
            }
        }
    }

    #[test]
    fn single_court_inclusion_proof_verifies_and_fails_on_tampered_sibling() {
        let mut b = DossierBuilder::new();
        b.add_court("solo", head(7)).expect("unique");
        let (root, proofs) = b.seal_with_proofs().expect("nonempty");
        assert_eq!(proofs.len(), 1);
        assert!(proofs[0].path.is_empty(), "single leaf needs no steps");
        assert_eq!(proofs[0].verify(&root), Ok(()));

        let fabricated = CourtProof {
            court_id: "solo".into(),
            head: head(7),
            path: vec![ProofStep { sibling: [0u8; 32], is_left: false }],
        };
        assert!(fabricated.verify(&root).is_err(), "fabricated sibling must fail");
    }

    #[test]
    fn empty_dossier_root_law_and_proof_refusal() {
        let b = DossierBuilder::new();
        assert_eq!(b.seal(), Err(DossierError::EmptyDossier));
        assert_eq!(b.seal_with_proofs(), Err(DossierError::EmptyDossier));
        assert_eq!(empty_root(), *blake3::hash(b"P13-EMPTY").as_bytes());
        let forged = CourtProof {
            court_id: "ghost".into(),
            head: head(1),
            path: vec![ProofStep { sibling: [0u8; 32], is_left: false }],
        };
        assert!(forged.verify(&DossierRoot(empty_root())).is_err(), "forged proof fails");
    }

    #[test]
    fn duplicate_court_id_is_rejected() {
        let mut b = DossierBuilder::new();
        b.add_court("dup", head(1)).expect("first add ok");
        assert_eq!(b.add_court("dup", head(2)), Err(DossierError::DuplicateCourt("dup".into())));
        let mut expected = DossierBuilder::new();
        expected.add_court("dup", head(1)).expect("ok");
        assert_eq!(b.seal(), expected.seal(), "original leaf survives a refused add");
    }

    #[test]
    fn court_chain_zeroed_genesis_then_advances() {
        let mut ledger = CourtLedgerBuilder::new("court-a");
        assert_eq!(ledger.head(), [0u8; 32], "genesis head must be zeroed");
        ledger.append(receipt("court-a", 1)).expect("same court");
        let h1 = ledger.head();
        assert_ne!(h1, [0u8; 32]);
        ledger.append(threaded("court-a", h1, 2)).expect("same court");
        assert_ne!(ledger.head(), h1, "head must advance per append");
    }

    #[test]
    fn court_chain_rejects_cross_court_append() {
        let mut ledger = CourtLedgerBuilder::new("court-a");
        assert_eq!(
            ledger.append(receipt("court-b", 1)),
            Err(LedgerError::CrossCourtAppend {
                expected: "court-a".into(),
                actual: "court-b".into(),
            })
        );
        assert_eq!(ledger.head(), [0u8; 32], "refused append must not advance head");
        ledger.append(receipt("court-a", 1)).expect("same court after refusal");
    }

    #[test]
    fn court_chain_build_rejects_empty() {
        let ledger = CourtLedgerBuilder::new("court-a");
        assert_eq!(ledger.build(), Err(LedgerError::EmptyChain { court_id: "court-a".into() }));
    }

    #[test]
    fn court_chain_follows_blake3_prev_content_law() {
        let mut ledger = CourtLedgerBuilder::new("court-a");
        let r1 = receipt("court-a", 5);
        let mut h = blake3::Hasher::new();
        h.update(&[0u8; 32]);
        h.update(&r1.stored_hash());
        let e1 = *h.finalize().as_bytes();
        let r2 = threaded("court-a", e1, 6);
        ledger.append(r1).expect("ok");
        ledger.append(r2.clone()).expect("ok");
        let mut h = blake3::Hasher::new();
        h.update(&e1);
        h.update(&r2.stored_hash());
        let e2 = *h.finalize().as_bytes();
        assert_eq!(ledger.head(), e2);
    }

    #[test]
    fn court_chain_refuses_byte_identical_replay() {
        let mut ledger = CourtLedgerBuilder::new("court-a");
        let r1 = receipt("court-a", 1);
        ledger.append(r1.clone()).expect("first append of r1");
        let head_after_first = ledger.head();
        assert_eq!(
            ledger.append(r1.clone()),
            Err(LedgerError::ChainDiscontinuity {
                expected_parent: head_after_first,
                actual: [0u8; 32],
            }),
            "byte-identical replay of an appended receipt must be refused"
        );
        // A replay threaded onto the CURRENT head (freshly re-parented but
        // otherwise identical content) is still a different stored hash, so
        // it appends — the replay gate is on stored_hash identity.
        ledger.append(threaded("court-a", head_after_first, 2)).expect("child appends");
        assert_eq!(ledger.build().expect("nonempty").len(), 2);
    }

    #[test]
    fn court_chain_refuses_wrong_parent_with_typed_error() {
        let mut ledger = CourtLedgerBuilder::new("court-a");
        ledger.append(receipt("court-a", 1)).expect("genesis appends");
        let head1 = ledger.head();

        // Stale parent (genesis zeros) at index 1.
        assert_eq!(
            ledger.append(receipt("court-a", 2)),
            Err(LedgerError::ChainDiscontinuity { expected_parent: head1, actual: [0u8; 32] })
        );
        // Fabricated parent (never observed head).
        let fabricated = threaded("court-a", head(200), 2);
        assert_eq!(
            ledger.append(fabricated),
            Err(LedgerError::ChainDiscontinuity { expected_parent: head1, actual: head(200) })
        );
        assert_eq!(ledger.head(), head1, "refused appends must not advance head");
        // Valid threading still accepted after refusals.
        ledger.append(threaded("court-a", head1, 2)).expect("threaded child");
        assert_ne!(ledger.head(), head1);
    }

    #[test]
    fn court_chain_refuses_second_genesis_and_accepts_valid_threading() {
        let mut ledger = CourtLedgerBuilder::new("court-a");
        ledger.append(receipt("court-a", 1)).expect("genesis");
        // A second all-zero-parent receipt after the head advanced is a
        // discontinuity, not a new genesis.
        assert!(matches!(
            ledger.append(receipt("court-a", 2)),
            Err(LedgerError::ChainDiscontinuity { .. })
        ));
        let h = ledger.head();
        let mut h2 = blake3::Hasher::new();
        h2.update(&h);
        h2.update(&threaded("court-a", h, 2).stored_hash());
        let expected_head = *h2.finalize().as_bytes();
        ledger.append(threaded("court-a", h, 2)).expect("valid threading");
        assert_eq!(ledger.head(), expected_head, "head law holds across a threaded append");
    }
}
