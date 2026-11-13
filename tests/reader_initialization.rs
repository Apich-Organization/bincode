extern crate bincode_next as bincode;

use bincode::BorrowDecode;
use bincode::Decode;
use bincode::config::Config;
use bincode::config::FingerprintDisabled;
use bincode::de::DecoderImpl;
use bincode::de::read::BorrowReader;
use bincode::de::read::Reader;
use bincode::error::DecodeError;

struct InspectingReader<'a> {
    data: &'a [u8],
}

fn inspect(bytes: &[u8]) {
    // Safe readers may inspect every byte of their destination before writing to it.
    // Miri must be able to evaluate this checksum without reading uninitialized memory.
    std::hint::black_box(bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)));
}

impl Reader for InspectingReader<'_> {
    fn read(
        &mut self,
        bytes: &mut [u8],
    ) -> Result<(), DecodeError> {
        inspect(bytes);
        if bytes.len() > self.data.len() {
            return Err(DecodeError::UnexpectedEnd {
                additional: bytes.len() - self.data.len(),
            });
        }
        let (source, rest) = self.data.split_at(bytes.len());
        bytes.copy_from_slice(source);
        self.data = rest;
        Ok(())
    }

    fn peek_read(
        &mut self,
        len: usize,
    ) -> Option<&[u8]> {
        // Support CBOR chunk headers and break markers, but force bulk reads through read().
        if len == 1 {
            self.data.get(..len)
        } else {
            None
        }
    }

    fn consume(
        &mut self,
        len: usize,
    ) {
        self.data = &self.data[len..];
    }
}

impl<'de> BorrowReader<'de> for InspectingReader<'de> {
    fn take_bytes(
        &mut self,
        len: usize,
    ) -> Result<&'de [u8], DecodeError> {
        if len > self.data.len() {
            return Err(DecodeError::UnexpectedEnd {
                additional: len - self.data.len(),
            });
        }
        let (source, rest) = self.data.split_at(len);
        self.data = rest;
        Ok(source)
    }
}

#[cfg(feature = "std")]
impl std::io::Read for InspectingReader<'_> {
    fn read(
        &mut self,
        bytes: &mut [u8],
    ) -> std::io::Result<usize> {
        inspect(bytes);
        // Exercise read_exact's handling of partial reads as well.
        let len = bytes.len().min(self.data.len()).min(2);
        bytes[..len].copy_from_slice(&self.data[..len]);
        self.data = &self.data[len..];
        Ok(len)
    }
}

fn assert_reader<T, C>(
    bytes: &[u8],
    expected: &T,
    config: C,
) where
    T: Decode<()> + for<'de> BorrowDecode<'de, ()> + PartialEq + core::fmt::Debug,
    C: Config<Mode = FingerprintDisabled>,
{
    let decoded =
        bincode::decode_from_reader::<T, _, _>(InspectingReader { data: bytes }, config).unwrap();
    assert_eq!(&decoded, expected);

    let mut decoder = DecoderImpl::new(InspectingReader { data: bytes }, config, ());
    assert_eq!(&T::borrow_decode(&mut decoder).unwrap(), expected);
}

#[cfg(feature = "alloc")]
fn assert_roundtrip<T, C>(
    expected: T,
    config: C,
) where
    T: bincode::Encode + Decode<()> + for<'de> BorrowDecode<'de, ()> + PartialEq + core::fmt::Debug,
    C: Config<Mode = FingerprintDisabled>,
{
    let bytes = bincode::encode_to_vec(&expected, config).unwrap();
    assert_reader(&bytes, &expected, config);

    // Buffered readers must still copy the right bytes and consume exactly the encoded value.
    let (decoded, used) = bincode::decode_from_slice::<T, _>(&bytes, config).unwrap();
    assert_eq!(decoded, expected);
    assert_eq!(used, bytes.len());
    let (decoded, used) = bincode::borrow_decode_from_slice::<T, _>(&bytes, config).unwrap();
    assert_eq!(decoded, expected);
    assert_eq!(used, bytes.len());

    #[cfg(feature = "std")]
    {
        let mut reader = InspectingReader { data: &bytes };
        assert_eq!(
            bincode::decode_from_std_read::<T, _, _>(&mut reader, config).unwrap(),
            expected
        );
        assert!(reader.data.is_empty());
    }
}

#[test]
fn byte_arrays_allow_readers_to_inspect_destinations() {
    assert_reader(&[1, 2, 3, 4], &[1u8, 2, 3, 4], bincode::config::standard());
    assert_reader(&[1, 2, 3, 4], &[1i8, 2, 3, 4], bincode::config::legacy());
    assert_reader(&[], &[0u8; 0], bincode::config::standard());
}

