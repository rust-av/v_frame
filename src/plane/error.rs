// Copyright (c) 2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

use core::fmt;

/// An error representing why data couldn't be copied into a Plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CopyError {
    /// Returned when the provided data buffer size does not match the expected size.
    ///
    /// This typically occurs when constructing a plane or frame from raw data with
    /// incorrect dimensions.
    DataLength {
        /// The expected (i.e. destination) data length based on the provided dimensions.
        expected: usize,
        /// The actual (i.e. source) length of the provided data.
        found: usize,
    },

    /// Returned when a plane's stride is smaller than its visible width.
    ///
    /// The stride must be at least as large as the width to accommodate each row of pixels.
    InvalidStride {
        /// The stride which triggered the error.
        stride: usize,
        /// The visible width of the plane.
        width: usize,
    },
}

impl fmt::Display for CopyError {
    #[expect(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataLength { expected, found } => write!(
                f,
                "data length mismatch, expected {expected}, found {found}"
            ),
            Self::InvalidStride { stride, width } => write!(
                f,
                "provided stride {stride} was less than the visible width {width}"
            ),
        }
    }
}

impl core::error::Error for CopyError {}
