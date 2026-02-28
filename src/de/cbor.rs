//! CBOR (RFC 8949) decoding implementation.

use crate::de::read::Reader;
use crate::error::DecodeError;
use crate::error::cold_decode_error_invalid_boolean_value;
use crate::error::cold_decode_error_outside_isize_range;
use crate::error::cold_decode_error_outside_usize_range;
use crate::error::cold_decode_error_unexpected_end;

/// Decodes a CBOR "additional info" value (0-27).
#[inline]
fn decode_additional_info<R: Reader>(
    reader: &mut R,
    info: u8,
) -> Result<u64, DecodeError> {
    match info {
        | 0..=23 => Ok(info as u64),
        | 24 => Ok(reader.read_u8()? as u64),
        | 25 => {
            let mut bytes = [0u8; 2];
            reader.read(&mut bytes)?;
            Ok(u16::from_be_bytes(bytes) as u64)
        },
        | 26 => {
            let mut bytes = [0u8; 4];
            reader.read(&mut bytes)?;
            Ok(u32::from_be_bytes(bytes) as u64)
        },
        | 27 => {
            let mut bytes = [0u8; 8];
            reader.read(&mut bytes)?;
            Ok(u64::from_be_bytes(bytes))
        },
        | _ => cold_decode_error_unexpected_end(1),
    }
}

/// Decode a `u8` value from CBOR.
#[inline]
pub fn decode_u8<R: Reader>(reader: &mut R) -> Result<u8, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    if major != 0 {
        return cold_decode_error_unexpected_end(0);
    }
    let val = decode_additional_info(reader, info)?;
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<u8>(val).unwrap_err())
}

/// Decode a `u16` value from CBOR.
#[inline]
pub fn decode_u16<R: Reader>(reader: &mut R) -> Result<u16, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    if major != 0 {
        return cold_decode_error_unexpected_end(0);
    }
    let val = decode_additional_info(reader, info)?;
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<u16>(val).unwrap_err())
}

/// Decode a `u32` value from CBOR.
#[inline]
pub fn decode_u32<R: Reader>(reader: &mut R) -> Result<u32, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    if major != 0 {
        return cold_decode_error_unexpected_end(0);
    }
    let val = decode_additional_info(reader, info)?;
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<u32>(val).unwrap_err())
}

/// Decode a `u64` value from CBOR.
#[inline]
pub fn decode_u64<R: Reader>(reader: &mut R) -> Result<u64, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    if major != 0 {
        return cold_decode_error_unexpected_end(0);
    }
    decode_additional_info(reader, info)
}

