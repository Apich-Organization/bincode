## 2024-07-28 - Fast Paths for Intermediate Varint Sizes
**Learning:** The previous varint decoding implementation in bincode successfully optimized decoding for the largest integer size (e.g. u64 encoded as U64_BYTE) when `peek_read` could supply enough bytes. However, it neglected intermediate sizes (e.g. u16 encoded as U16_BYTE inside a u64 or u128 field), forcing them to fall back to a non-inlined `cold` function, severely degrading performance for intermediate varints despite having enough bytes buffered.
**Action:** Always verify if an inline optimization (e.g. buffer checks) applies to the entire range of possibilities or if it inadvertently penalizes intermediate states. Providing fast paths for intermediate values inside larger functions resolves this bottleneck efficiently.

## 2024-08-08 - [Optimize decoding of single-byte varints]
**Learning:** `peek_read(N)` fails entirely if fewer than `N` bytes are available, causing a fallback to a slow cold path for decoding varints near the end of buffers even if the value only takes 1 byte.
**Action:** Always check `peek_read(1)` first for single-byte varints to avoid unnecessary cold path execution when near the end of a buffer.
