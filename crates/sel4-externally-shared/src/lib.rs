//! Provides the wrapper type `ExternallyShared`, which wraps a reference to any copy-able type and
//! allows for ergonomic access to the referenced value via raw pointer operations.
//!
//! This type is meant to faciliate access to memory resources that are shared by other processes or
//! system components outside of the Rust language runtime. Such cases violate the assumptions that
//! the Rust compiler makes about `&` references, so raw pointer operations must be used. This crate
//! enables wraps accesses and enables safe and ergonimic pointer arithmatic.

#![no_std]
#![cfg_attr(feature = "unstable", feature(core_intrinsics))]
#![cfg_attr(feature = "unstable", feature(slice_range))]
#![cfg_attr(feature = "unstable", allow(incomplete_features))]
#![cfg_attr(all(feature = "unstable", test), feature(slice_as_chunks))]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

use access::{ReadOnly, ReadWrite, Readable, Writable, WriteOnly};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr,
    slice::SliceIndex,
};
#[cfg(feature = "unstable")]
use core::{
    ops::{Range, RangeBounds},
    slice::range,
};

/// Allows creating read-only and write-only `ExternallyShared` values.
pub mod access;

/// Wraps a reference to facilitate accesses to the referenced value carried via raw pointer
/// operations.
///
/// Since not all externally shared memory resources are both readable and writable, this type
/// supports limiting the allowed access types through an optional second generic parameter `A` that
/// can be one of `ReadWrite`, `ReadOnly`, or `WriteOnly`. It defaults to `ReadWrite`, which allows
/// all operations.
///
/// The size of this struct is the same as the size of the contained reference.
#[derive(Clone)]
#[repr(transparent)]
pub struct ExternallyShared<R, A = ReadWrite> {
    reference: R,
    access: PhantomData<A>,
}

/// Constructor functions for creating new values
///
/// These functions allow to construct a new `ExternallyShared` instance from a reference type. While
/// the `new` function creates a `ExternallyShared` instance with unrestricted access, there are also
/// functions for creating read-only or write-only instances.
impl<R> ExternallyShared<R> {
    /// Constructs a new instance wrapping the given reference.
    ///
    /// While it is possible to construct `ExternallyShared` instances from arbitrary values (including
    /// non-reference values), most of the methods are only available when the wrapped type is
    /// a reference. The only reason that we don't forbid non-reference types in the constructor
    /// functions is that the Rust compiler does not support trait bounds on generic `const`
    /// functions yet. When this becomes possible, we will release a new version of this library
    /// with removed support for non-references. For these reasons it is recommended to use
    /// the `ExternallyShared` type only with references.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut value = 0u32;
    ///
    /// let mut shared = ExternallyShared::new(&mut value);
    /// shared.write(1);
    /// assert_eq!(shared.read(), 1);
    /// ```
    pub const fn new(reference: R) -> ExternallyShared<R> {
        ExternallyShared {
            reference,
            access: PhantomData,
        }
    }

    /// Constructs a new read-only instance wrapping the given reference.
    ///
    /// This is equivalent to the `new` function with the difference that the returned
    /// `ExternallyShared` instance does not permit write operations. This is for example useful
    /// with memory-mapped hardware registers that are defined as read-only by the hardware.
    ///
    /// ## Example
    ///
    /// Reading is allowed:
    ///
    /// ```rust
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let value = 0u32;
    ///
    /// let shared = ExternallyShared::new_read_only(&value);
    /// assert_eq!(shared.read(), 0);
    /// ```
    ///
    /// But writing is not:
    ///
    /// ```compile_fail
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut value = 0u32;
    ///
    /// let mut shared = ExternallyShared::new_read_only(&mut value);
    /// shared.write(1);
    /// //ERROR: ^^^^^ the trait `shared::access::Writable` is not implemented
    /// //             for `shared::access::ReadOnly`
    /// ```
    pub const fn new_read_only(reference: R) -> ExternallyShared<R, ReadOnly> {
        ExternallyShared {
            reference,
            access: PhantomData,
        }
    }

