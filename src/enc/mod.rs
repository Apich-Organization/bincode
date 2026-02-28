//! Encoder-based structs and traits.

mod encoder;
mod impl_tuples;
mod impls;

use self::write::Writer;
use crate::config::Config;
use crate::error::EncodeError;
use crate::utils::Sealed;

/// Bit-level writer for space-optimized packing.
pub mod bit_writer;
pub mod cbor;
pub mod write;

pub use self::encoder::EncoderImpl;

/// Any source that can be encoded. This trait should be implemented for all types that you want to be able to use with any of the `encode_with` methods.
///
/// This trait will be automatically implemented if you enable the `derive` feature and add `#[derive(bincode::Encode)]` to your trait.
///
/// # Implementing this trait manually
///
/// If you want to implement this trait for your type, the easiest way is to add a `#[derive(bincode::Encode)]`, build and check your `target/generated/bincode/` folder. This should generate a `<Struct name>_Encode.rs` file.
///
/// For this struct:
///
/// ```
/// struct Entity {
///     pub x: f32,
///     pub y: f32,
/// }
/// ```
/// It will look something like:
///
/// ```
/// # struct Entity {
/// #     pub x: f32,
/// #     pub y: f32,
/// # }
/// impl bincode_next::Encode for Entity {
///     fn encode<E: bincode_next::enc::Encoder>(
///         &self,
///         encoder: &mut E,
///     ) -> core::result::Result<(), bincode_next::error::EncodeError> {
///         bincode_next::Encode::encode(&self.x, encoder)?;
///         bincode_next::Encode::encode(&self.y, encoder)?;
///         Ok(())
///     }
/// }
/// ```
///
/// From here you can add/remove fields, or add custom logic.
pub trait Encode {
    /// Encode a given type.
    ///
    /// # Errors
    ///
    /// Returns any error encountered during encoding.
    fn encode<E: Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError>;
}

/// Helper trait to encode basic types into.
pub trait Encoder: Sealed + crate::error_path::BincodeErrorPathCovered<1> {
    /// The concrete [Writer] type
    type W: Writer;

    /// The concrete [Config] type
    type C: Config;

    /// Returns a mutable reference to the writer
    fn writer(&mut self) -> &mut Self::W;

    /// Returns a reference to the config
    fn config(&self) -> &Self::C;

