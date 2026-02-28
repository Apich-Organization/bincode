//! CBOR (RFC 8949) encoding implementation.

use crate::enc::write::Writer;
use crate::error::EncodeError;

/// Encode a `u8` value into CBOR.
#[inline]
pub fn encode_u8<W: Writer>(
    writer: &mut W,
    val: u8,
) -> Result<(), EncodeError> {
    if val <= 23 {
        writer.write_u8(val)
    } else {
        writer.write_u8(24)?;
        writer.write_u8(val)
    }
}

/// Encode a `u16` value into CBOR.
#[inline]
pub fn encode_u16<W: Writer>(
    writer: &mut W,
    val: u16,
) -> Result<(), EncodeError> {
    if val <= 23 {
        writer.write_u8(val as u8)
    } else if val <= 0xFF {
        writer.write_u8(24)?;
        writer.write_u8(val as u8)
    } else {
        writer.write_u8(25)?;
        writer.write(&val.to_be_bytes())
    }
}

/// Encode a `u32` value into CBOR.
#[inline]
pub fn encode_u32<W: Writer>(
    writer: &mut W,
    val: u32,
) -> Result<(), EncodeError> {
    if val <= 23 {
        writer.write_u8(val as u8)
    } else if val <= 0xFF {
        writer.write_u8(24)?;
        writer.write_u8(val as u8)
    } else if val <= 0xFFFF {
        writer.write_u8(25)?;
        writer.write(&(val as u16).to_be_bytes())
    } else {
        writer.write_u8(26)?;
        writer.write(&val.to_be_bytes())
    }
}

/// Encode a `u64` value into CBOR.
#[inline]
pub fn encode_u64<W: Writer>(
    writer: &mut W,
    val: u64,
) -> Result<(), EncodeError> {
    if val <= 23 {
        writer.write_u8(val as u8)
    } else if val <= 0xFF {
        writer.write_u8(24)?;
        writer.write_u8(val as u8)
    } else if val <= 0xFFFF {
        writer.write_u8(25)?;
        writer.write(&(val as u16).to_be_bytes())
    } else if val <= 0xFFFFFFFF {
        writer.write_u8(26)?;
        writer.write(&(val as u32).to_be_bytes())
    } else {
        writer.write_u8(27)?;
        writer.write(&val.to_be_bytes())
    }
}

/// Encode an `i8` value into CBOR.
#[inline]
pub fn encode_i8<W: Writer>(
    writer: &mut W,
    val: i8,
) -> Result<(), EncodeError> {
    if val >= 0 {
        encode_u8(writer, val as u8)
    } else {
        let val = -1 - val;
        let val = val as u8;
        if val <= 23 {
            writer.write_u8(0x20 | val)
        } else {
            writer.write_u8(0x20 | 24)?;
            writer.write_u8(val)
        }
    }
}

/// Encode an `i16` value into CBOR.
#[inline]
pub fn encode_i16<W: Writer>(
    writer: &mut W,
    val: i16,
) -> Result<(), EncodeError> {
    if val >= 0 {
        encode_u16(writer, val as u16)
    } else {
        let val = -1 - val; // i16::MIN is -32768, -1 - (-32768) = 32767 (fits in u16)
        let val = val as u16;
        if val <= 23 {
            writer.write_u8(0x20 | val as u8)
        } else if val <= 0xFF {
            writer.write_u8(0x20 | 24)?;
            writer.write_u8(val as u8)
        } else {
            writer.write_u8(0x20 | 25)?;
            writer.write(&val.to_be_bytes())
        }
    }
}

/// Encode an `i32` value into CBOR.
#[inline]
pub fn encode_i32<W: Writer>(
    writer: &mut W,
    val: i32,
) -> Result<(), EncodeError> {
    if val >= 0 {
        encode_u32(writer, val as u32)
    } else {
        let val = -1 - val;
        let val = val as u32;
        if val <= 23 {
            writer.write_u8(0x20 | val as u8)
        } else if val <= 0xFF {
            writer.write_u8(0x20 | 24)?;
            writer.write_u8(val as u8)
        } else if val <= 0xFFFF {
            writer.write_u8(0x20 | 25)?;
            writer.write(&(val as u16).to_be_bytes())
        } else {
            writer.write_u8(0x20 | 26)?;
            writer.write(&val.to_be_bytes())
        }
    }
}

