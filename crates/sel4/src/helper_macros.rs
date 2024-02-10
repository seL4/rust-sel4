//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

macro_rules! newtype_methods {
    ($inner_vis:vis $inner:path) => {
        $inner_vis const fn from_inner(inner: $inner) -> Self {
            Self(inner)
        }

        $inner_vis const fn into_inner(self) -> $inner {
            self.0
        }

        $inner_vis const fn inner(&self) -> &$inner {
            &self.0
        }

        $inner_vis fn inner_mut(&mut self) -> &mut $inner {
            &mut self.0
        }
    };
}

macro_rules! declare_cap_type {
    (
        $(#[$outer:meta])*
        $t:ident
    ) => {
        $(#[$outer])*
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct $t;

        impl $crate::CapType for $t {
            const NAME: &'static str = stringify!($t);
        }
    };
}

macro_rules! declare_cap_alias {
    (
        $(#[$outer:meta])*
        $t:ident
    ) => {
        $(#[$outer])*
        pub type $t<C = $crate::NoExplicitInvocationContext> =
            $crate::Cap<$crate::cap_type::$t, C>;
    };
}

macro_rules! declare_fault_newtype {
    ($t:ident, $sys:path) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $t($sys);

        impl $t {
            $crate::newtype_methods!(pub $sys);
        }
    };
}

pub(crate) use declare_cap_alias;
pub(crate) use declare_cap_type;
pub(crate) use declare_fault_newtype;
pub(crate) use newtype_methods;
