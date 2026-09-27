## 2024-05-24 - Custom read_u* overrides for BufReader
**Learning:** `std::io::BufReader<R>` defaults to slow byte-by-byte reads for integers. Implementing custom `read_u*` overrides that use `peek_read` and `core::ptr::read_unaligned` significantly speeds up varint decoding.
**Action:** Always override `read_u*` methods in `BufReader` implementations to optimize integer decoding loops.
