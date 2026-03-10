use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort");
    for size in [100u64, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("sort_unstable", size), &size, |b, &size| {
            b.iter(|| {
                let mut v: Vec<u64> = (0..size).rev().collect();
                v.sort_unstable();
                black_box(v)
            })
        });
        group.bench_with_input(BenchmarkId::new("sort_stable", size), &size, |b, &size| {
            b.iter(|| {
                let mut v: Vec<u64> = (0..size).rev().collect();
                v.sort();
                black_box(v)
            })
        });
        group.bench_with_input(BenchmarkId::new("iter_sum", size), &size, |b, &size| {
            b.iter(|| black_box((0..size).sum::<u64>()))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_sort);
criterion_main!(benches);
