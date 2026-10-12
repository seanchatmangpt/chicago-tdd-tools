//! merger-guests — merger-matrix leaf kernels behind one uniform stateless ABI.
//!
//! This crate is the merger-matrix sibling of `ctt-wasm-guest`: a `cdylib`
//! whose exported symbols are the `mgi_*` entry points, one per leaf kernel.
//! Every leaf conforms to a single stateless ABI shape:
//!
//! ```text
//! i32 mgi_<leaf>_compute(in: *const u8, in_len: u32,
//!                        out: *mut u8, out_cap: u32) -> i32
//! ```
//!
//! `0` = ok; **positive** = guest errno (see [`errno`], table 0-19);
//! **negative** = reserved for the host transport, never returned by a leaf.
//! There is no session table, no handle, no global state: each call is a pure
//! function of its input buffer, so any host that can instantiate the module
//! can replay any call byte-for-byte (reduction-yield and fail-closed panic
//! semantics inherited from the `ctt-wasm-guest` precedent).
//!
//! ## Zero-import module law
//!
//! The built module MUST have an empty import section. The leaf kernels are
//! pure computation over caller-provided linear memory: no wasm imports, no
//! host functions, no `wasm-bindgen`. Leaf modules use `core`+`alloc` only
//! (no `std`), and this crate consumes the parent
//! (`chicago-tdd-tools`) with `default-features = false, features =
//! ["logging", "conformance"]` (conformance is an empty additive feature
//! gating the core-only pn64 kernel re-exported by `pn64_audit`) —
//! no uuid-v4, no async, no otel — so nothing drags an import
//! into the module. A build whose import section is non-empty is a Gate-0
//! build failure (check with `wasm-objdump -x`).
//!
//! ## Leaf module lanes
//!
//! The eight `extern "C"` leaf entry points are defined in sibling leaf
//! modules and re-exported here so they are exported from the compiled
//! cdylib (same pattern as `ctt-wasm-guest` re-exporting the parent's guest
//! surface). The `pub use` lines below land with the leaf lanes:
//!
//! - **MG2**: 2 leaf kernels (ids 1-2: hhi_calc, netting_engine)
//! - **MG3**: 2 leaf kernels (ids 3-4: simm_matrix, collateral_lp); **MG4** carries ids 7-10 (vote_reconciler, netting_settler, fractional_shares, pn64_audit) — see the ten-slot table below
//!
//! Each leaf module MUST define its entry points as
//! `#[no_mangle] pub extern "C" fn mgi_<leaf>_compute(...)` matching the
//! uniform signature above, and this file re-exports them unmodified —
//! a duplicate `#[no_mangle]` symbol here would break the cdylib link.
//!
//! ```text
//! // After MG2/MG3 land, uncomment (module names per the landing lanes):
//! // pub use crate::{alpha::*, beta::*}; // MG2 leaves
//! // pub use crate::{gamma::*, delta::*}; // MG3 leaves
//! ```
//!
//! ## MGI1 header (16 bytes, little-endian, tightly packed, no padding)
//!
//! Every `mgi_*_compute` input buffer starts with this header; every output
//! buffer written by a leaf starts with it too (echoing the computed
//! leaf/scale).
//!
//! | Offset | Size | Field | Law |
//! |--------|------|-------|-----|
//! | 0      | 4    | `magic` | ASCII `MGI1`; anything else is `ERR_BAD_MAGIC` |
//! | 4      | 1    | `leaf_id` | leaf kernel identity, stable across releases; a leaf must refuse a header naming a different leaf (`ERR_BAD_HANDLE`-class mismatch → `ERR_BAD_MAGIC`) |
//! | 5      | 1    | `scale_tag` | one of [`fixed::TAG_MONEY`]/[`fixed::TAG_RATIO`]/[`fixed::TAG_HHI`]/[`fixed::TAG_SHARES`]; anything else is `ERR_BAD_SCALE` |
//! | 6      | 2    | `reserved` | must be zero (`ERR_RESERVED_NOT_ZERO`) |
//! | 8      | 8    | `dims` | `u64` element count of the payload that follows; `ERR_TRUNCATED` when `in_len < 16 + dims * elem_size` for the leaf's element size, `ERR_LENGTH` when `dims` is outside the leaf's documented bounds |
//!
//! Parse law: verify magic → version-consistent layout → reserved-zero →
//! scale → dims bounds, in that order, returning the first failing errno.
//! Encode law: the [`Mgi1Header::encode`] writer emits exactly 16 bytes and
//! is the inverse of [`parse_header`] on well-formed input.

#![deny(warnings)]

pub mod errno;
pub mod fixed;

/// ABI revision reported by [`mgi_abi_version`] (v1).
pub const MGI_ABI_VERSION: u32 = 1;

/// Magic bytes `MGI1`.
pub const MGI1_MAGIC: [u8; 4] = *b"MGI1";

