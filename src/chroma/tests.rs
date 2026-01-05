// Copyright (c) 2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

#![allow(clippy::unwrap_used, reason = "test file")]

use super::*;
use std::num::NonZeroU8;

#[test]
fn has_chroma() {
    // Test that all variants except Monochrome have chroma
    assert!(ChromaSubsampling::Yuv420.has_chroma());
    assert!(ChromaSubsampling::Yuv422.has_chroma());
    assert!(ChromaSubsampling::Yuv444.has_chroma());
    assert!(!ChromaSubsampling::Monochrome.has_chroma());
}

#[test]
fn subsample_ratio() {
    // Test subsampling ratios for each variant
    let yuv420_ratio = ChromaSubsampling::Yuv420.subsample_ratio();
    assert_eq!(
        yuv420_ratio,
        Some((NonZeroU8::new(2).unwrap(), NonZeroU8::new(2).unwrap()))
    );

    let yuv422_ratio = ChromaSubsampling::Yuv422.subsample_ratio();
    assert_eq!(
        yuv422_ratio,
        Some((NonZeroU8::new(2).unwrap(), NonZeroU8::new(1).unwrap()))
    );

    let yuv444_ratio = ChromaSubsampling::Yuv444.subsample_ratio();
    assert_eq!(
        yuv444_ratio,
        Some((NonZeroU8::new(1).unwrap(), NonZeroU8::new(1).unwrap()))
    );

    let monochrome_ratio = ChromaSubsampling::Monochrome.subsample_ratio();
    assert_eq!(monochrome_ratio, None);
}

#[test]
fn chroma_dimensions_valid() {
    // Test valid dimensions for each subsampling type

    // YUV420: half width, half height
    let yuv420_dims = ChromaSubsampling::Yuv420.chroma_dimensions(1920, 1080);
    assert_eq!(yuv420_dims, Some((960, 540)));

    // YUV422: half width, full height
    let yuv422_dims = ChromaSubsampling::Yuv422.chroma_dimensions(1920, 1080);
    assert_eq!(yuv422_dims, Some((960, 1080)));

    // YUV444: full resolution
    let yuv444_dims = ChromaSubsampling::Yuv444.chroma_dimensions(1920, 1080);
    assert_eq!(yuv444_dims, Some((1920, 1080)));

    // Monochrome: no chroma planes
    let mono_dims = ChromaSubsampling::Monochrome.chroma_dimensions(1920, 1080);
    assert_eq!(mono_dims, None);
}

#[test]
fn chroma_dimensions_invalid() {
    // Test invalid dimensions (odd resolutions for YUV420)

    // YUV420 with odd width
    let yuv420_odd_width = ChromaSubsampling::Yuv420.chroma_dimensions(1921, 1080);
    assert_eq!(yuv420_odd_width, None);

    // YUV420 with odd height
    let yuv420_odd_height = ChromaSubsampling::Yuv420.chroma_dimensions(1920, 1081);
    assert_eq!(yuv420_odd_height, None);

    // YUV420 with both odd dimensions
    let yuv420_both_odd = ChromaSubsampling::Yuv420.chroma_dimensions(1921, 1081);
    assert_eq!(yuv420_both_odd, None);

    // YUV422 with odd width (should fail because width must be divisible by 2)
    let yuv422_odd_width = ChromaSubsampling::Yuv422.chroma_dimensions(1921, 1080);
    assert_eq!(yuv422_odd_width, None);
}

#[test]
fn chroma_dimensions_edge_cases() {
    // Test edge cases

    // Minimum valid dimensions for YUV420 (2x2)
    let min_yuv420 = ChromaSubsampling::Yuv420.chroma_dimensions(2, 2);
    assert_eq!(min_yuv420, Some((1, 1)));

    // Minimum valid dimensions for YUV422 (2x1)
    let min_yuv422 = ChromaSubsampling::Yuv422.chroma_dimensions(2, 1);
    assert_eq!(min_yuv422, Some((1, 1)));

    // Minimum valid dimensions for YUV444 (1x1)
    let min_yuv444 = ChromaSubsampling::Yuv444.chroma_dimensions(1, 1);
    assert_eq!(min_yuv444, Some((1, 1)));

    // Zero dimensions should return 0
    let zero_dims = ChromaSubsampling::Yuv420.chroma_dimensions(0, 0);
    assert_eq!(zero_dims, Some((0, 0)));
}

#[test]
fn chroma_dimensions_large_values() {
    // Test with large values to ensure no overflow issues
    let large_yuv420 = ChromaSubsampling::Yuv420.chroma_dimensions(3840, 2160);
    assert_eq!(large_yuv420, Some((1920, 1080)));

    let large_yuv422 = ChromaSubsampling::Yuv422.chroma_dimensions(3840, 2160);
    assert_eq!(large_yuv422, Some((1920, 2160)));

    let large_yuv444 = ChromaSubsampling::Yuv444.chroma_dimensions(3840, 2160);
    assert_eq!(large_yuv444, Some((3840, 2160)));
}
