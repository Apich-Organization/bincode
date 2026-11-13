#![cfg(feature = "serde")]
extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use serde::Deserialize;
use serde::Serialize;

extern crate bincode_next as bincode;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct V1 {
    pub a: u32,
    pub b: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct V2 {
    pub a: u32,
    pub b: String,
    #[serde(default)]
    pub c: u32, // Added later
    #[serde(default)]
    pub d: Option<u64>, // Added later
}

#[test]
fn test_issue_179_missing_fields_are_rejected() {
    let v1 = V1 {
        a: 42,
        b: "hello".to_string(),
    };

    let encoded_v1 = bincode::serde::encode_to_vec(&v1, bincode::config::standard()).unwrap();

    // Without field metadata, missing trailing fields cannot be distinguished from truncation,
    // even when they have serde defaults.
    let result =
        bincode::serde::decode_from_slice::<V2, _>(&encoded_v1, bincode::config::standard());
    assert!(matches!(
        result,
        Err(bincode::error::DecodeError::UnexpectedEnd { .. })
    ));

    let result = bincode::serde::decode_from_reader::<V2, _, _>(
        bincode::de::read::SliceReader::new(&encoded_v1),
        bincode::config::standard(),
    );
    assert!(matches!(
        result,
        Err(bincode::error::DecodeError::UnexpectedEnd { .. })
    ));
}