/// 16-byte MGI1 header (see crate docs for the field law).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mgi1Header {
    /// Leaf kernel identity (stable across releases).
    pub leaf_id: u8,
    /// Fixed-point scale tag ([`fixed`] documented set).
    pub scale_tag: u8,
    /// Element count of the payload that follows.
    pub dims: u64,
}

impl Mgi1Header {
    /// Header size in bytes.
    pub const SIZE: usize = 16;

    /// Encode into the front of `buf` (exactly 16 bytes; panics-in-test /
    /// truncates-by-contract only if `buf.len() < 16` — callers pass
    /// `out_cap >= 16`, enforced at the ABI layer by `ERR_CAP_TOO_SMALL`).
    pub fn encode(&self, buf: &mut [u8]) {
        let b = &mut buf[..Self::SIZE];
        b[0..4].copy_from_slice(&MGI1_MAGIC);
        b[4] = self.leaf_id;
        b[5] = self.scale_tag;
        b[6..8].copy_from_slice(&0u16.to_le_bytes());
        b[8..16].copy_from_slice(&self.dims.to_le_bytes());
    }

    /// Byte encoding.
    // JUSTIFICATION: `to_bytes` keeps `&self` (a cheap fixed-size copy out of
    // the header, matching the C-style guest ABI surface); `self` by value
    // would force a move at every wire-encode call site.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut b = [0u8; Self::SIZE];
        self.encode(&mut b);
        b
    }
}

/// Parse-law errno outcome for [`parse_header`].
pub type HeaderResult<T> = Result<T, i32>;

/// Parse a 16-byte MGI1 header from the front of `buf`, applying the
/// crate-doc parse law in order: magic → reserved → scale. `leaf_id`
/// agreement and `dims` bounds are leaf-specific and checked by the leaf
/// after this returns.
pub fn parse_header(buf: &[u8]) -> HeaderResult<Mgi1Header> {
    if buf.len() < Mgi1Header::SIZE {
        return Err(errno::ERR_TRUNCATED);
    }
    if buf[0..4] != MGI1_MAGIC {
        return Err(errno::ERR_BAD_MAGIC);
    }
    if buf[6..8] != [0, 0] {
        return Err(errno::ERR_RESERVED_NOT_ZERO);
    }
    let leaf_id = buf[4];
    let scale_tag = buf[5];
    if fixed::scale_of(scale_tag).is_none() {
        return Err(errno::ERR_BAD_SCALE);
    }
    let dims = u64::from_le_bytes(buf[8..16].try_into().map_err(|_| errno::ERR_TRUNCATED)?);
    Ok(Mgi1Header { leaf_id, scale_tag, dims })
}

/// ABI version probe. Stable at v1 for the life of the MGI1 header format.
#[no_mangle]
pub extern "C" fn mgi_abi_version() -> u32 {
    MGI_ABI_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abi_version_is_stable_v1() {
        assert_eq!(mgi_abi_version(), 1);
    }

    #[test]
    fn header_roundtrip_is_inverse() {
        let h = Mgi1Header { leaf_id: 3, scale_tag: fixed::TAG_HHI, dims: 42 };
        let bytes = h.to_bytes();
        assert_eq!(bytes.len(), 16);
        assert_eq!(&bytes[0..4], b"MGI1");
        assert_eq!(parse_header(&bytes).unwrap(), h);
    }

    #[test]
    fn header_parse_law_order() {
        // truncated
        assert_eq!(parse_header(&[0u8; 15]).unwrap_err(), errno::ERR_TRUNCATED);
        // bad magic
        let mut b = [0u8; 16];
        b[0..4].copy_from_slice(b"CTTG");
        assert_eq!(parse_header(&b).unwrap_err(), errno::ERR_BAD_MAGIC);
        // reserved non-zero
        let mut b = [0u8; 16];
        b[0..4].copy_from_slice(b"MGI1");
        b[6] = 1;
        assert_eq!(parse_header(&b).unwrap_err(), errno::ERR_RESERVED_NOT_ZERO);
        // reserved checked before scale
        let mut b = [0u8; 16];
        b[0..4].copy_from_slice(b"MGI1");
        b[5] = 99;
        b[6] = 1;
        assert_eq!(parse_header(&b).unwrap_err(), errno::ERR_RESERVED_NOT_ZERO);
        // bad scale
        let mut b = [0u8; 16];
        b[0..4].copy_from_slice(b"MGI1");
        b[5] = 99;
        assert_eq!(parse_header(&b).unwrap_err(), errno::ERR_BAD_SCALE);
    }

    #[test]
    fn header_dims_decode_little_endian() {
        let mut b = [0u8; 16];
        b[0..4].copy_from_slice(b"MGI1");
        b[5] = fixed::TAG_MONEY; // MX8 repair: scale_tag 0 is ERR_BAD_SCALE
        b[8..16].copy_from_slice(&0xDEAD_BEEF_u64.to_le_bytes());
        assert_eq!(parse_header(&b).unwrap().dims, 0xDEAD_BEEF);
    }

    #[test]
    fn zero_import_law_is_documented_dependency_set() {
        // Chicago check on the real manifest: the dependency set must stay
        // exactly the proven minimal set (no uuid/async/otel features that
        // would drag imports into the wasm module).
        let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
            .expect("read Cargo.toml");
        assert!(
            manifest.contains("default-features = false"),
            "parent dep must disable default features"
        );
        assert!(manifest.contains("\"logging\""), "logging feature required");
        for banned in ["uuid", "wasm-bindgen", "tokio", "async", "otel"] {
            assert!(
                !manifest.contains(banned),
                "banned dependency token {banned:?} in Cargo.toml — zero-import law"
            );
        }
    }
}

