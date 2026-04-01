#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(internal_features)]
#![cfg_attr(is_nightly, feature(core_intrinsics))]

//! Bincode-next is a crate for encoding and decoding using a tiny binary
//! serialization strategy.  Using it, you can easily go from having
//! an object in memory, quickly serialize it to bytes, and then
//! deserialize it back just as fast!
//!
//! If you're coming from bincode 1, check out our [migration guide](migration_guide/index.html)
//!
//! # Serde
//!
//! Starting from bincode 2, serde is now an optional dependency. If you want to use serde, please enable the `serde` feature. See [Features](#features) for more information.
//!
//! # Features
//!
//! |Name  |Default?|Affects MSRV?|Supported types for Encode/Decode|Enabled methods                                                  |Other|
//! |------|--------|-------------|-----------------------------------------|-----------------------------------------------------------------|-----|
//! |std   | Yes    | No          |`HashMap` and `HashSet`|`decode_from_std_read` and `encode_into_std_write`|
//! |alloc | Yes    | No          |All common containers in alloc, like `Vec`, `String`, `Box`|`encode_to_vec`|
//! |atomic| Yes    | No          |All `Atomic*` integer types, e.g. `AtomicUsize`, and `AtomicBool`||
//! |derive| Yes    | No          |||Enables the `BorrowDecode`, `Decode`, `Encode`, `Fingerprint` and `BitPacked` derive macros|
//! |serde | No     | Yes (MSRV reliant on serde)|`Compat` and `BorrowCompat`, which will work for all types that implement serde's traits|serde-specific encode/decode functions in the [`serde`\] module|Note: There are several [known issues](serde/index.html#known-issues) when using serde and bincode|
//! |zero-copy| No    | No          |`RelativePtr`, `ZeroArray`, `ZeroSlice`, `ZeroStr`, `ZeroString`|Enables the `relative_ptr` module and the `ZeroCopy` derive macro|Zero-copy nested structures using offsets|
//! |static-size| No    | No          |||Enables the `static_size` module, the `bounded` module and the `StaticSize` derive macro|Compile-time size verification|
//!
//! # Which functions to use
//!
//! Bincode has a couple of pairs of functions that are used in different situations.
//!
//! |Situation|Encode|Decode|
//! |---|---|---
//! |You're working with [`fs::File`\] or [`net::TcpStream`\]|[`encode_into_std_write`\]|[`decode_from_std_read`\]|
//! |you're working with in-memory buffers|[`encode_to_vec`\]|[`decode_from_slice`\]|
//! |You want to use a custom [Reader] and [Writer]|[`encode_into_writer`\]|[`decode_from_reader`\]|
//! |You're working with pre-allocated buffers or on embedded targets|[`encode_into_slice`\]|[`decode_from_slice`\]|
//!
//! **Note:** If you're using `serde`, use `bincode_next::serde::...` instead of `bincode_next::...`
//!
//! # Example
//!
//! ```rust
//! let mut slice = [0u8; 100];
//!
//! // You can encode any type that implements `Encode`.
//! // You can automatically implement this trait on custom types with the `derive` feature.
//! let input = (
//!     0u8,
//!     10u32,
//!     10000i128,
//!     'a',
//!     [0u8, 1u8, 2u8, 3u8]
//! );
//!
//! let length = bincode_next::encode_into_slice(
//!     input,
//!     &mut slice,
//!     bincode_next::config::standard()
//! ).unwrap();
//!
//! let slice = &slice[..length];
//! println!("Bytes written: {:?}", slice);
//!
//! // Decoding works the same as encoding.
//! // The trait used is `Decode`, and can also be automatically implemented with the `derive` feature.
//! let decoded: (u8, u32, i128, char, [u8; 4]) = bincode_next::decode_from_slice(slice, bincode_next::config::standard()).unwrap().0;
//!
//! assert_eq!(decoded, input);
//! ```
//!
//! [`fs::File`\]: `std::fs::File`
//! [`net::TcpStream`\]: `std::net::TcpStream`

// =========================================================================
// RUST LINT CONFIGURATION: bincode-next
// =========================================================================

