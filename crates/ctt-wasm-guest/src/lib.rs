//! ctt-wasm-guest — WASM guest cdylib for chicago-tdd-tools.
//!
//! This crate is the temprun-hostable artifact: a `cdylib` that re-exports
//! the guest ABI surface of `chicago_tdd_tools::conformance::guest` so the
//! functions are exported from the compiled `.wasm` module, plus an explicit
//! `ctt_guest_abi_version` probe.
//!
//! ## Imports == ∅ law
//!
//! The built module MUST have an empty import section. The parent crate's
//! guest surface is pure computation over caller-provided linear memory
//! (no wasm imports, no host functions, no `wasm-bindgen`), matching the
//! `pn_kernel` precedent: the module imports nothing, so any host that can
//! instantiate it can be proven to host it. A build whose import section is
//! non-empty is a build failure for Gate 0 purposes (check with
//! `wat2wasm`-equivalent tooling or `wasm-objdump -x`).
//!
//! ## Reduction-yield protocol
//!
//! The guest executes at most `quota` reduction steps per ABI call, then
//! returns control to the BEAM scheduler (temprun). It never loops past the
//! quota: each entry does a bounded slice of work and returns a status that
//! tells the host whether the session is `Running`, `Complete`, or `Failed`.
//! This keeps the BEAM scheduler live — a guest call is one reduction
//! window, not a process.
//!
//! ## Panic == trap == fail-closed
//!
//! The parent crate builds without unwinding on wasm32 (`panic = "abort"`
//! profile); a Rust panic becomes a wasm trap. temprun treats a trap as a
//! conformance failure of the session — fail-closed, never fail-open. No
//! guest code path catches panics to convert them into "success" values.
//!
//! ## ABI buffer memory layout
//!
//! All buffers live in guest linear memory at offsets handed over by the
//! host. Each buffer is a `(ptr: u32, len: u32)` pair; `len` is in bytes
//! unless stated. Fields are little-endian, tightly packed, no padding.
//!
//! | Buffer       | Layout (byte offsets)                                     |
//! |--------------|-----------------------------------------------------------|
//! | model        | opaque POWL model bytes produced by the parent's loader   |
//! | session      | opaque ConformanceSession state bytes (parent-owned codec)|
//! | event        | flat LE `u64` OCEL64 event words: bits `[7:0]` activity, bits `[15:8]` tick (per `session.rs`); length must be a multiple of 8 |
//! | report       | out: verdict byte `[0]` (0=Incomplete, 1=Complete, 2=Faulted), remaining bytes: report payload |
//! | seal         | out: BLAKE3 chain receipt bytes (parent `ledger` codec)   |

#![deny(warnings)]

// Re-export the guest ABI surface so `#[no_mangle] extern "C"` functions
// defined in the parent are exported from this cdylib.
pub use chicago_tdd_tools::conformance::guest::*;

// ABI version probe (`ctt_guest_abi_version`, v2) is re-exported from the
// parent's `conformance::guest` module above — do NOT redefine it here;
// a duplicate #[no_mangle] symbol breaks the cdylib link.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abi_version_is_stable_v2() {
        assert_eq!(ctt_guest_abi_version(), 2);
    }
}
