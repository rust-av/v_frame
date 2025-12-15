// Copyright (c) 2018-2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

//! YUV video frame structures and builders.
//!
//! This module provides the [`Frame`] type, which represents a complete YUV video frame
//! consisting of one luma (Y) plane and optionally two chroma (U and V) planes. Frames
//! are constructed using the [`FrameBuilder`] pattern to ensure type safety and correct
//! configuration.
//!
//! # Frame Structure
//!
//! A YUV frame contains:
//! - **Y plane**: Luma (brightness) information, always present
//! - **U plane**: First chroma component (Cb), present unless monochrome
//! - **V plane**: Second chroma component (Cr), present unless monochrome
//!
//! The relative dimensions of the chroma planes are determined by the
//! [`ChromaSubsampling`](crate::chroma::ChromaSubsampling) format.
//!
//! # Type Safety
//!
//! Frames are generic over the pixel type `T: Pixel` and bit depth `BIT_DEPTH`:
//! - Use `Frame<u8, 8>` for 8-bit video
//! - Use `Frame<u16, BIT_DEPTH>` for high bit-depth (9-16 bit) video
//!
//! The builder validates that the pixel type matches the specified bit depth,
//! returning [`Error::DataTypeMismatch`](crate::error::Error::DataTypeMismatch) if they
//! don't align.
//!
//! # Padding
//!
//! Frames support optional padding around the luma plane, which is automatically
//! propagated to the chroma planes according to the subsampling ratio. Padding is
//! useful for video codec algorithms that need to access pixels beyond the visible
//! frame boundaries.
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
//!
//! // Access the planes
//! assert_eq!(frame.y_plane.width().get(), 1920);
//! assert_eq!(frame.y_plane.height().get(), 1080);
//!
//! // Chroma planes are half size for YUV420
//! let u_plane = frame.u_plane.as_ref().unwrap();
//! assert_eq!(u_plane.width().get(), 960);
//! assert_eq!(u_plane.height().get(), 540);
//! ```
//!
//! # Creating Frames with Padding
//!
//! ```rust
//! use v_frame::frame::FrameBuilder;
//! use v_frame::chroma::ChromaSubsampling;
//! use std::num::NonZeroUsize;
//!
//! let width = NonZeroUsize::new(1920).unwrap();
//! let height = NonZeroUsize::new(1080).unwrap();
//!
//! let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420)
//! .luma_padding_left(16)
//! .luma_padding_right(16)
//! .luma_padding_top(16)
//! .luma_padding_bottom(16)
//! .build::<u16, 10>().unwrap();
//! ```

#[cfg(test)]
mod tests;

use std::num::NonZeroUsize;

use crate::{
    chroma::ChromaSubsampling,
    error::Error,
    pixel::Pixel,
    plane::{Plane, PlaneGeometry},
};

/// Contains the data representing one YUV video frame.
#[derive(Clone)]
pub struct Frame<T: Pixel, const BIT_DEPTH: u8> {
    /// The luma plane for this frame
    pub y_plane: Plane<T, BIT_DEPTH>,
    /// The first chroma plane for this frame, or `None` if this is a grayscale frame
    pub u_plane: Option<Plane<T, BIT_DEPTH>>,
    /// The second chroma plane for this frame, or `None` if this is a grayscale frame
    pub v_plane: Option<Plane<T, BIT_DEPTH>>,
    /// The chroma subsampling for this frame
    pub subsampling: ChromaSubsampling,
}

/// A builder for constructing [`Frame`] instances with validation.
///
/// `FrameBuilder` uses the builder pattern to construct frames safely, validating
/// that all parameters are compatible (bit depth matches pixel type, dimensions are
/// compatible with chroma subsampling, padding is properly aligned, etc.).
///
/// # Required Parameters
///
/// The following parameters must be provided when creating a new builder:
/// - `width`: Frame width in pixels
/// - `height`: Frame height in pixels
/// - `subsampling`: Chroma subsampling format
///
/// The bit depth is specified as a const generic parameter when calling `build()`.
///
/// # Optional Parameters
///
/// Luma padding can be set via setter methods. When padding is set, it is automatically
/// propagated to the chroma planes according to the subsampling ratio.
///
/// # Example
///
/// ```rust
/// use v_frame::frame::FrameBuilder;
/// use v_frame::chroma::ChromaSubsampling;
/// use std::num::NonZeroUsize;
///
/// let frame = FrameBuilder::new(
///     NonZeroUsize::new(1920).unwrap(),
///     NonZeroUsize::new(1080).unwrap(),
///     ChromaSubsampling::Yuv420,
/// )
/// .luma_padding_left(8)
/// .luma_padding_right(8)
/// .build::<u8, 8>().unwrap();
/// ```
pub struct FrameBuilder {
    /// Visible width in pixels.
    width: NonZeroUsize,
    /// Visible height in pixels.
    height: NonZeroUsize,
    /// Chroma subsampling format.
    subsampling: ChromaSubsampling,
    /// Number of padding pixels on the left of the luma plane.
    luma_padding_left: usize,
    /// Number of padding pixels on the right of the luma plane.
    luma_padding_right: usize,
    /// Number of padding pixels on the top of the luma plane.
    luma_padding_top: usize,
    /// Number of padding pixels on the bottom of the luma plane.
    luma_padding_bottom: usize,
}

