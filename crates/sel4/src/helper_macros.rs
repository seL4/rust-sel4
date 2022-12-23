macro_rules! newtype_methods {
    ($inner:path) => {
        pub const fn from_inner(inner: $inner) -> Self {
            Self(inner)
        }

        pub const fn into_inner(self) -> $inner {
            self.0
        }

        pub const fn inner(&self) -> &$inner {
            &self.0
        }

        pub fn inner_mut(&mut self) -> &mut $inner {
            &mut self.0
        }
    };
}

macro_rules! declare_cap_type {
    ($t:ident) => {
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct $t;

        impl $crate::CapType for $t {
            const NAME: &'static str = stringify!($t);
        }
    };
}

macro_rules! declare_local_cptr_alias {
    ($t:ident) => {
        pub type $t<C = $crate::NoExplicitInvocationContext> =
            $crate::LocalCPtr<$crate::cap_type::$t, C>;
    };
}

macro_rules! declare_fault_newtype {
    ($t:ident, $sys:path) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $t($sys);

        impl $t {
            pub const fn from_inner(inner: $sys) -> Self {
                Self(inner)
            }

            pub const fn inner(&self) -> &$sys {
                &self.0
            }
        }
    };
}

pub(crate) use declare_cap_type;
pub(crate) use declare_fault_newtype;
pub(crate) use declare_local_cptr_alias;
pub(crate) use newtype_methods;