/// Encode an `i64` value into CBOR.
#[inline]
pub fn encode_i64<W: Writer>(
    writer: &mut W,
    val: i64,
) -> Result<(), EncodeError> {
    if val >= 0 {
        encode_u64(writer, val as u64)
    } else {
        let val = -1 - val;
        let val = val as u64;
        if val <= 23 {
            writer.write_u8(0x20 | val as u8)
        } else if val <= 0xFF {
            writer.write_u8(0x20 | 24)?;
            writer.write_u8(val as u8)
        } else if val <= 0xFFFF {
            writer.write_u8(0x20 | 25)?;
            writer.write(&(val as u16).to_be_bytes())
        } else if val <= 0xFFFFFFFF {
            writer.write_u8(0x20 | 26)?;
            writer.write(&(val as u32).to_be_bytes())
        } else {
            writer.write_u8(0x20 | 27)?;
            writer.write(&val.to_be_bytes())
        }
    }
}

/// Encode a `u128` value into CBOR.
///
/// Values ≤ u64::MAX use standard CBOR unsigned integer encoding.
/// Values > u64::MAX use Tag 2 (positive bignum) with a byte string
/// containing the minimal big-endian representation (RFC 8949 §3.4.3).
#[inline]
pub fn encode_u128<W: Writer>(
    writer: &mut W,
    val: u128,
) -> Result<(), EncodeError> {
    if val <= u64::MAX as u128 {
        return encode_u64(writer, val as u64);
    }
    // Tag 2 = positive bignum
    writer.write_u8(0xC2)?;
    // Encode as byte string (major type 2) with minimal big-endian bytes
    let bytes = val.to_be_bytes(); // 16 bytes
    // Find first non-zero byte for minimal representation
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(15);
    let len = 16 - start;
    encode_bytestring_header(writer, len)?;
    writer.write(&bytes[start..])
}

/// Encode an `i128` value into CBOR.
///
/// Non-negative values use standard CBOR unsigned encoding (or Tag 2 bignum).
/// Negative values that fit in i64 use standard CBOR negative integer encoding.
/// Negative values < i64::MIN use Tag 3 (negative bignum) with a byte string
/// containing the minimal big-endian representation of `-1 - val` (RFC 8949 §3.4.3).
#[inline]
pub fn encode_i128<W: Writer>(
    writer: &mut W,
    val: i128,
) -> Result<(), EncodeError> {
    if val >= 0 {
        return encode_u128(writer, val as u128);
    }
    // For negative values, CBOR encodes -1 - val as the unsigned magnitude
    let magnitude = (-1i128 - val) as u128;
    if magnitude <= u64::MAX as u128 {
        // Fits in standard CBOR negative integer encoding
        let mag64 = magnitude as u64;
        if mag64 <= 23 {
            writer.write_u8(0x20 | mag64 as u8)
        } else if mag64 <= 0xFF {
            writer.write_u8(0x20 | 24)?;
            writer.write_u8(mag64 as u8)
        } else if mag64 <= 0xFFFF {
            writer.write_u8(0x20 | 25)?;
            writer.write(&(mag64 as u16).to_be_bytes())
        } else if mag64 <= 0xFFFF_FFFF {
            writer.write_u8(0x20 | 26)?;
            writer.write(&(mag64 as u32).to_be_bytes())
        } else {
            writer.write_u8(0x20 | 27)?;
            writer.write(&mag64.to_be_bytes())
        }
    } else {
        // Tag 3 = negative bignum; value is -1 - n where n is the bignum
        writer.write_u8(0xC3)?;
        let bytes = magnitude.to_be_bytes();
        let start = bytes.iter().position(|&b| b != 0).unwrap_or(15);
        let len = 16 - start;
        encode_bytestring_header(writer, len)?;
        writer.write(&bytes[start..])
    }
}