    /// Encode a `u8` value.
    fn encode_u8(
        &mut self,
        val: u8,
    ) -> Result<(), EncodeError> {
        use crate::config::Format;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => self.writer().write_u8(val),
            | Format::Cbor | Format::CborDeterministic => cbor::encode_u8(self.writer(), val),
        }
    }

    /// Encode a `u16` value.
    fn encode_u16(
        &mut self,
        val: u16,
    ) -> Result<(), EncodeError> {
        use crate::config::Endianness;
        use crate::config::Format;
        use crate::config::IntEncoding;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                match <Self::C as crate::config::InternalIntEncodingConfig>::INT_ENCODING {
                    | IntEncoding::Variable => {
                        crate::varint::varint_encode_u16(
                            self.writer(),
                            <Self::C as crate::config::InternalEndianConfig>::ENDIAN,
                            val,
                        )
                    },
                    | IntEncoding::Fixed => {
                        match <Self::C as crate::config::InternalEndianConfig>::ENDIAN {
                            | Endianness::Big => self.writer().write(&val.to_be_bytes()),
                            | Endianness::Little => self.writer().write(&val.to_le_bytes()),
                        }
                    },
                }
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_u16(self.writer(), val),
        }
    }

    /// Encode a `u32` value.
    fn encode_u32(
        &mut self,
        val: u32,
    ) -> Result<(), EncodeError> {
        use crate::config::Endianness;
        use crate::config::Format;
        use crate::config::IntEncoding;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                match <Self::C as crate::config::InternalIntEncodingConfig>::INT_ENCODING {
                    | IntEncoding::Variable => {
                        crate::varint::varint_encode_u32(
                            self.writer(),
                            <Self::C as crate::config::InternalEndianConfig>::ENDIAN,
                            val,
                        )
                    },
                    | IntEncoding::Fixed => {
                        match <Self::C as crate::config::InternalEndianConfig>::ENDIAN {
                            | Endianness::Big => self.writer().write(&val.to_be_bytes()),
                            | Endianness::Little => self.writer().write(&val.to_le_bytes()),
                        }
                    },
                }
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_u32(self.writer(), val),
        }
    }

    /// Encode a `u64` value.
    fn encode_u64(
        &mut self,
        val: u64,
    ) -> Result<(), EncodeError> {
        use crate::config::Endianness;
        use crate::config::Format;
        use crate::config::IntEncoding;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                match <Self::C as crate::config::InternalIntEncodingConfig>::INT_ENCODING {
                    | IntEncoding::Variable => {
                        crate::varint::varint_encode_u64(
                            self.writer(),
                            <Self::C as crate::config::InternalEndianConfig>::ENDIAN,
                            val,
                        )
                    },
                    | IntEncoding::Fixed => {
                        match <Self::C as crate::config::InternalEndianConfig>::ENDIAN {
                            | Endianness::Big => self.writer().write(&val.to_be_bytes()),
                            | Endianness::Little => self.writer().write(&val.to_le_bytes()),
                        }
                    },
                }
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_u64(self.writer(), val),
        }
    }

    /// Encode a `u128` value.
    fn encode_u128(
        &mut self,
        val: u128,
    ) -> Result<(), EncodeError> {
        use crate::config::Endianness;
        use crate::config::Format;
        use crate::config::IntEncoding;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                match <Self::C as crate::config::InternalIntEncodingConfig>::INT_ENCODING {
                    | IntEncoding::Variable => {
                        crate::varint::varint_encode_u128(
                            self.writer(),
                            <Self::C as crate::config::InternalEndianConfig>::ENDIAN,
                            val,
                        )
                    },
                    | IntEncoding::Fixed => {
                        match <Self::C as crate::config::InternalEndianConfig>::ENDIAN {
                            | Endianness::Big => self.writer().write(&val.to_be_bytes()),
                            | Endianness::Little => self.writer().write(&val.to_le_bytes()),
                        }
                    },
                }
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_u128(self.writer(), val),
        }
    }

    /// Encode a `usize` value.
    fn encode_usize(
        &mut self,
        val: usize,
    ) -> Result<(), EncodeError> {
        use crate::config::Format;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => self.encode_u64(val as u64),
            | Format::Cbor | Format::CborDeterministic => {
                cbor::encode_u64(self.writer(), val as u64)
            },
        }
    }

    /// Encode an `i8` value.
    fn encode_i8(
        &mut self,
        val: i8,
    ) -> Result<(), EncodeError> {
        use crate::config::Format;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => self.writer().write_u8(val as u8),
            | Format::Cbor | Format::CborDeterministic => cbor::encode_i8(self.writer(), val),
        }
    }

    /// Encode an `i16` value.
    fn encode_i16(
        &mut self,
        val: i16,
    ) -> Result<(), EncodeError> {
        use crate::config::Endianness;
        use crate::config::Format;
        use crate::config::IntEncoding;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                match <Self::C as crate::config::InternalIntEncodingConfig>::INT_ENCODING {
                    | IntEncoding::Variable => {
                        crate::varint::varint_encode_i16(
                            self.writer(),
                            <Self::C as crate::config::InternalEndianConfig>::ENDIAN,
                            val,
                        )
                    },
                    | IntEncoding::Fixed => {
                        match <Self::C as crate::config::InternalEndianConfig>::ENDIAN {
                            | Endianness::Big => self.writer().write(&val.to_be_bytes()),
                            | Endianness::Little => self.writer().write(&val.to_le_bytes()),
                        }
                    },
                }
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_i16(self.writer(), val),
        }
    }

    /// Encode an `i32` value.
    fn encode_i32(
        &mut self,
        val: i32,
    ) -> Result<(), EncodeError> {
        use crate::config::Endianness;
        use crate::config::Format;
        use crate::config::IntEncoding;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                match <Self::C as crate::config::InternalIntEncodingConfig>::INT_ENCODING {
                    | IntEncoding::Variable => {
                        crate::varint::varint_encode_i32(
                            self.writer(),
                            <Self::C as crate::config::InternalEndianConfig>::ENDIAN,
                            val,
                        )
                    },
                    | IntEncoding::Fixed => {
                        match <Self::C as crate::config::InternalEndianConfig>::ENDIAN {
                            | Endianness::Big => self.writer().write(&val.to_be_bytes()),
                            | Endianness::Little => self.writer().write(&val.to_le_bytes()),
                        }
                    },
                }
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_i32(self.writer(), val),
        }
    }

    /// Encode an `i64` value.
    fn encode_i64(
        &mut self,
        val: i64,
    ) -> Result<(), EncodeError> {
        use crate::config::Endianness;
        use crate::config::Format;
        use crate::config::IntEncoding;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                match <Self::C as crate::config::InternalIntEncodingConfig>::INT_ENCODING {
                    | IntEncoding::Variable => {
                        crate::varint::varint_encode_i64(
                            self.writer(),
                            <Self::C as crate::config::InternalEndianConfig>::ENDIAN,
                            val,
                        )
                    },
                    | IntEncoding::Fixed => {
                        match <Self::C as crate::config::InternalEndianConfig>::ENDIAN {
                            | Endianness::Big => self.writer().write(&val.to_be_bytes()),
                            | Endianness::Little => self.writer().write(&val.to_le_bytes()),
                        }
                    },
                }
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_i64(self.writer(), val),
        }
    }

    /// Encode an `i128` value.
    fn encode_i128(
        &mut self,
        val: i128,
    ) -> Result<(), EncodeError> {
        use crate::config::Endianness;
        use crate::config::Format;
        use crate::config::IntEncoding;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                match <Self::C as crate::config::InternalIntEncodingConfig>::INT_ENCODING {
                    | IntEncoding::Variable => {
                        crate::varint::varint_encode_i128(
                            self.writer(),
                            <Self::C as crate::config::InternalEndianConfig>::ENDIAN,
                            val,
                        )
                    },
                    | IntEncoding::Fixed => {
                        match <Self::C as crate::config::InternalEndianConfig>::ENDIAN {
                            | Endianness::Big => self.writer().write(&val.to_be_bytes()),
                            | Endianness::Little => self.writer().write(&val.to_le_bytes()),
                        }
                    },
                }
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_i128(self.writer(), val),
        }
    }

    /// Encode an `isize` value.
    fn encode_isize(
        &mut self,
        val: isize,
    ) -> Result<(), EncodeError> {
        use crate::config::Format;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => self.encode_i64(val as i64),
            | Format::Cbor | Format::CborDeterministic => {
                cbor::encode_i64(self.writer(), val as i64)
            },
        }
    }

    /// Encode an `f32` value.
    fn encode_f32(
        &mut self,
        val: f32,
    ) -> Result<(), EncodeError> {
        use crate::config::Endianness;
        use crate::config::Format;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                match <Self::C as crate::config::InternalEndianConfig>::ENDIAN {
                    | Endianness::Big => self.writer().write(&val.to_be_bytes()),
                    | Endianness::Little => self.writer().write(&val.to_le_bytes()),
                }
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_f32(self.writer(), val),
        }
    }

    /// Encode an `f64` value.
    fn encode_f64(
        &mut self,
        val: f64,
    ) -> Result<(), EncodeError> {
        use crate::config::Endianness;
        use crate::config::Format;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                match <Self::C as crate::config::InternalEndianConfig>::ENDIAN {
                    | Endianness::Big => self.writer().write(&val.to_be_bytes()),
                    | Endianness::Little => self.writer().write(&val.to_le_bytes()),
                }
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_f64(self.writer(), val),
        }
    }

    /// Encode a `bool` value.
    fn encode_bool(
        &mut self,
        val: bool,
    ) -> Result<(), EncodeError> {
        use crate::config::Format;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => self.encode_u8(val as u8),
            | Format::Cbor | Format::CborDeterministic => cbor::encode_bool(self.writer(), val),
        }
    }

    /// Encode a string.
    fn encode_str(
        &mut self,
        val: &str,
    ) -> Result<(), EncodeError> {
        use crate::config::Format;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => {
                self.encode_slice_len(val.len())?;
                self.writer().write(val.as_bytes())
            },
            | Format::Cbor | Format::CborDeterministic => cbor::encode_str(self.writer(), val),
        }
    }

    /// Encode the length of a slice.
    fn encode_slice_len(
        &mut self,
        len: usize,
    ) -> Result<(), EncodeError> {
        use crate::config::Format;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => self.encode_u64(len as u64),
            | Format::Cbor | Format::CborDeterministic => {
                cbor::encode_slice_len(self.writer(), len)
            },
        }
    }

    /// Encode the length of an array.
    fn encode_array_len(
        &mut self,
        len: usize,
    ) -> Result<(), EncodeError> {
        self.encode_slice_len(len)
    }

    /// Encode the length of a map.
    fn encode_map_len(
        &mut self,
        len: usize,
    ) -> Result<(), EncodeError> {
        self.encode_slice_len(len)
    }

    /// Encode an enum variant index.
    ///
    /// Variant indices are always encoded as a single `u8` for Bincode format,
    /// matching the decode side which uses `u8::decode()`.
    fn encode_variant_index(
        &mut self,
        idx: u32,
    ) -> Result<(), EncodeError> {
        use crate::config::Format;
        match <Self::C as crate::config::InternalFormatConfig>::FORMAT {
            | Format::Bincode => self.encode_u8(idx as u8),
            | Format::Cbor | Format::CborDeterministic => cbor::encode_u32(self.writer(), idx),
        }
    }
}

