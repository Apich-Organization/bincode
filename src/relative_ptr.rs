#![allow(unsafe_code)]

use core::marker::PhantomData;

/// Indicates that a type has a fixed size known at compile time.
/// This allows us to perform bounds checking efficiently.
pub trait StaticSize {
    /// The size of the type in bytes.
    const SIZE: usize;
}

macro_rules! impl_static_size {
    ($($t:ty),*) => {
        $(
            impl StaticSize for $t {
                const SIZE: usize = core::mem::size_of::<$t>();
            }
        )*
    };
}

impl_static_size!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, char, bool
);

impl<T: StaticSize, const N: usize> StaticSize for [T; N] {
    const SIZE: usize = T::SIZE * N;
}

/// A relative pointer that stores the offset from its own address to the target data.
/// This allows zero-copy deserialization without runtime allocations.
#[repr(transparent)]
pub struct RelativePtr<T, const ALIGN: usize> {
    offset: i32,
    _marker: PhantomData<T>,
}

impl<T, const ALIGN: usize> RelativePtr<T, ALIGN> {
    /// Resolves the pointer within the given buffer.
    /// Returns `Some(&T)` if the computed pointer is within the bounds of `buffer`
    /// and correctly aligned. Otherwise, returns `None`.
    pub fn get<'a>(&self, buffer: &'a [u8]) -> Option<&'a T>
    where
        T: StaticSize,
    {
        // Compile-time check: alignment must be a power of two
        const {
            assert!(
                ALIGN > 0 && (ALIGN & (ALIGN - 1)) == 0,
                "Alignment must be a power of two"
            )
        };

        let self_ptr = self as *const _ as usize;
        let buffer_start = buffer.as_ptr() as usize;
        let buffer_end = buffer_start + buffer.len();

        let self_end = self_ptr + core::mem::size_of::<Self>();

        if self_ptr < buffer_start || self_end > buffer_end {
            return None;
        }

        let target_addr = self_ptr.wrapping_add_signed(self.offset as isize);
        let target_end = target_addr.wrapping_add(T::SIZE);

        if target_addr < buffer_start || target_end > buffer_end {
            return None;
        }

        // Runtime alignment check
        if target_addr % ALIGN != 0 {
            return None;
        }

        // Derive target_ptr from buffer.as_ptr() to maintain provenance over the whole buffer
        let target_offset_in_buffer = target_addr - buffer_start;
        let target_ptr = unsafe { buffer.as_ptr().add(target_offset_in_buffer) };

        // Safe because we bounds checked against the buffer, alignment checked,
        // and we derive the lifetime from the buffer.
        Some(unsafe { &*(target_ptr as *const T) })
    }
}

/// A zero-copy array collection equivalent to `[T; N]`.
#[repr(transparent)]
pub struct ZeroArray<T, const N: usize, const ALIGN: usize> {
    ptr: RelativePtr<[T; N], ALIGN>,
}

impl<T: StaticSize, const N: usize, const ALIGN: usize> ZeroArray<T, N, ALIGN> {
    /// Resolves the array within the given buffer.
    pub fn get<'a>(&self, buffer: &'a [u8]) -> Option<&'a [T; N]> {
        self.ptr.get(buffer)
    }
}

/// A zero-copy string type with compile-time known max capacity.
#[repr(transparent)]
pub struct ZeroString<const CAP: usize> {
    ptr: RelativePtr<[u8; CAP], 1>, // Strings are byte-aligned (align 1)
}

impl<const CAP: usize> ZeroString<CAP> {
    /// Resolves the string within the given buffer.
    pub fn get<'a>(&self, buffer: &'a [u8]) -> Option<&'a str> {
        let bytes = self.ptr.get(buffer)?;
        core::str::from_utf8(bytes).ok()
    }
}

/// A trait for validating zero-copy types after basic decoding.
pub trait Validator {
    /// Validates internal state against the bounding buffer.
    fn is_valid(&self, buffer: &[u8]) -> bool;
}

impl<T: StaticSize, const ALIGN: usize> Validator for RelativePtr<T, ALIGN> {
    fn is_valid(&self, buffer: &[u8]) -> bool {
        self.get(buffer).is_some()
    }
}

impl<T: StaticSize, const N: usize, const ALIGN: usize> Validator for ZeroArray<T, N, ALIGN> {
    fn is_valid(&self, buffer: &[u8]) -> bool {
        self.ptr.is_valid(buffer)
    }
}

impl<const CAP: usize> Validator for ZeroString<CAP> {
    fn is_valid(&self, buffer: &[u8]) -> bool {
        self.get(buffer).is_some()
    }
}

/// A zero-copy slice collection conceptually equivalent to `&[T]` or `Vec<T>`.
#[repr(C)]
pub struct ZeroSlice<T, const ALIGN: usize> {
    len: u32,
    ptr: RelativePtr<T, ALIGN>,
}