    /// Constructs a new write-only instance wrapping the given reference.
    ///
    /// This is equivalent to the `new` function with the difference that the returned
    /// `ExternallyShared` instance does not permit read operations. This is for example useful
    /// with memory-mapped hardware registers that are defined as write-only by the hardware.
    ///
    /// ## Example
    ///
    /// Writing is allowed:
    ///
    /// ```rust
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut value = 0u32;
    ///
    /// let mut shared = ExternallyShared::new_write_only(&mut value);
    /// shared.write(1);
    /// ```
    ///
    /// But reading is not:
    ///
    /// ```compile_fail
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let value = 0u32;
    ///
    /// let shared = ExternallyShared::new_write_only(&value);
    /// shared.read();
    /// //ERROR: ^^^^ the trait `shared::access::Readable` is not implemented
    /// //            for `shared::access::WriteOnly`
    /// ```
    pub const fn new_write_only(reference: R) -> ExternallyShared<R, WriteOnly> {
        ExternallyShared {
            reference,
            access: PhantomData,
        }
    }
}

/// Methods for references to `Copy` types
impl<R, T, A> ExternallyShared<R, A>
where
    R: Deref<Target = T>,
    T: Copy,
{
    /// Performs a raw pointer read of the contained value.
    ///
    /// Returns a copy of the read value. Volatile reads are guaranteed not to be optimized
    /// away by the compiler, but by themselves do not have atomic ordering
    /// guarantees. To also get atomicity, consider looking at the `Atomic` wrapper types of
    /// the standard/`core` library.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let value = 42;
    /// let shared_reference = ExternallyShared::new(&value);
    /// assert_eq!(shared_reference.read(), 42);
    ///
    /// let mut value = 50;
    /// let mut_reference = ExternallyShared::new(&mut value);
    /// assert_eq!(mut_reference.read(), 50);
    /// ```
    pub fn read(&self) -> T
    where
        A: Readable,
    {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::read(&*self.reference) }
    }

    /// Performs a raw pointer write, setting the contained value to the given `value`.
    ///
    /// Volatile writes are guaranteed to not be optimized away by the compiler, but by
    /// themselves do not have atomic ordering guarantees. To also get atomicity, consider
    /// looking at the `Atomic` wrapper types of the standard/`core` library.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut value = 42;
    /// let mut shared = ExternallyShared::new(&mut value);
    /// shared.write(50);
    ///
    /// assert_eq!(shared.read(), 50);
    /// ```
    pub fn write(&mut self, value: T)
    where
        A: Writable,
        R: DerefMut,
    {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::write(&mut *self.reference, value) };
    }

    /// Updates the contained value using the given closure and raw pointer operations.
    ///
    /// Performs a raw pointer read of the contained value, passes a mutable reference to it to the
    /// function `f`, and then performs a raw pointer write of the (potentially updated) value back
    /// to the contained value.
    ///
    /// ```rust
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut value = 42;
    /// let mut shared = ExternallyShared::new(&mut value);
    /// shared.update(|val| *val += 1);
    ///
    /// assert_eq!(shared.read(), 43);
    /// ```
    pub fn update<F>(&mut self, f: F)
    where
        A: Readable + Writable,
        R: DerefMut,
        F: FnOnce(&mut T),
    {
        let mut value = self.read();
        f(&mut value);
        self.write(value);
    }
}

/// Method for extracting the wrapped value.
impl<R, A> ExternallyShared<R, A> {
    /// Extracts the inner value stored in the wrapper type.
    ///
    /// This method gives direct access to the wrapped reference and thus allows normal access
    /// again. This is seldom what you want since there is usually a reason that a reference is
    /// wrapped in `ExternallyShared`. However, in some cases it might be required or useful to use the
    /// `ptr::read`/`ptr::write` pointer methods of the standard library directly, which
    /// this method makes possible.
    ///
    /// Since no memory safety violation can occur when accessing the referenced value using normal
    /// operations, this method is safe. However, it _can_ lead to bugs at the application level, so
    /// this method should be used with care.
    ///
    /// ## Example
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut value = 42;
    /// let mut shared = ExternallyShared::new(&mut value);
    /// shared.write(50);
    /// let unwrapped: &mut i32 = shared.extract_inner();
    ///
    /// assert_eq!(*unwrapped, 50); // reference access subject to more optimization, be careful!
    /// ```
    pub fn extract_inner(self) -> R {
        self.reference
    }
}

