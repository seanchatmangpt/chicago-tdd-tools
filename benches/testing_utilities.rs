// Performance benchmarks for mutation testing functionality
// Tests the performance of mutation operators and detection

#![allow(missing_docs)]

use chicago_tdd_tools::testing::mutation::{MutationOperator, MutationTester};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

fn benchmark_mutation_operator_creation(c: &mut Criterion) {
    c.bench_function("create_mutation_operator", |b| {
        b.iter(|| {
            let _op = black_box(MutationOperator::AddKey("key".to_string(), "value".to_string()));
        });
    });
}

fn benchmark_mutation_tester_creation(c: &mut Criterion) {
    c.bench_function("create_mutation_tester", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            map.insert("key".to_string(), "value".to_string());
            let _tester = black_box(MutationTester::new(map));
        });
    });
}

fn benchmark_apply_mutations(c: &mut Criterion) {
    c.bench_function("apply_10_mutations", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            map.insert("key".to_string(), "value".to_string());
            let mut tester = MutationTester::new(map);
            for i in 0..10 {
                let op = MutationOperator::AddKey(format!("key_{}", i), "val".to_string());
                let _mutated = tester.apply_mutation(op);
            }
        });
    });
}

criterion_group!(
    benches,
    benchmark_mutation_operator_creation,
    benchmark_mutation_tester_creation,
    benchmark_apply_mutations
);
criterion_main!(benches);
