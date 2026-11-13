#![cfg(all(feature = "zero-copy", feature = "derive"))]
use bincode_next::ZeroBuilder;
use bincode_next::ZeroCopyBuilder;

#[derive(bincode_derive_next::ZeroCopy)]
#[repr(C)]
struct Padded {
    tag: u8,
    value: u64,
}

#[derive(bincode_derive_next::ZeroCopy)]
#[repr(C, u8)]
enum PaddedEnum {
    A(u32),
    B { x: u64, y: u8 },
}

#[inline(never)]
fn dirty_stack() -> u8 {
    let buf = [0xA5u8; 4096];
    std::hint::black_box(&buf);
    buf[17]
}

#[test]
fn struct_padding_is_zeroed() {
    dirty_stack();
    let mut b = ZeroBuilder::new();
    PaddedBuilder {
        tag: 11,
        value: 0x0102030405060708,
    }
    .build(&mut b);
    let out = b.finish();
    let bytes: &[u8] = out.as_ref();
    assert_eq!(bytes[0], 11);
    assert!(bytes[1..8].iter().all(|&x| x == 0), "{bytes:02x?}");
    assert_eq!(&bytes[8..16], &0x0102030405060708u64.to_ne_bytes());
}

#[test]
fn enum_padding_is_zeroed() {
    dirty_stack();
    let mut b = ZeroBuilder::new();
    PaddedEnumBuilder::A(7).build(&mut b);
    let bytes: Vec<u8> = b.finish().as_ref().to_vec();
    // tag at 0, payload at 8; everything else must be zero
    assert_eq!(bytes[0], 0);
    assert!(bytes[1..8].iter().all(|&x| x == 0), "{bytes:02x?}");
    assert_eq!(&bytes[8..12], &7u32.to_ne_bytes());
    assert!(bytes[12..].iter().all(|&x| x == 0), "{bytes:02x?}");

    let mut b = ZeroBuilder::new();
    PaddedEnumBuilder::B { x: 5, y: 9 }.build(&mut b);
    let bytes: Vec<u8> = b.finish().as_ref().to_vec();
    assert_eq!(bytes[0], 1);
    assert!(bytes[1..8].iter().all(|&x| x == 0), "{bytes:02x?}");
    assert_eq!(bytes[16], 9);
    assert!(bytes[17..].iter().all(|&x| x == 0), "{bytes:02x?}");
}
