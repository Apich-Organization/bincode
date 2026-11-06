//! CBOR (RFC 8949) decoding implementation.
#![allow(clippy::extra_unused_type_parameters)]
#![allow(clippy::redundant_pub_crate)]
#![allow(unsafe_code)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::cast_possible_wrap)]

use crate::de::read::Reader;
use crate::error::DecodeError;
use crate::error::cold_decode_error_invalid_boolean_value;
use crate::error::cold_decode_error_invalid_cbor_info;
use crate::error::cold_decode_error_outside_isize_range;
use crate::error::cold_decode_error_outside_usize_range;
use crate::error::cold_decode_error_unexpected_end;

/// Decodes a CBOR header (major type + value/argument + indefinite flag).
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_header<R: Reader>(reader: &mut R) -> Result<(u8, u64, bool), DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        let info = first & 0x1F;
        if crate::utils::is_likely!(info < 24) {
            reader.consume(1);
            return Ok((first >> 5, u64::from(info), false));
        }
    }
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        let major = first >> 5;
        let info = first & 0x1F;
        if crate::utils::is_likely!(info < 28) {
            if major == 7 {
                // Special case for Major Type 7: simple values and floats.
                // Don't consume bytes for Info 24-27 here, let the caller handle it.
                reader.consume(1);
                return Ok((major, u64::from(info), false));
            }
            let ptr = unsafe { bytes.as_ptr().add(1) };
            let val = match info {
                | 24 => {
                    let v = unsafe { *ptr };
                    reader.consume(2);
                    u64::from(v)
                },
                | 25 => {
                    let v = unsafe { ptr.cast::<u16>().read_unaligned() };
                    reader.consume(3);
                    u64::from(u16::from_be(v))
                },
                | 26 => {
                    let v = unsafe { ptr.cast::<u32>().read_unaligned() };
                    reader.consume(5);
                    u64::from(u32::from_be(v))
                },
                | 27 => {
                    let v = unsafe { ptr.cast::<u64>().read_unaligned() };
                    reader.consume(9);
                    u64::from_be(v)
                },
                | _ => unsafe { core::hint::unreachable_unchecked() },
            };
            return Ok((major, val, false));
        }
        if info == 31 {
            if major == 0 || major == 1 || major == 6 {
                return cold_decode_error_invalid_cbor_info(info);
            }
            reader.consume(1);
            return Ok((major, 0, true));
        }
        return cold_decode_error_invalid_cbor_info(info);
    }
    decode_header_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_header_cold<R: Reader>(reader: &mut R) -> Result<(u8, u64, bool), DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;

    if crate::utils::is_likely!(info < 24) {
        Ok((major, u64::from(info), false))
    } else if crate::utils::is_likely!(info < 28) {
        if major == 7 {
            return Ok((major, u64::from(info), false));
        }
        let val = match info {
            | 24 => u64::from(reader.read_u8()?),
            | 25 => u64::from(u16::from_be(reader.read_u16()?)),
            | 26 => u64::from(u32::from_be(reader.read_u32()?)),
            | 27 => u64::from_be(reader.read_u64()?),
            | _ => unsafe { core::hint::unreachable_unchecked() },
        };
        Ok((major, val, false))
    } else if info == 31 {
        if major == 0 || major == 1 || major == 6 {
            return cold_decode_error_invalid_cbor_info(info);
        }
        Ok((major, 0, true))
    } else {
        cold_decode_error_invalid_cbor_info(info)
    }
}

/// Decode a `u8` value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_u8<R: Reader>(reader: &mut R) -> Result<u8, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first < 24) {
            reader.consume(1);
            return Ok(first);
        }
    }
    if let Some(bytes) = reader.peek_read(2) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first == 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(val);
        }
    }
    decode_u8_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_u8_cold<R: Reader>(reader: &mut R) -> Result<u8, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if major != 0 || indefinite {
        return cold_decode_error_unexpected_end(0);
    }
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<u8>(val).unwrap_err())
}

