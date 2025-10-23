//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::mem::MaybeUninit;

mod common;

pub mod serial;
pub mod timer;

pub use common::PeerMisbehaviorError;

#[macro_export]
macro_rules! config {
    ($section:literal, $symbol:ident: $ty:ty) => {{
        use $crate::_private::{ImmutableCell, UncheckedConfig};

        #[allow(non_upper_case_globals)]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = $section)]
        static $symbol: ImmutableCell<UncheckedConfig<$ty>> =
            ImmutableCell::new(UncheckedConfig::uninit());

        $symbol
            .get()
            .check_magic()
            .unwrap_or_else(|| panic!("invalid magic for {:?}", stringify!($symbol)))
    }};
}

// For macros
#[doc(hidden)]
pub mod _private {
    pub use sel4_immutable_cell::ImmutableCell;

    pub use crate::UncheckedConfig;
}

pub unsafe trait Config {
    fn is_magic_valid(&self) -> bool;
}

#[doc(hidden)]
#[repr(transparent)]
pub struct UncheckedConfig<T>(MaybeUninit<T>);

impl<T: Config> UncheckedConfig<T> {
    pub const fn uninit() -> Self {
        Self(MaybeUninit::uninit())
    }

    pub fn check_magic(&self) -> Option<&T> {
        let unchecked = unsafe { self.0.assume_init_ref() };
        if unchecked.is_magic_valid() {
            Some(unchecked)
        } else {
            None
        }
    }
}
