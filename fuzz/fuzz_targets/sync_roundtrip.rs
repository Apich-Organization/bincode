#![no_main]
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use bincode_next::config::{
    self,
};
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

#[derive(Arbitrary, Debug)]
struct FuzzConfig {
    endian: bool,       // true = Big, false = Little
    int_encoding: bool, // true = Varint, false = Fixint
    limit: bool,        // true = 4KB limit, false = NoLimit
}

macro_rules! do_roundtrip {
    ($target_data:expr, $config:expr) => {{
        let config = $config;
        let target = $target_data;
        let encoded = bincode_next::encode_to_vec(target, config).expect("Failed to encode");
        let (decoded, len): (FuzzType, usize) =
            bincode_next::decode_from_slice(&encoded, config).expect("Failed to decode");

        assert_eq!(len, encoded.len(), "Decode length mismatch");
        assert_eq!(target, &decoded, "Roundtrip value mismatch");
    }};
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);

    // 1. Generate Config Choice
    let fuzz_conf = match FuzzConfig::arbitrary(&mut u) {
        | Ok(c) => c,
        | Err(_) => return,
    };

    // 2. Generate Data
    let target_data = match FuzzType::arbitrary(&mut u) {
        | Ok(d) => d,
        | Err(_) => return,
    };

    // 3. Dispatch based on generated choice
    // Bincode 3.0 uses static types for config, so we use a macro to handle each permutation.
    match (fuzz_conf.endian, fuzz_conf.int_encoding, fuzz_conf.limit) {
        | (true, true, true) => {
            do_roundtrip!(
                &target_data,
                config::standard()
                    .with_big_endian()
                    .with_variable_int_encoding()
                    .with_limit::<1048576>()
            )
        },
        | (true, true, false) => {
            do_roundtrip!(
                &target_data,
                config::standard()
                    .with_big_endian()
                    .with_variable_int_encoding()
                    .with_no_limit()
            )
        },
        | (true, false, true) => {
            do_roundtrip!(
                &target_data,
                config::standard()
                    .with_big_endian()
                    .with_fixed_int_encoding()
                    .with_limit::<1048576>()
            )
        },
        | (true, false, false) => {
            do_roundtrip!(
                &target_data,
                config::standard()
                    .with_big_endian()
                    .with_fixed_int_encoding()
                    .with_no_limit()
            )
        },
        | (false, true, true) => {
            do_roundtrip!(
                &target_data,
                config::standard()
                    .with_little_endian()
                    .with_variable_int_encoding()
                    .with_limit::<1048576>()
            )
        },
        | (false, true, false) => {
            do_roundtrip!(
                &target_data,
                config::standard()
                    .with_little_endian()
                    .with_variable_int_encoding()
                    .with_no_limit()
            )
        },
        | (false, false, true) => {
            do_roundtrip!(
                &target_data,
                config::standard()
                    .with_little_endian()
                    .with_fixed_int_encoding()
                    .with_limit::<1048576>()
            )
        },
        | (false, false, false) => {
            do_roundtrip!(
                &target_data,
                config::standard()
                    .with_little_endian()
                    .with_fixed_int_encoding()
                    .with_no_limit()
            )
        },
    }
});
