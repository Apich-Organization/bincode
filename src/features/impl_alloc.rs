#![allow(unsafe_code)]
use crate::BorrowDecode;
use crate::Config;
use crate::config::Endianness;
use crate::config::IntEncoding;
use crate::config::InternalEndianConfig;
use crate::config::InternalIntEncodingConfig;
use crate::config::internal::InternalFingerprintGuard;
use crate::de::BorrowDecoder;
use crate::de::Decode;
use crate::de::Decoder;
use crate::de::read::Reader;
use crate::enc::Encode;
use crate::enc::Encoder;
use crate::enc::write::SizeWriter;
use crate::enc::write::Writer;
use crate::enc::{
    self,
};
use crate::error::DecodeError;
use crate::error::EncodeError;
use crate::impl_borrow_decode;
use alloc::borrow::Cow;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::collections::BinaryHeap;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(target_has_atomic = "ptr")]
use alloc::sync::Arc;

/// A writer that writes into a `Vec<u8>`.
#[derive(Default)]
pub struct VecWriter {
    inner: Vec<u8>,
}

impl VecWriter {
    /// Create a new vec writer with the given capacity
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
        }
    }

    // May not be used in all feature combinations
    #[allow(dead_code)]
    pub(crate) fn collect(self) -> Vec<u8> {
        self.inner
    }
}

impl enc::write::Writer for VecWriter {
    #[inline]
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> Result<(), EncodeError> {
        self.inner.extend_from_slice(bytes);
        Ok(())
    }

    #[inline]
    fn write_u8(
        &mut self,
        value: u8,
    ) -> Result<(), EncodeError> {
        self.inner.push(value);
        Ok(())
    }
}

/// Encode the given value into a `Vec<u8>` with the given `Config`. See the [config] module for more information.
///
/// [config]: config/index.html
/// # Errors
///
/// Returns an `EncodeError` if the value cannot be encoded.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn encode_to_vec<E: enc::Encode, C: Config>(
    val: E,
    config: C,
) -> Result<Vec<u8>, EncodeError>
where
    C::Mode: crate::config::InternalFingerprintGuard<E, C>,
{
    let size = {
        let mut size_writer = enc::EncoderImpl::<_, C>::new(SizeWriter::default(), config);
        val.encode(&mut size_writer)?;
        size_writer.into_writer().bytes_written
    };
    // Include the 8-byte prefix size if fingerprinting is enabled, we could just do capacity + 8 to be safe
    // but the write below handles resizing anyway.
    let mut writer = VecWriter::with_capacity(size + 8);
    C::Mode::encode_check(&config, &mut writer)?;
    let mut encoder = enc::EncoderImpl::<_, C>::new(writer, config);
    val.encode(&mut encoder)?;
    Ok(encoder.into_writer().inner)
}

impl<Context, T> Decode<Context> for BinaryHeap<T>
where
    T: Decode<Context> + Ord,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Vec::<T>::decode(decoder)?.into())
    }
}
impl<'de, T, Context> BorrowDecode<'de, Context> for BinaryHeap<T>
where
    T: BorrowDecode<'de, Context> + Ord,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        Ok(Vec::<T>::borrow_decode(decoder)?.into())
    }
}

impl<T> Encode for BinaryHeap<T>
where
    T: Encode + Ord,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        // BLOCKEDTODO(https://github.com/rust-lang/rust/issues/83659): we can u8 optimize this with `.as_slice()`
        crate::enc::encode_slice_len(encoder, self.len())?;
        for val in self {
            val.encode(encoder)?;
        }
        Ok(())
    }
}

impl<Context, K, V> Decode<Context> for BTreeMap<K, V>
where
    K: Decode<Context> + Ord,
    V: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let len = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<(K, V)>(len)?;

        let mut map = Self::new();
        for _ in 0..len {
            // See the documentation on `unclaim_bytes_read` as to why we're doing this here
            decoder.unclaim_bytes_read(core::mem::size_of::<(K, V)>());

            let key = K::decode(decoder)?;
            let value = V::decode(decoder)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}
