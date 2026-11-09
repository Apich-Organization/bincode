## 2024-11-10 - Optimizing Custom read_u* Overrides in std::io::BufReader

**Learning:** Replacing dynamic dispatch over `peek_read` in custom `read_u*` implementations with manual buffer checking (`std::io::BufRead::fill_buf(self)`) improves varint decoding throughput.

**Action:** In `src/features/impl_std.rs`, within `impl<R> Reader for std::io::BufReader<R>`, modify `read_u8`, `read_u16`, `read_u32`, `read_u64`, and `read_u128` to manually check if the internal buffer (`self.buffer()`) has enough bytes. If not, explicitly call `std::io::BufRead::fill_buf(self)` and then get the chunk via `self.buffer().get(..n)`. Do not use `peek_read`. Also, use `<Self as std::io::BufRead>::consume` instead of `Reader::consume`, and `std::io::Read::read_exact` instead of `Reader::read` as the fallback to avoid indirection.
