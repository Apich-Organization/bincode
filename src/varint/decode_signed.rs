#![allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
use crate::config::Endianness;
use crate::de::read::Reader;
use crate::error::DecodeError;
use crate::error::IntegerType;

#[inline(always)]
pub fn varint_decode_i16<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<i16, DecodeError> {
    let n = super::varint_decode_u16(read, endian)
        .map_err(DecodeError::change_integer_type_to_signed)?;
    Ok(if n % 2 == 0 {
        // positive number
        (n / 2) as _
    } else {
        // negative number
        // !m * 2 + 1 = n
        // !m * 2 = n - 1
        // !m = (n - 1) / 2
        // m = !((n - 1) / 2)
        // since we have n is odd, we have floor(n / 2) = floor((n - 1) / 2)
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
        // positive number
        (n / 2) as _
    } else {
        // negative number
        // !m * 2 + 1 = n
        // !m * 2 = n - 1
        // !m = (n - 1) / 2
        // m = !((n - 1) / 2)
        // since we have n is odd, we have floor(n / 2) = floor((n - 1) / 2)
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
        // positive number
        (n / 2) as _
    } else {
        // negative number
        // !m * 2 + 1 = n
        // !m * 2 = n - 1
        // !m = (n - 1) / 2
        // m = !((n - 1) / 2)
        // since we have n is odd, we have floor(n / 2) = floor((n - 1) / 2)
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
        // positive number
        (n / 2) as _
    } else {
        // negative number
        // !m * 2 + 1 = n
        // !m * 2 = n - 1
        // !m = (n - 1) / 2
        // m = !((n - 1) / 2)
        // since we have n is odd, we have floor(n / 2) = floor((n - 1) / 2)
        !(n / 2) as _
    })
}

#[inline(always)]
pub fn varint_decode_isize<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<isize, DecodeError> {
    match varint_decode_i64(read, endian) {
        | Ok(val) => {
            val.try_into().map_err(|_| {
                crate::error::cold_decode_error_outside_isize_range::<()>(val).unwrap_err()
            })
        },
        | Err(DecodeError::InvalidIntegerType { found, .. }) => {
            crate::error::cold_decode_error_invalid_integer_type(
                IntegerType::Isize,
                found.into_signed(),
            )
        },
        | Err(e) => Err(e),
    }
}

#[test]
fn test_decode_i16() {
    let cases: &[(&[u8], &[u8], i16)] = &[
        (&[0], &[0], 0),
        (&[4], &[4], 2),
        (&[3], &[3], -2),
        (
            &[crate::varint::U16_BYTE, 0, 2],
            &[crate::varint::U16_BYTE, 2, 0],
            256,
        ),
        (
            &[crate::varint::U16_BYTE, 255, 1],
            &[crate::varint::U16_BYTE, 1, 255],
            -256,
        ),
        (
            &[crate::varint::U16_BYTE, 0, 125],
            &[crate::varint::U16_BYTE, 125, 0],
            16000,
        ),
        (
            &[crate::varint::U16_BYTE, 255, 124],
            &[crate::varint::U16_BYTE, 124, 255],
            -16000,
        ),
        (
            &[crate::varint::U16_BYTE, 252, 255],
            &[crate::varint::U16_BYTE, 255, 252],
            32766,
        ),
        (
            &[crate::varint::U16_BYTE, 254, 255],
            &[crate::varint::U16_BYTE, 255, 254],
            32767,
        ),
        (
            &[crate::varint::U16_BYTE, 253, 255],
            &[crate::varint::U16_BYTE, 255, 253],
            -32767,
        ),
        (
            &[crate::varint::U16_BYTE, 255, 255],
            &[crate::varint::U16_BYTE, 255, 255],
            -32768,
        ),
    ];

    for &(slice_le, slice_be, expected) in cases {
        let mut reader = crate::de::read::SliceReader::new(slice_le);
        let found = varint_decode_i16(&mut reader, Endianness::Little).unwrap();
        assert_eq!(expected, found);

        let mut reader = crate::de::read::SliceReader::new(slice_be);
        let found = varint_decode_i16(&mut reader, Endianness::Big).unwrap();
        assert_eq!(expected, found);
    }

    let errors: &[(&[u8], DecodeError)] = &[
        (
            &[crate::varint::U32_BYTE],
            DecodeError::InvalidIntegerType {
                expected: IntegerType::I16,
                found: IntegerType::I32,
            },
        ),
        (
            &[crate::varint::U64_BYTE],
            DecodeError::InvalidIntegerType {
                expected: IntegerType::I16,
                found: IntegerType::I64,
            },
        ),
        (
            &[crate::varint::U128_BYTE],
            DecodeError::InvalidIntegerType {
                expected: IntegerType::I16,
                found: IntegerType::I128,
            },
        ),
        (
            &[crate::varint::U16_BYTE],
            DecodeError::UnexpectedEnd { additional: 2 },
        ),
        (
            &[crate::varint::U16_BYTE, 0],
            DecodeError::UnexpectedEnd { additional: 1 },
        ),
    ];

    for (slice, expected) in errors {
        let mut reader = crate::de::read::SliceReader::new(slice);
        let found = varint_decode_i16(&mut reader, Endianness::Little).unwrap_err();
        assert_eq!(std::format!("{expected:?}"), std::format!("{found:?}"));
    }
}
