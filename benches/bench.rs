#![allow(unused_must_use)]

use criterion::{criterion_group, criterion_main, Criterion};
use para::*;

pub fn bench_one_map(c: &mut Criterion) {
    c.bench_function("One map", |b| b.iter(|| {
        run_pipeline!(0..10000 => |x| {x * 99;});
    }));
}

pub fn bench_two_maps(c: &mut Criterion) {
    c.bench_function("Two maps", |b| b.iter(|| {
        run_pipeline!(0..10000 => |x| x * 99 => |x| {x - 1;});
    }));
}

pub fn bench_fanout(c: &mut Criterion) {
    c.bench_function("Fanout", |b| b.iter(|| {
        let f = Fanout::new(vec![&|x| { x + 1; }, &|x| { x - 1; }]);
        run_pipeline!(0..10000 => f);
    }));
}

criterion_group!(benches, bench_one_map, bench_two_maps, bench_fanout);
criterion_main!(benches);