impl<'de, K, V, Context> BorrowDecode<'de, Context> for BTreeMap<K, V>
where
    K: BorrowDecode<'de, Context> + Ord,
    V: BorrowDecode<'de, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let len = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<(K, V)>(len)?;

        let mut map = Self::new();
        for _ in 0..len {
            // See the documentation on `unclaim_bytes_read` as to why we're doing this here
            decoder.unclaim_bytes_read(core::mem::size_of::<(K, V)>());

            let key = K::borrow_decode(decoder)?;
            let value = V::borrow_decode(decoder)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<K, V> Encode for BTreeMap<K, V>
where
    K: Encode + Ord,
    V: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        crate::enc::encode_slice_len(encoder, self.len())?;
        for (key, val) in self {
            key.encode(encoder)?;
            val.encode(encoder)?;
        }
        Ok(())
    }
}

impl<Context, T> Decode<Context> for BTreeSet<T>
where
    T: Decode<Context> + Ord,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let len = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<T>(len)?;

        let mut map = Self::new();
        for _ in 0..len {
            // See the documentation on `unclaim_bytes_read` as to why we're doing this here
            decoder.unclaim_bytes_read(core::mem::size_of::<T>());

            let key = T::decode(decoder)?;
            map.insert(key);
        }
        Ok(map)
    }
}
impl<'de, T, Context> BorrowDecode<'de, Context> for BTreeSet<T>
where
    T: BorrowDecode<'de, Context> + Ord,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let len = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<T>(len)?;

        let mut map = Self::new();
        for _ in 0..len {
            // See the documentation on `unclaim_bytes_read` as to why we're doing this here
            decoder.unclaim_bytes_read(core::mem::size_of::<T>());

            let key = T::borrow_decode(decoder)?;
            map.insert(key);
        }
        Ok(map)
    }
}

impl<T> Encode for BTreeSet<T>
where
    T: Encode + Ord,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        crate::enc::encode_slice_len(encoder, self.len())?;
        for item in self {
            item.encode(encoder)?;
        }
        Ok(())
    }
}

impl<Context, T> Decode<Context> for VecDeque<T>
where
    T: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Vec::<T>::decode(decoder)?.into())
    }
}
impl<'de, T, Context> BorrowDecode<'de, Context> for VecDeque<T>
where
    T: BorrowDecode<'de, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        Ok(Vec::<T>::borrow_decode(decoder)?.into())
    }
}

impl<T> Encode for VecDeque<T>
where
    T: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        crate::enc::encode_slice_len(encoder, self.len())?;
        if unty::type_equal::<T, u8>() {
            let slices: (&[T], &[T]) = self.as_slices();
            // Safety: T is u8 so turning this into `&[u8]` is okay
            let slices: (&[u8], &[u8]) = unsafe {
                (
                    core::slice::from_raw_parts(slices.0.as_ptr().cast(), slices.0.len()),
                    core::slice::from_raw_parts(slices.1.as_ptr().cast(), slices.1.len()),
                )
            };

            encoder.writer().write(slices.0)?;
            encoder.writer().write(slices.1)?;
        } else {
            for item in self {
                item.encode(encoder)?;
            }
        }
        Ok(())
    }
}

