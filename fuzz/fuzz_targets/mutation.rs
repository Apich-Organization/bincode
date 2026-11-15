//! Structure-aware mutation fuzzing.
//!
//! Starting from a *valid* encoding of an arbitrary value, apply fuzzer-chosen
//! byte edits (flip, insert, delete, duplicate, truncate, overwrite with hostile
//! varint / length markers) and decode. Raw-byte fuzzers need many executions to
//! rediscover a valid prefix; mutating real encodings reaches deep decoder states
//! (corrupted length prefixes, bad enum tags, invalid UTF-8, oversized lengths
//! followed by little data) immediately.
//!
//! Properties: no panic / abort / huge allocation under a bounded limit, limit
//! monotonicity, fixed-point re-encoding, and unmodified encodings round-trip.
#![no_main]
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;

#[macro_use]
#[path = "../common/mod.rs"]
mod common;
use common::Msg;

#[derive(Arbitrary, Debug)]
enum Edit {
    Flip {
        at: u16,
        mask: u8,
    },
    Set {
        at: u16,
        val: u8,
    },
    Insert {
        at: u16,
        bytes: Vec<u8>,
    },
    Delete {
        at: u16,
        len: u8,
    },
    Truncate {
        at: u16,
    },
    Dup {
        at: u16,
        len: u8,
    },
    /// Overwrite with a maximal varint / length marker to simulate hostile lengths.
    Hostile {
        at: u16,
        kind: u8,
    },
}

fn apply(
    buf: &mut Vec<u8>,
    e: &Edit,
) {
    let len = buf.len();
    let pos = |at: u16| {
        if len == 0 {
            0
        } else {
            usize::from(at) % len
        }
    };
    match e {
        | Edit::Flip { at, mask } => {
            if len > 0 {
                buf[pos(*at)] ^= mask;
            }
        },
        | Edit::Set { at, val } => {
            if len > 0 {
                buf[pos(*at)] = *val;
            }
        },
        | Edit::Insert { at, bytes } => {
            let p = pos(*at);
            buf.splice(p..p, bytes.iter().copied().take(64));
        },
        | Edit::Delete { at, len: l } => {
            let p = pos(*at);
            let end = (p + usize::from(*l)).min(len);
            buf.drain(p..end);
        },
        | Edit::Truncate { at } => buf.truncate(pos(*at)),
        | Edit::Dup { at, len: l } => {
            let p = pos(*at);
            let end = (p + usize::from(*l)).min(len);
            let chunk: Vec<u8> = buf[p..end].to_vec();
            buf.splice(p..p, chunk);
        },
        | Edit::Hostile { at, kind } => {
            let p = pos(*at);
            let marker: &[u8] = match kind % 6 {
                | 0 => &[0xFB, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], // u64::MAX varint
                | 1 => &[0xFA, 0xFF, 0xFF, 0xFF, 0xFF],                         // u32::MAX varint
                | 2 => &[0xFC, 0xFF, 0xFF, 0xFF, 0x7F],                         // 2^31-1
                | 3 => &[0x1B, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], // CBOR u64::MAX
                | 4 => &[0x9F], // CBOR indefinite array
                | _ => &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            };
            for (i, b) in marker.iter().enumerate() {
                if p + i < buf.len() {
                    buf[p + i] = *b;
                } else {
                    buf.push(*b);
                }
            }
        },
    }
}

macro_rules! mutate_and_check {
    ($msg:expr, $edits:expr, $cfg:expr) => {{
        let cfg = $cfg;
        let huge = cfg.with_limit::<67108864>();
        let valid = bincode_next::encode_to_vec($msg, huge).unwrap();

        // Unmodified encoding must round-trip exactly.
        let (back, n) = bincode_next::decode_from_slice::<Msg, _>(&valid, huge)
            .expect("valid encoding must decode");
        assert_eq!(n, valid.len());
        assert_eq!(bincode_next::encode_to_vec(&back, huge).unwrap(), valid);

        let mut mutated = valid;
        for e in $edits {
            apply(&mut mutated, e);
        }
        check_types!(&mutated, cfg, true, [Msg]);
    }};
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(sel) = u8::arbitrary(&mut u) else {
        return;
    };
    let Ok(msg) = Msg::arbitrary(&mut u) else {
        return;
    };
    let Ok(edits) = Vec::<Edit>::arbitrary(&mut u) else {
        return;
    };
    let edits = &edits[..edits.len().min(8)];
    use bincode_next::config::standard;
    match sel >> 4 {
        | 0..=11 => with_bincode_config!(sel, |cfg| mutate_and_check!(&msg, edits, cfg)),
        | 12..=13 => mutate_and_check!(&msg, edits, standard().with_cbor_format()),
        | _ => mutate_and_check!(&msg, edits, standard().with_deterministic_cbor()),
    }
});
