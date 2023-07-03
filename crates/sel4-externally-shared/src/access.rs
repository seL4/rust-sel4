//! Marker types for limiting access.

/// Private trait that is implemented for the types in this module.
pub trait Access: Copy + Default {
    /// Ensures that this trait cannot be implemented outside of this crate.
    #[doc(hidden)]
    fn _private() -> _Private {
        _Private
    }

    /// Reduced access level to safely share the corresponding value.
    type RestrictShared: Access;
}

/// Helper trait that is implemented by [`ReadWrite`] and [`ReadOnly`].
pub trait Readable: Copy + Default {
    /// Reduced access level to safely share the corresponding value.
    type RestrictShared: Readable + Access;

    /// Ensures that this trait cannot be implemented outside of this crate.
    fn _private() -> _Private {
        _Private
    }
}

/// Helper trait that is implemented by [`ReadWrite`] and [`WriteOnly`].
pub trait Writable: Access {
    /// Ensures that this trait cannot be implemented outside of this crate.
    fn _private() -> _Private {
        _Private
    }
}

/// Implemented for access types that permit copying of `VolatileRef`.
pub trait Copyable {
    /// Ensures that this trait cannot be implemented outside of this crate.
    fn _private() -> _Private {
        _Private
    }
}

impl<T> Access for T
where
    T: Readable + Default + Copy,
{
    type RestrictShared = <T as Readable>::RestrictShared;
}

/// Zero-sized marker type for allowing both read and write access.
#[derive(Debug, Default, Copy, Clone)]
pub struct ReadWrite;
impl Readable for ReadWrite {
    type RestrictShared = ReadOnly;
}
impl Writable for ReadWrite {}

/// Zero-sized marker type for allowing only read access.
#[derive(Debug, Default, Copy, Clone)]
pub struct ReadOnly;
impl Readable for ReadOnly {
    type RestrictShared = ReadOnly;
}
impl Copyable for ReadOnly {}

/// Zero-sized marker type for allowing only write access.
#[derive(Debug, Default, Copy, Clone)]
pub struct WriteOnly;
impl Access for WriteOnly {
    type RestrictShared = NoAccess;
}
impl Writable for WriteOnly {}

/// Zero-sized marker type that grants no access.
#[derive(Debug, Default, Copy, Clone)]
pub struct NoAccess;
impl Access for NoAccess {
    type RestrictShared = NoAccess;
}
impl Copyable for NoAccess {}

#[non_exhaustive]
#[doc(hidden)]
pub struct _Private;