#[test]
#[cfg(feature = "alloc")]
fn vectors_and_strings_allow_readers_to_inspect_destinations() {
    fn check<C: Config<Mode = FingerprintDisabled>>(config: C) {
        assert_roundtrip(vec![1u8, 2, 3, 4], config);
        assert_roundtrip(vec![-1i8, 2, 3, 4], config);
        assert_roundtrip(String::from("hello"), config);
        assert_roundtrip(Vec::<u8>::new(), config);
        assert_roundtrip(String::new(), config);
    }
    check(bincode::config::standard());
    check(bincode::config::legacy());
    check(bincode::config::standard().with_cbor_format());
}

#[test]
#[cfg(feature = "alloc")]
fn fixed_width_numeric_containers_allow_inspecting_readers() {
    fn check<C: Config<Mode = FingerprintDisabled>>(config: C) {
        assert_roundtrip(vec![1u16, 258], config);
        assert_roundtrip(vec![-1i64, 258], config);
        assert_roundtrip(vec![1.5f32, -2.0], config);
        assert_roundtrip([1u32, 258], config);
        assert_roundtrip([-1i128, 258], config);
        assert_roundtrip([1.5f64, -2.0], config);
    }
    check(bincode::config::legacy());
    check(bincode::config::legacy().with_big_endian());
}

#[test]
fn cbor_byte_arrays_allow_inspecting_readers() {
    let config = bincode::config::standard().with_cbor_format();
    assert_reader(&[0x44, 1, 2, 3, 4], &[1u8, 2, 3, 4], config);
}

#[test]
#[cfg(feature = "alloc")]
fn cbor_indefinite_byte_strings_initialize_each_chunk() {
    let config = bincode::config::standard().with_cbor_format();
    let bytes = [0x5f, 0x44, 1, 2, 3, 4, 0x40, 0x44, 5, 6, 7, 8, 0xff];
    assert_reader(&bytes, &vec![1u8, 2, 3, 4, 5, 6, 7, 8], config);
    assert_reader(&bytes, &vec![1i8, 2, 3, 4, 5, 6, 7, 8], config);
}

struct NoWriteReader {
    length: Option<u8>,
}

impl Reader for NoWriteReader {
    fn read(
        &mut self,
        bytes: &mut [u8],
    ) -> Result<(), DecodeError> {
        if let Some(length) = self.length.take() {
            bytes[0] = length;
        }
        // Violates the read contract, but safe code still must not cause undefined behavior.
        Ok(())
    }
}

#[test]
fn readers_that_do_not_write_cannot_leave_uninitialized_arrays() {
    let result = bincode::decode_from_reader::<[u8; 4], _, _>(
        NoWriteReader { length: None },
        bincode::config::standard(),
    )
    .unwrap();
    assert_eq!(result, [0; 4]);
}

#[test]
#[cfg(feature = "alloc")]
fn readers_that_do_not_write_cannot_leave_uninitialized_vectors_or_strings() {
    let config = bincode::config::standard();
    let result =
        bincode::decode_from_reader::<Vec<u8>, _, _>(NoWriteReader { length: Some(4) }, config)
            .unwrap();
    assert_eq!(result, [0; 4]);
    let result =
        bincode::decode_from_reader::<String, _, _>(NoWriteReader { length: Some(4) }, config)
            .unwrap();
    assert_eq!(result.as_bytes(), [0; 4]);
}

#[test]
#[cfg(feature = "alloc")]
fn inspecting_readers_preserve_truncation_errors() {
    fn check<T: Decode<()> + for<'de> BorrowDecode<'de, ()>>(bytes: &[u8]) {
        let config = bincode::config::standard();
        let result =
            bincode::decode_from_reader::<T, _, _>(InspectingReader { data: bytes }, config);
        assert!(matches!(result, Err(DecodeError::UnexpectedEnd { .. })));
        let mut decoder = DecoderImpl::new(InspectingReader { data: bytes }, config, ());
        assert!(matches!(
            T::borrow_decode(&mut decoder),
            Err(DecodeError::UnexpectedEnd { .. })
        ));
    }
    check::<Vec<u8>>(&[4, 1, 2]);
    check::<String>(&[4, b'h', b'i']);
    check::<[u8; 4]>(&[1, 2]);
}

#[test]
fn short_peek_results_cannot_cause_out_of_bounds_copies() {
    struct ShortPeekReader;
    impl Reader for ShortPeekReader {
        fn read(
            &mut self,
            bytes: &mut [u8],
        ) -> Result<(), DecodeError> {
            bytes.fill(0);
            Ok(())
        }

        fn peek_read(
            &mut self,
            _len: usize,
        ) -> Option<&[u8]> {
            // This violates the peek contract, but the implementation is entirely safe.
            Some(&[])
        }
    }

    let result =
        bincode::decode_from_reader::<[u8; 4], _, _>(ShortPeekReader, bincode::config::standard());
    assert!(matches!(
        result,
        Err(DecodeError::UnexpectedEnd { additional: 4 })
    ));
}
