use criterion::{criterion_group, criterion_main};

mod parse_bench;

criterion_group!(
    benches,
    parse_bench::bench_parser,
    parse_bench::bench_min_values,
    parse_bench::bench_max_values,
    parse_bench::bench_corrupted_data,
    parse_bench::bench_consecutive_packets
);
criterion_main!(benches);
