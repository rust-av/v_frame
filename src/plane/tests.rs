// Copyright (c) 2017-2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

#![allow(clippy::unwrap_used, reason = "test file")]

use super::*;
use std::num::NonZeroUsize;

/// Helper function to create a simple plane geometry without padding
fn simple_geometry(width: usize, height: usize) -> PlaneGeometry {
    let width = NonZeroUsize::new(width).unwrap();
    let height = NonZeroUsize::new(height).unwrap();
    PlaneGeometry {
        width,
        height,
        stride: width,
        pad_left: 0,
        pad_right: 0,
        pad_top: 0,
        pad_bottom: 0,
    }
}

/// Helper function to create a plane geometry with padding
fn padded_geometry(
    width: usize,
    height: usize,
    pad_left: usize,
    pad_right: usize,
    pad_top: usize,
    pad_bottom: usize,
) -> PlaneGeometry {
    let width = NonZeroUsize::new(width).unwrap();
    let height = NonZeroUsize::new(height).unwrap();
    let stride = NonZeroUsize::new(width.get() + pad_left + pad_right).unwrap();
    PlaneGeometry {
        width,
        height,
        stride,
        pad_left,
        pad_right,
        pad_top,
        pad_bottom,
    }
}

#[test]
fn plane_new_u8() {
    let geometry = simple_geometry(4, 4);
    let plane: Plane<u8> = Plane::new(geometry);

    assert_eq!(plane.width().get(), 4);
    assert_eq!(plane.height().get(), 4);

    // All pixels should be initialized to zero
    for pixel in plane.pixels() {
        assert_eq!(pixel, 0);
    }
}

#[test]
fn plane_new_u16() {
    let geometry = simple_geometry(8, 8);
    let plane: Plane<u16> = Plane::new(geometry);

    assert_eq!(plane.width().get(), 8);
    assert_eq!(plane.height().get(), 8);

    // All pixels should be initialized to zero
    for pixel in plane.pixels() {
        assert_eq!(pixel, 0);
    }
}

#[test]
fn plane_dimensions() {
    let geometry = simple_geometry(16, 9);
    let plane: Plane<u8> = Plane::new(geometry);

    assert_eq!(plane.width().get(), 16);
    assert_eq!(plane.height().get(), 9);
}

#[test]
fn row_access() {
    let geometry = simple_geometry(4, 3);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Modify the second row
    if let Some(row) = plane.row_mut(1) {
        for (i, pixel) in row.iter_mut().enumerate() {
            *pixel = i as u8;
        }
    }

    // Verify the second row
    let row = plane.row(1).unwrap();
    assert_eq!(row, &[0, 1, 2, 3]);

    // Verify other rows are still zero
    assert_eq!(plane.row(0).unwrap(), &[0, 0, 0, 0]);
    assert_eq!(plane.row(2).unwrap(), &[0, 0, 0, 0]);
}

#[test]
fn row_out_of_bounds() {
    let geometry = simple_geometry(4, 3);
    let plane: Plane<u8> = Plane::new(geometry);

    assert!(plane.row(3).is_none());
    assert!(plane.row(100).is_none());
}

#[test]
fn rows_iterator() {
    let geometry = simple_geometry(3, 4);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Fill each row with its index
    for (y, row) in plane.rows_mut().enumerate() {
        for pixel in row {
            *pixel = y as u8;
        }
    }

    // Verify using rows iterator
    for (y, row) in plane.rows().enumerate() {
        for pixel in row {
            assert_eq!(*pixel, y as u8);
        }
    }
}

#[test]
fn pixel_access() {
    let geometry = simple_geometry(4, 4);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Set some pixels
    *plane.pixel_mut(0, 0).unwrap() = 10;
    *plane.pixel_mut(2, 1).unwrap() = 20;
    *plane.pixel_mut(3, 3).unwrap() = 30;

    // Read them back
    assert_eq!(plane.pixel(0, 0).unwrap(), 10);
    assert_eq!(plane.pixel(2, 1).unwrap(), 20);
    assert_eq!(plane.pixel(3, 3).unwrap(), 30);
    assert_eq!(plane.pixel(1, 1).unwrap(), 0);
}

