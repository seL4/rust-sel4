#![no_std]

use core::marker::PhantomData;
use core::mem;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Range, Shl, Shr};

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
    const NUM_BITS: usize = mem::size_of::<Self>() * 8;

    fn zero() -> Self {
        Default::default()
    }
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

use sealing::{PrimIntSealed, UnsignedPrimIntSealed};

mod sealing {
    pub trait UnsignedPrimIntSealed {}

    pub trait PrimIntSealed {}

    impl<T: UnsignedPrimIntSealed> PrimIntSealed for T {}
}

macro_rules! impl_prim_int {
    ($unsigned:ty, $signed:ty) => {
        impl UnsignedPrimInt for $unsigned {}

        impl PrimInt for $signed {
            type Unsigned = $unsigned;

            fn cast_from_unsigned(val: Self::Unsigned) -> Self {
                val as Self
            }

            fn cast_to_unsigned(val: Self) -> Self::Unsigned {
                val as Self::Unsigned
            }
        }

        impl UnsignedPrimIntSealed for $unsigned {}

        impl PrimIntSealed for $signed {}
    };
}

impl_prim_int!(u8, i8);
impl_prim_int!(u16, i16);
impl_prim_int!(u32, i32);
impl_prim_int!(u64, i64);
impl_prim_int!(u128, i128);
impl_prim_int!(usize, isize);

// // //

trait UnsignedPrimIntExt: UnsignedPrimInt {
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

// // //

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

fn check_range<T: UnsignedPrimInt, U: UnsignedPrimInt>(arr: &[T], range: &Range<usize>) {
    assert!(range.start <= range.end);
    assert!(range.end <= arr.len() * T::NUM_BITS);
    assert!(range.end - range.start <= U::NUM_BITS);
}

fn checked_cast<T: TryInto<U>, U>(val: T) -> U {
    val.try_into().map_err(|_| unreachable!()).unwrap()
}

pub fn set_bits_from_slice<T: UnsignedPrimInt, U: UnsignedPrimInt>(
    dst: &mut [T],
    dst_range: Range<usize>,
    src: &[U],
    src_start: usize,
) where
    T: TryFrom<usize>,
    usize: TryFrom<U>,
{
    let num_bits = dst_range.len();

    assert!(dst_range.start <= dst_range.end);
    assert!(dst_range.end <= dst.len() * T::NUM_BITS);
    assert!(src_start + num_bits <= src.len() * U::NUM_BITS);

    let mut cur_xfer_start = 0;
    while cur_xfer_start < num_bits {
        let cur_xfer_end = num_bits.min(cur_xfer_start + usize::NUM_BITS);
        let cur_xfer_src_range = (src_start + cur_xfer_start)..(src_start + cur_xfer_end);
        let cur_xfer_dst_range =
            (dst_range.start + cur_xfer_start)..(dst_range.start + cur_xfer_end);
        let xfer: usize = get_bits(src, cur_xfer_src_range);
        set_bits(dst, cur_xfer_dst_range, xfer);
        cur_xfer_start = cur_xfer_end;
    }
}

// // //

pub fn get<T: UnsignedPrimInt, U: PrimInt>(src: &[T], src_start_bit: usize) -> U
where
    U::Unsigned: TryFrom<T>,
{
    let src_range = src_start_bit..(src_start_bit + U::Unsigned::NUM_BITS);
    U::cast_from_unsigned(get_bits(src, src_range))
}

pub fn set<T: UnsignedPrimInt, U: PrimInt>(dst: &mut [T], dst_start_bit: usize, src: U)
where
    U::Unsigned: TryInto<T>,
{
    let dst_range = dst_start_bit..(dst_start_bit + U::Unsigned::NUM_BITS);
    set_bits(dst, dst_range, U::cast_to_unsigned(src))
}

// // //

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Bitfield<T, U> {
    inner: T,
    _phantom: PhantomData<U>,
}

impl<T, U> Bitfield<T, U> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: UnsignedPrimInt, const N: usize> Bitfield<[T; N], T> {
    pub fn zeroed() -> Self {
        Self::new([T::zero(); N])
    }
}

impl<T: AsRef<[U]>, U: UnsignedPrimInt> Bitfield<T, U> {
    pub fn bits(&self) -> &[U] {
        self.inner().as_ref()
    }

    pub fn get_bits<V: UnsignedPrimInt + TryFrom<U>>(&self, range: Range<usize>) -> V {
        get_bits(self.bits(), range)
    }

    pub fn get_bits_into_slice<V: UnsignedPrimInt>(
        &self,
        range: Range<usize>,
        dst: &mut [V],
        dst_start: usize,
    ) where
        V: TryFrom<usize>,
        usize: TryFrom<U>,
    {
        let dst_range = dst_start..(dst_start + range.len());
        set_bits_from_slice(dst, dst_range, self.bits(), range.start)
    }

    pub fn get<V: PrimInt>(&self, start_bit: usize) -> V
    where
        V::Unsigned: TryFrom<U>,
    {
        get(self.bits(), start_bit)
    }
}

impl<T: AsMut<[U]>, U: UnsignedPrimInt> Bitfield<T, U> {
    pub fn bits_mut(&mut self) -> &mut [U] {
        self.inner_mut().as_mut()
    }

    pub fn set_bits<V: UnsignedPrimInt + TryInto<U>>(&mut self, range: Range<usize>, src: V) {
        set_bits(self.bits_mut(), range, src)
    }

    pub fn set_bits_from_slice<V: UnsignedPrimInt>(
        &mut self,
        range: Range<usize>,
        src: &[V],
        src_start: usize,
    ) where
        U: TryFrom<usize>,
        usize: TryFrom<V>,
    {
        set_bits_from_slice(self.bits_mut(), range, src, src_start)
    }

    pub fn set<V: PrimInt>(&mut self, start_bit: usize, src: V)
    where
        V::Unsigned: TryInto<U>,
    {
        set(self.bits_mut(), start_bit, src)
    }
}

// // //

#[cfg(test)]
mod test {
    #![allow(unused_imports)]

    extern crate std;

    use std::eprintln;
    use std::fmt;

    use super::*;

    #[test]
    fn zero_gets_zero() {
        assert_eq!(Bitfield::<[u64; 2], _>::zeroed().get_bits::<u64>(50..80), 0);
    }

    fn set_and_get<
        T: UnsignedPrimInt,
        const N: usize,
        U: UnsignedPrimInt + TryInto<T> + TryFrom<T> + fmt::Debug,
    >(
        range: Range<usize>,
        val: U,
    ) {
        let mut arr = Bitfield::<[T; N], _>::zeroed();
        set_bits(arr.inner_mut(), range.clone(), val);
        let observed_val: U = get_bits(arr.inner(), range);
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
            let mut arr = Bitfield::<[u64; 1], u64>::new([init]);
            arr.set_bits::<u64>(0..2, 0b11);
            arr.set_bits::<u64>(60..64, 0b1111);
            arr.set_bits::<u64>(10..11, 0b1);
            assert_eq!(arr.get_bits::<u64>(0..2), 0b11);
            assert_eq!(arr.get_bits::<u64>(60..64), 0b1111);
            assert_eq!(arr.get_bits::<u64>(10..11), 0b1);
        }
    }
}
