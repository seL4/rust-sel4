use core::{fmt, marker::PhantomData, ptr::NonNull};

use crate::access::ReadWrite;

mod macros;
mod operations;

#[cfg(test)]
mod tests;
#[cfg(feature = "unstable")]
mod unstable;
#[cfg(feature = "very_unstable")]
mod very_unstable;

/// Wraps a pointer to make accesses to the referenced value volatile.
///
/// Allows volatile reads and writes on the referenced value. The referenced value needs to
/// be `Copy` for reading and writing, as volatile reads and writes take and return copies
/// of the value.
///
/// Since not all volatile resources (e.g. memory mapped device registers) are both readable
/// and writable, this type supports limiting the allowed access types through an optional second
/// generic parameter `A` that can be one of `ReadWrite`, `ReadOnly`, or `WriteOnly`. It defaults
/// to `ReadWrite`, which allows all operations.
///
/// The size of this struct is the same as the size of the contained reference.
#[repr(transparent)]
pub struct VolatilePtr<'a, T, A = ReadWrite>
where
    T: ?Sized,
{
    pointer: NonNull<T>,
    reference: PhantomData<&'a T>,
    access: PhantomData<A>,
}

impl<'a, T, A> Copy for VolatilePtr<'a, T, A> where T: ?Sized {}

impl<T, A> Clone for VolatilePtr<'_, T, A>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, A> fmt::Debug for VolatilePtr<'_, T, A>
where
    T: Copy + fmt::Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VolatilePtr")
            .field("pointer", &self.pointer)
            .field("access", &self.access)
            .finish()
    }
}
