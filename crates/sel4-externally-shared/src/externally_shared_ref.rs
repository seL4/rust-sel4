use crate::{
    access::{Access, Copyable, ReadOnly, ReadWrite, WriteOnly},
    externally_shared_ptr::ExternallySharedPtr,
};
use core::{fmt, marker::PhantomData, ptr::NonNull};

/// Externally shared pointer type that respects Rust's aliasing rules.
///
/// This pointer type behaves similar to Rust's reference types:
///
/// - it requires exclusive `&mut self` access for mutability
/// - only read-only types implement [`Clone`] and [`Copy`]
/// - [`Send`] and [`Sync`] are implemented if `T: Sync`
///
/// To perform pointer operations on `ExternallySharedRef` types, use the [`as_ptr`][Self::as_ptr]
/// or [`as_mut_ptr`](Self::as_mut_ptr) methods to create a temporary
/// [`ExternallySharedPtr`][crate::ExternallySharedPtr] instance.
///
/// Since not all externallyshared resources (e.g. memory mapped device registers) are both readable
/// and writable, this type supports limiting the allowed access types through an optional second
/// generic parameter `A` that can be one of `ReadWrite`, `ReadOnly`, or `WriteOnly`. It defaults
/// to `ReadWrite`, which allows all operations.
///
/// The size of this struct is the same as the size of the contained reference.
#[repr(transparent)]
pub struct ExternallySharedRef<'a, T, A = ReadWrite>
where
    T: ?Sized,
{
    pointer: NonNull<T>,
    reference: PhantomData<&'a T>,
    access: PhantomData<A>,
}

/// Constructor functions.
///
/// These functions construct new `ExternallySharedRef` values. While the `new`
/// function creates a `ExternallySharedRef` instance with unrestricted access, there
/// are also functions for creating read-only or write-only instances.
impl<'a, T> ExternallySharedRef<'a, T>
where
    T: ?Sized,
{
    /// Turns the given pointer into a `ExternallySharedRef`.
    ///
    /// ## Safety
    ///
    /// - The pointer must be properly aligned.
    /// - It must be “dereferenceable” in the sense defined in the [`core::ptr`] documentation.
    /// - The pointer must point to an initialized instance of T.
    /// - You must enforce Rust’s aliasing rules, since the returned lifetime 'a is arbitrarily
    ///   chosen and does not necessarily reflect the actual lifetime of the data. In particular,
    ///   while this `ExternallySharedRef` exists, the memory the pointer points to must not get accessed
    ///   (_read or written_) through any other pointer.
    pub unsafe fn new(pointer: NonNull<T>) -> Self {
        unsafe { ExternallySharedRef::new_restricted(ReadWrite, pointer) }
    }

    /// Turns the given pointer into a read-only `ExternallySharedRef`.
    ///
    /// ## Safety
    ///
    /// - The pointer must be properly aligned.
    /// - It must be “dereferenceable” in the sense defined in the [`core::ptr`] documentation.
    /// - The pointer must point to an initialized instance of T.
    /// - You must enforce Rust’s aliasing rules, since the returned lifetime 'a is arbitrarily
    ///   chosen and does not necessarily reflect the actual lifetime of the data. In particular,
    ///   while this `ExternallySharedRef` exists, the memory the pointer points to _must not get mutated_.
    pub const unsafe fn new_read_only(pointer: NonNull<T>) -> ExternallySharedRef<'a, T, ReadOnly> {
        unsafe { Self::new_restricted(ReadOnly, pointer) }
    }

    /// Turns the given pointer into a `ExternallySharedRef` instance with the given access.
    ///
    /// ## Safety
    ///
    /// - The pointer must be properly aligned.
    /// - It must be “dereferenceable” in the sense defined in the [`core::ptr`] documentation.
    /// - The pointer must point to an initialized instance of T.
    /// - You must enforce Rust’s aliasing rules, since the returned lifetime 'a is arbitrarily
    ///   chosen and does not necessarily reflect the actual lifetime of the data. In particular,
    ///   while this `ExternallySharedRef` exists, the memory the pointer points to _must not get mutated_.
    ///   If the given `access` parameter allows write access, the pointer _must not get read
    ///   either_ while this `ExternallySharedRef` exists.
    pub const unsafe fn new_restricted<A>(
        access: A,
        pointer: NonNull<T>,
    ) -> ExternallySharedRef<'a, T, A>
    where
        A: Access,
    {
        let _ = access;
        unsafe { Self::new_generic(pointer) }
    }

    /// Creates a `ExternallySharedRef` from the given shared reference.
    ///
    /// **Note:** This function is only intended for testing.
    pub fn from_ref(reference: &'a T) -> ExternallySharedRef<'a, T, ReadOnly>
    where
        T: 'a,
    {
        unsafe { ExternallySharedRef::new_restricted(ReadOnly, reference.into()) }
    }

    /// Creates a `ExternallySharedRef` from the given mutable reference.
    ///
    /// **Note:** This function is only intended for testing.
    pub fn from_mut_ref(reference: &'a mut T) -> Self
    where
        T: 'a,
    {
        unsafe { ExternallySharedRef::new(reference.into()) }
    }

    const unsafe fn new_generic<A>(pointer: NonNull<T>) -> ExternallySharedRef<'a, T, A> {
        ExternallySharedRef {
            pointer,
            reference: PhantomData,
            access: PhantomData,
        }
    }
}

