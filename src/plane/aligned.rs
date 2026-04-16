use std::alloc::{Layout, alloc, alloc_zeroed, dealloc, handle_alloc_error};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::num::NonZeroUsize;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use crate::pixel::Pixel;

/// Alignment for plane data on WASM platforms (8 bytes).
#[cfg(target_arch = "wasm32")]
const DATA_ALIGNMENT: usize = 1 << 3;

/// Alignment for plane data on non-WASM platforms (64 bytes for SIMD optimization).
#[cfg(not(target_arch = "wasm32"))]
const DATA_ALIGNMENT: usize = 1 << 6;

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
        let t_size = const { NonZeroUsize::new(size_of::<T>()).expect("T is Sized") };

        let size = len
            .checked_mul(t_size)
            .expect("allocation size does not overflow usize");

        match Layout::from_size_align(size.get(), DATA_ALIGNMENT) {
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

impl<T: Debug> Debug for AlignedData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <[T] as Debug>::fmt(self, f)
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
