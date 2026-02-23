# Bincode-Next

<img align="right" src="./logo.svg" height="200" />

[![Discord Server](https://img.shields.io/discord/1459399539403522074.svg?label=Discord&logo=discord&color=blue)](https://discord.gg/D5e2czMTT9)
[![](https://img.shields.io/crates/v/bincode-next.svg)](https://crates.io/crates/bincode-next)
[![](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Scc Count Badge Code](https://sloc.xyz/github/Apich-Organization/bincode/?category=code)](https://github.com/Apich-Organization/bincode/)

**Bincode-Next** is a high-performance binary encoder/decoder pair that uses a zero-fluff encoding scheme. It is a modernized fork of the original `bincode` library, maintained by the Apich Organization to ensure continued development and extreme performance optimizations for the Rust ecosystem.

The size of the encoded object will be the same or smaller than the size that the object takes up in memory in a running Rust program.

## Key Features

- **Extreme Performance**: Leverages SIMD (SSE2 on x86_64, NEON on AArch64) for rapid varint scanning and bulk primitive copying for massive throughput.
- **Zero-Copy**: Optimized for minimal memory allocations during both encoding and decoding.
- **Bit-Packing**: Support for bit-level field packing to squeeze every last bit of efficiency out of your data.
- **Cross-Platform**: Rigorously tested for compatibility across different architectures and endianness.
- **Stream Support**: Works seamlessly with `std::io` (Reader/Writer) and `no_std` environments.

## Getting Started

Add `bincode-next` to your `Cargo.toml`:

```toml
[dependencies]
bincode-next = "2.2"
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

## Performance Optimizations

Bincode-Next includes advanced optimizations for extreme performance:
- **SIMD Varint Scanning**: Accelerates decoding of collections (like `Vec<u64>`) by scanning for small values using SSE2 or NEON instructions.
- **Bulk Native Copy**: Automatically detects when data can be copied directly from memory (e.g., slices of primitives with matching endianness) to avoid element-wise processing.
- **Uninitialized Memory**: Utilizes `MaybeUninit` and `set_len` optimizations for `Vec` decoding to avoid redundant zero-initialization.

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
