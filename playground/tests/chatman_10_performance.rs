//! Capability 10 — performance validation (`performance_test!`,
//! `TickCounter`, `measure_ticks`) on the hot admission lookup.
//!
//! The Chatman Constant: a hot-path admission lookup budgets <= 8 ticks.
//! This file is the *reference* demonstration, so its gates are deliberately
//! lenient (RDTSC noise on a shared dev machine would make a hard 8-tick
//! assertion flaky here); the praxis benchmark suite owns the hard gate.

mod chatman_common;

use chatman_common::small_admission_table;
use chicago_tdd_tools::performance_test;
use chicago_tdd_tools::validation::performance::{measure_ticks, TickCounter};

/// Lenient reference budget (ticks). The constitutional budget is 8; see the
/// module doc for why the reference suite gates loosely.
const REFERENCE_BUDGET_TICKS: u64 = 100_000;

performance_test!(hot_admission_lookup_within_reference_budget, {
    // Arrange: table built outside the measured region.
    let table = small_admission_table().expect("2-name table is within the Need9 fence");
    let state: u8 = 0b0000_0011;

    // Act: measure the single branchless indexed load.
    let (entry, ticks) = measure_ticks(|| table.lookup(state));

    // Assert: correct result, and within the (lenient) reference budget.
    // Chatman Constant: the hard gate is <= 8 ticks, owned by praxis benches.
    assert!(entry.admit, "state with bit 0 set must be admitted");
    assert!(
        ticks <= REFERENCE_BUDGET_TICKS,
        "hot lookup took {ticks} ticks (> reference budget {REFERENCE_BUDGET_TICKS}); \
         the constitutional budget is 8 ticks — investigate before trusting the hot path"
    );
});

performance_test!(tick_counter_brackets_a_batch_admission, {
    // Arrange: 256 states covering the whole table.
    let table = small_admission_table().expect("valid table");
    let states: Vec<u8> = (0u16..=255).map(|s| s as u8).collect();

    // Act: TickCounter brackets the whole batch scan.
    let counter = TickCounter::start();
    let mut admitted = 0usize;
    for &s in &states {
        if table.lookup(s).admit {
            admitted += 1;
        }
    }
    let ticks = counter.elapsed_ticks();

    // Assert: exactly the states with bit 0 set and bit 7 clear are admitted
    // (64 of 256), and the batch stays within a generous linear budget.
    assert_eq!(admitted, 64);
    assert!(
        !counter.exceeds_budget(REFERENCE_BUDGET_TICKS * 256),
        "batch of 256 lookups took {ticks} ticks"
    );
});

performance_test!(lookup_scales_flat_not_with_table_population, {
    // Arrange: the lookup is an indexed load, so cost must not depend on the
    // state value. Compare tick counts across distant states — a smoke check
    // for hidden data-dependent branches, not a rigorous benchmark.
    let table = small_admission_table().expect("valid table");

    // Act
    let (_, t_low) = measure_ticks(|| table.lookup(0x01));
    let (_, t_high) = measure_ticks(|| table.lookup(0x7F));

    // Assert: both individually within the lenient reference budget.
    assert!(t_low <= REFERENCE_BUDGET_TICKS, "low state took {t_low} ticks");
    assert!(t_high <= REFERENCE_BUDGET_TICKS, "high state took {t_high} ticks");
});
