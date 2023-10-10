pub trait SubKey: Sized + Ord {
    fn min() -> Self;

    fn max() -> Self;

    fn succ(&self) -> Option<Self>;
}

macro_rules! sub_key_impl {
    ($t:ty) => {
        impl SubKey for $t {
            fn min() -> Self {
                <$t>::MIN
            }

            fn max() -> Self {
                <$t>::MAX
            }

            fn succ(&self) -> Option<Self> {
                self.checked_add(1)
            }
        }
    };
}

sub_key_impl!(u8);
sub_key_impl!(u16);
sub_key_impl!(u32);
sub_key_impl!(u64);
sub_key_impl!(u128);
sub_key_impl!(usize);
