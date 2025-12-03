mod config_generator;

use config_generator::generate_config;
use criterion::{criterion_group, criterion_main, Criterion};
use hyprlang::Config;

fn parsing_benchmarks(c: &mut Criterion) {
    // Generate configs of different sizes
    let small = generate_config(50);
    let medium = generate_config(300);
    let large = generate_config(1_000);
    let xlarge = generate_config(10_000);

    let mut group = c.benchmark_group("parsing");

    group.bench_function("small_50_lines", |b| {
        b.iter(|| {
            let mut config = Config::new();
            config.parse(&small).unwrap()
        })
    });

    group.bench_function("medium_300_lines", |b| {
        b.iter(|| {
            let mut config = Config::new();
            config.parse(&medium).unwrap()
        })
    });

    group.bench_function("large_1000_lines", |b| {
        b.iter(|| {
            let mut config = Config::new();
            config.parse(&large).unwrap()
        })
    });

    group.bench_function("xlarge_10000_lines", |b| {
        b.iter(|| {
            let mut config = Config::new();
            config.parse(&xlarge).unwrap()
        })
    });

    group.finish();
}

fn perf_benchmark(c: &mut Criterion) {
    // 1 million lines - generated once, benchmarked separately
    let perf = generate_config(1_000_000);

    let mut group = c.benchmark_group("perf");
    group.sample_size(10); // Fewer samples for very large configs

    group.bench_function("perf_1M_lines", |b| {
        b.iter(|| {
            let mut config = Config::new();
            config.parse(&perf).unwrap()
        })
    });

    group.finish();
}

criterion_group!(benches, parsing_benchmarks, perf_benchmark);
criterion_main!(benches);
