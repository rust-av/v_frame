// Copyright (c) 2018-2020, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

use crate::math::*;
use crate::pixel::*;
use crate::plane::*;

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Represents a raw video frame
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Frame<T: Pixel> {
    /// Planes constituting the frame.
    pub planes: [Plane<T>; 3],
}

impl<T: Pixel> Frame<T> {
    /// Creates a new frame with the given parameters.
    ///
    /// Allocates data for the planes.
    pub fn new_with_padding(
        width: usize,
        height: usize,
        chroma_sampling: ChromaSampling,
        luma_padding: usize,
    ) -> Self {
        let luma_width = width.align_power_of_two(3);
        let luma_height = height.align_power_of_two(3);

        let (chroma_decimation_x, chroma_decimation_y) =
            chroma_sampling.get_decimation().unwrap_or((0, 0));
        let (chroma_width, chroma_height) =
            chroma_sampling.get_chroma_dimensions(luma_width, luma_height);
        let chroma_padding_x = luma_padding >> chroma_decimation_x;
        let chroma_padding_y = luma_padding >> chroma_decimation_y;

        Frame {
            planes: [
                Plane::new(luma_width, luma_height, 0, 0, luma_padding, luma_padding),
                Plane::new(
                    chroma_width,
                    chroma_height,
                    chroma_decimation_x,
                    chroma_decimation_y,
                    chroma_padding_x,
                    chroma_padding_y,
                ),
                Plane::new(
                    chroma_width,
                    chroma_height,
                    chroma_decimation_x,
                    chroma_decimation_y,
                    chroma_padding_x,
                    chroma_padding_y,
                ),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestPixel = u8;

    fn get_plane_resolution(plane: &Plane<TestPixel>) -> (usize, usize) {
        (plane.cfg.width, plane.cfg.height)
    }

    fn test_frame_resolutions(
        resolution: (usize, usize),
        chroma_sampling: ChromaSampling,
        expected_luma_res: (usize, usize),
        expected_chroma_res: (usize, usize),
        luma_padding: usize,
    ) {
        let (width, height) = resolution;
        let frame =
            Frame::<TestPixel>::new_with_padding(width, height, chroma_sampling, luma_padding);

        assert_eq!(expected_luma_res, get_plane_resolution(&frame.planes[0]));
        assert_eq!(expected_chroma_res, get_plane_resolution(&frame.planes[1]));
        assert_eq!(expected_chroma_res, get_plane_resolution(&frame.planes[2]));
    }

    #[test]
    fn test_1280x720_420_subsampling_no_padding() {
        test_frame_resolutions(
            (1280, 720),
            ChromaSampling::Cs420,
            (1280, 720),
            (640, 360),
            0,
        );
    }

    #[test]
    fn test_1920x1080_420_subsampling_no_padding() {
        test_frame_resolutions(
            (1920, 1080),
            ChromaSampling::Cs420,
            (1920, 1080),
            (960, 540),
            0,
        );
    }

    #[test]
    fn test_1280x720_444_subsampling_no_padding() {
        test_frame_resolutions(
            (1280, 720),
            ChromaSampling::Cs444,
            (1280, 720),
            (1280, 720),
            0,
        );
    }

    #[test]
    fn test_1280x720_444_subsampling_2_padding() {
        test_frame_resolutions(
            (1280, 720),
            ChromaSampling::Cs444,
            (1280, 720),
            (1280, 720),
            2,
        );
    }
}
