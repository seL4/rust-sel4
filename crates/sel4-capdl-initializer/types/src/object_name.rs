//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;
use core::str;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{NamedObject, Object, SelfContained};

pub trait SelfContainedObjectName {
    fn self_contained_object_name(&self) -> Option<&str>;
}

impl SelfContainedObjectName for &str {
    fn self_contained_object_name(&self) -> Option<&str> {
        Some(self)
    }
}

#[cfg(feature = "alloc")]
impl SelfContainedObjectName for String {
    fn self_contained_object_name(&self) -> Option<&str> {
        Some(self)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Unnamed;

impl SelfContainedObjectName for Unnamed {
    fn self_contained_object_name(&self) -> Option<&str> {
        None
    }
}

impl<T: SelfContainedObjectName> SelfContainedObjectName for Option<T> {
    fn self_contained_object_name(&self) -> Option<&str> {
        self.as_ref()
            .and_then(SelfContainedObjectName::self_contained_object_name)
    }
}

pub trait ObjectName {
    type Source: ?Sized;

    fn object_name<'a>(&'a self, means: &'a Self::Source) -> Option<&'a str>;
}

impl<T: SelfContainedObjectName> ObjectName for SelfContained<T> {
    type Source = ();

    fn object_name<'a>(&'a self, _source: &'a Self::Source) -> Option<&'a str> {
        self.inner().self_contained_object_name()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndirectObjectName {
    pub range: Range<usize>,
}

impl ObjectName for IndirectObjectName {
    type Source = [u8];

    fn object_name<'a>(&'a self, source: &'a Self::Source) -> Option<&'a str> {
        Some(str::from_utf8(&source[self.range.clone()]).unwrap())
    }
}

impl<T: ObjectName> ObjectName for Option<T> {
    type Source = T::Source;

    fn object_name<'a>(&'a self, source: &'a Self::Source) -> Option<&'a str> {
        self.as_ref().and_then(|name| name.object_name(source))
    }
}

// // //

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectNamesLevel {
    All,
    JustTcbs,
    None,
}

impl ObjectNamesLevel {
    pub fn apply<'a, N, D, M>(&self, named_obj: &'a NamedObject<N, D, M>) -> Option<&'a N> {
        match self {
            Self::All => Some(&named_obj.name),
            Self::JustTcbs => match &named_obj.object {
                Object::Tcb(_) => Some(&named_obj.name),
                _ => None,
            },
            Self::None => None,
        }
    }
}