/// Decode a `u128` value from CBOR.
///
/// Handles both standard CBOR unsigned integers (major type 0, values ≤ u64::MAX)
/// and Tag 2 (positive bignum) for values > u64::MAX (RFC 8949 §3.4.3).
#[inline]
pub fn decode_u128<R: Reader>(reader: &mut R) -> Result<u128, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    match major {
        // Standard unsigned integer (major type 0)
        | 0 => {
            let val = decode_additional_info(reader, info)?;
            Ok(val as u128)
        },
        // Tag (major type 6) — expect Tag 2 (positive bignum)
        | 6 => {
            let tag = decode_additional_info(reader, info)?;
            if tag != 2 {
                return cold_decode_error_unexpected_end(0);
            }
            decode_bignum_bytes(reader)
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode an `i8` value from CBOR.
#[inline]
pub fn decode_i8<R: Reader>(reader: &mut R) -> Result<i8, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    match major {
        | 0 => {
            let val = decode_additional_info(reader, info)?;
            val.try_into()
                .map_err(|_| cold_decode_error_outside_isize_range::<i8>(val as i64).unwrap_err())
        },
        | 1 => {
            let val = decode_additional_info(reader, info)?;
            let res = -1 - (val as i64);
            res.try_into()
                .map_err(|_| cold_decode_error_outside_isize_range::<i8>(res).unwrap_err())
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode an `i16` value from CBOR.
#[inline]
pub fn decode_i16<R: Reader>(reader: &mut R) -> Result<i16, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    match major {
        | 0 => {
            let val = decode_additional_info(reader, info)?;
            val.try_into()
                .map_err(|_| cold_decode_error_outside_isize_range::<i16>(val as i64).unwrap_err())
        },
        | 1 => {
            let val = decode_additional_info(reader, info)?;
            let res = -1 - (val as i64);
            res.try_into()
                .map_err(|_| cold_decode_error_outside_isize_range::<i16>(res).unwrap_err())
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode an `i32` value from CBOR.
#[inline]
pub fn decode_i32<R: Reader>(reader: &mut R) -> Result<i32, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    match major {
        | 0 => {
            let val = decode_additional_info(reader, info)?;
            val.try_into()
                .map_err(|_| cold_decode_error_outside_isize_range::<i32>(val as i64).unwrap_err())
        },
        | 1 => {
            let val = decode_additional_info(reader, info)?;
            let res = -1 - (val as i64);
            res.try_into()
                .map_err(|_| cold_decode_error_outside_isize_range::<i32>(res).unwrap_err())
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode an `i64` value from CBOR.
#[inline]
pub fn decode_i64<R: Reader>(reader: &mut R) -> Result<i64, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    match major {
        | 0 => {
            let val = decode_additional_info(reader, info)?;
            val.try_into()
                .map_err(|_| cold_decode_error_outside_isize_range::<i64>(val as i64).unwrap_err())
        },
        | 1 => {
            let val = decode_additional_info(reader, info)?;
            if val > (i64::MIN.unsigned_abs() - 1) {
                return cold_decode_error_outside_isize_range(-1);
            }
            Ok(-1 - (val as i64))
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode an `i128` value from CBOR.
///
/// Handles standard CBOR integers (major types 0 and 1),
/// Tag 2 (positive bignum), and Tag 3 (negative bignum) per RFC 8949 §3.4.3.
#[inline]
pub fn decode_i128<R: Reader>(reader: &mut R) -> Result<i128, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    match major {
        // Standard unsigned integer (major type 0)
        | 0 => {
            let val = decode_additional_info(reader, info)?;
            Ok(val as i128)
        },
        // Standard negative integer (major type 1): value = -1 - additional
        | 1 => {
            let val = decode_additional_info(reader, info)?;
            Ok(-1 - (val as i128))
        },
        // Tag (major type 6)
        | 6 => {
            let tag = decode_additional_info(reader, info)?;
            match tag {
                // Tag 2 = positive bignum
                | 2 => {
                    let val = decode_bignum_bytes(reader)?;
                    Ok(val as i128)
                },
                // Tag 3 = negative bignum: value = -1 - bignum
                | 3 => {
                    let magnitude = decode_bignum_bytes(reader)?;
                    // -1 - magnitude, checking for overflow into i128
                    // i128::MIN = -170141183460469231731687303715884105728
                    // max magnitude representable: u128::MAX but we only need up to |i128::MIN| - 1
                    let result = -1i128 - (magnitude as i128);
                    // If magnitude was large enough that casting to i128 wraps, detect overflow
                    if magnitude > i128::MAX as u128 {
                        // magnitude as i128 would be negative; compute directly
                        // -1 - magnitude where magnitude > i128::MAX
                        //   = -(1 + magnitude)
                        // This is valid only if magnitude == i128::MAX as u128 + 1 (i.e., |i128::MIN|)
                        // which gives i128::MIN
                        if magnitude == (i128::MAX as u128) + 1 {
                            return Ok(i128::MIN);
                        }
                        return cold_decode_error_outside_isize_range(-1);
                    }
                    Ok(result)
                },
                | _ => cold_decode_error_unexpected_end(0),
            }
        },
        | _ => cold_decode_error_unexpected_end(0),
    }
}

/// Decode a CBOR byte string into a u128 (big-endian bignum).
#[inline]
fn decode_bignum_bytes<R: Reader>(reader: &mut R) -> Result<u128, DecodeError> {
    // Read byte string header (major type 2)
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    if major != 2 {
        return cold_decode_error_unexpected_end(0);
    }
    let len = decode_additional_info(reader, info)?;
    if len > 16 {
        // Value too large for u128
        return cold_decode_error_outside_usize_range::<u128>(len);
    }
    let len = len as usize;
    let mut buf = [0u8; 16];
    // Read into the end of the buffer (big-endian, right-aligned)
    reader.read(&mut buf[16 - len..])?;
    Ok(u128::from_be_bytes(buf))
}

/// Decode a `bool` value from CBOR.
#[inline]
pub fn decode_bool<R: Reader>(reader: &mut R) -> Result<bool, DecodeError> {
    let first = reader.read_u8()?;
    match first {
        | 0xF4 => Ok(false),
        | 0xF5 => Ok(true),
        | _ => cold_decode_error_invalid_boolean_value(first),
    }
}

/// Decode an `f32` value from CBOR.
#[inline]
pub fn decode_f32<R: Reader>(reader: &mut R) -> Result<f32, DecodeError> {
    let first = reader.read_u8()?;
    if first != 0xFA {
        return cold_decode_error_unexpected_end(4);
    }
    let mut bytes = [0u8; 4];
    reader.read(&mut bytes)?;
    Ok(f32::from_be_bytes(bytes))
}

/// Decode an `f64` value from CBOR.
#[inline]
pub fn decode_f64<R: Reader>(reader: &mut R) -> Result<f64, DecodeError> {
    let first = reader.read_u8()?;
    if first != 0xFB {
        return cold_decode_error_unexpected_end(8);
    }
    let mut bytes = [0u8; 8];
    reader.read(&mut bytes)?;
    Ok(f64::from_be_bytes(bytes))
}

/// Decode a slice length from CBOR.
#[inline]
pub fn decode_slice_len<R: Reader>(reader: &mut R) -> Result<usize, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    if major != 4 {
        return cold_decode_error_unexpected_end(0);
    }
    let val = decode_additional_info(reader, info)?;
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())
}

/// Decode a map length from CBOR.
#[inline]
pub fn decode_map_len<R: Reader>(reader: &mut R) -> Result<usize, DecodeError> {
    let first = reader.read_u8()?;
    let major = first >> 5;
    let info = first & 0x1F;
    if major != 5 {
        return cold_decode_error_unexpected_end(0);
    }
    let val = decode_additional_info(reader, info)?;
    val.try_into()
        .map_err(|_| cold_decode_error_outside_usize_range::<usize>(val).unwrap_err())
}
