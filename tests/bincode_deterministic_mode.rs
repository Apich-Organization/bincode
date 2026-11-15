#![cfg(all(feature = "std", feature = "derive"))]

//! Tests for `BincodeDeterministicMode` (Core / LengthFirst), including
//! property-based tests. Case counts are reduced under Miri.

use bincode_next::config::BincodeDeterministicMode;
use bincode_next::config::Config;
use bincode_next::config::Format;
use bincode_next::config::standard;
use bincode_next::decode_from_slice;
use bincode_next::encode_to_vec;
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;

const fn cases() -> u32 {
    if cfg!(miri) { 4 } else { 512 }
}

// Concrete types (not `impl Config`) so the fingerprint-guard bounds resolve.
const fn core_cfg() -> bincode_next::config::Configuration<
    bincode_next::config::LittleEndian,
    bincode_next::config::Varint,
    bincode_next::config::NoLimit,
    bincode_next::config::SkipBitPacking,
    bincode_next::config::LsbFirst,
    bincode_next::config::FingerprintDisabled,
    bincode_next::config::BincodeConfig<1>,
> {
    standard().with_bincode_deterministic_mode::<1>()
}

const fn lf_cfg() -> bincode_next::config::Configuration<
    bincode_next::config::LittleEndian,
    bincode_next::config::Varint,
    bincode_next::config::NoLimit,
    bincode_next::config::SkipBitPacking,
    bincode_next::config::LsbFirst,
    bincode_next::config::FingerprintDisabled,
    bincode_next::config::BincodeConfig<2>,
> {
    standard().with_deterministic_bincode_length_first()
}

/// Key whose bytewise order and length-first order disagree:
/// `Ok((a,b,c))` encodes as `[0,a,b,c]`, `Err(x)` as `[1,x]`.
type Key = Result<(u8, u8, u8), u8>;

#[test]
fn options_report_expected_mode() {
    assert_eq!(
        standard().bincode_options().deterministic_mode,
        BincodeDeterministicMode::None
    );
    assert_eq!(
        standard()
            .with_deterministic_bincode()
            .bincode_options()
            .deterministic_mode,
        BincodeDeterministicMode::Core
    );
    assert_eq!(
        lf_cfg().bincode_options().deterministic_mode,
        BincodeDeterministicMode::LengthFirst
    );
    assert_eq!(
        standard().with_bincode_deterministic_mode::<0>().format(),
        Format::Bincode
    );
    assert_eq!(core_cfg().format(), Format::BincodeDeterministic);
    assert_eq!(lf_cfg().format(), Format::BincodeDeterministic);
}

