use crate::{
    access::{Access, Copyable, ReadOnly, ReadWrite, WriteOnly},
    volatile_ptr::VolatilePtr,
};
use core::{fmt, marker::PhantomData, ptr::NonNull};

/// Volatile pointer type that respects Rust's aliasing rules.
///
/// This pointer type behaves similar to Rust's reference types:
///
/// - it requires exclusive `&mut self` access for mutability
/// - only read-only types implement [`Clone`] and [`Copy`]
/// - [`Send`] and [`Sync`] are implemented if `T: Sync`
///
/// To perform volatile operations on `VolatileRef` types, use the [`as_ptr`][Self::as_ptr]
/// or [`as_mut_ptr`](Self::as_mut_ptr) methods to create a temporary
/// [`VolatilePtr`][crate::VolatilePtr] instance.
///
/// Since not all volatile resources (e.g. memory mapped device registers) are both readable
/// and writable, this type supports limiting the allowed access types through an optional second
/// generic parameter `A` that can be one of `ReadWrite`, `ReadOnly`, or `WriteOnly`. It defaults
/// to `ReadWrite`, which allows all operations.
///
/// The size of this struct is the same as the size of the contained reference.
#[repr(transparent)]
pub struct VolatileRef<'a, T, A = ReadWrite>
where
    T: ?Sized,
{
    pointer: NonNull<T>,
    reference: PhantomData<&'a T>,
    access: PhantomData<A>,
}

/// Constructor functions.
///
/// These functions construct new `VolatileRef` values. While the `new`
/// function creates a `VolatileRef` instance with unrestricted access, there
/// are also functions for creating read-only or write-only instances.
impl<'a, T> VolatileRef<'a, T>
where
    T: ?Sized,
{
    /// Turns the given pointer into a `VolatileRef`.
    ///
    /// ## Safety
    ///
    /// - The pointer must be properly aligned.
    /// - It must be “dereferenceable” in the sense defined in the [`core::ptr`] documentation.
    /// - The pointer must point to an initialized instance of T.
    /// - You must enforce Rust’s aliasing rules, since the returned lifetime 'a is arbitrarily
    ///   chosen and does not necessarily reflect the actual lifetime of the data. In particular,
    ///   while this `VolatileRef` exists, the memory the pointer points to must not get accessed
    ///   (_read or written_) through any other pointer.
    pub unsafe fn new(pointer: NonNull<T>) -> Self {
        unsafe { VolatileRef::new_restricted(ReadWrite, pointer) }
    }

    /// Turns the given pointer into a read-only `VolatileRef`.
    ///
    /// ## Safety
    ///
    /// - The pointer must be properly aligned.
    /// - It must be “dereferenceable” in the sense defined in the [`core::ptr`] documentation.
    /// - The pointer must point to an initialized instance of T.
    /// - You must enforce Rust’s aliasing rules, since the returned lifetime 'a is arbitrarily
    ///   chosen and does not necessarily reflect the actual lifetime of the data. In particular,
    ///   while this `VolatileRef` exists, the memory the pointer points to _must not get mutated_.
    pub const unsafe fn new_read_only(pointer: NonNull<T>) -> VolatileRef<'a, T, ReadOnly> {
        unsafe { Self::new_restricted(ReadOnly, pointer) }
    }

    /// Turns the given pointer into a `VolatileRef` instance with the given access.
    ///
    /// ## Safety
    ///
    /// - The pointer must be properly aligned.
    /// - It must be “dereferenceable” in the sense defined in the [`core::ptr`] documentation.
    /// - The pointer must point to an initialized instance of T.
    /// - You must enforce Rust’s aliasing rules, since the returned lifetime 'a is arbitrarily
    ///   chosen and does not necessarily reflect the actual lifetime of the data. In particular,
    ///   while this `VolatileRef` exists, the memory the pointer points to _must not get mutated_.
    ///   If the given `access` parameter allows write access, the pointer _must not get read
    ///   either_ while this `VolatileRef` exists.
    pub const unsafe fn new_restricted<A>(access: A, pointer: NonNull<T>) -> VolatileRef<'a, T, A>
    where
        A: Access,
    {
        let _ = access;
        unsafe { Self::new_generic(pointer) }
    }

    /// Creates a `VolatileRef` from the given shared reference.
    ///
    /// **Note:** This function is only intended for testing, not for accessing real volatile
    /// data. The reason is that the `&mut T` argument is considered _dereferenceable_ by Rust,
    /// so the compiler is allowed to insert non-volatile reads. This might lead to undesired
    /// (or even undefined?) behavior when accessing volatile data. So to be safe, only create
    /// raw pointers to volatile data and use the [`Self::new`] constructor instead.
    pub fn from_ref(reference: &'a T) -> VolatileRef<'a, T, ReadOnly>
    where
        T: 'a,
    {
        unsafe { VolatileRef::new_restricted(ReadOnly, reference.into()) }
    }

    /// Creates a `VolatileRef` from the given mutable reference.
    ///
    /// **Note:** This function is only intended for testing, not for accessing real volatile
    /// data. The reason is that the `&mut T` argument is considered _dereferenceable_ by Rust,
    /// so the compiler is allowed to insert non-volatile reads. This might lead to undesired
    /// (or even undefined?) behavior when accessing volatile data. So to be safe, only create
    /// raw pointers to volatile data and use the [`Self::new`] constructor instead.
    pub fn from_mut_ref(reference: &'a mut T) -> Self
    where
        T: 'a,
    {
        unsafe { VolatileRef::new(reference.into()) }
    }

    const unsafe fn new_generic<A>(pointer: NonNull<T>) -> VolatileRef<'a, T, A> {
        VolatileRef {
            pointer,
            reference: PhantomData,
            access: PhantomData,
        }
    }
}

