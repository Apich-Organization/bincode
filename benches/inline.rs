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

fn bulk_reader_decode(c: &mut Criterion) {
    use bincode::de::read::Reader;
    use bincode::de::read::SliceReader;
    use bincode::error::DecodeError;

    // A reader with no peek support exercises the initialized-buffer fallback.
    struct ReadOnly<'a>(SliceReader<'a>);
    impl Reader for ReadOnly<'_> {
        fn read(
            &mut self,
            bytes: &mut [u8],
        ) -> Result<(), DecodeError> {
            self.0.read(bytes)
        }
    }

    let config = config::legacy();
    let bytes = bincode::encode_to_vec(vec![42u8; 4096], config).unwrap();
    c.bench_function("bulk_reader_decode/vec_buffered", |b| {
        b.iter(|| {
            black_box(bincode::decode_from_slice::<Vec<u8>, _>(black_box(&bytes), config).unwrap())
        })
    });
    c.bench_function("bulk_reader_decode/vec_custom", |b| {
        b.iter(|| {
            let reader = ReadOnly(SliceReader::new(black_box(&bytes)));
            black_box(bincode::decode_from_reader::<Vec<u8>, _, _>(reader, config).unwrap())
        })
    });
    c.bench_function("bulk_reader_decode/array_custom", |b| {
        b.iter(|| {
            let reader = ReadOnly(SliceReader::new(black_box(&bytes[8..])));
            black_box(bincode::decode_from_reader::<[u8; 4096], _, _>(reader, config).unwrap())
        })
    });
    let bytes = bincode::encode_to_vec("x".repeat(4096), config).unwrap();
    c.bench_function("bulk_reader_decode/string_custom", |b| {
        b.iter(|| {
            let reader = ReadOnly(SliceReader::new(black_box(&bytes)));
            black_box(bincode::decode_from_reader::<String, _, _>(reader, config).unwrap())
        })
    });
}

criterion_group!(benches, inline_decoder_claim_bytes_read, bulk_reader_decode);
criterion_main!(benches);
