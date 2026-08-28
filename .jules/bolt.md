# 2026-08-30
## Title: Avoiding N Allocations in Deterministic Encoding
## Learning: When deterministically serializing a map or a slice, sorting requires buffering elements. Previously we allocated a `Vec<u8>` for *each* element to be sorted. By allocating a single `buffer = Vec<u8>` and pushing elements continuously, while tracking `indices = Vec<Range<usize>>`, we convert N allocations into 2 allocations (`buffer` and `indices`).
## Action: Look for opportunities where many small intermediate vectors are created to be sorted or stored; switch to an arena or single contiguous buffer. Use `sort_unstable_by` over the stored indices array.
