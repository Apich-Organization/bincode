
## 2023-10-24 - Optimizing varint decoding loops
**Learning:** In tight varint decoding loops for `BufReader`, explicit array constructions using index access (e.g. `[buf[0], buf[1]]`) and `from_ne_bytes` introduce measurable implicit overhead compared to reading memory directly using `core::ptr::read_unaligned`. Using `read_unaligned` parses integers natively in host endianness and is much faster for types from `u16` up to `u128`.
**Action:** When optimizing decoding loops, use `core::ptr::read_unaligned` on pointers to parse integers safely and directly, rather than building arrays character-by-character. Ensure to wrap with `unsafe` blocks and explicitly allow `unsafe_code` for the relevant functions.
