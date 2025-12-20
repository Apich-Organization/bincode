use crate::{
    config::Config,
    de::{read::Reader, BorrowDecode, BorrowDecoder, Decode, Decoder, DecoderImpl},
    enc::{write::Writer, Encode, Encoder, EncoderImpl},
    error::{DecodeError, EncodeError},
    impl_borrow_decode,
};
use core::time::Duration;
use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, CString},
    hash::Hash,
    io::Read,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
    time::SystemTime,
};

/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub fn decode_from_std_read<D: Decode<()>, C: Config, R: std::io::Read>(
    src: &mut R,
    config: C,
) -> Result<D, DecodeError> {
    decode_from_std_read_with_context(src, config, ())
}

/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub fn decode_from_std_read_with_context<
    Context,
    D: Decode<Context>,
    C: Config,
    R: std::io::Read,
>(
    src: &mut R,
    config: C,
    context: Context,
) -> Result<D, DecodeError> {
    let reader = IoReader::new(src);
    let mut decoder = DecoderImpl::<_, C, Context>::new(reader, config, context);
    D::decode(&mut decoder)
}

/// A reader that reads from a `std::io::Read`.
pub struct IoReader<R> {
    reader: R,
}

impl<R> IoReader<R> {
    /// Create a new `IoReader` from the given reader.
    pub const fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R> Reader for IoReader<R>
where
    R: std::io::Read,
{
    #[inline(always)]
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
        self.reader
            .read_exact(bytes)
            .map_err(|inner| DecodeError::Io {
                inner,
                additional: bytes.len(),
            })
    }
}

impl<R> Reader for std::io::BufReader<R>
where
    R: std::io::Read,
{
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
        self.read_exact(bytes).map_err(|inner| DecodeError::Io {
            inner,
            additional: bytes.len(),
        })
    }

    #[inline]
    fn peek_read(&mut self, n: usize) -> Option<&[u8]> {
        self.buffer().get(..n)
    }

    #[inline]
    fn consume(&mut self, n: usize) {
        <Self as std::io::BufRead>::consume(self, n);
    }
}

///
/// # Errors
///
/// Returns an `EncodeError` if the writer fails.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub fn encode_into_std_write<E: Encode, C: Config, W: std::io::Write>(
    val: E,
    dst: &mut W,
    config: C,
) -> Result<usize, EncodeError> {
    let writer = IoWriter::new(dst);
    let mut encoder = EncoderImpl::<_, C>::new(writer, config);
    val.encode(&mut encoder)?;
    Ok(encoder.into_writer().bytes_written())
}

/// A writer that writes to a `std::io::Write`.
pub struct IoWriter<'a, W: std::io::Write> {
    writer: &'a mut W,
    bytes_written: usize,
}

impl<'a, W: std::io::Write> IoWriter<'a, W> {
    /// Create a new `IoWriter` from the given writer.
    pub const fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            bytes_written: 0,
        }
    }

    /// Returns the number of bytes written to the underlying writer.
    #[must_use]
    pub const fn bytes_written(&self) -> usize {
        self.bytes_written
    }
}

impl<W: std::io::Write> Writer for IoWriter<'_, W> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.writer
            .write_all(bytes)
            .map_err(|inner| EncodeError::Io {
                inner,
                index: self.bytes_written,
            })?;
        self.bytes_written += bytes.len();
        Ok(())
    }
}

impl Encode for &CStr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.to_bytes().encode(encoder)
    }
}

impl Encode for CString {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_bytes().encode(encoder)
    }
}

impl<Context> Decode<Context> for CString {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let vec = std::vec::Vec::decode(decoder)?;
        Self::new(vec).map_err(|inner| DecodeError::CStringNulError {
            position: inner.nul_position(),
        })
    }
}
impl_borrow_decode!(CString);

impl<T> Encode for Mutex<T>
where
    T: Encode,
{
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let t = self.lock().map_err(|_| EncodeError::LockFailed {
            type_name: core::any::type_name::<Self>(),
        })?;
        t.encode(encoder)
    }
}

