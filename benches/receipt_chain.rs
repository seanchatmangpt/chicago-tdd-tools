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

criterion_group!(benches, bench_receipt_chain);
criterion_main!(benches);
