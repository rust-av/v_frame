# Changelog

## Version 0.7.0

- [Breaking] `PlaneGeometry` fields are now private; use the provided getters instead.
- [Breaking] Arithmetic and bitwise operations removed from the `Pixel` trait; convert to a concrete numeric type before performing arithmetic.
- The crate is now `#![no_std]` (requires `alloc`).
- Fix alignment promise for empty `AlignedData`; MSRV bumped to 1.95.

## Version 0.6.0

- [Breaking] `FrameBuilder`, `Plane`, and geometry APIs now take plain `usize` / zeroable values instead of `NonZero*` wrappers.
- [Breaking] Manual plane geometry construction is no longer supported; geometry is managed by the crate.
- [Breaking] Error types are split into `frame::error` and `plane::error`.
- Allow uninitialized `Plane<T>`.
- Add `std` traits to `Frame`, `Plane`, and `AlignedData`.
- Fix soundness issues.

## Version 0.5.2

- perf: copy_from_slice per row
- refactor: remove aligned-vec dependency

## Version 0.5.1

- [Feature] Add index-based plane accessors to `Frame`

## Version 0.5.0

- [Breaking] Change the `copy_from_u8_slice_with_stride` to take stride in bytes instead of in pixels.
  This seems to be the more common case from other APIs, and it also addresses a theoretical edge case
  where stride may not be a multiple of byte depth.

## Version 0.4.2

- Improve performance of `rows` iterators

## Version 0.4.1

- Implement `ExactSizeIterator` and `DoubleEndedIterator` for plane iterators

## Version 0.4.0

- [Breaking] Rewrite the API
  - This is a large change with significant breakage, consumers may want to refer to
    [the pull request](https://github.com/rust-av/v_frame/pull/44) for more context

## Version 0.3.8

- Eliminate unsound code
- Remove several unnecessary dependencies
- Internal refactoring

## Version 0.3.7

- Add documentation for `Frame` struct
- Replace `hawktracer` with `tracing` crate

## Version 0.3.6

- Revert changes in downsampling in 0.3.4 which changed its behavior

## Version 0.3.5

- Bump num-derive to 0.4

## Version 0.3.4

- Fix cases of unsoundness (#14)
- Slight optimizations for downsampling

## Version 0.3.3

- Add `row_cropped` and `row_slice_cropped` methods to get rows without padding
- Make `RowsIter` and `RowsIterMut` return rows without right-side padding for greater consistency/predictability
- Fix clippy lints

## Version 0.3.1

- Add `rows_iter_mut` method to `Plane`

## Version 0.2.6

- Split into separate repository
- Remove unused rayon dependency
- Fix some clippy lints
