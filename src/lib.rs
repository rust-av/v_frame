// Copyright (c) 2020-2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

//! A library for handling YUV video frames and planes.
//!
//! `v_frame` provides data structures and utilities for working with YUV video data,
//! originally extracted from the rav1e video encoder. The library supports various
//! chroma subsampling formats (YUV420, YUV422, YUV444, and Monochrome) and both 8-bit
//! and high bit-depth (9-16 bit) pixel data.
//!
//! # Core Components
//!
//! - [`Pixel`](pixel::Pixel): Trait abstracting over pixel data types (`u8` and `u16`)
//! - [`Plane`](plane::Plane): A single plane of pixel data with optional padding
//! - [`Frame`](frame::Frame): A complete YUV frame containing Y, U, and V planes
//! - [`ChromaSubsampling`](chroma::ChromaSubsampling): Enum specifying chroma subsampling format
//!
//! # Example
//!
//! ```rust
//! use v_frame::frame::FrameBuilder;
//! use v_frame::chroma::ChromaSubsampling;
//! use std::num::NonZeroUsize;
//!
//! // Create a 1920x1080 YUV420 8-bit frame
//! let width = NonZeroUsize::new(1920).unwrap();
//! let height = NonZeroUsize::new(1080).unwrap();
//!
//! let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420)
//!     .build::<u8, 8>()
//!     .unwrap();
//! ```

pub mod chroma;
pub mod error;
pub mod frame;
pub mod pixel;
pub mod plane;
