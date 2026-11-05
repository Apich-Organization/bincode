## 2024-05-24 - `std::io::BufReader` read overrides optimization
**Learning:** `std::io::BufReader<R>` custom `read_u*` overrides can be optimized by replacing `Reader::peek_read(self, n)` with manual buffering checks (`std::io::BufRead::fill_buf(self)` and `self.buffer().get(..n)`). This avoids indirection/dynamic dispatch overhead and measurably improves varint decoding performance.
**Action:** Always check if `BufReader`'s `read_u*` overrides can inline the `peek_read` logic.
