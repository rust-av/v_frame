// Copyright (c) 2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

//! Chroma subsampling formats for YUV video frames.
//!
//! This module defines the [`ChromaSubsampling`] enum, which specifies how chroma
//! (color) information is sampled relative to luma (brightness) information in YUV
//! video frames. Chroma subsampling is a common technique in video compression that
//! takes advantage of the human visual system's lower sensitivity to color detail
//! compared to brightness detail.
//!
//! # Subsampling Formats
//!
//! - **YUV420**: Chroma at half width and half height (most common in video compression)
//! - **YUV422**: Chroma at half width, full height (common in professional video)
//! - **YUV444**: Full resolution chroma (highest quality, no subsampling)
//! - **Monochrome**: No chroma planes (grayscale)
//!
//! # Resolution Constraints
//!
//! Each subsampling format imposes constraints on valid frame dimensions:
//! - YUV420 requires even width and even height
//! - YUV422 requires even width
//! - YUV444 has no constraints
//! - Monochrome has no constraints
//!
//! These constraints ensure that chroma dimensions are exact integers after division.
//!
//! # Example
//!
//! ```rust
//! use v_frame::chroma::ChromaSubsampling;
//!
//! let subsampling = ChromaSubsampling::Yuv420;
//!
//! // Check if this format has chroma planes
//! assert!(subsampling.has_chroma());
//!
//! // Calculate chroma dimensions for a 1920x1080 frame
//! let (chroma_width, chroma_height) = subsampling
//!     .chroma_dimensions(1920, 1080)
//!     .unwrap();
//! assert_eq!(chroma_width, 960);
//! assert_eq!(chroma_height, 540);
//!
//! // Odd dimensions are invalid for YUV420
//! assert!(subsampling.chroma_dimensions(1919, 1080).is_none());
//! ```

#[cfg(test)]
mod tests;

use std::num::NonZeroU8;

/// Specifies the chroma subsampling for a YUV frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaSubsampling {
    /// Chroma at half width, half height
    Yuv420,
    /// Chroma at half width, full height
    Yuv422,
    /// Chroma at full resolution
    Yuv444,
    /// No chroma planes
    Monochrome,
}

impl ChromaSubsampling {
    /// Whether the specified chroma subsampling has chroma planes.
    #[inline]
    #[must_use]
    pub fn has_chroma(&self) -> bool {
        *self != Self::Monochrome
    }

    /// Computes the dimensions for a chroma plane with the current subsampling
    /// for the given luma dimensions.
    ///
    /// Returns `None` if the subsampling has no chroma planes,
    /// or if the subsampling is invalid for the luma dimensions
    /// (e.g. odd resolution for YUV420).
    #[inline]
    #[must_use]
    pub fn chroma_dimensions(
        &self,
        luma_width: usize,
        luma_height: usize,
    ) -> Option<(usize, usize)> {
        let subsample = self.subsample_ratio()?;

        let ss_x = subsample.0.get() as usize;
        let ss_y = subsample.1.get() as usize;

        // Check if the division is exact (no remainder)
        (luma_width % ss_x == 0 && luma_height % ss_y == 0)
            .then(|| (luma_width / ss_x, luma_height / ss_y))
    }

    /// Returns the divisor for the chroma dimensions for the given subsampling.
    #[inline]
    #[must_use]
    pub fn subsample_ratio(&self) -> Option<(NonZeroU8, NonZeroU8)> {
        const ONE: NonZeroU8 = NonZeroU8::new(1).unwrap();
        const TWO: NonZeroU8 = NonZeroU8::new(2).unwrap();

        match self {
            ChromaSubsampling::Yuv420 => Some((TWO, TWO)),
            ChromaSubsampling::Yuv422 => Some((TWO, ONE)),
            ChromaSubsampling::Yuv444 => Some((ONE, ONE)),
            ChromaSubsampling::Monochrome => None,
        }
    }
}