impl<'a, T, A> ExternallySharedRef<'a, T, A>
where
    T: ?Sized,
{
    /// Borrows this `ExternallySharedRef` as a read-only [`ExternallySharedPtr`].
    ///
    /// Use this method to do (partial) reads of the referenced data.
    pub fn as_ptr(&self) -> ExternallySharedPtr<'_, T, A::RestrictShared>
    where
        A: Access,
    {
        unsafe { ExternallySharedPtr::new_restricted(Default::default(), self.pointer) }
    }

    /// Borrows this `ExternallySharedRef` as a mutable [`ExternallySharedPtr`].
    ///
    /// Use this method to do (partial) reads or writes of the referenced data.
    pub fn as_mut_ptr(&mut self) -> ExternallySharedPtr<'_, T, A>
    where
        A: Access,
    {
        unsafe { ExternallySharedPtr::new_restricted(Default::default(), self.pointer) }
    }

    /// Converts this `ExternallySharedRef` into a [`ExternallySharedPtr`] with full access without shortening
    /// the lifetime.
    ///
    /// Use this method when you need a [`ExternallySharedPtr`] instance that lives for the full
    /// lifetime `'a`.
    ///
    /// This method consumes the `ExternallySharedRef`.
    pub fn into_ptr(self) -> ExternallySharedPtr<'a, T, A>
    where
        A: Access,
    {
        unsafe { ExternallySharedPtr::new_restricted(Default::default(), self.pointer) }
    }
}

/// Methods for restricting access.
impl<'a, T> ExternallySharedRef<'a, T, ReadWrite>
where
    T: ?Sized,
{
    /// Restricts access permissions to read-only.
    ///
    /// ## Example
    ///
    /// ```
    /// use sel4_externally_shared::ExternallySharedRef;
    /// use core::ptr::NonNull;
    ///
    /// let mut value: i16 = -4;
    /// let mut shared = ExternallySharedRef::from_mut_ref(&mut value);
    ///
    /// let read_only = shared.read_only();
    /// assert_eq!(read_only.as_ptr().read(), -4);
    /// // read_only.as_ptr().write(10); // compile-time error
    /// ```
    pub fn read_only(self) -> ExternallySharedRef<'a, T, ReadOnly> {
        unsafe { ExternallySharedRef::new_restricted(ReadOnly, self.pointer) }
    }

    /// Restricts access permissions to write-only.
    ///
    /// ## Example
    ///
    /// Creating a write-only reference to a struct field:
    ///
    /// ```
    /// use sel4_externally_shared::{ExternallySharedRef};
    /// use core::ptr::NonNull;
    ///
    /// #[derive(Clone, Copy)]
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut shared = ExternallySharedRef::from_mut_ref(&mut value);
    ///
    /// let write_only = shared.write_only();
    /// // write_only.as_ptr().read(); // compile-time error
    /// ```
    pub fn write_only(self) -> ExternallySharedRef<'a, T, WriteOnly> {
        unsafe { ExternallySharedRef::new_restricted(WriteOnly, self.pointer) }
    }
}

impl<'a, T, A> Clone for ExternallySharedRef<'a, T, A>
where
    T: ?Sized,
    A: Access + Copyable,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T, A> Copy for ExternallySharedRef<'a, T, A>
where
    T: ?Sized,
    A: Access + Copyable,
{
}

unsafe impl<T, A> Send for ExternallySharedRef<'_, T, A> where T: Sync + ?Sized {}
unsafe impl<T, A> Sync for ExternallySharedRef<'_, T, A> where T: Sync + ?Sized {}

impl<T, A> fmt::Debug for ExternallySharedRef<'_, T, A>
where
    T: Copy + fmt::Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExternallySharedRef")
            .field("pointer", &self.pointer)
            .field("access", &self.access)
            .finish()
    }
}
