use core::{
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::{
    access::{Access, ReadOnly, ReadWrite, Readable, Writable, WriteOnly},
    ExternallySharedPtr,
};

/// Constructor functions.
///
/// These functions construct new `ExternallySharedPtr` values. While the `new`
/// function creates a `ExternallySharedPtr` instance with unrestricted access, there
/// are also functions for creating read-only or write-only instances.
impl<'a, T> ExternallySharedPtr<'a, T>
where
    T: ?Sized,
{
    /// Turns the given pointer into a `ExternallySharedPtr`.
    ///
    /// ## Safety
    ///
    /// - The given pointer must be valid.
    /// - No other thread must have access to the given pointer. This must remain true
    ///   for the whole lifetime of the `ExternallySharedPtr`.
    pub unsafe fn new(pointer: NonNull<T>) -> ExternallySharedPtr<'a, T, ReadWrite> {
        unsafe { ExternallySharedPtr::new_restricted(ReadWrite, pointer) }
    }

    /// Creates a new read-only volatile pointer from the given raw pointer.
    ///
    /// ## Safety
    ///
    /// The requirements for [`Self::new`] apply to this function too.
    pub const unsafe fn new_read_only(pointer: NonNull<T>) -> ExternallySharedPtr<'a, T, ReadOnly> {
        unsafe { Self::new_restricted(ReadOnly, pointer) }
    }

    /// Creates a new volatile pointer with restricted access from the given raw pointer.
    ///
    /// ## Safety
    ///
    /// The requirements for [`Self::new`] apply to this function too.
    pub const unsafe fn new_restricted<A>(access: A, pointer: NonNull<T>) -> ExternallySharedPtr<'a, T, A>
    where
        A: Access,
    {
        let _ = access;
        unsafe { Self::new_generic(pointer) }
    }

    pub(super) const unsafe fn new_generic<A>(pointer: NonNull<T>) -> ExternallySharedPtr<'a, T, A> {
        ExternallySharedPtr {
            pointer,
            reference: PhantomData,
            access: PhantomData,
        }
    }
}