impl<Context, T> Decode<Context> for Mutex<T>
where
    T: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let t = T::decode(decoder)?;
        Ok(Self::new(t))
    }
}
impl<'de, T, Context> BorrowDecode<'de, Context> for Mutex<T>
where
    T: BorrowDecode<'de, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let t = T::borrow_decode(decoder)?;
        Ok(Self::new(t))
    }
}

impl<T> Encode for RwLock<T>
where
    T: Encode,
{
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let t = self.read().map_err(|_| EncodeError::LockFailed {
            type_name: core::any::type_name::<Self>(),
        })?;
        t.encode(encoder)
    }
}

impl<Context, T> Decode<Context> for RwLock<T>
where
    T: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let t = T::decode(decoder)?;
        Ok(Self::new(t))
    }
}
impl<'de, T, Context> BorrowDecode<'de, Context> for RwLock<T>
where
    T: BorrowDecode<'de, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let t = T::borrow_decode(decoder)?;
        Ok(Self::new(t))
    }
}

impl Encode for SystemTime {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let duration = self.duration_since(Self::UNIX_EPOCH).map_err(|e| {
            EncodeError::InvalidSystemTime {
                inner: e,
                time: std::boxed::Box::new(*self),
            }
        })?;
        duration.encode(encoder)
    }
}

impl<Context> Decode<Context> for SystemTime {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let duration = Duration::decode(decoder)?;
        Self::UNIX_EPOCH
            .checked_add(duration)
            .ok_or(DecodeError::InvalidSystemTime { duration })
    }
}
impl_borrow_decode!(SystemTime);

impl Encode for &'_ Path {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.to_str()
            .ok_or(EncodeError::InvalidPathCharacters)?
            .encode(encoder)
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for &'de Path {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let str = <&'de str>::borrow_decode(decoder)?;
        Ok(Path::new(str))
    }
}

impl Encode for PathBuf {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_path().encode(encoder)
    }
}

impl<Context> Decode<Context> for PathBuf {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let string = std::string::String::decode(decoder)?;
        Ok(string.into())
    }
}
impl_borrow_decode!(PathBuf);

impl Encode for IpAddr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Self::V4(v4) => {
                0u32.encode(encoder)?;
                v4.encode(encoder)
            }
            Self::V6(v6) => {
                1u32.encode(encoder)?;
                v6.encode(encoder)
            }
        }
    }
}

impl<Context> Decode<Context> for IpAddr {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match u32::decode(decoder)? {
            0 => Ok(Self::V4(Ipv4Addr::decode(decoder)?)),
            1 => Ok(Self::V6(Ipv6Addr::decode(decoder)?)),
            found => Err(DecodeError::UnexpectedVariant {
                allowed: &crate::error::AllowedEnumVariants::Range { min: 0, max: 1 },
                found,
                type_name: core::any::type_name::<Self>(),
            }),
        }
    }
}
impl_borrow_decode!(IpAddr);

impl Encode for Ipv4Addr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.writer().write(&self.octets())
    }
}

impl<Context> Decode<Context> for Ipv4Addr {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let mut buff = [0u8; 4];
        decoder.reader().read(&mut buff)?;
        Ok(Self::from(buff))
    }
}
impl_borrow_decode!(Ipv4Addr);

impl Encode for Ipv6Addr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.writer().write(&self.octets())
    }
}

impl<Context> Decode<Context> for Ipv6Addr {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let mut buff = [0u8; 16];
        decoder.reader().read(&mut buff)?;
        Ok(Self::from(buff))
    }
}
impl_borrow_decode!(Ipv6Addr);

impl Encode for SocketAddr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Self::V4(v4) => {
                0u32.encode(encoder)?;
                v4.encode(encoder)
            }
            Self::V6(v6) => {
                1u32.encode(encoder)?;
                v6.encode(encoder)
            }
        }
    }
}

impl<Context> Decode<Context> for SocketAddr {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match u32::decode(decoder)? {
            0 => Ok(Self::V4(SocketAddrV4::decode(decoder)?)),
            1 => Ok(Self::V6(SocketAddrV6::decode(decoder)?)),
            found => Err(DecodeError::UnexpectedVariant {
                allowed: &crate::error::AllowedEnumVariants::Range { min: 0, max: 1 },
                found,
                type_name: core::any::type_name::<Self>(),
            }),
        }
    }
}
impl_borrow_decode!(SocketAddr);