#[test]
fn pixel_out_of_bounds() {
    let geometry = simple_geometry(4, 4);
    let plane: Plane<u8> = Plane::new(geometry);

    // pixel() checks against data buffer bounds, not just visible area
    // With 4x4 plane (stride=4, height=4), valid indices are 0-15
    // pixel(x, y) calculates: data_origin() + stride * y + x

    // These should be out of bounds:
    assert!(plane.pixel(0, 4).is_none()); // y=4 is past height
    assert!(plane.pixel(100, 100).is_none()); // way out of bounds
    assert!(plane.pixel(16, 0).is_none()); // x beyond any valid row

    // Note: pixel(4, 0) would calculate index 4, which is in the data buffer
    // (it's the start of row 1), so it returns Some even though x >= width.
    // The visible-area constraint is enforced by the rows()/pixels() iterators,
    // not by pixel() which is a lower-level accessor.
}

#[test]
fn pixels_iterator() {
    let geometry = simple_geometry(2, 3);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Fill with sequential values
    for (i, pixel) in plane.pixels_mut().enumerate() {
        *pixel = i as u8;
    }

    // Verify
    let expected = vec![0, 1, 2, 3, 4, 5];
    let actual: Vec<u8> = plane.pixels().collect();
    assert_eq!(actual, expected);
}

#[test]
fn byte_data_u8() {
    let geometry = simple_geometry(2, 2);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Fill with test data
    for (i, pixel) in plane.pixels_mut().enumerate() {
        *pixel = (i + 1) as u8;
    }

    let bytes: Vec<u8> = plane.byte_data().collect();
    assert_eq!(bytes, vec![1, 2, 3, 4]);
}

#[test]
fn byte_data_u16() {
    let geometry = simple_geometry(2, 2);
    let mut plane: Plane<u16> = Plane::new(geometry);

    // Fill with test data (values larger than u8 range)
    *plane.pixel_mut(0, 0).unwrap() = 0x0102;
    *plane.pixel_mut(1, 0).unwrap() = 0x0304;
    *plane.pixel_mut(0, 1).unwrap() = 0x0506;
    *plane.pixel_mut(1, 1).unwrap() = 0x0708;

    let bytes: Vec<u8> = plane.byte_data().collect();
    // Little endian encoding
    assert_eq!(bytes, vec![0x02, 0x01, 0x04, 0x03, 0x06, 0x05, 0x08, 0x07]);
}

#[test]
fn copy_from_slice_u8() {
    let geometry = simple_geometry(3, 2);
    let mut plane: Plane<u8> = Plane::new(geometry);

    let data = vec![1, 2, 3, 4, 5, 6];
    plane.copy_from_slice(&data).unwrap();

    let result: Vec<u8> = plane.pixels().collect();
    assert_eq!(result, data);
}

#[test]
fn copy_from_slice_u16() {
    let geometry = simple_geometry(2, 2);
    let mut plane: Plane<u16> = Plane::new(geometry);

    let data = vec![100, 200, 300, 400];
    plane.copy_from_slice(&data).unwrap();

    let result: Vec<u16> = plane.pixels().collect();
    assert_eq!(result, data);
}

#[test]
fn copy_from_slice_wrong_length() {
    let geometry = simple_geometry(3, 2);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Too short
    let data = vec![1, 2, 3];
    let result = plane.copy_from_slice(&data);
    assert!(matches!(result, Err(Error::DataLength { .. })));

    // Too long
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let result = plane.copy_from_slice(&data);
    assert!(matches!(result, Err(Error::DataLength { .. })));
}

#[test]
fn copy_from_u8_slice_u8() {
    let geometry = simple_geometry(3, 2);
    let mut plane: Plane<u8> = Plane::new(geometry);

    let data = vec![10, 20, 30, 40, 50, 60];
    plane.copy_from_u8_slice(&data).unwrap();

    let result: Vec<u8> = plane.pixels().collect();
    assert_eq!(result, data);
}

#[test]
fn copy_from_u8_slice_u16() {
    let geometry = simple_geometry(2, 2);
    let mut plane: Plane<u16> = Plane::new(geometry);

    // Little endian u16 values: 0x0102, 0x0304, 0x0506, 0x0708
    let data = vec![0x02, 0x01, 0x04, 0x03, 0x06, 0x05, 0x08, 0x07];
    plane.copy_from_u8_slice(&data).unwrap();

    assert_eq!(plane.pixel(0, 0).unwrap(), 0x0102);
    assert_eq!(plane.pixel(1, 0).unwrap(), 0x0304);
    assert_eq!(plane.pixel(0, 1).unwrap(), 0x0506);
    assert_eq!(plane.pixel(1, 1).unwrap(), 0x0708);
}

