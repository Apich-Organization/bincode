//! Cost of deterministic encoding vs plain bincode for unordered collections.
extern crate bincode_next as bincode;
use bincode::config;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use std::collections::HashMap;
use std::collections::HashSet;

fn make_map(n: u32) -> HashMap<u32, String> {
    (0..n)
        .map(|i| (i.wrapping_mul(2_654_435_761), format!("value-{i}")))
        .collect()
}

fn encode_hashmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("deterministic/encode_hashmap");
    for n in [16u32, 256, 4096] {
        let map = make_map(n);
        group.bench_with_input(BenchmarkId::new("plain", n), &map, |b, m| {
            let cfg = config::standard();
            b.iter(|| black_box(bincode::encode_to_vec(black_box(m), cfg).unwrap()));
        });
        group.bench_with_input(BenchmarkId::new("core", n), &map, |b, m| {
            let cfg = config::standard().with_deterministic_bincode();
            b.iter(|| black_box(bincode::encode_to_vec(black_box(m), cfg).unwrap()));
        });
        group.bench_with_input(BenchmarkId::new("length_first", n), &map, |b, m| {
            let cfg = config::standard().with_deterministic_bincode_length_first();
            b.iter(|| black_box(bincode::encode_to_vec(black_box(m), cfg).unwrap()));
        });
    }
    group.finish();
}

fn decode_hashmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("deterministic/decode_hashmap");
    let map = make_map(1024);
    let plain_cfg = config::standard();
    let det_cfg = config::standard().with_deterministic_bincode();
    let plain = bincode::encode_to_vec(&map, plain_cfg).unwrap();
    let det = bincode::encode_to_vec(&map, det_cfg).unwrap();
    group.bench_function("plain", |b| {
        b.iter(|| {
            let r: (HashMap<u32, String>, usize) =
                bincode::decode_from_slice(black_box(&plain), plain_cfg).unwrap();
            black_box(r)
        });
    });
    group.bench_function("core_dup_check", |b| {
        b.iter(|| {
            let r: (HashMap<u32, String>, usize) =
                bincode::decode_from_slice(black_box(&det), det_cfg).unwrap();
            black_box(r)
        });
    });
    group.finish();
}

fn encode_hashset(c: &mut Criterion) {
    let set: HashSet<u64> = (0..2048u64)
        .map(|i| i.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .collect();
    let mut group = c.benchmark_group("deterministic/encode_hashset");
    group.bench_function("plain", |b| {
        let cfg = config::standard();
        b.iter(|| black_box(bincode::encode_to_vec(black_box(&set), cfg).unwrap()));
    });
    group.bench_function("core", |b| {
        let cfg = config::standard().with_deterministic_bincode();
        b.iter(|| black_box(bincode::encode_to_vec(black_box(&set), cfg).unwrap()));
    });
    group.bench_function("length_first", |b| {
        let cfg = config::standard().with_deterministic_bincode_length_first();
        b.iter(|| black_box(bincode::encode_to_vec(black_box(&set), cfg).unwrap()));
    });
    group.finish();
}

criterion_group!(benches, encode_hashmap, decode_hashmap, encode_hashset);
criterion_main!(benches);
