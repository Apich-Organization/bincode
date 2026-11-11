## 2024-05-20 - BufReader Optimizations

**Learning:** In `bincode-next`, optimizing `std::io::BufReader<R>` custom `read_u*` overrides by replacing `Reader::peek_read(self, n)` with manual buffering checks (`std::io::BufRead::fill_buf(self)` and `self.buffer().get(..n)`) avoids indirection/dynamic dispatch overhead and measurably improves varint decoding performance.

**Action:** When working on `bincode-next` optimizations, directly utilize manual buffering checks for custom overrides in `BufReader` implementations. Note that for mutable access without triggering NLL issues, do not use conditional early returns tied to `self`; instead check and update buffer state sequentially, and access the underlying buffer via the inherent `self.buffer()` instead of `BufRead::buffer()`. Use `.len() < n` instead of `is_empty()` for length checks for multiple bytes. Always run the `varint` benchmark to verify performance improvements for these primitives.
