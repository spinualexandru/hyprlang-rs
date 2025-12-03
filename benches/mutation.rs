mod config_generator;

use config_generator::generate_config;
use criterion::{criterion_group, criterion_main, Criterion};
use hyprlang::Config;

fn mutation_benchmarks(c: &mut Criterion) {
    let small = generate_config(50);
    let large = generate_config(1000);

    let mut group = c.benchmark_group("mutation");

    // Value mutation
    group.bench_function("set_int", |b| {
        let mut config = Config::new();
        config.parse(&small).unwrap();
        b.iter(|| config.set_int("test:value", 42))
    });

    // Serialization - small config
    group.bench_function("serialize_small", |b| {
        let mut config = Config::new();
        config.parse(&small).unwrap();
        b.iter(|| config.serialize())
    });

    // Serialization - large config
    group.bench_function("serialize_large", |b| {
        let mut config = Config::new();
        config.parse(&large).unwrap();
        b.iter(|| config.serialize())
    });

    // Round-trip: parse -> mutate -> serialize
    group.bench_function("round_trip", |b| {
        b.iter(|| {
            let mut config = Config::new();
            config.parse(&small).unwrap();
            config.set_int("test:value", 42);
            let output = config.serialize();
            let mut config2 = Config::new();
            config2.parse(&output).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, mutation_benchmarks);
criterion_main!(benches);
