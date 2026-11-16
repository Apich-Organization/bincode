#![cfg(all(feature = "std", feature = "derive"))]

//! Precision-oriented property tests across formats and configurations:
//! bit-exact floats, integer boundaries, strict truncation behaviour, and
//! consumed-length accounting. Case counts are reduced under Miri.

use bincode_next::config::standard;
use bincode_next::decode_from_slice;
use bincode_next::encode_to_vec;
use proptest::prelude::*;

const fn cases() -> u32 {
    if cfg!(miri) { 4 } else { 512 }
}

macro_rules! for_each_config {
    ($body:ident) => {{
        $body!(standard());
        $body!(standard().with_big_endian());
        $body!(standard().with_fixed_int_encoding());
        $body!(standard().with_big_endian().with_fixed_int_encoding());
        $body!(standard().with_deterministic_bincode());
        $body!(standard().with_deterministic_bincode_length_first());
        $body!(standard().with_cbor_format());
        $body!(standard().with_deterministic_cbor());
    }};
}

proptest! {
    #![proptest_config(ProptestConfig { cases: cases(), .. ProptestConfig::default() })]

    /// Integers round-trip exactly and consume exactly what was written.
    #[test]
    fn prop_integers_exact(a in any::<u64>(), b in any::<i64>(), c in any::<u128>(), d in any::<i128>(), e in any::<u16>(), f in any::<i8>()) {
        let v = (a, b, c, d, e, f);
        macro_rules! check { ($cfg:expr) => {{
            let cfg = $cfg;
            let enc = encode_to_vec(&v, cfg).unwrap();
            let (dec, n): ((u64, i64, u128, i128, u16, i8), usize) = decode_from_slice(&enc, cfg).unwrap();
            prop_assert_eq!(dec, v);
            prop_assert_eq!(n, enc.len());
        }}; }
        for_each_config!(check);
    }

    /// Floats are bit-exact for non-NaN values (including -0.0 and subnormals);
    /// NaNs must decode to a NaN.
    #[test]
    fn prop_floats_bit_exact(x in any::<u64>(), y in any::<u32>()) {
        let (a, b) = (f64::from_bits(x), f32::from_bits(y));
        macro_rules! check { ($cfg:expr) => {{
            let cfg = $cfg;
            let enc = encode_to_vec(&(a, b), cfg).unwrap();
            let ((da, db), n): ((f64, f32), usize) = decode_from_slice(&enc, cfg).unwrap();
            prop_assert_eq!(n, enc.len());
            if a.is_nan() { prop_assert!(da.is_nan()); } else { prop_assert_eq!(da.to_bits(), a.to_bits()); }
            if b.is_nan() { prop_assert!(db.is_nan()); } else { prop_assert_eq!(db.to_bits(), b.to_bits()); }
        }}; }
        for_each_config!(check);
    }

    /// Every strict prefix of a valid encoding must fail to decode (never
    /// panic, never succeed with a shorter value).
    #[test]
    fn prop_truncation_always_errors(s in any::<String>(), v in proptest::collection::vec(any::<u32>(), 0..16)) {
        let val = (s, v);
        macro_rules! check { ($cfg:expr) => {{
            let cfg = $cfg;
            let enc = encode_to_vec(&val, cfg).unwrap();
            for cut in 0..enc.len() {
                prop_assert!(
                    decode_from_slice::<(String, Vec<u32>), _>(&enc[..cut], cfg).is_err(),
                    "prefix {} of {} decoded", cut, enc.len()
                );
            }
        }}; }
        for_each_config!(check);
    }

    /// Trailing bytes are left untouched and the consumed length is exact.
    #[test]
    fn prop_trailing_bytes_not_consumed(x in any::<u64>(), tail in proptest::collection::vec(any::<u8>(), 0..16)) {
        macro_rules! check { ($cfg:expr) => {{
            let cfg = $cfg;
            let mut enc = encode_to_vec(x, cfg).unwrap();
            let len = enc.len();
            enc.extend_from_slice(&tail);
            let (dec, n): (u64, usize) = decode_from_slice(&enc, cfg).unwrap();
            prop_assert_eq!(dec, x);
            prop_assert_eq!(n, len);
        }}; }
        for_each_config!(check);
    }

    /// Encoding is a pure function of the value (re-encoding is byte-stable).
    #[test]
    fn prop_encoding_is_stable(v in proptest::collection::vec(any::<(u8, i16, String)>(), 0..16)) {
        macro_rules! check { ($cfg:expr) => {{
            let cfg = $cfg;
            let a = encode_to_vec(&v, cfg).unwrap();
            let (d, _): (Vec<(u8, i16, String)>, usize) = decode_from_slice(&a, cfg).unwrap();
            prop_assert_eq!(&d, &v);
            prop_assert_eq!(encode_to_vec(&d, cfg).unwrap(), a);
        }}; }
        for_each_config!(check);
    }

    /// Arbitrary garbage must never panic any format's decoder.
    #[test]
    fn prop_garbage_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
        macro_rules! check { ($cfg:expr) => {{
            let cfg = $cfg.with_limit::<8192>();
            let _ = decode_from_slice::<(String, Vec<u64>, Option<i32>), _>(&bytes, cfg);
            let _ = decode_from_slice::<std::collections::HashMap<String, u8>, _>(&bytes, cfg);
        }}; }
        for_each_config!(check);
    }

    /// A configured byte limit is enforced: a limit smaller than the
    /// encoding must error rather than succeed.
    #[test]
    fn prop_limit_enforced(v in proptest::collection::vec(any::<u8>(), 8..64)) {
        let cfg = standard().with_limit::<4>();
        let enc = encode_to_vec(&v, standard()).unwrap();
        prop_assert!(decode_from_slice::<Vec<u8>, _>(&enc, cfg).is_err());
    }
}

#[test]
fn integer_boundaries_exact() {
    macro_rules! check {
        ($cfg:expr) => {{
            let cfg = $cfg;
            let v = (
                [
                    0u64,
                    1,
                    250,
                    251,
                    252,
                    65_535,
                    65_536,
                    u64::from(u32::MAX),
                    u64::MAX,
                ],
                [i64::MIN, -1, 0, 1, i64::MAX],
                [u128::MAX, 0],
                [i128::MIN, i128::MAX],
            );
            let enc = encode_to_vec(&v, cfg).unwrap();
            let (d, n): (([u64; 9], [i64; 5], [u128; 2], [i128; 2]), usize) =
                decode_from_slice(&enc, cfg).unwrap();
            assert_eq!(d, v);
            assert_eq!(n, enc.len());
        }};
    }
    for_each_config!(check);
}

#[test]
fn special_floats_exact() {
    for bits in [
        0x0000_0000_0000_0000u64, // +0
        0x8000_0000_0000_0000,    // -0
        0x0000_0000_0000_0001,    // smallest subnormal
        0x7FF0_0000_0000_0000,    // +inf
        0xFFF0_0000_0000_0000,    // -inf
        0x7FEF_FFFF_FFFF_FFFF,    // max
    ] {
        let x = f64::from_bits(bits);
        macro_rules! check {
            ($cfg:expr) => {{
                let cfg = $cfg;
                let enc = encode_to_vec(x, cfg).unwrap();
                let (d, _): (f64, usize) = decode_from_slice(&enc, cfg).unwrap();
                assert_eq!(d.to_bits(), bits, "bits {bits:#x}");
            }};
        }
        for_each_config!(check);
    }
}
