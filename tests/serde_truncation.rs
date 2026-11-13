#![cfg(all(feature = "serde", feature = "alloc"))]

extern crate bincode_next as bincode;

use bincode::config::Config;
use bincode::de::read::SliceReader;
use bincode::error::DecodeError;
use serde::de::DeserializeOwned;

fn assert_truncated<T: DeserializeOwned + core::fmt::Debug, C: Config>(
    bytes: &[u8],
    config: C,
) {
    let result = bincode::serde::decode_from_slice::<T, _>(bytes, config);
    assert!(
        matches!(result, Err(DecodeError::UnexpectedEnd { .. })),
        "{result:?}"
    );

    let result = bincode::serde::borrow_decode_from_slice::<T, _>(bytes, config);
    assert!(
        matches!(result, Err(DecodeError::UnexpectedEnd { .. })),
        "{result:?}"
    );

    let result = bincode::serde::decode_from_reader::<T, _, _>(SliceReader::new(bytes), config);
    assert!(
        matches!(result, Err(DecodeError::UnexpectedEnd { .. })),
        "{result:?}"
    );

    #[cfg(feature = "std")]
    {
        let mut reader = bytes;
        let result = bincode::serde::decode_from_std_read::<T, _, _>(&mut reader, config);
        assert!(
            matches!(result, Err(DecodeError::UnexpectedEnd { .. })),
            "{result:?}"
        );
    }
}

#[test]
fn truncated_byte_sequences_are_rejected() {
    let config = bincode::config::legacy();
    let mut bytes = 5u64.to_le_bytes().to_vec();
    assert_truncated::<Vec<u8>, _>(&bytes, config);
    bytes.extend_from_slice(&[1, 2]);
    assert_truncated::<Vec<u8>, _>(&bytes, config);

    assert_truncated::<Vec<u8>, _>(&[5, 1, 2], bincode::config::standard());
}

#[test]
#[cfg(target_pointer_width = "64")]
fn truncated_sequence_with_huge_length_is_rejected() {
    let bytes = (1u64 << 40).to_le_bytes();
    assert_truncated::<Vec<u8>, _>(&bytes, bincode::config::legacy());
}

#[test]
fn truncated_non_byte_sequences_are_rejected() {
    let config = bincode::config::legacy();
    let mut bytes = 2u64.to_le_bytes().to_vec();
    bytes.extend_from_slice(&1u16.to_le_bytes());
    assert_truncated::<Vec<u16>, _>(&bytes, config);
    // Truncate in the middle of the second element as well.
    bytes.push(2);
    assert_truncated::<Vec<u16>, _>(&bytes, config);

    let bytes = bincode::serde::encode_to_vec(vec![String::from("abc")], config).unwrap();
    assert_truncated::<Vec<String>, _>(&bytes[..bytes.len() - 1], config);
}

#[test]
fn truncated_tuple_elements_preserve_unexpected_end() {
    let config = bincode::config::legacy();
    let mut bytes = 1u64.to_le_bytes().to_vec();
    bytes.push(42);
    assert_truncated::<Vec<(u8, u8)>, _>(&bytes, config);
}

#[test]
fn complete_byte_sequences_decode_with_correct_consumption() {
    fn check<C: Config>(config: C) {
        for expected in [vec![], vec![1u8, 2, 3]] {
            let mut bytes = bincode::serde::encode_to_vec(&expected, config).unwrap();
            let encoded_len = bytes.len();
            bytes.push(99);

            let result = bincode::serde::decode_from_slice::<Vec<u8>, _>(&bytes, config).unwrap();
            assert_eq!(result, (expected.clone(), encoded_len));
            let result =
                bincode::serde::borrow_decode_from_slice::<Vec<u8>, _>(&bytes, config).unwrap();
            assert_eq!(result, (expected.clone(), encoded_len));

            let mut reader = SliceReader::new(&bytes);
            let result =
                bincode::serde::decode_from_reader::<Vec<u8>, _, _>(&mut reader, config).unwrap();
            assert_eq!(result, expected);
            assert_eq!(bincode::de::read::Reader::read_u8(&mut reader).unwrap(), 99);

            #[cfg(feature = "std")]
            {
                let mut reader = bytes.as_slice();
                let result =
                    bincode::serde::decode_from_std_read::<Vec<u8>, _, _>(&mut reader, config)
                        .unwrap();
                assert_eq!(result, expected);
                assert_eq!(reader, &[99]);
            }
        }
    }

    check(bincode::config::legacy());
    check(bincode::config::standard());
}
