// Performance benchmarks for test data builder functionality
// Tests the performance of data structure creation and building

#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[allow(dead_code)]
struct TestData {
    id: usize,
    name: String,
}

impl TestData {
    fn new(id: usize, name: String) -> Self {
        Self { id, name }
    }
}

fn benchmark_struct_creation(c: &mut Criterion) {
    c.bench_function("struct_creation", |b| {
        b.iter(|| {
            let _data = black_box(TestData::new(1, "test".to_string()));
        });
    });
}

fn benchmark_vector_of_structs(c: &mut Criterion) {
    c.bench_function("create_100_structs", |b| {
        b.iter(|| {
            let items: Vec<_> = (0..100)
                .map(|i| {
                    black_box(TestData::new(
                        black_box(i),
                        black_box(format!("item_{}", i)),
                    ))
                })
                .collect();
            black_box(items)
        });
    });
}

fn benchmark_string_building(c: &mut Criterion) {
    c.bench_function("build_complex_string", |b| {
        b.iter(|| {
            let mut s = String::new();
            for i in 0..50 {
                s.push_str(&format!("item_{};", i));
            }
            black_box(s)
        });
    });
}

criterion_group!(
    benches,
    benchmark_struct_creation,
    benchmark_vector_of_structs,
    benchmark_string_building
);
criterion_main!(benches);
