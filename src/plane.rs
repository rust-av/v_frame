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
//! - Non-empty allocations are aligned to at least 64 bytes on most targets
//!   (SIMD-friendly)
//! - Non-empty allocations are aligned to at least 8 bytes on `wasm32` targets
//!   that are not WASI
//! - If `T` has stricter alignment requirements, the allocation uses
//!   `std::mem::align_of::<T>()` instead
//! - Padding pixels surround the visible area for codec algorithms that need border access
//!
//! More precisely, non-empty plane data is allocated with
//! `max(DATA_ALIGNMENT, std::mem::align_of::<T>())`, where `DATA_ALIGNMENT` is
//! 64 except on non-WASI `wasm32` targets, where it is 8. Empty planes do not
//! allocate and must not be assumed to have this extra SIMD alignment.
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

mod error;
pub use error::CopyError;

#[cfg(test)]
mod tests;

#[cfg(feature = "padding_api")]
use std::mem::MaybeUninit;
use std::num::NonZeroUsize;

mod aligned;
use aligned::AlignedData;

mod geometry;
pub use geometry::{PlaneGeometry, SubsamplingError};

use crate::pixel::Pixel;

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
/// - Non-empty allocations are aligned to at least 64 bytes on most targets
///   (optimized for SIMD operations)
/// - Non-empty allocations are aligned to at least 8 bytes on `wasm32` targets
///   that are not WASI
/// - If `T` has stricter alignment requirements, the allocation uses
///   `std::mem::align_of::<T>()` instead
///
/// More precisely, non-empty plane data is allocated with
/// `max(DATA_ALIGNMENT, std::mem::align_of::<T>())`, where `DATA_ALIGNMENT` is
/// 64 except on non-WASI `wasm32` targets, where it is 8. Empty planes do not
/// allocate and must not be assumed to have this extra SIMD alignment.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plane<T> {
    /// The underlying pixel data buffer, including padding.
    pub(crate) data: AlignedData<T>,
    /// Geometry information describing dimensions and padding.
    pub(crate) geometry: PlaneGeometry,
}

impl<T> Plane<T> {
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

#[cfg(feature = "padding_api")]
impl<T> Plane<T> {
    /// Creates a new plane with the given [`PlaneGeometry`] over unitialized memory.
    ///
    /// The underlying data needs to be initialized before this plane can be
    /// converted and used in a [`Frame`][crate::frame::Frame].
    ///
    /// [`data_mut`][Plane::data_mut] can be used to initialize the underlying memory.
    ///
    /// # Allocation Alignment
    ///
    /// For non-empty planes, the allocation returned by [`data`][Plane::data]
    /// and [`data_mut`][Plane::data_mut] is aligned to
    /// `max(DATA_ALIGNMENT, std::mem::align_of::<T>())`, where `DATA_ALIGNMENT`
    /// is 64 except on non-WASI `wasm32` targets, where it is 8. This matters
    /// if you use unsafe code to access the backing pointer directly, especially
    /// with over-aligned `T`.
    ///
    /// Empty planes do not allocate and must not be assumed to have the extra
    /// SIMD alignment beyond what Rust requires for an empty `[T]` slice.
    ///
    /// # Example
    ///
    /// ```
    /// use std::num::{NonZeroU8, NonZeroUsize};
    /// use v_frame::plane::{Plane, PlaneGeometry};
    ///
    /// let other_data = vec![42u8; 120 * 80];
    ///
    /// let geometry = PlaneGeometry::unpadded(120, 80, 1, 1)
    ///     .expect("can build geometry");
    ///
    /// let mut plane = Plane::new_uninit(geometry);
    /// assert_eq!(plane.data_mut().len(), other_data.len());
    /// for (dst, src) in plane.data_mut().iter_mut().zip(other_data) {
    ///     dst.write(src);
    /// }
    ///
    /// // SAFETY: All values initialized above
    /// let plane = unsafe { plane.assume_init() };
    /// ```
    #[inline]
    #[must_use]
    pub fn new_uninit(geometry: PlaneGeometry) -> Plane<MaybeUninit<T>> {
        let geometry = geometry
            .normalized()
            .expect("plane geometry dimensions must not overflow");
        let pixels = geometry
            .allocation_len()
            .expect("plane allocation size must not overflow usize");

        Plane {
            data: AlignedData::new_uninit(pixels),
            geometry,
        }
    }

    /// Returns the geometry of the current plane.
    ///
    /// This is a low-level API intended only for functions that require access to the padding.
    #[inline]
    #[must_use]
    pub fn geometry(&self) -> PlaneGeometry {
        self.geometry
    }

