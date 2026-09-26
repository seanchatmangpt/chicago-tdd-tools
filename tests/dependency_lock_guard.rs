//! Dependency-bump conservation guards (PR #104 hardening, v26.9.26).
//!
//! A lockfile bump (dependabot patch group: futures, thiserror, log, blake3,
//! trybuild, syn, which) claims "no behavior change". These tests turn that
//! claim into falsifiable checks against real collaborators only:
//!
//! - the real `Cargo.lock` on disk, parsed with the real `toml` crate
//!   (integrity: no dangling dependency references, no duplicate entries,
//!   bumped packages never silently downgraded — the stale-subject case);
//! - the real `blake3` crate (official known-answer vector) and the real
//!   receipt chain code (golden chain digests pinned across the bump, plus
//!   negative cases: wrong digest, stale predecessor, reordering, duplicate
//!   delivery, empty chain).
//!
//! Every guard has an anti-vacuity twin: a mutated lock text that the same
//! guard must refuse.
//!
//! Run: `cargo test --test dependency_lock_guard --features receipt-validation`

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

// ─── lockfile model + guards ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Pkg {
    name: String,
    version: String,
    source: Option<String>,
}

/// Each package with the dependency reference strings it declares.
type DepRefs = Vec<(Pkg, Vec<String>)>;

/// Parse a Cargo.lock text into (packages, dependency-references per package).
fn parse_lock(text: &str) -> Result<(Vec<Pkg>, DepRefs), String> {
    let value: toml::Value = toml::from_str(text).map_err(|e| format!("MALFORMED_TOML: {e}"))?;
    let packages = value
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "MALFORMED_LOCK: no [[package]] array".to_string())?;
    let mut pkgs = Vec::new();
    let mut deps = Vec::new();
    for p in packages {
        let name = p
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| "MALFORMED_LOCK: package without name".to_string())?;
        let version = p
            .get("version")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| format!("MALFORMED_LOCK: package {name} without version"))?;
        let source = p.get("source").and_then(toml::Value::as_str).map(str::to_string);
        let pkg = Pkg { name: name.to_string(), version: version.to_string(), source };
        let refs = p
            .get("dependencies")
            .and_then(toml::Value::as_array)
            .map(|a| a.iter().filter_map(|d| d.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        pkgs.push(pkg.clone());
        deps.push((pkg, refs));
    }
    Ok((pkgs, deps))
}

/// Integrity guard: duplicate entries and dangling dependency references are refused.
fn check_integrity(text: &str) -> Result<usize, String> {
    let (pkgs, deps) = parse_lock(text)?;
    let mut seen = BTreeSet::new();
    let mut by_name: BTreeMap<&str, Vec<&Pkg>> = BTreeMap::new();
    for p in &pkgs {
        if !seen.insert(p.clone()) {
            return Err(format!("DUPLICATE_ENTRY: {} {}", p.name, p.version));
        }
        by_name.entry(p.name.as_str()).or_default().push(p);
    }
    let mut resolved = 0;
    for (owner, refs) in &deps {
        for r in refs {
            // Forms: "name", "name version", "name version (source)".
            let mut it = r.splitn(3, ' ');
            let name = it.next().unwrap_or_default();
            let version = it.next();
            let candidates = by_name.get(name).map_or(&[][..], Vec::as_slice);
            let matching =
                candidates.iter().filter(|p| version.is_none_or(|v| p.version == v)).count();
            match matching {
                0 => {
                    return Err(format!(
                        "DANGLING_REFERENCE: {} {} -> `{r}`",
                        owner.name, owner.version
                    ))
                }
                1 => resolved += 1,
                // An unqualified name must be unique; a versioned ref may still
                // need its source to disambiguate — accept only if a source is given.
                _ if r.contains('(') => resolved += 1,
                n => {
                    return Err(format!(
                        "AMBIGUOUS_REFERENCE: {} {} -> `{r}` matches {n} packages",
                        owner.name, owner.version
                    ))
                }
            }
        }
    }
    Ok(resolved)
}

