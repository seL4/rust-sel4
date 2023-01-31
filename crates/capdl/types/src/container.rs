#![allow(unused_imports)]

use core::borrow::Borrow;
use core::cmp;
use core::fmt;
use core::iter;
use core::marker::PhantomData;
use core::slice;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait Container<'a>: 'a {
    type ContainerType<A: 'a>: ~const Borrow<[A]>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceContainer<'b>(pub PhantomData<&'b ()>);

impl<'a> Container<'a> for SliceContainer<'a> {
    type ContainerType<A: 'a> = &'a [A];
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VecContainer;

#[cfg(feature = "alloc")]
impl<'a> Container<'a> for VecContainer {
    type ContainerType<A: 'a> = Vec<A>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ContainerType<'a, T: Container<'a>, A: 'a>(pub T::ContainerType<A>);

impl<'a, T: Container<'a>, A: 'a> const Borrow<[A]> for ContainerType<'a, T, A> {
    fn borrow(&self) -> &[A] {
        self.0.borrow()
    }
}

impl<'a, T: Container<'a>, A: 'a> ContainerType<'a, T, A> {
    pub const fn as_slice(&self) -> &[A] {
        self.borrow()
    }
}

// TODO should this be an alias instead?
// pub type ContainerType<'a, T, A> = <T as Container<'a>>::ContainerType<A>;