impl<T: StaticSize, const ALIGN: usize> ZeroSlice<T, ALIGN> {
    /// Resolves the slice within the given buffer.
    pub fn get<'a>(&self, buffer: &'a [u8]) -> Option<&'a [T]> {
        if self.len == 0 {
            // For zero-length slices, we can just return an empty slice,
            // bypassing the pointer lookup (which might be invalid or dummy).
            return Some(&[]);
        }

        // Get the first element's reference to validate base bounds, alignment, and offset
        let first_ref = self.ptr.get(buffer)?;

        let slice_len = self.len as usize;

        // Calculate the total size required for the full slice.
        let total_size = T::SIZE.checked_mul(slice_len)?;

        let first_addr = first_ref as *const T as usize;
        let buffer_start = buffer.as_ptr() as usize;
        let target_offset = first_addr - buffer_start;

        let target_end = target_offset.checked_add(total_size)?;
        if target_end > buffer.len() {
            return None;
        }

        // Derive target_ptr from buffer.as_ptr() to maintain provenance over the whole slice
        let target_ptr = unsafe { buffer.as_ptr().add(target_offset) as *const T };

        Some(unsafe { core::slice::from_raw_parts(target_ptr, slice_len) })
    }
}

impl<T, const ALIGN: usize> StaticSize for ZeroSlice<T, ALIGN> {
    const SIZE: usize = core::mem::size_of::<Self>();
}

impl<T: StaticSize, const ALIGN: usize> Validator for ZeroSlice<T, ALIGN> {
    fn is_valid(&self, buffer: &[u8]) -> bool {
        self.get(buffer).is_some()
    }
}

/// A dynamically sized zero-copy string conceptually equivalent to `&str` or `String`.
#[repr(transparent)]
pub struct ZeroStr {
    slice: ZeroSlice<u8, 1>,
}

impl ZeroStr {
    /// Resolves the string within the given buffer.
    pub fn get<'a>(&self, buffer: &'a [u8]) -> Option<&'a str> {
        let bytes = self.slice.get(buffer)?;
        core::str::from_utf8(bytes).ok()
    }
}

impl StaticSize for ZeroStr {
    const SIZE: usize = core::mem::size_of::<Self>();
}

impl Validator for ZeroStr {
    fn is_valid(&self, buffer: &[u8]) -> bool {
        self.get(buffer).is_some()
    }
}

// Bincode integration
use crate::de::{BorrowDecode, BorrowDecoder, Decode, Decoder};
use crate::enc::{Encode, Encoder};
use crate::error::{DecodeError, EncodeError};

impl<T: StaticSize, const ALIGN: usize, Context> Decode<Context> for RelativePtr<T, ALIGN> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let offset = i32::decode(decoder)?;
        Ok(Self {
            offset,
            _marker: PhantomData,
        })
    }
}

impl<'de, T: StaticSize, const ALIGN: usize, Context> BorrowDecode<'de, Context>
    for RelativePtr<T, ALIGN>
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let offset = i32::borrow_decode(decoder)?;
        Ok(Self {
            offset,
            _marker: PhantomData,
        })
    }
}

impl<T: StaticSize, const ALIGN: usize> Encode for RelativePtr<T, ALIGN> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.offset.encode(encoder)
    }
}

impl<T: StaticSize, const N: usize, const ALIGN: usize, Context> Decode<Context>
    for ZeroArray<T, N, ALIGN>
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let ptr = RelativePtr::decode(decoder)?;
        Ok(Self { ptr })
    }
}

impl<'de, T: StaticSize, const N: usize, const ALIGN: usize, Context> BorrowDecode<'de, Context>
    for ZeroArray<T, N, ALIGN>
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let ptr = RelativePtr::borrow_decode(decoder)?;
        Ok(Self { ptr })
    }
}

impl<T: StaticSize, const N: usize, const ALIGN: usize> Encode for ZeroArray<T, N, ALIGN> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.ptr.encode(encoder)
    }
}

impl<const CAP: usize, Context> Decode<Context> for ZeroString<CAP> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let ptr = RelativePtr::decode(decoder)?;
        Ok(Self { ptr })
    }
}

impl<'de, const CAP: usize, Context> BorrowDecode<'de, Context> for ZeroString<CAP> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let ptr = RelativePtr::borrow_decode(decoder)?;
        Ok(Self { ptr })
    }
}

impl<const CAP: usize> Encode for ZeroString<CAP> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.ptr.encode(encoder)
    }
}

impl<T: StaticSize, const ALIGN: usize, Context> Decode<Context> for ZeroSlice<T, ALIGN> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let len = u32::decode(decoder)?;
        let ptr = RelativePtr::decode(decoder)?;
        Ok(Self { len, ptr })
    }
}

impl<'de, T: StaticSize, const ALIGN: usize, Context> BorrowDecode<'de, Context>
    for ZeroSlice<T, ALIGN>
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let len = u32::borrow_decode(decoder)?;
        let ptr = RelativePtr::borrow_decode(decoder)?;
        Ok(Self { len, ptr })
    }
}

impl<T: StaticSize, const ALIGN: usize> Encode for ZeroSlice<T, ALIGN> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.len.encode(encoder)?;
        self.ptr.encode(encoder)
    }
}

impl<Context> Decode<Context> for ZeroStr {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let slice = ZeroSlice::decode(decoder)?;
        Ok(Self { slice })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for ZeroStr {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let slice = ZeroSlice::borrow_decode(decoder)?;
        Ok(Self { slice })
    }
}

impl Encode for ZeroStr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.slice.encode(encoder)
    }
}
