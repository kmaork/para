#![allow(unused_must_use)]

use criterion::{criterion_group, criterion_main, Criterion};
use para::*;
use std::time::Duration;

pub fn bench_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipelines");
    group.noise_threshold(0.03);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("one_map", |b| {
        b.iter(|| {
            run_pipeline!(0..10000 => |x| {x * 99;});
        })
    });
    group.bench_function("two_maps", |b| {
        b.iter(|| {
            run_pipeline!(0..10000 => |x| x * 99 => |x| {x - 1;});
        })
    });
    group.bench_function("fanout", |b| {
        b.iter(|| {
            let f = Fanout::new(vec![
                &|x| {
                    x + 1;
                },
                &|x| {
                    x - 1;
                },
            ]);
            run_pipeline!(0..10000 => f);
        })
    });
}

criterion_group!(pipelines, bench_pipeline);
criterion_main!(pipelines);
