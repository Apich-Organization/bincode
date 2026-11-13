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