// -------------------------------------------------------------------------
// LEVEL 1: CRITICAL ERRORS (Deny)
// -------------------------------------------------------------------------
#![deny(
    // Rust Compiler Errors
    unreachable_code,
    improper_ctypes_definitions,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    clippy::perf,
    clippy::correctness,
    clippy::suspicious,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::missing_safety_doc,
    clippy::same_item_push,
    clippy::implicit_clone,
    clippy::all,
    clippy::pedantic,
    missing_docs,
    clippy::nursery,
    clippy::single_call_fn,
)]
// -------------------------------------------------------------------------
// LEVEL 2: STYLE WARNINGS (Warn)
// -------------------------------------------------------------------------
#![warn(
    // For `no-std` Situation Issues
    dead_code,
    warnings,
    unsafe_code,
    clippy::dbg_macro,
    clippy::todo,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::unnecessary_safety_comment
)]
// -------------------------------------------------------------------------
// LEVEL 3: ALLOW/IGNORABLE (Allow)
// -------------------------------------------------------------------------
#![allow(
    clippy::restriction,
    clippy::inline_always,
    unused_doc_comments,
    clippy::empty_line_after_doc_comments
)]
#![crate_name = "bincode_next"]
#![crate_type = "rlib"]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(any(feature = "std", test))]
extern crate std;

mod atomic;
#[doc(hidden)]
pub mod error_path;
mod features;
#[doc(hidden)]
pub mod utils;
pub(crate) mod varint;

use de::Decoder;
use de::read::Reader;
use enc::write::Writer;

#[cfg(any(
    feature = "alloc",
    feature = "std",
    feature = "derive",
    feature = "serde",
    feature = "zero-copy",
    feature = "static-size"
))]
pub use features::*;

/// The major version of the bincode library.
pub const BINCODE_MAJOR_VERSION: u64 = 3;

#[doc(hidden)]
pub use rapidhash;

pub mod config;
/// Fingerprinting support for schema verification.
pub mod fingerprint;

#[macro_use]
pub mod de;
pub mod enc;
pub mod error;

#[cfg(feature = "static-size")]
pub use static_size::StaticSize;

pub use de::BorrowDecode;
pub use de::Decode;
pub use enc::Encode;
pub use fingerprint::Fingerprint;
#[cfg(feature = "zero-copy")]
pub use relative_ptr::ZeroCopy;
#[cfg(feature = "zero-copy")]
pub use relative_ptr::ZeroCopyType;

#[cfg(not(any(feature = "std", feature = "serde")))]
#[cfg(feature = "no-std")]
use panic_halt as _;

use config::Config;
use config::internal::InternalFingerprintGuard;

/// Encode the given value into the given slice. Returns the amount of bytes that have been written.
///
/// See the [config] module for more information on configurations.
///
/// # Errors
///
/// Returns an `EncodeError` if the slice is too small or the value cannot be encoded.
///
/// [config]: config/index.html
#[inline]
pub fn encode_into_slice<E: enc::Encode, C: Config>(
    val: E,
    dst: &mut [u8],
    config: C,
) -> Result<usize, error::EncodeError>
where
    C::Mode: config::InternalFingerprintGuard<E, C>,
{
    let mut writer = enc::write::SliceWriter::new(dst);
    C::Mode::encode_check(&config, &mut writer)?;
    let mut encoder = enc::EncoderImpl::<_, C>::new(writer, config);
    val.encode(&mut encoder)?;
    Ok(encoder.into_writer().bytes_written())
}

/// Encode the given value into a custom [`Writer`\].
///
/// See the [config] module for more information on configurations.
///
/// # Errors
///
/// Returns an `EncodeError` if the writer fails or the value cannot be encoded.
///
/// [config]: config/index.html
#[inline]
pub fn encode_into_writer<E: enc::Encode, W: Writer, C: Config>(
    val: E,
    mut writer: W,
    config: C,
) -> Result<(), error::EncodeError>
where
    C::Mode: config::InternalFingerprintGuard<E, C>,
{
    C::Mode::encode_check(&config, &mut writer)?;
    let mut encoder = enc::EncoderImpl::<_, C>::new(writer, config);
    val.encode(&mut encoder)?;
    Ok(())
}

/// Attempt to decode a given type `D` from the given slice. Returns the decoded output and the amount of bytes read.
///
/// Note that this does not work with borrowed types like `&str` or `&[u8]`. For that use [`borrow_decode_from_slice`\].
///
/// See the [config] module for more information on configurations.
///
/// # Errors
///
/// Returns a `DecodeError` if the slice is too small or the data is invalid.
///
/// [config]: config/index.html
#[inline(always)]
pub fn decode_from_slice<D: de::Decode<()>, C: Config>(
    src: &[u8],
    config: C,
) -> Result<(D, usize), error::DecodeError>
where
    C::Mode: config::InternalFingerprintGuard<D, C>,
{
    decode_from_slice_with_context(src, config, ())
}

