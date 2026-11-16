//! Raw-bytes decoding across the whole wire-config matrix.
//!
//! The first byte selects the configuration (endianness, int encoding,
//! bit-packing, legacy, deterministic); the rest is fed straight to many
//! decoders. Checks limit monotonicity, fixed-point re-encoding and
//! self-delimiting behaviour (see `common::check_decode`).
#![no_main]
use libfuzzer_sys::fuzz_target;
use std::collections::BTreeMap;
use std::collections::VecDeque;

#[macro_use]
#[path = "../common/mod.rs"]
mod common;
use common::Kind;
use common::Msg;

fuzz_target!(|data: &[u8]| {
    let Some((&sel, rest)) = data.split_first() else {
        return;
    };
    with_bincode_config!(sel, |cfg| {
        check_types!(
            rest,
            cfg,
            true,
            [
                // Primitive sequences hit the varint batch / SIMD / bulk-copy fast paths.
                Vec<u8>, Vec<i8>, Vec<u16>, Vec<i16>, Vec<u32>, Vec<i32>, Vec<u64>, Vec<i64>,
                Vec<u128>, Vec<i128>, Vec<usize>, Vec<isize>, Vec<f32>, Vec<f64>, Vec<bool>,
                Vec<char>,
                // Nested / generic containers.
                Vec<Vec<u32>>, Vec<Vec<i16>>, Vec<String>, VecDeque<u64>, Box<[u32]>,
                BTreeMap<u16, Vec<u32>>, Option<Vec<i64>>, Result<Vec<u16>, String>,
                (u8, Vec<i32>, String), [u16; 4],
                // Scalars and text.
                u128, i128, f64, char, String,
                // Enums, recursion, and the kitchen-sink struct.
                Kind, Msg,
            ]
        );
    });
});
