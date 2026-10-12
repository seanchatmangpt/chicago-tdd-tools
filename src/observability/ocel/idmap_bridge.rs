//! Ported from strata-protocol src/idmap.rs @ 0de13a3.
//!
//! Deterministic string → 16-bit object-id assignment for the offline tier.
//!
//! Bridges named entities (activities, OCEL object types) to a `u16` id
//! space. Ids are assigned by first-seen order (1..=n in input order) — the
//! id space is deterministic and stable across runs given the same input
//! order. blake3 is used ONLY as a collision sentinel: each name's digest is
//! truncated to 16 bits and compared against the others; two distinct names
//! landing in the same 16-bit bucket make `build` refuse, so the digest never
//! participates in id derivation.
//!
//! # Capacity planning and the birthday bound
//!
//! The sentinel is a 16-bit hash, so distinct names collide with probability
//! governed by the birthday problem over N = 2^16 = 65,536 buckets. For `n`
//! distinct names the probability of at least one collision is
//!
//! ```text
//! p(n) = 1 - prod_{k=0}^{n-1} (1 - k/N) ≈ 1 - exp(-n(n-1)/(2N))
//! ```
//!
//! Setting p = 1/2 and solving: n ≈ sqrt(2N·ln 2) = sqrt(2 · 65536 · 0.693147)
//! = sqrt(90,849.4) ≈ 301.4, i.e. **~302 names give a ~50% chance of a
//! sentinel collision** somewhere in the input. The exact 50% crossing is
//! n = 302 (p(301) ≈ 0.4997, p(302) ≈ 0.5013). Practical guidance: keep the
//! corpus under ~83 names for < 5% collision probability (sqrt(2·N·ln(1/0.95))
//! ≈ 82.5; exact crossing at 83 gives p ≈ 0.0501), and treat anything in the
//! hundreds as a design smell — a real workload with more than a few hundred
//! named objects needs a wider id space, not a better hash.
//!
//! [`collision_risk`] exists so capacity planning can measure the real corpus
//! instead of trusting the asymptotic bound.
//!
//! # Empirical collision data
//!
//! Empirical: sequential `o#####` names collide at 141 distinct objects
//! (`blake3("o00140") == blake3("o00141")` — a 16-bit sentinel collision hit
//! far below the ~302 random-name birthday bound, because sequential names
//! are adversarial for the truncated digest, not random). The random-name
//! birthday bound of ~302 still stands for non-sequential inputs. Practical
//! hard limit: keep distinct named objects under ~100 at the packed tier, or
//! use the full-fidelity `OcelLog` tier.
#![cfg(feature = "ocel-ingest")]

/// Typed refusal: two distinct names share a 16-bit blake3-truncated digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdMapError {
    /// `(index_a, name_a, index_b, name_b)` — first colliding pair in input
    /// order. Indices are positions in the `build` input slice.
    Collision {
        /// Position of the earlier name in the `build` input slice.
        index_a: usize,
        /// The earlier colliding name.
        name_a: String,
        /// Position of the later name in the `build` input slice.
        index_b: usize,
        /// The later colliding name.
        name_b: String,
    },
}

/// Deterministic name → `u16` id map for the offline tier.
///
/// Construct only through [`IdMap::build`], which verifies practical
/// bijectivity (distinct names ⇒ distinct ids ⇒ distinct 16-bit sentinels)
/// before handing out the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdMap {
    entries: Vec<(u16, String)>,
}

/// Truncate blake3(name) to 16 bits (first two bytes, big-endian).
fn sentinel(name: &[u8]) -> u16 {
    let h = blake3::hash(name);
    u16::from_be_bytes([h.as_bytes()[0], h.as_bytes()[1]])
}

impl IdMap {
    /// Assign ids `1..=n` in first-seen input order, refusing if any two
    /// distinct names collide on the 16-bit truncated blake3 sentinel.
    ///
    /// Duplicate names are an [`IdMapError::Collision`] with both indices
    /// pointing at the same name (a duplicate is the degenerate collision:
    /// same name, same digest — it would break bijectivity identically).
    ///
    /// # Errors
    ///
    /// [`IdMapError::Collision`] when two distinct names (or a duplicate)
    /// share one 16-bit sentinel bucket.
    pub fn build(names: &[&str]) -> Result<Self, IdMapError> {
        let mut entries: Vec<(u16, String)> = Vec::with_capacity(names.len());
        for (i, name) in names.iter().enumerate() {
            for (j, existing) in entries.iter().enumerate() {
                if existing.1 == *name
                    || sentinel(existing.1.as_bytes()) == sentinel(name.as_bytes())
                {
                    return Err(IdMapError::Collision {
                        index_a: j,
                        name_a: existing.1.clone(),
                        index_b: i,
                        name_b: (*name).to_string(),
                    });
                }
            }
            // JUSTIFICATION: ids are 1..=n with n <= u16::MAX - 1 enforced
            // by the MAX_ENTRIES capacity gate in `BridgeVocab::build`; the
            // index arithmetic is the documented id assignment itself.
            #[allow(clippy::cast_possible_truncation)]
            let id = (i + 1) as u16;
            entries.push((id, (*name).to_string()));
        }
        Ok(Self { entries })
    }

