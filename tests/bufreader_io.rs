#![cfg(feature = "std")]
use std::io::{BufReader, Read};

struct Failing;
impl Read for Failing {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("boom"))
    }
}

#[test]
fn bufreader_propagates_io_error_on_primitive_read() {
    let r = BufReader::new(Failing);
    let res: Result<u32, _> = bincode_next::decode_from_reader(r, bincode_next::config::legacy());
    assert!(matches!(res, Err(bincode_next::error::DecodeError::Io { .. })), "{res:?}");
}

#[test]
fn bufreader_reads_primitives() {
    let data: Vec<u8> = (1u8..=40).collect();
    let r = BufReader::with_capacity(4, &data[..]);
    let res: Result<(u8, u16, u32, u64, u128), _> =
        bincode_next::decode_from_reader(r, bincode_next::config::legacy());
    assert!(res.is_ok(), "{res:?}");
}
