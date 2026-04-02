#![no_main]
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use futures::executor::block_on;
use futures::io::AsyncRead;
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
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
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

/// A reader that yields according to a predefined pattern to stress-test async state.
struct TortureReader<'a> {
    data: &'a [u8],
    pos: usize,
    /// Yield pattern: each value is how many bytes to read before yielding.
    yield_pattern: &'a [u8],
    pattern_pos: usize,
}

impl<'a> AsyncRead for TortureReader<'a> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.pos >= self.data.len() {
            return Poll::Ready(Ok(0));
        }

        // Get how many bytes we are "allowed" to read from the fuzzer pattern.
        let pattern_val = self
            .yield_pattern
            .get(self.pattern_pos)
            .copied()
            .unwrap_or(255);

        if pattern_val == 0 {
            // Force a yield!
            self.pattern_pos += 1;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        let to_read = std::cmp::min(buf.len(), pattern_val as usize);
        let to_read = std::cmp::min(to_read, self.data.len() - self.pos);

        buf[..to_read].copy_from_slice(&self.data[self.pos..self.pos + to_read]);
        self.pos += to_read;
        self.pattern_pos += 1;

        Poll::Ready(Ok(to_read))
    }
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);

    // 1. Generate Target Data for Roundtrip
    let target = match FuzzType::arbitrary(&mut u) {
        | Ok(t) => t,
        | Err(_) => return,
    };

    // 2. Generate Yield Pattern for AsyncRead
    let pattern = match <Vec<u8>>::arbitrary(&mut u) {
        | Ok(p) => p,
        | Err(_) => return,
    };

    let config = bincode_next::config::standard();
    let encoded = bincode_next::encode_to_vec(&target, config).expect("Encode failed");

    // 3. Roundtrip Async via Fiber
    let reader = TortureReader {
        data: &encoded,
        pos: 0,
        yield_pattern: &pattern,
        pattern_pos: 0,
    };

    let decoded: FuzzType = match block_on(bincode_next::decode_async(config, reader)) {
        | Ok(d) => d,
        | Err(e) => {
            panic!(
                "Async decode failed: {:?}\nEncoded: {:?}\nPattern: {:?}",
                e, encoded, pattern
            )
        },
    };

    assert_eq!(target, decoded, "Async roundtrip mismatch");
});