impl Encode for SocketAddrV4 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.ip().encode(encoder)?;
        self.port().encode(encoder)
    }
}

impl<Context> Decode<Context> for SocketAddrV4 {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let ip = Ipv4Addr::decode(decoder)?;
        let port = u16::decode(decoder)?;
        Ok(Self::new(ip, port))
    }
}
impl_borrow_decode!(SocketAddrV4);

impl Encode for SocketAddrV6 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.ip().encode(encoder)?;
        self.port().encode(encoder)
    }
}

impl<Context> Decode<Context> for SocketAddrV6 {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let ip = Ipv6Addr::decode(decoder)?;
        let port = u16::decode(decoder)?;
        Ok(Self::new(ip, port, 0, 0))
    }
}
impl_borrow_decode!(SocketAddrV6);

impl<K, V, S> Encode for HashMap<K, V, S>
where
    K: Encode,
    V: Encode,
{
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        crate::enc::encode_slice_len(encoder, self.len())?;
        for (k, v) in self {
            Encode::encode(k, encoder)?;
            Encode::encode(v, encoder)?;
        }
        Ok(())
    }
}

impl<Context, K, V, S> Decode<Context> for HashMap<K, V, S>
where
    K: Decode<Context> + Eq + std::hash::Hash,
    V: Decode<Context>,
    S: std::hash::BuildHasher + Default,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let len = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<(K, V)>(len)?;

        let hash_builder: S = Default::default();
        let mut map = Self::with_capacity_and_hasher(len, hash_builder);
        for _ in 0..len {
            // See the documentation on `unclaim_bytes_read` as to why we're doing this here
            decoder.unclaim_bytes_read(core::mem::size_of::<(K, V)>());

            let k = K::decode(decoder)?;
            let v = V::decode(decoder)?;
            map.insert(k, v);
        }
        Ok(map)
    }
}
impl<'de, K, V, S, Context> BorrowDecode<'de, Context> for HashMap<K, V, S>
where
    K: BorrowDecode<'de, Context> + Eq + std::hash::Hash,
    V: BorrowDecode<'de, Context>,
    S: std::hash::BuildHasher + Default,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let len = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<(K, V)>(len)?;

        let hash_builder: S = Default::default();
        let mut map = Self::with_capacity_and_hasher(len, hash_builder);
        for _ in 0..len {
            // See the documentation on `unclaim_bytes_read` as to why we're doing this here
            decoder.unclaim_bytes_read(core::mem::size_of::<(K, V)>());

            let k = K::borrow_decode(decoder)?;
            let v = V::borrow_decode(decoder)?;
            map.insert(k, v);
        }
        Ok(map)
    }
}

impl<Context, T, S> Decode<Context> for HashSet<T, S>
where
    T: Decode<Context> + Eq + Hash,
    S: std::hash::BuildHasher + Default,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let len = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<T>(len)?;

        let hash_builder: S = Default::default();
        let mut map: Self = Self::with_capacity_and_hasher(len, hash_builder);
        for _ in 0..len {
            // See the documentation on `unclaim_bytes_read` as to why we're doing this here
            decoder.unclaim_bytes_read(core::mem::size_of::<T>());

            let key = T::decode(decoder)?;
            map.insert(key);
        }
        Ok(map)
    }
}

impl<'de, T, S, Context> BorrowDecode<'de, Context> for HashSet<T, S>
where
    T: BorrowDecode<'de, Context> + Eq + Hash,
    S: std::hash::BuildHasher + Default,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let len = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<T>(len)?;

        let mut map = Self::with_capacity_and_hasher(len, S::default());
        for _ in 0..len {
            // See the documentation on `unclaim_bytes_read` as to why we're doing this here
            decoder.unclaim_bytes_read(core::mem::size_of::<T>());

            let key = T::borrow_decode(decoder)?;
            map.insert(key);
        }
        Ok(map)
    }
}

impl<T, S> Encode for HashSet<T, S>
where
    T: Encode,
{
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        crate::enc::encode_slice_len(encoder, self.len())?;
        for item in self {
            item.encode(encoder)?;
        }
        Ok(())
    }
}