impl<Context, T> Decode<Context> for Vec<T>
where
    T: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let len = crate::de::decode_slice_len(decoder)?;

        decoder.claim_container_read::<T>(len)?;

        let is_u8 = unty::type_equal::<T, u8>() || unty::type_equal::<T, i8>();
        let is_fixed = matches!(D::C::INT_ENCODING, IntEncoding::Fixed);
        let is_native_endian = match D::C::ENDIAN {
            | Endianness::Little => cfg!(target_endian = "little"),
            | Endianness::Big => cfg!(target_endian = "big"),
        };

        if is_u8
            || (is_fixed
                && is_native_endian
                && (unty::type_equal::<T, u16>()
                    || unty::type_equal::<T, i16>()
                    || unty::type_equal::<T, u32>()
                    || unty::type_equal::<T, i32>()
                    || unty::type_equal::<T, u64>()
                    || unty::type_equal::<T, i64>()
                    || unty::type_equal::<T, u128>()
                    || unty::type_equal::<T, i128>()
                    || unty::type_equal::<T, f32>()
                    || unty::type_equal::<T, f64>()))
        {
            let mut vec = Vec::with_capacity(len);
            unsafe {
                let bytes_to_read = len * core::mem::size_of::<T>();
                let ptr = vec.as_mut_ptr() as *mut u8;
                let slice = core::slice::from_raw_parts_mut(ptr, bytes_to_read);
                decoder.reader().read(slice)?;
                vec.set_len(len);
            }
            Ok(vec)
        } else {
            let is_varint = matches!(D::C::INT_ENCODING, IntEncoding::Variable);
            let mut vec = Vec::with_capacity(len);
            let mut i = 0;

            if is_varint {
                let max_size = if unty::type_equal::<T, u16>() || unty::type_equal::<T, i16>() {
                    Some(3)
                } else if unty::type_equal::<T, u32>() || unty::type_equal::<T, i32>() {
                    Some(5)
                } else if unty::type_equal::<T, u128>() || unty::type_equal::<T, i128>() {
                    Some(17)
                } else if unty::type_equal::<T, u64>()
                    || unty::type_equal::<T, i64>()
                    || unty::type_equal::<T, usize>()
                    || unty::type_equal::<T, isize>()
                {
                    Some(9)
                } else {
                    None
                };

                if let Some(ms) = max_size {
                    // Try the "Ultimate Batch" peek first.
                    // This is extremely fast as it eliminates almost all overhead.
                    if let Some(bytes) = decoder.reader().peek_read(len * ms) {
                        let mut reader = crate::de::read::SliceReader::new(bytes);
                        let endian = D::C::ENDIAN;

                        macro_rules! decode_mega {
                            ($decode_fn:ident, $type:ty) => {{
                                let v = unsafe {
                                    core::mem::transmute::<&mut Vec<T>, &mut Vec<$type>>(&mut vec)
                                };
                                let ptr = v.as_mut_ptr();
                                for j in 0..len {
                                    // Architecture-specific prefetching for large batches
                                    #[cfg(target_arch = "x86_64")]
                                    unsafe {
                                        if j % 8 == 0 {
                                            core::arch::x86_64::_mm_prefetch(
                                                reader.slice.as_ptr().add(64) as *const i8,
                                                core::arch::x86_64::_MM_HINT_T0,
                                            );
                                        }
                                    }
                                    #[cfg(target_arch = "aarch64")]
                                    unsafe {
                                        if j % 8 == 0 {
                                            core::arch::aarch64::__prefetch(
                                                reader.slice.as_ptr().add(64) as *const i8,
                                            );
                                        }
                                    }

                                    unsafe {
                                        std::ptr::write(
                                            ptr.add(j),
                                            crate::varint::$decode_fn(&mut reader, endian)?,
                                        );
                                    }
                                }
                                unsafe {
                                    v.set_len(len);
                                }
                            }};
                        }

                        if unty::type_equal::<T, u64>() {
                            decode_mega!(varint_decode_u64, u64);
                        } else if unty::type_equal::<T, i64>() {
                            decode_mega!(varint_decode_i64, i64);
                        } else if unty::type_equal::<T, u32>() {
                            decode_mega!(varint_decode_u32, u32);
                        } else if unty::type_equal::<T, i32>() {
                            decode_mega!(varint_decode_i32, i32);
                        } else if unty::type_equal::<T, usize>() {
                            decode_mega!(varint_decode_usize, usize);
                        } else if unty::type_equal::<T, isize>() {
                            decode_mega!(varint_decode_isize, isize);
                        } else if unty::type_equal::<T, u16>() {
                            decode_mega!(varint_decode_u16, u16);
                        } else if unty::type_equal::<T, i16>() {
                            decode_mega!(varint_decode_i16, i16);
                        } else if unty::type_equal::<T, u128>() {
                            decode_mega!(varint_decode_u128, u128);
                        } else if unty::type_equal::<T, i128>() {
                            decode_mega!(varint_decode_i128, i128);
                        }

                        let consumed = bytes.len() - reader.slice.len();
                        decoder.reader().consume(consumed);
                        if <D::C as crate::config::InternalLimitConfig>::LIMIT.is_some() {
                            decoder.unclaim_bytes_read(len * core::mem::size_of::<T>() - consumed);
                        }
                        return Ok(vec);
                    }
                }
            }

            if is_native_endian
                && !is_fixed
                && (unty::type_equal::<T, u32>()
                    || unty::type_equal::<T, i32>()
                    || unty::type_equal::<T, u64>()
                    || unty::type_equal::<T, i64>()
                    || unty::type_equal::<T, usize>()
                    || unty::type_equal::<T, isize>())
            {
                if let Some(bytes) = decoder.reader().peek_read(len) {
                    let scan = crate::varint::simd::scan_single_byte_varints(bytes);
                    let to_decode = scan.single_byte_count.min(len);
                    if to_decode > 0 {
                        macro_rules! decode_simd {
                            ($type:ty, $cast:expr) => {{
                                let v = unsafe {
                                    core::mem::transmute::<&mut Vec<T>, &mut Vec<$type>>(&mut vec)
                                };
                                let ptr = v.as_mut_ptr();
                                for j in 0..to_decode {
                                    let b = bytes[j];
                                    unsafe {
                                        std::ptr::write(ptr.add(j), $cast(b));
                                    }
                                }
                                unsafe {
                                    v.set_len(to_decode);
                                }
                            }};
                        }
                        if unty::type_equal::<T, u32>() {
                            decode_simd!(u32, |b| b as u32);
                        } else if unty::type_equal::<T, i32>() {
                            decode_simd!(i32, |b: u8| ((b >> 1) as i32) ^ (-((b & 1) as i32)));
                        } else if unty::type_equal::<T, u64>() {
                            decode_simd!(u64, |b| b as u64);
                        } else if unty::type_equal::<T, i64>() {
                            decode_simd!(i64, |b: u8| ((b >> 1) as i64) ^ (-((b & 1) as i64)));
                        } else if unty::type_equal::<T, usize>() {
                            decode_simd!(usize, |b| b as usize);
                        } else if unty::type_equal::<T, isize>() {
                            decode_simd!(isize, |b: u8| {
                                ((b >> 1) as isize) ^ (-((b & 1) as isize))
                            });
                        }

                        decoder.reader().consume(to_decode);
                        if <D::C as crate::config::InternalLimitConfig>::LIMIT.is_some() {
                            decoder.unclaim_bytes_read(to_decode * core::mem::size_of::<T>());
                        }
                        i = to_decode;
                    }
                }
            }

            for _ in i..len {
                if <D::C as crate::config::InternalLimitConfig>::LIMIT.is_some() {
                    decoder.unclaim_bytes_read(core::mem::size_of::<T>());
                }
                vec.push(T::decode(decoder)?);
            }
            Ok(vec)
        }
    }
}


