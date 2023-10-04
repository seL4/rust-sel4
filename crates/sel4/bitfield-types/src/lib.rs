#![no_std]

use core::mem;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Range, Shl, Shr};

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

pub trait UnsignedPrimInt:
    UnsignedPrimIntSealed
    + Copy
    + Eq
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitAndAssign
    + BitOrAssign
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Default // HACK for generic 0
{
}

trait UnsignedPrimIntExt: UnsignedPrimInt {
    const NUM_BITS: usize = mem::size_of::<Self>() * 8;

    fn zero() -> Self {
        Default::default()
    }

    fn mask(range: Range<usize>) -> Self {
        debug_assert!(range.start <= range.end);
        debug_assert!(range.end <= Self::NUM_BITS);
        let num_bits = range.end - range.start;
        // avoid overflow
        match num_bits {
            0 => Self::zero(),
            _ if num_bits == Self::NUM_BITS => !Self::zero(),
            _ => !(!Self::zero() << num_bits) << range.start,
        }
    }

    fn take(self, num_bits: usize) -> Self {
        self & Self::mask(0..num_bits)
    }
}

impl<T: UnsignedPrimInt> UnsignedPrimIntExt for T {}

impl UnsignedPrimInt for u8 {}
impl UnsignedPrimInt for u16 {}
impl UnsignedPrimInt for u32 {}
impl UnsignedPrimInt for u64 {}
impl UnsignedPrimInt for u128 {}
impl UnsignedPrimInt for usize {}

use unsigned_prim_int_sealing::UnsignedPrimIntSealed;

mod unsigned_prim_int_sealing {
    pub trait UnsignedPrimIntSealed {}

    impl UnsignedPrimIntSealed for u8 {}
    impl UnsignedPrimIntSealed for u16 {}
    impl UnsignedPrimIntSealed for u32 {}
    impl UnsignedPrimIntSealed for u64 {}
    impl UnsignedPrimIntSealed for u128 {}
    impl UnsignedPrimIntSealed for usize {}
}

pub trait PrimInt: PrimIntSealed {
    type Unsigned: UnsignedPrimInt;

    fn cast_from_unsigned(val: Self::Unsigned) -> Self;
    fn cast_to_unsigned(val: Self) -> Self::Unsigned;
}

impl<T> PrimInt for T
where
    T: UnsignedPrimInt + PrimIntSealed,
{
    type Unsigned = Self;

    fn cast_from_unsigned(val: Self::Unsigned) -> Self {
        val
    }

    fn cast_to_unsigned(val: Self) -> Self::Unsigned {
        val
    }
}

macro_rules! impl_prim_int {
    ($maybe_signed:ty, $unsigned:ty) => {
        impl PrimInt for $maybe_signed {
            type Unsigned = $unsigned;

            fn cast_from_unsigned(val: Self::Unsigned) -> Self {
                val as Self
            }

            fn cast_to_unsigned(val: Self) -> Self::Unsigned {
                val as Self::Unsigned
            }
        }
    };
}

impl_prim_int!(i8, u8);
impl_prim_int!(i16, u16);
impl_prim_int!(i32, u32);
impl_prim_int!(i64, u64);
impl_prim_int!(i128, u128);
impl_prim_int!(isize, usize);

use prim_int_sealing::PrimIntSealed;

mod prim_int_sealing {
    use super::UnsignedPrimIntSealed;

    pub trait PrimIntSealed {}

    impl<T: UnsignedPrimIntSealed> PrimIntSealed for T {}

    impl PrimIntSealed for i8 {}
    impl PrimIntSealed for i16 {}
    impl PrimIntSealed for i32 {}
    impl PrimIntSealed for i64 {}
    impl PrimIntSealed for i128 {}
    impl PrimIntSealed for isize {}
}

pub fn get_bits<T: UnsignedPrimInt, U: UnsignedPrimInt + TryFrom<T>>(
    src: &[T],
    src_range: Range<usize>,
) -> U {
    check_range::<T, U>(src, &src_range);

    let num_bits = src_range.end - src_range.start;
    let index_of_first_primitive = src_range.start / T::NUM_BITS;
    let offset_into_first_primitive = src_range.start % T::NUM_BITS;
    let num_bits_from_first_primitive = (T::NUM_BITS - offset_into_first_primitive).min(num_bits);

    let bits_from_first_primitive = (src[index_of_first_primitive] >> offset_into_first_primitive)
        .take(num_bits_from_first_primitive);

    let mut bits = checked_cast::<T, U>(bits_from_first_primitive);
    let mut num_bits_so_far = num_bits_from_first_primitive;
    let mut index_of_cur_primitive = index_of_first_primitive + 1;

    while num_bits_so_far < num_bits {
        let num_bits_from_cur_primitive = (num_bits - num_bits_so_far).min(T::NUM_BITS);
        let bits_from_cur_primitive = src[index_of_cur_primitive].take(num_bits_from_cur_primitive);
        bits |= checked_cast::<T, U>(bits_from_cur_primitive) << num_bits_so_far;
        num_bits_so_far += num_bits_from_cur_primitive;
        index_of_cur_primitive += 1;
    }

    bits
}

