//! Optional features and non-wire configuration knobs.
//!
//! * `Fingerprint` (schema hash) with several seeds / endianness: matching config
//!   round-trips, any mismatch (seed, config, type shape) is a clean error, and
//!   arbitrary bytes never panic the fingerprint check.
//! * `StaticSize` / `BoundedVec` / `BoundedString`: encodings never exceed
//!   `MAX_SIZE`, fit the static buffer APIs, over-capacity values are rejected on
//!   encode and decode, arbitrary bytes never panic.
//! * Borrowed decoding (`&str`, `&[u8]`) agrees with owned decoding and only ever
//!   points inside the input buffer.
#![no_main]
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use bincode_next::bounded::BoundedString;
use bincode_next::bounded::BoundedVec;
use bincode_next::Fingerprint;
use bincode_next::StaticSize;
use libfuzzer_sys::fuzz_target;

#[macro_use]
#[path = "../common/mod.rs"]
mod common;

#[derive(Arbitrary, Fingerprint, bincode_next::Encode, bincode_next::Decode, PartialEq, Debug, Clone)]
struct Rec {
    a: u32,
    b: String,
    c: Vec<u16>,
    e: Option<i64>,
}

/// Same field types as `Rec` but a different schema.
#[derive(Fingerprint, bincode_next::Encode, bincode_next::Decode, PartialEq, Debug, Clone)]
struct RecOther {
    a: u32,
    b: String,
    c: Vec<u16>,
    e: Option<i64>,
    extra: u8,
}

#[derive(StaticSize, bincode_next::Encode, bincode_next::Decode, PartialEq, Debug, Clone)]
struct Fixed {
    a: u32,
    b: u64,
    c: i128,
    d: BoundedString<8>,
    e: BoundedVec<u16, 4>,
    f: Option<u128>,
    g: [u8; 3],
}

#[derive(Arbitrary, Debug)]
struct FixedSeed {
    a: u32,
    b: u64,
    c: i128,
    d: String,
    e: Vec<u16>,
    f: Option<u128>,
    g: [u8; 3],
}

impl FixedSeed {
    fn build(
        self,
        oversize: bool,
    ) -> Fixed {
        let mut d = self.d;
        let mut e = self.e;
        if !oversize {
            while d.len() > 8 {
                d.pop();
            }
            e.truncate(4);
        }
        Fixed {
            a: self.a,
            b: self.b,
            c: self.c,
            d: BoundedString(d),
            e: BoundedVec(e),
            f: self.f,
            g: self.g,
        }
    }
}

const CAP: usize = 128;

fn static_size_checks<C: bincode_next::config::Config>(
    seed: FixedSeed,
    raw: &[u8],
    cfg: C,
) where
    C::Mode: bincode_next::config::InternalFingerprintGuard<Fixed, C>,
    for<'a> C::Mode: bincode_next::config::InternalFingerprintGuard<&'a Fixed, C>,
{
    // In-bounds values encode within MAX_SIZE and round-trip through the static APIs.
    let ok = seed.build(false);
    let enc = bincode_next::encode_to_vec(&ok, cfg).expect("bounded value must encode");
    assert!(
        enc.len() <= <Fixed as StaticSize>::MAX_SIZE,
        "encoded {} > MAX_SIZE {}",
        enc.len(),
        <Fixed as StaticSize>::MAX_SIZE
    );
    let mut buf = [0u8; CAP];
    let n = bincode_next::encode_into_slice(&ok, &mut buf, cfg).expect("fits static buffer");
    assert_eq!(&buf[..n], &enc[..]);
    let back: Fixed = bincode_next::decode_from_slice_static(&buf, cfg).expect("static decode");
    assert_eq!(back, ok);

    // Over-capacity values must be rejected on encode.
    let big = FixedSeed {
        a: 0,
        b: 0,
        c: 0,
        d: "0123456789".into(),
        e: vec![0; 5],
        f: None,
        g: [0; 3],
    }
    .build(true);
    assert!(bincode_next::encode_to_vec(&big, cfg).is_err());

    // Arbitrary bytes: never panic; anything accepted respects the capacity bounds.
    let mut arr = [0u8; CAP];
    let m = raw.len().min(CAP);
    arr[..m].copy_from_slice(&raw[..m]);
    if let Ok(v) = bincode_next::decode_from_slice_static::<Fixed, CAP, _>(&arr, cfg) {
        assert!(v.d.0.len() <= 8 && v.e.0.len() <= 4, "decoded value exceeds capacity");
        assert!(bincode_next::encode_to_vec(&v, cfg).is_ok());
    }
}

