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

pub(crate) use newtype_methods;
