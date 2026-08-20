use criterion::{Criterion, criterion_group, criterion_main};
use radix_spline::RadixSpline;
use std::hint::black_box;

fn spline_lookup(c: &mut Criterion) {
    let mut data = (0..100).collect::<Vec<u64>>();
    data.extend(200..300);
    data.extend(500..=700);
    let mut builder = RadixSpline::builder(0, 700);
    builder.add_keys(data.iter().copied());
    let spline = builder.build();
    c.bench_function("spline_lookup", |b| {
        b.iter(|| spline.find(fastrand::u64(0..710)))
    });
}

fn spline_construction(c: &mut Criterion) {
    let mut data = (0..100).collect::<Vec<u64>>();
    data.extend(200..300);
    data.extend(500..=700);
    c.bench_function("spline_construction", |b| {
        b.iter(|| {
            let mut builder = RadixSpline::builder(0, 700);
            builder.add_keys(black_box(data.iter().copied()));
            let _ = black_box(builder.build());
        })
    });
}

criterion_group!(benches, spline_construction, spline_lookup);
criterion_main!(benches);
