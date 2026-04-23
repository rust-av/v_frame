use std::num::{NonZeroU8, NonZeroUsize};

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
    /// The horizontal subsampling ratio of this plane compared to the luma plane
    /// Will be 1 if no subsampling
    pub subsampling_x: NonZeroU8,
    /// The horizontal subsampling ratio of this plane compared to the luma plane
    /// Will be 1 if no subsampling
    pub subsampling_y: NonZeroU8,
}

impl PlaneGeometry {
    /// Returns the total height of the plane, including padding
    #[inline]
    #[must_use]
    #[cfg_attr(not(feature = "padding_api"), doc(hidden))]
    pub fn alloc_height(&self) -> NonZeroUsize {
        self.height
            .saturating_add(self.pad_top)
            .saturating_add(self.pad_bottom)
    }
}
