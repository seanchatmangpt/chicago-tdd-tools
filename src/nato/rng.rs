#![cfg(feature = "nato")]
//! Deterministic pseudo-randomness for courts.
//!
//! Knuth multiplicative LCG: `state = state * M + C (mod 2^64)`. Seeded
//! from a fixed constant — never from wall-clock, env, or entropy. The
//! same seed always yields the same sequence on every platform.

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

/// Knuth LCG. Deterministic, no wall-clock, no env, no external entropy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lcg {
    state: u64,
}

impl Lcg {
    /// Multiplier.
    pub const M: u64 = 6_364_136_223_846_793_005;
    /// Increment.
    pub const C: u64 = 1_442_695_040_888_963_407;

    /// Create an LCG from a fixed seed. Same seed, same sequence.
    pub const fn new(seed: u64) -> Self {
        Lcg { state: seed }
    }

    /// Advance and return the raw 64-bit state.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(Self::M).wrapping_add(Self::C);
        self.state
    }

    /// Uniform sample in `0..n` via `(next() >> 32) % n`. Returns 0 when
    /// `n == 0` (nothing to sample) — never panics.
    pub fn sample(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        (self.next_u64() >> 32) % n
    }
}