impl<'de, T, Context> BorrowDecode<'de, Context> for Vec<T>
where
    T: BorrowDecode<'de, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let len = crate::de::decode_slice_len(decoder)?;

        decoder.claim_container_read::<T>(len)?;

        let is_u8 = unty::type_equal::<T, u8>() || unty::type_equal::<T, i8>();
        let is_fixed = matches!(D::C::INT_ENCODING, IntEncoding::Fixed);
        let is_native_endian = match D::C::ENDIAN {
            | Endianness::Little => cfg!(target_endian = "little"),
            | Endianness::Big => cfg!(target_endian = "big"),
        };

        if is_u8
            || (is_fixed
                && is_native_endian
                && (unty::type_equal::<T, u16>()
                    || unty::type_equal::<T, i16>()
                    || unty::type_equal::<T, u32>()
                    || unty::type_equal::<T, i32>()
                    || unty::type_equal::<T, u64>()
                    || unty::type_equal::<T, i64>()
                    || unty::type_equal::<T, u128>()
                    || unty::type_equal::<T, i128>()
                    || unty::type_equal::<T, f32>()
                    || unty::type_equal::<T, f64>()))
        {
            let mut vec = Vec::with_capacity(len);
            unsafe {
                let bytes_to_read = len * core::mem::size_of::<T>();
                let ptr = vec.as_mut_ptr() as *mut u8;
                let slice = core::slice::from_raw_parts_mut(ptr, bytes_to_read);
                decoder.reader().read(slice)?;
                vec.set_len(len);
            }
            Ok(vec)
        } else {
            let is_varint = matches!(D::C::INT_ENCODING, IntEncoding::Variable);
            let mut vec = Vec::with_capacity(len);
            let mut i = 0;

            if is_varint {
                let max_size = if unty::type_equal::<T, u16>() || unty::type_equal::<T, i16>() {
                    Some(3)
                } else if unty::type_equal::<T, u32>() || unty::type_equal::<T, i32>() {
                    Some(5)
                } else if unty::type_equal::<T, u128>() || unty::type_equal::<T, i128>() {
                    Some(17)
                } else if unty::type_equal::<T, u64>()
                    || unty::type_equal::<T, i64>()
                    || unty::type_equal::<T, usize>()
                    || unty::type_equal::<T, isize>()
                {
                    Some(9)
                } else {
                    None
                };

                if let Some(ms) = max_size {
                    if let Some(bytes) = decoder.reader().peek_read(len * ms) {
                        let mut reader = crate::de::read::SliceReader::new(bytes);
                        let endian = D::C::ENDIAN;

                        macro_rules! borrow_mega {
                            ($decode_fn:ident, $type:ty) => {{
                                let v = unsafe {
                                    core::mem::transmute::<&mut Vec<T>, &mut Vec<$type>>(&mut vec)
                                };
                                let ptr = v.as_mut_ptr();
                                for j in 0..len {
                                    // Architecture-specific prefetching for large batches
                                    #[cfg(target_arch = "x86_64")]
                                    unsafe {
                                        if j % 8 == 0 {
                                            core::arch::x86_64::_mm_prefetch(
                                                reader.slice.as_ptr().add(64) as *const i8,
                                                core::arch::x86_64::_MM_HINT_T0,
                                            );
                                        }
                                    }
                                    #[cfg(target_arch = "aarch64")]
                                    unsafe {
                                        if j % 8 == 0 {
                                            core::arch::aarch64::__prefetch(
                                                reader.slice.as_ptr().add(64) as *const i8,
                                            );
                                        }
                                    }
                                    unsafe {
                                        std::ptr::write(
                                            ptr.add(j),
                                            crate::varint::$decode_fn(&mut reader, endian)?,
                                        );
                                    }
                                }
                                unsafe {
                                    v.set_len(len);
                                }
                            }};
                        }

                        if unty::type_equal::<T, u64>() {
                            borrow_mega!(varint_decode_u64, u64);
                        } else if unty::type_equal::<T, i64>() {
                            borrow_mega!(varint_decode_i64, i64);
                        } else if unty::type_equal::<T, u32>() {
                            borrow_mega!(varint_decode_u32, u32);
                        } else if unty::type_equal::<T, i32>() {
                            borrow_mega!(varint_decode_i32, i32);
                        } else if unty::type_equal::<T, usize>() {
                            borrow_mega!(varint_decode_usize, usize);
                        } else if unty::type_equal::<T, isize>() {
                            borrow_mega!(varint_decode_isize, isize);
                        } else if unty::type_equal::<T, u16>() {
                            borrow_mega!(varint_decode_u16, u16);
                        } else if unty::type_equal::<T, i16>() {
                            borrow_mega!(varint_decode_i16, i16);
                        } else if unty::type_equal::<T, u128>() {
                            borrow_mega!(varint_decode_u128, u128);
                        } else if unty::type_equal::<T, i128>() {
                            borrow_mega!(varint_decode_i128, i128);
                        }

                        let consumed = bytes.len() - reader.slice.len();
                        decoder.reader().consume(consumed);
                        if <D::C as crate::config::InternalLimitConfig>::LIMIT.is_some() {
                            decoder.unclaim_bytes_read(len * core::mem::size_of::<T>() - consumed);
                        }
                        return Ok(vec);
                    }
                }
            }

            if is_native_endian
                && !is_fixed
                && (unty::type_equal::<T, u32>()
                    || unty::type_equal::<T, i32>()
                    || unty::type_equal::<T, u64>()
                    || unty::type_equal::<T, i64>()
                    || unty::type_equal::<T, usize>()
                    || unty::type_equal::<T, isize>())
            {
                if let Some(bytes) = decoder.reader().peek_read(len) {
                    let scan = crate::varint::simd::scan_single_byte_varints(bytes);
                    let to_decode = scan.single_byte_count.min(len);
                    if to_decode > 0 {
                        macro_rules! borrow_simd {
                            ($type:ty, $cast:expr) => {{
                                let v = unsafe {
                                    core::mem::transmute::<&mut Vec<T>, &mut Vec<$type>>(&mut vec)
                                };
                                let ptr = v.as_mut_ptr();
                                for j in 0..to_decode {
                                    let b = bytes[j];
                                    unsafe {
                                        std::ptr::write(ptr.add(j), $cast(b));
                                    }
                                }
                                unsafe {
                                    v.set_len(to_decode);
                                }
                            }};
                        }
                        if unty::type_equal::<T, u32>() {
                            borrow_simd!(u32, |b| b as u32);
                        } else if unty::type_equal::<T, i32>() {
                            borrow_simd!(i32, |b: u8| ((b >> 1) as i32) ^ (-((b & 1) as i32)));
                        } else if unty::type_equal::<T, u64>() {
                            borrow_simd!(u64, |b| b as u64);
                        } else if unty::type_equal::<T, i64>() {
                            borrow_simd!(i64, |b: u8| ((b >> 1) as i64) ^ (-((b & 1) as i64)));
                        } else if unty::type_equal::<T, usize>() {
                            borrow_simd!(usize, |b| b as usize);
                        } else if unty::type_equal::<T, isize>() {
                            borrow_simd!(isize, |b: u8| {
                                ((b >> 1) as isize) ^ (-((b & 1) as isize))
                            });
                        }

                        decoder.reader().consume(to_decode);
                        if <D::C as crate::config::InternalLimitConfig>::LIMIT.is_some() {
                            decoder.unclaim_bytes_read(to_decode * core::mem::size_of::<T>());
                        }
                        i = to_decode;
                    }
                }
            }

            for _ in i..len {
                if <D::C as crate::config::InternalLimitConfig>::LIMIT.is_some() {
                    decoder.unclaim_bytes_read(core::mem::size_of::<T>());
                }
                vec.push(T::borrow_decode(decoder)?);
            }
            Ok(vec)
        }
    }
}


