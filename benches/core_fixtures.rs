// Performance benchmarks for core assertion functionality
// Tests the performance of basic assertion operations

#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_basic_equality_check(c: &mut Criterion) {
    c.bench_function("basic_equality_check", |b| {
        b.iter(|| {
            let a = black_box(42);
            let b_val = black_box(42);
            let _result = a == b_val;
        });
    });
}

fn benchmark_comparison_chain(c: &mut Criterion) {
    c.bench_function("comparison_chain_10_values", |b| {
        b.iter(|| {
            let values: Vec<_> = (0..10).map(|i| black_box(i)).collect();
            let _all_positive = values.iter().all(|&v| v >= 0);
        });
    });
}

fn benchmark_formatting_for_error_message(c: &mut Criterion) {
    c.bench_function("format_error_message", |b| {
        b.iter(|| {
            let actual = black_box(42);
            let expected = black_box(43);
            let _msg = format!("assertion failed: {} != {}", actual, expected);
        });
    });
}

criterion_group!(
    benches,
    benchmark_basic_equality_check,
    benchmark_comparison_chain,
    benchmark_formatting_for_error_message
);
criterion_main!(benches);
