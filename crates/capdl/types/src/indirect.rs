use core::borrow::Borrow;
use core::fmt;
use core::ops::Deref;

#[cfg(feature = "alloc")]
use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "serde")]
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

#[cfg(not(any(feature = "borrowed-indirect", feature = "alloc")))]
compile_error!("at least on of feature = \"alloc\" or feature = \"borrowed-indirect\" is required");

// NOTE
// Box<[T]> vs Vec<T>: Box<[T]> has no wasted capacity, but is always constructed by first constructing a Vec<T>.
// Does this have a positive or negative impact on memory footprint?
// Minimization but possibly incurring fragmentation.

pub struct Indirect<'a, T: ?Sized>(IndirectImpl<'a, T>);

enum IndirectImpl<'a, T: ?Sized> {
    #[cfg(feature = "borrowed-indirect")]
    Borrowed { borrowed: &'a T },
    #[cfg(feature = "alloc")]
    Owned {
        owned: Box<T>,
        phantom: PhantomData<&'a ()>,
    },
}

impl<'a, T: ?Sized> Indirect<'a, T> {
    #[cfg(feature = "borrowed-indirect")]
    pub const fn from_borrowed(borrowed: &'a T) -> Self {
        Self(IndirectImpl::Borrowed { borrowed })
    }

    #[cfg(feature = "alloc")]
    pub const fn from_owned(owned: Box<T>) -> Self {
        Self(IndirectImpl::Owned {
            owned,
            phantom: PhantomData,
        })
    }

    pub(crate) fn inner(&self) -> &T {
        match self.0 {
            #[cfg(feature = "borrowed-indirect")]
            IndirectImpl::Borrowed { borrowed } => borrowed,
            #[cfg(feature = "alloc")]
            IndirectImpl::Owned { ref owned, .. } => owned.borrow(),
        }
    }

    pub const fn const_inner(&self) -> &T {
        match self.0 {
            #[cfg(feature = "borrowed-indirect")]
            IndirectImpl::Borrowed { borrowed } => borrowed,
            #[cfg(feature = "alloc")]
            IndirectImpl::Owned { .. } => panic!(),
        }
    }
}

impl<T: ?Sized> Deref for Indirect<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner()
    }
}

impl<T: ?Sized> Borrow<T> for Indirect<'_, T> {
    fn borrow(&self) -> &T {
        self.inner()
    }
}

impl<T: Clone> Clone for Indirect<'_, T> {
    fn clone(&self) -> Self {
        Self(match self.0 {
            #[cfg(feature = "borrowed-indirect")]
            IndirectImpl::Borrowed { borrowed } => IndirectImpl::Borrowed { borrowed },
            #[cfg(feature = "alloc")]
            IndirectImpl::Owned { ref owned, phantom } => IndirectImpl::Owned {
                owned: owned.clone(),
                phantom,
            },
        })
    }
}

impl<T: Clone> Clone for Indirect<'_, [T]> {
    fn clone(&self) -> Self {
        Self(match self.0 {
            #[cfg(feature = "borrowed-indirect")]
            IndirectImpl::Borrowed { borrowed } => IndirectImpl::Borrowed { borrowed },
            #[cfg(feature = "alloc")]
            IndirectImpl::Owned { ref owned, phantom } => IndirectImpl::Owned {
                owned: owned.clone(),
                phantom,
            },
        })
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for Indirect<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner().fmt(f)
    }
}

impl<'a, 'b, T: ?Sized, U: ?Sized> PartialEq<Indirect<'b, U>> for Indirect<'a, T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &Indirect<'b, U>) -> bool {
        self.inner().eq(other.inner())
    }
}

impl<T: Eq + ?Sized> Eq for Indirect<'_, T> {}

#[cfg(feature = "serde")]
impl<T: Serialize + ?Sized> Serialize for Indirect<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: ?Sized> Deserialize<'de> for Indirect<'_, T>
where
    Box<T>: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(deserializer).map(Indirect::from_owned)
    }
}
