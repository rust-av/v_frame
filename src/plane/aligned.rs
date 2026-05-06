use std::alloc::{Layout, alloc, alloc_zeroed, dealloc, handle_alloc_error};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit, align_of};
use std::num::NonZeroUsize;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use crate::pixel::Pixel;

// Minimum data alignment to help with SIMD. Non-empty allocations use this or
// `align_of::<T>()`, whichever is larger.
const DATA_ALIGNMENT: usize = {
    if cfg!(target_arch = "wasm32") && cfg!(not(target_os = "wasi")) {
        // wasm32-unknown-unknown, wasm32-unknown-emscripten
        // these targets may have problems allocating with alignments > 8 bytes
        8
    } else {
        // Others (x86, arm, wasm32-wasip1)
        64
    }
};

pub struct AlignedData<T> {
    ptr: NonNull<[T]>,
    _marker: PhantomData<T>,
}

// SAFETY: `ptr` is unique, we have exclusive access to the data and all access to
//         the underlying data follows Rust borrowing/aliasing rules.
unsafe impl<T: Send> Send for AlignedData<T> {}
// SAFETY: See above.
unsafe impl<T: Sync> Sync for AlignedData<T> {}

impl<T> AlignedData<T> {
    const fn layout(len: NonZeroUsize) -> Layout {
        const { assert!(DATA_ALIGNMENT.is_power_of_two()) };
        let alignment = if align_of::<T>() > DATA_ALIGNMENT {
            align_of::<T>()
        } else {
            DATA_ALIGNMENT
        };
        let t_size = const { NonZeroUsize::new(size_of::<T>()).expect("T is Sized") };

        let size = len
            .checked_mul(t_size)
            .expect("allocation size does not overflow usize");

        match Layout::from_size_align(size.get(), alignment) {
            Ok(l) => l,
            _ => panic!("invalid layout"),
        }
    }

