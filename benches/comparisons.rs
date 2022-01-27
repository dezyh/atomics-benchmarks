use criterion::{black_box, criterion_group, criterion_main, Criterion};
use playground::{atomics_relaxed, atomics_seqcst, unsafe_ub};

fn criterion_benchmark(c: &mut Criterion) {
    let n = 100000;
    c.bench_function("relaxed atomic u64", |b| {
        b.iter(|| atomics_relaxed(black_box(n)))
    });
    c.bench_function("unsafe ub u64", |b| b.iter(|| unsafe_ub(black_box(n))));
    c.bench_function("seqcst atomic u64", |b| {
        b.iter(|| atomics_seqcst(black_box(n)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
