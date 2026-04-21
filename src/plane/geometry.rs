use core::num::{NonZeroU8, NonZeroUsize};

use crate::chroma::ChromaSubsampling;

/// Describes the geometry of a plane, including dimensions and padding.
///
/// This struct contains all the information needed to interpret the layout of
/// a plane's data buffer, including the visible dimensions and the padding on
/// all four sides.
///
/// The `stride` represents the number of pixels per row in the data buffer,
/// which is equal to `width + pad_left + pad_right`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaneGeometry {
    width: NonZeroUsize,
    height: NonZeroUsize,
    stride: NonZeroUsize,
    pad_left: usize,
    pad_right: usize,
    pad_top: usize,
    pad_bottom: usize,
    subsampling_x: NonZeroU8,
    subsampling_y: NonZeroU8,
}

impl PlaneGeometry {
    /// Returns a new [`PlaneGeometry`] with the given dimensions, padding and subsampling.
    ///
    /// Will return `None` if:
    /// - `width`, `height`, `subsampling_x` or `subsampling_y` are zero or
    /// - the data stride would overflow a `usize`.
    #[inline]
    #[must_use]
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        width: usize,
        height: usize,
        pad_left: usize,
        pad_right: usize,
        pad_top: usize,
        pad_bottom: usize,
        subsampling_x: u8,
        subsampling_y: u8,
    ) -> Option<Self> {
        let width = NonZeroUsize::new(width)?;
        let height = NonZeroUsize::new(height)?;
        let subsampling_x = NonZeroU8::new(subsampling_x)?;
        let subsampling_y = NonZeroU8::new(subsampling_y)?;

        let stride = width.checked_add(pad_left)?.checked_add(pad_right)?;

        Some(Self {
            width,
            height,
            stride,
            pad_left,
            pad_right,
            pad_top,
            pad_bottom,
            subsampling_x,
            subsampling_y,
        })
    }

    /// Returns a new [`PlaneGeometry`] with the given dimensions, subsampling and zero padding.
    ///
    /// Returns `None` if `width`, `height`, `subsampling_x` or `subsampling_y` are zero.
    #[inline]
    #[must_use]
    pub fn unpadded(
        width: usize,
        height: usize,
        subsampling_x: u8,
        subsampling_y: u8,
    ) -> Option<Self> {
        Self::new(width, height, 0, 0, 0, 0, subsampling_x, subsampling_y)
    }

    /// Width of the visible area in pixels.
    ///
    /// Guaranteed to be non-zero.
    #[inline]
    #[must_use]
    pub fn width(&self) -> usize {
        self.width.get()
    }

    /// Height of the visible area in pixels.
    ///
    /// Guaranteed to be non-zero.
    #[inline]
    #[must_use]
    pub fn height(&self) -> usize {
        self.height.get()
    }

    /// Data stride (pixels per row in the buffer, including padding).
    ///
    /// Guaranteed to be non-zero.
    #[inline]
    #[must_use]
    pub fn stride(&self) -> usize {
        self.stride.get()
    }

    /// Number of padding pixels on the left side.
    #[inline]
    #[must_use]
    pub fn pad_left(&self) -> usize {
        self.pad_left
    }

    /// Number of padding pixels on the right side.
    #[inline]
    #[must_use]
    pub fn pad_right(&self) -> usize {
        self.pad_right
    }

    /// Number of padding pixels on the top.
    #[inline]
    #[must_use]
    pub fn pad_top(&self) -> usize {
        self.pad_top
    }

    /// Number of padding pixels on the bottom.
    #[inline]
    #[must_use]
    pub fn pad_bottom(&self) -> usize {
        self.pad_bottom
    }

    /// The horizontal subsampling ratio of this plane compared to the luma plane.
    ///
    /// Guaranteed to be non-zero, and 1 if not subsampled.
    #[inline]
    #[must_use]
    pub fn subsampling_x(&self) -> u8 {
        self.subsampling_x.get()
    }

    /// The horizontal subsampling ratio of this plane compared to the luma plane.
    ///
    /// Guaranteed to be non-zero, and 1 if not subsampled.
    #[inline]
    #[must_use]
    pub fn subsampling_y(&self) -> u8 {
        self.subsampling_y.get()
    }

    /// Returns a new [`PlaneGeometry`] based on `self` and according to `subsampling`.
    ///
    /// Returns:
    /// - `Ok(None)` if the subsampling specifies no other planes (i.e. monochrome),
    /// - `Ok(Some(g))` if `self` was successfully subsampled into a new geometry `g`,
    /// - `Err(..)` if `self` could not be subsampled into a new geometry, for example
    ///   due to `self` having an odd width or height.
    ///
    /// # Errors
    ///
    /// See [`SubsamplingError`].
    #[inline]
    pub fn for_subsampling(
        self,
        subsampling: ChromaSubsampling,
    ) -> Result<Option<Self>, SubsamplingError> {
        match subsampling {
            ChromaSubsampling::Monochrome => Ok(None),
            ChromaSubsampling::Yuv444 => Ok(Some(self)),
            ChromaSubsampling::Yuv420 => self.subsampled::<2, 2>().map(Some),
            ChromaSubsampling::Yuv422 => self.subsampled::<2, 1>().map(Some),
        }
    }

    #[inline]
    fn subsampled<const X: usize, const Y: usize>(self) -> Result<Self, SubsamplingError> {
        let x = const { NonZeroUsize::new(X).expect("X nonzero") };
        let y = const { NonZeroUsize::new(Y).expect("Y nonzero") };

        if self.width.get() % x != 0
            || self.height.get() % y != 0
            || self.pad_left % x != 0
            || self.pad_right % x != 0
            || self.pad_top % y != 0
            || self.pad_bottom % y != 0
        {
            return Err(SubsamplingError);
        }

        // X and Y must fit into the subsampling_* fields
        const { assert!(X <= u8::MAX as usize && Y <= u8::MAX as usize) };

        Self::new(
            self.width.get() / x,
            self.height.get() / y,
            self.pad_left / x,
            self.pad_right / x,
            self.pad_top / y,
            self.pad_bottom / y,
            X as u8,
            Y as u8,
        )
        .ok_or(SubsamplingError)
    }

    /// Returns the index for the first visible pixel.
    #[inline]
    #[must_use]
    pub fn data_origin(&self) -> usize {
        self.stride() * self.pad_top + self.pad_left
    }

    /// Returns the total height of the plane, including padding.
    #[inline]
    #[must_use]
    pub fn alloc_height(&self) -> usize {
        self.height() + self.pad_top + self.pad_bottom
    }

    /// Returns the total allocation size of the plane, including padding.
    #[inline]
    #[must_use]
    pub fn alloc_size(&self) -> usize {
        self.alloc_height() * self.stride()
    }
}

