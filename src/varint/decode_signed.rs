#![allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
use crate::config::Endianness;
use crate::de::read::Reader;
use crate::error::DecodeError;

#[inline(always)]
pub fn varint_decode_i16<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i16, DecodeError> {
    let n = super::varint_decode_u16(read, endian)
        .map_err(DecodeError::change_integer_type_to_signed)?;
    Ok(if n % 2 == 0 {
        (n / 2) as _
    } else {
        !(n / 2) as _
    })
}

#[inline(always)]
pub fn varint_decode_i32<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i32, DecodeError> {
    let n = super::varint_decode_u32(read, endian)
        .map_err(DecodeError::change_integer_type_to_signed)?;
    Ok(if n % 2 == 0 {
        (n / 2) as _
    } else {
        !(n / 2) as _
    })
}

#[inline(always)]
pub fn varint_decode_i64<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i64, DecodeError> {
    let n = super::varint_decode_u64(read, endian)
        .map_err(DecodeError::change_integer_type_to_signed)?;
    Ok(if n % 2 == 0 {
        (n / 2) as _
    } else {
        !(n / 2) as _
    })
}

#[inline(always)]
pub fn varint_decode_i128<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i128, DecodeError> {
    let n = super::varint_decode_u128(read, endian)
        .map_err(DecodeError::change_integer_type_to_signed)?;
    Ok(if n % 2 == 0 {
        (n / 2) as _
    } else {
        !(n / 2) as _
    })
}

#[inline(always)]
pub fn varint_decode_isize<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<isize, DecodeError> {
    let n = super::varint_decode_usize(read, endian)
        .map_err(DecodeError::change_integer_type_to_signed)?;
    Ok(if n % 2 == 0 {
        (n / 2) as _
    } else {
        !(n / 2) as _
    })
}