/// Decode a `u16` value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_u16<R: Reader>(reader: &mut R) -> Result<u16, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first < 24) {
            reader.consume(1);
            return Ok(u16::from(first));
        }
    }
    if let Some(bytes) = reader.peek_read(3) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first == 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(u16::from(val));
        }
        if crate::utils::is_likely!(first == 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(u16::from_be(val));
        }
    }
    decode_u16_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_u16_cold<R: Reader>(reader: &mut R) -> Result<u16, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if major != 0 || indefinite {
        return cold_decode_error_unexpected_end(0);
    }
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<u16>(val).unwrap_err())
}

/// Decode a `u32` value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_u32<R: Reader>(reader: &mut R) -> Result<u32, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first < 24) {
            reader.consume(1);
            return Ok(u32::from(first));
        }
    }
    if let Some(bytes) = reader.peek_read(5) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_unlikely!(first == 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(u32::from(val));
        }
        if crate::utils::is_unlikely!(first == 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(u32::from(u16::from_be(val)));
        }
        if crate::utils::is_likely!(first == 26) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(u32::from_be(val));
        }
    }
    decode_u32_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_u32_cold<R: Reader>(reader: &mut R) -> Result<u32, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if major != 0 || indefinite {
        return cold_decode_error_unexpected_end(0);
    }
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<u32>(val).unwrap_err())
}

/// Decode a `u64` value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_u64<R: Reader>(reader: &mut R) -> Result<u64, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first < 24) {
            reader.consume(1);
            return Ok(u64::from(first));
        }
    }
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_unlikely!(first == 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(u64::from(val));
        }
        if crate::utils::is_unlikely!(first == 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(u64::from(u16::from_be(val)));
        }
        if crate::utils::is_unlikely!(first == 26) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(u64::from(u32::from_be(val)));
        }
        if crate::utils::is_likely!(first == 27) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            reader.consume(9);
            return Ok(u64::from_be(val));
        }
    }
    decode_u64_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_u64_cold<R: Reader>(reader: &mut R) -> Result<u64, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if major != 0 || indefinite {
        return cold_decode_error_unexpected_end(0);
    }
    Ok(val)
}