impl<T> Encode for Vec<T>
where
    T: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        crate::enc::encode_slice_len(encoder, self.len())?;

        let is_u8 = unty::type_equal::<T, u8>() || unty::type_equal::<T, i8>();
        let is_fixed = matches!(E::C::INT_ENCODING, IntEncoding::Fixed);
        let is_native_endian = match E::C::ENDIAN {
            | Endianness::Little => cfg!(target_endian = "little"),
            | Endianness::Big => cfg!(target_endian = "big"),
        };

        if is_u8
            || (is_native_endian
                && (unty::type_equal::<T, f32>()
                    || unty::type_equal::<T, f64>()
                    || (is_fixed
                        && (unty::type_equal::<T, u16>()
                            || unty::type_equal::<T, i16>()
                            || unty::type_equal::<T, u32>()
                            || unty::type_equal::<T, i32>()
                            || unty::type_equal::<T, u64>()
                            || unty::type_equal::<T, i64>()
                            || unty::type_equal::<T, u128>()
                            || unty::type_equal::<T, i128>()))))
        {
            let bytes_to_copy = self.len() * core::mem::size_of::<T>();
            // SAFETY: T is a primitive type (pod), so it's safe to copy its bytes.
            // We've checked that the encoding is Fixed and Endianness matches,
            // or that it's a 1-byte type (u8/i8).
            unsafe {
                let slice_ptr = self.as_ptr() as *const u8;
                let slice = core::slice::from_raw_parts(slice_ptr, bytes_to_copy);
                encoder.writer().write(slice)?;
            }
        } else {
            for item in self {
                item.encode(encoder)?;
            }
        }
        Ok(())
    }
}


