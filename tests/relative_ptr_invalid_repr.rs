#![cfg(feature = "zero-copy")]
use bincode_next::RelativePtr;
use bincode_next::relative_ptr::DeepValidator;

#[repr(align(4))]
struct Aligned([u8; 8]);

fn root<T>(b: &[u8]) -> &RelativePtr<T, 1> {
    unsafe { &*(b.as_ptr() as *const RelativePtr<T, 1>) }
}

#[test]
fn invalid_bool_rejected() {
    let blob = Aligned([4, 0, 0, 0, 2, 0, 0, 0]);
    let p = root::<bool>(&blob.0);
    assert!(p.get(&blob.0).is_none());
    assert!(!p.is_valid_deep(&blob.0));
}

#[test]
fn valid_bool_accepted() {
    for v in [0u8, 1] {
        let blob = Aligned([4, 0, 0, 0, v, 0, 0, 0]);
        assert_eq!(root::<bool>(&blob.0).get(&blob.0), Some(&(v == 1)));
    }
}

#[test]
fn invalid_char_rejected() {
    let mut raw = [4u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    raw[4..8].copy_from_slice(&0xD800u32.to_ne_bytes());
    #[repr(align(4))]
    struct A12([u8; 12]);
    let blob = A12(raw);
    let p = unsafe { &*(blob.0.as_ptr() as *const RelativePtr<char, 0>) };
    assert!(p.get(&blob.0).is_none());
    assert!(!p.is_valid_deep(&blob.0));
}

#[cfg(feature = "derive")]
#[derive(bincode_derive_next::ZeroCopy, Debug, PartialEq, Eq)]
#[repr(C)]
struct Flags {
    id: u32,
    on: bool,
    pad: [u8; 3],
}

#[cfg(feature = "derive")]
#[derive(bincode_derive_next::ZeroCopy, Debug, PartialEq, Eq)]
#[repr(C, u8)]
enum Packet {
    Ping,
    Error(u32),
}

#[cfg(feature = "derive")]
#[derive(bincode_derive_next::ZeroCopy, Debug, PartialEq, Eq)]
#[repr(C, u8)]
enum PacketWithBool {
    Ping,
    Status(bool),
}

#[cfg(feature = "derive")]
#[derive(bincode_derive_next::ZeroCopy, Debug, PartialEq, Eq)]
#[repr(C)]
struct TupleFlags(u32, bool);

#[test]
#[cfg(feature = "derive")]
fn derived_struct_invalid_bool_rejected() {
    #[repr(align(4))]
    struct A16([u8; 16]);
    // offset 4 -> Flags { id: 1, on: 2, pad: [0; 3] }
    let blob = A16([4, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0]);
    let b = &blob.0;
    let ptr = unsafe { &*(b.as_ptr() as *const RelativePtr<Flags, 0>) };
    assert!(ptr.get(b).is_none());
    assert!(!ptr.is_valid_deep(b));
}

#[test]
#[cfg(feature = "derive")]
fn derived_struct_valid_bool_accepted() {
    #[repr(align(4))]
    struct A16([u8; 16]);
    // offset 4 -> Flags { id: 1, on: 1, pad: [0; 3] }
    let blob = A16([4, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
    let b = &blob.0;
    let ptr = unsafe { &*(b.as_ptr() as *const RelativePtr<Flags, 0>) };
    assert!(ptr.get(b).is_some());
    assert!(ptr.is_valid_deep(b));
    let flags = ptr.get(b).unwrap();
    assert_eq!(flags.id, 1);
    assert!(flags.on);
}

#[test]
#[cfg(feature = "derive")]
fn derived_enum_invalid_tag_rejected() {
    #[repr(align(4))]
    struct A12([u8; 12]);
    // offset 4 -> tag byte 7 (only 0 and 1 exist)
    let blob = A12([4, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0]);
    let b = &blob.0;
    let ptr = unsafe { &*(b.as_ptr() as *const RelativePtr<Packet, 0>) };
    assert!(ptr.get(b).is_none());
    assert!(!ptr.is_valid_deep(b));
}

#[test]
#[cfg(feature = "derive")]
fn derived_enum_valid_tag_accepted() {
    #[repr(align(4))]
    struct A12([u8; 12]);
    // offset 4 -> Ping (tag 0)
    let blob = A12([4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    let b = &blob.0;
    let ptr = unsafe { &*(b.as_ptr() as *const RelativePtr<Packet, 0>) };
    assert_eq!(ptr.get(b), Some(&Packet::Ping));
    assert!(ptr.is_valid_deep(b));

    // offset 4 -> Error(42) (tag 1, u32 at payload offset 4)
    let mut raw = [4u8, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0];
    raw[8..12].copy_from_slice(&42u32.to_ne_bytes());
    let blob = A12(raw);
    let b = &blob.0;
    let ptr = unsafe { &*(b.as_ptr() as *const RelativePtr<Packet, 0>) };
    assert_eq!(ptr.get(b), Some(&Packet::Error(42)));
    assert!(ptr.is_valid_deep(b));
}

#[test]
#[cfg(feature = "derive")]
fn derived_enum_invalid_variant_payload_rejected() {
    #[repr(align(4))]
    struct A8([u8; 8]);
    // offset 4 -> tag 1 (Status), bool byte 2 at payload
    let blob = A8([4, 0, 0, 0, 1, 2, 0, 0]);
    let b = &blob.0;
    let ptr = unsafe { &*(b.as_ptr() as *const RelativePtr<PacketWithBool, 0>) };
    assert!(ptr.get(b).is_none());
    assert!(!ptr.is_valid_deep(b));
}

#[test]
#[cfg(feature = "derive")]
fn derived_tuple_struct_invalid_bool_rejected() {
    #[repr(align(4))]
    struct A12([u8; 12]);
    // offset 4 -> TupleFlags(100, 2)
    let mut raw = [4u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    raw[4..8].copy_from_slice(&100u32.to_ne_bytes());
    raw[8] = 2; // invalid bool
    let blob = A12(raw);
    let b = &blob.0;
    let ptr = unsafe { &*(b.as_ptr() as *const RelativePtr<TupleFlags, 0>) };
    assert!(ptr.get(b).is_none());
    assert!(!ptr.is_valid_deep(b));
}

#[test]
#[cfg(all(feature = "derive", feature = "alloc"))]
fn derived_zeroslice_and_zeroarray_reject_invalid_repr() {
    use bincode_next::ZeroArray;
    use bincode_next::ZeroSlice;

    #[repr(align(4))]
    struct Aligned32([u8; 32]);

    // ZeroSlice with 2 elements of Flags, second element has on = 2
    let mut raw = [0u8; 32];
    // len = 2, offset = 4 (from raw[4] to raw[8])
    raw[0..4].copy_from_slice(&2u32.to_ne_bytes());
    raw[4..8].copy_from_slice(&4i32.to_ne_bytes());
    // element 0 (Flags at 8..20): id = 1, on = 1
    raw[8..12].copy_from_slice(&1u32.to_ne_bytes());
    raw[12] = 1;
    // element 1 (Flags at 20..32): id = 2, on = 2 (INVALID)
    raw[20..24].copy_from_slice(&2u32.to_ne_bytes());
    raw[24] = 2;

    let slice_buf = Aligned32(raw);
    let slice_ptr = unsafe { &*(slice_buf.0.as_ptr() as *const ZeroSlice<Flags, 0>) };
    assert!(slice_ptr.get(&slice_buf.0).is_none());
    assert!(!slice_ptr.is_valid_deep(&slice_buf.0));

    // ZeroArray [Flags; 2]
    let mut arr_raw = [0u8; 32];
    arr_raw[0..4].copy_from_slice(&4i32.to_ne_bytes()); // offset to arr_raw[4]
    // element 0: id = 1, on = 1
    arr_raw[4..8].copy_from_slice(&1u32.to_ne_bytes());
    arr_raw[8] = 1;
    // element 1: id = 2, on = 2 (INVALID)
    arr_raw[16..20].copy_from_slice(&2u32.to_ne_bytes());
    arr_raw[20] = 2;

    let arr = Aligned32(arr_raw);
    let arr_ptr = unsafe { &*(arr.0.as_ptr() as *const ZeroArray<Flags, 2, 0>) };
    assert!(arr_ptr.get(&arr.0).is_none());
}
