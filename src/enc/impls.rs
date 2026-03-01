#![allow(unsafe_code)]
use super::Encode;
use super::Encoder;
use super::write::Writer;
use crate::error::EncodeError;
use core::cell::Cell;
use core::cell::RefCell;
use core::cmp::Reverse;
use core::marker::PhantomData;
use core::num::NonZeroI8;
use core::num::NonZeroI16;
use core::num::NonZeroI32;
use core::num::NonZeroI64;
use core::num::NonZeroI128;
use core::num::NonZeroIsize;
use core::num::NonZeroU8;
use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;
use core::num::NonZeroU128;
use core::num::NonZeroUsize;
use core::num::Wrapping;
use core::ops::Bound;
use core::ops::Range;
use core::ops::RangeInclusive;
use core::time::Duration;

impl Encode for () {
    fn encode<E: Encoder>(
        &self,
        _: &mut E,
    ) -> Result<(), EncodeError> {
        Ok(())
    }
}

impl<T> Encode for PhantomData<T> {
    fn encode<E: Encoder>(
        &self,
        _: &mut E,
    ) -> Result<(), EncodeError> {
        Ok(())
    }
}

impl Encode for bool {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_bool(*self)
    }
}

impl Encode for u8 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_u8(*self)
    }
}

impl Encode for NonZeroU8 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for u16 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_u16(*self)
    }
}

impl Encode for NonZeroU16 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for u32 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_u32(*self)
    }
}

impl Encode for NonZeroU32 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for u64 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_u64(*self)
    }
}

impl Encode for NonZeroU64 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for u128 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_u128(*self)
    }
}

impl Encode for NonZeroU128 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for usize {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_usize(*self)
    }
}

impl Encode for NonZeroUsize {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for i8 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_i8(*self)
    }
}

impl Encode for NonZeroI8 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for i16 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_i16(*self)
    }
}

impl Encode for NonZeroI16 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for i32 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_i32(*self)
    }
}

impl Encode for NonZeroI32 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for i64 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_i64(*self)
    }
}

impl Encode for NonZeroI64 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for i128 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_i128(*self)
    }
}

impl Encode for NonZeroI128 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for isize {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_isize(*self)
    }
}

impl Encode for NonZeroIsize {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl Encode for f32 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_f32(*self)
    }
}

impl Encode for f64 {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_f64(*self)
    }
}

impl<T: Encode> Encode for Wrapping<T> {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.0.encode(encoder)
    }
}

impl<T: Encode> Encode for Reverse<T> {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.0.encode(encoder)
    }
}

impl Encode for char {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encode_utf8(encoder.writer(), *self)
    }
}

impl<T> Encode for [T]
where
    T: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        let is_u8 = unty::type_equal::<T, u8>() || unty::type_equal::<T, i8>();
        if is_u8 {
            // SAFETY: self is [T], and T is u8/i8, so it's safe to cast to [u8]
            let bytes =
                unsafe { core::slice::from_raw_parts(self.as_ptr().cast::<u8>(), self.len()) };
            return encoder.encode_byte_slice(bytes);
        }

        super::encode_slice_len(encoder, self.len())?;

        let is_fixed = matches!(
            <E::C as crate::config::InternalIntEncodingConfig>::INT_ENCODING,
            crate::config::IntEncoding::Fixed
        );
        let is_native_endian = match <E::C as crate::config::InternalEndianConfig>::ENDIAN {
            | crate::config::Endianness::Little => cfg!(target_endian = "little"),
            | crate::config::Endianness::Big => cfg!(target_endian = "big"),
        };
        let is_bincode = matches!(
            <E::C as crate::config::InternalFormatConfig>::FORMAT,
            crate::config::Format::Bincode
        );

        if is_bincode
            && (is_native_endian
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
            let bytes_to_copy = core::mem::size_of_val(self);
            // SAFETY: T is a primitive type (pod), so it's safe to copy its bytes.
            // We've checked that the encoding is Fixed and Endianness matches.
            unsafe {
                let slice_ptr = self.as_ptr().cast::<u8>();
                let slice = core::slice::from_raw_parts(slice_ptr, bytes_to_copy);
                encoder.writer().write(slice)?;
            }
            return Ok(());
        }

        for item in self {
            item.encode(encoder)?;
        }
        Ok(())
    }
}


const TAG_CONT: u8 = 0b1000_0000;
const TAG_TWO_B: u8 = 0b1100_0000;
const TAG_THREE_B: u8 = 0b1110_0000;
const TAG_FOUR_B: u8 = 0b1111_0000;
const MAX_ONE_B: u32 = 0x80;
const MAX_TWO_B: u32 = 0x800;
const MAX_THREE_B: u32 = 0x10000;

