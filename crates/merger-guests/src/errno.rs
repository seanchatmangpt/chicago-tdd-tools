//! Merger-guest errno table.
//!
//! Codes `0..=13` are the landed `ctt_*` guest semantics, copied verbatim
//! from `src/conformance/guest.rs` (do not renumber — hosts already key on
//! these values). Codes `14..=19` are merger-guest extensions, appended in
//! the next free range so the two families never collide.

/// ok — no error.
pub const OK: i32 = 0;
/// a required pointer/out-pointer argument was null.
pub const ERR_NULL_PTR: i32 = 1;
/// buffer does not start with the expected magic (`MGI1` here, `CTTG` for ctt).
pub const ERR_BAD_MAGIC: i32 = 2;
/// wire version byte is not 1.
pub const ERR_BAD_VERSION: i32 = 3;
/// buffer shorter than its declared layout requires.
pub const ERR_TRUNCATED: i32 = 4;
/// bytes present after the declared payload.
pub const ERR_TRAILING: i32 = 5;
/// a must-be-zero field is non-zero.
pub const ERR_RESERVED_NOT_ZERO: i32 = 6;
/// model DSL failed parse/validate/compile.
pub const ERR_BAD_MODEL: i32 = 7;
/// all `MAX_SESSIONS` slots occupied.
pub const ERR_TABLE_FULL: i32 = 8;
/// handle is free, out of range, or generation-stale.
pub const ERR_BAD_HANDLE: i32 = 9;
/// attach of a zero-length stream.
pub const ERR_STREAM_EMPTY: i32 = 10;
/// out-buffer capacity below the record's fixed length.
pub const ERR_CAP_TOO_SMALL: i32 = 11;
/// sealing requested without the `ledger` feature.
pub const ERR_SEAL_UNAVAILABLE: i32 = 12;
/// stream byte length not a multiple of 8.
pub const ERR_STREAM_ALIGNMENT: i32 = 13;

// --- merger-guest extensions (14..=19) ---

/// scale_tag is not one of the four documented scales.
pub const ERR_BAD_SCALE: i32 = 14;
/// input dimensionality does not match the leaf's required arity.
pub const ERR_DIM: i32 = 15;
/// a fixed-point operation overflowed its i64 value domain.
pub const ERR_OVERFLOW: i32 = 16;
/// reduction quota exhausted before the kernel finished.
pub const ERR_FUEL: i32 = 17;
/// solver found no feasible solution for the given inputs.
pub const ERR_NO_SOLUTION: i32 = 18;
/// input/output element count outside the leaf's documented bounds.
pub const ERR_LENGTH: i32 = 19;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_matches_landed_ctt_semantics() {
        assert_eq!(OK, 0);
        assert_eq!(ERR_NULL_PTR, 1);
        assert_eq!(ERR_BAD_MAGIC, 2);
        assert_eq!(ERR_BAD_VERSION, 3);
        assert_eq!(ERR_TRUNCATED, 4);
        assert_eq!(ERR_TRAILING, 5);
        assert_eq!(ERR_RESERVED_NOT_ZERO, 6);
        assert_eq!(ERR_BAD_MODEL, 7);
        assert_eq!(ERR_TABLE_FULL, 8);
        assert_eq!(ERR_BAD_HANDLE, 9);
        assert_eq!(ERR_STREAM_EMPTY, 10);
        assert_eq!(ERR_CAP_TOO_SMALL, 11);
        assert_eq!(ERR_SEAL_UNAVAILABLE, 12);
        assert_eq!(ERR_STREAM_ALIGNMENT, 13);
    }

    #[test]
    fn extensions_start_after_ctt_range_without_gaps() {
        assert_eq!(ERR_BAD_SCALE, 14);
        assert_eq!(ERR_DIM, 15);
        assert_eq!(ERR_OVERFLOW, 16);
        assert_eq!(ERR_FUEL, 17);
        assert_eq!(ERR_NO_SOLUTION, 18);
        assert_eq!(ERR_LENGTH, 19);
    }
}
