## 2023-10-24 - Unaligned reads for numbers in BufReader
**Learning:** `std::io::BufReader<R>` doesn't implement `read_u16`/`read_u32` etc., falling back to `read` which copies bytes array by array. `SliceReader` overrides `read_u16` etc. to use `core::ptr::read_unaligned`, which is much faster.
**Action:** Implement `read_u16`, `read_u32`, `read_u64`, `read_u128` on `IoReader` and `BufReader` where possible by checking the buffer, similar to `SliceReader`.
