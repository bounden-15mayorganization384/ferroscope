use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_iter(c: &mut Criterion) {
    let data: Vec<u64> = (0..10_000).collect();

    c.bench_function("iter_map_filter_sum", |b| {
        b.iter(|| {
            black_box(
                data.iter()
                    .filter(|&&x| x % 2 == 0)
                    .map(|&x| x * 2)
                    .sum::<u64>(),
            )
        })
    });

    c.bench_function("manual_loop_sum", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for &x in &data {
                if x % 2 == 0 {
                    sum += x * 2;
                }
            }
            black_box(sum)
        })
    });

    c.bench_function("iter_chain_complex", |b| {
        b.iter(|| {
            black_box(
                data.iter()
                    .enumerate()
                    .filter(|(i, _)| i % 3 == 0)
                    .map(|(_, &x)| x * x)
                    .take(100)
                    .sum::<u64>(),
            )
        })
    });
}

criterion_group!(benches, bench_iter);
criterion_main!(benches);