/// Attempt to decode a given type `D` from the given slice with `Context`. Returns the decoded output and the amount of bytes read.
///
/// Note that this does not work with borrowed types like `&str` or `&[u8]`. For that use [`borrow_decode_from_slice`\].
///
/// See the [config] module for more information on configurations.
///
/// # Errors
///
/// Returns a `DecodeError` if the slice is too small or the data is invalid.
///
/// [config]: config/index.html
#[inline]
pub fn decode_from_slice_with_context<Context, D: de::Decode<Context>, C: Config>(
    src: &[u8],
    config: C,
    context: Context,
) -> Result<(D, usize), error::DecodeError>
where
    C::Mode: config::InternalFingerprintGuard<D, C>,
{
    let mut reader = de::read::SliceReader::new(src);
    C::Mode::decode_check(&config, &mut reader)?;
    let mut decoder = de::DecoderImpl::<_, C, Context>::new(reader, config, context);
    let result = D::decode(&mut decoder)?;
    let bytes_read = src.len() - decoder.reader().slice.len();
    Ok((result, bytes_read))
}

/// Attempt to decode a given type `D` from the given slice. Returns the decoded output and the amount of bytes read.
///
/// See the [config] module for more information on configurations.
///
/// # Errors
///
/// Returns a `DecodeError` if the slice is too small or the data is invalid.
///
/// [config]: config/index.html
#[inline(always)]
pub fn borrow_decode_from_slice<'a, D: de::BorrowDecode<'a, ()>, C: Config>(
    src: &'a [u8],
    config: C,
) -> Result<(D, usize), error::DecodeError>
where
    C::Mode: config::InternalFingerprintGuard<D, C>,
{
    borrow_decode_from_slice_with_context(src, config, ())
}

/// Attempt to decode a given type `D` from the given slice with `Context`. Returns the decoded output and the amount of bytes read.
///
/// See the [config] module for more information on configurations.
///
/// # Errors
///
/// Returns a `DecodeError` if the slice is too small or the data is invalid.
///
/// [config]: config/index.html
#[inline]
pub fn borrow_decode_from_slice_with_context<
    'a,
    Context,
    D: de::BorrowDecode<'a, Context>,
    C: Config,
>(
    src: &'a [u8],
    config: C,
    context: Context,
) -> Result<(D, usize), error::DecodeError>
where
    C::Mode: config::InternalFingerprintGuard<D, C>,
{
    let mut reader = de::read::SliceReader::new(src);
    C::Mode::decode_check(&config, &mut reader)?;
    let mut decoder = de::DecoderImpl::<_, C, Context>::new(reader, config, context);
    let result = D::borrow_decode(&mut decoder)?;
    let bytes_read = src.len() - decoder.reader().slice.len();
    Ok((result, bytes_read))
}

/// Attempt to decode a given type `D` from the given slice with a compile-time bound check.
///
/// This function ensures that the target type `D` cannot exceed the provided buffer capacity `CAP` at compile-time.
///
/// # Errors
///
/// Returns a `DecodeError` if the slice contains invalid data.
#[cfg(feature = "static-size")]
#[inline(always)]
pub fn decode_from_slice_static<D, const CAP: usize, C>(
    src: &[u8; CAP],
    config: C,
) -> Result<D, error::DecodeError>
where
    D: de::Decode<()> + static_size::StaticSize,
    C: Config,
    C::Mode: config::InternalFingerprintGuard<D, C>,
{
    const {
        assert!(D::MAX_SIZE <= CAP, "Buffer too small for target type");
    }
    let (val, _) = decode_from_slice(src, config)?;
    Ok(val)
}

/// Attempt to decode a given type `D` from the given slice with a compile-time bound check and a
/// decoding context.
///
/// This function ensures that the target type `D` cannot exceed the provided buffer capacity `CAP`
/// at compile-time.
///
/// # Errors
///
/// Returns a `DecodeError` if the slice contains invalid data.
#[cfg(feature = "static-size")]
#[inline(always)]
pub fn decode_from_slice_static_with_context<Context, D, const CAP: usize, C>(
    src: &[u8; CAP],
    config: C,
    context: Context,
) -> Result<D, error::DecodeError>
where
    D: de::Decode<Context> + static_size::StaticSize,
    C: Config,
    C::Mode: config::InternalFingerprintGuard<D, C>,
{
    const {
        assert!(D::MAX_SIZE <= CAP, "Buffer too small for target type");
    }
    let (val, _) = decode_from_slice_with_context(src, config, context)?;
    Ok(val)
}

