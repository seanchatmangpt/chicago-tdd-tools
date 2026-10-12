//! Receipt-chain throughput benchmark (PR #104 hardening, v26.9.26).
//!
//! Measures the BLAKE3 receipt chain (build + validate) that the blake3
//! 1.8.6 -> 1.8.7 bump sits under. Recorded numbers and the regression bound
//! live in `benches/receipts/receipt_chain-v26.9.26.json`; the bound is also
//! enforced as a test in `tests/dependency_lock_guard.rs`.
//!
//! Run: `cargo bench --bench receipt_chain --features receipt-validation`

// criterion_group! expands to an undocumented pub fn.
#![allow(missing_docs)]

use chicago_tdd_tools::observability::receipt::{
    Blake3ChainValidator, RawReceiptEntry, ReceiptChainBuilder,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn chain(n: u64) -> Vec<RawReceiptEntry> {
    let mut b = ReceiptChainBuilder::new();
    for i in 0..n {
        b = b.add_entry(i, i.wrapping_mul(0x9E37_79B9_7F4A_7C15), i.to_le_bytes()[0]);
    }
    b.build()
}

fn bench_receipt_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_chain");
    for n in [16u64, 1_024, 10_000] {
        group.throughput(Throughput::Elements(n));
        let entries = chain(n);
        group.bench_with_input(BenchmarkId::new("validate", n), &entries, |b, e| {
            b.iter(|| Blake3ChainValidator::validate_chain(black_box(e)).is_ok());
        });
        group.bench_with_input(BenchmarkId::new("build", n), &n, |b, &n| {
            b.iter(|| chain(black_box(n)).len());
        });
    }
    group.finish();
}


// ─── ChainedCourtReceipt (352B ledger records, the v26.10.x real chain) ───
//
// The group above measures the small synthetic RawReceiptEntry; the ledger
// actually chains 352-byte `ChainedCourtReceipt` records (~336 bytes hashed
// per link vs ~41), so those numbers under-represent ledger validation cost
// by roughly 5-8x on the hashing-dominated path. This group measures the
// REAL surface. (Audit W11.)

use chicago_tdd_tools::ledger::receipt::{
    ChainedCourtReceipt, CourtOutcome, CourtReceiptV1, ReceiptMetrics,
};
use chicago_tdd_tools::ledger::receipt::{COURT_RECORD_LEN};
use chicago_tdd_tools::ledger::receipt::ThermalClass;

/// Builds an n-long chain of 352B ChainedCourtReceipt records, each linking
/// to its parent's chain hash (the ledger's real chaining shape).
fn court_chain(n: u64) -> Vec<ChainedCourtReceipt> {
    let mut parent = [0u8; 32]; // genesis
    let mut out = Vec::with_capacity(n as usize);
    for i in 0..n {
        let receipt = CourtReceiptV1::new(
            [i as u8; 32], // subject_hash
            parent,        // parent_receipt_hash = previous chain hash
            ReceiptMetrics {
                timestamp_ms: i,
                run_id: 1,
                total_ticks: i,
                wall_ms: 1,
                thermal: ThermalClass::Warm,
                outcome: CourtOutcome::Pass,
                budget_met: true,
            },
            [0u8; 32], // positive_control
            [0u8; 32], // metadata_digest
            [0u8; 32], // prev_self_hash
            "bench",   // name (valid literal; single justified expect below)
        )
        .unwrap_or_else(|e| panic!("bench receipt construction failed: {e}"));
        let chained = ChainedCourtReceipt::from_receipt(&receipt)
            .unwrap_or_else(|e| panic!("bench chain link failed: {e}"));
        let encoded = chained.encode();
        parent = encoded[COURT_RECORD_LEN - 32..].try_into().expect("32B tail");
        out.push(chained);
    }
    out
}

fn bench_chained_court_receipt(c: &mut Criterion) {
    let mut group = c.benchmark_group("chained_court_receipt");
    for n in [16u64, 1_024, 10_000] {
        group.throughput(Throughput::Elements(n));
        let entries = court_chain(n);
        group.bench_with_input(BenchmarkId::new("validate", n), &entries, |b, e| {
            b.iter(|| Blake3ChainValidator::validate_chain(black_box(e)).is_ok());
        });
        group.bench_with_input(BenchmarkId::new("build", n), &n, |b, &n| {
            b.iter(|| court_chain(black_box(n)).len());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_receipt_chain, bench_chained_court_receipt);
criterion_main!(benches);
