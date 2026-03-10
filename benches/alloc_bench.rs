use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_alloc(c: &mut Criterion) {
    c.bench_function("vec_alloc_1000", |b| {
        b.iter(|| {
            let v: Vec<u8> = Vec::with_capacity(black_box(1000));
            black_box(v)
        })
    });
    c.bench_function("string_alloc_256", |b| {
        b.iter(|| {
            let s = String::with_capacity(black_box(256));
            black_box(s)
        })
    });
    c.bench_function("box_alloc_u64", |b| {
        b.iter(|| {
            let b = Box::new(black_box(42u64));
            black_box(b)
        })
    });
}

criterion_group!(benches, bench_alloc);
criterion_main!(benches);