#[test]
fn legacy_deterministic_equals_core() {
    let mut m = HashMap::new();
    for i in 0..50u32 {
        m.insert(i.wrapping_mul(2_654_435_761), i);
    }
    let a = encode_to_vec(&m, standard().with_deterministic_bincode()).unwrap();
    let b = encode_to_vec(&m, core_cfg()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn core_and_length_first_order_differently() {
    let set: HashSet<Key> = [Ok((1, 2, 3)), Err(9)].into_iter().collect();

    let core = encode_to_vec(&set, core_cfg()).unwrap();
    // len=2, then [0,1,2,3] (Ok first bytewise), then [1,9]
    assert_eq!(core, vec![2, 0, 1, 2, 3, 1, 9]);

    let lf = encode_to_vec(&set, lf_cfg()).unwrap();
    // shorter encoding first
    assert_eq!(lf, vec![2, 1, 9, 0, 1, 2, 3]);

    // Both round-trip under their own config.
    let (d, n): (HashSet<Key>, usize) = decode_from_slice(&core, core_cfg()).unwrap();
    assert_eq!((d, n), (set.clone(), core.len()));
    let (d, n): (HashSet<Key>, usize) = decode_from_slice(&lf, lf_cfg()).unwrap();
    assert_eq!((d, n), (set, lf.len()));
}

#[test]
fn length_first_map_ordering() {
    let mut m: HashMap<Key, u8> = HashMap::new();
    m.insert(Ok((0, 0, 0)), 1);
    m.insert(Err(200), 2);
    m.insert(Err(3), 3);
    let lf = encode_to_vec(&m, lf_cfg()).unwrap();
    assert_eq!(lf, vec![3, 1, 3, 3, 1, 200, 2, 0, 0, 0, 0, 1]);
}

#[test]
fn binary_heap_is_canonical() {
    let a: BinaryHeap<u32> = [5, 1, 9, 3, 7].into_iter().collect();
    let b: BinaryHeap<u32> = [9, 7, 5, 3, 1].into_iter().collect();
    assert_eq!(
        encode_to_vec(&a, core_cfg()).unwrap(),
        encode_to_vec(&b, core_cfg()).unwrap()
    );
    assert_eq!(
        encode_to_vec(&a, lf_cfg()).unwrap(),
        encode_to_vec(&b, lf_cfg()).unwrap()
    );
}

#[test]
fn length_first_output_is_plain_bincode() {
    let mut m = BTreeMap::new();
    m.insert(1u64, "a".to_string());
    m.insert(70_000u64, "b".to_string());
    let enc = encode_to_vec(&m, lf_cfg()).unwrap();
    let (dec, n): (BTreeMap<u64, String>, usize) = decode_from_slice(&enc, standard()).unwrap();
    assert_eq!(n, enc.len());
    assert_eq!(dec, m);
}

#[test]
fn deterministic_modes_reject_duplicate_keys() {
    let mut p = Vec::new();
    p.extend(encode_to_vec(2u64, standard()).unwrap());
    for _ in 0..2 {
        p.extend(encode_to_vec(1u32, standard()).unwrap());
        p.extend(encode_to_vec(1u32, standard()).unwrap());
    }
    assert!(decode_from_slice::<HashMap<u32, u32>, _>(&p, lf_cfg()).is_err());
    assert!(decode_from_slice::<BTreeMap<u32, u32>, _>(&p, lf_cfg()).is_err());
    assert!(decode_from_slice::<HashMap<u32, u32>, _>(&p, core_cfg()).is_err());
    // Plain bincode still accepts it.
    assert!(decode_from_slice::<HashMap<u32, u32>, _>(&p, standard()).is_ok());
}

#[test]
fn config_hash_differs_per_mode() {
    use bincode_next::config::InternalConfigFingerprint;
    fn h<C: InternalConfigFingerprint>(_: &C) -> u64 {
        C::CONFIG_HASH
    }
    let none = h(&standard());
    let core = h(&core_cfg());
    let lf = h(&lf_cfg());
    assert_ne!(none, core);
    assert_ne!(core, lf);
    assert_ne!(none, lf);
}

proptest! {
    #![proptest_config(ProptestConfig { cases: cases(), .. ProptestConfig::default() })]

    /// Insertion order never changes the encoding.
    #[test]
    fn prop_insertion_order_independent(
        entries in proptest::collection::vec((any::<Key>(), any::<i64>()), 0..40)
    ) {
        let mut dedup: BTreeMap<Key, i64> = BTreeMap::new();
        for (k, v) in entries { dedup.insert(k, v); }
        let fwd: HashMap<Key, i64> = dedup.iter().map(|(k, v)| (*k, *v)).collect();
        let rev: HashMap<Key, i64> = dedup.iter().rev().map(|(k, v)| (*k, *v)).collect();
        prop_assert_eq!(
            encode_to_vec(&fwd, core_cfg()).unwrap(),
            encode_to_vec(&rev, core_cfg()).unwrap()
        );
        prop_assert_eq!(
            encode_to_vec(&fwd, lf_cfg()).unwrap(),
            encode_to_vec(&rev, lf_cfg()).unwrap()
        );
        // And the same as the BTreeMap in the same mode.
        prop_assert_eq!(
            encode_to_vec(&fwd, lf_cfg()).unwrap(),
            encode_to_vec(&dedup, lf_cfg()).unwrap()
        );
    }

    #[test]
    fn prop_roundtrip(
        entries in proptest::collection::vec((any::<String>(), any::<u64>()), 0..20),
        set in proptest::collection::vec(any::<Vec<u8>>(), 0..20)
    ) {
        let h: HashMap<String, u64> = entries.into_iter().collect();
        let s: HashSet<Vec<u8>> = set.into_iter().collect();
        type T = (HashMap<String, u64>, HashSet<Vec<u8>>);

        let enc = encode_to_vec(&(&h, &s), core_cfg()).unwrap();
        let ((dh, ds), n): (T, usize) = decode_from_slice(&enc, core_cfg()).unwrap();
        prop_assert_eq!(n, enc.len());
        prop_assert_eq!(&dh, &h);
        prop_assert_eq!(&ds, &s);

        let enc = encode_to_vec(&(&h, &s), lf_cfg()).unwrap();
        let ((dh, ds), n): (T, usize) = decode_from_slice(&enc, lf_cfg()).unwrap();
        prop_assert_eq!(n, enc.len());
        prop_assert_eq!(&dh, &h);
        prop_assert_eq!(&ds, &s);
    }

    #[test]
    fn prop_length_first_sorted(set in proptest::collection::vec(any::<Key>(), 0..30)) {
        let s: HashSet<Key> = set.into_iter().collect();
        let enc = encode_to_vec(&s, lf_cfg()).unwrap();
        // Same wire format as a Vec: decode in encounter order.
        let (items, _): (Vec<Key>, usize) = decode_from_slice(&enc, standard()).unwrap();
        prop_assert_eq!(items.len(), s.len());
        let encs: Vec<Vec<u8>> =
            items.iter().map(|i| encode_to_vec(i, standard()).unwrap()).collect();
        for w in encs.windows(2) {
            prop_assert!((w[0].len(), &w[0]) < (w[1].len(), &w[1]));
        }
    }

    #[test]
    fn prop_core_sorted_bytewise(set in proptest::collection::vec(any::<Key>(), 0..30)) {
        let s: BTreeSet<Key> = set.into_iter().collect();
        let enc = encode_to_vec(&s, core_cfg()).unwrap();
        let (items, _): (Vec<Key>, usize) = decode_from_slice(&enc, standard()).unwrap();
        prop_assert_eq!(items.len(), s.len());
        let encs: Vec<Vec<u8>> =
            items.iter().map(|i| encode_to_vec(i, standard()).unwrap()).collect();
        for w in encs.windows(2) {
            prop_assert!(w[0] < w[1]);
        }
    }

    /// Arbitrary bytes must never panic the deterministic decoders.
    #[test]
    fn prop_decode_garbage_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..128)) {
        let _ = decode_from_slice::<HashMap<u32, u32>, _>(&bytes, core_cfg().with_limit::<4096>());
        let _ = decode_from_slice::<HashSet<String>, _>(&bytes, lf_cfg().with_limit::<4096>());
        let _ = decode_from_slice::<BTreeMap<u8, Vec<u8>>, _>(&bytes, lf_cfg().with_limit::<4096>());
    }

    /// Decode -> re-encode of deterministic output is byte-stable.
    #[test]
    fn prop_reencode_is_stable(entries in proptest::collection::vec((any::<Key>(), any::<u8>()), 0..30)) {
        let h: HashMap<Key, u8> = entries.into_iter().collect();
        let e1 = encode_to_vec(&h, lf_cfg()).unwrap();
        let (d, _): (HashMap<Key, u8>, usize) = decode_from_slice(&e1, lf_cfg()).unwrap();
        let e2 = encode_to_vec(&d, lf_cfg()).unwrap();
        prop_assert_eq!(e1, e2);
    }
}