/// Numeric (major, minor, patch) with build metadata / pre-release stripped.
fn triple(v: &str) -> (u64, u64, u64) {
    let core = v.split(['+', '-']).next().unwrap_or(v);
    let mut n = core.split('.').map(|x| x.parse::<u64>().unwrap_or(0));
    (n.next().unwrap_or(0), n.next().unwrap_or(0), n.next().unwrap_or(0))
}

/// Same semver-compatibility line (0.x.* pins major+minor, x.*.* pins major).
const fn same_line(a: (u64, u64, u64), b: (u64, u64, u64)) -> bool {
    if a.0 == 0 {
        a.0 == b.0 && a.1 == b.1
    } else {
        a.0 == b.0
    }
}

/// The versions PR #104 claims to resolve. A later bump may raise them; a
/// resolution below any of them on the same compatibility line is a stale
/// (downgraded) subject and is refused.
const BUMP_FLOORS: &[(&str, &str)] = &[
    ("futures", "0.3.34"),
    ("thiserror", "2.0.20"),
    ("log", "0.4.34"),
    ("blake3", "1.8.7"),
    ("trybuild", "1.0.121"),
    ("syn", "3.0.6"),
    ("which", "8.0.6"),
];

fn check_floors(text: &str) -> Result<(), String> {
    let (pkgs, _) = parse_lock(text)?;
    for (name, floor) in BUMP_FLOORS {
        let f = triple(floor);
        let line: Vec<_> = pkgs
            .iter()
            .filter(|p| p.name == *name && same_line(triple(&p.version), f))
            .collect();
        if line.is_empty() {
            return Err(format!("MISSING_BUMPED_PACKAGE: {name} on the {floor} line"));
        }
        for p in line {
            if triple(&p.version) < f {
                return Err(format!("STALE_SUBJECT: {name} resolves {} < {floor}", p.version));
            }
        }
    }
    Ok(())
}

fn real_lock() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ─── positive: the real lockfile ─────────────────────────────────────────────

#[test]
fn real_lock_has_no_dangling_or_duplicate_entries() {
    let resolved = check_integrity(&real_lock()).expect("real Cargo.lock integrity");
    assert!(resolved > 100, "suspiciously few resolved references: {resolved}");
}

#[test]
fn real_lock_resolves_every_bumped_package_at_or_above_its_floor() {
    check_floors(&real_lock()).expect("bump floors");
}

#[test]
fn renamed_crate_leaves_no_reference_to_its_old_name() {
    // The bump replaced `target-triple` with `target-tuple` (trybuild 1.0.121).
    let (pkgs, deps) = parse_lock(&real_lock()).unwrap();
    assert!(pkgs.iter().any(|p| p.name == "target-tuple"), "target-tuple must be locked");
    assert!(!pkgs.iter().any(|p| p.name == "target-triple"), "stale target-triple entry");
    for (owner, refs) in deps {
        assert!(
            !refs.iter().any(|r| r.split(' ').next() == Some("target-triple")),
            "{} still references target-triple",
            owner.name
        );
    }
}

// ─── anti-vacuity: mutated lock texts must be refused ────────────────────────

#[test]
fn malformed_toml_is_refused() {
    let err = check_integrity("[[package]\nname = ").unwrap_err();
    assert!(err.starts_with("MALFORMED_TOML"), "{err}");
}

#[test]
fn lock_without_packages_is_refused() {
    let err = check_integrity("version = 4\n").unwrap_err();
    assert!(err.starts_with("MALFORMED_LOCK"), "{err}");
}

#[test]
fn dangling_reference_to_old_name_is_refused() {
    let mutated = real_lock().replacen("\"target-tuple\"", "\"target-triple\"", 1);
    assert_ne!(mutated, real_lock(), "mutation must apply");
    let err = check_integrity(&mutated).unwrap_err();
    assert!(err.starts_with("DANGLING_REFERENCE"), "{err}");
}

