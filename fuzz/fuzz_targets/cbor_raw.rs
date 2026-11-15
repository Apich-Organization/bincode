//! Raw-bytes decoding for the CBOR format family.
//!
//! Complements `cbor` (single document type) with many more shapes: indefinite
//! length containers, tags, half/single/double floats, bignums, byte-string vs
//! array sequences. Under every CBOR option combination we assert limit
//! monotonicity (a value accepted under a small limit must be accepted under a
//! larger one) and fixed-point re-encoding.
#![no_main]
use libfuzzer_sys::fuzz_target;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[macro_use]
#[path = "../common/mod.rs"]
mod common;
use common::Kind;
use common::Msg;

macro_rules! run {
    ($rest:expr, $base:expr) => {
        check_types!(
            $rest,
            $base,
            true,
            [
                Vec<u8>, Vec<u32>, Vec<i64>, Vec<u128>, Vec<f64>, Vec<Vec<u8>>, Vec<String>,
                BTreeMap<String, Vec<i32>>, BTreeSet<Vec<u8>>, Option<Vec<i64>>,
                Result<Vec<u16>, String>, String, Kind, Msg,
            ]
        )
    };
}

fuzz_target!(|data: &[u8]| {
    let Some((&sel, rest)) = data.split_first() else {
        return;
    };
    use bincode_next::config::standard;
    match sel % 4 {
        | 0 => run!(rest, standard().with_cbor_format()),
        | 1 => run!(rest, standard().with_deterministic_cbor()),
        // (det, preferred-serialization, canonical NaN, normalise -0, allow indefinite, strict tags)
        | 2 => {
            run!(
                rest,
                standard().with_cbor_options::<0, true, true, true, true, false>()
            )
        },
        | _ => {
            run!(
                rest,
                standard().with_cbor_options::<2, true, true, true, false, true>()
            )
        },
    }
});
