//! Shared types and property checks for the "raw bytes" fuzz targets.
//!
//! These targets feed arbitrary bytes straight into decoders (so the fuzzer
//! explores the decoder state machine instead of only valid encodings) and then
//! assert format-independent invariants that must hold for *any* input.
#![allow(dead_code)]

use bincode_next::config::Config;
use bincode_next::Decode;
use bincode_next::Encode;
use std::collections::BTreeMap;
use std::collections::VecDeque;

#[derive(
    Debug,
    Clone,
    arbitrary::Arbitrary,
    bincode_next::Encode,
    bincode_next::Decode,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Kind {
    Unit,
    Tuple(u32, i64),
    Struct {
        xs: Vec<i64>,
        name: Option<String>,
    },
    Nested(Box<Kind>),
}

/// A struct touching every primitive decode fast path (varint batches, SIMD
/// scans, fixed-width bulk copies, byte vs. array sequences, nested containers).
#[derive(
    Debug,
    Clone,
    arbitrary::Arbitrary,
    bincode_next::Encode,
    bincode_next::Decode,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Msg {
    pub a: u8,
    pub b: u16,
    pub c: u32,
    pub d: u64,
    pub e: u128,
    pub f: usize,
    pub g: i8,
    pub h: i16,
    pub i: i32,
    pub j: i64,
    pub k: i128,
    pub l: isize,
    pub flag: bool,
    pub ch: char,
    pub fl: f32,
    pub db: f64,
    pub vu8: Vec<u8>,
    pub vi8: Vec<i8>,
    pub vu16: Vec<u16>,
    pub vi16: Vec<i16>,
    pub vu32: Vec<u32>,
    pub vi32: Vec<i32>,
    pub vu64: Vec<u64>,
    pub vi64: Vec<i64>,
    pub vu128: Vec<u128>,
    pub vi128: Vec<i128>,
    pub vusize: Vec<usize>,
    pub visize: Vec<isize>,
    pub vf32: Vec<f32>,
    pub vf64: Vec<f64>,
    pub vbool: Vec<bool>,
    pub vchar: Vec<char>,
    pub s: String,
    pub opt: Option<Vec<u32>>,
    pub nested: Vec<Vec<i16>>,
    pub arr: [u16; 4],
    pub tup: (u8, i64, String),
    pub kind: Kind,
    pub kinds: Vec<Kind>,
    pub map: BTreeMap<u16, Vec<u32>>,
    pub dq: VecDeque<u32>,
    pub boxed: Box<[u32]>,
    pub res: Result<u32, String>,
}

/// Run `$body` once per entry of the wire-config matrix, with `$cfg` bound to a
/// configuration of that shape and `$sel` (a byte) choosing the entry.
#[macro_export]
macro_rules! with_bincode_config {
    ($sel:expr, |$cfg:ident| $body:expr) => {{
        use bincode_next::config::standard;
        match $sel & 0b111 {
            | 0 => {
                let $cfg = standard().with_little_endian().with_variable_int_encoding();
                $body
            },
            | 1 => {
                let $cfg = standard().with_big_endian().with_variable_int_encoding();
                $body
            },
            | 2 => {
                let $cfg = standard().with_little_endian().with_fixed_int_encoding();
                $body
            },
            | 3 => {
                let $cfg = standard().with_big_endian().with_fixed_int_encoding();
                $body
            },
            | 4 => {
                let $cfg = standard()
                    .with_little_endian()
                    .with_variable_int_encoding()
                    .with_bit_packing();
                $body
            },
            | 5 => {
                let $cfg = standard()
                    .with_big_endian()
                    .with_fixed_int_encoding()
                    .with_bit_packing();
                $body
            },
            | 6 => {
                let $cfg = bincode_next::config::legacy();
                $body
            },
            | _ => {
                let $cfg = standard().with_deterministic_bincode();
                $body
            },
        }
    }};
}