/// Decode a `u128` value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_u128<R: Reader>(reader: &mut R) -> Result<u128, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first < 24) {
            reader.consume(1);
            return Ok(u128::from(first));
        }
    }
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_unlikely!(first == 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(u128::from(val));
        }
        if crate::utils::is_unlikely!(first == 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(u128::from(u16::from_be(val)));
        }
        if crate::utils::is_unlikely!(first == 26) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(u128::from(u32::from_be(val)));
        }
        if crate::utils::is_likely!(first == 27) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            reader.consume(9);
            return Ok(u128::from(u64::from_be(val)));
        }
    }
    decode_u128_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_u128_cold<R: Reader>(reader: &mut R) -> Result<u128, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    match major {
        | 0 if !indefinite => Ok(u128::from(val)),
        | 6 if !indefinite => {
            if val != 2 {
                return cold_decode_error_unexpected_end(0);
            }
            decode_bignum_bytes(reader)
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode an i8 value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_i8<R: Reader>(reader: &mut R) -> Result<i8, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first < 24) {
            reader.consume(1);
            return Ok(first as i8);
        }
        if crate::utils::is_likely!((0x20..0x20 + 24).contains(&first)) {
            reader.consume(1);
            return Ok((-1 - i64::from(first - 0x20)) as i8);
        }
    }
    decode_i8_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_i8_cold<R: Reader>(reader: &mut R) -> Result<i8, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if indefinite {
        return cold_decode_error_unexpected_end(0);
    }
    match major {
        | 0 => val
            .try_into()
            .map_err(|_| cold_decode_error_outside_isize_range::<i8>(val as i64).unwrap_err()),
        | 1 => {
            let res = -1 - (val as i64);
            res.try_into()
                .map_err(|_| cold_decode_error_outside_isize_range::<i8>(res).unwrap_err())
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode an i16 value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_i16<R: Reader>(reader: &mut R) -> Result<i16, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first < 24) {
            reader.consume(1);
            return Ok(i16::from(first));
        }
        if crate::utils::is_likely!((0x20..0x20 + 24).contains(&first)) {
            reader.consume(1);
            return Ok(-1 - i16::from(first - 0x20));
        }
    }
    if let Some(bytes) = reader.peek_read(3) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first == 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(i16::from(val));
        }
        if crate::utils::is_likely!(first == 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            let val = u16::from_be(val);
            if i16::try_from(val).is_ok() {
                reader.consume(3);
                return Ok(val as i16);
            }
        }
        if crate::utils::is_likely!(first == 0x38) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(-1 - i16::from(val));
        }
        if crate::utils::is_likely!(first == 0x39) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            let val = u16::from_be(val);
            if i16::try_from(val).is_ok() {
                reader.consume(3);
                return Ok((-1 - i32::from(val)) as i16);
            }
        }
    }
    decode_i16_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_i16_cold<R: Reader>(reader: &mut R) -> Result<i16, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if indefinite {
        return cold_decode_error_unexpected_end(0);
    }
    match major {
        | 0 => val
            .try_into()
            .map_err(|_| cold_decode_error_outside_isize_range::<i16>(val as i64).unwrap_err()),
        | 1 => {
            let res = -1 - (val as i64);
            res.try_into()
                .map_err(|_| cold_decode_error_outside_isize_range::<i16>(res).unwrap_err())
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode an i32 value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_i32<R: Reader>(reader: &mut R) -> Result<i32, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first < 24) {
            reader.consume(1);
            return Ok(i32::from(first));
        }
        if crate::utils::is_likely!((0x20..0x20 + 24).contains(&first)) {
            reader.consume(1);
            return Ok(-1 - i32::from(first - 0x20));
        }
    }
    if let Some(bytes) = reader.peek_read(5) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_unlikely!(first == 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(i32::from(val));
        }
        if crate::utils::is_unlikely!(first == 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(i32::from(u16::from_be(val)));
        }
        if crate::utils::is_likely!(first == 26) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            let val = u32::from_be(val);
            if i32::try_from(val).is_ok() {
                reader.consume(5);
                return Ok(val as i32);
            }
        }
        if crate::utils::is_unlikely!(first == 0x38) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(-1 - i32::from(val));
        }
        if crate::utils::is_unlikely!(first == 0x39) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            let val = u16::from_be(val);
            reader.consume(3);
            return Ok((-1 - i64::from(val)) as i32);
        }
        if crate::utils::is_likely!(first == 0x3A) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            let val = u32::from_be(val);
            if i32::try_from(val).is_ok() {
                reader.consume(5);
                return Ok((-1 - i64::from(val)) as i32);
            }
        }
    }
    decode_i32_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_i32_cold<R: Reader>(reader: &mut R) -> Result<i32, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if indefinite {
        return cold_decode_error_unexpected_end(0);
    }
    match major {
        | 0 => val
            .try_into()
            .map_err(|_| cold_decode_error_outside_isize_range::<i32>(val as i64).unwrap_err()),
        | 1 => {
            let res = -1 - (val as i64);
            res.try_into()
                .map_err(|_| cold_decode_error_outside_isize_range::<i32>(res).unwrap_err())
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode an i64 value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_i64<R: Reader>(reader: &mut R) -> Result<i64, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first < 24) {
            reader.consume(1);
            return Ok(i64::from(first));
        }
        if crate::utils::is_likely!((0x20..0x20 + 24).contains(&first)) {
            reader.consume(1);
            return Ok(-1 - i64::from(first - 0x20));
        }
    }
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_unlikely!(first == 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(i64::from(val));
        }
        if crate::utils::is_unlikely!(first == 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(i64::from(u16::from_be(val)));
        }
        if crate::utils::is_unlikely!(first == 26) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(i64::from(u32::from_be(val)));
        }
        if crate::utils::is_likely!(first == 27) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            let val = u64::from_be(val);
            if i64::try_from(val).is_ok() {
                reader.consume(9);
                return Ok(val as i64);
            }
        }
        if crate::utils::is_unlikely!(first == 0x38) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(-1 - i64::from(val));
        }
        if crate::utils::is_unlikely!(first == 0x39) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(-1 - i64::from(u16::from_be(val)));
        }
        if crate::utils::is_unlikely!(first == 0x3A) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(-1 - i64::from(u32::from_be(val)));
        }
        if crate::utils::is_likely!(first == 0x3B) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            let val = u64::from_be(val);
            if val <= (i64::MIN.unsigned_abs() - 1) {
                reader.consume(9);
                return Ok(-1 - (val as i64));
            }
        }
    }
    decode_i64_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_i64_cold<R: Reader>(reader: &mut R) -> Result<i64, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if indefinite {
        return cold_decode_error_unexpected_end(0);
    }
    match major {
        | 0 => val
            .try_into()
            .map_err(|_| cold_decode_error_outside_isize_range::<i64>(val as i64).unwrap_err()),
        | 1 => {
            if val > (i64::MIN.unsigned_abs() - 1) {
                return cold_decode_error_outside_isize_range(-1);
            }
            Ok(-1 - (val as i64))
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode an i128 value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_i128<R: Reader>(reader: &mut R) -> Result<i128, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first < 24) {
            reader.consume(1);
            return Ok(i128::from(first));
        }
        if crate::utils::is_likely!((0x20..0x20 + 24).contains(&first)) {
            reader.consume(1);
            return Ok(-1 - i128::from(first - 0x20));
        }
    }
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_unlikely!(first == 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(i128::from(val));
        }
        if crate::utils::is_unlikely!(first == 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(i128::from(u16::from_be(val)));
        }
        if crate::utils::is_unlikely!(first == 26) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(i128::from(u32::from_be(val)));
        }
        if crate::utils::is_likely!(first == 27) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            reader.consume(9);
            return Ok(i128::from(u64::from_be(val)));
        }
        if crate::utils::is_unlikely!(first == 0x38) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(-1 - i128::from(val));
        }
        if crate::utils::is_unlikely!(first == 0x39) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(-1 - i128::from(u16::from_be(val)));
        }
        if crate::utils::is_unlikely!(first == 0x3A) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(-1 - i128::from(u32::from_be(val)));
        }
        if crate::utils::is_likely!(first == 0x3B) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            reader.consume(9);
            return Ok(-1 - i128::from(u64::from_be(val)));
        }
    }
    decode_i128_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_i128_cold<R: Reader>(reader: &mut R) -> Result<i128, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if indefinite {
        return cold_decode_error_unexpected_end(0);
    }
    match major {
        | 0 => Ok(i128::from(val)),
        | 1 => Ok(-1 - i128::from(val)),
        | 6 => match val {
            | 2 => Ok(decode_bignum_bytes(reader)? as i128),
            | 3 => {
                let magnitude = decode_bignum_bytes(reader)?;
                if magnitude > (i128::MAX as u128) + 1 {
                    return cold_decode_error_outside_isize_range(-1);
                }
                if magnitude == (i128::MAX as u128) + 1 {
                    return Ok(i128::MIN);
                }
                Ok(-1 - (magnitude as i128))
            },
            | _ => cold_decode_error_unexpected_end(0),
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode a CBOR byte string into a u128 (big-endian bignum).
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
fn decode_bignum_bytes<R: Reader>(reader: &mut R) -> Result<u128, DecodeError> {
    let (major, len, indefinite) = decode_header(reader)?;
    if major != 2 || indefinite {
        return cold_decode_error_unexpected_end(0);
    }
    if len > 16 {
        return cold_decode_error_outside_usize_range::<u128>(len);
    }
    let len = len as usize;
    let mut buf = [0u8; 16];
    reader.read(&mut buf[16 - len..])?;
    Ok(u128::from_be_bytes(buf))
}

/// Decode a `bool` value from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_bool<R: Reader>(reader: &mut R) -> Result<bool, DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let b = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(b == 0xF4) {
            reader.consume(1);
            return Ok(false);
        }
        if crate::utils::is_likely!(b == 0xF5) {
            reader.consume(1);
            return Ok(true);
        }
    }
    decode_bool_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_bool_cold<R: Reader>(reader: &mut R) -> Result<bool, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if major != 7 || indefinite {
        return cold_decode_error_invalid_boolean_value(0);
    }
    match val {
        | 20 => Ok(false),
        | 21 => Ok(true),
        | _ => cold_decode_error_invalid_boolean_value(val as u8),
    }
}

