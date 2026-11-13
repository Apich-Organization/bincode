extern crate bincode_next as bincode;
use bincode::config;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use std::hint::black_box;

fn inline_decoder_claim_bytes_read(c: &mut Criterion) {
    let config = config::standard().with_limit::<100000>();
    let slice = bincode::encode_to_vec(vec![String::from("Hello world"); 1000], config).unwrap();

    c.bench_function("inline_decoder_claim_bytes_read", |b| {
        b.iter(|| {
            let _: (Vec<String>, usize) =
                black_box(bincode::decode_from_slice(black_box(&slice), config).unwrap());
        })
    });
}

#[cfg(feature = "serde")]
fn serde_sequence_decode(c: &mut Criterion) {
    let config = config::legacy();
    let bytes = bincode::serde::encode_to_vec(vec![42u8; 1024], config).unwrap();

    c.bench_function("serde_sequence_decode/slice", |b| {
        b.iter(|| {
            let result =
                bincode::serde::decode_from_slice::<Vec<u8>, _>(black_box(&bytes), config).unwrap();
            black_box(result);
        })
    });

    c.bench_function("serde_sequence_decode/reader", |b| {
        b.iter(|| {
            let reader = bincode::de::read::SliceReader::new(black_box(&bytes));
            let result =
                bincode::serde::decode_from_reader::<Vec<u8>, _, _>(reader, config).unwrap();
            black_box(result);
        })
    });
}

#[cfg(feature = "serde")]
criterion_group!(
    benches,
    inline_decoder_claim_bytes_read,
    serde_sequence_decode
);
#[cfg(not(feature = "serde"))]
criterion_group!(benches, inline_decoder_claim_bytes_read);
criterion_main!(benches);
