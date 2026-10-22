## 2024-10-22 - Optimizing Integer Decoding in Readers
**Learning:** For reader trait implementations, overriding the default single-byte reads (`read_u16`, `read_u32`, etc.) with `core::ptr::read_unaligned` when peering ahead directly into the buffer provides measurable performance wins for slices and slightly less impact for `BufReader`.
**Action:** Use `core::ptr::read_unaligned` for integers decoded from slices in `bincode-next` to bypass overhead. Ensure to include `#![allow(unsafe_code)]` when utilizing unsafe pointer reads to conform to linting boundaries.