/// Attempt to decode a given type `D` from the given slice with a compile-time bound check.
///
/// This function ensures that the target type `D` cannot exceed the provided buffer capacity `CAP`
/// at compile-time.
///
/// # Errors
///
/// Returns a `DecodeError` if the slice contains invalid data.
#[cfg(feature = "static-size")]
#[inline(always)]
pub fn borrow_decode_from_slice_static<'a, D, const CAP: usize, C>(
    src: &'a [u8; CAP],
    config: C,
) -> Result<D, error::DecodeError>
where
    D: de::BorrowDecode<'a, ()> + static_size::StaticSize,
    C: Config,
    C::Mode: config::InternalFingerprintGuard<D, C>,
{
    const {
        assert!(D::MAX_SIZE <= CAP, "Buffer too small for target type");
    }
    let (val, _) = borrow_decode_from_slice(src, config)?;
    Ok(val)
}

/// Attempt to borrow-decode a given type `D` from the given slice with a compile-time bound check
/// and a decoding context.
///
/// This function ensures that the target type `D` cannot exceed the provided buffer capacity `CAP`
/// at compile-time.
///
/// # Errors
///
/// Returns a `DecodeError` if the slice contains invalid data.
#[cfg(feature = "static-size")]
#[inline(always)]
pub fn borrow_decode_from_slice_static_with_context<'a, Context, D, const CAP: usize, C>(
    src: &'a [u8; CAP],
    config: C,
    context: Context,
) -> Result<D, error::DecodeError>
where
    D: de::BorrowDecode<'a, Context> + static_size::StaticSize,
    C: Config,
    C::Mode: config::InternalFingerprintGuard<D, C>,
{
    const {
        assert!(D::MAX_SIZE <= CAP, "Buffer too small for target type");
    }
    let (val, _) = borrow_decode_from_slice_with_context(src, config, context)?;
    Ok(val)
}

/// Attempt to decode a given type `D` from the given [`Reader`\].
///
/// See the [config] module for more information on configurations.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[inline]
pub fn decode_from_reader<D: de::Decode<()>, R: Reader, C: Config>(
    mut reader: R,
    config: C,
) -> Result<D, error::DecodeError>
where
    C::Mode: config::InternalFingerprintGuard<D, C>,
{
    C::Mode::decode_check(&config, &mut reader)?;
    let mut decoder = de::DecoderImpl::<_, C, ()>::new(reader, config, ());
    D::decode(&mut decoder)
}

/// Attempt to decode a given type `T` from the given async reader safely using a non-blocking fiber.
///
/// Requires the `async-fiber` feature.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(feature = "async-fiber")]
#[inline(always)]
pub async fn decode_async<T, R, C>(
    config: C,
    reader: R,
) -> Result<T, crate::error::DecodeError>
where
    T: crate::Decode<()>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
    C::Mode: crate::config::InternalFingerprintGuard<T, C>,
{
    decode_async_with_context::<T, R, C, ()>(config, reader, ()).await
}

/// Attempt to decode a given type `T` from the given async reader using a non-blocking fiber and a context.
///
/// This is the primary implementation for runtimes that use the `futures-io` traits (e.g. `async-std`, `smol`).
///
/// Requires the `async-fiber` feature.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(feature = "async-fiber")]
#[inline]
pub async fn decode_async_with_context<T, R, C, Context>(
    config: C,
    reader: R,
    context: Context,
) -> Result<T, crate::error::DecodeError>
where
    T: crate::Decode<Context>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
    C::Mode: crate::config::InternalFingerprintGuard<T, C>,
{
    let bridge = crate::de::async_fiber::AsyncFiberBridge::new(reader);
    bridge
        .run(move |fiber_reader| {
            C::Mode::decode_check(&config, fiber_reader)?;
            let mut decoder =
                crate::de::DecoderImpl::<_, C, Context>::new(fiber_reader, config, context);
            T::decode(&mut decoder)
        })
        .await
}

/// Attempt to decode a given type `T` from an `async-std` reader.
///
/// Requires the `async-fiber` feature.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(feature = "async-fiber")]
#[inline(always)]
pub async fn decode_async_std<T, R, C>(
    config: C,
    reader: R,
) -> Result<T, crate::error::DecodeError>
where
    T: crate::Decode<()>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
    C::Mode: crate::config::InternalFingerprintGuard<T, C>,
{
    decode_async_with_context::<T, R, C, ()>(config, reader, ()).await
}

