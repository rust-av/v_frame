// Copyright (c) 2018-2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

#![allow(clippy::unwrap_used, reason = "test file")]

use super::*;
use crate::chroma::ChromaSubsampling;

#[test]
fn plane_access() {
    let mut frame = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 8)
        .build::<u8>()
        .unwrap();

    let y_ptr = frame.y_plane.data.as_ptr();
    assert_eq!(y_ptr, frame.plane(0).expect("plane 0 exists").data.as_ptr());
    assert_eq!(
        y_ptr,
        frame.plane_mut(0).expect("plane 0 exists").data.as_ptr()
    );

    let u_ptr = frame
        .u_plane
        .as_ref()
        .expect("plane 1 exists")
        .data
        .as_ptr();
    assert_eq!(u_ptr, frame.plane(1).expect("plane 1 exists").data.as_ptr());
    assert_eq!(
        u_ptr,
        frame.plane_mut(1).expect("plane 1 exists").data.as_ptr()
    );

    let v_ptr = frame
        .v_plane
        .as_ref()
        .expect("plane 2 exists")
        .data
        .as_ptr();
    assert_eq!(v_ptr, frame.plane(2).expect("plane 2 exists").data.as_ptr());
    assert_eq!(
        v_ptr,
        frame.plane_mut(2).expect("plane 2 exists").data.as_ptr()
    );
}

#[test]
fn basic_8bit_frame() {
    let frame = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 8)
        .build::<u8>()
        .unwrap();

    assert_eq!(frame.y_plane.width().get(), 1920);
    assert_eq!(frame.y_plane.height().get(), 1080);
    assert_eq!(frame.bit_depth.get(), 8);
    assert_eq!(frame.subsampling, ChromaSubsampling::Yuv420);
}

#[test]
fn basic_10bit_frame() {
    let frame = FrameBuilder::new(3840, 2160, ChromaSubsampling::Yuv420, 10)
        .build::<u16>()
        .unwrap();

    assert_eq!(frame.y_plane.width().get(), 3840);
    assert_eq!(frame.y_plane.height().get(), 2160);
    assert_eq!(frame.bit_depth.get(), 10);
}

#[test]
fn yuv420_chroma_dimensions() {
    let frame = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 8)
        .build::<u8>()
        .unwrap();

    let u_plane = frame.u_plane.as_ref().unwrap();
    let v_plane = frame.v_plane.as_ref().unwrap();

    // YUV420 has chroma planes at half width and half height
    assert_eq!(u_plane.width().get(), 960);
    assert_eq!(u_plane.height().get(), 540);
    assert_eq!(v_plane.width().get(), 960);
    assert_eq!(v_plane.height().get(), 540);
}

#[test]
fn yuv422_chroma_dimensions() {
    let frame = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv422, 8)
        .build::<u8>()
        .unwrap();

    let u_plane = frame.u_plane.as_ref().unwrap();
    let v_plane = frame.v_plane.as_ref().unwrap();

    // YUV422 has chroma planes at half width and full height
    assert_eq!(u_plane.width().get(), 960);
    assert_eq!(u_plane.height().get(), 1080);
    assert_eq!(v_plane.width().get(), 960);
    assert_eq!(v_plane.height().get(), 1080);
}

#[test]
fn yuv444_chroma_dimensions() {
    let frame = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv444, 8)
        .build::<u8>()
        .unwrap();

    let u_plane = frame.u_plane.as_ref().unwrap();
    let v_plane = frame.v_plane.as_ref().unwrap();

    // YUV444 has chroma planes at full resolution
    assert_eq!(u_plane.width().get(), 1920);
    assert_eq!(u_plane.height().get(), 1080);
    assert_eq!(v_plane.width().get(), 1920);
    assert_eq!(v_plane.height().get(), 1080);
}

#[test]
fn monochrome_no_chroma_planes() {
    let frame = FrameBuilder::new(1920, 1080, ChromaSubsampling::Monochrome, 8)
        .build::<u8>()
        .unwrap();

    assert_eq!(frame.y_plane.width().get(), 1920);
    assert_eq!(frame.y_plane.height().get(), 1080);
    assert!(frame.u_plane.is_none());
    assert!(frame.v_plane.is_none());
    assert_eq!(frame.subsampling, ChromaSubsampling::Monochrome);
}

#[test]
fn unsupported_bit_depth_too_low() {
    let result = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 7).build::<u8>();

    assert!(matches!(
        result,
        Err(Error::UnsupportedBitDepth { found: 7 })
    ));
}

#[test]
fn unsupported_bit_depth_too_high() {
    let result = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 17).build::<u16>();

    assert!(matches!(
        result,
        Err(Error::UnsupportedBitDepth { found: 17 })
    ));
    assert!(
        format!("{}", result.err().unwrap()).starts_with("only 8-16 bit frame data is supported,")
    );
}

#[test]
fn data_type_mismatch_u8_with_10bit() {
    let result = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 10).build::<u8>();

    assert!(matches!(result, Err(Error::DataTypeMismatch)));
    assert!(format!("{}", result.err().unwrap()).starts_with("bit depth did not match"));
}

#[test]
fn data_type_mismatch_u16_with_8bit() {
    let result = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 8).build::<u16>();

    assert!(matches!(result, Err(Error::DataTypeMismatch)));
}

