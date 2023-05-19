#![no_std]

use core::mem;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Range, Shl, Shr};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Bitfield<T, const N: usize> {
    arr: [T; N],
}

impl<T, const N: usize> Bitfield<T, N>
where
    T: BitfieldPrimitive,
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

pub trait BitfieldPrimitive:
    bitfield_primative_sealing::BitfieldPrimitiveSealed
    + Copy
    + Eq
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitOrAssign
    + BitAndAssign
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + From<bool> // HACK for generic 0
{
}

trait BitfieldPrimitiveExt: BitfieldPrimitive {
    const BITS: usize = mem::size_of::<Self>() * 8;

    fn zero() -> Self {
        false.into()
    }
}

impl<T: BitfieldPrimitive> BitfieldPrimitiveExt for T {}

impl BitfieldPrimitive for u128 {}
impl BitfieldPrimitive for u64 {}
impl BitfieldPrimitive for u32 {}
impl BitfieldPrimitive for u16 {}
impl BitfieldPrimitive for u8 {}

mod bitfield_primative_sealing {
    pub trait BitfieldPrimitiveSealed {}

    impl BitfieldPrimitiveSealed for u128 {}
    impl BitfieldPrimitiveSealed for u64 {}
    impl BitfieldPrimitiveSealed for u32 {}
    impl BitfieldPrimitiveSealed for u16 {}
    impl BitfieldPrimitiveSealed for u8 {}
}

//

pub fn get_bits<T: BitfieldPrimitive, const N: usize>(arr: &[T; N], range: Range<usize>) -> T {
    check_range::<T, N>(&range);
    let size = range.end - range.start;
    let index_of_first_primitive = range.start / T::BITS;
    let offset_into_first_primitive = range.start % T::BITS;
    if range_spans_primitive_boundary::<T>(&range) {
        let size_in_first_primitive = T::BITS - offset_into_first_primitive;
        let size_in_second_primitive = size - size_in_first_primitive;
        let bits_from_first_primitive = (arr[index_of_first_primitive]
            >> offset_into_first_primitive)
            & !(!T::zero() << size_in_first_primitive);
        let bits_from_second_primitive =
            arr[index_of_first_primitive + 1] & !(!T::zero() << size_in_second_primitive);
        bits_from_first_primitive | (bits_from_second_primitive << size_in_first_primitive)
    } else {
        let size_in_first_primitive = size;
        (arr[index_of_first_primitive] >> offset_into_first_primitive)
            & !(if size_in_first_primitive == T::BITS {
                T::zero()
            } else {
                !T::zero() << size_in_first_primitive
            })
    }
}

pub fn set_bits<T: BitfieldPrimitive, const N: usize>(
    arr: &mut [T; N],
    range: Range<usize>,
    bits: T,
) {
    check_range::<T, N>(&range);
    let size = range.end - range.start;
    let index_of_first_primitive = range.start / T::BITS;
    let offset_into_first_primitive = range.start % T::BITS;
    assert!(size == T::BITS || bits >> size == T::zero());
    if range_spans_primitive_boundary::<T>(&range) {
        let size_in_first_primitive = T::BITS - offset_into_first_primitive;
        let size_in_second_primitive = size - size_in_first_primitive;
        arr[index_of_first_primitive] &= !(!T::zero() << offset_into_first_primitive);
        arr[index_of_first_primitive] |= bits << offset_into_first_primitive;
        arr[index_of_first_primitive + 1] &= !T::zero() << size_in_second_primitive;
        arr[index_of_first_primitive + 1] |= bits >> size_in_first_primitive;
    } else {
        let size_in_first_primitive = size;
        arr[index_of_first_primitive] &= if size_in_first_primitive == T::BITS {
            T::zero()
        } else {
            !(!(!T::zero() << size_in_first_primitive) << offset_into_first_primitive)
        };
        arr[index_of_first_primitive] |= bits << offset_into_first_primitive;
    };
}

fn check_range<T: BitfieldPrimitive, const N: usize>(range: &Range<usize>) {
    assert!(range.end - range.start <= T::BITS);
    assert!(range.end <= N * T::BITS);
}

fn range_spans_primitive_boundary<T: BitfieldPrimitive>(range: &Range<usize>) -> bool {
    range.start / T::BITS != (range.end - 1) / T::BITS
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn zero() {
        assert_eq!(Bitfield::<u64, 2>::zeroed().get_bits(50..80), 0);
    }
}
