# Benchmark of bincode-next

This is a benchmark of the `bincode-next` crate (v3.1.1), comparing its performance with other serialization libraries.
Workflow run: https://github.com/Apich-Organization/bincode/actions/runs/27864808824/job/82466972303
Full reports: [report](./criterion-report/report/index.html)

### CBOR Encoding & Decoding

| Implementation | Encode (µs) | **Relative Speed (Enc)** | Decode (µs) | **Relative Speed (Dec)** |
| --- | --- | --- | --- | --- |
| `bincode-next` | 5.64 | **1.00x** | 30.47 | **1.00x** |
| `bincode-next` (det.) | 5.68 | 1.01x | 30.49 | 1.00x |
| `minicbor` | 9.36 | 1.66x | 41.42 | 1.36x |
| `cbor4ii` | 11.98 | 2.12x | 63.54 | 2.09x |

---

### Complex World Benchmarks

*Baseline: `bincode-next` (fixed) for encoding, `bincode-next` (varint) for decoding.*

| Implementation | Encode (µs) | **Rel. Speed** | Decode (µs) | **Rel. Speed** |
| --- | --- | --- | --- | --- |
| `bincode-next` (fixed) | 3.23 | **1.00x** | 20.63 | 1.10x |
| `bincode-next` (varint) | 3.44 | 1.07x | 18.74 | **1.00x** |
| `bincode-v1` (serde) | 3.31 | 1.02x | 18.96 | 1.01x |
| `bincode-v2` (fixed) | 3.43 | 1.06x | 18.71 | 1.00x |
| `bincode-v2` (varint) | 4.22 | 1.31x | 25.00 | 1.33x |
| `bincode-next` (cbor) | 5.99 | 1.85x | 24.82 | 1.32x |
| `bincode-next` (cbor-det) | 6.01 | 1.86x | 24.68 | 1.32x |

---

### Postcard Comparison

| Implementation | Encode (µs) | **Rel. Speed (Enc)** | Decode (µs) | **Rel. Speed (Dec)** |
| --- | --- | --- | --- | --- |
| `bincode-next` (fixed) | 3.58 | **1.00x** | 25.03 | **1.00x** |
| `bincode-next` (varint) | 5.06 | 1.41x | 25.49 | 1.02x |
| `postcard` | 8.38 | 2.34x | 30.96 | 1.24x |
