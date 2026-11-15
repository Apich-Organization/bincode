#![no_main]
use libfuzzer_sys::fuzz_target;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;

type Key = Result<(u8, u8, u8), Vec<u8>>;

#[derive(bincode_next::Decode, bincode_next::Encode, PartialEq, Debug, arbitrary::Arbitrary)]
struct Payload {
    map: BTreeMap<Key, String>,
    set: BTreeSet<Vec<u8>>,
    heap: Vec<u32>,
}

#[macro_use]
#[path = "../common/mod.rs"]
mod common;

fuzz_target!(|data: &[u8]| {
    // Limit-monotonicity / fixed-point / self-delimiting checks in both deterministic
    // bincode modes (Core and LengthFirst) plus the explicit-mode constructors.
    {
        use bincode_next::config::standard;
        check_types!(
            data,
            standard().with_deterministic_bincode(),
            true,
            [HashMap<Key, u8>, HashSet<Key>, BTreeMap<Key, String>, Payload]
        );
        check_types!(
            data,
            standard().with_deterministic_bincode_length_first(),
            true,
            [HashMap<Key, u8>, HashSet<Key>, BTreeSet<Vec<u8>>, Payload]
        );
        check_types!(
            data,
            standard()
                .with_big_endian()
                .with_fixed_int_encoding()
                .with_bincode_deterministic_mode::<2>(),
            true,
            [HashMap<Key, u8>, Payload]
        );
    }

    use bincode_next::config::standard;
    let core = standard().with_deterministic_bincode().with_limit::<4096>();
    let lf = standard()
        .with_deterministic_bincode_length_first()
        .with_limit::<4096>();

    // 1. Decoding arbitrary bytes must never panic, and anything that decodes
    //    must re-encode to a decodable, equal value.
    macro_rules! decode_roundtrip {
        ($cfg:expr, $t:ty) => {
            if let Ok((v, _)) = bincode_next::decode_from_slice::<$t, _>(data, $cfg) {
                let enc = bincode_next::encode_to_vec(&v, $cfg).expect("re-encode");
                let (v2, n): ($t, _) =
                    bincode_next::decode_from_slice(&enc, $cfg).expect("re-decode");
                assert_eq!(v, v2);
                assert_eq!(n, enc.len());
                // Deterministic: encoding again is byte-identical.
                assert_eq!(enc, bincode_next::encode_to_vec(&v2, $cfg).unwrap());
            }
        };
    }
    decode_roundtrip!(core, HashMap<Key, u8>);
    decode_roundtrip!(lf, HashMap<Key, u8>);
    decode_roundtrip!(core, HashSet<Key>);
    decode_roundtrip!(lf, HashSet<Key>);
    decode_roundtrip!(lf, Payload);

    // 2. Structured: insertion order must not change encoding.
    let mut u = arbitrary::Unstructured::new(data);
    if let Ok(entries) = u.arbitrary::<Vec<(Key, u8)>>() {
        let fwd: HashMap<Key, u8> = entries.iter().cloned().collect();
        let mut rev_entries = entries.clone();
        rev_entries.reverse();
        // Only comparable when duplicate keys cannot cause differing values.
        let rev: HashMap<Key, u8> = rev_entries.into_iter().collect();
        if fwd == rev {
            assert_eq!(
                bincode_next::encode_to_vec(&fwd, core).unwrap(),
                bincode_next::encode_to_vec(&rev, core).unwrap()
            );
            assert_eq!(
                bincode_next::encode_to_vec(&fwd, lf).unwrap(),
                bincode_next::encode_to_vec(&rev, lf).unwrap()
            );
        }
    }
    if let Ok(items) = u.arbitrary::<Vec<u32>>() {
        let a: BinaryHeap<u32> = items.iter().cloned().collect();
        let mut r = items;
        r.reverse();
        let b: BinaryHeap<u32> = r.into_iter().collect();
        assert_eq!(
            bincode_next::encode_to_vec(&a, lf).unwrap(),
            bincode_next::encode_to_vec(&b, lf).unwrap()
        );
    }
});