    /// Id for `name`, if present.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<u16> {
        self.entries.iter().find(|(_, n)| n == name).map(|(id, _)| *id)
    }

    /// Name for `id`, if present.
    #[must_use]
    pub fn name(&self, id: u16) -> Option<&str> {
        self.entries.iter().find(|(i, _)| *i == id).map(|(_, n)| n.as_str())
    }

    /// True iff every id is unique, every name is unique, and every sentinel
    /// is unique — i.e. name ↔ id is a bijection and the 16-bit sentinel
    /// space is collision-free for this corpus.
    #[must_use]
    pub fn is_bijective(&self) -> bool {
        let n = self.entries.len();
        // ids are 1..=n by construction; check distinctness of names and
        // sentinels (a sentinel collision here can only come from a map built
        // by a different path, but the check is cheap and keeps the invariant
        // self-evident rather than trusted).
        for i in 0..n {
            for j in (i + 1)..n {
                let (ia, na) = &self.entries[i];
                let (ib, nb) = &self.entries[j];
                if ia == ib || na == nb || sentinel(na.as_bytes()) == sentinel(nb.as_bytes()) {
                    return false;
                }
            }
        }
        true
    }

    /// Number of entries (== max id, since ids are 1..=n).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Always false; an empty map is valid.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Standalone capacity probe: count distinct sentinel collisions among
/// `names` without building a map.
///
/// A "collision" here is a 16-bit bucket holding more than one distinct name;
/// the return value is the number of such buckets. (Names appearing twice do
/// not count — same name, same digest is expected, not a collision.)
///
/// For an i.i.d. uniform hash the expected number of colliding buckets among
/// `n` names over `N = 65536` buckets follows the birthday bound in the
/// module docs: expect the first collision around n ≈ 302 (p ≈ 50%).
#[must_use]
pub fn collision_risk(names: &[&str]) -> usize {
    use std::collections::HashMap;
    let mut buckets: HashMap<u16, (usize, String)> = HashMap::new();
    let mut collided_buckets = 0usize;
    for name in names {
        let s = sentinel(name.as_bytes());
        match buckets.get_mut(&s) {
            None => {
                buckets.insert(s, (1, (*name).to_string()));
            }
            Some((count, first_name)) => {
                if first_name != name {
                    if *count == 1 {
                        collided_buckets += 1;
                    }
                    *count += 1;
                }
                let _ = first_name;
            }
        }
    }
    collided_buckets
}

/// Typed refusal surfaced by [`BridgeVocab::build`]: either a within-field
/// [`IdMapError::Collision`] labeled with its field, or a field exceeding the
/// 16-bit id-space capacity bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    /// Two distinct names in `field` share a 16-bit blake3-truncated digest.
    Collision {
        /// Which vocabulary collided: `"activities"` or `"objects"`.
        field: &'static str,
        /// The underlying first-colliding-pair error.
        collision: IdMapError,
    },
    /// `field` has more names than the id space can hold bijectively.
    CapacityExceeded {
        /// Which vocabulary overflowed: `"activities"` or `"objects"`.
        field: &'static str,
        /// Number of names supplied for the field.
        count: usize,
        /// Maximum supported (`u16::MAX - 1` usable ids, 1..=65534; id 0 is
        /// reserved as a null sentinel for downstream OCEL64 encodings).
        bound: usize,
    },
}

/// Hard capacity bound: ids run `1..=u16::MAX - 1`, leaving id 0 reserved as
/// a null sentinel for downstream OCEL64 encodings.
const MAX_ENTRIES: usize = u16::MAX as usize - 1;

