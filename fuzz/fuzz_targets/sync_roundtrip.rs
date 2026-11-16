#![no_main]
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::ffi::CString;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::SocketAddrV6;
use std::num::NonZeroI128;
use std::num::NonZeroI32;
use std::num::NonZeroU128;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

#[derive(Arbitrary, bincode_next::Decode, bincode_next::Encode, Debug, Clone)]
enum FuzzType {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),
    String(String),
    Vec(Vec<FuzzType>),
    Map(BTreeMap<u32, FuzzType>),
    Set(BTreeSet<u32>),
    Option(Option<Box<FuzzType>>),
    Tuple((u64, String, bool)),
    Char(char),
    BTreeMap(BTreeMap<u8, u8>),
    HashMap(HashMap<u8, u8>),
    HashSet(HashSet<u8>),
    BTreeSet(BTreeSet<u8>),
    VecDeque(VecDeque<FuzzType>),
    Box(Box<FuzzType>),
    BoxSlice(Box<[FuzzType]>),
    Rc(Rc<FuzzType>),
    Arc(Arc<FuzzType>),
    CString(CString),
    Duration(Duration),
    PathBuf(PathBuf),
    IpAddr(IpAddr),
    Ipv4Addr(Ipv4Addr),
    Ipv6Addr(Ipv6Addr),
    SocketAddr(SocketAddr),
    SocketAddrV4(SocketAddrV4),
    SocketAddrV6(SocketAddrV6),
    NonZeroU32(NonZeroU32),
    NonZeroI32(NonZeroI32),
    NonZeroU128(NonZeroU128),
    NonZeroI128(NonZeroI128),
    Nested(Vec<FuzzType>),
}

impl PartialEq for FuzzType {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        match (self, other) {
            | (FuzzType::F32(a), FuzzType::F32(b)) => a.to_bits() == b.to_bits(),
            | (FuzzType::F64(a), FuzzType::F64(b)) => a.to_bits() == b.to_bits(),
            | (FuzzType::Bool(a), FuzzType::Bool(b)) => a == b,
            | (FuzzType::U8(a), FuzzType::U8(b)) => a == b,
            | (FuzzType::U16(a), FuzzType::U16(b)) => a == b,
            | (FuzzType::U32(a), FuzzType::U32(b)) => a == b,
            | (FuzzType::U64(a), FuzzType::U64(b)) => a == b,
            | (FuzzType::U128(a), FuzzType::U128(b)) => a == b,
            | (FuzzType::I8(a), FuzzType::I8(b)) => a == b,
            | (FuzzType::I16(a), FuzzType::I16(b)) => a == b,
            | (FuzzType::I32(a), FuzzType::I32(b)) => a == b,
            | (FuzzType::I64(a), FuzzType::I64(b)) => a == b,
            | (FuzzType::I128(a), FuzzType::I128(b)) => a == b,
            | (FuzzType::String(a), FuzzType::String(b)) => a == b,
            | (FuzzType::Vec(a), FuzzType::Vec(b)) => a == b,
            | (FuzzType::Map(a), FuzzType::Map(b)) => a == b,
            | (FuzzType::Set(a), FuzzType::Set(b)) => a == b,
            | (FuzzType::Option(a), FuzzType::Option(b)) => a == b,
            | (FuzzType::Tuple(a), FuzzType::Tuple(b)) => a == b,
            | (FuzzType::Char(a), FuzzType::Char(b)) => a == b,
            | (FuzzType::BTreeMap(a), FuzzType::BTreeMap(b)) => a == b,
            | (FuzzType::HashMap(a), FuzzType::HashMap(b)) => a == b,
            | (FuzzType::HashSet(a), FuzzType::HashSet(b)) => a == b,
            | (FuzzType::BTreeSet(a), FuzzType::BTreeSet(b)) => a == b,
            | (FuzzType::VecDeque(a), FuzzType::VecDeque(b)) => a == b,
            | (FuzzType::Box(a), FuzzType::Box(b)) => a == b,
            | (FuzzType::BoxSlice(a), FuzzType::BoxSlice(b)) => a == b,
            | (FuzzType::Rc(a), FuzzType::Rc(b)) => a == b,
            | (FuzzType::Arc(a), FuzzType::Arc(b)) => a == b,
            | (FuzzType::CString(a), FuzzType::CString(b)) => a == b,
            | (FuzzType::Duration(a), FuzzType::Duration(b)) => a == b,
            | (FuzzType::PathBuf(a), FuzzType::PathBuf(b)) => a == b,
            | (FuzzType::IpAddr(a), FuzzType::IpAddr(b)) => a == b,
            | (FuzzType::Ipv4Addr(a), FuzzType::Ipv4Addr(b)) => a == b,
            | (FuzzType::Ipv6Addr(a), FuzzType::Ipv6Addr(b)) => a == b,
            | (FuzzType::SocketAddr(a), FuzzType::SocketAddr(b)) => a == b,
            | (FuzzType::SocketAddrV4(a), FuzzType::SocketAddrV4(b)) => a == b,
            | (FuzzType::SocketAddrV6(a), FuzzType::SocketAddrV6(b)) => a == b,
            | (FuzzType::NonZeroU32(a), FuzzType::NonZeroU32(b)) => a == b,
            | (FuzzType::NonZeroI32(a), FuzzType::NonZeroI32(b)) => a == b,
            | (FuzzType::NonZeroU128(a), FuzzType::NonZeroU128(b)) => a == b,
            | (FuzzType::NonZeroI128(a), FuzzType::NonZeroI128(b)) => a == b,
            | (FuzzType::Nested(a), FuzzType::Nested(b)) => a == b,
            | _ => false,
        }
    }
}