pub fn set_bits<T: UnsignedPrimInt, U: UnsignedPrimInt + TryInto<T>>(
    dst: &mut [T],
    dst_range: Range<usize>,
    src: U,
) {
    check_range::<T, U>(dst, &dst_range);

    let num_bits = dst_range.end - dst_range.start;

    assert!(num_bits == U::NUM_BITS || src >> num_bits == U::zero());

    let index_of_first_primitive = dst_range.start / T::NUM_BITS;
    let offset_into_first_primitive = dst_range.start % T::NUM_BITS;
    let num_bits_for_first_primitive = (T::NUM_BITS - offset_into_first_primitive).min(num_bits);
    let bits_for_first_primitive = src.take(num_bits_for_first_primitive);

    dst[index_of_first_primitive] = (dst[index_of_first_primitive]
        & !T::mask(
            offset_into_first_primitive
                ..(offset_into_first_primitive + num_bits_for_first_primitive),
        ))
        | checked_cast(bits_for_first_primitive) << offset_into_first_primitive;

    let mut num_bits_so_far = num_bits_for_first_primitive;
    let mut index_of_cur_primitive = index_of_first_primitive + 1;

    while num_bits_so_far < num_bits {
        let num_bits_for_cur_primitive = (num_bits - num_bits_so_far).min(T::NUM_BITS);
        let bits_for_cur_primitive = (src >> num_bits_so_far).take(num_bits_for_cur_primitive);
        dst[index_of_cur_primitive] = (dst[index_of_cur_primitive]
            & T::mask(num_bits_for_cur_primitive..T::NUM_BITS))
            | checked_cast(bits_for_cur_primitive);
        num_bits_so_far += num_bits_for_cur_primitive;
        index_of_cur_primitive += 1;
    }
}

pub fn get_bits_maybe_signed<T: UnsignedPrimInt, U: PrimInt>(arr: &[T], range: Range<usize>) -> U
where
    U::Unsigned: TryFrom<T>,
{
    U::cast_from_unsigned(get_bits(arr, range))
}

pub fn set_bits_maybe_signed<T: UnsignedPrimInt, U: PrimInt>(
    arr: &mut [T],
    range: Range<usize>,
    bits: U,
) where
    U::Unsigned: TryInto<T>,
{
    set_bits(arr, range, U::cast_to_unsigned(bits))
}

fn check_range<T: UnsignedPrimInt, U: UnsignedPrimInt>(arr: &[T], range: &Range<usize>) {
    assert!(range.start <= range.end);
    assert!(range.end <= arr.len() * T::NUM_BITS);
    assert!(range.end - range.start <= U::NUM_BITS);
}

fn checked_cast<T: TryInto<U>, U>(val: T) -> U {
    val.try_into().map_err(|_| unreachable!()).unwrap()
}

#[cfg(test)]
mod test {
    #![allow(unused_imports)]

    extern crate std;

    use std::eprintln;
    use std::fmt;

    use super::*;

    #[test]
    fn zero_gets_zero() {
        assert_eq!(Bitfield::<u64, 2>::zeroed().get_bits(50..80), 0);
    }

    fn set_and_get<
        T: UnsignedPrimInt,
        const N: usize,
        U: UnsignedPrimInt + TryInto<T> + TryFrom<T> + fmt::Debug,
    >(
        range: Range<usize>,
        val: U,
    ) {
        let mut arr = Bitfield::<T, N>::zeroed();
        set_bits(arr.as_mut_arr(), range.clone(), val);
        let observed_val: U = get_bits(arr.as_arr(), range);
        assert_eq!(observed_val, val);
    }

    #[test]
    fn get_returns_what_was_set() {
        set_and_get::<u8, 3, _>(8..16, !0u8);
        set_and_get::<u8, 3, _>(2..18, !0u32 >> 16);
        set_and_get::<u128, 1, _>(2..18, !0u32 >> 16);
    }

    #[test]
    fn this_works_too() {
        for init in [0, !0] {
            let mut arr = Bitfield::<u64, 1>::from_arr([init]);
            arr.set_bits(0..2, 0b11);
            arr.set_bits(60..64, 0b1111);
            arr.set_bits(10..11, 0b1);
            assert_eq!(arr.get_bits(0..2), 0b11);
            assert_eq!(arr.get_bits(60..64), 0b1111);
            assert_eq!(arr.get_bits(10..11), 0b1);
        }
    }
}
