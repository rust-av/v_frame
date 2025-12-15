# v_frame

[![docs.rs](https://img.shields.io/docsrs/v_frame)](https://docs.rs/v_frame)
[![Crates.io](https://img.shields.io/crates/v/v_frame)](https://crates.io/crates/v_frame)
[![LICENSE](https://img.shields.io/crates/l/v_frame)](https://github.com/rust-av/v_frame/blob/main/LICENSE)
[![dependency status](https://deps.rs/repo/github/rust-av/v_frame/status.svg)](https://deps.rs/repo/github/rust-av/v_frame)
[![codecov](https://codecov.io/github/rust-av/v_frame/branch/main/graph/badge.svg?token=MKT1AZREF0)](https://codecov.io/github/rust-av/v_frame)

A Rust library providing efficient data structures and utilities for handling YUV video frames and planes. Originally developed as part of the [rav1e](https://github.com/xiph/rav1e) video encoder, v_frame has been extracted into a standalone crate for broader use across the Rust AV ecosystem.

## Features

- **Type-safe pixel handling**: Generic `Pixel` trait supporting both 8-bit (`u8`) and high bit-depth (`u16`) video
- **Flexible plane structure**: Efficient memory layout with configurable padding for SIMD operations
- **Multiple chroma formats**: Support for YUV 4:2:0, 4:2:2, 4:4:4, and monochrome
- **Builder pattern API**: Safe and ergonomic frame construction with compile-time guarantees
- **SIMD-friendly alignment**: 64-byte alignment (8-byte on WASM) for optimal performance
- **WebAssembly support**: Works in both browser (`wasm32-unknown-unknown`) and WASI environments
- **Zero-copy iterators**: Efficient row-based and pixel-based iteration without allocations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
v_frame = "0.4"
```

## Quick Start

```rust
use v_frame::{
    frame::FrameBuilder,
    chroma::ChromaSubsampling,
};
use std::num::NonZeroUsize;

// Create a 1920x1080 YUV 4:2:0 frame with 8-bit pixels
let frame = FrameBuilder::new(
    NonZeroUsize::new(1920).unwrap(),
    NonZeroUsize::new(1080).unwrap(),
    ChromaSubsampling::Yuv420,
)
.build::<u8, 8>()
.unwrap();

// Access the Y plane (luma)
let y_plane = &frame.y_plane;
println!("Y plane: {}x{}", y_plane.width(), y_plane.height());

// Iterate over rows
for row in y_plane.rows() {
    // Process each row of pixels
}
```

## Core Concepts

### The Pixel Trait

v_frame is built around a generic `Pixel` trait that abstracts over pixel data types:
- `u8` for 8-bit video
- `u16` for high bit-depth video (9-16 bits)

The type system enforces correct usage at compile time, preventing mismatches between declared bit depth and pixel type.

### Frame Structure

A `Frame` contains:
- `y_plane`: Luma (brightness) plane
- `u_plane`: First chroma plane (None for grayscale)
- `v_plane`: Second chroma plane (None for grayscale)
- `subsampling`: Chroma subsampling mode

The bit depth is specified as a const generic parameter on the `Frame` type.

### Chroma Subsampling

v_frame supports standard YUV formats:
- `Yuv420`: Half-resolution chroma (most common, used in H.264/H.265)
- `Yuv422`: Half-width chroma (used in professional video)
- `Yuv444`: Full-resolution chroma (highest quality)
- `Monochrome`: Grayscale, no chroma planes

## Usage Examples

### Creating a High Bit-Depth Frame

```rust
use v_frame::{frame::FrameBuilder, chroma::ChromaSubsampling};
use std::num::NonZeroUsize;

// 10-bit 4K UHD frame
let frame = FrameBuilder::new(
    NonZeroUsize::new(3840).unwrap(),
    NonZeroUsize::new(2160).unwrap(),
    ChromaSubsampling::Yuv420,
)
.build::<u16, 10>()
.unwrap();
```

### Adding Padding for SIMD Operations

```rust
use v_frame::{frame::FrameBuilder, chroma::ChromaSubsampling};
use std::num::NonZeroUsize;

let frame = FrameBuilder::new(
    NonZeroUsize::new(1920).unwrap(),
    NonZeroUsize::new(1080).unwrap(),
    ChromaSubsampling::Yuv420,
)
.luma_padding_left(16)
.luma_padding_right(16)
.luma_padding_top(16)
.luma_padding_bottom(16)
.build::<u8, 8>().unwrap();
```

### Working with Plane Data

```rust
use v_frame::{frame::FrameBuilder, chroma::ChromaSubsampling};
use std::num::NonZeroUsize;

let mut frame = FrameBuilder::new(
    NonZeroUsize::new(640).unwrap(),
    NonZeroUsize::new(480).unwrap(),
    ChromaSubsampling::Yuv420,
)
.build::<u8, 8>()
.unwrap();

// Access a specific row
if let Some(row) = frame.y_plane.row_mut(10) {
    // Fill row with a value
    row.fill(128);
}

// Iterate over all pixels in the plane
for pixel_row in frame.y_plane.rows() {
    for &pixel in pixel_row {
        // Process each pixel
    }
}
```

### Creating a Grayscale Frame

```rust
use v_frame::{frame::FrameBuilder, chroma::ChromaSubsampling};
use std::num::NonZeroUsize;

let frame = FrameBuilder::new(
    NonZeroUsize::new(1280).unwrap(),
    NonZeroUsize::new(720).unwrap(),
    ChromaSubsampling::Monochrome,
)
.build::<u8, 8>()
.unwrap();

// u_plane and v_plane are None for monochrome
assert!(frame.u_plane.is_none());
assert!(frame.v_plane.is_none());
```

## WebAssembly Support

v_frame works in WebAssembly environments with appropriate feature detection:

```bash
# Build for browser
cargo build --target wasm32-unknown-unknown

# Build for WASI
cargo build --target wasm32-wasi

# Test in browsers
wasm-pack test --headless --chrome --firefox
```

The crate automatically adjusts memory alignment for WASM targets (8-byte vs 64-byte on native).

## Feature Flags

- `padding_api`: Exposes low-level APIs for direct access to plane padding data (`geometry()`, `data()`, `data_mut()`)

## Requirements

- Rust 1.85.0 or later
- For WebAssembly: `wasm-bindgen` is automatically included for `wasm32-unknown-unknown` target

## Documentation

- [API Documentation](https://docs.rs/v_frame)
- [Crates.io](https://crates.io/crates/v_frame)

## Building and Testing

```bash
# Build with linting
cargo clippy

# Run tests
cargo test

# Verify MSRV
cargo msrv verify
```

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## License

v_frame is licensed under the BSD 2-Clause License. See [LICENSE](LICENSE) for details.