/// Convert f16 bits to f32.
#[inline(always)]
fn f16_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits & 0x8000);
    let exp = u32::from((bits & 0x7C00) >> 10);
    let mat = u32::from(bits & 0x03FF);

    if exp == 0x1F {
        f32::from_bits((sign << 16) | (0xFF << 23) | (mat << 13))
    } else if exp == 0 {
        if mat == 0 {
            f32::from_bits(sign << 16)
        } else {
            // Subnormal f16 to normal f32
            let mut m = mat;
            let mut e = 0i32;
            while (m & 0x400) == 0 {
                m <<= 1;
                e -= 1;
            }
            e += 1;
            m &= !0x400;
            let f32_exp = (127 - 15 + e) as u32;
            f32::from_bits((sign << 16) | (f32_exp << 23) | (m << 13))
        }
    } else {
        f32::from_bits((sign << 16) | ((exp + 127 - 15) << 23) | (mat << 13))
    }
}

/// Decode an `f32` value from CBOR. Supports f16, f32, f64 per RFC preference.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_f32<R: Reader>(reader: &mut R) -> Result<f32, DecodeError> {
    if let Some(bytes) = reader.peek_read(5) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first == 0xFA) {
            let bits = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(f32::from_bits(u32::from_be(bits)));
        }
    }
    decode_f32_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_f32_cold<R: Reader>(reader: &mut R) -> Result<f32, DecodeError> {
    let (major, info, indefinite) = decode_header_cold(reader)?;
    if major != 7 || indefinite {
        return cold_decode_error_unexpected_end(4);
    }
    match info {
        | 25 => Ok(f16_to_f32(u16::from_be(reader.read_u16()?))),
        | 26 => Ok(f32::from_bits(u32::from_be(reader.read_u32()?))),
        | 27 => Ok(f64::from_bits(u64::from_be(reader.read_u64()?)) as f32),
        | _ => cold_decode_error_unexpected_end(4),
    }
}