/// Attempt to decode a given type `T` from an `async-std` reader.
///
/// Requires the `async-fiber` feature.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(feature = "async-fiber")]
#[inline(always)]
pub async fn decode_async_std_with_context<T, R, C, Context>(
    config: C,
    reader: R,
    context: Context,
) -> Result<T, crate::error::DecodeError>
where
    T: crate::Decode<Context>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
    C::Mode: crate::config::InternalFingerprintGuard<T, C>,
{
    decode_async_with_context::<T, R, C, Context>(config, reader, context).await
}

/// Attempt to decode a given type `T` from a `smol` reader.
///
/// Requires the `async-fiber` feature.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(feature = "async-fiber")]
#[inline(always)]
pub async fn decode_async_smol<T, R, C>(
    config: C,
    reader: R,
) -> Result<T, crate::error::DecodeError>
where
    T: crate::Decode<()>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
    C::Mode: crate::config::InternalFingerprintGuard<T, C>,
{
    decode_async_with_context::<T, R, C, ()>(config, reader, ()).await
}

/// Attempt to decode a given type `T` from a `smol` reader.
///
/// Requires the `async-fiber` feature.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(feature = "async-fiber")]
#[inline(always)]
pub async fn decode_async_smol_with_context<T, R, C, Context>(
    config: C,
    reader: R,
    context: Context,
) -> Result<T, crate::error::DecodeError>
where
    T: crate::Decode<Context>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
    C::Mode: crate::config::InternalFingerprintGuard<T, C>,
{
    decode_async_with_context::<T, R, C, Context>(config, reader, context).await
}

/// Attempt to decode a given type `T` from the given tokio async reader safely using a non-blocking fiber.
///
/// Requires the `tokio` and `async-fiber` features.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(all(feature = "tokio", feature = "async-fiber"))]
#[inline(always)]
pub async fn decode_tokio_async<T, R, C>(
    config: C,
    reader: R,
) -> Result<T, crate::error::DecodeError>
where
    T: crate::Decode<()>,
    R: tokio::io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
    C::Mode: crate::config::InternalFingerprintGuard<T, C>,
{
    decode_async_tokio_with_context::<T, R, C, ()>(config, reader, ()).await
}

/// Attempt to decode a given type `T` from the given tokio async reader using a non-blocking fiber and a context.
///
/// Requires the `tokio` and `async-fiber` features.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(all(feature = "tokio", feature = "async-fiber"))]
#[inline(always)]
pub async fn decode_async_tokio_with_context<T, R, C, Context>(
    config: C,
    reader: R,
    context: Context,
) -> Result<T, crate::error::DecodeError>
where
    T: crate::Decode<Context>,
    R: tokio::io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
    C::Mode: crate::config::InternalFingerprintGuard<T, C>,
{
    let reader = crate::de::async_fiber::TokioReader(reader);
    decode_async_with_context::<T, _, C, Context>(config, reader, context).await
}

/// Attempt to decode a given serde-compatible type `T` from the given async reader safely using a non-blocking fiber.
///
/// Requires the `async-fiber` and `serde` features.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(all(feature = "async-fiber", feature = "serde"))]
#[inline(always)]
pub async fn decode_serde_async<'de, T, R, C>(
    config: C,
    reader: R,
) -> Result<T, crate::error::DecodeError>
where
    T: ::serde::Deserialize<'de>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
{
    decode_serde_async_with_context::<'de, T, R, C, ()>(config, reader, ()).await
}

/// Attempt to decode a given serde-compatible type `T` from an `async-std` reader.
///
/// Requires the `async-fiber` and `serde` features.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(all(feature = "async-fiber", feature = "serde"))]
#[inline(always)]
pub async fn decode_serde_async_std<'de, T, R, C>(
    config: C,
    reader: R,
) -> Result<T, crate::error::DecodeError>
where
    T: ::serde::Deserialize<'de>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
{
    decode_serde_async_with_context::<'de, T, R, C, ()>(config, reader, ()).await
}

