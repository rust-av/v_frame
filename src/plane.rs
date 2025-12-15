// Copyright (c) 2017-2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

//! Plane data structure for storing two-dimensional pixel data.
//!
//! This module provides the [`Plane`] type, which represents a single plane of pixel data
//! with optional padding. Planes are the building blocks of video frames, with a YUV frame
//! typically consisting of one luma (Y) plane and two chroma (U and V) planes.
//!
//! # Memory Layout
//!
//! Planes store data in a contiguous, aligned buffer with support for padding on all sides:
//! - Data is aligned to 64 bytes on non-WASM platforms (SIMD-friendly)
//! - Data is aligned to 8 bytes on WASM platforms
//! - Padding pixels surround the visible area for codec algorithms that need border access
//!
//! # API Design
//!
//! The public API exposes only the "visible" pixels by default, abstracting away the padding.
//! Methods like [`rows()`](Plane::rows), [`pixels()`](Plane::pixels), and indexing operations
//! work only with the visible area. Low-level padding access is available via the
//! `padding_api` feature flag.
//!
//! To ensure safety, planes must be instantiated by building a [`Frame`](crate::frame::Frame)
//! through the [`FrameBuilder`](crate::frame::FrameBuilder) interface.

#[cfg(test)]
mod tests;

use std::{iter, num::NonZeroUsize};

use aligned_vec::{ABox, AVec, ConstAlign};

use crate::{error::Error, pixel::Pixel};

/// Alignment for plane data on WASM platforms (8 bytes).
#[cfg(target_arch = "wasm32")]
const DATA_ALIGNMENT: usize = 1 << 3;

/// Alignment for plane data on non-WASM platforms (64 bytes for SIMD optimization).
#[cfg(not(target_arch = "wasm32"))]
const DATA_ALIGNMENT: usize = 1 << 6;

/// A two-dimensional plane of pixel data with optional padding.
///
/// `Plane<T>` represents a rectangular array of pixels of type `T`, where `T` implements
/// the [`Pixel`] trait (currently `u8` or `u16`). The plane supports arbitrary padding
/// on all four sides, which is useful for video codec algorithms that need to access
/// pixels beyond the visible frame boundaries.
///
/// # Memory Layout
///
/// The data is stored in a contiguous, aligned buffer:
/// - 64-byte alignment on non-WASM platforms (optimized for SIMD operations)
/// - 8-byte alignment on WASM platforms
///
/// The visible pixels are surrounded by optional padding pixels. The public API
/// provides access only to the visible area by default; padding access requires
/// the `padding_api` feature flag.
///
/// # Accessing Pixels
///
/// Planes provide several ways to access pixel data:
/// - [`row()`](Plane::row) / [`row_mut()`](Plane::row_mut): Access a single row by index
/// - [`rows()`](Plane::rows) / [`rows_mut()`](Plane::rows_mut): Iterate over all visible rows
/// - [`pixel()`](Plane::pixel) / [`pixel_mut()`](Plane::pixel_mut): Access individual pixels
/// - [`pixels()`](Plane::pixels) / [`pixels_mut()`](Plane::pixels_mut): Iterate over all visible pixels
#[derive(Clone)]
pub struct Plane<T: Pixel, const BIT_DEPTH: u8> {
    /// The underlying pixel data buffer, including padding.
    pub(crate) data: ABox<[T], ConstAlign<DATA_ALIGNMENT>>,
    /// Geometry information describing dimensions and padding.
    pub(crate) geometry: PlaneGeometry,
}