/// Transformation methods for accessing struct fields
impl<R, T, A> ExternallyShared<R, A>
where
    R: Deref<Target = T>,
    T: ?Sized,
{
    /// Constructs a new `ExternallyShared` reference by mapping the wrapped value.
    ///
    /// This method is useful for accessing individual fields of externally shared structs.
    ///
    /// Note that this method gives temporary access to the wrapped reference, which allows
    /// accessing the value in arbitrary ways. This is normally not what you want, so **this method
    /// should only be used for reference-to-reference transformations**.
    ///
    /// ## Examples
    ///
    /// Accessing a struct field:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut shared = ExternallyShared::new(&mut value);
    ///
    /// // construct a shared reference to a field
    /// let field_2 = shared.map(|example| &example.field_2);
    /// assert_eq!(field_2.read(), 255);
    /// ```
    ///
    /// Don't misuse this method to do a arbitrary reads of the referenced value:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut value = 5;
    /// let mut shared = ExternallyShared::new(&mut value);
    ///
    /// // DON'T DO THIS:
    /// let mut readout = 0;
    /// shared.map(|value| {
    ///    readout = *value; // normal reference read, might lead to bugs
    ///    value
    /// });
    /// ```
    pub fn map<'a, F, U>(&'a self, f: F) -> ExternallyShared<&'a U, A>
    where
        F: FnOnce(&'a T) -> &'a U,
        U: ?Sized,
        T: 'a,
    {
        ExternallyShared {
            reference: f(self.reference.deref()),
            access: self.access,
        }
    }

    /// Constructs a new mutable `ExternallyShared` reference by mapping the wrapped value.
    ///
    /// This method is useful for accessing individual fields of externally shared structs.
    ///
    /// Note that this method gives temporary access to the wrapped reference, which allows
    /// accessing the value in arbirary ways. This is normally not what you want, so
    /// **this method should only be used for reference-to-reference transformations**.
    ///
    /// ## Examples
    ///
    /// Accessing a struct field:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut shared = ExternallyShared::new(&mut value);
    ///
    /// // construct a shared reference to a field
    /// let mut field_2 = shared.map_mut(|example| &mut example.field_2);
    /// field_2.write(128);
    /// assert_eq!(field_2.read(), 128);
    /// ```
    ///
    /// Don't misuse this method to do abitrary reads or writes of the referenced value:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut value = 5;
    /// let mut shared = ExternallyShared::new(&mut value);
    ///
    /// // DON'T DO THIS:
    /// shared.map_mut(|value| {
    ///    *value = 10; // normal reference write, might lead to bugs
    ///    value
    /// });
    /// ```
    pub fn map_mut<'a, F, U>(&'a mut self, f: F) -> ExternallyShared<&'a mut U, A>
    where
        F: FnOnce(&mut T) -> &mut U,
        R: DerefMut,
        U: ?Sized,
        T: 'a,
    {
        ExternallyShared {
            reference: f(&mut self.reference),
            access: self.access,
        }
    }
}