    /// Returns a reference to the current plane's data, including padding.
    ///
    /// This is a low-level API intended only for functions that require access to the padding.
    #[inline]
    #[must_use]
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Returns a mutable reference to the current plane's data, including padding.
    ///
    /// This is a low-level API intended only for functions that require access to the padding.
    #[inline]
    #[must_use]
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

#[cfg(feature = "padding_api")]
impl<T> Plane<MaybeUninit<T>> {
    /// Converts to `Plane<T>`.
    ///
    /// # Safety
    /// It is up to the caller to ensure that all contained values are
    /// initialized properly (see [`MaybeUninit::assume_init`]).
    #[inline]
    #[must_use]
    pub unsafe fn assume_init(self) -> Plane<T> {
        // SAFETY: Safety invariants are upheld by the caller.
        let data = unsafe { self.data.assume_init() };

        Plane {
            data,
            geometry: self.geometry,
        }
    }
}

impl<T: Pixel> Plane<T> {
    /// Creates a new plane with the given geometry, initialized with zero-valued pixels.
    pub(crate) fn new(geometry: PlaneGeometry) -> Self {
        let geometry = geometry
            .normalized()
            .expect("plane geometry dimensions must not overflow");
        let pixels = geometry
            .allocation_len()
            .expect("plane allocation size must not overflow usize");

        Self {
            data: AlignedData::new(pixels),
            geometry,
        }
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
    #[must_use]
    pub fn rows(&self) -> impl DoubleEndedIterator<Item = &[T]> + ExactSizeIterator {
        self.data
            .chunks_exact(self.geometry.stride.get())
            .skip(self.geometry.pad_top)
            .take(self.geometry.height.get())
            .map(|row| {
                let start_idx = self.geometry.pad_left;
                let end_idx = start_idx + self.geometry.width.get();
                // SAFETY: The plane creation interface ensures the data is large enough
                unsafe { row.get_unchecked(start_idx..end_idx) }
            })
    }

    /// Returns a mutable iterator over the visible pixels of each row
    /// in the plane, from top to bottom.
    #[inline]
    pub fn rows_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut [T]> + ExactSizeIterator {
        self.data
            .chunks_exact_mut(self.geometry.stride.get())
            .skip(self.geometry.pad_top)
            .take(self.geometry.height.get())
            .map(|row| {
                let start_idx = self.geometry.pad_left;
                let end_idx = start_idx + self.geometry.width.get();
                // SAFETY: The plane creation interface ensures the data is large enough
                unsafe { row.get_unchecked_mut(start_idx..end_idx) }
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
    #[must_use]
    pub fn pixels(&self) -> impl DoubleEndedIterator<Item = T> + ExactSizeIterator {
        let total = self.width().get() * self.height().get();
        ExactSizeWrapper {
            iter: self.rows().flatten().copied(),
            len: total,
        }
    }

    /// Returns a mutable iterator over the visible pixels in the plane,
    /// in row-major order.
    #[inline]
    pub fn pixels_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + ExactSizeIterator {
        let total = self.width().get() * self.height().get();
        ExactSizeWrapper {
            iter: self.rows_mut().flatten(),
            len: total,
        }
    }

    /// Returns an iterator over the visible byte data in the plane,
    /// in row-major order. High-bit-depth data is converted to `u8`
    /// using low endianness.
    #[inline]
    #[must_use]
    pub fn byte_data(&self) -> impl DoubleEndedIterator<Item = u8> + ExactSizeIterator {
        let byte_width = size_of::<T>();
        assert!(
            byte_width <= 2,
            "unsupported pixel byte width: {byte_width}"
        );

        let total = self.width().get() * self.height().get() * byte_width;
        ExactSizeWrapper {
            iter: self.pixels().flat_map(move |pix| {
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
            }),
            len: total,
        }
    }

    /// Copies the data from `src` into this plane's visible pixels.
    ///
    /// # Errors
    /// - Returns `CopyError::DataLength` if the length of `src` does not match
    ///   this plane's `width * height`
    #[inline]
    pub fn copy_from_slice(&mut self, src: &[T]) -> Result<(), CopyError> {
        let width = self.width().get();
        let pixel_count = width * self.height().get();
        if pixel_count != src.len() {
            return Err(CopyError::DataLength {
                expected: pixel_count,
                found: src.len(),
            });
        }

        for (dst_row, src_row) in self.rows_mut().zip(src.chunks_exact(width)) {
            dst_row.copy_from_slice(src_row);
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
    /// - Returns `CopyError::DataLength` if the length of `src` does not match
    ///   this plane's `width * height * bytes_per_pixel`
    #[inline]
    pub fn copy_from_u8_slice(&mut self, src: &[u8]) -> Result<(), CopyError> {
        self.copy_from_u8_slice_with_stride(src, self.width().get() * size_of::<T>())
    }

    /// Copies the data from `src` into this plane's visible pixels.
    /// This version accepts inputs where the row stride is longer than the visible data width.
    /// The `input_stride` must be in bytes.
    ///
    /// # Errors
    /// - Returns `CopyError::DataLength` if the length of `src` does not match
    ///   this plane's `width * height * bytes_per_pixel`
    /// - Returns `CopyError::InvalidStride` if the stride is shorter than the visible width
    #[inline]
    pub fn copy_from_u8_slice_with_stride(
        &mut self,
        src: &[u8],
        stride: usize,
    ) -> Result<(), CopyError> {
        let byte_width = size_of::<T>();
        assert!(
            byte_width <= 2,
            "unsupported pixel byte width: {byte_width}"
        );

        if stride < self.width().get() {
            return Err(CopyError::InvalidStride {
                stride,
                width: self.width().get(),
            });
        }

        let byte_count = stride * self.height().get();
        if byte_count != src.len() {
            return Err(CopyError::DataLength {
                expected: byte_count,
                found: src.len(),
            });
        }

        let width = self.width().get();
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
                let src_offset = row_idx * stride;
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
}

/// Wrapper to add `ExactSizeIterator` implementation to iterators with known length.
struct ExactSizeWrapper<I> {
    iter: I,
    len: usize,
}

impl<I: Iterator> Iterator for ExactSizeWrapper<I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.iter.next() {
            self.len = self.len.saturating_sub(1);
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for ExactSizeWrapper<I> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.iter.next_back() {
            self.len = self.len.saturating_sub(1);
            Some(item)
        } else {
            None
        }
    }
}

impl<I: Iterator> ExactSizeIterator for ExactSizeWrapper<I> {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}