impl<T, const BIT_DEPTH: u8> Plane<T, BIT_DEPTH>
where
    T: Pixel,
{
    /// Creates a new plane with the given geometry, initialized with zero-valued pixels.
    pub(crate) fn new(geometry: PlaneGeometry) -> Self {
        let rows = geometry
            .height
            .saturating_add(geometry.pad_top)
            .saturating_add(geometry.pad_bottom);
        Self {
            data: AVec::from_iter(
                DATA_ALIGNMENT,
                iter::repeat_n(T::zero(), geometry.stride.get() * rows.get()),
            )
            .into_boxed_slice(),
            geometry,
        }
    }

    /// Returns the visible width of the plane in pixels
    #[inline]
    #[must_use]
    pub fn width(&self) -> NonZeroUsize {
        self.geometry.width
    }

    /// Returns the visible height of the plane in pixels
    #[inline]
    #[must_use]
    pub fn height(&self) -> NonZeroUsize {
        self.geometry.height
    }

    /// Returns a slice containing the visible pixels in
    /// the row at vertical index `y`.
    #[inline]
    #[must_use]
    pub fn row(&self, y: usize) -> Option<&[T]> {
        self.rows().nth(y)
    }

    /// Returns a mutable slice containing the visible pixels in
    /// the row at vertical index `y`.
    #[inline]
    #[must_use]
    pub fn row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        self.rows_mut().nth(y)
    }

    /// Returns an iterator over the visible pixels of each row
    /// in the plane, from top to bottom.
    #[inline]
    pub fn rows(&self) -> impl Iterator<Item = &[T]> {
        let origin = self.data_origin();
        // SAFETY: The plane creation interface ensures the data is large enough
        let visible_data = unsafe { self.data.get_unchecked(origin..) };
        visible_data
            .chunks(self.geometry.stride.get())
            .take(self.geometry.height.get())
            .map(|row| {
                // SAFETY: The plane creation interface ensures the data is large enough
                unsafe { row.get_unchecked(..self.geometry.width.get()) }
            })
    }

    /// Returns a mutable iterator over the visible pixels of each row
    /// in the plane, from top to bottom.
    #[inline]
    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut [T]> {
        let origin = self.data_origin();
        // SAFETY: The plane creation interface ensures the data is large enough
        let visible_data = unsafe { self.data.get_unchecked_mut(origin..) };
        visible_data
            .chunks_mut(self.geometry.stride.get())
            .take(self.geometry.height.get())
            .map(|row| {
                // SAFETY: The plane creation interface ensures the data is large enough
                unsafe { row.get_unchecked_mut(..self.geometry.width.get()) }
            })
    }

    /// Return the value of the pixel at the given `(x, y)` coordinate,
    /// or `None` if the index is out of bounds.
    ///
    /// Since this performs bounds checking, it is likely less performant
    /// and should not be used to iterate over rows and pixels.
    #[inline]
    #[must_use]
    pub fn pixel(&self, x: usize, y: usize) -> Option<T> {
        let index = self.data_origin() + self.geometry.stride.get() * y + x;
        self.data.get(index).copied()
    }

    /// Return a mutable reference to the pixel at the given `(x, y)` coordinate,
    /// or `None` if the index is out of bounds.
    ///
    /// Since this performs bounds checking, it is likely less performant
    /// and should not be used to iterate over rows and pixels.
    #[inline]
    pub fn pixel_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        let index = self.data_origin() + self.geometry.stride.get() * y + x;
        self.data.get_mut(index)
    }

    /// Returns an iterator over the visible pixels in the plane,
    /// in row-major order.
    #[inline]
    pub fn pixels(&self) -> impl Iterator<Item = T> {
        self.rows().flatten().copied()
    }

    /// Returns a mutable iterator over the visible pixels in the plane,
    /// in row-major order.
    #[inline]
    pub fn pixels_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.rows_mut().flatten()
    }

    /// Returns an iterator over the visible byte data in the plane,
    /// in row-major order. High-bit-depth data is converted to `u8`
    /// using low endianness.
    #[inline]
    pub fn byte_data(&self) -> impl Iterator<Item = u8> {
        let byte_width = size_of::<T>();
        assert!(
            byte_width <= 2,
            "unsupported pixel byte width: {byte_width}"
        );

        self.pixels().flat_map(move |pix| {
            let bytes: [u8; 2] = if byte_width == 1 {
                [
                    pix.to_u8()
                        .expect("Pixel::byte_data only supports u8 and u16 pixels"),
                    0,
                ]
            } else {
                pix.to_u16()
                    .expect("Pixel::byte_data only supports u8 and u16 pixels")
                    .to_le_bytes()
            };
            bytes.into_iter().take(byte_width)
        })
    }

    /// Copies the data from `src` into this plane's visible pixels.
    ///
    /// # Errors
    /// - Returns `Error::Datalength` if the length of `src` does not match
    ///   this plane's `width * height`
    #[inline]
    pub fn copy_from_slice(&mut self, src: &[T]) -> Result<(), Error> {
        let pixel_count = self.width().get() * self.height().get();
        if pixel_count != src.len() {
            return Err(Error::DataLength {
                expected: pixel_count,
                found: src.len(),
            });
        }

        for (dest, src) in self.pixels_mut().zip(src.iter()) {
            *dest = *src;
        }
        Ok(())
    }

    /// Copies the data from `src` into this plane's visible pixels.
    /// This differs from `copy_from_slice` in that it accepts a raw slice
    /// of `u8` data, which is often what is provided by decoders even if
    /// the pixel data is high-bit-depth. This will convert high-bit-depth
    /// pixels to `u16`, assuming low endian encoding. For low-bit-depth data,
    /// this is equivalent to `copy_from_slice`.
    ///
    /// # Errors
    /// - Returns `Error::Datalength` if the length of `src` does not match
    ///   this plane's `width * height * bytes_per_pixel`
    #[inline]
    pub fn copy_from_u8_slice(&mut self, src: &[u8]) -> Result<(), Error> {
        self.copy_from_u8_slice_with_stride(src, self.width())
    }

    /// Copies the data from `src` into this plane's visible pixels.
    /// This version accepts inputs where the row stride is longer than the visible data width.
    /// The `input_stride` must be in pixels.
    ///
    /// # Errors
    /// - Returns `Error::Datalength` if the length of `src` does not match
    ///   this plane's `width * height * bytes_per_pixel`
    /// - Returns `Error::InvalidStride` if the stride is shorter than the visible width
    #[inline]
    pub fn copy_from_u8_slice_with_stride(
        &mut self,
        src: &[u8],
        input_stride: NonZeroUsize,
    ) -> Result<(), Error> {
        let byte_width = size_of::<T>();
        assert!(
            byte_width <= 2,
            "unsupported pixel byte width: {byte_width}"
        );

        if input_stride < self.width() {
            return Err(Error::InvalidStride {
                stride: input_stride.get(),
                width: self.width().get(),
            });
        }

        let byte_count = input_stride.get() * self.height().get() * byte_width;
        if byte_count != src.len() {
            return Err(Error::DataLength {
                expected: byte_count,
                found: src.len(),
            });
        }

        let width = self.width().get();
        let stride = input_stride.get();

        if byte_width == 1 {
            // Fast path for u8 pixels
            for (row_idx, dest_row) in self.rows_mut().enumerate() {
                let src_offset = row_idx * stride;
                let src_row = &src[src_offset..src_offset + width];
                // SAFETY: we know that `T` is `u8`
                let src_row_typed = unsafe { &*(src_row as *const [u8] as *const [T]) };
                dest_row.copy_from_slice(src_row_typed);
            }
        } else {
            // u16 pixels - need to convert from little-endian bytes
            let row_byte_width = width * byte_width;
            for (row_idx, dest_row) in self.rows_mut().enumerate() {
                let src_offset = row_idx * stride * byte_width;
                let src_row = &src[src_offset..src_offset + row_byte_width];

                for (dest_pixel, src_chunk) in dest_row.iter_mut().zip(src_row.chunks_exact(2)) {
                    // SAFETY: we know that each chunk has 2 bytes
                    let bytes =
                        unsafe { [*src_chunk.get_unchecked(0), *src_chunk.get_unchecked(1)] };
                    // SAFETY: we know that `T` is `u16`
                    let dest = unsafe { &mut *(dest_pixel as *mut T as *mut u16) };
                    *dest = u16::from_le_bytes(bytes);
                }
            }
        }

        Ok(())
    }

    /// Returns the geometry of the current plane.
    ///
    /// This is a low-level API intended only for functions that require access to the padding.
    #[inline]
    #[must_use]
    #[cfg(feature = "padding_api")]
    pub fn geometry(&self) -> PlaneGeometry {
        self.geometry
    }

    /// Returns a reference to the current plane's data, including padding.
    ///
    /// This is a low-level API intended only for functions that require access to the padding.
    #[inline]
    #[must_use]
    #[cfg(feature = "padding_api")]
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Returns a mutable reference to the current plane's data, including padding.
    ///
    /// This is a low-level API intended only for functions that require access to the padding.
    #[inline]
    #[must_use]
    #[cfg(feature = "padding_api")]
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Returns the index for the first visible pixel in `data`.
    ///
    /// This is a low-level API intended only for functions that require access to the padding.
    #[inline]
    #[must_use]
    #[cfg_attr(not(feature = "padding_api"), doc(hidden))]
    pub fn data_origin(&self) -> usize {
        self.geometry.stride.get() * self.geometry.pad_top + self.geometry.pad_left
    }
}

/// Describes the geometry of a plane, including dimensions and padding.
///
/// This struct contains all the information needed to interpret the layout of
/// a plane's data buffer, including the visible dimensions and the padding on
/// all four sides.
///
/// The `stride` represents the number of pixels per row in the data buffer,
/// which is equal to `width + pad_left + pad_right`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(feature = "padding_api"), doc(hidden))]
pub struct PlaneGeometry {
    /// Width of the visible area in pixels.
    pub width: NonZeroUsize,
    /// Height of the visible area in pixels.
    pub height: NonZeroUsize,
    /// Data stride (pixels per row in the buffer, including padding).
    pub stride: NonZeroUsize,
    /// Number of padding pixels on the left side.
    pub pad_left: usize,
    /// Number of padding pixels on the right side.
    pub pad_right: usize,
    /// Number of padding pixels on the top.
    pub pad_top: usize,
    /// Number of padding pixels on the bottom.
    pub pad_bottom: usize,
}
