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
