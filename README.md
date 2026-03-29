# Bincode-Next

<img align="right" src="./logo.svg" height="200" />

[![Discord Server](https://img.shields.io/discord/1459399539403522074.svg?label=Discord&logo=discord&color=blue)](https://discord.gg/D5e2czMTT9)
[![License](https://img.shields.io/crates/l/bincode-next)](https://opensource.org/licenses/MIT)
[![Scc Count Badge Code](https://sloc.xyz/github/Apich-Organization/bincode/?category=code)](https://github.com/Apich-Organization/bincode/)
[![DOI](https://zenodo.org/badge/DOI/10.6084/m9.figshare.31410402.svg)](https://doi.org/10.6084/m9.figshare.31410402)

**Bincode-Next** is a high-performance binary encoder/decoder pair that uses a zero-fluff encoding scheme. It is a modernized fork of the original `bincode` library, maintained by the Apich Organization to ensure continued development and extreme performance optimizations for the Rust ecosystem.

The size of the encoded object will be the same or smaller than the size that the object takes up in memory in a running Rust program.

## Key Features

- **Performance**: Leverages SIMD (SSE2 on x86_64, NEON on AArch64) for rapid varint scanning and bulk primitive copying for massive throughput.
- **Zero-Copy**: Nested Zero-copy support via Relative Pointers and Const Alignment. (optional feature, using the `zerocopy` feature to enable)
- **Bit-Packing**: Bit-level Packing for Space-Optimized Serialization. (optional, using the `BitPacked` derive macro with the `config::standard().with_bit_packing()` config to enable)
- **Schema Fingerprinting**: Schema Fingerprinting for Safe Versioning. (optional, using the `config::standard().with_fingerprint()` with the derive macro `Fingerprint` to enable)
- **Compile-time Memory Bound Validation**: Compile-time Memory Bound Validation via Const Generics. (optional feature, enable the `static-size` feature to use it)
- **Stream Support**: Works seamlessly with `std::io` (Reader/Writer) and `no_std` environments.

## Getting Started

Add `bincode-next` to your `Cargo.toml`:

```toml
[dependencies]
bincode-next = "3.0.0-rc.5"
```

### Basic Example

```rust
use bincode_next::{config, Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug)]
struct Entity {
    x: f32,
    y: f32,
}

#[derive(Encode, Decode, PartialEq, Debug)]
struct World(Vec<Entity>);

fn main() {
    let config = config::standard();

    let world = World(vec![Entity { x: 0.0, y: 4.0 }, Entity { x: 10.0, y: 20.5 }]);

    // Encode to a Vec<u8>
    let encoded: Vec<u8> = bincode_next::encode_to_vec(&world, config).unwrap();

    // Decode from a slice
    let (decoded, len): (World, usize) = bincode_next::decode_from_slice(&encoded[..], config).unwrap();

    assert_eq!(world, decoded);
    assert_eq!(len, encoded.len());
}
```

### Bit-Packing Example

Enable bit-packing in your configuration to use bit-level field sizing:

```rust
use bincode_next::{config, BitPacked};

#[derive(BitPacked, PartialEq, Debug)]
struct Packed {
    #[bincode(bits = 3)]
    a: u8,
    #[bincode(bits = 5)]
    b: u8,
}

fn main() {
    let config = config::standard().with_bit_packing();
    let val = Packed { a: 7, b: 31 };
    
    let encoded = bincode_next::encode_to_vec(&val, config).unwrap();
    // 'a' (3 bits) + 'b' (5 bits) = 8 bits (1 byte)
    assert_eq!(encoded.len(), 1); 
}
```

### High-Performance Async Decoding

Bincode-Next supports true zero-cost asynchronous decoding using **Unified Fiber-backed Async (UFA)**. By executing synchronous traits on a dedicated lightweight fiber stack, we avoid the overhead of state machine generation and achieve high performance over `tokio::io::AsyncRead`.

```rust
use bincode_next::{config, decode_async, encode_to_vec, Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug)]
struct Entity { x: f32, y: f32 }

#[tokio::main]
async fn main() {
    let my_entity = Entity { x: 1.0, y: 2.0 };
    let encoded = encode_to_vec(&my_entity, config::standard()).unwrap();
    // You can use any type that implements `futures_io::AsyncRead`.
    // For this example, we'll use a simple byte slice, which implements it.
    let mut reader = &encoded[..];
    
    // Decodes asynchronously on a fiber without custom async traits!
    let entity: Entity = decode_async(config::standard(), &mut reader).await.unwrap();
    assert_eq!(my_entity, entity);
}
```

## Performance Optimizations

Bincode-Next includes advanced optimizations for extreme performance:
- **SIMD Varint Scanning**: Accelerates decoding of collections (like `Vec<u64>`) by scanning for small values using SSE2 or NEON instructions.
- **Bulk Native Copy**: Automatically detects when data can be copied directly from memory (e.g., slices of primitives with matching endianness) to avoid element-wise processing.
- **Uninitialized Memory**: Utilizes `MaybeUninit` and `set_len` optimizations for `Vec` decoding to avoid redundant zero-initialization.

```shell
git clone https://github.com/Apich-Organization/bincode.git
cd bincode
# We need root permission and use nightly compiler to ensure the most accurate result
sudo cargo +nightly bench --bench extreme_perf
sudo cargo +nightly bench --bench complex
```

TL;DR:Please visit [https://bincode-next.apich.org/](https://bincode-next.apich.org/) for more detailed information.

### **Performance Comparison: Decoding**

*Baseline: **bincode-next (traits, varint)** at 16.878 µs*

| Rank  | Implementation   | Interface | Int Encoding | Median Time   | Relative Speed |
| ----- | ---------------- | --------- | ------------ | ------------- | -------------- |
| **1** | **bincode-next** | traits    | varint       | **16.878 µs** | **1.00x**      |
| 2     | **bincode-next** | traits    | fixed        | 21.872 µs     | 1.30x          |
| 3     | **bincode-v2**   | serde     | fixed        | 21.973 µs     | 1.30x          |
| 4     | **bincode-v1**   | serde     | N/A          | 22.074 µs     | 1.31x          |
| 5     | **bincode-v2**   | serde     | varint       | 25.727 µs     | 1.52x          |

---

### **Performance Comparison: Encoding**

*Baseline: **bincode-next (traits, fixed)** at 2.9350 µs*

| Rank  | Implementation   | Interface | Int Encoding | Median Time   | Relative Speed |
| ----- | ---------------- | --------- | ------------ | ------------- | -------------- |
| **1** | **bincode-next** | traits    | fixed        | **2.9350 µs** | **1.00x**      |
| 2     | **bincode-v1**   | serde     | N/A          | 3.0767 µs     | 1.05x          |
| 3     | **bincode-v2**   | serde     | fixed        | 3.3295 µs     | 1.13x          |
| 4     | **bincode-next** | traits    | varint       | 3.3467 µs     | 1.14x          |
| 5     | **bincode-v2**   | serde     | varint       | 4.2489 µs     | 1.45x          |

---

### **Efficiency Score: Combined Round-Trip Performance**

*Sum of Median Decode + Median Encode (Normalized to the fastest = 1.00x)*

| Rank  | Implementation   | Interface  | Int Encoding | Total Time    | Efficiency Score |
| ----- | ---------------- | ---------- | ------------ | ------------- | ---------------- |
| **1** | **bincode-next** | **traits** | **varint**   | **20.225 µs** | **1.00x**        |
| 2     | **bincode-next** | traits     | fixed        | 24.807 µs     | 1.23x            |
| 3     | **bincode-v1**   | serde      | N/A          | 25.151 µs     | 1.24x            |
| 4     | **bincode-v2**   | serde      | fixed        | 25.303 µs     | 1.25x            |
| 5     | **bincode-v2**   | serde      | varint       | 29.976 µs     | 1.48x            |

---

### **Vector `u64` Decoding: Varint Performance**

*Contrasting small vs. large integer varint decoding.*

| Dataset          | Implementation             | Median Time   | Relative Speed |
| ---------------- | -------------------------- | ------------- | -------------- |
| **Small Varint** | **bincode-next (current)** | **2.8256 µs** | **1.00x**      |
| Small Varint     | bincode-v2 (original)      | 12.450 µs     | 4.41x          |
|                  |                            |               |                |
| **Large Varint** | **bincode-next (current)** | **13.062 µs** | **1.00x**      |
| Large Varint     | bincode-v2 (original)      | 17.635 µs     | 1.35x          |

---

### **Vector `u64` Decoding: Fixed Performance**

*Baseline: **bincode-next (current)** at 1.8373 µs*

| Rank  | Implementation             | Median Time   | Relative Speed |
| ----- | -------------------------- | ------------- | -------------- |
| **1** | **bincode-next (current)** | **1.8373 µs** | **1.00x**      |
| 2     | bincode-v1                 | 7.5378 µs     | 4.10x          |
| 3     | bincode-v2 (original)      | 10.129 µs     | 5.51x          |

---

### **Bulk `u8` Decoding: Throughput Performance**

*Baseline: **bincode-next (current)** at 160.44 ns*

| Rank  | Implementation             | Median Time   | Relative Speed |
| ----- | -------------------------- | ------------- | -------------- |
| **1** | **bincode-next (current)** | **160.44 ns** | **1.00x**      |
| 2     | bincode-v2 (original)      | 273.86 ns     | 1.71x          |
| 3     | bincode-v1                 | 6307.00 ns    | 39.31x         |

## About Security and Code Quality

For security issues, please visit [the Security Team Homepage](https://security.apich.org) for more details on reporting.

All code tests passed `miri` and all main crate source code passed `clippy` without errors.

```shell
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --all-features --no-fail-fast
cargo clippy --all-features
```

We remain committed to code security and welcomed security reporting.

And please notice that contributors shall follow the community guide lines of `bincode-next`.

## Specification

The formal wire-format specification is available in [docs/spec.md](docs/spec.md).

## FAQ

### Why Bincode-Next?
Bincode-Next was created to continue the legacy of the original Bincode project while pushing the boundaries of what's possible with modern Rust performance techniques and AI-assisted development.

### Is it compatible with Bincode 1.x / 2.x?
Yes, Bincode-Next is designed to be wire-compatible with Bincode 2.x when using the same configurations. It also supports legacy 1.x formats via configuration.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## License

Bincode-Next is licensed under either of:

- The MIT License (MIT)
- The Apache License, Version 2.0

See [LICENSE.md](LICENSE.md) for details.