/// Methods for externally shared slices
impl<T, R, A> ExternallyShared<R, A>
where
    R: Deref<Target = [T]>,
{
    /// Length of the wrapped slice.
    pub fn len(&self) -> usize {
        self.reference.deref().len()
    }

    /// Applies the index operation on the wrapped slice.
    ///
    /// Returns a shared `ExternallyShared` reference to the resulting subslice.
    ///
    /// This is a convenience method for the `map(|slice| slice.index(index))` operation, so it
    /// has the same behavior as the indexing operation on slice (e.g. panic if index is
    /// out-of-bounds).
    ///
    /// ## Examples
    ///
    /// Accessing a single slice element:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let array = [1, 2, 3];
    /// let slice = &array[..];
    /// let shared = ExternallyShared::new(slice);
    /// assert_eq!(shared.index(1).read(), 2);
    /// ```
    ///
    /// Accessing a subslice:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let array = [1, 2, 3];
    /// let slice = &array[..];
    /// let shared = ExternallyShared::new(slice);
    /// let subslice = shared.index(1..);
    /// assert_eq!(subslice.index(0).read(), 2);
    /// ```
    pub fn index<'a, I>(&'a self, index: I) -> ExternallyShared<&'a I::Output, A>
    where
        I: SliceIndex<[T]>,
        T: 'a,
    {
        self.map(|slice| slice.index(index))
    }

    /// Applies the mutable index operation on the wrapped slice.
    ///
    /// Returns a mutable `ExternallyShared` reference to the resulting subslice.
    ///
    /// This is a convenience method for the `map_mut(|slice| slice.index_mut(index))`
    /// operation, so it has the same behavior as the indexing operation on slice
    /// (e.g. panic if index is out-of-bounds).
    ///
    /// ## Examples
    ///
    /// Accessing a single slice element:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut array = [1, 2, 3];
    /// let slice = &mut array[..];
    /// let mut shared = ExternallyShared::new(slice);
    /// shared.index_mut(1).write(6);
    /// assert_eq!(shared.index(1).read(), 6);
    /// ```
    ///
    /// Accessing a subslice:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut array = [1, 2, 3];
    /// let slice = &mut array[..];
    /// let mut shared = ExternallyShared::new(slice);
    /// let mut subslice = shared.index_mut(1..);
    /// subslice.index_mut(0).write(6);
    /// assert_eq!(subslice.index(0).read(), 6);
    /// ```
    pub fn index_mut<'a, I>(&'a mut self, index: I) -> ExternallyShared<&mut I::Output, A>
    where
        I: SliceIndex<[T]>,
        R: DerefMut,
        T: 'a,
    {
        self.map_mut(|slice| slice.index_mut(index))
    }

    /// Copies all elements from `self` into `dst`, using memcpy.
    ///
    /// The length of `dst` must be the same as `self`.
    ///
    /// The method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if the two slices have different lengths.
    ///
    /// ## Examples
    ///
    /// Copying two elements from an externally shared slice:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let src = [1, 2];
    /// // the `ExternallyShared` type does not work with arrays, so convert `src` to a slice
    /// let slice = &src[..];
    /// let shared = ExternallyShared::new(slice);
    /// let mut dst = [5, 0, 0];
    ///
    /// // Because the slices have to be the same length,
    /// // we slice the destination slice from three elements
    /// // to two. It will panic if we don't do this.
    /// shared.copy_into_slice(&mut dst[1..]);
    ///
    /// assert_eq!(src, [1, 2]);
    /// assert_eq!(dst, [5, 1, 2]);
    /// ```
    #[cfg(feature = "unstable")]
    pub fn copy_into_slice(&self, dst: &mut [T])
    where
        T: Copy,
    {
        let src = self.reference.deref();
        assert_eq!(
            src.len(),
            dst.len(),
            "destination and source slices have different lengths"
        );
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
        }
    }

    /// Copies all elements from `src` into `self`, using memcpy.
    ///
    /// The length of `src` must be the same as `self`.
    ///
    /// This method is similar to the `slice::copy_from_slice` method of the standard library. The
    /// difference is that this method performs a raw pointer copy.
    ///
    /// The method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if the two slices have different lengths.
    ///
    /// ## Examples
    ///
    /// Copying two elements from a slice into an externally shared slice:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let src = [1, 2, 3, 4];
    /// let mut dst = [0, 0];
    /// // the `ExternallyShared` type does not work with arrays, so convert `dst` to a slice
    /// let slice = &mut dst[..];
    /// let mut shared = ExternallyShared::new(slice);
    ///
    /// // Because the slices have to be the same length,
    /// // we slice the source slice from four elements
    /// // to two. It will panic if we don't do this.
    /// shared.copy_from_slice(&src[2..]);
    ///
    /// assert_eq!(src, [1, 2, 3, 4]);
    /// assert_eq!(dst, [3, 4]);
    /// ```
    #[cfg(feature = "unstable")]
    pub fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
        R: DerefMut,
    {
        let dest = self.reference.deref_mut();
        assert_eq!(
            dest.len(),
            src.len(),
            "destination and source slices have different lengths"
        );
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), dest.len());
        }
    }

    /// Copies elements from one part of the slice to another part of itself, using `memmove`.
    ///
    /// `src` is the range within `self` to copy from. `dest` is the starting index of the
    /// range within `self` to copy to, which will have the same length as `src`. The two ranges
    /// may overlap. The ends of the two ranges must be less than or equal to `self.len()`.
    ///
    /// This method is similar to the `slice::copy_within` method of the standard library. The
    /// difference is that this method performs a raw pointer copy.
    ///
    /// This method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if either range exceeds the end of the slice, or if the end
    /// of `src` is before the start.
    ///
    /// ## Examples
    ///
    /// Copying four bytes within a slice:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut byte_array = *b"Hello, World!";
    /// let mut slice: &mut [u8] = &mut byte_array[..];
    /// let mut shared = ExternallyShared::new(slice);
    ///
    /// shared.copy_within(1..5, 8);
    ///
    /// assert_eq!(&byte_array, b"Hello, Wello!");
    #[cfg(feature = "unstable")]
    pub fn copy_within(&mut self, src: impl RangeBounds<usize>, dest: usize)
    where
        T: Copy,
        R: DerefMut,
    {
        let slice = self.reference.deref_mut();
        // implementation taken from https://github.com/rust-lang/rust/blob/683d1bcd405727fcc9209f64845bd3b9104878b8/library/core/src/slice/mod.rs#L2726-L2738
        let Range {
            start: src_start,
            end: src_end,
        } = range(src, ..slice.len());
        let count = src_end - src_start;
        assert!(dest <= slice.len() - count, "dest is out of bounds");
        // SAFETY: the conditions for `ptr::copy` have all been checked above,
        // as have those for `ptr::add`.
        unsafe {
            ptr::copy(
                slice.as_ptr().add(src_start),
                slice.as_mut_ptr().add(dest),
                count,
            );
        }
    }

    /// Copies all elements from `self` into a `Vec`.
    #[cfg(feature = "alloc")]
    pub fn copy_to_vec(&self) -> Vec<T>
    where
        T: Copy,
    {
        let src = self.reference.deref();
        let n = src.len();
        let mut v = Vec::with_capacity(n);
        // SAFETY:
        // allocated above with the capacity of `src`, and initialize to `src.len()` in
        // ptr::copy_to_non_overlapping below.
        unsafe {
            src.as_ptr().copy_to_nonoverlapping(v.as_mut_ptr(), n);
            v.set_len(n);
        }
        v
    }
}

