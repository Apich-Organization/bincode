## YYYY-MM-DD - [Title]
**Learning:** [Insight]
**Action:** [How to apply next time]

## 2024-11-20 - BufReader Optimization
**Learning:** `bincode-next` custom `read_u*` overrides in `BufReader` use `peek_read` to speed up decoding. However, `peek_read` must proactively call `fill_buf()` if `self.buffer().len() < n` to ensure sufficient data is buffered to hit these fast paths. Since `BufReader`'s `peek_read` currently tries to `fill_buf()`, it could potentially be faster to call `fill_buf()` first in the custom `read_u*` overrides before checking the buffer size directly. Wait, the memory states:
`When modifying Rust functions returning references (like peek_read returning Option<&[u8]>), avoid placing conditional returns tied to self in an early branch (e.g., if self.buffer().len() >= n { return Some(self.buffer().get(..n)); }), as it can trigger NLL (Non-Lexical Lifetimes) borrow checker issues. Instead, sequentially mutate the state and fetch the final reference at the end of the block (e.g., if self.buffer().len() < n { let _ = std::io::BufRead::fill_buf(self); } self.buffer().get(..n)).`
`In bincode-next, std::io::BufReader<R> custom read_u* overrides use peek_read to speed up varint decoding. However, peek_read must proactively call fill_buf() if self.buffer().len() < n to ensure sufficient data is buffered to hit these fast paths (rather than falling back to slow reads), especially for larger varints like u32 and u64.`