fn borrow_checks<C: bincode_next::config::Config>(
    raw: &[u8],
    cfg: C,
) where
    C::Mode: bincode_next::config::InternalFingerprintGuard<(String, Vec<u8>), C>,
    for<'a> C::Mode: bincode_next::config::InternalFingerprintGuard<(&'a str, &'a [u8]), C>,
{
    let borrowed = bincode_next::borrow_decode_from_slice::<(&str, &[u8]), _>(raw, cfg);
    let owned = bincode_next::decode_from_slice::<(String, Vec<u8>), _>(raw, cfg);
    match (borrowed, owned) {
        | (Ok(((s, b), n1)), Ok(((os, ob), n2))) => {
            assert_eq!(s, os);
            assert_eq!(b, &ob[..]);
            assert_eq!(n1, n2);
            let range = raw.as_ptr_range();
            for (p, len) in [(s.as_ptr(), s.len()), (b.as_ptr(), b.len())] {
                // Empty slices may legitimately use a dangling pointer.
                assert!(
                    len == 0 || (p >= range.start && p <= range.end),
                    "borrow points outside input"
                );
            }
        },
        | (Err(_), Err(_)) => {},
        | (a, b) => panic!("borrow/owned mismatch: borrow ok={} owned ok={}", a.is_ok(), b.is_ok()),
    }
}

macro_rules! fingerprint_checks {
    ($rec:expr, $raw:expr, $base:expr, $other_seed:expr) => {{
        let base = $base;
        let cfg = base.with_fingerprint();
        // Raw (attacker-shaped) input uses a bounded limit: with NoLimit a huge length
        // prefix is allowed to request an arbitrarily large allocation.
        let raw_cfg = base.with_fingerprint().with_limit::<1048576>();
        let enc = bincode_next::encode_to_vec($rec, cfg).expect("fingerprint encode");
        let (back, n): (Rec, usize) =
            bincode_next::decode_from_slice(&enc, cfg).expect("fingerprint decode");
        assert_eq!(&back, $rec);
        assert_eq!(n, enc.len());

        // Wrong seed / no fingerprint / different schema must fail cleanly.
        assert!(bincode_next::decode_from_slice::<Rec, _>(
            &enc,
            base.with_fingerprint_and_seed::<0xDEAD_BEEF_u64>()
        )
        .is_err());
        assert!(bincode_next::decode_from_slice::<RecOther, _>(&enc, cfg).is_err());
        let _ = $other_seed;

        // Arbitrary bytes behind a fingerprint check: no panic; accepted => stable.
        if let Ok((v, _)) = bincode_next::decode_from_slice::<Rec, _>($raw, raw_cfg) {
            let e = bincode_next::encode_to_vec(&v, raw_cfg).unwrap();
            assert_eq!(bincode_next::decode_from_slice::<Rec, _>(&e, raw_cfg).unwrap().0, v);
        }
        let _ = bincode_next::decode_from_slice::<Rec, _>($raw, base.with_fingerprint_and_seed::<7>().with_limit::<1048576>());
    }};
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(sel) = u8::arbitrary(&mut u) else {
        return;
    };
    let Ok(rec) = Rec::arbitrary(&mut u) else {
        return;
    };
    let Ok(seed) = FixedSeed::arbitrary(&mut u) else {
        return;
    };
    let raw = u.take_rest();
    use bincode_next::config::standard;

    // Fingerprint: four endian/int combinations.
    match sel & 3 {
        | 0 => fingerprint_checks!(&rec, raw, standard().with_little_endian().with_variable_int_encoding(), 0),
        | 1 => fingerprint_checks!(&rec, raw, standard().with_big_endian().with_variable_int_encoding(), 0),
        | 2 => fingerprint_checks!(&rec, raw, standard().with_little_endian().with_fixed_int_encoding(), 0),
        | _ => fingerprint_checks!(&rec, raw, standard().with_big_endian().with_fixed_int_encoding(), 0),
    }

    with_bincode_config!(sel >> 2, |cfg| {
        static_size_checks(seed, raw, cfg.with_limit::<1048576>());
        borrow_checks(raw, cfg.with_limit::<1048576>());
    });
});
