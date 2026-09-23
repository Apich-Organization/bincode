## 2024-06-19 - [Fast path buffering for BufReader]
**Learning:** Overriding `read_u*` methods for `std::io::BufReader` to leverage direct memory access via `Reader::peek_read` and `core::ptr::read_unaligned` drastically speeds up deserialization routines where the base reader fallback defaults to unbuffered, byte-by-byte reads.
**Action:** Always verify if a buffered reader wrapper can provide a fast-path for direct integer reads instead of falling back to default slicing logic.