impl<T> crate::error_path::BincodeErrorPathCovered<1> for &mut T where
    T: crate::error_path::BincodeErrorPathCovered<1>
{
}

impl<T> Encoder for &mut T
where
    T: Encoder,
{
    type C = T::C;
    type W = T::W;

    #[inline]
    fn writer(&mut self) -> &mut Self::W {
        T::writer(self)
    }

    #[inline]
    fn config(&self) -> &Self::C {
        T::config(self)
    }

    #[inline]
    fn encode_u8(
        &mut self,
        val: u8,
    ) -> Result<(), EncodeError> {
        T::encode_u8(self, val)
    }

    #[inline]
    fn encode_u16(
        &mut self,
        val: u16,
    ) -> Result<(), EncodeError> {
        T::encode_u16(self, val)
    }

    #[inline]
    fn encode_u32(
        &mut self,
        val: u32,
    ) -> Result<(), EncodeError> {
        T::encode_u32(self, val)
    }

    #[inline]
    fn encode_u64(
        &mut self,
        val: u64,
    ) -> Result<(), EncodeError> {
        T::encode_u64(self, val)
    }

    #[inline]
    fn encode_u128(
        &mut self,
        val: u128,
    ) -> Result<(), EncodeError> {
        T::encode_u128(self, val)
    }

    #[inline]
    fn encode_usize(
        &mut self,
        val: usize,
    ) -> Result<(), EncodeError> {
        T::encode_usize(self, val)
    }

    #[inline]
    fn encode_i8(
        &mut self,
        val: i8,
    ) -> Result<(), EncodeError> {
        T::encode_i8(self, val)
    }

    #[inline]
    fn encode_i16(
        &mut self,
        val: i16,
    ) -> Result<(), EncodeError> {
        T::encode_i16(self, val)
    }

    #[inline]
    fn encode_i32(
        &mut self,
        val: i32,
    ) -> Result<(), EncodeError> {
        T::encode_i32(self, val)
    }

    #[inline]
    fn encode_i64(
        &mut self,
        val: i64,
    ) -> Result<(), EncodeError> {
        T::encode_i64(self, val)
    }

    #[inline]
    fn encode_i128(
        &mut self,
        val: i128,
    ) -> Result<(), EncodeError> {
        T::encode_i128(self, val)
    }

    #[inline]
    fn encode_isize(
        &mut self,
        val: isize,
    ) -> Result<(), EncodeError> {
        T::encode_isize(self, val)
    }

    #[inline]
    fn encode_f32(
        &mut self,
        val: f32,
    ) -> Result<(), EncodeError> {
        T::encode_f32(self, val)
    }

    #[inline]
    fn encode_f64(
        &mut self,
        val: f64,
    ) -> Result<(), EncodeError> {
        T::encode_f64(self, val)
    }

    #[inline]
    fn encode_bool(
        &mut self,
        val: bool,
    ) -> Result<(), EncodeError> {
        T::encode_bool(self, val)
    }

    #[inline]
    fn encode_str(
        &mut self,
        val: &str,
    ) -> Result<(), EncodeError> {
        T::encode_str(self, val)
    }

    #[inline]
    fn encode_slice_len(
        &mut self,
        len: usize,
    ) -> Result<(), EncodeError> {
        T::encode_slice_len(self, len)
    }

    #[inline]
    fn encode_array_len(
        &mut self,
        len: usize,
    ) -> Result<(), EncodeError> {
        T::encode_array_len(self, len)
    }

    #[inline]
    fn encode_map_len(
        &mut self,
        len: usize,
    ) -> Result<(), EncodeError> {
        T::encode_map_len(self, len)
    }

    #[inline]
    fn encode_variant_index(
        &mut self,
        idx: u32,
    ) -> Result<(), EncodeError> {
        T::encode_variant_index(self, idx)
    }
}

/// Encode the variant of the given option. Will not encode the option itself.
#[inline]
pub(crate) fn encode_option_variant<E: Encoder, T>(
    encoder: &mut E,
    value: Option<&T>,
) -> Result<(), EncodeError> {
    E::assert_covered();
    match value {
        | None => 0u8.encode(encoder),
        | Some(_) => 1u8.encode(encoder),
    }
}

/// Encodes the length of any slice, container, etc into the given encoder
#[inline]
pub(crate) fn encode_slice_len<E: Encoder>(
    encoder: &mut E,
    len: usize,
) -> Result<(), EncodeError> {
    E::assert_covered();
    encoder.encode_slice_len(len)
}
