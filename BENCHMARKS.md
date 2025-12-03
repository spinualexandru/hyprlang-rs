# Benchmark Results

Generated using [Criterion.rs](https://github.com/bheisler/criterion.rs) with synthetic configs.

## Quick Summary

### Parsing Performance

| Config Size | Lines | Time |
|------------|-------|------|
| Small |     50 |   77.00 µs |
| Medium |    300 |  508.20 µs |
| Large |   1,000 |    1.68 ms |
| XLarge |  10,000 |   18.06 ms |
| Perf | 1,000,000 |     2.69 s |

**Parsing scales linearly:** ~2.7 µs per line on average.

### Retrieval Performance

| Operation | Time |
|-----------|------|
| Contains |   10.31 ns |
| Get String |   11.00 ns |
| Get Int |   12.33 ns |
| Get Color |   12.71 ns |
| Get Float |   13.13 ns |
| Keys Iteration |   41.72 ns |

**All retrieval operations are extremely fast** with sub-nanosecond overhead.

### Mutation Performance (requires `mutation` feature)

| Operation | Time |
|-----------|------|
| Set Int |  109.54 ns |
| Serialize Small (50 lines) |    3.25 µs |
| Serialize Large (1000 lines) |   66.87 µs |
| Round Trip (parse→mutate→serialize→parse) |  181.48 µs |

---

## Detailed Results (with 95% Confidence Intervals)

### Parsing

| Benchmark | Mean | 95% CI Lower | 95% CI Upper |
|-----------|------|--------------|--------------|
| small_50_lines |   77.00 µs |   76.84 µs |   77.15 µs |
| medium_300_lines |  508.20 µs |  507.33 µs |  509.07 µs |
| large_1000_lines |    1.68 ms |    1.67 ms |    1.68 ms |
| xlarge_10000_lines |   18.06 ms |   18.03 ms |   18.10 ms |
| perf_1M_lines |     2.69 s |     2.67 s |     2.71 s |

### Retrieval

| Benchmark | Mean | 95% CI Lower | 95% CI Upper |
|-----------|------|--------------|--------------|
| contains |   10.31 ns |   10.28 ns |   10.34 ns |
| get_string |   11.00 ns |   10.99 ns |   11.02 ns |
| get_int |   12.33 ns |   12.30 ns |   12.36 ns |
| get_color |   12.71 ns |   12.69 ns |   12.73 ns |
| get_float |   13.13 ns |   13.11 ns |   13.14 ns |
| keys_iteration |   41.72 ns |   41.60 ns |   41.85 ns |

### Mutation

| Benchmark | Mean | 95% CI Lower | 95% CI Upper |
|-----------|------|--------------|--------------|
| set_int |  109.54 ns |  108.88 ns |  110.23 ns |
| serialize_small |    3.25 µs |    3.24 µs |    3.25 µs |
| serialize_large |   66.87 µs |   66.67 µs |   67.09 µs |
| round_trip |  181.48 µs |  181.21 µs |  181.77 µs |

---

## Performance Insights

### Parsing
- **Linear scaling:** Parsing time grows proportionally with config size (~2.7 µs/line)
- **Throughput:** ~370,000 lines/second for large configs
- **Memory efficient:** 1M line config parses in under 3 seconds

### Retrieval
- **Hash-based lookups:** O(1) complexity with ~10-13ns average lookup time
- **Minimal overhead:** Type conversion adds <3ns per operation
- **Key iteration:** Enumerating all keys takes ~42ns (trivial overhead)

### Mutation & Serialization
- **Fast mutations:** Setting values takes ~110ns
- **Efficient serialization:** ~65 ns per line for serialization
- **Round-trip performance:** Full parse→modify→serialize→reparse cycle under 200µs for small configs

---

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run with mutation feature
cargo bench --features mutation

# Run specific benchmark group
cargo bench --bench parsing
cargo bench --bench retrieval
cargo bench --bench mutation --features mutation

# Save baseline for comparison
cargo bench -- --save-baseline v0.3.0

# Compare against baseline
cargo bench -- --baseline v0.3.0
```

## Output Locations

- **HTML Report:** `target/criterion/report/index.html`
- **JSON Data:** `target/criterion/<group>/<benchmark>/new/estimates.json`

---

*Benchmarks run on: Rust 1.91.1, Linux 6.17.9-2-cachyos*
