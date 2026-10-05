## 2024-05-19 - Fast paths for integers in BufReader

**Learning:** When using `BufReader`, custom `read_u*` overrides must defensively call `fill_buf()` when calling `peek_read` if the reader is near a chunk boundary. Because `peek_read` itself just returned `self.buffer().get(..n)`, it wasn't guaranteed to give enough data for `read_unaligned`, and so it frequently fell back to `read(&mut bytes)`, degrading performance for reading varints (especially u16, u32, and u64 which couldn't find enough contiguous bytes).

**Action:** Ensure `peek_read` calls `std::io::BufRead::fill_buf(self)` when `self.buffer().len() < n` so it actually populates the buffer before peeking, allowing `read_u*` to hit the fast `core::ptr::read_unaligned` path almost always.