/// An error occurred when trying to create a subsampled geometry.
///
/// This usually means that there was an odd number of pixels in one of the dimensions
/// that was being subsampled:
/// - width, horizontal padding for [`ChromaSubsampling::Yuv422`] or [`ChromaSubsampling::Yuv420`]
/// - height, vertical padding for [`ChromaSubsampling::Yuv420`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubsamplingError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpadded() {
        assert!(PlaneGeometry::unpadded(0, 1080, 1, 1).is_none());
        assert!(PlaneGeometry::unpadded(1920, 0, 1, 1).is_none());
        assert!(PlaneGeometry::unpadded(1920, 1080, 0, 1).is_none());
        assert!(PlaneGeometry::unpadded(1920, 1080, 1, 0).is_none());

        let g = PlaneGeometry::unpadded(1920, 1080, 1, 1).expect("unpadded geometry works");
        assert_eq!(g.width(), 1920);
        assert_eq!(g.height(), 1080);
        assert_eq!(g.stride, g.width);
        assert_eq!(g.pad_left(), 0);
        assert_eq!(g.pad_right(), 0);
        assert_eq!(g.pad_top(), 0);
        assert_eq!(g.pad_bottom(), 0);
        assert_eq!(g.subsampling_x(), 1);
        assert_eq!(g.subsampling_y(), 1);
    }

    #[test]
    fn new() {
        assert!(PlaneGeometry::new(0, 1080, 40, 30, 20, 10, 1, 1).is_none());
        assert!(PlaneGeometry::new(1920, 0, 40, 30, 20, 10, 1, 1).is_none());
        assert!(PlaneGeometry::new(1920, 1080, 40, 30, 20, 10, 0, 1).is_none());
        assert!(PlaneGeometry::new(1920, 1080, 40, 30, 20, 10, 1, 0).is_none());

        let g = PlaneGeometry::new(1920, 1080, 40, 30, 20, 10, 1, 1).expect("new geometry works");
        assert_eq!(g.width(), 1920);
        assert_eq!(g.height(), 1080);
        assert_eq!(g.stride(), 1920 + 40 + 30);
        assert_eq!(g.pad_left(), 40);
        assert_eq!(g.pad_right(), 30);
        assert_eq!(g.pad_top(), 20);
        assert_eq!(g.pad_bottom(), 10);
        assert_eq!(g.subsampling_x(), 1);
        assert_eq!(g.subsampling_y(), 1);
    }

    #[test]
    fn new_overflow() {
        // Together, (width + pad_left + pad_right) overflow usize
        assert!(PlaneGeometry::new(usize::MAX - 50, 1080, 30, 30, 0, 0, 1, 1).is_none());
    }

    #[test]
    fn subsampled() {
        let g = PlaneGeometry::new(1920, 1080, 40, 30, 20, 10, 1, 1).expect("can make geometry");

        assert_eq!(g.for_subsampling(ChromaSubsampling::Monochrome), Ok(None));

        let sub = g
            .for_subsampling(ChromaSubsampling::Yuv444)
            .expect("can 'subsample' to Yuv444")
            .expect("subsampled plane exists for Yuv444");
        assert_eq!(sub, g);

        let sub = g
            .for_subsampling(ChromaSubsampling::Yuv422)
            .expect("can subsample to Yuv422")
            .expect("subsampled plane exists for Yuv422");
        assert_eq!(sub.width(), 960);
        assert_eq!(sub.height(), 1080);
        assert_eq!(sub.stride(), 960 + 20 + 15);
        assert_eq!(sub.pad_left(), 20);
        assert_eq!(sub.pad_right(), 15);
        assert_eq!(sub.pad_top(), 20);
        assert_eq!(sub.pad_bottom(), 10);
        assert_eq!(sub.subsampling_x(), 2);
        assert_eq!(sub.subsampling_y(), 1);

        let sub = g
            .for_subsampling(ChromaSubsampling::Yuv420)
            .expect("can subsample to Yuv420")
            .expect("subsampled plane exists for Yuv420");
        assert_eq!(sub.width(), 960);
        assert_eq!(sub.height(), 540);
        assert_eq!(sub.stride(), 960 + 20 + 15);
        assert_eq!(sub.pad_left, 20);
        assert_eq!(sub.pad_right, 15);
        assert_eq!(sub.pad_top, 10);
        assert_eq!(sub.pad_bottom, 5);
        assert_eq!(sub.subsampling_x(), 2);
        assert_eq!(sub.subsampling_y(), 2);
    }

    #[test]
    #[expect(clippy::unwrap_used)]
    fn subsampled_reject() {
        let g = PlaneGeometry::unpadded(1921, 1080, 1, 1).unwrap();
        assert!(g.for_subsampling(ChromaSubsampling::Monochrome).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv444).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv422).is_err());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv420).is_err());

        let g = PlaneGeometry::unpadded(1920, 1081, 1, 1).unwrap();
        assert!(g.for_subsampling(ChromaSubsampling::Monochrome).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv444).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv422).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv420).is_err());

        let g = PlaneGeometry::unpadded(17, 41, 1, 1).unwrap();
        assert!(g.for_subsampling(ChromaSubsampling::Monochrome).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv444).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv422).is_err());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv420).is_err());

        // again with padding
        let g = PlaneGeometry::new(1920, 1080, 10, 7, 40, 20, 1, 1).unwrap();
        assert!(g.for_subsampling(ChromaSubsampling::Monochrome).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv444).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv422).is_err());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv420).is_err());

        let g = PlaneGeometry::new(1920, 1080, 10, 10, 40, 15, 1, 1).unwrap();
        assert!(g.for_subsampling(ChromaSubsampling::Monochrome).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv444).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv422).is_ok());
        assert!(g.for_subsampling(ChromaSubsampling::Yuv420).is_err());
    }
}
