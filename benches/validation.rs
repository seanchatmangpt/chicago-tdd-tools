// Performance benchmarks for validation functionality
// Tests the performance of validation guards and coverage tracking

#![allow(missing_docs)]

use chicago_tdd_tools::validation::guards::{GuardValidator, ValidatedBatch, ValidatedRun};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_guard_validator_creation(c: &mut Criterion) {
    c.bench_function("guard_validator_creation", |b| {
        b.iter(|| {
            let _validator = black_box(GuardValidator::new());
        });
    });
}

fn benchmark_guard_validator_validate_run(c: &mut Criterion) {
    c.bench_function("guard_validator_validate_run", |b| {
        let validator = GuardValidator::new();
        b.iter(|| {
            let data = black_box(vec![1, 2, 3, 4, 5]);
            let _ = validator.validate_run(&data);
        });
    });
}

fn benchmark_validated_batch_creation(c: &mut Criterion) {
    c.bench_function("validated_batch_creation", |b| {
        b.iter(|| {
            let data = black_box(vec![0u8; 500]);
            let _ = ValidatedBatch::<500>::new(data);
        });
    });
}

fn benchmark_validated_run_creation(c: &mut Criterion) {
    c.bench_function("validated_run_creation", |b| {
        b.iter(|| {
            let data = black_box(vec![0u8; 5]);
            let _ = ValidatedRun::<5>::new(data);
        });
    });
}

criterion_group!(
    benches,
    benchmark_guard_validator_creation,
    benchmark_guard_validator_validate_run,
    benchmark_validated_batch_creation,
    benchmark_validated_run_creation
);
criterion_main!(benches);