impl Eq for FuzzType {}

#[macro_use]
#[path = "../common/mod.rs"]
mod common;

/// CBOR may canonicalise NaN payloads and -0.0, so exact equality only holds after a
/// first normalising pass: decode(enc(x)) must be a fixed point of enc/dec.
fn roundtrip_normalising<C: bincode_next::config::Config>(
    target: &FuzzType,
    config: C,
) where
    C::Mode: bincode_next::config::InternalFingerprintGuard<FuzzType, C>,
    for<'a> C::Mode: bincode_next::config::InternalFingerprintGuard<&'a FuzzType, C>,
{
    let e1 = bincode_next::encode_to_vec(target, config).expect("encode");
    let (v1, n1): (FuzzType, usize) = bincode_next::decode_from_slice(&e1, config).expect("decode");
    assert_eq!(n1, e1.len());
    let e2 = bincode_next::encode_to_vec(&v1, config).expect("re-encode");
    let (v2, _): (FuzzType, usize) =
        bincode_next::decode_from_slice(&e2, config).expect("re-decode");
    assert_eq!(v1, v2, "CBOR round trip is not stable after normalisation");
}

fn roundtrip<C: bincode_next::config::Config>(
    target: &FuzzType,
    config: C,
) where
    C::Mode: bincode_next::config::InternalFingerprintGuard<FuzzType, C>,
    for<'a> C::Mode: bincode_next::config::InternalFingerprintGuard<&'a FuzzType, C>,
{
    let encoded = bincode_next::encode_to_vec(target, config).expect("Failed to encode");
    let (decoded, len): (FuzzType, usize) =
        bincode_next::decode_from_slice(&encoded, config).expect("Failed to decode");
    assert_eq!(len, encoded.len(), "Decode length mismatch");
    assert_eq!(target, &decoded, "Roundtrip value mismatch");
}

fn contains_v6(v: &FuzzType) -> bool {
    match v {
        | FuzzType::SocketAddrV6(_) | FuzzType::SocketAddr(std::net::SocketAddr::V6(_)) => true,
        | FuzzType::Vec(x) | FuzzType::Nested(x) => x.iter().any(contains_v6),
        | FuzzType::VecDeque(x) => x.iter().any(contains_v6),
        | FuzzType::BoxSlice(x) => x.iter().any(contains_v6),
        | FuzzType::Map(m) => m.values().any(contains_v6),
        | FuzzType::Option(Some(x)) | FuzzType::Box(x) => contains_v6(x),
        | FuzzType::Rc(x) => contains_v6(x),
        | FuzzType::Arc(x) => contains_v6(x),
        | _ => false,
    }
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(sel) = u8::arbitrary(&mut u) else {
        return;
    };
    let Ok(limit_sel) = u8::arbitrary(&mut u) else {
        return;
    };
    let Ok(target_data) = FuzzType::arbitrary(&mut u) else {
        return;
    };

    // Note: the `legacy` config (selector 6) intentionally omits `SocketAddrV6`
    // flowinfo/scope_id for bincode 1.x compatibility, so exact value equality is
    // only asserted for the other configs.
    // 8 wire configs (endian / int encoding / bit packing / legacy / deterministic),
    // each with and without a limit, plus the CBOR family.
    if sel & 7 == 6 && contains_v6(&target_data) {
        return;
    }
    with_bincode_config!(sel, |cfg| {
        if limit_sel & 1 == 0 {
            roundtrip(&target_data, cfg.with_limit::<16777216>());
        } else {
            roundtrip(&target_data, cfg.with_no_limit());
        }
    });
    use bincode_next::config::standard;
    match limit_sel >> 1 & 3 {
        | 0 => {
            roundtrip_normalising(
                &target_data,
                standard().with_cbor_format().with_limit::<16777216>(),
            )
        },
        | 1 => {
            roundtrip_normalising(
                &target_data,
                standard().with_deterministic_cbor().with_no_limit(),
            )
        },
        | _ => {},
    }
});