impl<'a, T, A> VolatileRef<'a, T, A>
where
    T: ?Sized,
{
    /// Borrows this `VolatileRef` as a read-only [`VolatilePtr`].
    ///
    /// Use this method to do (partial) volatile reads of the referenced data.
    pub fn as_ptr(&self) -> VolatilePtr<'_, T, A::RestrictShared>
    where
        A: Access,
    {
        unsafe { VolatilePtr::new_restricted(Default::default(), self.pointer) }
    }

    /// Borrows this `VolatileRef` as a mutable [`VolatilePtr`].
    ///
    /// Use this method to do (partial) volatile reads or writes of the referenced data.
    pub fn as_mut_ptr(&mut self) -> VolatilePtr<'_, T, A>
    where
        A: Access,
    {
        unsafe { VolatilePtr::new_restricted(Default::default(), self.pointer) }
    }

    /// Converts this `VolatileRef` into a [`VolatilePtr`] with full access without shortening
    /// the lifetime.
    ///
    /// Use this method when you need a [`VolatilePtr`] instance that lives for the full
    /// lifetime `'a`.
    ///
    /// This method consumes the `VolatileRef`.
    pub fn into_ptr(self) -> VolatilePtr<'a, T, A>
    where
        A: Access,
    {
        unsafe { VolatilePtr::new_restricted(Default::default(), self.pointer) }
    }
}

/// Methods for restricting access.
impl<'a, T> VolatileRef<'a, T, ReadWrite>
where
    T: ?Sized,
{
    /// Restricts access permissions to read-only.
    ///
    /// ## Example
    ///
    /// ```
    /// use volatile::VolatileRef;
    /// use core::ptr::NonNull;
    ///
    /// let mut value: i16 = -4;
    /// let mut volatile = VolatileRef::from_mut_ref(&mut value);
    ///
    /// let read_only = volatile.read_only();
    /// assert_eq!(read_only.as_ptr().read(), -4);
    /// // read_only.as_ptr().write(10); // compile-time error
    /// ```
    pub fn read_only(self) -> VolatileRef<'a, T, ReadOnly> {
        unsafe { VolatileRef::new_restricted(ReadOnly, self.pointer) }
    }

    /// Restricts access permissions to write-only.
    ///
    /// ## Example
    ///
    /// Creating a write-only reference to a struct field:
    ///
    /// ```
    /// use volatile::{VolatileRef};
    /// use core::ptr::NonNull;
    ///
    /// #[derive(Clone, Copy)]
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut volatile = VolatileRef::from_mut_ref(&mut value);
    ///
    /// let write_only = volatile.write_only();
    /// // write_only.as_ptr().read(); // compile-time error
    /// ```
    pub fn write_only(self) -> VolatileRef<'a, T, WriteOnly> {
        unsafe { VolatileRef::new_restricted(WriteOnly, self.pointer) }
    }
}

impl<'a, T, A> Clone for VolatileRef<'a, T, A>
where
    T: ?Sized,
    A: Access + Copyable,
{
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer,
            reference: self.reference,
            access: self.access,
        }
    }
}

impl<'a, T, A> Copy for VolatileRef<'a, T, A>
where
    T: ?Sized,
    A: Access + Copyable,
{
}

unsafe impl<T, A> Send for VolatileRef<'_, T, A> where T: Sync {}
unsafe impl<T, A> Sync for VolatileRef<'_, T, A> where T: Sync {}

impl<T, A> fmt::Debug for VolatileRef<'_, T, A>
where
    T: Copy + fmt::Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VolatileRef")
            .field("pointer", &self.pointer)
            .field("access", &self.access)
            .finish()
    }
}
