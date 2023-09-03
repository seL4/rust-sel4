use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::Range;

use quote::ToTokens;

pub trait Scheme {
    type WordPrimitive: ToTokens + fmt::Debug;

    const PAGE_BITS: usize;
    const LEVEL_BITS: usize;
    const NUM_LEVELS: usize;

    const MIN_LEVEL_FOR_LEAF: usize;

    type LeafDescriptor: SchemeLeafDescriptor<Self::WordPrimitive> + fmt::Debug;

    const EMPTY_DESCRIPTOR: Self::WordPrimitive;
    const SYMBOLIC_BRANCH_DESCRIPTOR_OFFSET: Self::WordPrimitive;

    type Hepers = SchemeHelpers<Self>;
}

pub trait SchemeLeafDescriptor<WordPrimitive> {
    fn from_vaddr(vaddr: u64, level: usize) -> Self;

    fn to_raw(&self) -> WordPrimitive;
}

pub struct SchemeHelpers<T: ?Sized>(PhantomData<T>);

impl<T: Scheme> SchemeHelpers<T> {
    pub const fn word_bits() -> usize {
        mem::size_of::<T::WordPrimitive>() * 8
    }

    pub const fn num_entries_in_table() -> usize {
        1 << T::LEVEL_BITS
    }

    pub const fn vaddr_bits() -> usize {
        T::LEVEL_BITS * T::NUM_LEVELS + T::PAGE_BITS
    }

    pub const fn vaddr_mask() -> u64 {
        (1 << Self::vaddr_bits()) - 1
    }

    pub const fn virt_bounds() -> Range<u64> {
        0..(1 << Self::vaddr_bits())
    }

    pub fn largest_leaf_size_bits() -> usize {
        T::LEVEL_BITS * (T::NUM_LEVELS - T::MIN_LEVEL_FOR_LEAF - 1) + T::PAGE_BITS
    }

    pub(crate) fn leaf_descriptor_from_vaddr_with_check(
        vaddr: u64,
        level: usize,
    ) -> T::LeafDescriptor {
        let num_zero_bits = (T::NUM_LEVELS - level - 1) * T::LEVEL_BITS + T::PAGE_BITS;
        let mask = (1 << num_zero_bits) - 1;
        assert_eq!(vaddr & mask, 0);
        T::LeafDescriptor::from_vaddr(vaddr, level)
    }
}

#[derive(Debug)]
pub enum AArch64 {}

impl Scheme for AArch64 {
    type WordPrimitive = u64;

    const PAGE_BITS: usize = 12;
    const LEVEL_BITS: usize = 9;
    const NUM_LEVELS: usize = 4;

    const MIN_LEVEL_FOR_LEAF: usize = 1;

    type LeafDescriptor = AArch64LeafDescriptor;

    const EMPTY_DESCRIPTOR: Self::WordPrimitive = 0b0;
    const SYMBOLIC_BRANCH_DESCRIPTOR_OFFSET: Self::WordPrimitive = 0b11;
}

#[derive(Debug)]
pub struct AArch64LeafDescriptor(u64);

impl SchemeLeafDescriptor<u64> for AArch64LeafDescriptor {
    fn from_vaddr(vaddr: u64, level: usize) -> Self {
        let mask = if level == 3 { 0b11 } else { 0b01 };
        Self(vaddr | mask)
    }

    fn to_raw(&self) -> u64 {
        self.0
    }
}

impl AArch64LeafDescriptor {
    pub fn set_access_flag(self, value: bool) -> Self {
        Self(self.0 | (u64::from(value) << 10))
    }

    pub fn set_attribute_index(self, index: u64) -> Self {
        assert_eq!(index & !0b111, 0);
        Self(self.0 | (index << 2))
    }

    pub fn set_shareability(self, shareability: u64) -> Self {
        assert_eq!(shareability & !0b11, 0);
        Self(self.0 | (shareability << 8))
    }
}

const RISCV_ENCODE_FOR_LINKING_LEFT_ROTATION: u32 = 2;

#[allow(dead_code)]
const fn riscv32_encode_for_linking(word: u32) -> u32 {
    word.rotate_left(RISCV_ENCODE_FOR_LINKING_LEFT_ROTATION)
}

const fn riscv64_encode_for_linking(word: u64) -> u64 {
    word.rotate_left(RISCV_ENCODE_FOR_LINKING_LEFT_ROTATION)
}

#[derive(Debug)]
pub enum Riscv64Sv39 {}

impl Scheme for Riscv64Sv39 {
    type WordPrimitive = u64;

    const PAGE_BITS: usize = 12;
    const LEVEL_BITS: usize = 9;
    const NUM_LEVELS: usize = 3;

    const MIN_LEVEL_FOR_LEAF: usize = 0;

    type LeafDescriptor = Riscv64Sv39LeafDescriptor;

    const EMPTY_DESCRIPTOR: Self::WordPrimitive = riscv64_encode_for_linking(0b0);
    const SYMBOLIC_BRANCH_DESCRIPTOR_OFFSET: Self::WordPrimitive = riscv64_encode_for_linking(0b1);
}

#[derive(Debug)]
pub struct Riscv64Sv39LeafDescriptor(u64);

impl SchemeLeafDescriptor<u64> for Riscv64Sv39LeafDescriptor {
    fn from_vaddr(vaddr: u64, _level: usize) -> Self {
        Self((vaddr >> 2) | 0b1)
            .set_read(true)
            .set_write(true)
            .set_execute(true)
    }

    fn to_raw(&self) -> u64 {
        riscv64_encode_for_linking(self.0)
    }
}

impl Riscv64Sv39LeafDescriptor {
    pub fn set_read(self, value: bool) -> Self {
        let ix = 1;
        Self((self.0 & !(1 << ix)) | (u64::from(value) << ix))
    }

    pub fn set_write(self, value: bool) -> Self {
        let ix = 2;
        Self((self.0 & !(1 << ix)) | (u64::from(value) << ix))
    }

    pub fn set_execute(self, value: bool) -> Self {
        let ix = 3;
        Self((self.0 & !(1 << ix)) | (u64::from(value) << ix))
    }
}
