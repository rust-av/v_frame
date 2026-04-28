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

use std::fmt;

/// The error type for `v_frame` operations.
///
/// This enum represents all possible error conditions that can occur during
/// frame processing, including data validation errors, unsupported formats,
/// and configuration mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    /// Returned when attempting to create a frame with an unsupported bit depth.
    ///
    /// The library only supports bit depths from 8 to 16 bits inclusive.
    UnsupportedBitDepth {
        /// The requested bit depth which triggered the error
        found: u8,
    },

    /// Returned when the pixel data type does not match the specified bit depth.
    ///
    /// 8-bit frames must use `u8`, while 9-16 bit frames must use `u16`.
    DataTypeMismatch,

    /// Returned when frame dimensions are incompatible with the chroma subsampling format.
    ///
    /// For example, YUV420 requires even width and height, while YUV422 requires even width.
    UnsupportedResolution,
}

impl fmt::Display for Error {
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "string formatting often generates big code"
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnsupportedBitDepth { found } => write!(
                f,
                "only 8-16 bit frame data is supported, tried to create {found} bit frame"
            ),
            Error::DataTypeMismatch => write!(f, "bit depth did not match requested data type"),
            Error::UnsupportedResolution => write!(
                f,
                "selected chroma subsampling does not support odd resolutions"
            ),
        }
    }
}

impl std::error::Error for Error {}
