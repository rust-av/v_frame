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
//!
//! # Working with `T: Pixel`
//!
//! It is often necessary to convert between an abstract pixel and their concrete
//! integer representation in order to work with them, for example to implement an
//! algorithm operating on video frames.
//!
//! For this purpose, `Pixel` is a supertrait of some conversions from the standard
//! library, namely:
//! - `Into<u16>` to convert any pixel into a `u16`,
//! - `TryInto<u8>` to try to convert any pixel into a `u8` (not possible if `T` is larger
//!   than one byte),
//! - `From<u8>` to convert any `u8` into a Pixel containing the given value and
//! - `TryFrom<u16>` to try to convert any `u16` into a `T` (not possible if `T` is smaller
//!   than two bytes).
//!
//! For example, to sum all pixels in a row:
//!
//! ```
//! use v_frame::frame::Frame;
//! use v_frame::pixel::Pixel;
//!
//! pub fn summed_rows<T: Pixel>(frame: &Frame<T>) -> Vec<u64> {
//!     frame.y_plane
//!         .rows()
//!         // row is &[T] here
//!         .map(|row| {
//!             row.iter()
//!                 .map(|&pix| pix.into())    // convert T -> u16
//!                 .map(|pix| u64::from(pix)) // widen so sum doesn't overflow
//!                 .sum()
//!         })
//!         .collect()
//! }
//! ```
//!
//! Note that Pixel only provides one `Into<T>` implementation so it is not necessary to
//! explicitly specify the wanted type (`<T as Into<u16>>::into(pix)`).
//!
//! Here's a somewhat contrived example of how to convert integers into pixels:
//!
//! ```
//! use v_frame::frame::Frame;
//! use v_frame::pixel::Pixel;
//!
//! fn random_value() -> i32 {
//!     // chosen by a fair dice roll
//!     4
//! }
//!
//! fn clamp_to_range(value: i32, bit_depth: u8) -> u16 {
//!     let bit_depth_max = (1u16 << bit_depth) - 1;
//!
//!     if value < 0 {
//!         0
//!     } else if value > i32::from(bit_depth_max) {
//!         bit_depth_max
//!     } else {
//!         value as u16
//!     }
//! }
//!
//! pub fn change_some_rows<T: Pixel>(frame: &mut Frame<T>, bit_depth: u8) {
//!     for pix in frame.y_plane.pixels_mut() {
//!         let old_val = i32::from((*pix).into());
//!
//!         let new_val = clamp_to_range(old_val + random_value(), bit_depth);
//!         if size_of::<T>() == 1 {
//!             *pix = T::from(new_val as u8)
//!         } else {
//!             *pix = T::try_from(new_val).expect("T is u16");
//!         }
//!     }
//! }
//! ```
//!
//! Again, `From<u8>` and `TryFrom<u16>` are the only implementations for the respective traits
//! so it's not necessary to specify which one is being used.

use core::fmt::{Binary, Debug, Display, LowerExp, LowerHex, Octal, UpperExp, UpperHex};
use core::hash::Hash;

mod private {
    pub trait Sealed {}

    impl Sealed for u8 {}
    impl Sealed for u16 {}
}

/// A trait for types that can be used as pixel data.
///
/// This trait abstracts over the pixel data types supported by the library,
/// currently `u8` for 8-bit data and `u16` for high bit-depth (9-16 bit) data.
///
/// All frame and plane types are generic over `T: Pixel`, allowing the same
/// data structures and algorithms to work with both standard and high bit-depth
/// video content.
///
/// See the [module documentation][crate::pixel] for more details.
///
/// # Type Safety
///
/// The library enforces correct type usage through validation:
/// - Frames with 8-bit depth can only be created with `T = u8`
/// - Frames with 9-16 bit depth can only be created with `T = u16`
///
/// Attempting to create a frame with a mismatched type will result in
/// [`FrameError::DataTypeMismatch`][crate::frame::FrameError::DataTypeMismatch].
///
/// # Safety
///
/// All implementing types must be valid if represented by an all-zero byte-pattern,
/// i.e. using [`std::mem::zeroed`] must __not__ cause undefined behavior for
/// implementing types.
pub unsafe trait Pixel:
    Sized
    + Copy
    + Clone
    // formatting
    + Display
    + Debug
    + Octal
    + LowerHex
    + UpperHex
    + LowerExp
    + UpperExp
    + Binary
    + Default
    // comparisons
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    // conversions
    + TryInto<u8, Error: core::error::Error>
    + Into<u16>
    + From<u8>
    + TryFrom<u16, Error: core::error::Error>
    // markers
    + Send
    + Sync
    // misc.
    + Hash
    + 'static
    + private::Sealed
{
}

/// Pixel implementation for 8-bit video data.
// SAFETY: u8 is valid if represented by a zeroed byte.
unsafe impl Pixel for u8 {}

/// Pixel implementation for high bit-depth (9-16 bit) video data.
// SAFETY: u16 is valid if represented by zeroed bytes.
unsafe impl Pixel for u16 {}