impl FrameBuilder {
    /// Creates a new frame builder, taking the parameters that are required for all frames.
    /// The builder then allows for setting additional, optional parameters.
    #[inline]
    #[must_use]
    pub fn new(width: NonZeroUsize, height: NonZeroUsize, subsampling: ChromaSubsampling) -> Self {
        Self {
            width,
            height,
            subsampling,
            luma_padding_left: 0,
            luma_padding_right: 0,
            luma_padding_top: 0,
            luma_padding_bottom: 0,
        }
    }

    /// Set the `luma_padding_left` for the frame builder.
    #[inline]
    #[must_use]
    pub fn luma_padding_left(mut self, luma_padding_left: usize) -> Self {
        self.luma_padding_left = luma_padding_left;
        self
    }

    /// Set the `luma_padding_right` for the frame builder.
    #[inline]
    #[must_use]
    pub fn luma_padding_right(mut self, luma_padding_right: usize) -> Self {
        self.luma_padding_right = luma_padding_right;
        self
    }

    /// Set the `luma_padding_top` for the frame builder.
    #[inline]
    #[must_use]
    pub fn luma_padding_top(mut self, luma_padding_top: usize) -> Self {
        self.luma_padding_top = luma_padding_top;
        self
    }

    /// Set the `luma_padding_bottom` for the frame builder.
    #[inline]
    #[must_use]
    pub fn luma_padding_bottom(mut self, luma_padding_bottom: usize) -> Self {
        self.luma_padding_bottom = luma_padding_bottom;
        self
    }

    /// Constructs a `Frame` from the current builder.
    ///
    /// # Errors
    /// - Returns `Error::UnsupportedBitDepth` if the input bit depth is unsupported
    ///   (currently 8-16 bit inputs are supported)
    /// - Returns `Error::DataTypeMismatch` if the size of `T` does not match the input bit depth
    /// - Returns `Error::UnsupportedResolution` if the resolution or padding dimensions
    ///   do not support the requested subsampling
    #[inline]
    pub fn build<T: Pixel, const BIT_DEPTH: u8>(self) -> Result<Frame<T, BIT_DEPTH>, Error> {
        if BIT_DEPTH < 8 || BIT_DEPTH > 16 {
            return Err(Error::UnsupportedBitDepth { found: BIT_DEPTH });
        }

        let byte_width = size_of::<T>();
        assert!(
            byte_width <= 2,
            "unsupported pixel byte width: {byte_width}"
        );
        if (byte_width == 1 && BIT_DEPTH != 8) || (byte_width == 2 && BIT_DEPTH <= 8) {
            return Err(Error::DataTypeMismatch);
        }

        let luma_stride = self
            .width
            .saturating_add(self.luma_padding_left)
            .saturating_add(self.luma_padding_right);
        let luma_geometry = PlaneGeometry {
            width: self.width,
            height: self.height,
            stride: luma_stride,
            pad_left: self.luma_padding_left,
            pad_right: self.luma_padding_right,
            pad_top: self.luma_padding_top,
            pad_bottom: self.luma_padding_bottom,
        };
        if !self.subsampling.has_chroma() {
            return Ok(Frame {
                y_plane: Plane::new(luma_geometry),
                u_plane: None,
                v_plane: None,
                subsampling: self.subsampling,
            });
        }

        let Some((chroma_width, chroma_height)) = self
            .subsampling
            .chroma_dimensions(self.width.get(), self.height.get())
        else {
            return Err(Error::UnsupportedResolution);
        };

        let (ss_x, ss_y) = self.subsampling.subsample_ratio().expect("not monochrome");
        if self.luma_padding_left % ss_x.get() as usize > 0
            || self.luma_padding_right % ss_x.get() as usize > 0
            || self.luma_padding_top % ss_y.get() as usize > 0
            || self.luma_padding_bottom % ss_y.get() as usize > 0
        {
            return Err(Error::UnsupportedResolution);
        }
        let chroma_padding_left = self.luma_padding_left / ss_x.get() as usize;
        let chroma_padding_right = self.luma_padding_right / ss_x.get() as usize;
        let chroma_padding_top = self.luma_padding_top / ss_y.get() as usize;
        let chroma_padding_bottom = self.luma_padding_bottom / ss_y.get() as usize;
        let chroma_stride = chroma_width
            .saturating_add(chroma_padding_left)
            .saturating_add(chroma_padding_right);

        let chroma_geometry = PlaneGeometry {
            width: NonZeroUsize::new(chroma_width).expect("cannot be zero"),
            height: NonZeroUsize::new(chroma_height).expect("cannot be zero"),
            stride: NonZeroUsize::new(chroma_stride).expect("cannot be zero"),
            pad_left: chroma_padding_left,
            pad_right: chroma_padding_right,
            pad_top: chroma_padding_top,
            pad_bottom: chroma_padding_bottom,
        };
        Ok(Frame {
            y_plane: Plane::new(luma_geometry),
            u_plane: Some(Plane::new(chroma_geometry)),
            v_plane: Some(Plane::new(chroma_geometry)),
            subsampling: self.subsampling,
        })
    }
}
