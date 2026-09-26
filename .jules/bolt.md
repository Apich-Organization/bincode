## 2024-05-20 - BufReader Reader Implementation Overrides
**Learning:** `std::io::BufReader<R>` defaults to slow byte-by-byte reads for integers. Implementing custom `read_u*` overrides that use `peek_read` and `core::ptr::read_unaligned` significantly speeds up varint decoding.
**Action:** Overriding integer read methods for BufReader.