/// Methods for externally shared byte slices
impl<R, A> ExternallyShared<R, A>
where
    R: Deref<Target = [u8]>,
{
    /// Sets all elements of the byte slice to the given `value` using `memset`.
    ///
    /// This method is similar to the `slice::fill` method of the standard library, with the
    /// difference that this method performs raw pointer write operation. Another difference
    /// is that this method is only available for byte slices (not general `&mut [T]` slices)
    /// because there currently isn't a instrinsic function that allows non-`u8` values.
    ///
    /// This method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Example
    ///
    /// ```rust
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut buf = ExternallyShared::new(vec![0; 10]);
    /// buf.fill(1);
    /// assert_eq!(buf.extract_inner(), vec![1; 10]);
    /// ```
    #[cfg(feature = "unstable")]
    pub fn fill(&mut self, value: u8)
    where
        R: DerefMut,
    {
        let dest = self.reference.deref_mut();
        unsafe {
            ptr::write_bytes(dest.as_mut_ptr(), value, dest.len());
        }
    }
}

/// Methods for converting arrays to slices
impl<R, A, T, const N: usize> ExternallyShared<R, A>
where
    R: Deref<Target = [T; N]>,
{
    /// Converts an array reference to an externally shared slice.
    ///
    /// This makes it possible to use the methods defined on slices.
    ///
    /// ## Example
    ///
    /// Reading a subslice from an externally shared array reference using `index`:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let src = [1, 2, 3, 4];
    /// let shared = ExternallyShared::new(&src);
    ///
    /// // convert the `ExternallyShared<&[i32; 4]>` array reference to a `ExternallyShared<&[i32]>` slice
    /// let shared_slice = shared.as_slice();
    /// // we can now use the slice methods
    /// let subslice = shared_slice.index(2..);
    ///
    /// assert_eq!(subslice.index(0).read(), 3);
    /// assert_eq!(subslice.index(1).read(), 4);
    /// ```
    pub fn as_slice(&self) -> ExternallyShared<&[T], A> {
        self.map(|array| &array[..])
    }

    /// Converts a mutable array reference to a mutable slice.
    ///
    /// This makes it possible to use the methods defined on slices.
    ///
    /// ## Example
    ///
    /// Writing to an index of a mutable array reference:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut dst = [0, 0];
    /// let mut shared = ExternallyShared::new(&mut dst);
    ///
    /// // convert the `ExternallyShared<&mut [i32; 2]>` array reference to a `ExternallyShared<&mut [i32]>` slice
    /// let mut shared_slice = shared.as_mut_slice();
    /// // we can now use the slice methods
    /// shared_slice.index_mut(1).write(1);
    ///
    /// assert_eq!(dst, [0, 1]);
    /// ```
    pub fn as_mut_slice(&mut self) -> ExternallyShared<&mut [T], A>
    where
        R: DerefMut,
    {
        self.map_mut(|array| &mut array[..])
    }
}

