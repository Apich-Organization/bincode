//! Equivalence of the different encode/decode entry points.
//!
//! * `encode_to_vec`, `encode_into_slice` and `encode_into_std_write` must agree
//!   byte for byte, and an undersized slice must fail cleanly.
//! * Decoding through `std::io::Read` with arbitrary chunking (which defeats the
//!   slice-only `peek_read` fast paths) must agree with slice decoding on both
//!   valid *and* arbitrary input, including Ok/Err parity and consumed length.
#![no_main]
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use std::io::Read;

#[macro_use]
#[path = "../common/mod.rs"]
mod common;
use common::Msg;

/// Reader that hands out data in fuzzer-chosen chunk sizes.
struct Chunked<'a> {
    data: &'a [u8],
    pos: usize,
    chunks: &'a [u8],
    next: usize,
}

impl Read for Chunked<'_> {
    fn read(
        &mut self,
        buf: &mut [u8],
    ) -> std::io::Result<usize> {
        let pick = if self.chunks.is_empty() {
            1
        } else {
            self.chunks[self.next % self.chunks.len()]
        };
        let step = usize::from(pick) % 17 + 1;
        self.next += 1;
        let n = step.min(buf.len()).min(self.data.len() - self.pos);
        buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

fn encode_apis<C: bincode_next::config::Config>(
    msg: &Msg,
    cfg: C,
    chunks: &[u8],
) where
    C::Mode: bincode_next::config::InternalFingerprintGuard<Msg, C>,
    for<'a> C::Mode: bincode_next::config::InternalFingerprintGuard<&'a Msg, C>,
{
    let enc = bincode_next::encode_to_vec(msg, cfg).expect("encode_to_vec");

    // Exact-size slice must work and match; one byte short must fail, not panic.
    let mut buf = vec![0u8; enc.len()];
    let n = bincode_next::encode_into_slice(msg, &mut buf, cfg).expect("encode_into_slice");
    assert_eq!(n, enc.len());
    assert_eq!(buf, enc);
    if !enc.is_empty() {
        let mut short = vec![0u8; enc.len() - 1];
        assert!(bincode_next::encode_into_slice(msg, &mut short, cfg).is_err());
    }

    let mut sink = Vec::new();
    let w =
        bincode_next::encode_into_std_write(msg, &mut sink, cfg).expect("encode_into_std_write");
    assert_eq!(w, enc.len());
    assert_eq!(sink, enc);

    // Reader with arbitrary chunking decodes the same value and consumes everything.
    let mut rd = Chunked {
        data: &enc,
        pos: 0,
        chunks,
        next: 0,
    };
    let via_io: Msg =
        bincode_next::decode_from_std_read(&mut rd, cfg).expect("decode_from_std_read");
    assert_eq!(bincode_next::encode_to_vec(&via_io, cfg).unwrap(), enc);
    assert_eq!(rd.pos, enc.len(), "io reader over/under-read its own encoding");

    // A truncated stream must be an error.
    if !enc.is_empty() {
        let mut rd = Chunked {
            data: &enc[..enc.len() - 1],
            pos: 0,
            chunks,
            next: 0,
        };
        assert!(bincode_next::decode_from_std_read::<Msg, _, _>(&mut rd, cfg).is_err());
    }
}

fn raw_parity<C: bincode_next::config::Config>(
    raw: &[u8],
    cfg: C,
    chunks: &[u8],
) where
    C::Mode: bincode_next::config::InternalFingerprintGuard<Msg, C>,
    for<'a> C::Mode: bincode_next::config::InternalFingerprintGuard<&'a Msg, C>,
{
    let slice = bincode_next::decode_from_slice::<Msg, _>(raw, cfg);
    let mut rd = Chunked {
        data: raw,
        pos: 0,
        chunks,
        next: 0,
    };
    let io = bincode_next::decode_from_std_read::<Msg, _, _>(&mut rd, cfg);
    match (slice, io) {
        | (Ok((a, n)), Ok(b)) => {
            assert_eq!(
                bincode_next::encode_to_vec(&a, cfg).unwrap(),
                bincode_next::encode_to_vec(&b, cfg).unwrap(),
                "slice and io decode produced different values"
            );
            assert_eq!(n, rd.pos, "slice and io decode consumed different amounts");
        },
        | (Err(_), Err(_)) => {},
        | (a, b) => panic!(
            "slice/io disagree: slice ok={} io ok={}",
            a.is_ok(),
            b.is_ok()
        ),
    }
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(sel) = u8::arbitrary(&mut u) else {
        return;
    };
    let Ok(chunks) = Vec::<u8>::arbitrary(&mut u) else {
        return;
    };
    let structured = bool::arbitrary(&mut u).unwrap_or(true);
    use bincode_next::config::standard;
    if structured {
        let Ok(msg) = Msg::arbitrary(&mut u) else {
            return;
        };
        with_bincode_config!(sel, |cfg| {
            encode_apis(&msg, cfg.with_limit::<16777216>(), &chunks);
        });
    } else {
        let raw = u.take_rest();
        with_bincode_config!(sel, |cfg| {
            raw_parity(raw, cfg.with_limit::<1048576>(), &chunks);
        });
        raw_parity(
            raw,
            standard().with_cbor_format().with_limit::<1048576>(),
            &chunks,
        );
    }
});