/// Decode an `f64` value from CBOR. Supports f16, f32, f64.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_f64<R: Reader>(reader: &mut R) -> Result<f64, DecodeError> {
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!(first == 0xFB) {
            let bits = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            reader.consume(9);
            return Ok(f64::from_bits(u64::from_be(bits)));
        }
    }
    decode_f64_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_f64_cold<R: Reader>(reader: &mut R) -> Result<f64, DecodeError> {
    let (major, info, indefinite) = decode_header_cold(reader)?;
    if major != 7 || indefinite {
        return cold_decode_error_unexpected_end(8);
    }
    match info {
        | 25 => Ok(f64::from(f16_to_f32(u16::from_be(reader.read_u16()?)))),
        | 26 => Ok(f64::from(f32::from_bits(u32::from_be(reader.read_u32()?)))),
        | 27 => Ok(f64::from_bits(u64::from_be(reader.read_u64()?))),
        | _ => cold_decode_error_unexpected_end(8),
    }
}

/// Decode a slice length from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_slice_len<R: Reader>(reader: &mut R) -> Result<usize, DecodeError> {
    const TAG: u8 = 4 << 5;
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!((TAG..TAG + 24).contains(&first)) {
            reader.consume(1);
            return Ok((first - TAG) as usize);
        }
    }
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_unlikely!(first == TAG + 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(val as usize);
        }
        if crate::utils::is_unlikely!(first == TAG + 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(u16::from_be(val) as usize);
        }
        if crate::utils::is_unlikely!(first == TAG + 26) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(u32::from_be(val) as usize);
        }
        if crate::utils::is_likely!(first == TAG + 27) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            let val = u64::from_be(val);
            let res = val
                .try_into()
                .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())?;
            reader.consume(9);
            return Ok(res);
        }
        if crate::utils::is_unlikely!(first == TAG + 31) {
            reader.consume(1);
            return Ok(usize::MAX);
        }
    }
    decode_slice_len_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_slice_len_cold<R: Reader>(reader: &mut R) -> Result<usize, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if major != 4 {
        return cold_decode_error_unexpected_end(0);
    }
    if indefinite {
        return Ok(usize::MAX);
    }
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())
}