#[test]
fn copy_from_u8_slice_wrong_length() {
    let geometry = simple_geometry(2, 2);
    let mut plane: Plane<u16> = Plane::new(geometry);

    // Should need 8 bytes (4 pixels * 2 bytes each)
    let data = vec![1, 2, 3, 4]; // Only 4 bytes
    let result = plane.copy_from_u8_slice(&data);
    assert!(matches!(result, Err(Error::DataLength { .. })));
}

#[test]
fn plane_with_padding() {
    let geometry = padded_geometry(4, 3, 2, 2, 1, 1);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Width and height should reflect visible area
    assert_eq!(plane.width().get(), 4);
    assert_eq!(plane.height().get(), 3);

    // Fill visible pixels
    for (i, pixel) in plane.pixels_mut().enumerate() {
        *pixel = i as u8;
    }

    // Verify visible pixels
    let expected: Vec<u8> = (0..12).collect();
    let actual: Vec<u8> = plane.pixels().collect();
    assert_eq!(actual, expected);

    // Verify rows work correctly with padding
    assert_eq!(plane.row(0).unwrap().len(), 4);
    assert_eq!(plane.row(1).unwrap().len(), 4);
    assert_eq!(plane.row(2).unwrap().len(), 4);
}

#[test]
fn plane_clone() {
    let geometry = simple_geometry(3, 3);
    let mut plane1: Plane<u8> = Plane::new(geometry);

    // Fill with test data
    for (i, pixel) in plane1.pixels_mut().enumerate() {
        *pixel = i as u8;
    }

    // Clone the plane
    let plane2 = plane1.clone();

    // Verify clone has same data
    let data1: Vec<u8> = plane1.pixels().collect();
    let data2: Vec<u8> = plane2.pixels().collect();
    assert_eq!(data1, data2);

    // Modify original
    *plane1.pixel_mut(0, 0).unwrap() = 100;

    // Clone should be unchanged
    assert_eq!(plane1.pixel(0, 0).unwrap(), 100);
    assert_eq!(plane2.pixel(0, 0).unwrap(), 0);
}

#[test]
fn data_origin_no_padding() {
    let geometry = simple_geometry(4, 4);
    let plane: Plane<u8> = Plane::new(geometry);

    assert_eq!(plane.data_origin(), 0);
}

#[test]
fn data_origin_with_padding() {
    let geometry = padded_geometry(4, 3, 2, 2, 1, 1);
    let plane: Plane<u8> = Plane::new(geometry);

    // Origin should be: stride * pad_top + pad_left = 8 * 1 + 2 = 10
    assert_eq!(plane.data_origin(), 10);
}

#[cfg(feature = "padding_api")]
#[test]
fn padding_api_geometry() {
    let geometry = padded_geometry(4, 3, 1, 2, 1, 2);
    let plane: Plane<u8> = Plane::new(geometry);

    let retrieved = plane.geometry();
    assert_eq!(retrieved, geometry);
}

#[cfg(feature = "padding_api")]
#[test]
fn padding_api_data_access() {
    let geometry = padded_geometry(2, 2, 1, 1, 1, 1);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Total data size should be stride * (height + pad_top + pad_bottom)
    // stride = 4, total_height = 4, so 16 pixels
    assert_eq!(plane.data().len(), 16);
    assert_eq!(plane.data_mut().len(), 16);

    // Fill all data including padding
    for (i, pixel) in plane.data_mut().iter_mut().enumerate() {
        *pixel = i as u8;
    }

    // Verify we can read it back
    for (i, &pixel) in plane.data().iter().enumerate() {
        assert_eq!(pixel, i as u8);
    }
}

#[test]
fn copy_from_u8_slice_with_stride_u8() {
    let geometry = simple_geometry(3, 2);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Input has stride of 5, but plane width is 3
    // Row 0: [10, 20, 30, PAD, PAD]
    // Row 1: [40, 50, 60, PAD, PAD]
    let data = vec![10, 20, 30, 99, 99, 40, 50, 60, 99, 99];
    let stride = NonZeroUsize::new(5).unwrap();

    plane.copy_from_u8_slice_with_stride(&data, stride).unwrap();

    // Verify only the visible pixels were copied (padding should be ignored)
    assert_eq!(plane.pixel(0, 0).unwrap(), 10);
    assert_eq!(plane.pixel(1, 0).unwrap(), 20);
    assert_eq!(plane.pixel(2, 0).unwrap(), 30);
    assert_eq!(plane.pixel(0, 1).unwrap(), 40);
    assert_eq!(plane.pixel(1, 1).unwrap(), 50);
    assert_eq!(plane.pixel(2, 1).unwrap(), 60);

    let result: Vec<u8> = plane.pixels().collect();
    assert_eq!(result, vec![10, 20, 30, 40, 50, 60]);
}

