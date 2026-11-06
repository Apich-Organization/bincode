
## 2023-11-06 - BufReader buffer() usage and Code Review
**Learning:** `std::io::BufReader<R>` has an inherent `.buffer()` method that safely returns the current buffer without needing to borrow mutable `self` via `fill_buf`, avoiding NLL issues. The code review tool might incorrectly identify `.buffer()` as invalid if it mistakes the `impl` block for a generic `R: Read`.
**Action:** When working in `impl Reader for std::io::BufReader<R>`, continue using `self.buffer()` for safe buffer length checks and retrievals, despite what code review says. The `cargo check` compilation results are the source of truth.