impl<Context> Decode<Context> for String {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let bytes = Vec::<u8>::decode(decoder)?;
        Self::from_utf8(bytes)
            .map_err(|e| crate::error::cold_decode_error_utf8::<()>(e.utf8_error()).unwrap_err())
    }
}
impl_borrow_decode!(String);

impl<Context> Decode<Context> for Box<str> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        String::decode(decoder).map(String::into_boxed_str)
    }
}
impl_borrow_decode!(Box<str>);

impl Encode for String {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.as_bytes().encode(encoder)
    }
}

impl<Context, T> Decode<Context> for Box<T>
where
    T: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let t = T::decode(decoder)?;
        Ok(Self::new(t))
    }
}
impl<'de, T, Context> BorrowDecode<'de, Context> for Box<T>
where
    T: BorrowDecode<'de, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let t = T::borrow_decode(decoder)?;
        Ok(Self::new(t))
    }
}

impl<T> Encode for Box<T>
where
    T: Encode + ?Sized,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        T::encode(self, encoder)
    }
}

impl<Context, T> Decode<Context> for Box<[T]>
where
    T: Decode<Context> + 'static,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let vec = Vec::decode(decoder)?;
        Ok(vec.into_boxed_slice())
    }
}

impl<'de, T, Context> BorrowDecode<'de, Context> for Box<[T]>
where
    T: BorrowDecode<'de, Context> + 'de,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let vec = Vec::borrow_decode(decoder)?;
        Ok(vec.into_boxed_slice())
    }
}

