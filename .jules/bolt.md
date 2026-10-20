
## 2023-10-20 - [Zero-cost array bypass with read_unaligned]
**Learning:** Using `core::ptr::read_unaligned` natively parses integers in host endianness, providing a zero-cost bypass for array assembling in inner decoding loops (like varint decoding with `BufReader`).
**Action:** When optimizing tight decoding loops where `peek_read` guarantees sufficient buffer length, replace manual byte array reconstruction with direct memory reads via `core::ptr::read_unaligned` and cast the pointer directly to the target integer type.