#[test]
fn copy_from_u8_slice_with_stride_u16() {
    let geometry = simple_geometry(2, 2);
    let mut plane: Plane<u16> = Plane::new(geometry);

    // Input has stride of 3 pixels (6 bytes per row), but plane width is 2
    // Row 0: [0x0102, 0x0304, PAD]
    // Row 1: [0x0506, 0x0708, PAD]
    #[rustfmt::skip]
    let data = vec![
        0x02, 0x01, 0x04, 0x03, 0xFF, 0xFF,  // Row 0
        0x06, 0x05, 0x08, 0x07, 0xFF, 0xFF,  // Row 1
    ];
    let stride = NonZeroUsize::new(3).unwrap();

    plane.copy_from_u8_slice_with_stride(&data, stride).unwrap();

    // Verify only the visible pixels were copied
    assert_eq!(plane.pixel(0, 0).unwrap(), 0x0102);
    assert_eq!(plane.pixel(1, 0).unwrap(), 0x0304);
    assert_eq!(plane.pixel(0, 1).unwrap(), 0x0506);
    assert_eq!(plane.pixel(1, 1).unwrap(), 0x0708);
}

#[test]
fn copy_from_u8_slice_with_stride_equal_width() {
    let geometry = simple_geometry(3, 2);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // When stride == width, should delegate to copy_from_u8_slice
    let data = vec![1, 2, 3, 4, 5, 6];
    let stride = NonZeroUsize::new(3).unwrap();

    plane.copy_from_u8_slice_with_stride(&data, stride).unwrap();

    let result: Vec<u8> = plane.pixels().collect();
    assert_eq!(result, data);
}

#[test]
fn copy_from_u8_slice_with_stride_invalid_stride() {
    let geometry = simple_geometry(5, 2);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Stride smaller than width is invalid
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let stride = NonZeroUsize::new(4).unwrap();

    let result = plane.copy_from_u8_slice_with_stride(&data, stride);
    assert!(matches!(result, Err(Error::InvalidStride { .. })));
}

#[test]
fn copy_from_u8_slice_with_stride_wrong_length() {
    let geometry = simple_geometry(2, 2);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Should need stride * height = 3 * 2 = 6 bytes
    let data = vec![1, 2, 3, 4]; // Only 4 bytes
    let stride = NonZeroUsize::new(3).unwrap();

    let result = plane.copy_from_u8_slice_with_stride(&data, stride);
    assert!(matches!(result, Err(Error::DataLength { .. })));

    // u16 case: should need stride * height * 2 bytes
    let geometry = simple_geometry(2, 2);
    let mut plane: Plane<u16> = Plane::new(geometry);

    // Should need 3 * 2 * 2 = 12 bytes
    let data = vec![1, 2, 3, 4, 5, 6]; // Only 6 bytes
    let stride = NonZeroUsize::new(3).unwrap();

    let result = plane.copy_from_u8_slice_with_stride(&data, stride);
    assert!(matches!(result, Err(Error::DataLength { .. })));
}

#[test]
fn large_plane() {
    let geometry = simple_geometry(1920, 1080);
    let plane: Plane<u8> = Plane::new(geometry);

    assert_eq!(plane.width().get(), 1920);
    assert_eq!(plane.height().get(), 1080);
    assert_eq!(plane.pixels().count(), 1920 * 1080);
}

#[test]
fn row_mutation_isolation() {
    let geometry = simple_geometry(4, 4);
    let mut plane: Plane<u8> = Plane::new(geometry);

    // Modify row 1
    if let Some(row) = plane.row_mut(1) {
        for pixel in row {
            *pixel = 42;
        }
    }

    // Verify only row 1 changed
    assert_eq!(plane.row(0).unwrap(), &[0, 0, 0, 0]);
    assert_eq!(plane.row(1).unwrap(), &[42, 42, 42, 42]);
    assert_eq!(plane.row(2).unwrap(), &[0, 0, 0, 0]);
    assert_eq!(plane.row(3).unwrap(), &[0, 0, 0, 0]);
}

#[test]
fn rows_count() {
    let geometry = simple_geometry(10, 5);
    let plane: Plane<u8> = Plane::new(geometry);

    let row_count = plane.rows().count();
    assert_eq!(row_count, 5);
}

#[test]
fn pixels_count() {
    let geometry = simple_geometry(7, 11);
    let plane: Plane<u8> = Plane::new(geometry);

    let pixel_count = plane.pixels().count();
    assert_eq!(pixel_count, 7 * 11);
}
