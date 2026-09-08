#[cfg(feature = "alloc")]
use crate::alloc::vec::Vec;
use crate::enc::Encode;
use crate::enc::Encoder;
use crate::enc::write::Writer;
use crate::error::EncodeError;

/// Encode a map with deterministic ordering.
///
/// Keys are sorted by the bytewise lexicographic order of their bincode encodings.
///
/// # Errors
///
/// Returns `EncodeError` if the encoding fails.
#[cfg(feature = "alloc")]
#[inline(always)]
pub fn encode_map_deterministic<E, K, V, I>(
    encoder: &mut E,
    iter: I,
) -> Result<(), EncodeError>
where
    E: Encoder,
    K: Encode,
    V: Encode,
    I: IntoIterator<Item = (K, V)>,
    I::IntoIter: ExactSizeIterator,
{
    let iter = iter.into_iter();
    let len = iter.len();
    encoder.encode_map_len(len)?;

    let mut entries = Vec::with_capacity(len);
    let mut all_key_bytes = Vec::new();

    for (k, v) in iter {
        let start = all_key_bytes.len();
        let mut key_encoder =
            crate::enc::EncoderImpl::<_, E::C>::new(&mut all_key_bytes, *encoder.config());
        k.encode(&mut key_encoder)?;
        let end = all_key_bytes.len();
        entries.push((start..end, v));
    }

    entries.sort_by(|(ka, _), (kb, _)| {
        let a = &all_key_bytes[ka.clone()];
        let b = &all_key_bytes[kb.clone()];
        a.cmp(b)
    });

    for (k_range, v) in entries {
        encoder.writer().write(&all_key_bytes[k_range])?;
        v.encode(encoder)?;
    }

    Ok(())
}

/// Encode a slice with deterministic ordering.
///
/// Elements are sorted by the bytewise lexicographic order of their bincode encodings.
///
/// # Errors
///
/// Returns `EncodeError` if the encoding fails.
#[cfg(feature = "alloc")]
#[inline]
pub fn encode_slice_deterministic<E, T, I>(
    encoder: &mut E,
    iter: I,
) -> Result<(), EncodeError>
where
    E: Encoder,
    T: Encode,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    let iter = iter.into_iter();
    let len = iter.len();
    encoder.encode_slice_len(len)?;

    let mut entries = Vec::with_capacity(len);
    let mut all_bytes = Vec::new();

    for item in iter {
        let start = all_bytes.len();
        let mut key_encoder =
            crate::enc::EncoderImpl::<_, E::C>::new(&mut all_bytes, *encoder.config());
        item.encode(&mut key_encoder)?;
        let end = all_bytes.len();
        entries.push(start..end);
    }

    entries.sort_by(|a, b| {
        let a_bytes = &all_bytes[a.clone()];
        let b_bytes = &all_bytes[b.clone()];
        a_bytes.cmp(b_bytes)
    });

    for range in entries {
        encoder.writer().write(&all_bytes[range])?;
    }

    Ok(())
}