/// Decode a map length from CBOR.
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_map_len<R: Reader>(reader: &mut R) -> Result<usize, DecodeError> {
    const TAG: u8 = 5 << 5;
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!((TAG..TAG + 24).contains(&first)) {
            reader.consume(1);
            return Ok((first - TAG) as usize);
        }
    }
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_unlikely!(first == TAG + 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(val as usize);
        }
        if crate::utils::is_unlikely!(first == TAG + 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(u16::from_be(val) as usize);
        }
        if crate::utils::is_unlikely!(first == TAG + 26) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(u32::from_be(val) as usize);
        }
        if crate::utils::is_likely!(first == TAG + 27) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            let val = u64::from_be(val);
            let res = val
                .try_into()
                .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())?;
            reader.consume(9);
            return Ok(res);
        }
        if crate::utils::is_unlikely!(first == TAG + 31) {
            reader.consume(1);
            return Ok(usize::MAX);
        }
    }
    decode_map_len_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_map_len_cold<R: Reader>(reader: &mut R) -> Result<usize, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if major != 5 {
        return cold_decode_error_unexpected_end(0);
    }
    if indefinite {
        return Ok(usize::MAX);
    }
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())
}

/// Decode a byte slice length from CBOR (Major Type 2).
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_byte_slice_len<R: Reader>(reader: &mut R) -> Result<usize, DecodeError> {
    const TAG: u8 = 2 << 5;
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!((TAG..TAG + 24).contains(&first)) {
            reader.consume(1);
            return Ok((first - TAG) as usize);
        }
    }
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_unlikely!(first == TAG + 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(val as usize);
        }
        if crate::utils::is_unlikely!(first == TAG + 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(u16::from_be(val) as usize);
        }
        if crate::utils::is_unlikely!(first == TAG + 26) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(u32::from_be(val) as usize);
        }
        if crate::utils::is_likely!(first == TAG + 27) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            let val = u64::from_be(val);
            let res = val
                .try_into()
                .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())?;
            reader.consume(9);
            return Ok(res);
        }
        if crate::utils::is_unlikely!(first == TAG + 31) {
            reader.consume(1);
            return Ok(usize::MAX);
        }
    }
    decode_byte_slice_len_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_byte_slice_len_cold<R: Reader>(reader: &mut R) -> Result<usize, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if major != 2 {
        return cold_decode_error_unexpected_end(0);
    }
    if indefinite {
        return Ok(usize::MAX);
    }
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())
}

