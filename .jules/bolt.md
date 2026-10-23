## 2024-10-23 - BufRead override performance learning
**Learning:** `std::io::BufRead` custom `read_u*` overrides must pre-fill the buffer if it is too small before calling `get(..n)` directly.
**Action:** When implementing `read_u*` overrides for types with internal buffers (like `BufReader`), explicitly fetch the data and fill the buffer first to maximize hits on the fast path.
