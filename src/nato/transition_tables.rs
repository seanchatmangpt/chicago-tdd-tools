#![cfg(feature = "nato")]
//! RONR + DAO transition tables as pure data (NATO lane N2 substrate).
//!
//! Verbatim port of the `(name, pre, post)` tables from strata-protocol
//! `src/ronr.rs` + `src/dao_transitions.rs` (strata fix/kg-42-status-code,
//! Apache/MIT per strata LICENSE). Names live only in this data table; the
//! runtime surface is pure `u16` masks, zero strings on hot paths.
//!
//! # Place-bit maps (documented bits)
//!
//! RONR (15 documented bits, `0x8000` reserved):
//! `0x0001` MAIN_MOTION, `0x0002` STATED, `0x0004` SECONDED,
//! `0x0008` DEBATABLE, `0x0010` AMENDMENT_PENDING, `0x0020` PREV_Q_MOVED,
//! `0x0040` PREV_Q_ORDERED, `0x0080` FLOOR, `0x0100` ADOPTED,
//! `0x0200` TABLED, `0x0400` POSTPONED_INDEF, `0x0800` RECESS_PENDING,
//! `0x1000` ADJOURNED, `0x2000` POINT_OF_ORDER_RAISED, `0x4000` DIVISION.
//!
//! DAO (14 documented bits; `0x0040` MEMBER_SUSPENDED is documented but
//! unconsumed by any verb, `0x0080`/`0x0200`/`0x8000` reserved):
//! `0x0001` COLLATERAL_STAKED, `0x0002` STREAM_OPEN, `0x0004` ESCROW_LOCKED,
//! `0x0008` DELEGATED, `0x0010` LEASE_HELD, `0x0020` MEMBER_ACTIVE,
//! `0x0040` MEMBER_SUSPENDED, `0x0100` TREASURY_SOLVENT,
//! `0x0400` ESCROW_RELEASED, `0x0800` DELEGATION_PENDING,
//! `0x1000` COLLATERAL_WITHDRAWN, `0x2000` STREAM_HALTED,
//! `0x4000` MEMBER_EXPELLED.
//!
//! Zero ambient reads, zero unwraps, zero panics — pure data.

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

/// One transition shape: a stable name plus its `(pre, post)` place masks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    /// Stable machine name (snake_case verb).
    pub name: &'static str,
    /// Precondition place mask (places the transition consumes).
    pub pre: u16,
    /// Postcondition place mask (places the transition produces).
    pub post: u16,
}

/// Robert's Rules (RONR) place bits, verbatim from strata `ronr.rs`.
pub mod ronr {
    use super::Shape;

    /// Class flag: a main motion is the subject of the assembly.
    pub const N_MAIN_MOTION: u16 = 0x0001;
    /// Chair has stated the pending question.
    pub const N_STATED: u16 = 0x0002;
    /// Motion has been seconded.
    pub const N_SECONDED: u16 = 0x0004;
    /// Floor is open for debate.
    pub const N_DEBATABLE: u16 = 0x0008;
    /// A primary amendment is the immediately pending question.
    pub const N_AMENDMENT_PENDING: u16 = 0x0010;
    /// Previous question (call for the question) has been moved.
    pub const N_PREV_QUESTION_MOVED: u16 = 0x0020;
    /// Previous question ordered: debate is cut off.
    pub const N_PREV_QUESTION_ORDERED: u16 = 0x0040;
    /// A member is recognized by the chair.
    pub const N_FLOOR: u16 = 0x0080;
    /// Motion adopted by vote.
    pub const N_ADOPTED: u16 = 0x0100;
    /// Motion laid on the table.
    pub const N_TABLED: u16 = 0x0200;
    /// Motion postponed indefinitely (effectively killed).
    pub const N_POSTPONED_INDEF: u16 = 0x0400;
    /// Recess is the pending question.
    pub const N_RECESS_PENDING: u16 = 0x0800;
    /// Assembly adjourned.
    pub const N_ADJOURNED: u16 = 0x1000;
    /// Point of order raised; chair ruling pending.
    pub const N_POINT_OF_ORDER_RAISED: u16 = 0x2000;
    /// Division of the assembly demanded.
    pub const N_DIVISION_DEMANDED: u16 = 0x4000;

