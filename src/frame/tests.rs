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
fn basic_8bit_frame() {
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
        .build::<u8>()
        .unwrap();

    assert_eq!(frame.y_plane.width().get(), 1920);
    assert_eq!(frame.y_plane.height().get(), 1080);
    assert_eq!(frame.bit_depth.get(), 8);
    assert_eq!(frame.subsampling, ChromaSubsampling::Yuv420);
}

#[test]
fn basic_10bit_frame() {
    let width = NonZeroUsize::new(3840).unwrap();
    let height = NonZeroUsize::new(2160).unwrap();
    let bit_depth = NonZeroU8::new(10).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
        .build::<u16>()
        .unwrap();

    assert_eq!(frame.y_plane.width().get(), 3840);
    assert_eq!(frame.y_plane.height().get(), 2160);
    assert_eq!(frame.bit_depth.get(), 10);
}

#[test]
fn yuv420_chroma_dimensions() {
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
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
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv422, bit_depth)
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
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv444, bit_depth)
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
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Monochrome, bit_depth)
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
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(7).unwrap();

    let result =
        FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth).build::<u8>();

    assert!(matches!(
        result,
        Err(Error::UnsupportedBitDepth { found: 7 })
    ));
}

#[test]
fn unsupported_bit_depth_too_high() {
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(17).unwrap();

    let result =
        FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth).build::<u16>();

    assert!(matches!(
        result,
        Err(Error::UnsupportedBitDepth { found: 17 })
    ));
}

#[test]
fn data_type_mismatch_u8_with_10bit() {
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(10).unwrap();

    let result =
        FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth).build::<u8>();

    assert!(matches!(result, Err(Error::DataTypeMismatch)));
}

#[test]
fn data_type_mismatch_u16_with_8bit() {
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let result =
        FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth).build::<u16>();

    assert!(matches!(result, Err(Error::DataTypeMismatch)));
}

#[test]
fn yuv420_odd_width_resolution_error() {
    let width = NonZeroUsize::new(1921).unwrap(); // odd width
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let result =
        FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth).build::<u8>();

    assert!(matches!(result, Err(Error::UnsupportedResolution)));
}

#[test]
fn yuv420_odd_height_resolution_error() {
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1081).unwrap(); // odd height
    let bit_depth = NonZeroU8::new(8).unwrap();

    let result =
        FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth).build::<u8>();

    assert!(matches!(result, Err(Error::UnsupportedResolution)));
}

#[test]
fn yuv422_odd_width_resolution_error() {
    let width = NonZeroUsize::new(1921).unwrap(); // odd width
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let result =
        FrameBuilder::new(width, height, ChromaSubsampling::Yuv422, bit_depth).build::<u8>();

    assert!(matches!(result, Err(Error::UnsupportedResolution)));
}

#[test]
fn frame_with_luma_padding() {
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
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
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
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
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let result = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
        // Not divisible by 2 for YUV420
        .luma_padding_left(15)
        .build::<u8>();

    assert!(matches!(result, Err(Error::UnsupportedResolution)));
}

#[test]
fn padding_not_aligned_to_subsampling_yuv422() {
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let result = FrameBuilder::new(width, height, ChromaSubsampling::Yuv422, bit_depth)
        // Not divisible by 2 for YUV422
        .luma_padding_left(15)
        .build::<u8>();

    assert!(matches!(result, Err(Error::UnsupportedResolution)));
}

#[test]
fn yuv444_padding_any_value() {
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let result = FrameBuilder::new(width, height, ChromaSubsampling::Yuv444, bit_depth)
        // Any value works for YUV444
        .luma_padding_left(15)
        .build::<u8>();

    assert!(result.is_ok());
}

#[test]
fn monochrome_padding_any_value() {
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let result = FrameBuilder::new(width, height, ChromaSubsampling::Monochrome, bit_depth)
        .luma_padding_left(15)
        .luma_padding_right(17)
        .luma_padding_top(13)
        .luma_padding_bottom(19)
        .build::<u8>();

    assert!(result.is_ok());
}

#[test]
fn frame_clone() {
    let width = NonZeroUsize::new(640).unwrap();
    let height = NonZeroUsize::new(480).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
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
    let width = NonZeroUsize::new(640).unwrap();
    let height = NonZeroUsize::new(480).unwrap();

    // Test 8-bit with u8
    let bit_depth_8 = NonZeroU8::new(8).unwrap();
    assert!(
        FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth_8)
            .build::<u8>()
            .is_ok()
    );

    // Test 9-16 bit with u16
    for depth in 9..=16 {
        let bit_depth = NonZeroU8::new(depth).unwrap();
        assert!(
            FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
                .build::<u16>()
                .is_ok()
        );
    }
}

#[test]
fn small_resolution() {
    let width = NonZeroUsize::new(2).unwrap();
    let height = NonZeroUsize::new(2).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
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
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
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
    let width = NonZeroUsize::new(1920).unwrap();
    let height = NonZeroUsize::new(1080).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
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