/// Runs [`check_decode`] for every listed type under (small, large, huge-but-bounded) limits.
///
/// The largest limit is deliberately finite (64 MiB): a `NoLimit` decode of attacker
/// controlled lengths is allowed to allocate arbitrarily much, which would only make the
/// fuzzer report out-of-memory for behaviour that is by design.
#[macro_export]
macro_rules! check_types {
    ($data:expr, $base:expr, $strict:expr, [$($t:ty),+ $(,)?]) => {
        $crate::check_types!(@go $data, $base, $strict, true, [$($t),+])
    };
    // Same, for types containing `HashMap`/`HashSet` (iteration order is random per
    // instance, so byte-for-byte re-encoding stability cannot be asserted).
    ($data:expr, $base:expr, $strict:expr, unordered, [$($t:ty),+ $(,)?]) => {
        $crate::check_types!(@go $data, $base, $strict, false, [$($t),+])
    };
    (@go $data:expr, $base:expr, $strict:expr, $fp:expr, [$($t:ty),+ $(,)?]) => {{
        let base = $base;
        $(
            $crate::common::check_decode::<$t, _, _, _>(
                $data,
                base.with_limit::<384>(),
                base.with_limit::<1048576>(),
                base.with_limit::<67108864>(),
                $strict,
                $fp,
            );
        )+
    }};
}

/// Properties every decoder must satisfy on arbitrary bytes.
///
/// * never panics (implicit),
/// * a decode that succeeds under a small limit also succeeds, identically,
///   under larger limits (limits only ever reject, never change results),
/// * consumed byte count is within the input,
/// * re-encoding a decoded value is decodable, consumes exactly its encoding,
///   and is a fixed point (`enc(dec(enc(v))) == enc(v)`); bytes are compared
///   instead of values so NaN payloads do not break equality,
/// * decoding is self-delimiting: the exact consumed prefix decodes the same
///   and anything shorter is an error.
pub fn check_decode<T, CS, CB, CN>(
    data: &[u8],
    small: CS,
    big: CB,
    none: CN,
    strict_prefix: bool,
    fixed_point: bool,
) where
    T: Decode<()> + Encode,
    CS: Config,
    CB: Config,
    CN: Config,
    CS::Mode: bincode_next::config::InternalFingerprintGuard<T, CS>,
    CB::Mode: bincode_next::config::InternalFingerprintGuard<T, CB>,
    CN::Mode: bincode_next::config::InternalFingerprintGuard<T, CN>,
    for<'a> CN::Mode: bincode_next::config::InternalFingerprintGuard<&'a T, CN>,
{
    let r_small = bincode_next::decode_from_slice::<T, _>(data, small);
    let r_big = bincode_next::decode_from_slice::<T, _>(data, big);
    let r_none = bincode_next::decode_from_slice::<T, _>(data, none);

    // Limit monotonicity: Ok(small) => Ok(big) => Ok(none), same consumption.
    if let Ok((_, n_small)) = &r_small {
        let (_, n_big) = r_big
            .as_ref()
            .unwrap_or_else(|e| panic!("limit not monotonic: small ok but big failed: {e:?}"));
        assert_eq!(n_small, n_big, "consumed bytes depend on limit");
    }
    if let Ok((_, n_big)) = &r_big {
        let (_, n_none) = r_none
            .as_ref()
            .unwrap_or_else(|e| panic!("limit not monotonic: big ok but no-limit failed: {e:?}"));
        assert_eq!(n_big, n_none, "consumed bytes depend on limit");
    }

    let Ok((value, consumed)) = r_none else {
        return;
    };
    assert!(consumed <= data.len(), "consumed {consumed} > input {}", data.len());

    // Re-encode and decode again; the encoding must be a fixed point.
    let enc = bincode_next::encode_to_vec(&value, none).expect("re-encode of decoded value");
    let (value2, n2) =
        bincode_next::decode_from_slice::<T, _>(&enc, none).expect("re-decode of own encoding");
    assert_eq!(n2, enc.len(), "re-decode did not consume its whole encoding");
    let enc2 = bincode_next::encode_to_vec(&value2, none).expect("second re-encode");
    if fixed_point {
        assert_eq!(enc, enc2, "encoding is not a fixed point");
    }

    // Self-delimiting: trailing garbage is ignored, truncation is an error.
    let (_, n3) = bincode_next::decode_from_slice::<T, _>(&data[..consumed], none)
        .expect("exact consumed prefix must decode");
    assert_eq!(n3, consumed);
    if strict_prefix && consumed > 0 {
        assert!(
            bincode_next::decode_from_slice::<T, _>(&data[..consumed - 1], none).is_err(),
            "a strict prefix of a complete encoding decoded successfully"
        );
    }
}
