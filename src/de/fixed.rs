//! Fixed-size integer and float decoding with unaligned direct buffer access and cold-path separation.
#![allow(unsafe_code)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::cast_possible_wrap)]

use crate::config::Endianness;
use crate::de::read::Reader;
use crate::error::DecodeError;

#[inline(always)]
pub fn decode_u16<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<u16, DecodeError> {
    if let Some(bytes) = read.peek_read(2) {
        let val = unsafe {
            let val = bytes.as_ptr().cast::<u16>().read_unaligned();
            match endian {
                | Endianness::Little => u16::from_le(val),
                | Endianness::Big => u16::from_be(val),
            }
        };
        read.consume(2);
        Ok(val)
    } else {
        decode_u16_cold(read, endian)
    }
}

#[inline(never)]
#[cold]
fn decode_u16_cold<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<u16, DecodeError> {
    let mut bytes = [0u8; 2];
    read.read(&mut bytes)?;
    let val = unsafe { bytes.as_ptr().cast::<u16>().read_unaligned() };
    match endian {
        | Endianness::Little => Ok(u16::from_le(val)),
        | Endianness::Big => Ok(u16::from_be(val)),
    }
}

#[inline(always)]
pub fn decode_u32<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<u32, DecodeError> {
    if let Some(bytes) = read.peek_read(4) {
        let val = unsafe {
            let val = bytes.as_ptr().cast::<u32>().read_unaligned();
            match endian {
                | Endianness::Little => u32::from_le(val),
                | Endianness::Big => u32::from_be(val),
            }
        };
        read.consume(4);
        Ok(val)
    } else {
        decode_u32_cold(read, endian)
    }
}

#[inline(never)]
#[cold]
fn decode_u32_cold<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<u32, DecodeError> {
    let mut bytes = [0u8; 4];
    read.read(&mut bytes)?;
    let val = unsafe { bytes.as_ptr().cast::<u32>().read_unaligned() };
    match endian {
        | Endianness::Little => Ok(u32::from_le(val)),
        | Endianness::Big => Ok(u32::from_be(val)),
    }
}

#[inline(always)]
pub fn decode_u64<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<u64, DecodeError> {
    if let Some(bytes) = read.peek_read(8) {
        let val = unsafe {
            let val = bytes.as_ptr().cast::<u64>().read_unaligned();
            match endian {
                | Endianness::Little => u64::from_le(val),
                | Endianness::Big => u64::from_be(val),
            }
        };
        read.consume(8);
        Ok(val)
    } else {
        decode_u64_cold(read, endian)
    }
}

#[inline(never)]
#[cold]
fn decode_u64_cold<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<u64, DecodeError> {
    let mut bytes = [0u8; 8];
    read.read(&mut bytes)?;
    let val = unsafe { bytes.as_ptr().cast::<u64>().read_unaligned() };
    match endian {
        | Endianness::Little => Ok(u64::from_le(val)),
        | Endianness::Big => Ok(u64::from_be(val)),
    }
}

#[inline(always)]
pub fn decode_u128<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<u128, DecodeError> {
    if let Some(bytes) = read.peek_read(16) {
        let val = unsafe {
            let val = bytes.as_ptr().cast::<u128>().read_unaligned();
            match endian {
                | Endianness::Little => u128::from_le(val),
                | Endianness::Big => u128::from_be(val),
            }
        };
        read.consume(16);
        Ok(val)
    } else {
        decode_u128_cold(read, endian)
    }
}

#[inline(never)]
#[cold]
fn decode_u128_cold<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<u128, DecodeError> {
    let mut bytes = [0u8; 16];
    read.read(&mut bytes)?;
    let val = unsafe { bytes.as_ptr().cast::<u128>().read_unaligned() };
    match endian {
        | Endianness::Little => Ok(u128::from_le(val)),
        | Endianness::Big => Ok(u128::from_be(val)),
    }
}

#[inline(always)]
#[allow(dead_code)]
pub fn decode_usize<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<usize, DecodeError> {
    let val = decode_u64(read, endian)?;
    val.try_into()
        .map_err(|_| crate::error::cold_decode_error_outside_usize_range::<()>(val).unwrap_err())
}

