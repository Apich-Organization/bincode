#![cfg(all(feature = "std", feature = "derive"))]
//! Regression tests for inputs found by the CI fuzzers.

use bincode_next::Decode;
use bincode_next::Encode;
use bincode_next::config;
use bincode_next::decode_from_slice;
use bincode_next::encode_to_vec;
use bincode_next::error::DecodeError;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

type Key = Result<(u8, u8, u8), Vec<u8>>;

#[derive(Decode, Encode, PartialEq, Debug)]
struct Payload {
    map: BTreeMap<Key, String>,
    set: std::collections::BTreeSet<Vec<u8>>,
    heap: Vec<u32>,
}

/// `deterministic` fuzzer: batch varint decoding of `Vec<u32>` computed
/// `len * size_of::<T>() - consumed` and underflowed (panic in debug builds) because a
/// varint-encoded `u32` can occupy 5 bytes, more than its in-memory size of 4.
#[test]
fn varint_batch_decode_does_not_underflow_limit_accounting() {
    let crash: &[u8] = include_bytes!("fuzz_regressions/varint_vec_unclaim_underflow.bin");
    let lf = config::standard()
        .with_deterministic_bincode_length_first()
        .with_limit::<4096>();
    let core = config::standard()
        .with_deterministic_bincode()
        .with_limit::<4096>();
    let _ = decode_from_slice::<Payload, _>(crash, lf);
    let _ = decode_from_slice::<HashMap<Key, u8>, _>(crash, core);
    let _ = decode_from_slice::<HashSet<Key>, _>(crash, lf);

    // Minimal form: three 5-byte varint u32s (marker 0xFC + u32) under a byte limit.
    let mut bytes = vec![3u8];
    for v in [70_000u32, 80_000, 90_000] {
        bytes.push(252);
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    let cfg = config::standard().with_limit::<64>();
    let (v, n): (Vec<u32>, usize) = decode_from_slice(&bytes, cfg).unwrap();
    assert_eq!(v, [70_000, 80_000, 90_000]);
    assert_eq!(n, bytes.len());
    let (v, _): (Vec<u64>, usize) = decode_from_slice(&bytes, cfg.with_limit::<128>()).unwrap();
    assert_eq!(v, [70_000, 80_000, 90_000]);
}

#[derive(Decode, Encode, PartialEq, Debug)]
struct Doc {
    a: u64,
    b: i64,
    c: String,
    d: Vec<u8>,
    e: Option<f64>,
    f: BTreeMap<String, Vec<i32>>,
}

/// `cbor` fuzzer: an indefinite-length array was accepted without charging its elements'
/// in-memory size against the limit, so re-encoding the result as a (definite) array — which
/// charges `len * size_of::<T>()` up front — was rejected with `LimitExceeded`.
#[test]
fn cbor_indefinite_array_charges_element_size() {
    let crash: &[u8] = include_bytes!("fuzz_regressions/cbor_indefinite_array_limit.bin");
    let cbor = config::standard().with_cbor_format().with_limit::<8192>();
    if let Ok((doc, _)) = decode_from_slice::<Doc, _>(crash, cbor) {
        let enc = encode_to_vec(&doc, cbor).unwrap();
        decode_from_slice::<Doc, _>(&enc, cbor).expect("re-decode of own encoding");
    }

    // 3000 one-byte ints: tiny on the wire, 12000 bytes as `Vec<i32>`.
    let mut indefinite = vec![0x9F];
    indefinite.extend(std::iter::repeat(0x01).take(3000));
    indefinite.push(0xFF);
    let mut definite = vec![0x99, 0x0B, 0xB8]; // array(3000)
    definite.extend(std::iter::repeat(0x01).take(3000));

    let cfg = config::standard().with_cbor_format().with_limit::<8192>();
    let a = decode_from_slice::<Vec<i32>, _>(&definite, cfg);
    let b = decode_from_slice::<Vec<i32>, _>(&indefinite, cfg);
    assert!(matches!(a, Err(DecodeError::LimitExceeded)), "{a:?}");
    assert!(
        matches!(b, Err(DecodeError::LimitExceeded)),
        "indefinite encoding must obey the same limit as the definite one: {b:?}"
    );

    // Below the limit both forms decode to the same value.
    let big = config::standard().with_cbor_format().with_limit::<65536>();
    let (a, _): (Vec<i32>, _) = decode_from_slice(&definite, big).unwrap();
    let (b, _): (Vec<i32>, _) = decode_from_slice(&indefinite, big).unwrap();
    assert_eq!(a, b);
}