    /// Canonical RONR transitions (opcodes 0x01-0x0C), verbatim masks.
    pub static TRANSITIONS: [Shape; 12] = [
        Shape { name: "ronr_main_motion_stated", pre: 0x0081, post: 0x0082 },
        Shape { name: "ronr_second", pre: 0x0002, post: 0x000C },
        Shape { name: "ronr_amend", pre: 0x0008, post: 0x0010 },
        Shape { name: "ronr_adopt_amendment", pre: 0x0010, post: 0x0008 },
        Shape { name: "ronr_previous_question", pre: 0x0008, post: 0x0060 },
        Shape { name: "ronr_adopt_main_motion", pre: 0x0024, post: 0x0101 },
        Shape { name: "ronr_lay_on_table", pre: 0x000C, post: 0x0200 },
        Shape { name: "ronr_postpone_indefinitely", pre: 0x000C, post: 0x0400 },
        Shape { name: "ronr_recess", pre: 0x0008, post: 0x0800 },
        Shape { name: "ronr_adjourn", pre: 0x0008, post: 0x1000 },
        Shape { name: "ronr_point_of_order", pre: 0x0008, post: 0x2000 },
        Shape { name: "ronr_division", pre: 0x0000, post: 0x4000 },
    ];
}

/// DAO capital place bits, verbatim from strata `dao_transitions.rs`.
pub mod dao {
    use super::Shape;

    /// Collateral locked in the collateral pool.
    pub const P_COLLATERAL_STAKED: u16 = 0x0001;
    /// A payment stream is actively flowing.
    pub const P_STREAM_OPEN: u16 = 0x0002;
    /// Escrow holds locked funds.
    pub const P_ESCROW_LOCKED: u16 = 0x0004;
    /// Voting weight delegated.
    pub const P_DELEGATED: u16 = 0x0008;
    /// An execution lease is held.
    pub const P_LEASE_HELD: u16 = 0x0010;
    /// Membership active.
    pub const P_MEMBER_ACTIVE: u16 = 0x0020;
    /// Membership suspended (documented but unconsumed by any verb).
    pub const P_MEMBER_SUSPENDED: u16 = 0x0040;
    /// Treasury solvency proven for this scope.
    pub const P_TREASURY_SOLVENT: u16 = 0x0100;
    /// Escrow released without dispute.
    pub const P_ESCROW_RELEASED: u16 = 0x0400;
    /// Delegation awaiting acknowledgement.
    pub const P_DELEGATION_PENDING: u16 = 0x0800;
    /// Collateral returned to the member.
    pub const P_COLLATERAL_WITHDRAWN: u16 = 0x1000;
    /// Stream halted, settlement pending.
    pub const P_STREAM_HALTED: u16 = 0x2000;
    /// Membership terminated by expulsion.
    pub const P_MEMBER_EXPELLED: u16 = 0x4000;

    /// Canonical DAO transitions (13 verbs), verbatim masks.
    pub static TRANSITIONS: [Shape; 13] = [
        Shape { name: "dao_deposit_collateral", pre: 0x0000, post: 0x0101 },
        Shape { name: "dao_withdraw_collateral", pre: 0x0001, post: 0x1000 },
        Shape { name: "dao_stream", pre: 0x0101, post: 0x0103 },
        Shape { name: "dao_halt_stream", pre: 0x0002, post: 0x2000 },
        Shape { name: "dao_ragequit", pre: 0x0021, post: 0x1000 },
        Shape { name: "dao_lock_escrow", pre: 0x0020, post: 0x0024 },
        Shape { name: "dao_release_escrow", pre: 0x0004, post: 0x0020 },
        Shape { name: "dao_delegate", pre: 0x0020, post: 0x0028 },
        Shape { name: "dao_undelegate", pre: 0x0008, post: 0x0020 },
        Shape { name: "dao_acquire_lease", pre: 0x0020, post: 0x0030 },
        Shape { name: "dao_expire_lease", pre: 0x0010, post: 0x0020 },
        Shape { name: "dao_admit_member", pre: 0x0000, post: 0x0020 },
        Shape { name: "dao_expel_member", pre: 0x0020, post: 0x4000 },
    ];
}