#[inline(always)]
pub fn decode_i16<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i16, DecodeError> {
    if let Some(bytes) = read.peek_read(2) {
        let val = unsafe {
            let val = bytes.as_ptr().cast::<i16>().read_unaligned();
            match endian {
                | Endianness::Little => i16::from_le(val),
                | Endianness::Big => i16::from_be(val),
            }
        };
        read.consume(2);
        Ok(val)
    } else {
        decode_i16_cold(read, endian)
    }
}

#[inline(never)]
#[cold]
fn decode_i16_cold<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i16, DecodeError> {
    let mut bytes = [0u8; 2];
    read.read(&mut bytes)?;
    let val = unsafe { bytes.as_ptr().cast::<i16>().read_unaligned() };
    match endian {
        | Endianness::Little => Ok(i16::from_le(val)),
        | Endianness::Big => Ok(i16::from_be(val)),
    }
}

#[inline(always)]
pub fn decode_i32<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i32, DecodeError> {
    if let Some(bytes) = read.peek_read(4) {
        let val = unsafe {
            let val = bytes.as_ptr().cast::<i32>().read_unaligned();
            match endian {
                | Endianness::Little => i32::from_le(val),
                | Endianness::Big => i32::from_be(val),
            }
        };
        read.consume(4);
        Ok(val)
    } else {
        decode_i32_cold(read, endian)
    }
}

#[inline(never)]
#[cold]
fn decode_i32_cold<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i32, DecodeError> {
    let mut bytes = [0u8; 4];
    read.read(&mut bytes)?;
    let val = unsafe { bytes.as_ptr().cast::<i32>().read_unaligned() };
    match endian {
        | Endianness::Little => Ok(i32::from_le(val)),
        | Endianness::Big => Ok(i32::from_be(val)),
    }
}

#[inline(always)]
pub fn decode_i64<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i64, DecodeError> {
    if let Some(bytes) = read.peek_read(8) {
        let val = unsafe {
            let val = bytes.as_ptr().cast::<i64>().read_unaligned();
            match endian {
                | Endianness::Little => i64::from_le(val),
                | Endianness::Big => i64::from_be(val),
            }
        };
        read.consume(8);
        Ok(val)
    } else {
        decode_i64_cold(read, endian)
    }
}

#[inline(never)]
#[cold]
fn decode_i64_cold<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i64, DecodeError> {
    let mut bytes = [0u8; 8];
    read.read(&mut bytes)?;
    let val = unsafe { bytes.as_ptr().cast::<i64>().read_unaligned() };
    match endian {
        | Endianness::Little => Ok(i64::from_le(val)),
        | Endianness::Big => Ok(i64::from_be(val)),
    }
}

#[inline(always)]
pub fn decode_i128<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i128, DecodeError> {
    if let Some(bytes) = read.peek_read(16) {
        let val = unsafe {
            let val = bytes.as_ptr().cast::<i128>().read_unaligned();
            match endian {
                | Endianness::Little => i128::from_le(val),
                | Endianness::Big => i128::from_be(val),
            }
        };
        read.consume(16);
        Ok(val)
    } else {
        decode_i128_cold(read, endian)
    }
}

#[inline(never)]
#[cold]
fn decode_i128_cold<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i128, DecodeError> {
    let mut bytes = [0u8; 16];
    read.read(&mut bytes)?;
    let val = unsafe { bytes.as_ptr().cast::<i128>().read_unaligned() };
    match endian {
        | Endianness::Little => Ok(i128::from_le(val)),
        | Endianness::Big => Ok(i128::from_be(val)),
    }
}

#[inline(always)]
#[allow(dead_code)]
pub fn decode_isize<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<isize, DecodeError> {
    let val = decode_i64(read, endian)?;
    val.try_into()
        .map_err(|_| crate::error::cold_decode_error_outside_isize_range::<()>(val).unwrap_err())
}

#[inline(always)]
pub fn decode_f32<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<f32, DecodeError> {
    let bits = decode_u32(read, endian)?;
    Ok(f32::from_bits(bits))
}

#[inline(always)]
pub fn decode_f64<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<f64, DecodeError> {
    let bits = decode_u64(read, endian)?;
    Ok(f64::from_bits(bits))
}
