use criterion::{criterion_group, criterion_main, Criterion};
use hyprlang::Config;

fn retrieval_benchmarks(c: &mut Criterion) {
    // Pre-parse config once for retrieval benchmarks
    let input = include_str!("../examples/example.conf");
    let mut config = Config::new();
    config.parse(input).unwrap();

    let mut group = c.benchmark_group("retrieval");

    group.bench_function("get_int", |b| {
        b.iter(|| config.get_int("general:border_size"))
    });

    group.bench_function("get_float", |b| {
        b.iter(|| config.get_float("decoration:active_opacity"))
    });

    group.bench_function("get_string", |b| {
        b.iter(|| config.get_string("general:layout"))
    });

    group.bench_function("get_color", |b| {
        b.iter(|| config.get_color("general:col.active_border"))
    });

    group.bench_function("contains", |b| {
        b.iter(|| config.contains("general:border_size"))
    });

    group.bench_function("keys_iteration", |b| {
        b.iter(|| config.keys().len())
    });

    group.finish();
}

criterion_group!(benches, retrieval_benchmarks);
criterion_main!(benches);