fn encode_utf8(
    writer: &mut impl Writer,
    c: char,
) -> Result<(), EncodeError> {
    let code = c as u32;

    if code < MAX_ONE_B {
        writer.write_u8(c as u8)
    } else if code < MAX_TWO_B {
        let mut buf = [0u8; 2];
        buf[0] = ((code >> 6) & 0x1F) as u8 | TAG_TWO_B;
        buf[1] = (code & 0x3F) as u8 | TAG_CONT;
        writer.write(&buf)
    } else if code < MAX_THREE_B {
        let mut buf = [0u8; 3];
        buf[0] = ((code >> 12) & 0x0F) as u8 | TAG_THREE_B;
        buf[1] = ((code >> 6) & 0x3F) as u8 | TAG_CONT;
        buf[2] = (code & 0x3F) as u8 | TAG_CONT;
        writer.write(&buf)
    } else {
        let mut buf = [0u8; 4];
        buf[0] = ((code >> 18) & 0x07) as u8 | TAG_FOUR_B;
        buf[1] = ((code >> 12) & 0x3F) as u8 | TAG_CONT;
        buf[2] = ((code >> 6) & 0x3F) as u8 | TAG_CONT;
        buf[3] = (code & 0x3F) as u8 | TAG_CONT;
        writer.write(&buf)
    }
}

impl Encode for str {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.encode_str(self)
    }
}

impl<T, const N: usize> Encode for [T; N]
where
    T: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        let is_u8 = unty::type_equal::<T, u8>() || unty::type_equal::<T, i8>();
        let is_bincode = matches!(
            <E::C as crate::config::InternalFormatConfig>::FORMAT,
            crate::config::Format::Bincode
        );

        if is_u8 && !is_bincode {
            // SAFETY: self is [T; N], T is u8/i8, so it's safe to cast to [u8]
            let bytes = unsafe { core::slice::from_raw_parts(self.as_ptr().cast::<u8>(), N) };
            return encoder.encode_byte_slice(bytes);
        }

        let is_fixed = matches!(
            <E::C as crate::config::InternalIntEncodingConfig>::INT_ENCODING,
            crate::config::IntEncoding::Fixed
        );
        let is_native_endian = match <E::C as crate::config::InternalEndianConfig>::ENDIAN {
            | crate::config::Endianness::Little => cfg!(target_endian = "little"),
            | crate::config::Endianness::Big => cfg!(target_endian = "big"),
        };

        if is_bincode
            && (is_u8
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
                                || unty::type_equal::<T, i128>())))))
        {
            let bytes_to_copy = N * core::mem::size_of::<T>();
            // SAFETY: T is a primitive type (pod), so it's safe to copy its bytes.
            // We've checked that the encoding is Fixed and Endianness matches,
            // or that it's a 1-byte type (u8/i8).
            unsafe {
                let slice_ptr = self.as_ptr().cast::<u8>();
                let slice = core::slice::from_raw_parts(slice_ptr, bytes_to_copy);
                encoder.writer().write(slice)?;
            }
        } else {
            // Arrays in CBOR are always length-prefixed arrays if not byte strings?
            // Actually, bincode-next seems to encode [T; N] as just T...N WITHOUT length prefix?
            // Let's check bincode format spec for [T; N].
            // Ah, bincode encodes [T; N] as N elements of T without a length prefix.
            // In CBOR, we should probably encode it as an array if it's not a byte string.
            if !is_bincode {
                encoder.encode_array_len(N)?;
            }
            for item in self {
                item.encode(encoder)?;
            }
        }
        Ok(())
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        super::encode_option_variant(encoder, self.as_ref())?;
        if let Some(val) = self {
            val.encode(encoder)?;
        }
        Ok(())
    }
}

impl<T, U> Encode for Result<T, U>
where
    T: Encode,
    U: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        match self {
            | Ok(val) => {
                encoder.encode_variant_index(0)?;
                val.encode(encoder)
            },
            | Err(err) => {
                encoder.encode_variant_index(1)?;
                err.encode(encoder)
            },
        }
    }
}

impl<T> Encode for Cell<T>
where
    T: Encode + Copy,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        T::encode(&self.get(), encoder)
    }
}

impl<T> Encode for RefCell<T>
where
    T: Encode + ?Sized,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        let borrow_guard = self.try_borrow().map_err(|e| {
            crate::error::cold_encode_error_ref_cell_already_borrowed::<()>(
                e,
                core::any::type_name::<Self>(),
            )
            .unwrap_err()
        })?;
        T::encode(&borrow_guard, encoder)
    }
}

impl Encode for Duration {
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.as_secs().encode(encoder)?;
        self.subsec_nanos().encode(encoder)?;
        Ok(())
    }
}

impl<T> Encode for Range<T>
where
    T: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.start.encode(encoder)?;
        self.end.encode(encoder)?;
        Ok(())
    }
}

impl<T> Encode for RangeInclusive<T>
where
    T: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        self.start().encode(encoder)?;
        self.end().encode(encoder)?;
        Ok(())
    }
}

impl<T> Encode for Bound<T>
where
    T: Encode,
{
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        match self {
            | Self::Unbounded => {
                0u32.encode(encoder)?;
            },
            | Self::Included(val) => {
                1u32.encode(encoder)?;
                val.encode(encoder)?;
            },
            | Self::Excluded(val) => {
                2u32.encode(encoder)?;
                val.encode(encoder)?;
            },
        }
        Ok(())
    }
}

impl<T> Encode for &T
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
