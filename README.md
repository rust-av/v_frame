# v_frame

[![docs.rs](https://img.shields.io/docsrs/v_frame)](https://docs.rs/v_frame)
[![Crates.io](https://img.shields.io/crates/v/v_frame)](https://crates.io/crates/v_frame)
[![LICENSE](https://img.shields.io/crates/l/v_frame)](https://github.com/rust-av/v_frame/blob/main/LICENSE)
[![dependency status](https://deps.rs/repo/github/rust-av/v_frame/status.svg)](https://deps.rs/repo/github/rust-av/v_frame)
[![codecov](https://codecov.io/github/rust-av/v_frame/branch/main/graph/badge.svg?token=MKT1AZREF0)](https://codecov.io/github/rust-av/v_frame)

`v_frame` provides data structures and utilities for handling YUV video frames. Originally developed as part of the [rav1e](https://github.com/xiph/rav1e) video encoder, `v_frame` has been extracted into a standalone crate for broader use across the Rust AV ecosystem.

## Features

- **Type-safe pixel handling**: Generic `Pixel` trait supporting both 8-bit (`u8`) and high bit-depth (`u16`) video
- **Support for common subsampling formats**: YUV 4:2:0, 4:2:2, 4:4:4, and monochrome
- **Performant API**: Efficient row-based and pixel-based data access
- **SIMD-friendly data alignment**: Plane data is aligned to at least 64 bytes on most targets
- **WebAssembly support**: Works in both browser (`wasm32-unknown-unknown`) and WASI environments

## Installation

Run `cargo add v_frame` or add this to your `Cargo.toml`:

```toml
[dependencies]
v_frame = "0.6"
```

## The Pixel Trait

v_frame is built around a generic `Pixel` trait that abstracts the two possible underlying data types:
- `u8` for 8-bit video
- `u16` for high bit-depth video (9-16 bits)

The API prevents mismatches between declared bit depth and pixel type.

## Usage

### Creating a Frame

```rust
use v_frame::{frame::FrameBuilder, chroma::ChromaSubsampling};

// 10-bit 4K UHD frame
let frame = FrameBuilder::new(3840, 2160, ChromaSubsampling::Yuv420, 10)
    .build::<u16>()
    .unwrap();
```

### Adding padding pixels

```rust
use v_frame::{frame::FrameBuilder, chroma::ChromaSubsampling};

let mut builder = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 8);

// Add 16 pixels of padding on all sides for block-based algorithms
builder.luma_padding_left(16);
builder.luma_padding_right(16);
builder.luma_padding_top(16);
builder.luma_padding_bottom(16);

let frame = builder.build::<u8>().unwrap();
```

### Working with Plane Data

```rust
use v_frame::{frame::FrameBuilder, chroma::ChromaSubsampling};

let mut frame = FrameBuilder::new(640, 480, ChromaSubsampling::Yuv420, 8)
    .build::<u8>()
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

## WebAssembly Support

v_frame works in WebAssembly environments:

```bash
# Build for browser environments
cargo build --target wasm32-unknown-unknown

# Build for WASI
cargo build --target wasm32-wasip1
```

## Documentation

- [API Documentation](https://docs.rs/v_frame)
- [Crates.io](https://crates.io/crates/v_frame)

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## License

v_frame is licensed under the BSD 2-Clause License. See [LICENSE](LICENSE) for details.
