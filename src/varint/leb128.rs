use crate::{enc::write::Writer, error::EncodeError};

pub fn leb128_encode_u128<W: Writer>(writer: &mut W, mut val: u128) -> Result<(), EncodeError> {
    loop {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val == 0 {
            writer.write(&[byte])?;
            return Ok(());
        } else {
            byte |= 0x80;
            writer.write(&[byte])?;
        }
    }
}

pub fn leb128_encode_u64<W: Writer>(writer: &mut W, val: u64) -> Result<(), EncodeError> {
    leb128_encode_u128(writer, val as u128)
}

pub fn leb128_encode_u32<W: Writer>(writer: &mut W, val: u32) -> Result<(), EncodeError> {
    leb128_encode_u128(writer, val as u128)
}

pub fn leb128_encode_u16<W: Writer>(writer: &mut W, val: u16) -> Result<(), EncodeError> {
    leb128_encode_u128(writer, val as u128)
}

pub fn leb128_encode_usize<W: Writer>(writer: &mut W, val: usize) -> Result<(), EncodeError> {
    leb128_encode_u128(writer, val as u128)
}

pub fn sleb128_encode_i128<W: Writer>(writer: &mut W, mut val: i128) -> Result<(), EncodeError> {
    loop {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if (val == 0 && (byte & 0x40) == 0) || (val == -1 && (byte & 0x40) != 0) {
            writer.write(&[byte])?;
            return Ok(());
        } else {
            byte |= 0x80;
            writer.write(&[byte])?;
        }
    }
}

pub fn sleb128_encode_i64<W: Writer>(writer: &mut W, val: i64) -> Result<(), EncodeError> {
    sleb128_encode_i128(writer, val as i128)
}

pub fn sleb128_encode_i32<W: Writer>(writer: &mut W, val: i32) -> Result<(), EncodeError> {
    sleb128_encode_i128(writer, val as i128)
}

pub fn sleb128_encode_i16<W: Writer>(writer: &mut W, val: i16) -> Result<(), EncodeError> {
    sleb128_encode_i128(writer, val as i128)
}

pub fn sleb128_encode_isize<W: Writer>(writer: &mut W, val: isize) -> Result<(), EncodeError> {
    sleb128_encode_i128(writer, val as i128)
}

pub fn leb128_decode_u128<R: crate::de::read::Reader>(reader: &mut R) -> Result<u128, crate::error::DecodeError> {
    let mut result = 0;
    let mut shift = 0;
    loop {
        let mut byte = [0u8; 1];
        reader.read(&mut byte)?;
        let byte = byte[0];
        result |= ((byte & 0x7F) as u128) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 128 {
            return Err(crate::error::DecodeError::Other("LEB128 overflow"));
        }
    }
}

pub fn leb128_decode_u64<R: crate::de::read::Reader>(reader: &mut R) -> Result<u64, crate::error::DecodeError> {
    let val = leb128_decode_u128(reader)?;
    if val > u64::MAX as u128 {
        return Err(crate::error::DecodeError::Other("LEB128 overflow u64"));
    }
    Ok(val as u64)
}

pub fn leb128_decode_u32<R: crate::de::read::Reader>(reader: &mut R) -> Result<u32, crate::error::DecodeError> {
    let val = leb128_decode_u128(reader)?;
    if val > u32::MAX as u128 {
        return Err(crate::error::DecodeError::Other("LEB128 overflow u32"));
    }
    Ok(val as u32)
}

pub fn leb128_decode_u16<R: crate::de::read::Reader>(reader: &mut R) -> Result<u16, crate::error::DecodeError> {
    let val = leb128_decode_u128(reader)?;
    if val > u16::MAX as u128 {
        return Err(crate::error::DecodeError::Other("LEB128 overflow u16"));
    }
    Ok(val as u16)
}

pub fn leb128_decode_usize<R: crate::de::read::Reader>(reader: &mut R) -> Result<usize, crate::error::DecodeError> {
    let val = leb128_decode_u128(reader)?;
    if val > usize::MAX as u128 {
        return Err(crate::error::DecodeError::Other("LEB128 overflow usize"));
    }
    Ok(val as usize)
}

pub fn sleb128_decode_i128<R: crate::de::read::Reader>(reader: &mut R) -> Result<i128, crate::error::DecodeError> {
    let mut result = 0;
    let mut shift = 0;
    let mut byte;
    loop {
        let mut b = [0u8; 1];
        reader.read(&mut b)?;
        byte = b[0];
        result |= ((byte & 0x7F) as i128) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
    }
    if shift < 128 && (byte & 0x40) != 0 {
        result |= !0 << shift;
    }
    Ok(result)
}

pub fn sleb128_decode_i64<R: crate::de::read::Reader>(reader: &mut R) -> Result<i64, crate::error::DecodeError> {
    let val = sleb128_decode_i128(reader)?;
    if val < i64::MIN as i128 || val > i64::MAX as i128 {
        return Err(crate::error::DecodeError::Other("SLEB128 overflow i64"));
    }
    Ok(val as i64)
}

pub fn sleb128_decode_i32<R: crate::de::read::Reader>(reader: &mut R) -> Result<i32, crate::error::DecodeError> {
    let val = sleb128_decode_i128(reader)?;
    if val < i32::MIN as i128 || val > i32::MAX as i128 {
        return Err(crate::error::DecodeError::Other("SLEB128 overflow i32"));
    }
    Ok(val as i32)
}

pub fn sleb128_decode_i16<R: crate::de::read::Reader>(reader: &mut R) -> Result<i16, crate::error::DecodeError> {
    let val = sleb128_decode_i128(reader)?;
    if val < i16::MIN as i128 || val > i16::MAX as i128 {
        return Err(crate::error::DecodeError::Other("SLEB128 overflow i16"));
    }
    Ok(val as i16)
}

pub fn sleb128_decode_isize<R: crate::de::read::Reader>(reader: &mut R) -> Result<isize, crate::error::DecodeError> {
    let val = sleb128_decode_i128(reader)?;
    if val < isize::MIN as i128 || val > isize::MAX as i128 {
        return Err(crate::error::DecodeError::Other("SLEB128 overflow isize"));
    }
    Ok(val as isize)
}