impl<Context, T> Decode<Context> for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    <T as ToOwned>::Owned: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let t = <T as ToOwned>::Owned::decode(decoder)?;
        Ok(Cow::Owned(t))
    }
}
impl<'cow, T, Context> BorrowDecode<'cow, Context> for Cow<'cow, T>
where
    T: ToOwned + ?Sized,
    &'cow T: BorrowDecode<'cow, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'cow, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let t = <&T>::borrow_decode(decoder)?;
        Ok(Cow::Borrowed(t))
    }
}

impl<T> Encode for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    for<'a> &'a T: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.as_ref().encode(encoder)
    }
}

#[test]
fn test_cow_round_trip() {
    let start = Cow::Borrowed("Foo");
    let encoded = crate::encode_to_vec(&start, crate::config::standard()).unwrap();
    let (end, _) =
        crate::borrow_decode_from_slice::<Cow<'_, str>, _>(&encoded, crate::config::standard())
            .unwrap();
    assert_eq!(start, end);
    let (end, _) =
        crate::decode_from_slice::<Cow<'_, str>, _>(&encoded, crate::config::standard()).unwrap();
    assert_eq!(start, end);
}

impl<Context, T> Decode<Context> for Rc<T>
where
    T: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let t = T::decode(decoder)?;
        Ok(Self::new(t))
    }
}

impl<Context> Decode<Context> for Rc<str> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let string = String::decode(decoder)?;
        Ok(string.into())
    }
}

impl<'de, T, Context> BorrowDecode<'de, Context> for Rc<T>
where
    T: BorrowDecode<'de, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let t = T::borrow_decode(decoder)?;
        Ok(Self::new(t))
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for Rc<str> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let string = String::decode(decoder)?;
        Ok(string.into())
    }
}

impl<T> Encode for Rc<T>
where
    T: Encode + ?Sized,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        T::encode(self, encoder)
    }
}

impl<Context, T> Decode<Context> for Rc<[T]>
where
    T: Decode<Context> + 'static,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let vec = Vec::decode(decoder)?;
        Ok(vec.into())
    }
}

impl<'de, T, Context> BorrowDecode<'de, Context> for Rc<[T]>
where
    T: BorrowDecode<'de, Context> + 'de,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let vec = Vec::borrow_decode(decoder)?;
        Ok(vec.into())
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<Context, T> Decode<Context> for Arc<T>
where
    T: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let t = T::decode(decoder)?;
        Ok(Self::new(t))
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<Context> Decode<Context> for Arc<str> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let string = String::decode(decoder)?;
        Ok(string.into())
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<'de, T, Context> BorrowDecode<'de, Context> for Arc<T>
where
    T: BorrowDecode<'de, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let t = T::borrow_decode(decoder)?;
        Ok(Self::new(t))
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<'de, Context> BorrowDecode<'de, Context> for Arc<str> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let string = String::decode(decoder)?;
        Ok(string.into())
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<T> Encode for Arc<T>
where
    T: Encode + ?Sized,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        T::encode(self, encoder)
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<Context, T> Decode<Context> for Arc<[T]>
where
    T: Decode<Context> + 'static,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let vec = Vec::decode(decoder)?;
        Ok(vec.into())
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<'de, T, Context> BorrowDecode<'de, Context> for Arc<[T]>
where
    T: BorrowDecode<'de, Context> + 'de,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D
    ) -> Result<Self, DecodeError> {
        let vec = Vec::borrow_decode(decoder)?;
        Ok(vec.into())
    }
}