/// Encode a CBOR byte string header (major type 2).
#[inline]
fn encode_bytestring_header<W: Writer>(
    writer: &mut W,
    len: usize,
) -> Result<(), EncodeError> {
    if len <= 23 {
        writer.write_u8(0x40 | len as u8)
    } else if len <= 0xFF {
        writer.write_u8(0x40 | 24)?;
        writer.write_u8(len as u8)
    } else if len <= 0xFFFF {
        writer.write_u8(0x40 | 25)?;
        writer.write(&(len as u16).to_be_bytes())
    } else if len <= 0xFFFF_FFFF {
        writer.write_u8(0x40 | 26)?;
        writer.write(&(len as u32).to_be_bytes())
    } else {
        writer.write_u8(0x40 | 27)?;
        writer.write(&(len as u64).to_be_bytes())
    }
}

/// Encode a `bool` value into CBOR.
#[inline]
pub fn encode_bool<W: Writer>(
    writer: &mut W,
    val: bool,
) -> Result<(), EncodeError> {
    if val {
        writer.write_u8(0xF5) // True (Simple value 21)
    } else {
        writer.write_u8(0xF4) // False (Simple value 20)
    }
}

/// Encode an `f32` value into CBOR.
#[inline]
pub fn encode_f32<W: Writer>(
    writer: &mut W,
    val: f32,
) -> Result<(), EncodeError> {
    writer.write_u8(0xFA)?;
    writer.write(&val.to_be_bytes())
}

/// Encode an `f64` value into CBOR.
#[inline]
pub fn encode_f64<W: Writer>(
    writer: &mut W,
    val: f64,
) -> Result<(), EncodeError> {
    writer.write_u8(0xFB)?;
    writer.write(&val.to_be_bytes())
}

/// Encode a `str` value into CBOR.
#[inline]
pub fn encode_str<W: Writer>(
    writer: &mut W,
    val: &str,
) -> Result<(), EncodeError> {
    let len = val.len();
    if len <= 23 {
        writer.write_u8(0x60 | len as u8)?;
    } else if len <= 0xFF {
        writer.write_u8(0x60 | 24)?;
        writer.write_u8(len as u8)?;
    } else if len <= 0xFFFF {
        writer.write_u8(0x60 | 25)?;
        writer.write(&(len as u16).to_be_bytes())?;
    } else if len <= 0xFFFFFFFF {
        writer.write_u8(0x60 | 26)?;
        writer.write(&(len as u32).to_be_bytes())?;
    } else {
        writer.write_u8(0x60 | 27)?;
        writer.write(&(len as u64).to_be_bytes())?;
    }
    writer.write(val.as_bytes())
}

/// Encode a slice length into CBOR.
#[inline]
pub fn encode_slice_len<W: Writer>(
    writer: &mut W,
    len: usize,
) -> Result<(), EncodeError> {
    // Array (Major type 4)
    if len <= 23 {
        writer.write_u8(0x80 | len as u8)
    } else if len <= 0xFF {
        writer.write_u8(0x80 | 24)?;
        writer.write_u8(len as u8)
    } else if len <= 0xFFFF {
        writer.write_u8(0x80 | 25)?;
        writer.write(&(len as u16).to_be_bytes())
    } else if len <= 0xFFFFFFFF {
        writer.write_u8(0x80 | 26)?;
        writer.write(&(len as u32).to_be_bytes())
    } else {
        writer.write_u8(0x80 | 27)?;
        writer.write(&(len as u64).to_be_bytes())
    }
}

/// Encode a map length into CBOR.
#[inline]
pub fn encode_map_len<W: Writer>(
    writer: &mut W,
    len: usize,
) -> Result<(), EncodeError> {
    // Map (Major type 5)
    if len <= 23 {
        writer.write_u8(0xA0 | len as u8)
    } else if len <= 0xFF {
        writer.write_u8(0xA0 | 24)?;
        writer.write_u8(len as u8)
    } else if len <= 0xFFFF {
        writer.write_u8(0xA0 | 25)?;
        writer.write(&(len as u16).to_be_bytes())
    } else if len <= 0xFFFFFFFF {
        writer.write_u8(0xA0 | 26)?;
        writer.write(&(len as u32).to_be_bytes())
    } else {
        writer.write_u8(0xA0 | 27)?;
        writer.write(&(len as u64).to_be_bytes())
    }
}