#[test]
fn duplicated_package_entry_is_refused() {
    let lock = real_lock();
    let start = lock.find("[[package]]\nname = \"blake3\"").expect("blake3 entry");
    let end = lock[start + 1..].find("[[package]]").map_or(lock.len(), |i| start + 1 + i);
    let mutated = format!("{lock}\n{}", &lock[start..end]);
    let err = check_integrity(&mutated).unwrap_err();
    assert!(err.starts_with("DUPLICATE_ENTRY: blake3"), "{err}");
}

#[test]
fn downgraded_bumped_package_is_refused_as_stale_subject() {
    let mutated = real_lock().replacen(
        "name = \"blake3\"\nversion = \"1.8.7\"",
        "name = \"blake3\"\nversion = \"1.8.6\"",
        1,
    );
    assert_ne!(mutated, real_lock(), "mutation must apply (is blake3 still 1.8.7?)");
    let err = check_floors(&mutated).unwrap_err();
    assert_eq!(err, "STALE_SUBJECT: blake3 resolves 1.8.6 < 1.8.7");
}

#[test]
fn missing_bumped_package_is_refused() {
    let mutated = real_lock().replace("name = \"which\"", "name = \"which-renamed\"");
    let err = check_floors(&mutated).unwrap_err();
    assert!(err.starts_with("MISSING_BUMPED_PACKAGE: which"), "{err}");
}

#[test]
fn compatibility_lines_are_not_conflated() {
    // syn 2.x coexisting with syn 3.x must not count as a stale syn 3 resolution.
    assert!(!same_line(triple("2.0.117"), triple("3.0.6")));
    assert!(same_line(triple("0.3.34"), triple("0.3.1")));
    assert!(!same_line(triple("0.4.34"), triple("0.3.34")));
    assert_eq!(triple("1.1.3+spec-1.1.0"), (1, 1, 3));
}

// ─── blake3: behavior conservation across the bump ───────────────────────────

fn hex(b: &[u8]) -> String {
    use std::fmt::Write as _;
    b.iter().fold(String::with_capacity(b.len() * 2), |mut s, x| {
        let _ = write!(s, "{x:02x}");
        s
    })
}

#[test]
fn blake3_official_empty_input_vector() {
    // BLAKE3 reference test vector: hash of the empty input. Checked through
    // both the crate's own hex encoder and ours (used for the golden chain).
    let expected = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
    assert_eq!(blake3::hash(b"").to_hex().as_str(), expected);
    assert_eq!(hex(blake3::hash(b"").as_bytes()), expected);
}

#[test]
fn blake3_incremental_equals_one_shot_across_chunk_boundaries() {
    // 1024-byte chunk boundary and multi-chunk tree hashing (SIMD paths).
    for len in [0usize, 1, 63, 64, 65, 1023, 1024, 1025, 2048, 3073, 65_537] {
        let data: Vec<u8> = (0..len).map(|i| u8::try_from(i % 251).unwrap_or(0)).collect();
        let mut h = blake3::Hasher::new();
        for piece in data.chunks(7) {
            h.update(piece);
        }
        assert_eq!(h.finalize(), blake3::hash(&data), "len={len}");
    }
}

#[cfg(feature = "receipt-validation")]
mod receipt_chain {
    use super::hex;
    use chicago_tdd_tools::observability::receipt::{
        Blake3ChainValidator, Blake3ReceiptEntry, ChainError, RawReceiptEntry, ReceiptChainBuilder,
    };

    fn golden_chain() -> Vec<RawReceiptEntry> {
        ReceiptChainBuilder::new()
            .add_entry(1, 0b0001, 0x00)
            .add_entry(2, 0b0011, 0x00)
            .add_entry(3, 0b0111, 0x01)
            .add_entry(4, 0b1111, 0x81)
            .build()
    }