    pub fn new_uninit(len: usize) -> AlignedData<MaybeUninit<T>> {
        let ptr = if let Some(len) = NonZeroUsize::new(len) {
            let layout = Self::layout(len);
            // SAFETY: `Self::layout` guarantees that the layout is valid and has nonzero size.
            let ptr = unsafe { alloc(layout) as *mut MaybeUninit<T> };
            let Some(ptr) = NonNull::new(ptr) else {
                handle_alloc_error(layout);
            };

            NonNull::slice_from_raw_parts(ptr, len.get())
        } else {
            NonNull::slice_from_raw_parts(NonNull::dangling(), 0)
        };

        AlignedData {
            ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> AlignedData<MaybeUninit<T>> {
    /// Converts to [`AlignedData<T>`].
    ///
    /// # Safety
    /// It is up to the caller to ensure that all contained values are
    /// initialized properly (see [`MaybeUninit::assume_init`]).
    pub unsafe fn assume_init(self) -> AlignedData<T> {
        // The underlying memory would usually be deallocated when `Drop`
        // is run at the end of this scope. It needs to stay valid, so
        // inhibit the destructor here.
        let this = ManuallyDrop::new(self);

        AlignedData {
            ptr: NonNull::slice_from_raw_parts(this.ptr.cast(), this.len()),
            _marker: PhantomData,
        }
    }
}

impl<T: Pixel> AlignedData<T> {
    /// Zeroed.
    pub fn new(len: usize) -> Self {
        let ptr = if let Some(len) = NonZeroUsize::new(len) {
            let layout = Self::layout(len);
            // SAFETY:
            // - `Self::layout` guarantees that the layout is valid and has nonzero size
            // - The Pixel trait guarantees that zeroed memory is a valid T
            let ptr = unsafe { alloc_zeroed(layout) as *mut T };
            let Some(ptr) = NonNull::new(ptr) else {
                handle_alloc_error(layout);
            };

            NonNull::slice_from_raw_parts(ptr, len.get())
        } else {
            NonNull::slice_from_raw_parts(NonNull::dangling(), 0)
        };

        Self {
            ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for AlignedData<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - `self.ptr` is non-null and valid for `len` reads of `T`
        // - `self.ptr` + `len` describe a single allocation
        // - `self.ptr` is properly aligned (allocated with a valid Layout)
        // - all values of T are properly initialized, either via zeroing
        //   or manually before calling `Self::assume_init`
        // - immutable borrow is upheld by `&self`
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for AlignedData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: See `deref` above. Additionally:
        // - `self.ptr` is valid for `len` writes of `T`
        // - mutable borrow is upheld by `&mut self`
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: PartialEq<U>, U> PartialEq<AlignedData<U>> for AlignedData<T> {
    fn eq(&self, other: &AlignedData<U>) -> bool {
        <[T] as PartialEq<[U]>>::eq(self, other)
    }
}

impl<T: Eq> Eq for AlignedData<T> {}

impl<T: Debug> Debug for AlignedData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.len() > 5 {
            f.debug_list().entries(&self[..5]).finish_non_exhaustive()
        } else {
            f.debug_list().entries(&self[..]).finish()
        }
    }
}

impl<T: Clone> Clone for AlignedData<T> {
    fn clone(&self) -> Self {
        let mut new = Self::new_uninit(self.len());

        assert_eq!(
            self.len(),
            new.len(),
            "data length must be equal to clone safely"
        );

        for (new, old) in new.iter_mut().zip(self.iter()) {
            new.write(old.clone());
        }

        // SAFETY:
        // All values are properly initialized in the loop above.
        unsafe { new.assume_init() }
    }
}

impl<T> Drop for AlignedData<T> {
    fn drop(&mut self) {
        let layout = match NonZeroUsize::new(self.len()) {
            Some(len) => Self::layout(len),
            None => return, // nothing allocated, nothing to deallocate
        };

        // drop the contained T (i.e. dropping [T]), then dealloc

        // SAFETY: explained per line below
        unsafe {
            // - `ptr` is valid for read/write
            // - `ptr` is non-null and correctly aligned
            // - If this is T: Pixel, it does not need drop glue
            // - If this is any other T, we assume the initialization
            //   made all values valid for dropping
            // - we have exclusive access to the values contained in `ptr`
            self.ptr.drop_in_place();

            // - `ptr` was allocated via this (global) allocator
            // - `layout` is equal to the one used for allocation (returned from
            //   Self::layout for the same parameter `len`)
            dealloc(self.ptr.as_ptr() as _, layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        AlignedData::<u8>::new(0);
        AlignedData::<u16>::new(0);
        // would not compile:
        // AlignedData::<String>::new(0);
    }

    #[test]
    fn empty_uninit() {
        AlignedData::<u8>::new_uninit(0);
        AlignedData::<u16>::new_uninit(0);
        AlignedData::<String>::new_uninit(0);
    }

    #[cfg(miri)]
    #[test]
    fn new_uninit_underaligns_overaligned_types() {
        #[allow(dead_code)]
        #[repr(align(1048576))]
        struct OverAligned([u8; 1]);

        // Issue: `AlignedData::new_uninit` allocates with `DATA_ALIGNMENT`
        // rather than `max(DATA_ALIGNMENT, align_of::<T>())`. Safe callers can
        // therefore create `AlignedData<MaybeUninit<T>>` for an over-aligned
        // `T`, and the safe slice accessors form references whose required
        // alignment is stronger than the allocation's layout. The public
        // `Plane::new_uninit` padding API forwards to this helper for arbitrary
        // `T`, so this can be reached without an unsafe call before any
        // `assume_init`.
        let mut data = AlignedData::<OverAligned>::new_uninit(1);
        data[0].write(OverAligned([0]));
    }

    #[test]
    #[should_panic(expected = "invalid layout")]
    fn invalid_layout_panic() {
        let too_big = isize::MAX as usize + 100;
        AlignedData::<u8>::new_uninit(too_big);
    }

    #[test]
    fn basic_zeroed() {
        let data = AlignedData::<u16>::new(512);

        #[expect(
            clippy::needless_collect,
            reason = "explicit collect so we can drop(data) before iterating"
        )]
        let vec: Vec<_> = data
            .iter()
            .enumerate()
            .map(|(idx, val)| *val as usize + idx)
            .collect();

        drop(data);

        for (idx, v) in vec.into_iter().enumerate() {
            assert_eq!(idx, v);
        }
    }

    #[test]
    fn uninit_manual() {
        let mut data = AlignedData::<u8>::new_uninit(256);
        for (idx, x) in data.iter_mut().enumerate() {
            x.write((idx % 42) as u8);
        }

        // SAFETY: Initialized above.
        let data = unsafe { data.assume_init() };
        println!("{:?}", &data[100..140]);
    }

    #[test]
    fn uninit_with_drop() {
        let mut data = AlignedData::<String>::new_uninit(3);
        data[0].write("Hello World".into());
        data[1].write(String::new());
        data[2].write("This is a test".into());

        // SAFETY: Initialized above.
        let data = unsafe { data.assume_init() };
        println!("{:?}", &*data);
    }

    #[test]
    fn partialeq() {
        let mut data = AlignedData::<u8>::new(8);
        data[4] = 42;
        data[6] = 123;

        assert_eq!(data, data.clone());

        let other_data = AlignedData::<u8>::new(8);
        assert_ne!(data, other_data);

        let mut other_data = data.clone();
        other_data[2] = 50;
        assert_ne!(data, other_data);

        // does not compile:
        // let u16_data = AlignedData::<u16>::new(10);
        // assert_ne!(data, u16_data);
    }

    #[test]
    fn partialeq_different_types() {
        #[derive(Debug)]
        struct Test(u8);

        impl PartialEq<u8> for Test {
            fn eq(&self, other: &u8) -> bool {
                self.0.eq(other)
            }
        }

        let mut data = AlignedData::<u8>::new(3);
        data[0] = 42;

        let mut test_data = AlignedData::<Test>::new_uninit(3);
        test_data[0].write(Test(0));
        test_data[1].write(Test(0));
        test_data[2].write(Test(0));

        // SAFETY: Fully initialized above.
        let mut test_data = unsafe { test_data.assume_init() };
        assert_ne!(test_data, data);

        test_data[0] = Test(42);
        assert_eq!(test_data, data);
    }

    #[test]
    fn debug_fmt() {
        let mut data = AlignedData::<u8>::new(12);
        data[4] = 42;
        data[10] = 123;

        assert_eq!(format!("{data:?}"), "[0, 0, 0, 0, 42, ..]");

        let mut data = AlignedData::<u8>::new(3);
        data[1] = 7;
        // don't panic when len < 5
        assert_eq!(format!("{data:?}"), "[0, 7, 0]");
    }

    #[test]
    fn clone() {
        let mut data = AlignedData::<String>::new_uninit(3);
        data[0].write("Hello World".into());
        data[1].write(String::new());
        data[2].write("This is a test".into());

        // SAFETY: Initialized above.
        let data = unsafe { data.assume_init() };
        let data2 = data.clone();
        drop(data);
        println!("{:?}", &*data2);
    }
}
