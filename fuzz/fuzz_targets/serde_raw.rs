//! The serde integration path.
//!
//! * structured values round-trip through `bincode_next::serde` in every config,
//! * arbitrary bytes never panic the serde deserializer and obey the same
//!   fixed-point / limit-monotonicity rules as the native decoder,
//! * the serde and native paths agree on both validity and value for the shared
//!   type (they implement the same wire format for plain structs).
#![no_main]
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;

#[macro_use]
#[path = "../common/mod.rs"]
mod common;
use common::Msg;

macro_rules! serde_checks {
    ($data:expr, $msg:expr, $cfg:expr) => {{
        let cfg = $cfg;
        let big = cfg.with_limit::<1048576>();
        let small = cfg.with_limit::<384>();
        let huge = cfg.with_limit::<67108864>();

        // 1. Structured round trip through serde.
        if let Some(msg) = $msg {
            let enc = bincode_next::serde::encode_to_vec(msg, huge).expect("serde encode");
            let (back, n): (Msg, usize) =
                bincode_next::serde::decode_from_slice(&enc, huge).expect("serde decode");
            assert_eq!(n, enc.len());
            assert_eq!(
                bincode_next::serde::encode_to_vec(&back, huge).unwrap(),
                enc
            );
        }

        // 2. Raw bytes: limit monotonicity + fixed point.
        let rs = bincode_next::serde::decode_from_slice::<Msg, _>($data, small);
        let rb = bincode_next::serde::decode_from_slice::<Msg, _>($data, big);
        let rh = bincode_next::serde::decode_from_slice::<Msg, _>($data, huge);
        if let Ok((_, a)) = &rs {
            assert_eq!(
                rb.as_ref()
                    .expect("serde limit not monotonic (small->big)")
                    .1,
                *a
            );
        }
        if let Ok((_, a)) = &rb {
            assert_eq!(
                rh.as_ref()
                    .expect("serde limit not monotonic (big->huge)")
                    .1,
                *a
            );
        }
        if let Ok((v, n)) = &rh {
            assert!(*n <= $data.len());
            let enc = bincode_next::serde::encode_to_vec(v, huge).unwrap();
            let (v2, n2): (Msg, usize) =
                bincode_next::serde::decode_from_slice(&enc, huge).unwrap();
            assert_eq!(n2, enc.len());
            assert_eq!(bincode_next::serde::encode_to_vec(&v2, huge).unwrap(), enc);
        }

        // 3. Serde and native decoders must agree on arbitrary input.
        let native = bincode_next::decode_from_slice::<Msg, _>($data, huge);
        match (&rh, &native) {
            | (Ok((a, na)), Ok((b, nb))) => {
                assert_eq!(na, nb, "serde/native consumed different byte counts");
                assert_eq!(
                    bincode_next::encode_to_vec(a, huge).unwrap(),
                    bincode_next::encode_to_vec(b, huge).unwrap(),
                    "serde/native decoded different values"
                );
            },
            | (Err(_), Err(_)) => {},
            // The serde `MapAccess` path is generic over the key type, so it cannot
            // re-check key uniqueness in deterministic mode; only the native
            // `BTreeMap`/`HashMap` decoders reject duplicate keys. Known, documented gap.
            | (Ok(_), Err(bincode_next::error::DecodeError::DuplicateMapKey)) => {},
            | (s, n) => {
                panic!(
                    "serde/native validity mismatch: serde ok={} native ok={}",
                    s.is_ok(),
                    n.is_ok()
                )
            },
        }
    }};
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(sel) = u8::arbitrary(&mut u) else {
        return;
    };
    let msg = Msg::arbitrary(&mut u).ok();
    let raw = u.take_rest();
    with_bincode_config!(sel, |cfg| serde_checks!(raw, msg.as_ref(), cfg));
});