    /// Chain digests pinned before and after the blake3 1.8.6 -> 1.8.7 bump
    /// (both lockfiles produced these exact values). A replay mismatch here
    /// means persisted receipts would no longer verify.
    const GOLDEN: [&str; 4] = [
        "a03a13208251d75d17ae5288d5238f43027d55307080b0d0c719be395a40878d",
        "03deb149a7bceeda340ecebc0ca0710f1d2bc24b61cee8a7d9b0abc339bc063a",
        "99685f6f0677e0e77c68dedab8627057e4cf26b2dc9669f6bf0850fe4fda1337",
        "57bb78b27304660e911750cf6bdd8d34d63535dcfc286ff7050c3dbafa775388",
    ];

    #[test]
    fn golden_chain_digests_are_conserved() {
        let chain = golden_chain();
        let got: Vec<String> = chain.iter().map(|e| hex(&e.stored_hash())).collect();
        assert_eq!(got, GOLDEN, "receipt chain digests drifted");
        Blake3ChainValidator::validate_chain(&chain).expect("golden chain replays");
    }

    #[test]
    fn builder_agrees_with_an_independent_blake3_recomputation() {
        let mut prev = [0u8; 32];
        for e in golden_chain() {
            let mut h = blake3::Hasher::new();
            h.update(&prev);
            h.update(&e.run_id_le);
            h.update(&e.op_trace_le);
            h.update(&[e.topo_tag]);
            let computed = *h.finalize().as_bytes();
            assert_eq!(computed, e.chain_hash);
            prev = computed;
        }
    }

    #[test]
    fn wrong_digest_is_refused_at_its_index() {
        let mut chain = golden_chain();
        chain[3].chain_hash[0] ^= 0x01;
        assert!(matches!(
            Blake3ChainValidator::validate_chain(&chain),
            Err(ChainError::HashMismatch { index: 3, .. })
        ));
    }

    #[test]
    fn stale_predecessor_is_refused() {
        let mut chain = golden_chain();
        chain[2].prev = chain[0].chain_hash;
        assert!(matches!(
            Blake3ChainValidator::validate_chain(&chain),
            Err(ChainError::PrevHashMismatch { index: 2, .. })
        ));
    }

    #[test]
    fn reordered_delivery_is_refused() {
        let mut chain = golden_chain();
        chain.swap(1, 2);
        assert!(matches!(
            Blake3ChainValidator::validate_chain(&chain),
            Err(ChainError::PrevHashMismatch { index: 1, .. })
        ));
    }

    #[test]
    fn duplicate_delivery_is_refused() {
        let mut chain = golden_chain();
        let dup = chain[1].clone();
        chain.insert(2, dup);
        assert!(matches!(
            Blake3ChainValidator::validate_chain(&chain),
            Err(ChainError::PrevHashMismatch { index: 2, .. })
        ));
    }

    #[test]
    fn empty_chain_is_refused() {
        let empty: Vec<RawReceiptEntry> = Vec::new();
        assert_eq!(Blake3ChainValidator::validate_chain(&empty), Err(ChainError::Empty));
    }

    #[test]
    fn replay_pointer_is_outside_the_digest() {
        let mut chain = golden_chain();
        chain[1].replay_ptr_bytes = [0xFF; 8];
        Blake3ChainValidator::validate_chain(&chain).expect("replay_ptr is not hashed");
    }

    /// Regression bound (debug build, generous): validating a 10k-entry chain
    /// must stay well under 2 s. Criterion numbers live in
    /// `benches/receipt_chain.rs` and `benches/receipts/`.
    #[test]
    fn ten_thousand_entry_chain_validates_within_bound() {
        let mut b = ReceiptChainBuilder::new();
        for i in 0..10_000u64 {
            b = b.add_entry(i, i.wrapping_mul(0x9E37_79B9_7F4A_7C15), i.to_le_bytes()[0]);
        }
        let chain = b.build();
        let t = std::time::Instant::now();
        Blake3ChainValidator::validate_chain(&chain).expect("10k chain valid");
        let elapsed = t.elapsed();
        assert!(elapsed.as_millis() < 2000, "10k-entry validation took {elapsed:?}");
    }
}
