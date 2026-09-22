## 2024-05-24 - Custom BufReader implementation
**Learning:** `std::io::BufReader<R>` defaults to slow byte-by-byte reads for integers. Implementing custom `read_u*` overrides that use `peek_read` and `core::ptr::read_unaligned` significantly speeds up varint decoding.
**Action:** Apply `read_u16`, `read_u32`, `read_u64`, `read_u128` overrides in `src/features/impl_std.rs` for `BufReader<R>`.