impl<'a, T, A> ExternallySharedPtr<'a, T, A>
where
    T: ?Sized,
{
    /// Performs a volatile read of the contained value.
    ///
    /// Returns a copy of the read value. Volatile reads are guaranteed not to be optimized
    /// away by the compiler, but by themselves do not have atomic ordering
    /// guarantees. To also get atomicity, consider looking at the `Atomic` wrapper types of
    /// the standard/`core` library.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use volatile::{ExternallySharedPtr, access};
    /// use core::ptr::NonNull;
    ///
    /// let value = 42;
    /// let pointer = unsafe {
    ///     ExternallySharedPtr::new_restricted(access::ReadOnly, NonNull::from(&value))
    /// };
    /// assert_eq!(pointer.read(), 42);
    /// ```
    pub fn read(self) -> T
    where
        T: Copy,
        A: Readable,
    {
        unsafe { ptr::read_volatile(self.pointer.as_ptr()) }
    }

    /// Performs a volatile write, setting the contained value to the given `value`.
    ///
    /// Volatile writes are guaranteed to not be optimized away by the compiler, but by
    /// themselves do not have atomic ordering guarantees. To also get atomicity, consider
    /// looking at the `Atomic` wrapper types of the standard/`core` library.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use volatile::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let mut value = 42;
    /// let mut volatile = unsafe { ExternallySharedPtr::new((&mut value).into()) };
    /// volatile.write(50);
    ///
    /// assert_eq!(volatile.read(), 50);
    /// ```
    pub fn write(self, value: T)
    where
        T: Copy,
        A: Writable,
    {
        unsafe { ptr::write_volatile(self.pointer.as_ptr(), value) };
    }

    /// Updates the contained value using the given closure and volatile instructions.
    ///
    /// Performs a volatile read of the contained value, passes it to the
    /// function `f`, and then performs a volatile write of the returned value back to
    /// the target.
    ///
    /// ```rust
    /// use volatile::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let mut value = 42;
    /// let mut volatile = unsafe { ExternallySharedPtr::new((&mut value).into()) };
    /// volatile.update(|val| val + 1);
    ///
    /// assert_eq!(volatile.read(), 43);
    /// ```
    pub fn update<F>(self, f: F)
    where
        T: Copy,
        A: Readable + Writable,
        F: FnOnce(T) -> T,
    {
        let new = f(self.read());
        self.write(new);
    }

    /// Extracts the wrapped raw pointer.
    ///
    /// ## Example
    ///
    /// ```
    /// use volatile::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let mut value = 42;
    /// let mut volatile = unsafe { ExternallySharedPtr::new((&mut value).into()) };
    /// volatile.write(50);
    /// let unwrapped: *mut i32 = volatile.as_raw_ptr().as_ptr();
    ///
    /// assert_eq!(unsafe { *unwrapped }, 50); // non volatile access, be careful!
    /// ```
    pub fn as_raw_ptr(self) -> NonNull<T> {
        self.pointer
    }

    /// Constructs a new `ExternallySharedPtr` by mapping the wrapped pointer.
    ///
    /// This method is useful for accessing only a part of a volatile value, e.g. a subslice or
    /// a struct field. For struct field access, there is also the safe
    /// [`map_field`][crate::map_field] macro that wraps this function.
    ///
    /// ## Examples
    ///
    /// Accessing a struct field:
    ///
    /// ```
    /// use volatile::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut volatile = unsafe { ExternallySharedPtr::new((&mut value).into()) };
    ///
    /// // construct a volatile pointer to a field
    /// let field_2 = unsafe { volatile.map(|ptr| NonNull::new(core::ptr::addr_of_mut!((*ptr.as_ptr()).field_2)).unwrap()) };
    /// assert_eq!(field_2.read(), 255);
    /// ```
    ///
    /// Don't misuse this method to do a non-volatile read of the referenced value:
    ///
    /// ```
    /// use volatile::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let mut value = 5;
    /// let mut volatile = unsafe { ExternallySharedPtr::new((&mut value).into()) };
    ///
    /// // DON'T DO THIS:
    /// let mut readout = 0;
    /// unsafe { volatile.map(|value| {
    ///    readout = *value.as_ptr(); // non-volatile read, might lead to bugs
    ///    value
    /// })};
    /// ```
    ///
    /// ## Safety
    ///
    /// The pointer returned by `f` must satisfy the requirements of [`Self::new`].
    pub unsafe fn map<F, U>(self, f: F) -> ExternallySharedPtr<'a, U, A>
    where
        F: FnOnce(NonNull<T>) -> NonNull<U>,
        A: Access,
        U: ?Sized,
    {
        unsafe { ExternallySharedPtr::new_restricted(A::default(), f(self.pointer)) }
    }
}

/// Methods for restricting access.
impl<'a, T> ExternallySharedPtr<'a, T, ReadWrite>
where
    T: ?Sized,
{
    /// Restricts access permissions to read-only.
    ///
    /// ## Example
    ///
    /// ```
    /// use volatile::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let mut value: i16 = -4;
    /// let mut volatile = unsafe { ExternallySharedPtr::new((&mut value).into()) };
    ///
    /// let read_only = volatile.read_only();
    /// assert_eq!(read_only.read(), -4);
    /// // read_only.write(10); // compile-time error
    /// ```
    pub fn read_only(self) -> ExternallySharedPtr<'a, T, ReadOnly> {
        unsafe { ExternallySharedPtr::new_restricted(ReadOnly, self.pointer) }
    }

    /// Restricts access permissions to write-only.
    ///
    /// ## Example
    ///
    /// Creating a write-only pointer to a struct field:
    ///
    /// ```
    /// use volatile::{ExternallySharedPtr, map_field};
    /// use core::ptr::NonNull;
    ///
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut volatile = unsafe { ExternallySharedPtr::new((&mut value).into()) };
    ///
    /// // construct a volatile write-only pointer to `field_2`
    /// let mut field_2 = map_field!(volatile.field_2).write_only();
    /// field_2.write(14);
    /// // field_2.read(); // compile-time error
    /// ```
    pub fn write_only(self) -> ExternallySharedPtr<'a, T, WriteOnly> {
        unsafe { ExternallySharedPtr::new_restricted(WriteOnly, self.pointer) }
    }
}
