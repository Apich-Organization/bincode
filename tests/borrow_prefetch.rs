use bincode_next::config;

#[test]
fn borrow_decode_vec_u64_short_input() {
    let bytes: Vec<u8> = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let res: Result<(Vec<u64>, usize), _> =
        bincode_next::borrow_decode_from_slice(&bytes, config::standard());
    let (v, _n) = res.expect("decode");
    assert_eq!(v, vec![0u64]);
}
