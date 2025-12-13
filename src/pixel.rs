// Copyright (c) 2017-2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

//! Pixel data type abstractions.
//!
//! This module defines the [`Pixel`] trait, which abstracts over the pixel data types
//! used throughout the library. This allows the same code to work with both 8-bit
//! (`u8`) and high bit-depth (`u16`) pixel data.
//!
//! # Supported Pixel Types
//!
//! - `u8`: For 8-bit pixel data
//! - `u16`: For 9-16 bit pixel data (high bit-depth)
//!
//! The type used must match the bit depth specified when creating frames:
//! - 8-bit frames must use `u8`
//! - 9-16 bit frames must use `u16`

use num_traits::PrimInt;

/// A trait for types that can be used as pixel data.
///
/// This trait abstracts over the pixel data types supported by the library,
/// currently `u8` for 8-bit data and `u16` for high bit-depth (9-16 bit) data.
///
/// All frame and plane types are generic over `T: Pixel`, allowing the same
/// data structures and algorithms to work with both standard and high bit-depth
/// video content.
///
/// # Type Safety
///
/// The library enforces correct type usage through validation:
/// - Frames with 8-bit depth can only be created with `T = u8`
/// - Frames with 9-16 bit depth can only be created with `T = u16`
///
/// Attempting to create a frame with a mismatched type will result in
/// [`Error::DataTypeMismatch`](crate::error::Error::DataTypeMismatch).
pub trait Pixel: Copy + Clone + Default + Send + Sync + PrimInt {}

/// Pixel implementation for 8-bit video data.
impl Pixel for u8 {}

/// Pixel implementation for high bit-depth (9-16 bit) video data.
impl Pixel for u16 {}