/// Bridge of two named vocabularies (activities + objects) into a single
/// deterministic `u16` id space each, for OCEL64-style binary encodings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeVocab {
    /// Activity-name id map (ids 1..=n in first-seen order).
    pub activities: IdMap,
    /// Object-type-name id map (ids 1..=n in first-seen order).
    pub objects: IdMap,
}

impl BridgeVocab {
    /// Build both maps, labeling each [`IdMapError`] with its field name and
    /// refusing up front when either field exceeds `MAX_ENTRIES`.
    ///
    /// # Errors
    ///
    /// [`BridgeError::CapacityExceeded`] above `MAX_ENTRIES` and
    /// [`BridgeError::Collision`] on a sentinel collision in either field.
    pub fn build(activity_names: &[&str], object_names: &[&str]) -> Result<Self, BridgeError> {
        // Capacity errors surface as typed refusals with the field label, so
        // a caller can distinguish "corpus too large" from "sentinel
        // collision" without string matching.
        if activity_names.len() > MAX_ENTRIES {
            return Err(BridgeError::CapacityExceeded {
                field: "activities",
                count: activity_names.len(),
                bound: MAX_ENTRIES,
            });
        }
        if object_names.len() > MAX_ENTRIES {
            return Err(BridgeError::CapacityExceeded {
                field: "objects",
                count: object_names.len(),
                bound: MAX_ENTRIES,
            });
        }
        let activities = IdMap::build(activity_names)
            .map_err(|collision| BridgeError::Collision { field: "activities", collision })?;
        let objects = IdMap::build(object_names)
            .map_err(|collision| BridgeError::Collision { field: "objects", collision })?;
        Ok(Self { activities, objects })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 100 deterministic names: `name-000` .. `name-099`.
    fn hundred_names() -> Vec<String> {
        (0..100).map(|i| format!("name-{i:03}")).collect()
    }

    #[test]
    fn golden_build_and_roundtrip() {
        let names: Vec<String> = hundred_names();
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let m = IdMap::build(&refs).expect("100 synthetic names must not collide");
        assert_eq!(m.len(), 100);
        assert!(m.is_bijective());
        for (i, n) in names.iter().enumerate() {
            let id = (i + 1) as u16;
            assert_eq!(m.get(n), Some(id), "forward {n}");
            assert_eq!(m.name(id), Some(n.as_str()), "reverse {id}");
        }
        // Misses on both axes.
        assert_eq!(m.get("name-100"), None);
        assert_eq!(m.name(101), None);
        assert_eq!(m.name(0), None);
    }

    #[test]
    fn ids_are_first_seen_input_order_not_hash_order() {
        let names = ["zeta", "alpha", "mid"];
        let m = IdMap::build(&names).unwrap();
        assert_eq!(m.get("zeta"), Some(1));
        assert_eq!(m.get("alpha"), Some(2));
        assert_eq!(m.get("mid"), Some(3));
    }

    #[test]
    fn duplicate_name_is_refused() {
        let err = IdMap::build(&["dup", "other", "dup"]).unwrap_err();
        match err {
            IdMapError::Collision { index_a, name_a, index_b, name_b } => {
                assert_eq!((index_a, name_a.as_str()), (0, "dup"));
                assert_eq!((index_b, name_b.as_str()), (2, "dup"));
            }
        }
    }

    /// Brute-force an injected collision class: two DISTINCT short generated
    /// strings whose blake3 digests agree on the first 16 bits. 1,000
    /// generated names over a 65,536-bucket space collide with high
    /// probability (birthday: p ≈ 1 - exp(-1000·999/131072) ≈ 99.95%).
    #[test]
    fn build_rejects_injected_sentinel_collision() {
        let generated: Vec<String> = (0..1000).map(|i| format!("c{i}")).collect();
        let mut pair = None;
        'outer: for i in 0..generated.len() {
            for j in (i + 1)..generated.len() {
                if sentinel(generated[i].as_bytes()) == sentinel(generated[j].as_bytes()) {
                    pair = Some((i, j));
                    break 'outer;
                }
            }
        }
        let (i, j) = pair.expect("1000 generated names must contain a 16-bit collision");
        let refs: Vec<&str> = generated.iter().map(|s| s.as_str()).collect();
        let err = IdMap::build(&refs).unwrap_err();
        match err {
            IdMapError::Collision { name_a, name_b, .. } => {
                assert_ne!(name_a, name_b, "must be two distinct strings");
                assert_eq!(name_a, generated[i]);
                assert_eq!(name_b, generated[j]);
            }
        }
        // The colliding pair alone is refused directly, too.
        assert!(matches!(
            IdMap::build(&[generated[i].as_str(), generated[j].as_str()]),
            Err(IdMapError::Collision { .. })
        ));
    }

