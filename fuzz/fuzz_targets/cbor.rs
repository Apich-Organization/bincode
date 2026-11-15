#![no_main]
use libfuzzer_sys::fuzz_target;

use std::collections::BTreeMap;

#[derive(bincode_next::Decode, bincode_next::Encode, PartialEq, Debug, arbitrary::Arbitrary)]
struct Doc {
    a: u64,
    b: i64,
    c: String,
    d: Vec<u8>,
    e: Option<f64>,
    f: BTreeMap<String, Vec<i32>>,
}

fn same(
    x: &Doc,
    y: &Doc,
) -> bool {
    // f64 NaN != NaN, compare bitwise-insensitive for NaN.
    let e_ok = match (x.e, y.e) {
        | (Some(p), Some(q)) => (p.is_nan() && q.is_nan()) || p.to_bits() == q.to_bits(),
        | (None, None) => true,
        | _ => false,
    };
    e_ok && x.a == y.a && x.b == y.b && x.c == y.c && x.d == y.d && x.f == y.f
}

fuzz_target!(|data: &[u8]| {
    use bincode_next::config::standard;
    let cbor = standard().with_cbor_format().with_limit::<8192>();
    let det = standard().with_deterministic_cbor().with_limit::<8192>();

    // 1. Arbitrary bytes must never panic; whatever decodes must round-trip.
    macro_rules! decode_roundtrip {
        ($cfg:expr) => {
            if let Ok((v, _)) = bincode_next::decode_from_slice::<Doc, _>(data, $cfg) {
                let enc = bincode_next::encode_to_vec(&v, $cfg).expect("re-encode");
                let (v2, n): (Doc, _) =
                    bincode_next::decode_from_slice(&enc, $cfg).expect("re-decode");
                assert!(same(&v, &v2));
                assert_eq!(n, enc.len());
            }
        };
    }
    decode_roundtrip!(cbor);
    decode_roundtrip!(det);

    // 2. Structured values round-trip and deterministic encoding is stable.
    let mut u = arbitrary::Unstructured::new(data);
    if let Ok(doc) = u.arbitrary::<Doc>() {
        let enc = bincode_next::encode_to_vec(&doc, det).unwrap();
        let (back, n): (Doc, _) = bincode_next::decode_from_slice(&enc, det).unwrap();
        assert!(same(&doc, &back));
        assert_eq!(n, enc.len());
        assert_eq!(enc, bincode_next::encode_to_vec(&back, det).unwrap());
    }
});