#[test]
fn yuv420_odd_width_resolution_error() {
    let result = FrameBuilder::new(1921, 1080, ChromaSubsampling::Yuv420, 8).build::<u8>();

    assert!(matches!(result, Err(Error::UnsupportedResolution)));
    assert!(
        format!("{}", result.err().unwrap())
            .starts_with("selected chroma subsampling does not support")
    );
}

#[test]
fn yuv420_odd_height_resolution_error() {
    let result = FrameBuilder::new(1920, 1081, ChromaSubsampling::Yuv420, 8).build::<u8>();

    assert!(matches!(result, Err(Error::UnsupportedResolution)));
}

#[test]
fn yuv422_odd_width_resolution_error() {
    let result = FrameBuilder::new(1921, 1080, ChromaSubsampling::Yuv422, 8).build::<u8>();

    assert!(matches!(result, Err(Error::UnsupportedResolution)));
}

#[test]
fn frame_with_luma_padding() {
    let frame = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 8)
        .luma_padding_left(16)
        .luma_padding_right(16)
        .luma_padding_top(16)
        .luma_padding_bottom(16)
        .build::<u8>()
        .unwrap();

    // Visible dimensions should remain unchanged
    assert_eq!(frame.y_plane.width().get(), 1920);
    assert_eq!(frame.y_plane.height().get(), 1080);
}

#[test]
fn chroma_padding_derived_from_luma_yuv420() {
    let frame = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 8)
        .luma_padding_left(16)
        .luma_padding_right(16)
        .luma_padding_top(16)
        .luma_padding_bottom(16)
        .build::<u8>()
        .unwrap();

    // For YUV420, chroma padding should be half of luma padding
    let u_plane = frame.u_plane.as_ref().unwrap();
    assert_eq!(u_plane.width().get(), 960); // chroma width
    assert_eq!(u_plane.height().get(), 540); // chroma height
}

#[test]
fn padding_not_aligned_to_subsampling_yuv420() {
    let result = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 8)
        // Not divisible by 2 for YUV420
        .luma_padding_left(15)
        .build::<u8>();

    assert!(matches!(result, Err(Error::UnsupportedResolution)));
}

#[test]
fn padding_not_aligned_to_subsampling_yuv422() {
    let result = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv422, 8)
        // Not divisible by 2 for YUV422
        .luma_padding_left(15)
        .build::<u8>();

    assert!(matches!(result, Err(Error::UnsupportedResolution)));
}

#[test]
fn yuv444_padding_any_value() {
    let result = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv444, 8)
        // Any value works for YUV444
        .luma_padding_left(15)
        .build::<u8>();

    assert!(result.is_ok());
}

#[test]
fn monochrome_padding_any_value() {
    let result = FrameBuilder::new(1920, 1080, ChromaSubsampling::Monochrome, 8)
        .luma_padding_left(15)
        .luma_padding_right(17)
        .luma_padding_top(13)
        .luma_padding_bottom(19)
        .build::<u8>();

    assert!(result.is_ok());
}

#[test]
fn frame_clone() {
    let frame = FrameBuilder::new(640, 480, ChromaSubsampling::Yuv420, 8)
        .build::<u8>()
        .unwrap();

    let cloned_frame = frame.clone();

    assert_eq!(cloned_frame.y_plane.width(), frame.y_plane.width());
    assert_eq!(cloned_frame.y_plane.height(), frame.y_plane.height());
    assert_eq!(cloned_frame.bit_depth, frame.bit_depth);
    assert_eq!(cloned_frame.subsampling, frame.subsampling);
}

#[test]
fn all_supported_bit_depths() {
    let width = 640;
    let height = 480;

    // Test 8-bit with u8
    assert!(
        FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, 8)
            .build::<u8>()
            .is_ok()
    );

    // Test 9-16 bit with u16
    for bit_depth in 9..=16 {
        assert!(
            FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
                .build::<u16>()
                .is_ok()
        );
    }
}

#[test]
fn small_resolution() {
    let frame = FrameBuilder::new(2, 2, ChromaSubsampling::Yuv420, 8)
        .build::<u8>()
        .unwrap();

    assert_eq!(frame.y_plane.width().get(), 2);
    assert_eq!(frame.y_plane.height().get(), 2);

    let u_plane = frame.u_plane.as_ref().unwrap();
    assert_eq!(u_plane.width().get(), 1);
    assert_eq!(u_plane.height().get(), 1);
}

#[test]
fn builder_setters() {
    let frame = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 8)
        .luma_padding_left(8)
        .luma_padding_right(8)
        .luma_padding_top(8)
        .luma_padding_bottom(8)
        .build::<u8>()
        .unwrap();
    assert!(frame.y_plane.width().get() == 1920);
}

#[test]
fn asymmetric_padding() {
    let frame = FrameBuilder::new(1920, 1080, ChromaSubsampling::Yuv420, 8)
        .luma_padding_left(8)
        .luma_padding_right(16)
        .luma_padding_top(4)
        .luma_padding_bottom(12)
        .build::<u8>()
        .unwrap();

    // Visible dimensions should remain unchanged
    assert_eq!(frame.y_plane.width().get(), 1920);
    assert_eq!(frame.y_plane.height().get(), 1080);
}