/// Decode a length that can be either a byte slice (Major Type 2) or an array (Major Type 4).
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_byte_slice_or_array_len<R: Reader>(
    reader: &mut R
) -> Result<(u8, usize), DecodeError> {
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        let major = first >> 5;
        let info = first & 0x1F;
        if (major == 2 || major == 4) && crate::utils::is_likely!(info < 24) {
            reader.consume(1);
            return Ok((major, info as usize));
        }
    }
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        let major = first >> 5;
        let info = first & 0x1F;
        if major == 2 || major == 4 {
            if crate::utils::is_unlikely!(info == 24) {
                let val = unsafe { *bytes.as_ptr().add(1) };
                reader.consume(2);
                return Ok((major, val as usize));
            }
            if crate::utils::is_unlikely!(info == 25) {
                let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
                reader.consume(3);
                return Ok((major, u16::from_be(val) as usize));
            }
            if crate::utils::is_unlikely!(info == 26) {
                let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
                reader.consume(5);
                return Ok((major, u32::from_be(val) as usize));
            }
            if crate::utils::is_likely!(info == 27) {
                let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
                let val = u64::from_be(val);
                let res = val.try_into().map_err(|_| {
                    cold_decode_error_outside_usize_range::<usize>(val).unwrap_err()
                })?;
                reader.consume(9);
                return Ok((major, res));
            }
            if crate::utils::is_unlikely!(info == 31) {
                reader.consume(1);
                return Ok((major, usize::MAX));
            }
        }
    }
    decode_byte_slice_or_array_len_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_byte_slice_or_array_len_cold<R: Reader>(
    reader: &mut R
) -> Result<(u8, usize), DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if major != 2 && major != 4 {
        return cold_decode_error_unexpected_end(0);
    }
    let len = if indefinite {
        usize::MAX
    } else {
        val.try_into()
            .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())?
    };
    Ok((major, len))
}

/// Decode a string length from CBOR (Major Type 3).
///
/// # Errors
///
/// Returns `DecodeError` if the encoding fails.
#[inline(always)]
pub(crate) fn decode_str_len<R: Reader>(reader: &mut R) -> Result<usize, DecodeError> {
    const TAG: u8 = 3 << 5;
    if let Some(bytes) = reader.peek_read(1) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_likely!((TAG..TAG + 24).contains(&first)) {
            reader.consume(1);
            return Ok((first - TAG) as usize);
        }
    }
    if let Some(bytes) = reader.peek_read(9) {
        let first = unsafe { *bytes.as_ptr() };
        if crate::utils::is_unlikely!(first == TAG + 24) {
            let val = unsafe { *bytes.as_ptr().add(1) };
            reader.consume(2);
            return Ok(val as usize);
        }
        if crate::utils::is_unlikely!(first == TAG + 25) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u16>().read_unaligned() };
            reader.consume(3);
            return Ok(u16::from_be(val) as usize);
        }
        if crate::utils::is_unlikely!(first == TAG + 26) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u32>().read_unaligned() };
            reader.consume(5);
            return Ok(u32::from_be(val) as usize);
        }
        if crate::utils::is_likely!(first == TAG + 27) {
            let val = unsafe { bytes.as_ptr().add(1).cast::<u64>().read_unaligned() };
            let val = u64::from_be(val);
            let res = val
                .try_into()
                .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())?;
            reader.consume(9);
            return Ok(res);
        }
        if crate::utils::is_unlikely!(first == TAG + 31) {
            reader.consume(1);
            return Ok(usize::MAX);
        }
    }
    decode_str_len_cold(reader)
}

#[inline(never)]
#[cold]
fn decode_str_len_cold<R: Reader>(reader: &mut R) -> Result<usize, DecodeError> {
    let (major, val, indefinite) = decode_header_cold(reader)?;
    if major != 3 {
        return cold_decode_error_unexpected_end(0);
    }
    if indefinite {
        return Ok(usize::MAX);
    }
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())
}
