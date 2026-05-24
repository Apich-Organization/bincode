#![no_main]
use libfuzzer_sys::fuzz_target;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
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
use std::time::Duration;
use std::time::SystemTime;

#[derive(
    bincode_next::Decode,
    bincode_next::Encode,
    PartialEq,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Eq,
    PartialOrd,
    Ord,
)]
enum AllTypes {
    BTreeMap(BTreeMap<u8, AllTypes>),
    BTreeSet(BTreeSet<AllTypes>),
    VecDeque(VecDeque<AllTypes>),
    Vec(Vec<u8>),
    String(String),
    Box(Box<u8>),
    BoxSlice(Box<[u8]>),
    CString(CString),
    SystemTime(SystemTime),
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
    I128(i128),
    I8(i8),
    U128(u128),
    U8(u8),
    // Cow(Cow<'static, [u8]>), Blocked, see comment on decode
}

fuzz_target!(|data: &[u8]| {
    let config = bincode_next::config::legacy().with_limit::<1024>();
    #[allow(deprecated)]
    let mut configv1 = bincodev1::config();
    configv1.limit(1024);
    let bincode_v1: Result<AllTypes, _> = configv1.deserialize_from(data);
    let bincode_v2: Result<(AllTypes, _), _> = bincode_next::decode_from_slice(data, config);

    match (&bincode_v1, &bincode_v2) {
        | (Err(e), _) if e.to_string().contains("limit") => {},

        | (_, Err(bincode_next::error::DecodeError::LimitExceeded)) => {},
        | (Ok(bincode_v1), Ok((bincode_v2, _))) if bincode_v1 != bincode_v2 => {
            println!("Bytes:      {:?}", data);
            println!("Bincode V1: {:?}", bincode_v1);
            println!("Bincode V2: {:?}", bincode_v2);
            panic!("failed equality check");
        },
        | (Ok(_), Err(_)) | (Err(_), Ok(_)) => {
            println!("Bytes:      {:?}", data);
            println!("Bincode V1: {:?}", bincode_v1);
            println!("Bincode V2: {:?}", bincode_v2);
            panic!("failed equality check");
        },

        | _ => {},
    }
});