/// Attempt to decode a given serde-compatible type `T` from an `async-std` reader.
///
/// Requires the `async-fiber` and `serde` features.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(all(feature = "async-fiber", feature = "serde"))]
#[inline(always)]
pub async fn decode_serde_async_std_with_context<'de, T, R, C, Context>(
    config: C,
    reader: R,
    context: Context,
) -> Result<T, crate::error::DecodeError>
where
    T: ::serde::Deserialize<'de>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
{
    decode_serde_async_with_context::<'de, T, R, C, Context>(config, reader, context).await
}

/// Attempt to decode a given serde-compatible type `T` from a `smol` reader.
///
/// Requires the `async-fiber` and `serde` features.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(all(feature = "async-fiber", feature = "serde"))]
#[inline(always)]
pub async fn decode_serde_async_smol<'de, T, R, C>(
    config: C,
    reader: R,
) -> Result<T, crate::error::DecodeError>
where
    T: ::serde::Deserialize<'de>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
{
    decode_serde_async_with_context::<'de, T, R, C, ()>(config, reader, ()).await
}

/// Attempt to decode a given serde-compatible type `T` from a `smol` reader.
///
/// Requires the `async-fiber` and `serde` features.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(all(feature = "async-fiber", feature = "serde"))]
#[inline(always)]
pub async fn decode_serde_async_smol_with_context<'de, T, R, C, Context>(
    config: C,
    reader: R,
    context: Context,
) -> Result<T, crate::error::DecodeError>
where
    T: ::serde::Deserialize<'de>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
{
    decode_serde_async_with_context::<'de, T, R, C, Context>(config, reader, context).await
}

/// Attempt to decode a given serde-compatible type `T` from the given async reader using a non-blocking fiber and a context.
///
/// Requires the `async-fiber` and `serde` features.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(all(feature = "async-fiber", feature = "serde"))]
#[inline]
pub async fn decode_serde_async_with_context<'de, T, R, C, Context>(
    config: C,
    reader: R,
    context: Context,
) -> Result<T, crate::error::DecodeError>
where
    T: ::serde::Deserialize<'de>,
    R: futures_io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
{
    let bridge = crate::de::async_fiber::AsyncFiberBridge::new(reader);
    bridge
        .run(move |fiber_reader| {
            let decoder =
                crate::de::DecoderImpl::<_, C, Context>::new(fiber_reader, config, context);
            let mut serde_decoder = crate::features::serde::OwnedSerdeDecoder { de: decoder };
            T::deserialize(serde_decoder.as_deserializer())
        })
        .await
}

/// Attempt to decode a given serde-compatible type `T` from the given tokio async reader safely using a non-blocking fiber.
///
/// Requires the `tokio`, `async-fiber` and `serde` features.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(all(feature = "tokio", feature = "async-fiber", feature = "serde"))]
#[inline(always)]
pub async fn decode_serde_tokio_async<'de, T, R, C>(
    config: C,
    reader: R,
) -> Result<T, crate::error::DecodeError>
where
    T: ::serde::Deserialize<'de>,
    R: tokio::io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
{
    decode_serde_tokio_async_with_context::<'de, T, R, C, ()>(config, reader, ()).await
}

/// Attempt to decode a given serde-compatible type `T` from the given tokio async reader using a non-blocking fiber and a context.
///
/// Requires the `tokio`, `async-fiber` and `serde` features.
///
/// # Errors
///
/// Returns a `DecodeError` if the reader fails or the data is invalid.
///
/// [config]: config/index.html
#[cfg(all(feature = "tokio", feature = "async-fiber", feature = "serde"))]
#[inline(always)]
pub async fn decode_serde_tokio_async_with_context<'de, T, R, C, Context>(
    config: C,
    reader: R,
    context: Context,
) -> Result<T, crate::error::DecodeError>
where
    T: ::serde::Deserialize<'de>,
    R: tokio::io::AsyncRead + std::marker::Unpin,
    C: crate::config::Config,
{
    let reader = crate::de::async_fiber::TokioReader(reader);
    decode_serde_async_with_context::<'de, T, _, C, Context>(config, reader, context).await
}


#[cfg(all(feature = "alloc", feature = "derive", doc))]
pub mod spec {
    #![doc = include_str!("../docs/spec.md")]
}

#[cfg(doc)]
pub mod migration_guide {
    #![doc = include_str!("../docs/migration_guide.md")]
}

// Test the examples in readme.md
#[cfg(all(
    feature = "std",
    feature = "derive",
    feature = "serde",
    feature = "async-fiber",
    doctest
))]
#[cfg_attr(miri, ignore)]
mod readme {
    #![doc = include_str!("../README.md")]
}
