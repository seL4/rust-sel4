use core::ops::Range;

use sel4_bitfield_ops::{get_bits, set_bits, UnsignedPrimInt};

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Bitfield<T, const N: usize> {
    arr: [T; N],
}

impl<T, const N: usize> Bitfield<T, N>
where
    T: UnsignedPrimInt,
{
    pub fn from_arr(arr: [T; N]) -> Self {
        Self { arr }
    }

    pub fn into_arr(self) -> [T; N] {
        self.arr
    }

    pub fn as_arr(&self) -> &[T; N] {
        &self.arr
    }

    pub fn as_mut_arr(&mut self) -> &mut [T; N] {
        &mut self.arr
    }

    pub fn zeroed() -> Self {
        Self::from_arr([T::zero(); N])
    }

    pub fn get_bits(&self, range: Range<usize>) -> T {
        get_bits(&self.arr, range)
    }

    pub fn set_bits(&mut self, range: Range<usize>, bits: T) {
        set_bits(&mut self.arr, range, bits)
    }
}