/// One shape entry bound to its table family (`"ronr"` or `"dao"`).
pub type FamilyShape = (&'static str, Shape);

/// All 25 shapes in canonical order (RONR block then DAO block).
///
/// Verbatim mask correspondence with strata: RONR opcodes 0x01-0x0C in
/// table order, DAO verbs 0x70-0x97 in table order.
pub fn shapes() -> Vec<FamilyShape> {
    let mut v = Vec::with_capacity(25);
    for t in ronr::TRANSITIONS.iter() {
        v.push(("ronr", *t));
    }
    for t in dao::TRANSITIONS.iter() {
        v.push(("dao", *t));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
    fn table_sizes_match_strata() {
        assert_eq!(ronr::TRANSITIONS.len(), 12, "RONR table must carry 12 transitions");
        assert_eq!(dao::TRANSITIONS.len(), 13, "DAO table must carry 13 transitions");
        assert_eq!(shapes().len(), 25, "combined table must carry 12 + 13 shapes");
    }

    #[test]
    #[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
    fn ronr_masks_match_strata_verbatim() {
        let want: [(&str, u16, u16); 12] = [
            ("ronr_main_motion_stated", 0x0081, 0x0082),
            ("ronr_second", 0x0002, 0x000C),
            ("ronr_amend", 0x0008, 0x0010),
            ("ronr_adopt_amendment", 0x0010, 0x0008),
            ("ronr_previous_question", 0x0008, 0x0060),
            ("ronr_adopt_main_motion", 0x0024, 0x0101),
            ("ronr_lay_on_table", 0x000C, 0x0200),
            ("ronr_postpone_indefinitely", 0x000C, 0x0400),
            ("ronr_recess", 0x0008, 0x0800),
            ("ronr_adjourn", 0x0008, 0x1000),
            ("ronr_point_of_order", 0x0008, 0x2000),
            ("ronr_division", 0x0000, 0x4000),
        ];
        for (shape, (name, pre, post)) in ronr::TRANSITIONS.iter().zip(want.iter()) {
            assert_eq!(shape.name, *name, "RONR table order drifted");
            assert_eq!(shape.pre, *pre, "pre mask drifted for {name}");
            assert_eq!(shape.post, *post, "post mask drifted for {name}");
        }
        // Spot-check the named constants agree with the masks in the table.
        assert_eq!(ronr::TRANSITIONS[0].pre, ronr::N_MAIN_MOTION | ronr::N_FLOOR);
        assert_eq!(ronr::TRANSITIONS[5].post, ronr::N_MAIN_MOTION | ronr::N_ADOPTED);
        assert_eq!(ronr::TRANSITIONS[11].post, ronr::N_DIVISION_DEMANDED);
        assert_eq!(ronr::TRANSITIONS[0].name.len(), 23);
    }

    #[test]
    #[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
    fn dao_masks_match_strata_verbatim() {
        let want: [(&str, u16, u16); 13] = [
            ("dao_deposit_collateral", 0x0000, dao::P_COLLATERAL_STAKED | dao::P_TREASURY_SOLVENT),
            ("dao_withdraw_collateral", dao::P_COLLATERAL_STAKED, dao::P_COLLATERAL_WITHDRAWN),
            (
                "dao_stream",
                dao::P_COLLATERAL_STAKED | dao::P_TREASURY_SOLVENT,
                dao::P_STREAM_OPEN | dao::P_COLLATERAL_STAKED | dao::P_TREASURY_SOLVENT,
            ),
            ("dao_halt_stream", dao::P_STREAM_OPEN, dao::P_STREAM_HALTED),
            (
                "dao_ragequit",
                dao::P_MEMBER_ACTIVE | dao::P_COLLATERAL_STAKED,
                dao::P_COLLATERAL_WITHDRAWN,
            ),
            ("dao_lock_escrow", dao::P_MEMBER_ACTIVE, dao::P_MEMBER_ACTIVE | dao::P_ESCROW_LOCKED),
            ("dao_release_escrow", dao::P_ESCROW_LOCKED, dao::P_MEMBER_ACTIVE),
            ("dao_delegate", dao::P_MEMBER_ACTIVE, dao::P_MEMBER_ACTIVE | dao::P_DELEGATED),
            ("dao_undelegate", dao::P_DELEGATED, dao::P_MEMBER_ACTIVE),
            ("dao_acquire_lease", dao::P_MEMBER_ACTIVE, dao::P_MEMBER_ACTIVE | dao::P_LEASE_HELD),
            ("dao_expire_lease", dao::P_LEASE_HELD, dao::P_MEMBER_ACTIVE),
            ("dao_admit_member", 0x0000, dao::P_MEMBER_ACTIVE),
            ("dao_expel_member", dao::P_MEMBER_ACTIVE, dao::P_MEMBER_EXPELLED),
        ];
        for (shape, (name, pre, post)) in dao::TRANSITIONS.iter().zip(want.iter()) {
            assert_eq!(shape.name, *name, "DAO table order drifted");
            assert_eq!(shape.pre, *pre, "pre mask drifted for {name}");
            assert_eq!(shape.post, *post, "post mask drifted for {name}");
        }
        // The two documented self-start entries are the only pre == 0 rows.
        let self_starts: Vec<&str> =
            dao::TRANSITIONS.iter().filter(|t| t.pre == 0).map(|t| t.name).collect();
        assert_eq!(
            self_starts,
            vec!["dao_deposit_collateral", "dao_admit_member"],
            "self-start census drifted"
        );
    }

    #[test]
    fn every_transition_fires_from_its_pre_marking() {
        // m' = (pre ^ pre) | post for the canonical enabled marking; uses the
        // crate kernel so the data tables and the kernel agree.
        use crate::conformance::pn64;
        for (family, t) in shapes() {
            let w = pn64::pack(t.pre, t.pre, t.post, 0, 0);
            let fired = pn64::fire(w);
            let expected = (t.pre ^ t.pre) | t.post;
            assert!(
                fired.is_some_and(|f| pn64::marking(f) == expected),
                "{family} transition {} does not fire from its pre-marking",
                t.name
            );
        }
    }
}