// ## Leaf-id table (MX8 wiring, 10 slots: 8 landed + 5/6 reserved)
//
// | leaf_id | module              | entry point                     | lane |
// |---------|---------------------|---------------------------------|------|
// | 1       | hhi_calc            | mgi_hhi_calc_compute            | MG2  |
// | 2       | netting_engine      | mgi_netting_engine_compute      | MG2  |
// | 3       | simm_matrix         | mgi_simm_matrix_compute         | MG3  |
// | 4       | collateral_lp       | mgi_collateral_lp_compute       | MG3  |
// | 5       | — reserved —        | —                               | —    |
// | 6       | — reserved —        | —                               | —    |
// | 7       | vote_reconciler     | mgi_vote_reconciler_compute     | MG4  |
// | 8       | netting_settler     | mgi_netting_settler_compute     | MG4  |
// | 9       | fractional_shares   | mgi_fractional_shares_compute   | MG4  |
// | 10      | pn64_audit          | mgi_pn64_audit_compute          | MG4  |
//
// Leaf-id note resolved: MG3 landed 3-4 (simm/collateral) and MG4 chose
// 7-10 (vote/settler/fractional/pn64); MG1's draft slots 5/6 are RESERVED —
// not reassigned — so the landed lane choices stand unchanged and the
// `LEAF_ID_*` constants are the single authoritative assignment.

pub mod collateral_lp;
pub mod fractional_shares;
pub mod hhi_calc;
pub mod netting_engine;
pub mod netting_settler;
pub mod pn64_audit;
pub mod simm_matrix;
pub mod vote_reconciler;

/// Re-export the eight `mgi_*` extern entry points at the crate root so they
/// are exported from the compiled cdylib under their uniform `mgi_*` names
/// (MG1's slots, landed; the leaf modules own the `#[no_mangle]` definitions —
/// re-export here adds no duplicate symbol).
pub use crate::{
    collateral_lp::mgi_collateral_lp_compute, fractional_shares::mgi_fractional_shares_compute,
    hhi_calc::mgi_hhi_calc_compute, netting_engine::mgi_netting_engine_compute,
    netting_settler::mgi_netting_settler_compute, pn64_audit::mgi_pn64_audit_compute,
    simm_matrix::mgi_simm_matrix_compute, vote_reconciler::mgi_vote_reconciler_compute,
};

/// Stable leaf identities, re-exported alongside the leaf table.
pub use crate::{
    collateral_lp::LEAF_ID_COLLATERAL_LP, fractional_shares::LEAF_ID_FRACTIONAL_SHARES,
    hhi_calc::LEAF_ID_HHI_CALC, netting_engine::LEAF_ID_NETTING_ENGINE,
    netting_settler::LEAF_ID_NETTING_SETTLER, pn64_audit::LEAF_ID_PN64_AUDIT,
    simm_matrix::LEAF_ID_SIMM_MATRIX, vote_reconciler::LEAF_ID_VOTE_RECONCILER,
};

#[cfg(test)]
mod leaf_table_tests {
    use super::*;

    /// The leaf-id table is 10 slots: landed ids 1-4 and 7-10, no
    /// collisions; 5/6 are reserved (absent).
    #[test]
    fn leaf_ids_match_the_documented_ten_slot_table() {
        let ids = [
            LEAF_ID_HHI_CALC,
            LEAF_ID_NETTING_ENGINE,
            LEAF_ID_SIMM_MATRIX,
            LEAF_ID_COLLATERAL_LP,
            LEAF_ID_VOTE_RECONCILER,
            LEAF_ID_NETTING_SETTLER,
            LEAF_ID_FRACTIONAL_SHARES,
            LEAF_ID_PN64_AUDIT,
        ];
        let mut sorted = ids;
        sorted.sort_unstable();
        assert_eq!(sorted, [1, 2, 3, 4, 7, 8, 9, 10]);
        assert!(sorted.windows(2).all(|w| w[0] < w[1]), "no collisions");
    }
}
