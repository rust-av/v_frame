// Copyright (c) 2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

//! Error types for the `v_frame` crate.
//!
//! This module defines the error types used throughout the `v_frame` crate for
//! handling various error conditions related to frame processing, data validation,
//! and format compatibility.

use thiserror::Error;

/// The error type for `v_frame` operations.
///
/// This enum represents all possible error conditions that can occur during
/// frame processing, including data validation errors, unsupported formats,
/// and configuration mismatches.
#[derive(Error, Debug)]
pub enum Error {
    /// Returned when the provided data buffer size does not match the expected size.
    ///
    /// This typically occurs when constructing a plane or frame from raw data with
    /// incorrect dimensions.
    #[error("data length mismatch, expected {expected}, found {found}")]
    DataLength {
        /// The expected data length based on the provided dimensions
        expected: usize,
        /// The actual length of the provided data array
        found: usize,
    },

    /// Returned when attempting to create a frame with an unsupported bit depth.
    ///
    /// The library only supports bit depths from 8 to 16 bits inclusive.
    #[error("only 8-16 bit frame data is supported, tried to create {found} bit frame")]
    UnsupportedBitDepth {
        /// The requested bit depth which triggered the error
        found: u8,
    },

    /// Returned when the pixel data type does not match the specified bit depth.
    ///
    /// 8-bit frames must use `u8`, while 9-16 bit frames must use `u16`.
    #[error("bit depth did not match requested data type")]
    DataTypeMismatch,

    /// Returned when frame dimensions are incompatible with the chroma subsampling format.
    ///
    /// For example, YUV420 requires even width and height, while YUV422 requires even width.
    #[error("selected chroma subsampling does not support odd resolutions")]
    UnsupportedResolution,

    /// Returned when a plane's stride is smaller than its visible width.
    ///
    /// The stride must be at least as large as the width to accommodate each row of pixels.
    #[error("provided stride {stride} was less than the visible width {width}")]
    InvalidStride {
        /// The stride which triggered the error
        stride: usize,
        /// The visible width of the plane
        width: usize,
    },
}
