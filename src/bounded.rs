//! Bounded types for compile-time size guarantees.

use crate::de::{BorrowDecode, BorrowDecoder, Decode, Decoder};
use crate::enc::{Encode, Encoder};
use crate::error::{DecodeError, EncodeError};
use crate::static_size::StaticSize;
use crate::static_size::helpers::VARINT_MAX_64;
use alloc::string::String;
use alloc::vec::Vec;

/// A `Vec` wrapper with a compile-time capacity limit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundedVec<T, const CAP: usize>(pub Vec<T>);

impl<T, const CAP: usize> StaticSize for BoundedVec<T, CAP>
where
    T: StaticSize,
{
    const MAX_SIZE: usize = VARINT_MAX_64 + T::MAX_SIZE * CAP;
}

impl<T: Encode, const CAP: usize> Encode for BoundedVec<T, CAP> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        if self.0.len() > CAP {
            return Err(EncodeError::Other("BoundedVec exceeds capacity"));
        }
        self.0.encode(encoder)
    }
}

impl<Context, T: Decode<Context>, const CAP: usize> Decode<Context> for BoundedVec<T, CAP> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let vec: Vec<T> = Vec::decode(decoder)?;
        if vec.len() > CAP {
            return Err(DecodeError::Other("BoundedVec exceeds capacity"));
        }
        Ok(BoundedVec(vec))
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>, const CAP: usize> BorrowDecode<'de, Context>
    for BoundedVec<T, CAP>
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let vec: Vec<T> = Vec::borrow_decode(decoder)?;
        if vec.len() > CAP {
            return Err(DecodeError::Other("BoundedVec exceeds capacity"));
        }
        Ok(BoundedVec(vec))
    }
}

/// A `String` wrapper with a compile-time capacity limit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundedString<const CAP: usize>(pub String);

impl<const CAP: usize> StaticSize for BoundedString<CAP> {
    const MAX_SIZE: usize = VARINT_MAX_64 + CAP;
}

impl<const CAP: usize> Encode for BoundedString<CAP> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        if self.0.len() > CAP {
            return Err(EncodeError::Other("BoundedString exceeds capacity"));
        }
        self.0.encode(encoder)
    }
}

impl<Context, const CAP: usize> Decode<Context> for BoundedString<CAP> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let s: String = String::decode(decoder)?;
        if s.len() > CAP {
            return Err(DecodeError::Other("BoundedString exceeds capacity"));
        }
        Ok(BoundedString(s))
    }
}

impl<'de, Context, const CAP: usize> BorrowDecode<'de, Context> for BoundedString<CAP> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let s: String = String::borrow_decode(decoder)?;
        if s.len() > CAP {
            return Err(DecodeError::Other("BoundedString exceeds capacity"));
        }
        Ok(BoundedString(s))
    }
}

impl<T, const CAP: usize> From<Vec<T>> for BoundedVec<T, CAP> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

impl<const CAP: usize> From<String> for BoundedString<CAP> {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl<T, const CAP: usize> core::ops::Deref for BoundedVec<T, CAP> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const CAP: usize> core::ops::Deref for BoundedString<CAP> {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