    #[test]
    fn collision_risk_counts_buckets_and_matches_build() {
        // Clean corpus: zero risk.
        let clean: Vec<String> = hundred_names();
        let refs: Vec<&str> = clean.iter().map(|s| s.as_str()).collect();
        assert_eq!(collision_risk(&refs), 0);
        assert!(IdMap::build(&refs).is_ok());

        // The injected pair from the collision-class test: exactly 1 bucket.
        let generated: Vec<String> = (0..1000).map(|i| format!("c{i}")).collect();
        let mut found = None;
        'outer: for i in 0..generated.len() {
            for j in (i + 1)..generated.len() {
                if sentinel(generated[i].as_bytes()) == sentinel(generated[j].as_bytes()) {
                    found = Some((generated[i].as_str(), generated[j].as_str()));
                    break 'outer;
                }
            }
        }
        let (a, b) = found.expect("collision pair must exist");
        assert_eq!(collision_risk(&[a, b]), 1);
        assert!(IdMap::build(&[a, b]).is_err());
    }

    #[test]
    fn birthday_bound_matches_known_value() {
        // Exact product over the 16-bit space, checked against the documented
        // crossing points.
        let n_buckets = 65536.0f64;
        let p = |n: usize| {
            let mut no_coll = 1.0f64;
            for k in 0..n {
                no_coll *= (n_buckets - k as f64) / n_buckets;
            }
            1.0 - no_coll
        };
        // Documented: ~50% at ~301-302 names.
        assert!(p(301) < 0.5 && p(302) >= 0.5, "p(301)={}, p(302)={}", p(301), p(302));
        // Documented: < 5% under ~83 names.
        assert!(p(82) < 0.05 && p(83) >= 0.05, "p(82)={}, p(83)={}", p(82), p(83));
    }

    #[test]
    fn bridge_vocab_builds_both_fields() {
        let v = BridgeVocab::build(&["receive", "check", "ship"], &["order", "item"])
            .expect("disjoint small vocabs must build");
        assert_eq!(v.activities.get("receive"), Some(1));
        assert_eq!(v.activities.get("ship"), Some(3));
        assert_eq!(v.objects.get("order"), Some(1));
        assert_eq!(v.objects.get("item"), Some(2));
        assert!(v.activities.is_bijective() && v.objects.is_bijective());
    }

    #[test]
    fn bridge_vocab_labels_collision_with_field() {
        let err = BridgeVocab::build(&["ok"], &["dup", "dup"])
            .expect_err("duplicate object names must be refused");
        match err {
            BridgeError::Collision { field, collision } => {
                assert_eq!(field, "objects");
                assert!(matches!(collision, IdMapError::Collision { .. }));
            }
            other => panic!("expected Collision, got {other:?}"),
        }
    }

    #[test]
    fn bridge_vocab_capacity_refusal() {
        // A corpus above the u16 id-space bound is refused with the field
        // label before any sentinel work; use a synthetic bound check via a
        // huge input is impractical in-test, so verify the error shape by
        // exercising the same path with the real bound computed from u16.
        let bound = u16::MAX as usize - 1;
        // Building 65535+ names is cheap enough as bare strings.
        let huge: Vec<String> = (0..=bound).map(|i| format!("x{i:07}")).collect();
        let refs: Vec<&str> = huge.iter().map(|s| s.as_str()).collect();
        let err = BridgeVocab::build(&refs, &[]).expect_err("over-bound corpus must be refused");
        match err {
            BridgeError::CapacityExceeded { field, count, bound: b } => {
                assert_eq!(field, "activities");
                assert_eq!(count, bound + 1);
                assert_eq!(b, bound);
            }
            other => panic!("expected CapacityExceeded, got {other:?}"),
        }
    }

    #[test]
    fn collision_risk_documented_bounds_hold() {
        // Directly probe the real blake3 sentinel space at the documented
        // planning points: <83 names -> 0 collisions on this synthetic corpus
        // (p < 5%), and the risk probe agrees with build's refusal.
        let small: Vec<String> = (0..82).map(|i| format!("s{i:04}")).collect();
        let refs: Vec<&str> = small.iter().map(|s| s.as_str()).collect();
        assert_eq!(collision_risk(&refs), 0);
        assert!(IdMap::build(&refs).is_ok());
    }
}
