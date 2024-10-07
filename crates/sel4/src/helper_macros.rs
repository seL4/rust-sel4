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
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $t;

        impl $crate::CapType for $t {
            const NAME: &'static str = stringify!($t);
        }
    };
}

macro_rules! declare_cap_type_for_object {
    (
        $(#[$outer:meta])*
        $t:ident { $object_type:ident }
    ) => {
        $crate::declare_cap_type! {
            $(#[$outer])*
            $t
        }

        impl $crate::CapTypeForObject for $t {
            fn object_type() -> $crate::ObjectType {
                $crate::$object_type::$t.into()
            }
        }
    };
}

macro_rules! declare_cap_type_for_object_of_fixed_size {
    (
        $(#[$outer:meta])*
        $t:ident { $object_type:ident, $object_blueprint:ident }
    ) => {
        $crate::declare_cap_type_for_object! {
            $(#[$outer])*
            $t { $object_type }
        }

        impl $crate::CapTypeForObjectOfFixedSize for $t {
            fn object_blueprint() -> $crate::ObjectBlueprint {
                $crate::$object_blueprint::$t.into()
            }
        }
    };
}

macro_rules! declare_cap_type_for_object_of_variable_size {
    (
        $(#[$outer:meta])*
        $t:ident { $object_type:ident, $object_blueprint:ident }
    ) => {
        $crate::declare_cap_type_for_object! {
            $(#[$outer])*
            $t { $object_type }
        }

        impl $crate::CapTypeForObjectOfVariableSize for $t {
            fn object_blueprint(size_bits: usize) -> $crate::ObjectBlueprint {
                ($crate::$object_blueprint::$t { size_bits }).into()
            }
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
    ($t:ident, $sys:ident) => {
        #[doc = "Corresponds to `"]
        #[doc = stringify!($sys)]
        #[doc = "`."]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $t($crate::sys::$sys);

        impl $t {
            $crate::newtype_methods!(pub $crate::sys::$sys);
        }
    };
}

pub(crate) use declare_cap_alias;
pub(crate) use declare_cap_type;
pub(crate) use declare_cap_type_for_object;
pub(crate) use declare_cap_type_for_object_of_fixed_size;
pub(crate) use declare_cap_type_for_object_of_variable_size;
pub(crate) use declare_fault_newtype;
pub(crate) use newtype_methods;