/// Methods for restricting access.
impl<R> ExternallyShared<R> {
    /// Restricts access permissions to read-only.
    ///
    /// ## Example
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// let mut value: i16 = -4;
    /// let mut shared = ExternallyShared::new(&mut value);
    ///
    /// let read_only = shared.read_only();
    /// assert_eq!(read_only.read(), -4);
    /// // read_only.write(10); // compile-time error
    /// ```
    pub fn read_only(self) -> ExternallyShared<R, ReadOnly> {
        ExternallyShared {
            reference: self.reference,
            access: PhantomData,
        }
    }

    /// Restricts access permissions to write-only.
    ///
    /// ## Example
    ///
    /// Creating a write-only reference to a struct field:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallyShared;
    ///
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut shared = ExternallyShared::new(&mut value);
    ///
    /// // construct a shared write-only reference to `field_2`
    /// let mut field_2 = shared.map_mut(|example| &mut example.field_2).write_only();
    /// field_2.write(14);
    /// // field_2.read(); // compile-time error
    /// ```
    pub fn write_only(self) -> ExternallyShared<R, WriteOnly> {
        ExternallyShared {
            reference: self.reference,
            access: PhantomData,
        }
    }
}

impl<R, T, A> fmt::Debug for ExternallyShared<R, A>
where
    R: Deref<Target = T>,
    T: Copy + fmt::Debug,
    A: Readable,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ExternallyShared")
            .field(&self.read())
            .finish()
    }
}

impl<R> fmt::Debug for ExternallyShared<R, WriteOnly>
where
    R: Deref,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ExternallyShared")
            .field(&"[write-only]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::ExternallyShared;

    #[test]
    fn test_read() {
        let val = 42;
        assert_eq!(ExternallyShared::new(&val).read(), 42);
    }

    #[test]
    fn test_write() {
        let mut val = 50;
        let mut shared = ExternallyShared::new(&mut val);
        shared.write(50);
        assert_eq!(val, 50);
    }

    #[test]
    fn test_update() {
        let mut val = 42;
        let mut shared = ExternallyShared::new(&mut val);
        shared.update(|v| *v += 1);
        assert_eq!(val, 43);
    }

    #[test]
    fn test_slice() {
        let mut val = [1, 2, 3];
        let mut shared = ExternallyShared::new(&mut val[..]);
        shared.index_mut(0).update(|v| *v += 1);
        assert_eq!(val, [2, 2, 3]);
    }

    #[test]
    fn test_struct() {
        struct S {
            field_1: u32,
            field_2: bool,
        }

        let mut val = S {
            field_1: 60,
            field_2: true,
        };
        let mut shared = ExternallyShared::new(&mut val);
        shared.map_mut(|s| &mut s.field_1).update(|v| *v += 1);
        let mut field_2 = shared.map_mut(|s| &mut s.field_2);
        assert!(field_2.read());
        field_2.write(false);
        assert_eq!(shared.map(|s| &s.field_1).read(), 61);
        assert_eq!(shared.map(|s| &s.field_2).read(), false);
    }

    #[cfg(feature = "unstable")]
    #[test]
    fn test_chunks() {
        let mut val = [1, 2, 3, 4, 5, 6];
        let mut shared = ExternallyShared::new(&mut val[..]);
        let mut chunks = shared.map_mut(|s| s.as_chunks_mut().0);
        chunks.index_mut(1).write([10, 11, 12]);
        assert_eq!(chunks.index(0).read(), [1, 2, 3]);
        assert_eq!(chunks.index(1).read(), [10, 11, 12]);
    }
}
