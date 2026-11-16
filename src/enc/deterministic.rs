#[cfg(feature = "alloc")]
use crate::alloc::vec::Vec;
use crate::enc::Encode;
use crate::enc::Encoder;
use crate::enc::write::Writer;
use crate::error::EncodeError;

/// Compares two encoded entries according to the configured
/// [`BincodeDeterministicMode`](crate::config::BincodeDeterministicMode).
#[cfg(feature = "alloc")]
#[inline(always)]
fn compare_encoded<E: Encoder>(
    a: &[u8],
    b: &[u8],
) -> core::cmp::Ordering {
    use crate::config::BincodeDeterministicMode;
    match <E::C as crate::config::InternalFormatConfig>::BINCODE_OPTIONS.deterministic_mode {
        | BincodeDeterministicMode::LengthFirst => a.len().cmp(&b.len()).then_with(|| a.cmp(b)),
        | _ => a.cmp(b),
    }
}

/// Encode a map with deterministic ordering.
///
/// Keys are sorted by the bytewise lexicographic order of their bincode encodings
/// (or length-first, see [`BincodeDeterministicMode`](crate::config::BincodeDeterministicMode)).
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

    let mut buffer = Vec::new();
    let mut entries = Vec::with_capacity(len);

    for (k, v) in iter {
        let start = buffer.len();
        let mut key_encoder =
            crate::enc::EncoderImpl::<_, E::C>::new(&mut buffer, *encoder.config());
        k.encode(&mut key_encoder)?;
        entries.push((start..buffer.len(), v));
    }

    entries.sort_unstable_by(|(a, _), (b, _)| {
        compare_encoded::<E>(&buffer[a.clone()], &buffer[b.clone()])
    });

    for (range, v) in entries {
        encoder.writer().write(&buffer[range])?;
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

    let mut buffer = Vec::new();
    let mut indices = Vec::with_capacity(len);

    for item in iter {
        let start = buffer.len();
        let mut key_encoder =
            crate::enc::EncoderImpl::<_, E::C>::new(&mut buffer, *encoder.config());
        item.encode(&mut key_encoder)?;
        indices.push(start..buffer.len());
    }

    indices.sort_unstable_by(|a, b| compare_encoded::<E>(&buffer[a.clone()], &buffer[b.clone()]));

    for range in indices {
        encoder.writer().write(&buffer[range])?;
    }

    Ok(())
}
